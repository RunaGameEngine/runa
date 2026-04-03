use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use egui::{ColorImage, RichText, TextureHandle, Ui, Vec2};
use rfd::FileDialog;

use crate::editor_settings::EditorSettings;

pub struct ContentEntry {
    pub name: String,
    pub relative_path: String,
    pub full_path: PathBuf,
    pub is_dir: bool,
    pub is_rust_file: bool,
}

struct ContentBrowserIcons {
    folder: TextureHandle,
    folder_open: TextureHandle,
    file: TextureHandle,
    rust_file: TextureHandle,
}

struct RenameState {
    path: PathBuf,
    buffer: String,
    request_focus: bool,
}

#[derive(Clone)]
struct ClipboardEntry {
    path: PathBuf,
    cut: bool,
}

pub struct ContentBrowserState {
    project_root: PathBuf,
    current_dir: PathBuf,
    entries: Vec<ContentEntry>,
    sidebar_width: f32,
    icons: Option<ContentBrowserIcons>,
    selected_path: Option<PathBuf>,
    rename_state: Option<RenameState>,
    clipboard: Option<ClipboardEntry>,
    pending_open_dir: Option<PathBuf>,
    last_message: Option<String>,
}

impl ContentBrowserState {
    pub fn new(project_root: PathBuf) -> Self {
        let entries = collect_directory_entries(&project_root, &project_root, false);
        Self {
            current_dir: project_root.clone(),
            project_root,
            entries,
            sidebar_width: 220.0,
            icons: None,
            selected_path: None,
            rename_state: None,
            clipboard: None,
            pending_open_dir: None,
            last_message: None,
        }
    }

    pub fn open_dir(&mut self, dir: PathBuf, settings: &EditorSettings) {
        self.current_dir = dir;
        self.entries = collect_directory_entries(
            &self.project_root,
            &self.current_dir,
            settings.show_hidden_files,
        );
        self.selected_path = None;
        self.rename_state = None;
    }

    pub fn refresh(&mut self, settings: &EditorSettings) {
        self.entries = collect_directory_entries(
            &self.project_root,
            &self.current_dir,
            settings.show_hidden_files,
        );
    }

    pub fn set_project_root(&mut self, project_root: PathBuf, settings: &EditorSettings) {
        self.project_root = project_root.clone();
        self.current_dir = project_root;
        self.selected_path = None;
        self.rename_state = None;
        self.clipboard = None;
        self.refresh(settings);
    }

    pub fn take_message(&mut self) -> Option<String> {
        self.last_message.take()
    }

    pub fn current_dir_display(&self) -> String {
        self.current_dir.display().to_string()
    }

    pub fn ui(&mut self, ui: &mut Ui, settings: &EditorSettings) {
        self.ensure_icons(ui.ctx());
        self.handle_shortcuts(ui);

        let available = ui.available_size();
        ui.allocate_ui_with_layout(available, egui::Layout::top_down(egui::Align::Min), |ui| {
            let handle_width = 8.0;
            let min_sidebar = 180.0;
            let max_sidebar = (available.x - 180.0).max(min_sidebar);
            self.sidebar_width = self.sidebar_width.clamp(min_sidebar, max_sidebar);

            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(self.sidebar_width, available.y),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.label(RichText::new("Folders").strong());
                        ui.separator();
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .id_salt("folder_tree_scroll")
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                let root = self.project_root.clone();
                                let current = self.current_dir.clone();
                                let mut next_dir = None;
                                folder_tree_ui(
                                    ui,
                                    &root,
                                    &current,
                                    &mut next_dir,
                                    0,
                                    5,
                                    settings.show_hidden_files,
                                );
                                if let Some(dir) = next_dir {
                                    self.open_dir(dir, settings);
                                }
                            });
                    },
                );

                let (handle_rect, handle_response) = ui.allocate_exact_size(
                    egui::vec2(handle_width, available.y),
                    egui::Sense::click_and_drag(),
                );
                if handle_response.dragged() {
                    self.sidebar_width = (self.sidebar_width + handle_response.drag_delta().x)
                        .clamp(min_sidebar, max_sidebar);
                }
                ui.painter().rect_filled(
                    handle_rect.shrink2(egui::vec2(2.0, 4.0)),
                    3.0,
                    ui.visuals().widgets.inactive.bg_fill,
                );

                let right_width = ui.available_width().max(120.0);
                ui.allocate_ui_with_layout(
                    egui::vec2(right_width, available.y),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.label(RichText::new("Assets").strong());
                        ui.separator();
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .id_salt("content_grid_scroll")
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                self.content_grid_ui(ui, settings);
                            });
                        let blank_response = ui.allocate_response(
                            egui::vec2(ui.available_width(), ui.available_height().max(24.0)),
                            egui::Sense::click(),
                        );
                        if blank_response.clicked() {
                            self.selected_path = None;
                        }
                        blank_response.context_menu(|ui| {
                            self.content_area_context_menu(ui, settings);
                        });
                    },
                );
            });
        });

        if let Some(dir) = self.pending_open_dir.take() {
            self.open_dir(dir, settings);
        }
    }

    fn handle_shortcuts(&mut self, ui: &Ui) {
        let wants_rename = ui.ctx().input(|i| i.key_pressed(egui::Key::F2));
        if wants_rename && self.rename_state.is_none() {
            if let Some(path) = self.selected_path.clone() {
                self.start_rename(path);
            }
        }
    }

    fn content_grid_ui(&mut self, ui: &mut Ui, settings: &EditorSettings) {
        let cell_width = (settings.content_icon_size + 56.0).max(110.0);
        let cell_height = (settings.content_icon_size + 64.0).max(110.0);
        let columns = ((ui.available_width() / cell_width).floor() as usize).max(1);
        let entries_snapshot: Vec<ContentEntry> = self.entries.iter().map(clone_entry).collect();

        for row in entries_snapshot.chunks(columns) {
            ui.horizontal(|ui| {
                for entry in row {
                    ui.push_id(&entry.relative_path, |ui| {
                        self.draw_content_entry(ui, entry, cell_width, cell_height, settings);
                    });
                }
            });
            ui.add_space(6.0);
        }
    }

    fn draw_content_entry(
        &mut self,
        ui: &mut Ui,
        entry: &ContentEntry,
        cell_width: f32,
        cell_height: f32,
        settings: &EditorSettings,
    ) {
        let selected = self.selected_path.as_ref() == Some(&entry.full_path);
        let is_cut = self
            .clipboard
            .as_ref()
            .map(|clipboard| clipboard.cut && clipboard.path == entry.full_path)
            .unwrap_or(false);
        let is_renaming = self
            .rename_state
            .as_ref()
            .map(|state| state.path == entry.full_path)
            .unwrap_or(false);

        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(cell_width, cell_height), egui::Sense::click());
        let visuals = ui.visuals();
        let fill = if selected {
            visuals.selection.bg_fill
        } else if response.hovered() {
            visuals.widgets.hovered.bg_fill
        } else {
            visuals.widgets.inactive.bg_fill
        };
        let fill = if is_cut {
            fill.gamma_multiply(0.45)
        } else {
            fill
        };
        ui.painter().rect_filled(rect, 8.0, fill);

        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect.shrink2(egui::vec2(6.0, 6.0)))
                .layout(egui::Layout::top_down(egui::Align::Center)),
        );
        let content_tint = if is_cut {
            egui::Color32::from_white_alpha(110)
        } else {
            egui::Color32::WHITE
        };

        let icon = self.icon_for(entry);
        child.add(
            egui::Image::new(icon)
                .fit_to_exact_size(Vec2::splat(settings.content_icon_size))
                .tint(content_tint)
                .sense(egui::Sense::hover()),
        );
        child.add_space(8.0);

        if is_renaming {
            if let Some(rename_state) = self.rename_state.as_mut() {
                let text_edit = egui::TextEdit::singleline(&mut rename_state.buffer)
                    .desired_width(cell_width - 18.0);
                let edit_response = child.add(text_edit);
                if rename_state.request_focus {
                    edit_response.request_focus();
                    rename_state.request_focus = false;
                }

                let confirm = edit_response.lost_focus()
                    && child.ctx().input(|i| i.key_pressed(egui::Key::Enter));
                let cancel = child.ctx().input(|i| i.key_pressed(egui::Key::Escape));

                if confirm {
                    self.commit_rename(settings);
                } else if cancel {
                    self.rename_state = None;
                }
            }
        } else {
            child.add(
                egui::Label::new(
                    RichText::new(&entry.name)
                        .color(content_tint)
                        .text_style(egui::TextStyle::Small)
                        .strong(),
                )
                .wrap(),
            );
        }

        if response.clicked() {
            self.selected_path = Some(entry.full_path.clone());
        }
        if response.double_clicked() {
            self.selected_path = Some(entry.full_path.clone());
            if entry.is_dir {
                self.pending_open_dir = Some(entry.full_path.clone());
            } else {
                self.edit_file(entry, settings);
            }
        }

        let entry_clone = clone_entry(entry);
        response.context_menu(|ui| {
            if !entry_clone.is_dir && ui.button("Edit").clicked() {
                self.selected_path = Some(entry_clone.full_path.clone());
                self.edit_file(&entry_clone, settings);
                ui.close();
            }
            if ui.button("Copy").clicked() {
                self.copy_entry(entry_clone.full_path.clone(), false);
                ui.close();
            }
            if ui.button("Cut").clicked() {
                self.copy_entry(entry_clone.full_path.clone(), true);
                ui.close();
            }
            if ui
                .add_enabled(self.clipboard.is_some(), egui::Button::new("Paste"))
                .clicked()
            {
                self.paste_into(entry_clone.full_path.clone(), settings);
                ui.close();
            }
            if ui.button("Rename").clicked() {
                self.selected_path = Some(entry_clone.full_path.clone());
                self.start_rename(entry_clone.full_path.clone());
                ui.close();
            }
            if ui.button("Delete").clicked() {
                self.delete_entry(entry_clone.full_path.clone(), settings);
                ui.close();
            }
        });
    }

    fn content_area_context_menu(&mut self, ui: &mut Ui, settings: &EditorSettings) {
        if ui.button("New Empty Rust File").clicked() {
            self.create_new_rust_file(false, settings);
            ui.close();
            return;
        }
        if ui.button("New Object Script Template").clicked() {
            self.create_new_rust_file(true, settings);
            ui.close();
            return;
        }
        if ui.button("Import Assets...").clicked() {
            self.import_assets(settings);
            ui.close();
            return;
        }
        ui.separator();
        if ui
            .add_enabled(self.clipboard.is_some(), egui::Button::new("Paste"))
            .clicked()
        {
            self.paste_into(self.current_dir.clone(), settings);
            ui.close();
        }
    }

    fn start_rename(&mut self, path: PathBuf) {
        let buffer = path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default();
        self.rename_state = Some(RenameState {
            path,
            buffer,
            request_focus: true,
        });
    }

    fn commit_rename(&mut self, settings: &EditorSettings) {
        let Some(rename_state) = self.rename_state.take() else {
            return;
        };

        let trimmed = rename_state.buffer.trim();
        if trimmed.is_empty() {
            self.last_message = Some("Rename cancelled: empty name.".to_string());
            return;
        }

        let Some(parent) = rename_state.path.parent() else {
            self.last_message = Some("Rename failed: invalid parent path.".to_string());
            return;
        };

        let new_path = parent.join(trimmed);
        if new_path == rename_state.path {
            return;
        }

        match fs::rename(&rename_state.path, &new_path) {
            Ok(()) => {
                self.last_message = Some(format!("Renamed to {trimmed}"));
                self.selected_path = Some(new_path.clone());
                if self.current_dir == rename_state.path {
                    self.current_dir = new_path.clone();
                }
                self.entries = collect_directory_entries(
                    &self.project_root,
                    &self.current_dir,
                    settings.show_hidden_files,
                );
            }
            Err(error) => {
                self.last_message = Some(format!("Rename failed: {error}"));
            }
        }
    }

    fn copy_entry(&mut self, path: PathBuf, cut: bool) {
        self.selected_path = Some(path.clone());
        self.clipboard = Some(ClipboardEntry { path, cut });
        self.last_message = Some(if cut {
            "Marked item for cut.".to_string()
        } else {
            "Copied item.".to_string()
        });
    }

    fn paste_into(&mut self, target: PathBuf, settings: &EditorSettings) {
        let Some(clipboard) = self.clipboard.clone() else {
            return;
        };

        let destination_dir = if target.is_dir() {
            target
        } else {
            target
                .parent()
                .map(PathBuf::from)
                .unwrap_or_else(|| self.current_dir.clone())
        };
        let Some(file_name) = clipboard.path.file_name() else {
            self.last_message = Some("Paste failed: invalid source path.".to_string());
            return;
        };
        let destination = destination_dir.join(file_name);

        if destination == clipboard.path {
            self.last_message =
                Some("Paste skipped: source and destination are the same.".to_string());
            return;
        }
        if destination.exists() {
            self.last_message = Some("Paste failed: destination already exists.".to_string());
            return;
        }

        let result = if clipboard.cut {
            fs::rename(&clipboard.path, &destination)
        } else if clipboard.path.is_dir() {
            copy_directory_recursive(&clipboard.path, &destination)
        } else {
            fs::copy(&clipboard.path, &destination).map(|_| ())
        };

        match result {
            Ok(()) => {
                if clipboard.cut {
                    self.clipboard = None;
                }
                self.selected_path = Some(destination);
                self.refresh(settings);
                self.last_message = Some("Paste completed.".to_string());
            }
            Err(error) => {
                self.last_message = Some(format!("Paste failed: {error}"));
            }
        }
    }

    fn delete_entry(&mut self, path: PathBuf, settings: &EditorSettings) {
        let result = if path.is_dir() {
            fs::remove_dir_all(&path)
        } else {
            fs::remove_file(&path)
        };

        match result {
            Ok(()) => {
                if self.selected_path.as_ref() == Some(&path) {
                    self.selected_path = None;
                }
                if self
                    .clipboard
                    .as_ref()
                    .map(|clipboard| clipboard.path == path)
                    .unwrap_or(false)
                {
                    self.clipboard = None;
                }
                self.refresh(settings);
                self.last_message = Some("Deleted item.".to_string());
            }
            Err(error) => {
                self.last_message = Some(format!("Delete failed: {error}"));
            }
        }
    }

    fn create_new_rust_file(&mut self, with_template: bool, settings: &EditorSettings) {
        let base_name = if with_template {
            "NewObject"
        } else {
            "NewFile"
        };
        let path = unique_rust_file_path(&self.current_dir, base_name);
        let content = if with_template {
            object_script_template(
                path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("NewObject"),
            )
        } else {
            String::new()
        };

        match fs::write(&path, content) {
            Ok(()) => {
                self.refresh(settings);
                self.selected_path = Some(path.clone());
                self.last_message = Some(format!("Created {}.", path.display()));
            }
            Err(error) => {
                self.last_message = Some(format!("Create file failed: {error}"));
            }
        }
    }

    fn import_assets(&mut self, settings: &EditorSettings) {
        let Some(paths) = FileDialog::new().pick_files() else {
            return;
        };

        let mut imported = 0usize;
        for source in paths {
            let Some(name) = source.file_name() else {
                continue;
            };
            let destination = self.current_dir.join(name);
            if destination.exists() {
                continue;
            }
            if fs::copy(&source, &destination).is_ok() {
                imported += 1;
            }
        }

        self.refresh(settings);
        self.last_message = Some(format!("Imported {imported} asset(s)."));
    }

    fn edit_file(&mut self, entry: &ContentEntry, settings: &EditorSettings) {
        if entry.is_dir {
            return;
        }

        let executable = settings.external_editor_executable.trim();
        if executable.is_empty() {
            self.last_message = Some("External editor is not configured.".to_string());
            return;
        }

        let file = entry.full_path.to_string_lossy().to_string();
        let args: Vec<String> = settings
            .external_editor_args
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|line| line.replace("{file}", &file))
            .collect();

        match Command::new(executable).args(args).spawn() {
            Ok(_) => {
                self.last_message = Some(format!("Opened {} in external editor.", entry.name));
            }
            Err(error) => {
                self.last_message = Some(format!("Edit failed: {error}"));
            }
        }
    }

    fn ensure_icons(&mut self, ctx: &egui::Context) {
        if self.icons.is_some() {
            return;
        }

        self.icons = Some(ContentBrowserIcons {
            folder: load_png_texture(
                ctx,
                "content_browser_folder_icon",
                include_bytes!("../assets/icons/folder.png"),
            ),
            folder_open: load_png_texture(
                ctx,
                "content_browser_folder_open_icon",
                include_bytes!("../assets/icons/folder-open.png"),
            ),
            file: load_png_texture(
                ctx,
                "content_browser_file_icon",
                include_bytes!("../assets/icons/file.png"),
            ),
            rust_file: load_png_texture(
                ctx,
                "content_browser_rust_file_icon",
                include_bytes!("../assets/icons/rust-file.png"),
            ),
        });
    }

    fn icon_for(&self, entry: &ContentEntry) -> &TextureHandle {
        let icons = self.icons.as_ref().expect("icons must be initialized");
        if entry.is_dir {
            if self.current_dir == entry.full_path
                || self.selected_path.as_ref() == Some(&entry.full_path)
            {
                &icons.folder_open
            } else {
                &icons.folder
            }
        } else if entry.is_rust_file {
            &icons.rust_file
        } else {
            &icons.file
        }
    }
}

fn folder_tree_ui(
    ui: &mut Ui,
    root: &Path,
    current_dir: &Path,
    next_dir: &mut Option<PathBuf>,
    depth: usize,
    max_depth: usize,
    show_hidden_files: bool,
) {
    let Ok(read_dir) = fs::read_dir(root) else {
        return;
    };

    let mut directories: Vec<PathBuf> = read_dir
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter(|path| !is_ignored_path(path, show_hidden_files))
        .collect();
    directories.sort();

    for directory in directories {
        let name = directory
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| directory.display().to_string());

        ui.push_id(directory.display().to_string(), |ui| {
            ui.horizontal(|ui| {
                ui.add_space((depth as f32) * 12.0);
                let selected = current_dir == directory.as_path();
                if ui.selectable_label(selected, name).clicked() {
                    *next_dir = Some(directory.clone());
                }
            });
        });

        if depth < max_depth {
            folder_tree_ui(
                ui,
                &directory,
                current_dir,
                next_dir,
                depth + 1,
                max_depth,
                show_hidden_files,
            );
        }
    }
}

fn collect_directory_entries(
    project_root: &Path,
    current_dir: &Path,
    show_hidden_files: bool,
) -> Vec<ContentEntry> {
    let mut entries = Vec::new();
    let Ok(read_dir) = fs::read_dir(current_dir) else {
        return entries;
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        if is_ignored_path(&path, show_hidden_files) {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        let relative_path = path
            .strip_prefix(project_root)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| path.display().to_string());
        let is_dir = path.is_dir();
        let is_rust_file = path.extension().and_then(|ext| ext.to_str()) == Some("rs");

        entries.push(ContentEntry {
            name,
            relative_path,
            full_path: path,
            is_dir,
            is_rust_file,
        });
    }

    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });
    entries
}

fn is_ignored_path(path: &Path, show_hidden_files: bool) -> bool {
    let file_name = path.file_name().and_then(|name| name.to_str());
    if matches!(file_name, Some(".git" | "target" | ".obsidian" | ".proj")) {
        return true;
    }
    if !show_hidden_files {
        if let Some(name) = file_name {
            return name.starts_with('.');
        }
    }
    false
}

fn load_png_texture(ctx: &egui::Context, name: &str, bytes: &[u8]) -> TextureHandle {
    let image = image::load_from_memory(bytes)
        .expect("failed to decode png")
        .to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let pixels = image.into_raw();
    let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);
    ctx.load_texture(name, color_image, egui::TextureOptions::LINEAR)
}

fn clone_entry(entry: &ContentEntry) -> ContentEntry {
    ContentEntry {
        name: entry.name.clone(),
        relative_path: entry.relative_path.clone(),
        full_path: entry.full_path.clone(),
        is_dir: entry.is_dir,
        is_rust_file: entry.is_rust_file,
    }
}

fn copy_directory_recursive(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_directory_recursive(&source_path, &destination_path)?;
        } else {
            fs::copy(&source_path, &destination_path)?;
        }
    }
    Ok(())
}

fn unique_rust_file_path(directory: &Path, base_name: &str) -> PathBuf {
    let mut index = 0usize;
    loop {
        let file_name = if index == 0 {
            format!("{base_name}.rs")
        } else {
            format!("{base_name}{index}.rs")
        };
        let path = directory.join(file_name);
        if !path.exists() {
            return path;
        }
        index += 1;
    }
}

fn object_script_template(type_name: &str) -> String {
    format!(
        "use runa_engine::{{\n    runa_core::{{\n        components::{{SpriteRenderer, Transform}},\n        ocs::{{Object, Script}},\n        Vec3,\n    }},\n}};\n\npub struct {type_name} {{\n    speed: f32,\n    direction: Vec3,\n}}\n\nimpl {type_name} {{\n    pub fn new() -> Self {{\n        Self {{\n            speed: 1.0,\n            direction: Vec3::ZERO,\n        }}\n    }}\n}}\n\nimpl Script for {type_name} {{\n    fn construct(&self, object: &mut Object) {{\n        // Register the components this object needs.\n        object\n            .add_component(Transform::default())\n            .add_component(SpriteRenderer::default());\n\n        // Register this object in your place object registry so it appears in the editor.\n    }}\n\n    fn update(&mut self, object: &mut Object, dt: f32) {{\n        let _ = (&self.speed, &self.direction, object, dt);\n    }}\n}}\n"
    )
}
