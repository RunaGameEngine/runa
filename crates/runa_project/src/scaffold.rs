use std::fs;
use std::path::{Path, PathBuf};

use crate::project::{ProjectError, ProjectManifest, ProjectPaths};
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

    Ok(project)
}

pub fn ensure_editor_bridge_files(project_root: &Path) -> Result<(), ProjectError> {
    let proj_dir = project_root.join(".proj");
    fs::create_dir_all(&proj_dir)?;
    ensure_project_dependencies(project_root)?;

    let place_objects_path = proj_dir.join("place_objects.rs");
    fs::write(
        &place_objects_path,
        generate_place_objects_rs(project_root)?,
    )?;

    let bridge_path = proj_dir.join("runa_object_bridge.rs");
    fs::write(&bridge_path, object_bridge_rs_template())?;

    Ok(())
}

#[derive(Clone)]
struct AutoObjectSpec {
    id: String,
    name: String,
    module_name: String,
    include_path: String,
    constructor_expr: String,
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

fn generate_place_objects_rs(project_root: &Path) -> Result<String, ProjectError> {
    let auto_objects = collect_auto_objects(&project_root.join("src"))?;
    let mut content = String::from(
        r#"use runa_engine::runa_project::{
    AudioSourceAsset, CameraAsset, MeshPrimitiveAsset, MeshRendererAsset,
    PhysicsCollisionAsset, PlaceableObjectDescriptor, PlaceableObjectRecord,
    SpriteRendererAsset, TilemapAsset, TilemapLayerAsset, TransformAsset, WorldObjectAsset,
};

"#,
    );

    for object in &auto_objects {
        content.push_str(&format!(
            r#"mod {module_name} {{
    use runa_engine::runa_core::ocs::{{Object, Script}};
    use runa_engine::runa_project::WorldObjectAsset;

    include!({include_path:?});

    pub fn spawn() -> Option<WorldObjectAsset> {{
        let mut object = Object::new();
        let script = {constructor_expr};
        script.construct(&mut object);
        let mut asset = WorldObjectAsset::from_object(&object);
        if asset.name.is_empty() {{
            asset.name = {name:?}.to_string();
        }}
        asset.object_id = Some({id:?}.to_string());
        Some(asset)
    }}
}}

"#,
            module_name = object.module_name,
            include_path = object.include_path,
            constructor_expr = object.constructor_expr,
            name = object.name,
            id = object.id,
        ));
    }

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
    for object in &auto_objects {
        content.push_str(&format!(
            "    objects.push(descriptor({id:?}, {name:?}, \"Scripts\"));\n",
            id = object.id,
            name = object.name
        ));
    }
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
    for object in &auto_objects {
        content.push_str(&format!(
            "        {id:?} => {module_name}::spawn(),\n",
            id = object.id,
            module_name = object.module_name
        ));
    }
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

fn collect_auto_objects(src_dir: &Path) -> Result<Vec<AutoObjectSpec>, ProjectError> {
    let mut objects = Vec::new();
    if !src_dir.exists() {
        return Ok(objects);
    }

    collect_auto_objects_recursive(src_dir, src_dir, &mut objects)?;
    Ok(objects)
}

fn collect_auto_objects_recursive(
    root: &Path,
    current: &Path,
    objects: &mut Vec<AutoObjectSpec>,
) -> Result<(), ProjectError> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().and_then(|value| value.to_str()) == Some("bin") {
                continue;
            }
            collect_auto_objects_recursive(root, &path, objects)?;
            continue;
        }

        if path.extension().and_then(|value| value.to_str()) != Some("rs") {
            continue;
        }

        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        if matches!(file_name, "main.rs" | "lib.rs") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let relative_path = path.strip_prefix(root).unwrap_or(&path);
        for type_name in collect_script_type_names(&content) {
            let Some(constructor_expr) = infer_constructor_expr(&content, &type_name) else {
                continue;
            };
            let path_stem = normalize_relative_path(relative_path.with_extension(""));
            let path_id = path_stem.replace('/', "-");
            objects.push(AutoObjectSpec {
                id: format!("script:{path_id}:{}", type_name.to_lowercase()),
                name: type_name.clone(),
                module_name: format!(
                    "auto_{}_{}",
                    sanitize_identifier(&path_id),
                    sanitize_identifier(&type_name.to_lowercase())
                ),
                include_path: normalize_relative_path(
                    Path::new("..").join("src").join(relative_path),
                ),
                constructor_expr: constructor_expr.replace("{type}", &type_name),
            });
        }
    }

    Ok(())
}

fn collect_script_type_names(content: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in content.lines() {
        let Some(after_impl) = line.split("impl Script for ").nth(1) else {
            continue;
        };
        let name: String = after_impl
            .chars()
            .skip_while(|ch| ch.is_whitespace())
            .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
            .collect();
        if !name.is_empty() && !names.contains(&name) {
            names.push(name);
        }
    }
    names
}

fn infer_constructor_expr(content: &str, type_name: &str) -> Option<String> {
    let impl_block = format!("impl {type_name}");
    if content.contains(&impl_block)
        && (content.contains("pub fn new() -> Self") || content.contains("fn new() -> Self"))
    {
        return Some("{type}::new()".to_string());
    }

    if content.contains(&format!("impl Default for {type_name}")) {
        return Some("{type}::default()".to_string());
    }

    None
}

fn sanitize_identifier(value: &str) -> String {
    let mut result = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            result.push(ch.to_ascii_lowercase());
        } else if !result.ends_with('_') {
            result.push('_');
        }
    }
    result.trim_matches('_').to_string()
}

fn main_rs_template() -> &'static str {
    r#"use runa_engine::runa_app::{RunaApp, RunaWindowConfig};
use runa_engine::runa_project::{load_project, load_world_with_object_loader};

#[path = "../.proj/place_objects.rs"]
mod place_objects;

fn main() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let project = load_project(&current_dir).expect("Failed to load .runaproj manifest");
    let world = load_world_with_object_loader(project.startup_world_path(), |object_id| {
        place_objects::spawn(object_id)
    })
    .expect("Failed to load startup world");

    let config = RunaWindowConfig {
        title: project.manifest.name.clone(),
        ..RunaWindowConfig::default()
    };

    RunaApp::run_with_config(world, config).expect("Failed to run project");
}
"#
}

fn object_bridge_rs_template() -> &'static str {
    r#"mod place_objects;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 && args[1] == "--list-objects" {
        let content = ron::to_string(&place_objects::descriptors())
            .expect("Failed to serialize placeable object descriptors");
        println!("{content}");
        return;
    }
    if args.len() >= 2 && args[1] == "--list-object-records" {
        let content = ron::to_string(&place_objects::records())
            .expect("Failed to serialize placeable object records");
        println!("{content}");
        return;
    }
    if args.len() >= 3 && args[1] == "--spawn-object" {
        let object = place_objects::spawn(&args[2]).expect("Unknown object id");
        let content = ron::to_string(&object).expect("Failed to serialize spawned object");
        println!("{content}");
        return;
    }

    panic!("runa_object_bridge expects --list-objects, --list-object-records, or --spawn-object <id>");
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
        sprite_renderer: Some(SpriteRendererAsset { sprite: None }),
        tilemap: None,
        camera: None,
        active_camera: false,
        audio_source: None,
        physics_collision: None,
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
        assert_eq!(world.objects.len(), 1);

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
