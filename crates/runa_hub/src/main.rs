use eframe::{egui, App};
use egui::{IconData, TextureHandle, ViewportBuilder};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

mod icon_loader;

const TEMPLATES_DIR: &str = "../templates/default";
const HUB_CONFIG_PATH: &str = ".runa_hub/recent_projects.ron";

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct ProjectEntry {
    name: String,
    path: PathBuf,
}

#[derive(PartialEq, Clone)]
enum HubTab {
    Projects,
    Versions,
    Settings,
}

struct RunaHub {
    active_tab: HubTab,
    recent_projects: Vec<ProjectEntry>,
    new_project_name: String,
    new_project_path: String,

    icon_projects: Option<Arc<TextureHandle>>,
    icon_versions: Option<Arc<TextureHandle>>,
    icon_settings: Option<Arc<TextureHandle>>,
}

impl Default for RunaHub {
    fn default() -> Self {
        // Загружаем недавние проекты
        let recent_projects = load_recent_projects();
        Self {
            recent_projects,
            new_project_name: "MyGame".to_string(),
            new_project_path: dirs::document_dir()
                .unwrap_or(std::env::current_dir().unwrap())
                .join("MyGame")
                .to_string_lossy()
                .to_string(),
            active_tab: HubTab::Projects,
            icon_projects: None,
            icon_versions: None,
            icon_settings: None,
        }
    }
}

impl App for RunaHub {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.icon_projects.is_none() {
            self.load_icons(ctx);
        }

        egui::SidePanel::left("hub_nav")
            .min_width(200.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Runa Hub");
                ui.separator();

                if nav_button(
                    ui,
                    self.icon_projects.as_deref(),
                    "Projects",
                    self.active_tab == HubTab::Projects,
                )
                .clicked()
                {
                    self.active_tab = HubTab::Projects;
                }

                if nav_button(
                    ui,
                    self.icon_versions.as_deref(),
                    "Versions",
                    self.active_tab == HubTab::Versions,
                )
                .clicked()
                {
                    self.active_tab = HubTab::Versions;
                }

                if nav_button(
                    ui,
                    self.icon_settings.as_deref(),
                    "Settings",
                    self.active_tab == HubTab::Settings,
                )
                .clicked()
                {
                    self.active_tab = HubTab::Settings;
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| match self.active_tab {
            HubTab::Projects => self.ui_projects(ui),
            HubTab::Versions => self.ui_versions(ui),
            HubTab::Settings => self.ui_settings(ui),
        });
    }
}

impl RunaHub {
    fn load_icons(&mut self, ctx: &egui::Context) {
        self.icon_projects =
            crate::icon_loader::load_icon(ctx, include_bytes!("../assets/projects.png"));
        self.icon_versions =
            crate::icon_loader::load_icon(ctx, include_bytes!("../assets/version.png"));
        self.icon_settings =
            crate::icon_loader::load_icon(ctx, include_bytes!("../assets/settings.png"));
    }

    fn create_project(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let project_path = PathBuf::from(&self.new_project_path);
        let project_name = &self.new_project_name;

        // Создаём папку проекта
        fs::create_dir_all(&project_path)?;

        // Копируем шаблон
        copy_template(TEMPLATES_DIR, &project_path, project_name)?;

        // Добавляем в недавние
        self.recent_projects.push(ProjectEntry {
            name: project_name.clone(),
            path: project_path.clone(),
        });
        save_recent_projects(&self.recent_projects)?;

        // Открываем в редакторе
        self.open_project_in_editor(&project_path);

        Ok(())
    }

    fn open_project_in_editor(&self, project_path: &PathBuf) {
        // Запускаем: cargo run -p runa_editor --manifest-path <project>/Cargo.toml
        let status = Command::new("cargo")
            .args(&["run", "--bin", "runa_editor"])
            .current_dir(project_path)
            .spawn();

        if let Err(e) = status {
            eprintln!("Failed to start editor: {}", e);
        }
    }

    fn run_project(&self, project_path: &PathBuf) {
        let status = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                project_path.file_name().unwrap().to_str().unwrap(),
            ])
            .current_dir(project_path)
            .spawn();

        if let Err(e) = status {
            eprintln!("Failed to run project: {}", e);
        }
    }

    // UI
    fn ui_projects(&mut self, ui: &mut egui::Ui) {
        ui.heading("Projects");

        ui.separator();
        ui.label("Create New Project");
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut self.new_project_name);
        });
        ui.horizontal(|ui| {
            ui.label("Path:");
            if ui.button("📁").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.new_project_path = path
                        .join(&self.new_project_name)
                        .to_string_lossy()
                        .to_string();
                }
            }
            ui.text_edit_singleline(&mut self.new_project_path);
        });

        if ui.button("Create Project").clicked() {
            if let Err(e) = self.create_project() {
                eprintln!("Failed to create project: {}", e);
            }
        }

        ui.separator();
        ui.label("Recent Projects");
        for project in &self.recent_projects {
            ui.horizontal(|ui| {
                if ui.button("📁").clicked() {
                    open_in_file_manager(&project.path);
                }
                if ui.button(&project.name).clicked() {
                    self.open_project_in_editor(&project.path);
                }
                if ui.button("▶").clicked() {
                    self.run_project(&project.path);
                }
            });
        }
    }

    fn ui_versions(&mut self, ui: &mut egui::Ui) {
        ui.heading("Engine Versions");
        ui.label("Manage installed versions of Rune Engine.");

        egui::Grid::new("versions_grid")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("rune-engine v0.1.0");
                if ui.button("Set as default").clicked() {
                    // TODO
                }
                ui.end_row();

                ui.label("rune-engine main (dev)");
                if ui.button("Update").clicked() {
                    // TODO
                }
                ui.end_row();
            });
    }

    fn ui_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");

        ui.label("Editor Theme:");
        egui::ComboBox::from_label("Theme")
            .selected_text("Dark") // пока фиксировано
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut (), (), "Light");
                ui.selectable_value(&mut (), (), "Dark");
            });

        ui.label("Default Project Path:");
        ui.text_edit_singleline(&mut self.new_project_path);

        if ui.button("Save Settings").clicked() {
            // TODO: сохранить в ~/.runa_hub/settings.ron
            ui.label("Saved!");
        }
    }
}

fn copy_template(
    template_dir: &str,
    dest_dir: &PathBuf,
    project_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(template_dir)? {
        let entry = entry?;
        let src_path = entry.path();
        let rel_path = src_path.strip_prefix(template_dir)?;
        let dest_path = dest_dir.join(rel_path);

        if src_path.is_dir() {
            fs::create_dir_all(&dest_path)?;
        } else {
            fs::create_dir_all(dest_path.parent().unwrap())?;
            let content = fs::read_to_string(&src_path)?;
            let new_content = content.replace("{{project_name}}", project_name);
            fs::write(&dest_path, new_content)?;
        }
    }
    Ok(())
}

fn load_recent_projects() -> Vec<ProjectEntry> {
    let config_path = dirs::home_dir().unwrap().join(HUB_CONFIG_PATH);
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(projects) = ron::from_str(&content) {
                return projects;
            }
        }
    }
    Vec::new()
}

fn save_recent_projects(projects: &[ProjectEntry]) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = dirs::home_dir().unwrap().join(HUB_CONFIG_PATH);
    fs::create_dir_all(config_path.parent().unwrap())?;
    let content = ron::ser::to_string_pretty(projects, ron::ser::PrettyConfig::default())?;
    fs::write(config_path, content)?;
    Ok(())
}

fn open_in_file_manager(path: &PathBuf) {
    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer")
        .args(&[path])
        .spawn()
        .ok();
    #[cfg(target_os = "macos")]
    std::process::Command::new("open")
        .args(&[path])
        .spawn()
        .ok();
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open")
        .args(&[path])
        .spawn()
        .ok();
}

fn main() -> eframe::Result {
    let icon_data = crate::icon_loader::load_app_icon();

    let viewport = ViewportBuilder::default()
        .with_title("Runa Hub")
        .with_inner_size([1280.0, 720.0])
        .with_fullscreen(false);

    let viewport = if let Some(icon) = icon_data {
        viewport.with_icon(icon)
    } else {
        viewport
    };

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Runa Hub",
        options,
        Box::new(|cc| {
            setup_style(&cc.egui_ctx);
            Ok(Box::new(RunaHub::default()))
        }),
    )
}

fn setup_style(ctx: &egui::Context) {
    use egui::{Color32, Style, Visuals};

    let mut style = Style::default();

    style.interaction.selectable_labels = false;

    // Тёмная тема в духе UE5 / Zed
    style.visuals = Visuals::dark();
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(40, 40, 45);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(60, 60, 70);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(80, 80, 90);
    style.visuals.panel_fill = Color32::from_rgb(10, 10, 10);
    style.visuals.window_fill = Color32::from_rgb(25, 25, 30);
    style.visuals.code_bg_color = Color32::from_rgb(255, 255, 255);
    style.visuals.hyperlink_color = Color32::from_rgb(100, 180, 255);
    style.visuals.selection.bg_fill = Color32::from_rgb(60, 100, 160);

    // Опционально: настроить шрифт
    style
        .text_styles
        .get_mut(&egui::TextStyle::Body)
        .unwrap()
        .size = 18.0;

    ctx.set_style(style);
}

use egui::{Align, Layout, Response, RichText, Ui};

fn nav_button(
    ui: &mut Ui,
    icon: Option<&egui::TextureHandle>,
    label: &str,
    is_active: bool,
) -> Response {
    let button_height = 36.0;
    let desired_size = egui::vec2(ui.available_width(), button_height);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    // Фон: прозрачный по умолчанию, подсветка при наведении/активности
    let bg_fill = if is_active {
        egui::Color32::from_rgb(60, 60, 75)
    } else if response.hovered() {
        egui::Color32::from_rgb(50, 50, 60)
    } else {
        egui::Color32::TRANSPARENT
    };

    ui.painter().rect_filled(rect, 4.0, bg_fill);

    // Цвет текста
    let text_color = if is_active {
        egui::Color32::WHITE
    } else {
        egui::Color32::from_gray(200)
    };

    // Создаём дочерний UI для размещения иконки и текста
    let mut child_ui = ui.new_child(
        egui::UiBuilder::new()
            .ui_stack_info(egui::UiStackInfo::new(egui::UiKind::Window))
            .max_rect(rect.shrink(8.0))
            .layout(Layout::left_to_right(Align::Center)),
    );

    // Иконка
    if let Some(tex) = icon {
        child_ui.add(egui::Image::new(tex).max_width(24.0));
    }

    // Отступ между иконкой и текстом
    child_ui.add_space(8.0);

    // Текст
    child_ui.label(RichText::new(label).color(text_color));

    response
}
