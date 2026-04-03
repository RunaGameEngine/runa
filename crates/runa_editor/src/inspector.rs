use std::path::Path;

use egui::{RichText, Ui};
use rfd::FileDialog;
use runa_asset::loader::load_image;
use runa_asset::AudioAsset;
use runa_core::components::{
    ActiveCamera, AudioListener, AudioSource, Camera, CursorInteractable, MeshRenderer,
    PhysicsCollision, SpriteRenderer, Tilemap, Transform,
};
use runa_core::glam::{EulerRot, Quat, Vec3};
use runa_core::ocs::Object;

use crate::style;

pub fn inspector_ui(ui: &mut Ui, object: &mut Object, project_root: Option<&Path>) {
    ui.label("Name");
    ui.text_edit_singleline(&mut object.name);
    ui.separator();

    if let Some(transform) = object.get_component_mut::<Transform>() {
        egui::CollapsingHeader::new("Transform")
            .default_open(true)
            .show(ui, |ui| {
                vec3_editor(ui, "Position", &mut transform.position);
                quat_editor(ui, transform);
                vec3_editor(ui, "Scale", &mut transform.scale);
            });
    }

    if let Some(camera) = object.get_component_mut::<Camera>() {
        egui::CollapsingHeader::new("Camera")
            .default_open(true)
            .show(ui, |ui| {
                vec3_editor(ui, "Position", &mut camera.position);
                vec3_editor(ui, "Target", &mut camera.target);
                ui.horizontal(|ui| {
                    ui.label("FOV");
                    let mut degrees = camera.fov.to_degrees();
                    if ui
                        .add(egui::DragValue::new(&mut degrees).speed(0.25))
                        .changed()
                    {
                        camera.fov = degrees.to_radians();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Near");
                    ui.add(egui::DragValue::new(&mut camera.near).speed(0.01));
                    ui.label("Far");
                    ui.add(egui::DragValue::new(&mut camera.far).speed(1.0));
                });
            });
    }

    if object.get_component::<ActiveCamera>().is_some() {
        component_badge(ui, "ActiveCamera", "Selected runtime camera");
    }

    if let Some(mesh_renderer) = object.get_component_mut::<MeshRenderer>() {
        egui::CollapsingHeader::new("MeshRenderer")
            .default_open(true)
            .show(ui, |ui| {
                ui.label(format!("Vertices: {}", mesh_renderer.mesh.vertices.len()));
                ui.label(format!("Indices: {}", mesh_renderer.mesh.indices.len()));
                color_editor(ui, "Tint", &mut mesh_renderer.color);
            });
    }

    if let Some(sprite) = object.get_component_mut::<SpriteRenderer>() {
        egui::CollapsingHeader::new("SpriteRenderer")
            .default_open(true)
            .show(ui, |ui| {
                let mut error_message = None;
                editable_asset_path(ui, "Sprite", &mut sprite.texture_path);
                ui.horizontal(|ui| {
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
                ui.label(if sprite.texture.is_some() {
                    "Texture: assigned"
                } else {
                    "Texture: none"
                });
                if let Some(error) = error_message {
                    ui.colored_label(style::ERROR_COLOR, error);
                }
            });
    }

    if let Some(tilemap) = object.get_component_mut::<Tilemap>() {
        egui::CollapsingHeader::new("Tilemap")
            .default_open(true)
            .show(ui, |ui| {
                drag_u32(ui, "Width", &mut tilemap.width, 1.0);
                drag_u32(ui, "Height", &mut tilemap.height, 1.0);
                ui.horizontal(|ui| {
                    ui.label("Tile Size");
                    let mut x = tilemap.tile_size.x as u32;
                    let mut y = tilemap.tile_size.y as u32;
                    let x_changed = ui
                        .add(egui::DragValue::new(&mut x).range(1..=4096))
                        .changed();
                    let y_changed = ui
                        .add(egui::DragValue::new(&mut y).range(1..=4096))
                        .changed();
                    if x_changed || y_changed {
                        tilemap.tile_size.x = x as usize;
                        tilemap.tile_size.y = y as usize;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Offset");
                    ui.add(egui::DragValue::new(&mut tilemap.offset.x).speed(1.0));
                    ui.add(egui::DragValue::new(&mut tilemap.offset.y).speed(1.0));
                });
                ui.horizontal(|ui| {
                    ui.label(format!("Layers: {}", tilemap.layers.len()));
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
                            ui.text_edit_singleline(&mut layer.name);
                            ui.checkbox(&mut layer.visible, "Visible");
                            ui.horizontal(|ui| {
                                ui.label("Opacity");
                                ui.add(
                                    egui::DragValue::new(&mut layer.opacity)
                                        .range(0.0..=1.0)
                                        .speed(0.01),
                                );
                            });
                        });
                }
            });
    }

    if let Some(audio) = object.get_component_mut::<AudioSource>() {
        egui::CollapsingHeader::new("AudioSource")
            .default_open(true)
            .show(ui, |ui| {
                let mut error_message = None;
                editable_asset_path(ui, "Source", &mut audio.source_path);
                ui.horizontal(|ui| {
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
                ui.horizontal(|ui| {
                    ui.label("Volume");
                    ui.add(
                        egui::DragValue::new(&mut audio.volume)
                            .range(0.0..=1.0)
                            .speed(0.01),
                    );
                });
                ui.checkbox(&mut audio.looped, "Looped");
                ui.checkbox(&mut audio.play_on_awake, "Play On Awake");
                ui.checkbox(&mut audio.spatial, "Spatial");
                ui.horizontal(|ui| {
                    ui.label("Min Distance");
                    ui.add(egui::DragValue::new(&mut audio.min_distance).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("Max Distance");
                    ui.add(egui::DragValue::new(&mut audio.max_distance).speed(0.1));
                });
                if let Some(error) = error_message {
                    ui.colored_label(style::ERROR_COLOR, error);
                }
            });
    }

    if let Some(listener) = object.get_component::<AudioListener>() {
        egui::CollapsingHeader::new("AudioListener")
            .default_open(true)
            .show(ui, |ui| {
                ui.label(format!("Active: {}", listener.active));
                ui.label(format!("Volume: {:.2}", listener.volume));
                ui.label(format!(
                    "Stereo Separation: {:.2}",
                    listener.stereo_separation
                ));
            });
    }

    if let Some(interactable) = object.get_component::<CursorInteractable>() {
        egui::CollapsingHeader::new("CursorInteractable")
            .default_open(true)
            .show(ui, |ui| {
                ui.label(format!(
                    "Bounds: {:.2}, {:.2}, {:.2}",
                    interactable.bounds_size.x,
                    interactable.bounds_size.y,
                    interactable.bounds_size.z
                ));
                ui.label(format!("Hovered: {}", interactable.is_hovered));
                ui.label(format!("Pressed: {}", interactable.is_pressed));
            });
    }

    if let Some(collision) = object.get_component_mut::<PhysicsCollision>() {
        egui::CollapsingHeader::new("PhysicsCollision")
            .default_open(true)
            .show(ui, |ui| {
                ui.checkbox(&mut collision.enabled, "Enabled");
                ui.horizontal(|ui| {
                    ui.label("Size");
                    ui.add(egui::DragValue::new(&mut collision.size.x).speed(0.05));
                    ui.add(egui::DragValue::new(&mut collision.size.y).speed(0.05));
                });
            });
    }
}

fn vec3_editor(ui: &mut Ui, label: &str, value: &mut Vec3) {
    ui.label(label);
    ui.horizontal(|ui| {
        ui.add(egui::DragValue::new(&mut value.x).speed(0.05).prefix("x "));
        ui.add(egui::DragValue::new(&mut value.y).speed(0.05).prefix("y "));
        ui.add(egui::DragValue::new(&mut value.z).speed(0.05).prefix("z "));
    });
}

fn quat_editor(ui: &mut Ui, transform: &mut Transform) {
    let (mut x, mut y, mut z) = transform.rotation.to_euler(EulerRot::XYZ);
    x = x.to_degrees();
    y = y.to_degrees();
    z = z.to_degrees();

    ui.label("Rotation");
    let mut changed = false;
    ui.horizontal(|ui| {
        changed |= ui
            .add(egui::DragValue::new(&mut x).speed(0.5).prefix("x "))
            .changed();
        changed |= ui
            .add(egui::DragValue::new(&mut y).speed(0.5).prefix("y "))
            .changed();
        changed |= ui
            .add(egui::DragValue::new(&mut z).speed(0.5).prefix("z "))
            .changed();
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
    ui.label(label);
    ui.horizontal(|ui| {
        for channel in color.iter_mut() {
            ui.add(egui::DragValue::new(channel).range(0.0..=1.0).speed(0.01));
        }
    });
}

fn component_badge(ui: &mut Ui, label: &str, description: &str) {
    ui.group(|ui| {
        ui.label(RichText::new(label).strong());
        ui.label(description);
    });
}

fn drag_u32(ui: &mut Ui, label: &str, value: &mut u32, speed: f64) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::DragValue::new(value).range(1..=4096).speed(speed));
    });
}

fn editable_asset_path(ui: &mut Ui, label: &str, path: &mut Option<String>) {
    ui.label(label);
    let mut buffer = path.clone().unwrap_or_default();
    if ui.text_edit_singleline(&mut buffer).changed() {
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
