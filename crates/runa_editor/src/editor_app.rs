mod app_handler;
mod helpers;
mod placeables;
mod project;
mod ui;
mod viewport;
mod world_ops;

use std::any::TypeId;
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
    ActiveCamera, AudioListener, AudioSource, Camera, Canvas, Collider2D, ComponentRuntimeKind,
    CursorInteractable, Mesh, MeshRenderer, ObjectDefinitionInstance, PhysicsCollision,
    SerializedTypeEntry, SerializedTypeKind, SerializedTypeStorage, SpriteRenderer, Tilemap,
    TilemapRenderer, Transform,
};
use runa_core::glam::{EulerRot, Quat, Vec2, Vec3};
use runa_core::ocs::{Object, ObjectId};
use runa_core::registry::{RegisteredTypeKind, RuntimeRegistry};
use runa_core::World;
use runa_engine::Engine;
use runa_project::{
    create_empty_project, create_empty_world, ensure_editor_bridge_files,
    ensure_release_windows_subsystem, find_project_manifest, load_project, load_world, save_world,
    AudioSourceAsset, PlaceableObjectDescriptor,
    PlaceableObjectRecord, ProjectMetadataSnapshot, ProjectPaths, ProjectRegisteredTypeKind,
    ProjectRegisteredTypeRecord, SpriteRendererAsset, TilemapAsset, TilemapLayerAsset,
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
    metadata: ProjectMetadataSnapshot,
}

struct ObjectClipboard {
    asset: WorldObjectAsset,
    cut_id: Option<ObjectId>,
}

struct PlaceObjectState {
    objects: Vec<PlaceableObjectDescriptor>,
    templates: HashMap<String, WorldObjectAsset>,
    registered_types: Vec<ProjectRegisteredTypeRecord>,
    source_stamp: Option<SystemTime>,
    pending_stamp: Option<SystemTime>,
    refresh_in_progress: bool,
    refresh_result: Option<Receiver<Result<ProjectMetadataSnapshot, String>>>,
}

impl Default for PlaceObjectState {
    fn default() -> Self {
        Self {
            objects: Vec::new(),
            templates: HashMap::new(),
            registered_types: Vec::new(),
            source_stamp: None,
            pending_stamp: None,
            refresh_in_progress: false,
            refresh_result: None,
        }
    }
}

#[derive(Clone, Copy)]
enum GizmoHandleKind {
    Translate,
    ScaleX,
    ScaleY,
    Rotate,
}

struct ViewportDragState {
    object_id: ObjectId,
    kind: GizmoHandleKind,
    offset: Vec2,
    start_position: Vec3,
    start_rotation_z: f32,
    start_pointer_angle: f32,
}

pub struct EditorApp<'window> {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer<'window>>,
    egui_state: Option<EguiWinitState>,
    egui_renderer: Option<EguiRenderer>,
    egui_ctx: egui::Context,
    runtime_engine: Engine,

    world: World,
    scene_queue: RenderQueue,
    selection: Option<ObjectId>,
    content_browser: ContentBrowserState,
    panels: PanelState,
    settings: EditorSettings,
    editor_settings_open: bool,
    project_settings_open: bool,
    build_settings_open: bool,
    project_dialog: ProjectDialogState,
    project_session: Option<ProjectSession>,
    startup_project_path: Option<PathBuf>,
    runtime_process: Option<Child>,
    build_process: Option<Child>,
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
    viewport_camera: Option<Camera>,
    gizmo_enabled: bool,
    show_component_icons: bool,
    show_viewport_grid: bool,
    snap_enabled: bool,
    snap_step: f32,
    gizmo_drag: Option<ViewportDragState>,
    modifiers: ModifiersState,
    bottom_tab: BottomTab,
    bottom_bar_height: f32,
    view_settings_open: bool,
    rendering_settings_open: bool,
    gizmo_settings_open: bool,

    status_line: String,
    last_frame_time: Instant,
}

impl<'window> EditorApp<'window> {
    fn world_object_ids(&self) -> Vec<ObjectId> {
        let mut ids = self.world.query::<Transform>();
        ids.sort_unstable();
        ids
    }

    fn first_object_id(&self) -> Option<ObjectId> {
        self.world.find_first_with::<Transform>()
    }

    fn new(project_path: Option<PathBuf>) -> Self {
        let startup_root = std::env::current_dir().unwrap_or_default();
        let (output_tx, output_rx) = mpsc::channel();
        let runtime_engine = Engine::new();
        let mut world = helpers::create_preview_world();
        world.set_runtime_registry(Arc::new(runtime_engine.runtime_registry().clone()));
        let selection = world.find_first_with::<Transform>();
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
            editor_settings_open: false,
            project_settings_open: false,
            build_settings_open: false,
            project_dialog,
            project_session: None,
            startup_project_path: project_path.or_else(|| find_project_manifest(&startup_root)),
            runtime_process: None,
            build_process: None,
            project_load: None,
            place_object: PlaceObjectState::default(),
            hierarchy_clipboard: None,
            window: None,
            renderer: None,
            egui_state: None,
            egui_renderer: None,
            egui_ctx: egui::Context::default(),
            runtime_engine,
            world,
            scene_queue: RenderQueue::new(),
            selection,
            content_browser: ContentBrowserState::new(startup_root),
            panels: PanelState::default(),
            editor_camera: EditorCameraController::new(),
            viewport_target: None,
            viewport_texture_id: None,
            pending_viewport_size: style::panel_sizes::INITIAL_VIEWPORT,
            viewport_hovered: false,
            viewport_camera: None,
            gizmo_enabled: true,
            show_component_icons: true,
            show_viewport_grid: true,
            snap_enabled: false,
            snap_step: 1.0,
            gizmo_drag: None,
            modifiers: ModifiersState::default(),
            bottom_tab: BottomTab::ContentBrowser,
            bottom_bar_height: style::panel_sizes::BOTTOM_BAR_HEIGHT,
            view_settings_open: false,
            rendering_settings_open: false,
            gizmo_settings_open: false,
            status_line:
                "Viewport: wheel zoom, middle-drag pan, left click select, drag handles to edit, F frame selected."
                    .to_string(),
            last_frame_time: Instant::now(),
            output_tx,
            output_rx,
        }
    }
}
