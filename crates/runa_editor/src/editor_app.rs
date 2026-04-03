use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::Arc;
use std::time::{Instant, SystemTime};

use egui::{Align2, Layout, RichText};
use egui_wgpu::{Renderer as EguiRenderer, RendererOptions, ScreenDescriptor};
use egui_winit::State as EguiWinitState;
use rfd::FileDialog;
use runa_asset::load_window_icon;
use runa_core::components::{
    Mesh, MeshRenderer, ObjectDefinitionInstance, PhysicsCollision, Transform,
};
use runa_core::glam::{Vec2, Vec3};
use runa_core::ocs::Object;
use runa_core::World;
use runa_project::{
    create_empty_project, create_empty_world, ensure_editor_bridge_files, find_project_manifest,
    load_project, load_world, save_world, AudioSourceAsset, PlaceableObjectDescriptor,
    PlaceableObjectRecord, ProjectPaths, SpriteRendererAsset, TilemapAsset, TilemapLayerAsset,
    TransformAsset, WorldObjectAsset,
};
use runa_render::{RenderTarget, Renderer};
use runa_render_api::RenderQueue;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
use winit::platform::windows::WindowExtWindows;
use winit::window::{Window, WindowId};

use crate::content_browser::ContentBrowserState;
use crate::editor_camera::EditorCameraController;
use crate::editor_settings::EditorSettings;
use crate::inspector::inspector_ui;
use crate::style;

pub fn run(project_path: Option<PathBuf>) -> Result<(), winit::error::EventLoopError> {
    let event_loop = EventLoop::new()?;
    let mut app = EditorApp::new(project_path);
    event_loop.run_app(&mut app)
}

struct PanelState {
    hierarchy: bool,
    inspector: bool,
    bottom_bar: bool,
}

impl Default for PanelState {
    fn default() -> Self {
        Self {
            hierarchy: true,
            inspector: true,
            bottom_bar: true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BottomTab {
    ContentBrowser,
    Console,
}

#[derive(Default)]
struct ProjectDialogState {
    open: bool,
    name: String,
    location: String,
}

#[derive(Clone)]
struct ProjectSession {
    project: ProjectPaths,
    current_world_path: Option<PathBuf>,
}

struct ProjectLoadResult {
    project: ProjectPaths,
    object_records: Vec<PlaceableObjectRecord>,
}

struct ObjectClipboard {
    asset: WorldObjectAsset,
    cut_index: Option<usize>,
}

struct PlaceObjectState {
    objects: Vec<PlaceableObjectDescriptor>,
    templates: HashMap<String, WorldObjectAsset>,
    source_stamp: Option<SystemTime>,
    pending_stamp: Option<SystemTime>,
    refresh_in_progress: bool,
    refresh_result: Option<Receiver<Result<Vec<PlaceableObjectRecord>, String>>>,
}

impl Default for PlaceObjectState {
    fn default() -> Self {
        Self {
            objects: Vec::new(),
            templates: HashMap::new(),
            source_stamp: None,
            pending_stamp: None,
            refresh_in_progress: false,
            refresh_result: None,
        }
    }
}

pub struct EditorApp<'window> {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer<'window>>,
    egui_state: Option<EguiWinitState>,
    egui_renderer: Option<EguiRenderer>,
    egui_ctx: egui::Context,

    world: World,
    scene_queue: RenderQueue,
    selection: Option<usize>,
    content_browser: ContentBrowserState,
    panels: PanelState,
    settings: EditorSettings,
    settings_open: bool,
    project_dialog: ProjectDialogState,
    project_session: Option<ProjectSession>,
    startup_project_path: Option<PathBuf>,
    runtime_process: Option<Child>,
    project_load: Option<Receiver<Result<ProjectLoadResult, String>>>,
    place_object: PlaceObjectState,
    hierarchy_clipboard: Option<ObjectClipboard>,
    output_lines: Vec<String>,
    output_tx: Sender<String>,
    output_rx: Receiver<String>,

    editor_camera: EditorCameraController,
    viewport_target: Option<RenderTarget>,
    viewport_texture_id: Option<egui::TextureId>,
    pending_viewport_size: (u32, u32),
    viewport_hovered: bool,
    modifiers: ModifiersState,
    bottom_tab: BottomTab,
    bottom_bar_height: f32,

    status_line: String,
    last_frame_time: Instant,
}

impl<'window> EditorApp<'window> {
    fn new(project_path: Option<PathBuf>) -> Self {
        let startup_root = std::env::current_dir().unwrap_or_default();
        let (output_tx, output_rx) = mpsc::channel();
        let project_dialog = ProjectDialogState {
            open: false,
            name: "MyGame".to_string(),
            location: dirs::document_dir()
                .unwrap_or_else(|| startup_root.clone())
                .to_string_lossy()
                .to_string(),
        };

        Self {
            output_lines: vec!["Editor started.".to_string()],
            settings: EditorSettings::load(),
            settings_open: false,
            project_dialog,
            project_session: None,
            startup_project_path: project_path.or_else(|| find_project_manifest(&startup_root)),
            runtime_process: None,
            project_load: None,
            place_object: PlaceObjectState::default(),
            hierarchy_clipboard: None,
            window: None,
            renderer: None,
            egui_state: None,
            egui_renderer: None,
            egui_ctx: egui::Context::default(),
            world: create_preview_world(),
            scene_queue: RenderQueue::new(),
            selection: Some(0),
            content_browser: ContentBrowserState::new(startup_root),
            panels: PanelState::default(),
            editor_camera: EditorCameraController::new(),
            viewport_target: None,
            viewport_texture_id: None,
            pending_viewport_size: style::panel_sizes::INITIAL_VIEWPORT,
            viewport_hovered: false,
            modifiers: ModifiersState::default(),
            bottom_tab: BottomTab::ContentBrowser,
            bottom_bar_height: style::panel_sizes::BOTTOM_BAR_HEIGHT,
            status_line:
                "Right mouse in viewport: look. WASD move, Space/Ctrl vertical, Shift boost."
                    .to_string(),
            last_frame_time: Instant::now(),
            output_tx,
            output_rx,
        }
    }

    fn ensure_viewport_target(&mut self) {
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };
        let Some(egui_renderer) = self.egui_renderer.as_mut() else {
            return;
        };

        let needs_recreate = self
            .viewport_target
            .as_ref()
            .map(|target| target.size() != self.pending_viewport_size)
            .unwrap_or(true);

        if !needs_recreate {
            return;
        }

        let target = renderer.create_render_target(self.pending_viewport_size);
        if let Some(texture_id) = self.viewport_texture_id {
            egui_renderer.update_egui_texture_from_wgpu_texture(
                renderer.device(),
                target.color_view(),
                wgpu::FilterMode::Linear,
                texture_id,
            );
        } else {
            let texture_id = egui_renderer.register_native_texture(
                renderer.device(),
                target.color_view(),
                wgpu::FilterMode::Linear,
            );
            self.viewport_texture_id = Some(texture_id);
        }
        self.viewport_target = Some(target);
    }

    fn update_scene_preview(&mut self) {
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };
        let Some(target) = self.viewport_target.as_ref() else {
            return;
        };

        let now = Instant::now();
        let dt = (now - self.last_frame_time).as_secs_f32().min(0.1);
        self.last_frame_time = now;

        self.editor_camera
            .set_viewport_hovered(self.viewport_hovered);
        self.editor_camera.update(dt);

        let camera = self.editor_camera.camera(target.size());
        let virtual_size = Vec2::new(target.size().0 as f32, target.size().1 as f32);

        self.scene_queue.clear();
        self.world.render(&mut self.scene_queue, 1.0);
        renderer.draw_to_target(target, &self.scene_queue, camera.matrix(), virtual_size);
    }

    fn draw_ui(&mut self) -> egui::FullOutput {
        let window = self.window.as_ref().unwrap();
        let egui_state = self.egui_state.as_mut().unwrap();
        let raw_input = egui_state.take_egui_input(window);
        let egui_ctx = self.egui_ctx.clone();
        egui_ctx.run_ui(raw_input, |ctx| {
            self.build_ui(ctx);
        })
    }

    fn build_ui(&mut self, ctx: &egui::Context) {
        egui::Panel::top("editor_top_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Project...").clicked() {
                        self.project_dialog.open = true;
                        ui.close();
                    }
                    if ui.button("Open Project...").clicked() {
                        self.open_project_dialog();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("New World").clicked() {
                        self.new_world();
                        ui.close();
                    }
                    if ui.button("Open World...").clicked() {
                        self.open_world_dialog();
                        ui.close();
                    }
                    if ui.button("Save World").clicked() {
                        self.save_current_world();
                        ui.close();
                    }
                    if ui.button("Save World As...").clicked() {
                        self.save_world_as_dialog();
                        ui.close();
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Editor Settings").clicked() {
                        self.settings_open = true;
                        ui.close();
                    }
                    if ui.button("Project Settings").clicked() {
                        self.settings_open = true;
                        ui.close();
                    }
                });
                ui.menu_button("Build", |ui| if ui.button("Building Settings").clicked() {});
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.panels.hierarchy, "Hierarchy");
                    ui.checkbox(&mut self.panels.inspector, "Inspector");
                    ui.checkbox(&mut self.panels.bottom_bar, "Bottom Bar");
                });
                ui.separator();

                let play_label = if self.runtime_process.is_some() {
                    "Stop"
                } else {
                    "Play In Window"
                };
                let play_response = ui.add_enabled(
                    self.project_session.is_some(),
                    egui::Button::new(play_label),
                );
                if play_response.clicked() {
                    if self.runtime_process.is_some() {
                        self.stop_project();
                    } else {
                        self.play_project();
                    }
                }

                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(self.window_title()).strong());
                });
            });
        });

        egui::Panel::bottom("status_bar")
            .resizable(false)
            .exact_size(24.0)
            .show_separator_line(false)
            .show(ctx, |ui| {
                ui.label(&self.status_line);
            });

        if self.panels.bottom_bar {
            let max_bottom_bar_height = (ctx.content_rect().height() - 140.0).max(120.0);
            self.bottom_bar_height = self.bottom_bar_height.clamp(80.0, max_bottom_bar_height);
            egui::Panel::bottom("bottom_bar")
                .resizable(false)
                .exact_size(self.bottom_bar_height)
                .show(ctx, |ui| {
                    let handle_height = 10.0;
                    let (handle_rect, handle_response) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), handle_height),
                        egui::Sense::click_and_drag(),
                    );
                    if handle_response.dragged() {
                        self.bottom_bar_height = (self.bottom_bar_height
                            - handle_response.drag_delta().y)
                            .clamp(80.0, max_bottom_bar_height);
                    }
                    ui.painter().rect_filled(
                        handle_rect.shrink2(egui::vec2(48.0, 3.0)),
                        4.0,
                        ui.visuals().widgets.inactive.bg_fill,
                    );
                    ui.horizontal(|ui| {
                        let browser_selected = self.bottom_tab == BottomTab::ContentBrowser;
                        if ui
                            .selectable_label(browser_selected, "Content Browser")
                            .clicked()
                        {
                            self.bottom_tab = BottomTab::ContentBrowser;
                        }
                        let console_selected = self.bottom_tab == BottomTab::Console;
                        if ui.selectable_label(console_selected, "Console").clicked() {
                            self.bottom_tab = BottomTab::Console;
                        }
                        ui.separator();
                        match self.bottom_tab {
                            BottomTab::ContentBrowser => {
                                if ui.button("Import").clicked() {}
                                ui.separator();
                                ui.label(self.content_browser.current_dir_display());
                            }
                            BottomTab::Console => {
                                if ui.button("Clear").clicked() {
                                    self.output_lines.clear();
                                }
                            }
                        }
                    });
                    ui.separator();
                    match self.bottom_tab {
                        BottomTab::ContentBrowser => {
                            self.content_browser.ui(ui, &self.settings);
                            if let Some(message) = self.content_browser.take_message() {
                                self.status_line = message;
                            }
                        }
                        BottomTab::Console => {
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .stick_to_bottom(true)
                                .id_salt("editor_output_scroll")
                                .show(ui, |ui| {
                                    ui.set_min_width(ui.available_width());
                                    for line in &self.output_lines {
                                        ui.monospace(line);
                                    }
                                });
                        }
                    }
                });
        }

        if self.panels.hierarchy {
            egui::Panel::left("hierarchy_panel")
                .resizable(true)
                .default_size(240.0)
                .min_size(180.0)
                .show(ctx, |ui| {
                    ui.heading("Hierarchy");
                    ui.separator();
                    let hierarchy_items: Vec<String> = self
                        .world
                        .objects
                        .iter()
                        .enumerate()
                        .map(|(index, object)| object_title(index, object))
                        .collect();
                    egui::ScrollArea::vertical()
                        .id_salt("hierarchy_scroll")
                        .show(ui, |ui| {
                            for (index, title) in hierarchy_items.iter().enumerate() {
                                let selected = self.selection == Some(index);
                                let response = ui.selectable_label(selected, title);
                                if response.clicked() {
                                    self.selection = Some(index);
                                }
                                response.context_menu(|ui| {
                                    self.selection = Some(index);
                                    self.hierarchy_context_menu_ui(ui, Some(index));
                                });
                            }
                            let blank_response = ui.allocate_response(
                                egui::vec2(ui.available_width(), ui.available_height().max(24.0)),
                                egui::Sense::click(),
                            );
                            blank_response.context_menu(|ui| {
                                self.hierarchy_context_menu_ui(ui, None);
                            });
                        });
                });
        }

        if self.panels.inspector {
            egui::Panel::right("inspector_panel")
                .resizable(true)
                .default_size(320.0)
                .min_size(220.0)
                .show(ctx, |ui| {
                    ui.heading("Inspector");
                    ui.separator();
                    if let Some(index) = self.selection {
                        if let Some(object) = self.world.objects.get_mut(index) {
                            let project_root = self
                                .project_session
                                .as_ref()
                                .map(|session| session.project.root_dir.as_path());
                            inspector_ui(ui, object, project_root);
                        } else {
                            ui.label("Selection is out of bounds.");
                        }
                    } else {
                        ui.label("Select an object in the hierarchy.");
                    }
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("World");
                if let Some(path) = self.current_world_path() {
                    ui.label(path.display().to_string());
                } else {
                    ui.label("Unsaved world");
                }

                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut s: bool = true; // for tests
                    ui.menu_button("Gizmo", |ui| {
                        ui.checkbox(&mut s, "Debug render");
                        ui.checkbox(&mut s, "Icons");
                    });

                    ui.menu_button("View Settings", |ui| {
                        ui.heading("Editor Camera Settings");
                        ui.horizontal(|ui| {
                            ui.label("FOV");
                            let mut degrees = self.editor_camera.get_fov();
                            if ui
                                .add(egui::DragValue::new(&mut degrees).speed(1).range(30..=130))
                                .changed()
                            {
                                self.editor_camera.set_fov(degrees);
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Sensitivity");
                            let mut sens = self.editor_camera.get_sensitivity();
                            if ui
                                .add(
                                    egui::DragValue::new(&mut sens)
                                        .speed(0.01)
                                        .range(0.0..=999.0),
                                )
                                .changed()
                            {
                                self.editor_camera.set_sensitivity(sens);
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Speed");
                            let mut speed = self.editor_camera.get_speed();
                            if ui
                                .add(
                                    egui::DragValue::new(&mut speed)
                                        .speed(0.01)
                                        .range(0.0..=999.0),
                                )
                                .changed()
                            {
                                self.editor_camera.set_speed(speed);
                            }
                        });
                        if ui.button("Close").clicked() {
                            ui.close();
                        }
                    });
                    ui.menu_button("Rendering Settings", |ui| {
                        ui.heading("Preferences");
                        if ui.button("Close").clicked() {
                            ui.close();
                        }
                    });
                });
            });
            ui.separator();

            let desired_size = ui.available_size().max(egui::vec2(64.0, 64.0));
            let pixels_per_point = ctx.pixels_per_point();
            self.pending_viewport_size = (
                (desired_size.x * pixels_per_point).round().max(1.0) as u32,
                (desired_size.y * pixels_per_point).round().max(1.0) as u32,
            );

            let frame = egui::Frame::canvas(ui.style())
                .fill(style::VIEWPORT_BACKGROUND)
                .inner_margin(egui::Margin::same(0));

            frame.show(ui, |ui| {
                if let Some(texture_id) = self.viewport_texture_id {
                    let response = ui.add(egui::Image::new((texture_id, desired_size)));
                    self.viewport_hovered = response.hovered();
                } else {
                    self.viewport_hovered = false;
                    ui.allocate_ui_with_layout(
                        desired_size,
                        Layout::centered_and_justified(egui::Direction::TopDown),
                        |ui| {
                            ui.label("Viewport is initializing...");
                        },
                    );
                }
            });
        });

        self.settings_window(ctx);
        self.project_dialog_window(ctx);
        self.project_loading_overlay(ctx);
    }

    fn render_frame(&mut self) {
        let full_output = self.draw_ui();
        self.ensure_viewport_target();
        self.update_scene_preview();

        let Some(window) = self.window.as_ref() else {
            return;
        };
        let Some(egui_state) = self.egui_state.as_mut() else {
            return;
        };
        egui_state.handle_platform_output(window, full_output.platform_output);

        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };
        let Some(egui_renderer) = self.egui_renderer.as_mut() else {
            return;
        };

        for (id, image_delta) in &full_output.textures_delta.set {
            egui_renderer.update_texture(renderer.device(), renderer.queue(), *id, image_delta);
        }

        let size = window.inner_size();
        let pixels_per_point = window.scale_factor() as f32;
        let paint_jobs = self
            .egui_ctx
            .tessellate(full_output.shapes, pixels_per_point);
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [size.width.max(1), size.height.max(1)],
            pixels_per_point,
        };

        let surface_texture = match renderer.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            _ => return,
        };
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            renderer
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Editor UI Encoder"),
                });

        let mut command_buffers = egui_renderer.update_buffers(
            renderer.device(),
            renderer.queue(),
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );

        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Editor UI Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(style::RENDER_CLEAR_COLOR),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            let mut render_pass = render_pass.forget_lifetime();
            egui_renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
        }

        command_buffers.push(encoder.finish());
        renderer.queue().submit(command_buffers);
        surface_texture.present();

        for id in &full_output.textures_delta.free {
            egui_renderer.free_texture(id);
        }
    }

    fn new_world(&mut self) {
        self.world = create_empty_world();
        self.selection = if self.world.objects.is_empty() {
            None
        } else {
            Some(0)
        };
        if let Some(session) = self.project_session.as_mut() {
            session.current_world_path = None;
        }
        self.status_line = "Created a new world.".to_string();
    }

    fn open_project_dialog(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("Runa Project", &["runaproj"])
            .pick_file()
        {
            self.begin_load_project(path);
        }
    }

    fn begin_load_project(&mut self, path: PathBuf) {
        let output_tx = self.output_tx.clone();
        let (tx, rx) = mpsc::channel();
        self.project_load = Some(rx);
        self.status_line = format!("Opening project {}...", path.display());
        self.push_output(format!("Opening project {}...", path.display()));

        std::thread::spawn(move || {
            let _ = output_tx.send(format!("Loading project manifest: {}", path.display()));
            let result = (|| -> Result<ProjectLoadResult, String> {
                let project = load_project(&path).map_err(|error| error.to_string())?;
                ensure_editor_bridge_files(&project.root_dir).map_err(|error| error.to_string())?;
                let object_records = query_placeable_object_records(&project, &output_tx)?;

                Ok(ProjectLoadResult {
                    project,
                    object_records,
                })
            })();
            let _ = tx.send(result);
        });
    }

    fn apply_loaded_project(&mut self, result: ProjectLoadResult) {
        let startup_world_path = result.project.startup_world_path();
        let world = if startup_world_path.exists() {
            match load_world(&startup_world_path) {
                Ok(world) => world,
                Err(error) => {
                    self.push_output(format!(
                        "Failed to load startup world: {error}. Using an empty world instead."
                    ));
                    create_empty_world()
                }
            }
        } else {
            create_empty_world()
        };
        self.project_session = Some(ProjectSession {
            current_world_path: Some(startup_world_path),
            project: result.project.clone(),
        });
        self.world = world;
        self.selection = if self.world.objects.is_empty() {
            None
        } else {
            Some(0)
        };
        self.content_browser
            .set_project_root(result.project.root_dir.clone(), &self.settings);
        let merged_records = merge_placeable_object_records(result.object_records);
        self.place_object.objects = merged_records
            .iter()
            .map(|record| record.descriptor.clone())
            .collect();
        self.place_object.templates = merged_records
            .into_iter()
            .map(|record| (record.descriptor.id.clone(), record.object))
            .collect();
        self.place_object.source_stamp = None;
        self.status_line = format!("Opened project {}.", result.project.manifest.name);
        self.push_output(self.status_line.clone());
    }

    fn open_world_dialog(&mut self) {
        let start_dir = self
            .project_session
            .as_ref()
            .map(|session| session.project.worlds_dir())
            .unwrap_or_else(default_browse_root);
        if let Some(path) = FileDialog::new()
            .set_directory(start_dir)
            .add_filter("Runa World", &["ron"])
            .pick_file()
        {
            self.open_world_from_path(path);
        }
    }

    fn open_world_from_path(&mut self, path: PathBuf) {
        match load_world(&path) {
            Ok(world) => {
                self.world = world;
                self.selection = if self.world.objects.is_empty() {
                    None
                } else {
                    Some(0)
                };
                if let Some(session) = self.project_session.as_mut() {
                    session.current_world_path = Some(path.clone());
                }
                self.status_line = format!("Opened world {}.", path.display());
            }
            Err(error) => {
                self.status_line = format!("Failed to open world: {error}");
            }
        }
    }

    fn save_current_world(&mut self) {
        if let Some(path) = self.current_world_path() {
            self.save_world_to_path(path);
        } else {
            self.save_world_as_dialog();
        }
    }

    fn save_world_as_dialog(&mut self) {
        let suggested_dir = self
            .project_session
            .as_ref()
            .map(|session| session.project.worlds_dir())
            .unwrap_or_else(default_browse_root);
        if let Some(path) = FileDialog::new()
            .set_directory(suggested_dir)
            .set_file_name("main.world.ron")
            .add_filter("Runa World", &["ron"])
            .save_file()
        {
            self.save_world_to_path(ensure_world_extension(path));
        }
    }

    fn save_world_to_path(&mut self, path: PathBuf) {
        match save_world(&path, &self.world) {
            Ok(()) => {
                if let Some(session) = self.project_session.as_mut() {
                    session.current_world_path = Some(path.clone());
                    if let Ok(relative) = path.strip_prefix(&session.project.root_dir) {
                        session.project.manifest.startup_world =
                            relative.to_string_lossy().replace('\\', "/");
                        if let Err(error) = session.project.save_manifest() {
                            self.status_line =
                                format!("World saved but failed to update startup world: {error}");
                            return;
                        }
                    }
                }
                self.status_line = format!("Saved world to {}.", path.display());
            }
            Err(error) => {
                self.status_line = format!("Failed to save world: {error}");
            }
        }
    }

    fn play_project(&mut self) {
        let Some(session) = self.project_session.as_ref().cloned() else {
            self.status_line = "Open a project before starting Play mode.".to_string();
            return;
        };

        if self.runtime_process.is_some() {
            self.stop_project();
        }

        self.save_current_world();

        let mut command = Command::new("cargo");
        command
            .args(["run", "--bin", &session.project.manifest.binary_name])
            .current_dir(&session.project.root_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        configure_background_command(&mut command);

        match command.spawn() {
            Ok(child) => {
                self.runtime_process = Some(child);
                if let Some(child_ref) = self.runtime_process.as_mut() {
                    attach_child_output(child_ref, self.output_tx.clone(), "play");
                }
                self.status_line =
                    format!("Started Play mode for {}.", session.project.manifest.name);
                self.push_output(self.status_line.clone());
            }
            Err(error) => {
                self.status_line = format!("Failed to start Play mode: {error}");
                self.push_output(self.status_line.clone());
            }
        }
    }

    fn stop_project(&mut self) {
        let Some(mut child) = self.runtime_process.take() else {
            return;
        };

        match child.kill() {
            Ok(()) => {
                let _ = child.wait();
                self.status_line = "Stopped Play mode.".to_string();
                self.push_output(self.status_line.clone());
            }
            Err(error) => {
                self.status_line = format!("Failed to stop Play mode: {error}");
                self.push_output(self.status_line.clone());
            }
        }
    }

    fn update_runtime_process_state(&mut self) {
        let Some(child) = self.runtime_process.as_mut() else {
            return;
        };

        match child.try_wait() {
            Ok(Some(status)) => {
                self.runtime_process = None;
                self.status_line = format!("Play mode exited with status {status}.");
                self.push_output(self.status_line.clone());
            }
            Ok(None) => {}
            Err(error) => {
                self.runtime_process = None;
                self.status_line = format!("Failed to poll Play mode: {error}");
                self.push_output(self.status_line.clone());
            }
        }
    }

    fn poll_project_load(&mut self) {
        let Some(receiver) = self.project_load.as_ref() else {
            return;
        };

        match receiver.try_recv() {
            Ok(result) => {
                self.project_load = None;
                match result {
                    Ok(result) => self.apply_loaded_project(result),
                    Err(error) => {
                        self.status_line = format!("Failed to open project: {error}");
                        self.push_output(self.status_line.clone());
                    }
                }
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.project_load = None;
                self.status_line = "Project loading task disconnected.".to_string();
                self.push_output(self.status_line.clone());
            }
        }
    }

    fn poll_output(&mut self) {
        while let Ok(line) = self.output_rx.try_recv() {
            self.output_lines.push(line);
            if self.output_lines.len() > 500 {
                let drain_len = self.output_lines.len() - 500;
                self.output_lines.drain(0..drain_len);
            }
        }
    }

    fn push_output(&mut self, line: impl Into<String>) {
        self.output_lines.push(line.into());
        if self.output_lines.len() > 500 {
            let drain_len = self.output_lines.len() - 500;
            self.output_lines.drain(0..drain_len);
        }
    }

    fn project_loading_overlay(&mut self, ctx: &egui::Context) {
        if self.project_load.is_none() {
            return;
        }

        egui::Window::new("Loading Project")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.label("Project is loading in the background.");
                ui.add(egui::Spinner::new());
                ui.label("Preparing bridge, loading world, and caching placeable objects.");
            });
    }

    fn hierarchy_context_menu_ui(&mut self, ui: &mut egui::Ui, target_index: Option<usize>) {
        if let Some(index) = target_index {
            if ui.button("Copy").clicked() {
                self.copy_object(index, false);
                ui.close();
            }
            if ui.button("Cut").clicked() {
                self.copy_object(index, true);
                ui.close();
            }
            if ui
                .add_enabled(
                    self.hierarchy_clipboard.is_some(),
                    egui::Button::new("Paste"),
                )
                .clicked()
            {
                self.paste_object(target_index);
                ui.close();
            }
            if ui.button("Delete").clicked() {
                self.delete_object(index);
                ui.close();
            }
            ui.separator();
        } else if ui
            .add_enabled(
                self.hierarchy_clipboard.is_some(),
                egui::Button::new("Paste"),
            )
            .clicked()
        {
            self.paste_object(None);
            ui.close();
            return;
        }

        self.place_object_menu_ui(ui);
    }

    fn copy_object(&mut self, index: usize, cut: bool) {
        let Some(object) = self.world.objects.get(index) else {
            return;
        };
        self.hierarchy_clipboard = Some(ObjectClipboard {
            asset: WorldObjectAsset::from_object(object),
            cut_index: cut.then_some(index),
        });
        self.status_line = if cut {
            "Cut object.".to_string()
        } else {
            "Copied object.".to_string()
        };
    }

    fn paste_object(&mut self, target_index: Option<usize>) {
        let Some(clipboard) = self.hierarchy_clipboard.take() else {
            return;
        };
        let project_root = self
            .project_session
            .as_ref()
            .map(|session| session.project.root_dir.as_path());
        let mut object = clipboard.asset.clone().into_object(project_root);
        if let Some(object_id) = clipboard.asset.object_id.clone() {
            if object.get_component::<ObjectDefinitionInstance>().is_none() {
                object.add_component(ObjectDefinitionInstance::new(object_id));
            }
        }

        let mut insert_index = target_index
            .map(|index| index + 1)
            .unwrap_or(self.world.objects.len());
        if let Some(cut_index) = clipboard.cut_index {
            if cut_index < self.world.objects.len() {
                self.world.objects.remove(cut_index);
                if cut_index < insert_index {
                    insert_index = insert_index.saturating_sub(1);
                }
            }
        }
        self.world.objects.insert(insert_index, object);
        self.selection = Some(insert_index);
        self.status_line = "Pasted object.".to_string();
    }

    fn delete_object(&mut self, index: usize) {
        if index >= self.world.objects.len() {
            return;
        }
        self.world.objects.remove(index);
        self.selection = if self.world.objects.is_empty() {
            None
        } else {
            Some(index.min(self.world.objects.len() - 1))
        };
        self.status_line = "Deleted object.".to_string();
    }

    fn current_world_path(&self) -> Option<PathBuf> {
        self.project_session
            .as_ref()
            .and_then(|session| session.current_world_path.clone())
    }

    fn window_title(&self) -> String {
        if let Some(session) = self.project_session.as_ref() {
            format!("Runa Editor - {}", session.project.manifest.name)
        } else {
            "Runa Editor".to_string()
        }
    }

    fn settings_window(&mut self, ctx: &egui::Context) {
        if !self.settings_open {
            return;
        }

        let mut open = self.settings_open;
        egui::Window::new("Editor Settings")
            .open(&mut open)
            .resizable(true)
            .anchor(Align2::CENTER_CENTER, [0., 0.])
            .movable(true)
            .default_width(460.0)
            .show(ctx, |ui| {
                ui.heading("External Editor");
                ui.horizontal(|ui| {
                    ui.label("Executable");
                    ui.text_edit_singleline(&mut self.settings.external_editor_executable);
                });
                ui.label("Arguments");
                ui.label("One argument per line. Use {file} as placeholder.");
                ui.add(
                    egui::TextEdit::multiline(&mut self.settings.external_editor_args)
                        .desired_rows(3)
                        .desired_width(f32::INFINITY),
                );
                ui.horizontal(|ui| {
                    if ui.button("Use Zed").clicked() {
                        self.settings.external_editor_executable = "zed".to_string();
                        self.settings.external_editor_args = "{file}".to_string();
                    }
                    if ui.button("Use VS Code").clicked() {
                        self.settings.external_editor_executable = "code".to_string();
                        self.settings.external_editor_args = "--goto\n{file}".to_string();
                    }
                });

                ui.separator();
                ui.heading("Interface");
                ui.horizontal(|ui| {
                    ui.label("UI Scale");
                    if ui
                        .add(egui::Slider::new(&mut self.settings.ui_scale, 0.75..=2.0))
                        .changed()
                    {
                        ctx.set_zoom_factor(self.settings.ui_scale);
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Icon Size");
                    ui.add(
                        egui::Slider::new(&mut self.settings.content_icon_size, 32.0..=96.0)
                            .step_by(1.0),
                    );
                });
                if ui
                    .checkbox(&mut self.settings.show_hidden_files, "Show Hidden Files")
                    .changed()
                {
                    self.content_browser.refresh(&self.settings);
                }

                ui.separator();
                if ui.button("Save Settings").clicked() {
                    match self.settings.save() {
                        Ok(()) => {
                            self.content_browser.refresh(&self.settings);
                            self.status_line = "Editor settings saved.".to_string();
                        }
                        Err(error) => {
                            self.status_line = format!("Failed to save settings: {error}");
                        }
                    }
                }
            });
        self.settings_open = open;
    }

    fn project_dialog_window(&mut self, ctx: &egui::Context) {
        if !self.project_dialog.open {
            return;
        }

        let mut open = self.project_dialog.open;
        egui::Window::new("New Project")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(520.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name");
                    ui.text_edit_singleline(&mut self.project_dialog.name);
                });
                ui.horizontal(|ui| {
                    ui.label("Location");
                    ui.text_edit_singleline(&mut self.project_dialog.location);
                    if ui.button("Browse...").clicked() {
                        if let Some(path) = FileDialog::new().pick_folder() {
                            self.project_dialog.location = path.to_string_lossy().to_string();
                        }
                    }
                });

                ui.separator();
                if ui.button("Create Project").clicked() {
                    self.create_project_from_dialog();
                }
            });
        self.project_dialog.open = open;
    }

    fn create_project_from_dialog(&mut self) {
        let name = self.project_dialog.name.trim();
        let location = self.project_dialog.location.trim();
        if name.is_empty() || location.is_empty() {
            self.status_line = "Project name and location are required.".to_string();
            return;
        }

        let destination = PathBuf::from(location).join(name);
        match create_empty_project(&destination, name) {
            Ok(project) => {
                self.project_dialog.open = false;
                self.begin_load_project(project.manifest_path.clone());
                self.status_line = format!("Created project {}.", project.manifest.name);
            }
            Err(error) => {
                self.status_line = format!("Failed to create project: {error}");
            }
        }
    }

    fn place_object_menu_ui(&mut self, ui: &mut egui::Ui) {
        if self.project_session.is_none() {
            ui.label("Open a project to use Place Object.");
            return;
        }

        self.refresh_placeable_objects_if_needed(false);
        self.poll_place_object_refresh();

        if ui.button("Refresh Object List").clicked() {
            self.refresh_placeable_objects_if_needed(true);
            ui.close();
            return;
        }

        if self.place_object.refresh_in_progress {
            ui.separator();
            ui.label("Refreshing objects...");
            return;
        }

        if self.place_object.objects.is_empty() {
            ui.separator();
            ui.label("No placeable objects found.");
            return;
        }

        ui.separator();
        let mut categories: Vec<String> = self
            .place_object
            .objects
            .iter()
            .map(|object| object.category.clone())
            .collect();
        categories.sort();
        categories.dedup();

        for category in categories {
            ui.menu_button(category.clone(), |ui| {
                let matching_objects: Vec<PlaceableObjectDescriptor> = self
                    .place_object
                    .objects
                    .iter()
                    .filter(|object| object.category == category)
                    .cloned()
                    .collect();
                for object in matching_objects {
                    if ui.button(&object.name).clicked() {
                        self.place_object(&object);
                        ui.close();
                    }
                }
            });
        }
    }

    fn place_object(&mut self, object: &PlaceableObjectDescriptor) {
        let Some(mut asset) = self.place_object.templates.get(&object.id).cloned() else {
            self.status_line = format!("Failed to spawn object {}.", object.name);
            return;
        };

        asset.object_id = Some(object.id.clone());
        let project_root = self
            .project_session
            .as_ref()
            .map(|session| session.project.root_dir.as_path());
        let mut world_object = asset.into_object(project_root);
        if world_object
            .get_component::<ObjectDefinitionInstance>()
            .is_none()
        {
            world_object.add_component(ObjectDefinitionInstance::new(object.id.clone()));
        }

        self.world.objects.push(world_object);
        self.selection = Some(self.world.objects.len().saturating_sub(1));
        self.status_line = format!("Placed object {}.", object.name);
    }

    fn refresh_placeable_objects_if_needed(&mut self, force: bool) {
        let Some(session) = self.project_session.as_ref().cloned() else {
            return;
        };

        self.poll_place_object_refresh();
        let latest_stamp = latest_place_object_stamp(&session.project);
        if self.place_object.refresh_in_progress {
            return;
        }
        if !force && latest_stamp == self.place_object.source_stamp {
            return;
        }

        let project = session.project.clone();
        let output_tx = self.output_tx.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(query_placeable_object_records(&project, &output_tx));
        });
        self.place_object.pending_stamp = latest_stamp;
        self.place_object.refresh_in_progress = true;
        self.place_object.refresh_result = Some(rx);
    }

    fn poll_place_object_refresh(&mut self) {
        let Some(receiver) = self.place_object.refresh_result.as_ref() else {
            return;
        };

        match receiver.try_recv() {
            Ok(result) => {
                self.place_object.refresh_result = None;
                self.place_object.refresh_in_progress = false;
                match result {
                    Ok(records) => {
                        let merged_records = merge_placeable_object_records(records);
                        self.place_object.objects = merged_records
                            .iter()
                            .map(|record| record.descriptor.clone())
                            .collect();
                        self.place_object.templates = merged_records
                            .into_iter()
                            .map(|record| (record.descriptor.id.clone(), record.object))
                            .collect();
                        self.place_object.source_stamp = self.place_object.pending_stamp.take();
                    }
                    Err(error) => {
                        self.place_object.pending_stamp = None;
                        self.status_line = format!("Failed to refresh placeable objects: {error}");
                    }
                }
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.place_object.refresh_result = None;
                self.place_object.refresh_in_progress = false;
                self.place_object.pending_stamp = None;
                self.status_line =
                    "Object refresh failed because the background job disconnected.".to_string();
            }
        }
    }
}

impl<'window> ApplicationHandler for EditorApp<'window> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        event_loop.set_control_flow(ControlFlow::Poll);

        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Runa Editor")
                        .with_inner_size(egui_winit::winit::dpi::LogicalSize::new(1600.0, 960.0))
                        .with_min_inner_size(egui_winit::winit::dpi::LogicalSize::new(
                            1200.0, 720.0,
                        )),
                )
                .expect("Failed to create editor window"),
        );
        if let (Ok(icon), Ok(big_icon)) = (
            load_window_icon(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icon.png")),
            load_window_icon(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/big_icon.png")),
        ) {
            window.set_window_icon(Some(icon));
            window.set_taskbar_icon(Some(big_icon));
            // window.set_title_background_color(Some(Color::BLACK));
        }

        let renderer = Renderer::new(window.clone(), true);
        let egui_renderer = EguiRenderer::new(
            renderer.device(),
            renderer.surface_format(),
            RendererOptions::default(),
        );
        let egui_state = EguiWinitState::new(
            self.egui_ctx.clone(),
            egui::ViewportId::ROOT,
            window.as_ref(),
            Some(window.scale_factor() as f32),
            window.theme(),
            Some(renderer.device().limits().max_texture_dimension_2d as usize),
        );

        self.window = Some(window);
        self.renderer = Some(renderer);
        self.egui_renderer = Some(egui_renderer);
        self.egui_state = Some(egui_state);
        style::apply_editor_style(&self.egui_ctx);
        self.egui_ctx.set_zoom_factor(self.settings.ui_scale);
        self.content_browser.refresh(&self.settings);
        self.ensure_viewport_target();

        if let Some(path) = self.startup_project_path.clone() {
            self.begin_load_project(path);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref() else {
            return;
        };

        let camera_captured = self.editor_camera.handle_window_event(window, &event);
        if let Some(egui_state) = self.egui_state.as_mut() {
            let response = egui_state.on_window_event(window, &event);
            if response.repaint || camera_captured {
                window.request_redraw();
            }
        }

        match event {
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state.is_pressed()
                    && !event.repeat
                    && !self.editor_camera.is_look_active()
                    && self.modifiers.control_key()
                    && matches!(event.physical_key, PhysicalKey::Code(KeyCode::Space))
                {
                    self.panels.bottom_bar = !self.panels.bottom_bar;
                    window.request_redraw();
                }
            }
            WindowEvent::CloseRequested => {
                let shutdown_window = window.clone();
                self.stop_project();
                self.editor_camera.shutdown(&shutdown_window);
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize((size.width, size.height));
                }
            }
            WindowEvent::RedrawRequested => self.render_frame(),
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        self.editor_camera.handle_device_event(&event);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.poll_output();
        self.poll_project_load();
        self.update_runtime_process_state();
        self.poll_place_object_refresh();
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

fn create_preview_world() -> World {
    let mut world = World::default();

    let mut cube = Object::new();
    cube.name = "Preview Cube".to_string();
    let mut cube_transform = Transform::default();
    cube_transform.position = Vec3::new(0.0, 0.6, 0.0);
    cube_transform.scale = Vec3::splat(1.2);
    cube.add_component(cube_transform);
    let mut cube_mesh = MeshRenderer::new(Mesh::cube(1.5));
    cube_mesh.color = [1.0, 0.55, 0.2, 1.0];
    cube.add_component(cube_mesh);
    world.objects.push(cube);

    let mut floor = Object::new();
    floor.name = "Floor".to_string();
    let mut floor_transform = Transform::default();
    floor_transform.position = Vec3::new(0.0, -1.5, 0.0);
    floor_transform.scale = Vec3::new(8.0, 0.2, 8.0);
    floor.add_component(floor_transform);
    let mut floor_mesh = MeshRenderer::new(Mesh::cube(1.0));
    floor_mesh.color = [0.24, 0.27, 0.32, 1.0];
    floor.add_component(floor_mesh);
    floor.add_component(PhysicsCollision::new(8.0, 8.0));
    world.objects.push(floor);

    world
}

fn object_title(index: usize, object: &Object) -> String {
    if object.name.is_empty() {
        format!("Object {}", index)
    } else {
        object.name.clone()
    }
}

fn ensure_world_extension(path: PathBuf) -> PathBuf {
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.ends_with(".world.ron"))
        .unwrap_or(false)
    {
        path
    } else {
        PathBuf::from(format!("{}.world.ron", path.display()))
    }
}

fn default_browse_root() -> PathBuf {
    std::env::current_dir().unwrap_or_default()
}

fn latest_source_stamp(root: &PathBuf) -> Option<SystemTime> {
    fn visit(path: &std::path::Path, latest: &mut Option<SystemTime>) {
        let Ok(entries) = std::fs::read_dir(path) else {
            return;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                visit(&path, latest);
                continue;
            }

            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            let Ok(modified) = metadata.modified() else {
                continue;
            };

            match latest {
                Some(current) if *current >= modified => {}
                _ => *latest = Some(modified),
            }
        }
    }

    let mut latest = None;
    visit(root, &mut latest);
    latest
}

fn latest_place_object_stamp(project: &ProjectPaths) -> Option<SystemTime> {
    let mut latest = latest_source_stamp(&project.scripts_dir());
    let hidden_project_dir = project.root_dir.join(".proj");
    if let Some(hidden_stamp) = latest_source_stamp(&hidden_project_dir) {
        match latest {
            Some(current) if current >= hidden_stamp => {}
            _ => latest = Some(hidden_stamp),
        }
    }
    latest
}

fn query_placeable_object_records(
    project: &ProjectPaths,
    output_tx: &Sender<String>,
) -> Result<Vec<PlaceableObjectRecord>, String> {
    let _ = output_tx.send("Refreshing placeable objects...".to_string());

    let mut command = Command::new("cargo");
    command
        .args([
            "run",
            "--quiet",
            "--bin",
            "runa_object_bridge",
            "--",
            "--list-object-records",
        ])
        .current_dir(&project.root_dir);
    configure_background_command(&mut command);

    let output = command.output().map_err(|error| error.to_string())?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let _ = output_tx.send(format!("Bridge failed: {error}"));
        return Err(error);
    }

    ron::from_str(&String::from_utf8_lossy(&output.stdout)).map_err(|error| error.to_string())
}

fn attach_child_output(child: &mut Child, output_tx: Sender<String>, prefix: &'static str) {
    if let Some(stdout) = child.stdout.take() {
        let tx = output_tx.clone();
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                let _ = tx.send(format!("[{prefix}] {line}"));
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                let _ = output_tx.send(format!("[{prefix}] {line}"));
            }
        });
    }
}

fn configure_background_command(command: &mut Command) {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
}

fn merge_placeable_object_records(
    project_records: Vec<PlaceableObjectRecord>,
) -> Vec<PlaceableObjectRecord> {
    let mut merged = HashMap::new();
    for record in default_placeable_object_records() {
        merged.insert(record.descriptor.id.clone(), record);
    }
    for record in project_records {
        if record.descriptor.id == "player"
            && record.descriptor.name == "Player"
            && record.descriptor.category == "Gameplay"
        {
            continue;
        }
        merged.insert(record.descriptor.id.clone(), record);
    }

    let mut records: Vec<_> = merged.into_values().collect();
    records.sort_by(|left, right| {
        left.descriptor
            .category
            .cmp(&right.descriptor.category)
            .then(left.descriptor.name.cmp(&right.descriptor.name))
    });
    records
}

fn default_placeable_object_records() -> Vec<PlaceableObjectRecord> {
    vec![
        placeable_record("empty", "Empty", "Basic", empty_object_asset()),
        placeable_record("camera", "Camera", "Basic", camera_object_asset()),
        placeable_record("cube", "Cube", "Basic", cube_object_asset()),
        placeable_record("floor", "Floor", "Basic", floor_object_asset()),
        placeable_record("sprite", "Sprite", "2D", sprite_object_asset()),
        placeable_record("tilemap", "Tilemap", "2D", tilemap_object_asset()),
        placeable_record(
            "audio-source",
            "Audio Source",
            "Audio",
            audio_source_object_asset(),
        ),
    ]
}

fn placeable_record(
    id: &str,
    name: &str,
    category: &str,
    object: WorldObjectAsset,
) -> PlaceableObjectRecord {
    PlaceableObjectRecord {
        descriptor: PlaceableObjectDescriptor {
            id: id.to_string(),
            name: name.to_string(),
            category: category.to_string(),
        },
        object,
    }
}

fn empty_object_asset() -> WorldObjectAsset {
    WorldObjectAsset {
        name: "Empty".to_string(),
        object_id: None,
        transform: TransformAsset::default(),
        mesh_renderer: None,
        sprite_renderer: None,
        tilemap: None,
        camera: None,
        active_camera: false,
        audio_source: None,
        physics_collision: None,
    }
}

fn camera_object_asset() -> WorldObjectAsset {
    WorldObjectAsset {
        name: "Camera".to_string(),
        object_id: None,
        transform: TransformAsset {
            position: [0.0, 2.0, 6.0],
            ..TransformAsset::default()
        },
        mesh_renderer: None,
        sprite_renderer: None,
        tilemap: None,
        camera: Some(runa_project::CameraAsset::default()),
        active_camera: true,
        audio_source: None,
        physics_collision: None,
    }
}

fn cube_object_asset() -> WorldObjectAsset {
    WorldObjectAsset {
        name: "Cube".to_string(),
        object_id: None,
        transform: TransformAsset {
            position: [0.0, 0.75, 0.0],
            ..TransformAsset::default()
        },
        mesh_renderer: Some(runa_project::MeshRendererAsset {
            primitive: runa_project::MeshPrimitiveAsset::Cube { size: 1.5 },
            color: [0.95, 0.55, 0.22, 1.0],
        }),
        sprite_renderer: None,
        tilemap: None,
        camera: None,
        active_camera: false,
        audio_source: None,
        physics_collision: Some(runa_project::PhysicsCollisionAsset {
            size: [0.75, 0.75],
            enabled: true,
        }),
    }
}

fn floor_object_asset() -> WorldObjectAsset {
    WorldObjectAsset {
        name: "Floor".to_string(),
        object_id: None,
        transform: TransformAsset {
            position: [0.0, -1.5, 0.0],
            scale: [8.0, 0.2, 8.0],
            ..TransformAsset::default()
        },
        mesh_renderer: Some(runa_project::MeshRendererAsset {
            primitive: runa_project::MeshPrimitiveAsset::Cube { size: 1.0 },
            color: [0.24, 0.27, 0.32, 1.0],
        }),
        sprite_renderer: None,
        tilemap: None,
        camera: None,
        active_camera: false,
        audio_source: None,
        physics_collision: Some(runa_project::PhysicsCollisionAsset {
            size: [4.0, 4.0],
            enabled: true,
        }),
    }
}

fn sprite_object_asset() -> WorldObjectAsset {
    WorldObjectAsset {
        name: "Sprite".to_string(),
        object_id: None,
        transform: TransformAsset::default(),
        mesh_renderer: None,
        sprite_renderer: Some(SpriteRendererAsset { sprite: None }),
        tilemap: None,
        camera: None,
        active_camera: false,
        audio_source: None,
        physics_collision: None,
    }
}

fn tilemap_object_asset() -> WorldObjectAsset {
    WorldObjectAsset {
        name: "Tilemap".to_string(),
        object_id: None,
        transform: TransformAsset::default(),
        mesh_renderer: None,
        sprite_renderer: None,
        tilemap: Some(TilemapAsset {
            width: 16,
            height: 16,
            tile_size: [32, 32],
            offset: [-8, -8],
            layers: vec![TilemapLayerAsset {
                name: "Base".to_string(),
                visible: true,
                opacity: 1.0,
            }],
        }),
        camera: None,
        active_camera: false,
        audio_source: None,
        physics_collision: None,
    }
}

fn audio_source_object_asset() -> WorldObjectAsset {
    WorldObjectAsset {
        name: "Audio Source".to_string(),
        object_id: None,
        transform: TransformAsset::default(),
        mesh_renderer: None,
        sprite_renderer: None,
        tilemap: None,
        camera: None,
        active_camera: false,
        audio_source: Some(AudioSourceAsset {
            source: None,
            volume: 1.0,
            looped: false,
            play_on_awake: false,
            spatial: false,
            min_distance: 1.0,
            max_distance: 100.0,
        }),
        physics_collision: None,
    }
}
