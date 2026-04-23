use std::fs;
use std::path::{Path, PathBuf};

use crate::project::{ProjectAppConfig, ProjectBuildConfig, ProjectError, ProjectManifest, ProjectPaths};
use crate::world_asset::{create_empty_world, save_world};

pub fn create_empty_project(
    destination: &Path,
    project_name: &str,
) -> Result<ProjectPaths, ProjectError> {
    let root_dir = destination.to_path_buf();
    let binary_name = sanitize_binary_name(project_name);
    let manifest_file_name = format!("{binary_name}.runaproj");
    let manifest_path = root_dir.join(&manifest_file_name);
    let worlds_dir = root_dir.join("worlds");
    let assets_dir = root_dir.join("assets");
    let src_dir = root_dir.join("src");
    let startup_world_path = worlds_dir.join("main.world.ron");

    fs::create_dir_all(&root_dir)?;
    fs::create_dir_all(&worlds_dir)?;
    fs::create_dir_all(&assets_dir)?;
    fs::create_dir_all(&src_dir)?;
    fs::create_dir_all(root_dir.join(".proj"))?;

    let manifest = ProjectManifest {
        name: project_name.to_string(),
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
        startup_world: normalize_relative_path(Path::new("worlds").join("main.world.ron")),
        assets_dir: "assets".to_string(),
        worlds_dir: "worlds".to_string(),
        scripts_dir: "src".to_string(),
        binary_name: binary_name.clone(),
        app: ProjectAppConfig {
            window_title: project_name.to_string(),
            ..ProjectAppConfig::default()
        },
        build: ProjectBuildConfig::default(),
    };

    let project = ProjectPaths {
        manifest_path,
        root_dir: root_dir.clone(),
        manifest,
    };
    project.save_manifest()?;

    save_world(&startup_world_path, &create_empty_world())?;

    fs::write(root_dir.join(".gitignore"), default_gitignore())?;
    fs::write(root_dir.join("src").join("main.rs"), main_rs_template())?;
    fs::write(
        root_dir.join("Cargo.toml"),
        cargo_toml_template(project_name, &binary_name),
    )?;
    ensure_editor_bridge_files(&root_dir)?;
    ensure_release_windows_subsystem(&root_dir, true)?;

    Ok(project)
}

pub fn ensure_editor_bridge_files(project_root: &Path) -> Result<(), ProjectError> {
    let proj_dir = project_root.join(".proj");
    fs::create_dir_all(&proj_dir)?;
    ensure_project_dependencies(project_root)?;
    ensure_public_register_game_types(project_root)?;
    ensure_project_uses_manifest_app_config(project_root)?;

    let place_objects_path = proj_dir.join("place_objects.rs");
    fs::write(
        &place_objects_path,
        generate_place_objects_rs(project_root)?,
    )?;

    let bridge_path = proj_dir.join("runa_object_bridge.rs");
    fs::write(&bridge_path, object_bridge_rs_template())?;

    Ok(())
}

pub fn ensure_release_windows_subsystem(
    project_root: &Path,
    enabled: bool,
) -> Result<(), ProjectError> {
    const ATTR: &str = "#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = \"windows\")]";

    let main_rs_path = project_root.join("src").join("main.rs");
    if !main_rs_path.exists() {
        return Ok(());
    }

    let mut content = fs::read_to_string(&main_rs_path)?;
    let has_attr = content.lines().next().map(|line| line.trim()) == Some(ATTR);

    if enabled && !has_attr {
        content = format!("{ATTR}\n\n{content}");
    } else if !enabled && has_attr {
        content = content.replacen(&format!("{ATTR}\n\n"), "", 1);
        content = content.replacen(&format!("{ATTR}\n"), "", 1);
        content = content.replacen(ATTR, "", 1);
    }

    fs::write(main_rs_path, content)?;
    Ok(())
}

fn ensure_public_register_game_types(project_root: &Path) -> Result<(), ProjectError> {
    let main_rs_path = project_root.join("src").join("main.rs");
    if !main_rs_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&main_rs_path)?;
    if content.contains("pub fn register_game_types(") || !content.contains("fn register_game_types(")
    {
        return Ok(());
    }

    let updated = content.replacen(
        "fn register_game_types(",
        "pub fn register_game_types(",
        1,
    );
    fs::write(main_rs_path, updated)?;
    Ok(())
}

fn ensure_project_uses_manifest_app_config(project_root: &Path) -> Result<(), ProjectError> {
    let main_rs_path = project_root.join("src").join("main.rs");
    if !main_rs_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&main_rs_path)?;
    let old_block = r#"    let config = RunaWindowConfig {
        title: project.manifest.name.clone(),
        ..RunaWindowConfig::default()
    };"#;
    let new_block = r#"    let config = RunaWindowConfig {
        title: project.manifest.app.window_title.clone(),
        width: project.manifest.app.width,
        height: project.manifest.app.height,
        fullscreen: project.manifest.app.fullscreen,
        vsync: project.manifest.app.vsync,
        show_fps_in_title: project.manifest.app.show_fps_in_title,
        window_icon: project.manifest.app.window_icon.clone(),
    };"#;

    let updated = content.replace(old_block, new_block);
    if updated != content {
        fs::write(main_rs_path, updated)?;
    }
    Ok(())
}

fn ensure_project_dependencies(project_root: &Path) -> Result<(), ProjectError> {
    let cargo_toml_path = project_root.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Ok(());
    }

    let mut content = fs::read_to_string(&cargo_toml_path)?;
    let mut changed = false;

    if !content.contains("\nron = ") && !content.starts_with("ron = ") {
        if let Some(index) = content.find("[dependencies]") {
            let insert_at = content[index..]
                .find('\n')
                .map(|offset| index + offset + 1)
                .unwrap_or(content.len());
            content.insert_str(insert_at, "ron = \"0.8\"\n");
            changed = true;
        }
    }

    if !content.contains("name = \"runa_object_bridge\"") {
        content.push_str(
            "\n[[bin]]\nname = \"runa_object_bridge\"\npath = \".proj/runa_object_bridge.rs\"\n",
        );
        changed = true;
    }

    if changed {
        fs::write(cargo_toml_path, content)?;
    }

    Ok(())
}

fn cargo_toml_template(project_name: &str, binary_name: &str) -> String {
    let engine_root = engine_root_dir();
    let runa_engine_path = normalize_absolute_path(engine_root.join("crates").join("runa_engine"));

    format!(
        r#"[package]
name = "{binary_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
ron = "0.8"
runa_engine = {{ path = "{runa_engine_path}" }}

"#,
    )
    .replace("{binary_name}", binary_name)
    .replace("{runa_engine_path}", &runa_engine_path)
    .replace("{project_name}", project_name)
}

fn generate_place_objects_rs(_project_root: &Path) -> Result<String, ProjectError> {
    let mut content = String::from(
        r#"use runa_engine::runa_project::{
    AudioSourceAsset, CameraAsset, MeshPrimitiveAsset, MeshRendererAsset,
    PhysicsCollisionAsset, PlaceableObjectDescriptor, PlaceableObjectRecord,
    SpriteRendererAsset, TilemapAsset, TilemapLayerAsset, TransformAsset, WorldObjectAsset,
};

"#,
    );

    content.push_str(
        r#"pub fn descriptors() -> Vec<PlaceableObjectDescriptor> {
    let mut objects = vec![
        descriptor("empty", "Empty", "Basic"),
        descriptor("camera", "Camera", "Basic"),
        descriptor("cube", "Cube", "Basic"),
        descriptor("floor", "Floor", "Basic"),
        descriptor("sprite", "Sprite", "2D"),
        descriptor("tilemap", "Tilemap", "2D"),
        descriptor("audio-source", "Audio Source", "Audio"),
    ];
"#,
    );
    content.push_str(
        r#"    objects
}

pub fn spawn(object_id: &str) -> Option<WorldObjectAsset> {
    match object_id {
        "empty" => Some(empty_object()),
        "camera" => Some(camera_object()),
        "cube" => Some(cube_object()),
        "floor" => Some(floor_object()),
        "sprite" => Some(sprite_object()),
        "tilemap" => Some(tilemap_object()),
        "audio-source" => Some(audio_source_object()),
"#,
    );
    content.push_str(
        r#"        _ => None,
    }
}

pub fn records() -> Vec<PlaceableObjectRecord> {
    descriptors()
        .into_iter()
        .filter_map(|descriptor| {
            spawn(&descriptor.id).map(|object| PlaceableObjectRecord { descriptor, object })
        })
        .collect()
}

fn descriptor(id: &str, name: &str, category: &str) -> PlaceableObjectDescriptor {
    PlaceableObjectDescriptor {
        id: id.to_string(),
        name: name.to_string(),
        category: category.to_string(),
    }
}

"#,
    );
    content.push_str(place_objects_body_template());
    Ok(content)
}

fn main_rs_template() -> &'static str {
    r#"#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::sync::Arc;

use runa_engine::{
    runa_app::{RunaApp, RunaWindowConfig},
    runa_project::{load_project, load_world_with_runtime_registry},
    Engine,
};

pub fn register_game_types(_engine: &mut Engine) {
    // Register your components, scripts, and archetypes here.
}

fn main() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let project = load_project(&current_dir).expect("Failed to load .runaproj manifest");

    let mut engine = Engine::new();
    register_game_types(&mut engine);

    let mut world = load_world_with_runtime_registry(
        project.startup_world_path(),
        engine.runtime_registry(),
    ).expect("Failed to load startup world");
    world.set_runtime_registry(Arc::new(engine.runtime_registry().clone()));

    let config = RunaWindowConfig {
        title: project.manifest.app.window_title.clone(),
        width: project.manifest.app.width,
        height: project.manifest.app.height,
        fullscreen: project.manifest.app.fullscreen,
        vsync: project.manifest.app.vsync,
        show_fps_in_title: project.manifest.app.show_fps_in_title,
        window_icon: project.manifest.app.window_icon.clone(),
    };

    RunaApp::run_with_config(world, config).expect("Failed to run project");
}
"#
}

fn object_bridge_rs_template() -> &'static str {
    r#"#[path = "../src/main.rs"]
mod game_main;

use runa_engine::{
    runa_core::registry::{RegisteredTypeKind, RegistrationSource},
    runa_project::{
        PlaceableObjectDescriptor, PlaceableObjectRecord, ProjectMetadataSnapshot,
        ProjectRegisteredTypeKind, ProjectRegisteredTypeRecord, ProjectRegistrationSource,
        WorldObjectAsset,
    },
    Engine,
};

fn project_metadata() -> ProjectMetadataSnapshot {
    let mut engine = Engine::new();
    game_main::register_game_types(&mut engine);

    let registered_types = engine
        .runtime_registry()
        .types()
        .registered_types()
        .into_iter()
        .map(|metadata| ProjectRegisteredTypeRecord {
            type_name: metadata.type_name().to_string(),
            kind: match metadata.kind() {
                RegisteredTypeKind::Component => ProjectRegisteredTypeKind::Component,
                RegisteredTypeKind::Script => ProjectRegisteredTypeKind::Script,
            },
            source: match metadata.source() {
                RegistrationSource::BuiltIn => ProjectRegistrationSource::BuiltIn,
                RegistrationSource::User => ProjectRegistrationSource::User,
            },
            editor_addable: engine
                .runtime_registry()
                .types()
                .has_object_factory(metadata.type_id()),
            default_fields: {
                let mut object = runa_engine::runa_core::ocs::Object::new("Editor Preview");
                if engine
                    .runtime_registry()
                    .add_type_to_object(&mut object, metadata.type_id())
                {
                    object
                        .with_component_by_type_id(metadata.type_id(), |component| {
                            component.serialized_fields()
                        })
                        .unwrap_or_default()
                } else {
                    Vec::new()
                }
            },
        })
        .collect();

    let object_records = engine
        .runtime_registry()
        .archetypes()
        .registered_user_archetypes()
        .into_iter()
        .filter_map(|archetype| {
            let mut world = engine.create_world();
            let object_id = engine.spawn_archetype_by_key(&mut world, archetype.key())?;
            let object = world.get(object_id)?;
            Some(PlaceableObjectRecord {
                descriptor: PlaceableObjectDescriptor {
                    id: archetype.key().as_str().to_string(),
                    name: archetype.name().to_string(),
                    category: "Archetypes".to_string(),
                },
                object: WorldObjectAsset::from_object(object),
            })
        })
        .collect();

    ProjectMetadataSnapshot {
        object_records,
        registered_types,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 && args[1] == "--project-metadata" {
        let content = ron::to_string(&project_metadata())
            .expect("Failed to serialize project metadata");
        println!("{content}");
        return;
    }

    panic!("runa_object_bridge expects --project-metadata");
}
"#
}

fn place_objects_body_template() -> &'static str {
    r#"
fn empty_object() -> WorldObjectAsset {
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
        serialized_components: Vec::new(),
        serialized_scripts: Vec::new(),
    }
}

fn camera_object() -> WorldObjectAsset {
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
        camera: Some(CameraAsset::default()),
        active_camera: true,
        audio_source: None,
        physics_collision: None,
        serialized_components: Vec::new(),
        serialized_scripts: Vec::new(),
    }
}

fn cube_object() -> WorldObjectAsset {
    WorldObjectAsset {
        name: "Cube".to_string(),
        object_id: None,
        transform: TransformAsset {
            position: [0.0, 0.75, 0.0],
            scale: [1.0, 1.0, 1.0],
            ..TransformAsset::default()
        },
        mesh_renderer: Some(MeshRendererAsset {
            primitive: MeshPrimitiveAsset::Cube { size: 1.5 },
            color: [0.95, 0.55, 0.22, 1.0],
        }),
        sprite_renderer: None,
        tilemap: None,
        camera: None,
        active_camera: false,
        audio_source: None,
        physics_collision: Some(PhysicsCollisionAsset {
            size: [0.75, 0.75],
            enabled: true,
        }),
        serialized_components: Vec::new(),
        serialized_scripts: Vec::new(),
    }
}

fn floor_object() -> WorldObjectAsset {
    WorldObjectAsset {
        name: "Floor".to_string(),
        object_id: None,
        transform: TransformAsset {
            position: [0.0, -1.5, 0.0],
            scale: [8.0, 0.2, 8.0],
            ..TransformAsset::default()
        },
        mesh_renderer: Some(MeshRendererAsset {
            primitive: MeshPrimitiveAsset::Cube { size: 1.0 },
            color: [0.24, 0.27, 0.32, 1.0],
        }),
        sprite_renderer: None,
        tilemap: None,
        camera: None,
        active_camera: false,
        audio_source: None,
        physics_collision: Some(PhysicsCollisionAsset {
            size: [4.0, 4.0],
            enabled: true,
        }),
        serialized_components: Vec::new(),
        serialized_scripts: Vec::new(),
    }
}

fn sprite_object() -> WorldObjectAsset {
    WorldObjectAsset {
        name: "Sprite".to_string(),
        object_id: None,
        transform: TransformAsset {
            position: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            ..TransformAsset::default()
        },
        mesh_renderer: None,
        sprite_renderer: Some(SpriteRendererAsset {
            sprite: None,
            pixels_per_unit: 16.0,
        }),
        tilemap: None,
        camera: None,
        active_camera: false,
        audio_source: None,
        physics_collision: None,
        serialized_components: Vec::new(),
        serialized_scripts: Vec::new(),
    }
}

fn tilemap_object() -> WorldObjectAsset {
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
        serialized_components: Vec::new(),
        serialized_scripts: Vec::new(),
    }
}

fn audio_source_object() -> WorldObjectAsset {
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
        serialized_components: Vec::new(),
        serialized_scripts: Vec::new(),
    }
}
"#
}

fn default_gitignore() -> &'static str {
    "target/\n"
}

fn sanitize_binary_name(name: &str) -> String {
    let mut result = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            result.push(ch.to_ascii_lowercase());
        } else if matches!(ch, ' ' | '-' | '_') {
            if !result.ends_with('_') {
                result.push('_');
            }
        }
    }

    let trimmed = result.trim_matches('_');
    if trimmed.is_empty() {
        "runa_game".to_string()
    } else {
        trimmed.to_string()
    }
}

fn engine_root_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("runa_project must live in crates/runa_project")
        .to_path_buf()
}

fn normalize_absolute_path(path: PathBuf) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn normalize_relative_path(path: PathBuf) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::{load_project, load_world};

    use super::{create_empty_project, ensure_editor_bridge_files};

    #[test]
    fn creates_loadable_project_scaffold() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("runa_project_test_{unique}"));

        let project = create_empty_project(&root, "Example Project").unwrap();
        ensure_editor_bridge_files(&root).unwrap();
        let loaded = load_project(&project.manifest_path).unwrap();
        let world = load_world(loaded.startup_world_path()).unwrap();

        assert!(loaded.manifest_path.exists());
        assert!(loaded.root_dir.join("Cargo.toml").exists());
        assert!(loaded.root_dir.join("src").join("main.rs").exists());
        assert!(loaded
            .root_dir
            .join(".proj")
            .join("place_objects.rs")
            .exists());
        assert!(loaded
            .root_dir
            .join(".proj")
            .join("runa_object_bridge.rs")
            .exists());
        assert!(loaded.startup_world_path().exists());
        assert_eq!(loaded.manifest.name, "Example Project");
        assert_eq!(loaded.manifest.binary_name, "example_project");
        assert_eq!(world.query::<runa_core::components::Transform>().len(), 1);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn bridge_setup_backfills_ron_dependency() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("runa_project_migration_test_{unique}"));
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nruna_app = { path = \"x\" }\nruna_project = { path = \"y\" }\n",
        )
        .unwrap();

        ensure_editor_bridge_files(&root).unwrap();

        let cargo_toml = fs::read_to_string(root.join("Cargo.toml")).unwrap();
        assert!(cargo_toml.contains("ron = \"0.8\""));
        assert!(cargo_toml.contains("name = \"runa_object_bridge\""));
        assert!(root.join(".proj").join("place_objects.rs").exists());
        assert!(root.join(".proj").join("runa_object_bridge.rs").exists());

        let _ = fs::remove_dir_all(root);
    }
}
