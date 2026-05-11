use super::*;

pub(super) fn latest_source_stamp(root: &PathBuf) -> Option<SystemTime> {
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

pub(super) fn latest_place_object_stamp(project: &ProjectPaths) -> Option<SystemTime> {
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

pub(super) fn query_project_metadata(
    project: &ProjectPaths,
    output_tx: &Sender<String>,
) -> Result<ProjectMetadataSnapshot, String> {
    let _ = output_tx.send("Refreshing project metadata...".to_string());

    let mut command = Command::new("cargo");
    command
        .args([
            "run",
            "--quiet",
            "--bin",
            "runa_object_bridge",
            "--",
            "--project-metadata",
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

pub(super) fn attach_child_output(
    child: &mut Child,
    output_tx: Sender<String>,
    prefix: &'static str,
) {
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

pub(super) fn configure_background_command(command: &mut Command) {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
}

pub(super) fn merge_placeable_object_records(
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
        parent: None,
        transform: TransformAsset::default(),
        mesh_renderer: None,
        sprite_renderer: None,
        sprite_animator: None,
        sorting: None,
        tilemap: None,
        camera: None,
        active_camera: false,
        audio_source: None,
        collider2d: None,
        physics_collision: None,
        serialized_components: Vec::new(),
        serialized_scripts: Vec::new(),
    }
}

fn camera_object_asset() -> WorldObjectAsset {
    WorldObjectAsset {
        name: "Camera".to_string(),
        object_id: None,
        parent: None,
        transform: TransformAsset {
            position: [0.0, 2.0, 6.0],
            ..TransformAsset::default()
        },
        mesh_renderer: None,
        sprite_renderer: None,
        sprite_animator: None,
        sorting: None,
        tilemap: None,
        camera: Some(runa_project::CameraAsset::default()),
        active_camera: true,
        audio_source: None,
        collider2d: None,
        physics_collision: None,
        serialized_components: Vec::new(),
        serialized_scripts: Vec::new(),
    }
}

fn cube_object_asset() -> WorldObjectAsset {
    WorldObjectAsset {
        name: "Cube".to_string(),
        object_id: None,
        parent: None,
        transform: TransformAsset {
            position: [0.0, 0.75, 0.0],
            ..TransformAsset::default()
        },
        mesh_renderer: Some(runa_project::MeshRendererAsset {
            primitive: runa_project::MeshPrimitiveAsset::Cube { size: 1.5 },
            color: [0.95, 0.55, 0.22, 1.0],
        }),
        sprite_renderer: None,
        sprite_animator: None,
        sorting: None,
        tilemap: None,
        camera: None,
        active_camera: false,
        audio_source: None,
        collider2d: None,
        physics_collision: Some(runa_project::PhysicsCollisionAsset {
            size: [0.75, 0.75],
            enabled: true,
        }),
        serialized_components: Vec::new(),
        serialized_scripts: Vec::new(),
    }
}

fn floor_object_asset() -> WorldObjectAsset {
    WorldObjectAsset {
        name: "Floor".to_string(),
        object_id: None,
        parent: None,
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
        sprite_animator: None,
        sorting: None,
        tilemap: None,
        camera: None,
        active_camera: false,
        audio_source: None,
        collider2d: None,
        physics_collision: Some(runa_project::PhysicsCollisionAsset {
            size: [4.0, 4.0],
            enabled: true,
        }),
        serialized_components: Vec::new(),
        serialized_scripts: Vec::new(),
    }
}

fn sprite_object_asset() -> WorldObjectAsset {
    WorldObjectAsset {
        name: "Sprite".to_string(),
        object_id: None,
        parent: None,
        transform: TransformAsset::default(),
        mesh_renderer: None,
        sprite_renderer: Some(SpriteRendererAsset {
            sprite: None,
            pixels_per_unit: 16.0,
            uv_rect: [0.0, 0.0, 1.0, 1.0],
        }),
        sprite_animator: None,
        sorting: None,
        tilemap: None,
        camera: None,
        active_camera: false,
        audio_source: None,
        collider2d: None,
        physics_collision: None,
        serialized_components: Vec::new(),
        serialized_scripts: Vec::new(),
    }
}

fn tilemap_object_asset() -> WorldObjectAsset {
    WorldObjectAsset {
        name: "Tilemap".to_string(),
        object_id: None,
        parent: None,
        transform: TransformAsset::default(),
        mesh_renderer: None,
        sprite_renderer: None,
        sprite_animator: None,
        sorting: None,
        tilemap: Some(TilemapAsset {
            width: 16,
            height: 16,
            tile_size: [32, 32],
            offset: [-8, -8],
            pixels_per_unit: 16.0,
            atlas: None,
            selected_tile: 0,
            layers: vec![TilemapLayerAsset {
                name: "Base".to_string(),
                visible: true,
                opacity: 1.0,
                tiles: Vec::new(),
                self_order: 0,
            }],
        }),
        camera: None,
        active_camera: false,
        audio_source: None,
        collider2d: None,
        physics_collision: None,
        serialized_components: Vec::new(),
        serialized_scripts: Vec::new(),
    }
}

fn audio_source_object_asset() -> WorldObjectAsset {
    WorldObjectAsset {
        name: "Audio Source".to_string(),
        object_id: None,
        parent: None,
        transform: TransformAsset::default(),
        mesh_renderer: None,
        sprite_renderer: None,
        sprite_animator: None,
        sorting: None,
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
        collider2d: None,
        physics_collision: None,
        serialized_components: Vec::new(),
        serialized_scripts: Vec::new(),
    }
}
