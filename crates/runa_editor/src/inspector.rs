use std::any::TypeId;
use std::path::Path;

use egui::{Color32, RichText, Ui};
use rfd::FileDialog;
use runa_asset::loader::load_image;
use runa_asset::AudioAsset;
use runa_core::components::{
    BuiltinMeshPrimitive, Canvas, Collider2D, ComponentRuntimeKind,
    ActiveCamera, AudioListener, AudioSource, Camera, CursorInteractable, MeshRenderer,
    PhysicsCollision, ProjectionType, SerializedField, SerializedFieldValue, SpriteRenderer,
    SerializedTypeKind, SerializedTypeStorage, Tilemap, TilemapRenderer, Transform,
};
use runa_core::glam::{EulerRot, Quat, Vec3};
use runa_core::ocs::Object;

use crate::editor_textures::load_component_icon;
use crate::style;

#[derive(Debug, Default)]
pub struct InspectorActions {
    pub removals: Vec<InspectorRemoval>,
}

#[derive(Debug, Clone)]
pub struct InspectorRemoval {
    pub target: InspectorRemovalTarget,
}

#[derive(Debug, Clone)]
pub enum InspectorRemovalTarget {
    RuntimeType { type_id: TypeId, type_name: String },
    SerializedType { kind: SerializedTypeKind, type_name: String },
}

pub fn inspector_ui(
    ui: &mut Ui,
    object: &mut Object,
    project_root: Option<&Path>,
) -> InspectorActions {
    let mut actions = InspectorActions::default();

    object_section(ui, object);
    ui.separator();
    transform_section(ui, object);
    ui.separator();
    components_section(ui, object, project_root, &mut actions);
    ui.separator();
    scripts_section(ui, object, &mut actions);

    actions
}

fn object_section(ui: &mut Ui, object: &mut Object) {
    ui.heading("Object");
    let infos = object.component_infos();
    let runtime_component_count = infos
        .iter()
        .filter(|info| info.kind() == ComponentRuntimeKind::Component)
        .count();
    let runtime_script_count = infos
        .iter()
        .filter(|info| info.kind() == ComponentRuntimeKind::Script)
        .count();
    let (serialized_component_count, serialized_script_count) = object
        .get_component::<SerializedTypeStorage>()
        .map(|storage| {
            (
                storage.entries_of_kind(SerializedTypeKind::Component).count(),
                storage.entries_of_kind(SerializedTypeKind::Script).count(),
            )
        })
        .unwrap_or((0, 0));

    property_row(ui, "Id", |ui| {
        ui.label(
            object
                .id()
                .map(|id| id.to_string())
                .unwrap_or_else(|| "Unassigned".to_string()),
        );
    });
    property_row(ui, "Name", |ui| {
        ui.add_sized(
            [ui.available_width().max(120.0), 22.0],
            egui::TextEdit::singleline(&mut object.name),
        );
    });
    property_row(ui, "Components", |ui| {
        ui.label((runtime_component_count + serialized_component_count).to_string());
    });
    property_row(ui, "Scripts", |ui| {
        ui.label((runtime_script_count + serialized_script_count).to_string());
    });
}

fn transform_section(ui: &mut Ui, object: &mut Object) {
    if let Some(transform) = object.get_component_mut::<Transform>() {
        component_card(
            ui,
            TypeId::of::<Transform>(),
            ComponentRuntimeKind::Component,
            "Transform",
            false,
            None,
            |ui| {
                vec3_editor(ui, "Position", &mut transform.position);
                quat_editor(ui, transform);
                vec3_editor(ui, "Scale", &mut transform.scale);
            },
        );
    }
}

fn components_section(
    ui: &mut Ui,
    object: &mut Object,
    project_root: Option<&Path>,
    actions: &mut InspectorActions,
) {
    ui.heading("Components");

    let mut component_infos: Vec<_> = object
        .component_infos()
        .into_iter()
        .filter(|info| {
            info.kind() == ComponentRuntimeKind::Component
                && info.type_id() != TypeId::of::<Transform>()
                && info.type_id() != TypeId::of::<Tilemap>()
        })
        .collect();
    component_infos.sort_by(|left, right| left.type_name().cmp(right.type_name()));

    let has_serialized_components = object
        .get_component::<SerializedTypeStorage>()
        .map(|storage| storage.entries_of_kind(SerializedTypeKind::Component).next().is_some())
        .unwrap_or(false);
    if component_infos.is_empty() && !has_serialized_components {
        ui.label("No extra components attached.");
    }

    if let Some(camera) = object.get_component_mut::<Camera>() {
        component_block(
            ui,
            "Camera",
            true,
            actions,
            TypeId::of::<Camera>(),
            ComponentRuntimeKind::Component,
            |ui| {
                let previous_projection = camera.projection;
                vec3_editor(ui, "Position", &mut camera.position);
                vec3_editor(ui, "Target", &mut camera.target);
                property_row(ui, "Projection", |ui| {
                    ui.selectable_value(
                        &mut camera.projection,
                        ProjectionType::Orthographic,
                        "Ortho",
                    );
                    ui.selectable_value(
                        &mut camera.projection,
                        ProjectionType::Perspective,
                        "Perspective",
                    );
                });
                if previous_projection != camera.projection
                    && camera.projection == ProjectionType::Orthographic
                    && camera.ortho_size.length_squared() <= f32::EPSILON
                {
                    camera.ortho_size = runa_core::glam::Vec2::new(320.0, 180.0);
                }
                if camera.projection == ProjectionType::Orthographic {
                    property_row(ui, "Ortho Size", |ui| {
                        ui.add_sized(
                            [78.0, 22.0],
                            egui::DragValue::new(&mut camera.ortho_size.x).speed(1.0),
                        );
                        ui.add_sized(
                            [78.0, 22.0],
                            egui::DragValue::new(&mut camera.ortho_size.y).speed(1.0),
                        );
                    });
                }
                property_row(ui, "FOV", |ui| {
                    let mut degrees = camera.fov.to_degrees();
                    let response = ui.add_enabled(
                        camera.projection == ProjectionType::Perspective,
                        egui::DragValue::new(&mut degrees).speed(0.25),
                    );
                    if response.changed() {
                        camera.fov = degrees.to_radians();
                    }
                });
                property_row(ui, "Near", |ui| {
                    ui.add_sized([96.0, 22.0], egui::DragValue::new(&mut camera.near).speed(0.01));
                });
                property_row(ui, "Far", |ui| {
                    ui.add_sized([96.0, 22.0], egui::DragValue::new(&mut camera.far).speed(1.0));
                });
            },
        );
    }

    if object.get_component::<ActiveCamera>().is_some() {
        component_block(
            ui,
            "ActiveCamera",
            true,
            actions,
            TypeId::of::<ActiveCamera>(),
            ComponentRuntimeKind::Component,
            |ui| {
                property_row(ui, "State", |ui| {
                    ui.label("Selected runtime camera");
                });
            },
        );
    }

    if let Some(mesh_renderer) = object.get_component_mut::<MeshRenderer>() {
        component_block(
            ui,
            "MeshRenderer",
            true,
            actions,
            TypeId::of::<MeshRenderer>(),
            ComponentRuntimeKind::Component,
            |ui| {
                property_row(ui, "Builtin Mesh", |ui| {
                    let mut primitive =
                        mesh_renderer.mesh.primitive_hint.unwrap_or(BuiltinMeshPrimitive::Cube);
                    egui::ComboBox::from_id_salt("mesh_renderer_builtin_mesh")
                        .selected_text(mesh_primitive_label(primitive))
                        .show_ui(ui, |ui| {
                            for candidate in [
                                BuiltinMeshPrimitive::Cube,
                                BuiltinMeshPrimitive::Quad,
                                BuiltinMeshPrimitive::Plane,
                                BuiltinMeshPrimitive::Pyramid,
                            ] {
                                ui.selectable_value(
                                    &mut primitive,
                                    candidate,
                                    mesh_primitive_label(candidate),
                                );
                            }
                        });

                    if primitive != mesh_renderer.mesh.primitive_hint.unwrap_or(primitive) {
                        let extents = mesh_extents(&mesh_renderer.mesh);
                        mesh_renderer.mesh = build_builtin_mesh_from_extents(primitive, extents);
                    }
                });
                property_row(ui, "Vertices", |ui| {
                    ui.label(mesh_renderer.mesh.vertices.len().to_string());
                });
                property_row(ui, "Indices", |ui| {
                    ui.label(mesh_renderer.mesh.indices.len().to_string());
                });
                color_editor(ui, "Tint", &mut mesh_renderer.color);
            },
        );
    }

    if let Some(sprite) = object.get_component_mut::<SpriteRenderer>() {
        component_block(
            ui,
            "SpriteRenderer",
            true,
            actions,
            TypeId::of::<SpriteRenderer>(),
            ComponentRuntimeKind::Component,
            |ui| {
                let mut error_message = None;
                editable_asset_path(ui, "Sprite", &mut sprite.texture_path);
                property_row(ui, "Actions", |ui| {
                    if ui.button("Choose PNG").clicked() {
                        if let Some(path) =
                            pick_asset_file(project_root, &["png", "jpg", "jpeg", "webp"])
                        {
                            match load_texture_from_path(project_root, &path) {
                                Ok(texture) => sprite.set_texture(Some(texture), Some(path)),
                                Err(error) => error_message = Some(error),
                            }
                        }
                    }
                    if ui.button("Clear").clicked() {
                        sprite.set_texture(None, None);
                    }
                });
                property_row(ui, "Texture", |ui| {
                    ui.label(if sprite.texture.is_some() {
                        "assigned"
                    } else {
                        "none"
                    });
                });
                property_row(ui, "Pixels Per Unit", |ui| {
                    let response = ui.add_sized(
                        [96.0, 22.0],
                        egui::DragValue::new(&mut sprite.pixels_per_unit).speed(0.25),
                    );
                    if response.changed() {
                        sprite.pixels_per_unit = sprite.pixels_per_unit.max(f32::EPSILON);
                    }
                });
                if let Some(error) = error_message {
                    ui.colored_label(style::ERROR_COLOR, error);
                }
            },
        );
    }

    let has_tilemap_renderer = object.get_component::<TilemapRenderer>().is_some();
    if has_tilemap_renderer {
        if let Some(tilemap) = object.get_component_mut::<Tilemap>() {
            component_block(
                ui,
                "TilemapRenderer",
                true,
                actions,
                TypeId::of::<TilemapRenderer>(),
                ComponentRuntimeKind::Component,
                |ui| {
                    drag_u32(ui, "Width", &mut tilemap.width, 1.0);
                    drag_u32(ui, "Height", &mut tilemap.height, 1.0);
                    property_row(ui, "Tile Size", |ui| {
                        let mut x = tilemap.tile_size.x as u32;
                        let mut y = tilemap.tile_size.y as u32;
                        let x_changed = ui
                            .add_sized([78.0, 22.0], egui::DragValue::new(&mut x).range(1..=4096))
                            .changed();
                        let y_changed = ui
                            .add_sized([78.0, 22.0], egui::DragValue::new(&mut y).range(1..=4096))
                            .changed();
                        if x_changed || y_changed {
                            tilemap.tile_size.x = x as usize;
                            tilemap.tile_size.y = y as usize;
                        }
                    });
                    property_row(ui, "Offset", |ui| {
                        ui.add_sized(
                            [78.0, 22.0],
                            egui::DragValue::new(&mut tilemap.offset.x).speed(1.0),
                        );
                        ui.add_sized(
                            [78.0, 22.0],
                            egui::DragValue::new(&mut tilemap.offset.y).speed(1.0),
                        );
                    });
                    property_row(ui, "Layers", |ui| {
                        ui.label(tilemap.layers.len().to_string());
                        ui.separator();
                        if ui.button("Add Layer").clicked() {
                            let name = format!("Layer {}", tilemap.layers.len() + 1);
                            tilemap
                                .layers
                                .push(runa_core::components::TilemapLayer::new(
                                    name,
                                    tilemap.width,
                                    tilemap.height,
                                ));
                        }
                    });
                    for layer in &mut tilemap.layers {
                        egui::CollapsingHeader::new(layer.name.clone())
                            .default_open(true)
                            .show(ui, |ui| {
                                property_row(ui, "Name", |ui| {
                                    ui.add_sized(
                                        [ui.available_width().max(120.0), 22.0],
                                        egui::TextEdit::singleline(&mut layer.name),
                                    );
                                });
                                property_row(ui, "Visible", |ui| {
                                    ui.checkbox(&mut layer.visible, "");
                                });
                                property_row(ui, "Opacity", |ui| {
                                    ui.add_sized(
                                        [96.0, 22.0],
                                        egui::DragValue::new(&mut layer.opacity)
                                            .range(0.0..=1.0)
                                            .speed(0.01),
                                    );
                                });
                            });
                    }
                },
            );
        }
    }

    if let Some(audio) = object.get_component_mut::<AudioSource>() {
        component_block(
            ui,
            "AudioSource",
            true,
            actions,
            TypeId::of::<AudioSource>(),
            ComponentRuntimeKind::Component,
            |ui| {
                let mut error_message = None;
                editable_asset_path(ui, "Source", &mut audio.source_path);
                property_row(ui, "Actions", |ui| {
                    if ui.button("Choose OGG").clicked() {
                        if let Some(path) = pick_asset_file(project_root, &["ogg"]) {
                            match load_audio_from_path(project_root, &path) {
                                Ok(asset) => {
                                    audio.set_asset_with_path(Some(asset), Some(path));
                                }
                                Err(error) => error_message = Some(error),
                            }
                        }
                    }
                    if ui.button("Clear").clicked() {
                        audio.set_asset_with_path(None, None);
                    }
                });
                property_row(ui, "Volume", |ui| {
                    ui.add(
                        egui::DragValue::new(&mut audio.volume)
                            .range(0.0..=1.0)
                            .speed(0.01),
                    );
                });
                bool_row(ui, "Looped", &mut audio.looped);
                bool_row(ui, "Play On Awake", &mut audio.play_on_awake);
                bool_row(ui, "Spatial", &mut audio.spatial);
                property_row(ui, "Min Distance", |ui| {
                    ui.add_sized(
                        [96.0, 22.0],
                        egui::DragValue::new(&mut audio.min_distance).speed(0.1),
                    );
                });
                property_row(ui, "Max Distance", |ui| {
                    ui.add_sized(
                        [96.0, 22.0],
                        egui::DragValue::new(&mut audio.max_distance).speed(0.1),
                    );
                });
                if let Some(error) = error_message {
                    ui.colored_label(style::ERROR_COLOR, error);
                }
            },
        );
    }

    if let Some(listener) = object.get_component::<AudioListener>() {
        component_block(
            ui,
            "AudioListener",
            true,
            actions,
            TypeId::of::<AudioListener>(),
            ComponentRuntimeKind::Component,
            |ui| {
                property_row(ui, "Active", |ui| {
                    ui.label(listener.active.to_string());
                });
                property_row(ui, "Volume", |ui| {
                    ui.label(format!("{:.2}", listener.volume));
                });
                property_row(ui, "Stereo Separation", |ui| {
                    ui.label(format!("{:.2}", listener.stereo_separation));
                });
            },
        );
    }

    if let Some(interactable) = object.get_component::<CursorInteractable>() {
        component_block(
            ui,
            "CursorInteractable",
            true,
            actions,
            TypeId::of::<CursorInteractable>(),
            ComponentRuntimeKind::Component,
            |ui| {
                property_row(ui, "Bounds", |ui| {
                    ui.label(format!(
                        "{:.2}, {:.2}, {:.2}",
                        interactable.bounds_size.x,
                        interactable.bounds_size.y,
                        interactable.bounds_size.z
                    ));
                });
                property_row(ui, "Hovered", |ui| {
                    ui.label(interactable.is_hovered.to_string());
                });
                property_row(ui, "Pressed", |ui| {
                    ui.label(interactable.is_pressed.to_string());
                });
            },
        );
    }

    if let Some(collision) = object.get_component_mut::<PhysicsCollision>() {
        component_block(
            ui,
            "PhysicsCollision",
            true,
            actions,
            TypeId::of::<PhysicsCollision>(),
            ComponentRuntimeKind::Component,
            |ui| {
                bool_row(ui, "Enabled", &mut collision.enabled);
                property_row(ui, "Size", |ui| {
                    ui.add_sized(
                        [78.0, 22.0],
                        egui::DragValue::new(&mut collision.size.x).speed(0.05),
                    );
                    ui.add_sized(
                        [78.0, 22.0],
                        egui::DragValue::new(&mut collision.size.y).speed(0.05),
                    );
                });
            },
        );
    }

    for info in component_infos {
        if is_supported_component_type(info.type_id()) {
            continue;
        }

        generic_serialized_component_block(
            ui,
            object,
            short_type_name(info.type_name()),
            true,
            actions,
            info.type_id(),
            ComponentRuntimeKind::Component,
        );
    }

    serialized_storage_entries_section(
        ui,
        object,
        actions,
        SerializedTypeKind::Component,
        ComponentRuntimeKind::Component,
    );
}

fn scripts_section(ui: &mut Ui, object: &mut Object, actions: &mut InspectorActions) {
    ui.heading("Scripts");

    let mut script_infos: Vec<_> = object
        .component_infos()
        .into_iter()
        .filter(|info| info.kind() == ComponentRuntimeKind::Script)
        .collect();
    script_infos.sort_by(|left, right| left.type_name().cmp(right.type_name()));

    let has_serialized_scripts = object
        .get_component::<SerializedTypeStorage>()
        .map(|storage| storage.entries_of_kind(SerializedTypeKind::Script).next().is_some())
        .unwrap_or(false);
    if script_infos.is_empty() && !has_serialized_scripts {
        ui.label("No scripts attached.");
    }

    for info in script_infos {
        generic_serialized_component_block(
            ui,
            object,
            short_type_name(info.type_name()),
            true,
            actions,
            info.type_id(),
            ComponentRuntimeKind::Script,
        );
    }

    serialized_storage_entries_section(
        ui,
        object,
        actions,
        SerializedTypeKind::Script,
        ComponentRuntimeKind::Script,
    );
}

fn serialized_storage_entries_section(
    ui: &mut Ui,
    object: &mut Object,
    actions: &mut InspectorActions,
    kind: SerializedTypeKind,
    runtime_kind: ComponentRuntimeKind,
) {
    let entries = object
        .get_component::<SerializedTypeStorage>()
        .map(|storage| {
            storage
                .entries_of_kind(kind)
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    for entry in entries {
        let title = short_type_name(&entry.type_name).to_string();
        let remove_id = egui::Id::new(("remove_serialized_component", kind as u8, entry.type_name.clone()));
        let icon_name = component_icon_name(TypeId::of::<SerializedTypeStorage>(), runtime_kind);
        let icon = load_component_icon(
            ui.ctx(),
            &format!("inspector_component_icon_{icon_name}"),
            icon_name,
        );
        let card_id = ui.make_persistent_id(("serialized_component_card", kind as u8, entry.type_name.clone()));
        let state =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), card_id, true);

        egui::Frame::group(ui.style()).show(ui, |ui| {
            state
                .show_header(ui, |ui| {
                    ui.add(
                        egui::Image::new(&icon)
                            .fit_to_exact_size(egui::vec2(18.0, 18.0))
                            .sense(egui::Sense::hover()),
                    );
                    ui.label(RichText::new(&title).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Delete").clicked() {
                            ui.memory_mut(|memory| memory.data.insert_temp(remove_id, true));
                        }
                    });
                })
                .body(|ui| {
                    ui.separator();
                    if let Some(storage) = object.get_component_mut::<SerializedTypeStorage>() {
                        if let Some(target) = storage
                            .entries
                            .iter_mut()
                            .find(|target| target.kind == kind && target.type_name == entry.type_name)
                        {
                            if target.fields.is_empty() {
                                ui.colored_label(Color32::GRAY, "No serialized inspector fields.");
                            } else {
                                for field in &mut target.fields {
                                    serialized_field_asset_row(ui, field);
                                }
                            }
                        }
                    }
                });
        });

        if ui
            .memory(|memory| memory.data.get_temp::<bool>(remove_id).unwrap_or(false))
        {
            actions.removals.push(InspectorRemoval {
                target: InspectorRemovalTarget::SerializedType {
                    kind,
                    type_name: entry.type_name.clone(),
                },
            });
            ui.memory_mut(|memory| memory.data.remove::<bool>(remove_id));
        }
    }
}

fn generic_serialized_component_block(
    ui: &mut Ui,
    object: &mut Object,
    title: &str,
    removable: bool,
    actions: &mut InspectorActions,
    type_id: TypeId,
    kind: ComponentRuntimeKind,
) {
    component_block(ui, title, removable, actions, type_id, kind, |ui| {
        let Some(fields) = object.with_component_mut_by_type_id(type_id, |component| {
            component.serialized_fields()
        }) else {
            ui.colored_label(style::ERROR_COLOR, "Component is no longer attached.");
            return;
        };

        if fields.is_empty() {
            ui.colored_label(Color32::GRAY, "No serialized inspector fields.");
            return;
        }

        for field in fields {
            serialized_field_row(ui, object, type_id, field);
        }
    });
}

fn serialized_field_row(ui: &mut Ui, object: &mut Object, type_id: TypeId, field: SerializedField) {
    match field.value {
        SerializedFieldValue::Bool(mut value) => {
            let mut changed = false;
            property_row(ui, &field.name, |ui| {
                changed = ui.checkbox(&mut value, "").changed();
            });
            if changed {
                let _ = object.with_component_mut_by_type_id(type_id, |component| {
                    component.set_serialized_field(&field.name, SerializedFieldValue::Bool(value))
                });
            }
        }
        SerializedFieldValue::I32(mut value) => {
            let mut changed = false;
            property_row(ui, &field.name, |ui| {
                changed = ui
                    .add_sized([96.0, 22.0], egui::DragValue::new(&mut value).speed(1.0))
                    .changed();
            });
            if changed {
                let _ = object.with_component_mut_by_type_id(type_id, |component| {
                    component.set_serialized_field(&field.name, SerializedFieldValue::I32(value))
                });
            }
        }
        SerializedFieldValue::I64(mut value) => {
            let mut changed = false;
            property_row(ui, &field.name, |ui| {
                changed = ui
                    .add_sized([96.0, 22.0], egui::DragValue::new(&mut value).speed(1.0))
                    .changed();
            });
            if changed {
                let _ = object.with_component_mut_by_type_id(type_id, |component| {
                    component.set_serialized_field(&field.name, SerializedFieldValue::I64(value))
                });
            }
        }
        SerializedFieldValue::U32(mut value) => {
            let mut changed = false;
            property_row(ui, &field.name, |ui| {
                changed = ui
                    .add_sized([96.0, 22.0], egui::DragValue::new(&mut value).speed(1.0))
                    .changed();
            });
            if changed {
                let _ = object.with_component_mut_by_type_id(type_id, |component| {
                    component.set_serialized_field(&field.name, SerializedFieldValue::U32(value))
                });
            }
        }
        SerializedFieldValue::U64(mut value) => {
            let mut changed = false;
            property_row(ui, &field.name, |ui| {
                changed = ui
                    .add_sized([96.0, 22.0], egui::DragValue::new(&mut value).speed(1.0))
                    .changed();
            });
            if changed {
                let _ = object.with_component_mut_by_type_id(type_id, |component| {
                    component.set_serialized_field(&field.name, SerializedFieldValue::U64(value))
                });
            }
        }
        SerializedFieldValue::F32(mut value) => {
            let mut changed = false;
            property_row(ui, &field.name, |ui| {
                changed = ui
                    .add_sized([96.0, 22.0], egui::DragValue::new(&mut value).speed(0.05))
                    .changed();
            });
            if changed {
                let _ = object.with_component_mut_by_type_id(type_id, |component| {
                    component.set_serialized_field(&field.name, SerializedFieldValue::F32(value))
                });
            }
        }
        SerializedFieldValue::F64(mut value) => {
            let mut changed = false;
            property_row(ui, &field.name, |ui| {
                changed = ui
                    .add_sized([96.0, 22.0], egui::DragValue::new(&mut value).speed(0.05))
                    .changed();
            });
            if changed {
                let _ = object.with_component_mut_by_type_id(type_id, |component| {
                    component.set_serialized_field(&field.name, SerializedFieldValue::F64(value))
                });
            }
        }
        SerializedFieldValue::String(mut value) => {
            let mut changed = false;
            property_row(ui, &field.name, |ui| {
                changed = ui
                    .add_sized(
                        [ui.available_width().max(120.0), 22.0],
                        egui::TextEdit::singleline(&mut value),
                    )
                    .changed();
            });
            if changed {
                let _ = object.with_component_mut_by_type_id(type_id, |component| {
                    component.set_serialized_field(&field.name, SerializedFieldValue::String(value))
                });
            }
        }
        SerializedFieldValue::Vec2(mut value) => {
            let mut changed = false;
            property_row(ui, &field.name, |ui| {
                changed |= axis_drag(ui, "X", &mut value[0], 0.05);
                changed |= axis_drag(ui, "Y", &mut value[1], 0.05);
            });
            if changed {
                let _ = object.with_component_mut_by_type_id(type_id, |component| {
                    component.set_serialized_field(&field.name, SerializedFieldValue::Vec2(value))
                });
            }
        }
        SerializedFieldValue::Vec3(mut value) => {
            let mut changed = false;
            property_row(ui, &field.name, |ui| {
                changed |= axis_drag(ui, "X", &mut value[0], 0.05);
                changed |= axis_drag(ui, "Y", &mut value[1], 0.05);
                changed |= axis_drag(ui, "Z", &mut value[2], 0.05);
            });
            if changed {
                let _ = object.with_component_mut_by_type_id(type_id, |component| {
                    component.set_serialized_field(&field.name, SerializedFieldValue::Vec3(value))
                });
            }
        }
    }
}

fn serialized_field_asset_row(ui: &mut Ui, field: &mut SerializedField) {
    match &mut field.value {
        SerializedFieldValue::Bool(value) => {
            property_row(ui, &field.name, |ui| {
                ui.checkbox(value, "");
            });
        }
        SerializedFieldValue::I32(value) => {
            property_row(ui, &field.name, |ui| {
                ui.add_sized([96.0, 22.0], egui::DragValue::new(value).speed(1.0));
            });
        }
        SerializedFieldValue::I64(value) => {
            property_row(ui, &field.name, |ui| {
                ui.add_sized([96.0, 22.0], egui::DragValue::new(value).speed(1.0));
            });
        }
        SerializedFieldValue::U32(value) => {
            property_row(ui, &field.name, |ui| {
                ui.add_sized([96.0, 22.0], egui::DragValue::new(value).speed(1.0));
            });
        }
        SerializedFieldValue::U64(value) => {
            property_row(ui, &field.name, |ui| {
                ui.add_sized([96.0, 22.0], egui::DragValue::new(value).speed(1.0));
            });
        }
        SerializedFieldValue::F32(value) => {
            property_row(ui, &field.name, |ui| {
                ui.add_sized([96.0, 22.0], egui::DragValue::new(value).speed(0.05));
            });
        }
        SerializedFieldValue::F64(value) => {
            property_row(ui, &field.name, |ui| {
                ui.add_sized([96.0, 22.0], egui::DragValue::new(value).speed(0.05));
            });
        }
        SerializedFieldValue::String(value) => {
            property_row(ui, &field.name, |ui| {
                ui.add_sized(
                    [ui.available_width().max(120.0), 22.0],
                    egui::TextEdit::singleline(value),
                );
            });
        }
        SerializedFieldValue::Vec2(value) => {
            property_row(ui, &field.name, |ui| {
                axis_drag(ui, "X", &mut value[0], 0.05);
                axis_drag(ui, "Y", &mut value[1], 0.05);
            });
        }
        SerializedFieldValue::Vec3(value) => {
            property_row(ui, &field.name, |ui| {
                axis_drag(ui, "X", &mut value[0], 0.05);
                axis_drag(ui, "Y", &mut value[1], 0.05);
                axis_drag(ui, "Z", &mut value[2], 0.05);
            });
        }
    }
}

fn vec3_editor(ui: &mut Ui, label: &str, value: &mut Vec3) {
    property_row(ui, label, |ui| {
        axis_drag(ui, "X", &mut value.x, 0.05);
        axis_drag(ui, "Y", &mut value.y, 0.05);
        axis_drag(ui, "Z", &mut value.z, 0.05);
    });
}

fn quat_editor(ui: &mut Ui, transform: &mut Transform) {
    let (mut x, mut y, mut z) = transform.rotation.to_euler(EulerRot::XYZ);
    x = x.to_degrees();
    y = y.to_degrees();
    z = z.to_degrees();

    let mut changed = false;
    property_row(ui, "Rotation", |ui| {
        changed |= axis_drag(ui, "X", &mut x, 0.5);
        changed |= axis_drag(ui, "Y", &mut y, 0.5);
        changed |= axis_drag(ui, "Z", &mut z, 0.5);
    });

    if changed {
        transform.rotation = Quat::from_euler(
            EulerRot::XYZ,
            x.to_radians(),
            y.to_radians(),
            z.to_radians(),
        );
    }
}

fn color_editor(ui: &mut Ui, label: &str, color: &mut [f32; 4]) {
    property_row(ui, label, |ui| {
        for channel in color.iter_mut() {
            ui.add_sized(
                [70.0, 22.0],
                egui::DragValue::new(channel).range(0.0..=1.0).speed(0.01),
            );
        }
    });
}

fn component_badge(ui: &mut Ui, label: &str, description: &str) {
    ui.group(|ui| {
        ui.label(RichText::new(label).strong());
        ui.label(description);
    });
}

fn component_block(
    ui: &mut Ui,
    title: &str,
    removable: bool,
    actions: &mut InspectorActions,
    type_id: TypeId,
    kind: ComponentRuntimeKind,
    body: impl FnOnce(&mut Ui),
) {
    let removal = if removable {
        Some((type_id, title.to_string()))
    } else {
        None
    };
    component_card(ui, type_id, kind, title, removable, removal, body);
    if removable {
        if ui.memory(|memory| memory.data.get_temp::<bool>(egui::Id::new(("remove_component", type_id))).unwrap_or(false)) {
            actions.removals.push(InspectorRemoval {
                target: InspectorRemovalTarget::RuntimeType {
                    type_id,
                    type_name: title.to_string(),
                },
            });
            ui.memory_mut(|memory| memory.data.remove::<bool>(egui::Id::new(("remove_component", type_id))));
        }
    }
}

fn component_card(
    ui: &mut Ui,
    type_id: TypeId,
    kind: ComponentRuntimeKind,
    title: &str,
    removable: bool,
    removal: Option<(TypeId, String)>,
    body: impl FnOnce(&mut Ui),
) {
    let icon_name = component_icon_name(type_id, kind);
    let icon = load_component_icon(
        ui.ctx(),
        &format!("inspector_component_icon_{icon_name}"),
        icon_name,
    );
    let card_id = ui.make_persistent_id(("component_card", type_id));
    let state =
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), card_id, true);

    let frame = egui::Frame {
        fill: style::COMPONENT_BACKGROUND,
        ..egui::Frame::group(ui.style())
    };
    frame.show(ui, |ui| {
        state
            .show_header(ui, |ui| {
                ui.add(
                    egui::Image::new(&icon)
                        .fit_to_exact_size(egui::vec2(style::spacing::COMPONENT_ICON_SIZE, style::spacing::COMPONENT_ICON_SIZE))
                        .sense(egui::Sense::hover()),
                );
                ui.label(RichText::new(title).text_style(egui::TextStyle::Name("component_title".into())).strong().color(style::COMPONENT_TITLE_COLOR));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if removable && ui.button("Delete").clicked() {
                        if let Some((remove_type_id, _)) = &removal {
                            ui.memory_mut(|memory| {
                                memory.data.insert_temp(
                                    egui::Id::new(("remove_component", *remove_type_id)),
                                    true,
                                );
                            });
                        }
                    }
                });
            })
            .body(|ui| {
                ui.separator();
                body(ui);
            });
    });
}

fn axis_drag(ui: &mut Ui, label: &str, value: &mut f32, speed: f64) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        changed |= ui
            .add_sized([78.0, 22.0], egui::DragValue::new(value).speed(speed))
            .changed();
    });
    changed
}

fn property_row(ui: &mut Ui, label: &str, body: impl FnOnce(&mut Ui)) {
    let width_id = egui::Id::new("inspector_property_label_width");
    let mut label_width = ui
        .ctx()
        .data_mut(|data| data.get_persisted::<f32>(width_id))
        .unwrap_or(120.0)
        .clamp(90.0, 280.0);

    ui.horizontal(|ui| {
        ui.add_sized([label_width, 22.0], egui::Label::new(label));
        let (drag_rect, drag_response) =
            ui.allocate_exact_size(egui::vec2(6.0, 22.0), egui::Sense::click_and_drag());
        if drag_response.hovered() || drag_response.dragged() {
            ui.ctx()
                .set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        }
        if drag_response.dragged() {
            label_width = (label_width + drag_response.drag_delta().x).clamp(90.0, 280.0);
            ui.ctx()
                .data_mut(|data| data.insert_persisted(width_id, label_width));
        }
        ui.painter().line_segment(
            [drag_rect.center_top(), drag_rect.center_bottom()],
            egui::Stroke::new(1.0, ui.visuals().widgets.inactive.bg_stroke.color),
        );
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.set_min_width(ui.available_width());
            body(ui);
        });
    });
}

fn bool_row(ui: &mut Ui, label: &str, value: &mut bool) {
    property_row(ui, label, |ui| {
        ui.checkbox(value, "");
    });
}

fn component_icon_name(type_id: TypeId, kind: ComponentRuntimeKind) -> &'static str {
    if type_id == TypeId::of::<Transform>() {
        "c-Transform"
    } else if type_id == TypeId::of::<Camera>() {
        "c-Camera"
    } else if type_id == TypeId::of::<ActiveCamera>() {
        "c-ActiveCamera"
    } else if type_id == TypeId::of::<AudioSource>() {
        "c-AudioSource"
    } else if type_id == TypeId::of::<AudioListener>() { 
        "c-AudioListener"
    } else if type_id == TypeId::of::<Collider2D>() {
        "c-Collider"
    } else if type_id == TypeId::of::<PhysicsCollision>() { 
        "c-PhysicsCollision"
    } else if type_id == TypeId::of::<CursorInteractable>() {
        "c-CursorInteractable"
    } else if  type_id == TypeId::of::<Canvas>() {
        "c-Canvas"
    } else if type_id == TypeId::of::<MeshRenderer>() {
        "c-MeshRenderer"
    } else if type_id == TypeId::of::<SpriteRenderer>() {
        "c-SpriteRenderer"
    } else if type_id == TypeId::of::<TilemapRenderer>() {
        "c-TilemapRenderer"
    } else if kind == ComponentRuntimeKind::Script {
        "c-Script"
    } else {
        "c-Object"
    }
}

fn is_supported_component_type(type_id: TypeId) -> bool {
    [
        TypeId::of::<Camera>(),
        TypeId::of::<ActiveCamera>(),
        TypeId::of::<MeshRenderer>(),
        TypeId::of::<SpriteRenderer>(),
        TypeId::of::<TilemapRenderer>(),
        TypeId::of::<AudioSource>(),
        TypeId::of::<AudioListener>(),
        TypeId::of::<CursorInteractable>(),
        TypeId::of::<PhysicsCollision>(),
    ]
    .contains(&type_id)
}

fn mesh_primitive_label(primitive: BuiltinMeshPrimitive) -> &'static str {
    match primitive {
        BuiltinMeshPrimitive::Cube => "Cube",
        BuiltinMeshPrimitive::Quad => "Quad",
        BuiltinMeshPrimitive::Plane => "Plane",
        BuiltinMeshPrimitive::Pyramid => "Pyramid",
    }
}

fn mesh_extents(mesh: &runa_core::components::Mesh) -> Vec3 {
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);
    for vertex in &mesh.vertices {
        let p = Vec3::from_array(vertex.position);
        min = min.min(p);
        max = max.max(p);
    }
    (max - min).abs().max(Vec3::splat(1.0))
}

fn build_builtin_mesh_from_extents(primitive: BuiltinMeshPrimitive, extents: Vec3) -> runa_core::components::Mesh {
    match primitive {
        BuiltinMeshPrimitive::Cube => runa_core::components::Mesh::cube(extents.x.max(extents.y).max(extents.z)),
        BuiltinMeshPrimitive::Quad => runa_core::components::Mesh::quad(extents.x.max(0.01), extents.y.max(0.01)),
        BuiltinMeshPrimitive::Plane => runa_core::components::Mesh::plane(extents.x.max(0.01), extents.z.max(0.01)),
        BuiltinMeshPrimitive::Pyramid => {
            runa_core::components::Mesh::pyramid(extents.x.max(0.01), extents.y.max(0.01), extents.z.max(0.01))
        }
    }
}

fn short_type_name(type_name: &str) -> &str {
    type_name.rsplit("::").next().unwrap_or(type_name)
}

fn drag_u32(ui: &mut Ui, label: &str, value: &mut u32, speed: f64) {
    property_row(ui, label, |ui| {
        ui.add_sized(
            [96.0, 22.0],
            egui::DragValue::new(value).range(1..=4096).speed(speed),
        );
    });
}

fn editable_asset_path(ui: &mut Ui, label: &str, path: &mut Option<String>) {
    let mut buffer = path.clone().unwrap_or_default();
    let mut changed = false;
    property_row(ui, label, |ui| {
        changed = ui
            .add_sized(
                [ui.available_width().max(120.0), 22.0],
                egui::TextEdit::singleline(&mut buffer),
            )
            .changed();
    });
    if changed {
        let trimmed = buffer.trim();
        *path = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        };
    }
}

fn pick_asset_file(project_root: Option<&Path>, extensions: &[&str]) -> Option<String> {
    let mut dialog = FileDialog::new();
    if let Some(project_root) = project_root {
        dialog = dialog.set_directory(project_root.join("assets"));
    }
    let selected = dialog.add_filter("Assets", extensions).pick_file()?;
    project_root
        .and_then(|root| selected.strip_prefix(root).ok().map(path_to_string))
        .or_else(|| Some(path_to_string(&selected)))
}

fn load_texture_from_path(
    project_root: Option<&Path>,
    relative_path: &str,
) -> Result<runa_asset::Handle<runa_asset::TextureAsset>, String> {
    let Some(project_root) = project_root else {
        return Err("Open a project before assigning sprite assets.".to_string());
    };
    Ok(load_image(
        project_root.to_string_lossy().as_ref(),
        relative_path,
    ))
}

fn load_audio_from_path(
    project_root: Option<&Path>,
    relative_path: &str,
) -> Result<std::sync::Arc<AudioAsset>, String> {
    let Some(project_root) = project_root else {
        return Err("Open a project before assigning audio assets.".to_string());
    };
    AudioAsset::from_file(project_root.to_string_lossy().as_ref(), relative_path)
        .map(std::sync::Arc::new)
        .map_err(|error| error.to_string())
}

fn path_to_string(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}
