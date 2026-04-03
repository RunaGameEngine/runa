use std::fs;
use std::path::{Path, PathBuf};

use runa_asset::{AudioAsset, Handle, TextureAsset};
use runa_core::components::{
    ActiveCamera, AudioSource, Camera, Mesh, MeshRenderer, ObjectDefinitionInstance,
    PhysicsCollision, ProjectionType, SpriteRenderer, Tilemap, TilemapLayer, Transform,
};
use runa_core::glam::{IVec2, Quat, USizeVec2, Vec2, Vec3};
use runa_core::ocs::Object;
use runa_core::World;
use serde::{Deserialize, Serialize};

use crate::project::{find_project_manifest, load_project, ProjectError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldAsset {
    pub objects: Vec<WorldObjectAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldObjectAsset {
    pub name: String,
    pub object_id: Option<String>,
    pub transform: TransformAsset,
    pub mesh_renderer: Option<MeshRendererAsset>,
    pub sprite_renderer: Option<SpriteRendererAsset>,
    pub tilemap: Option<TilemapAsset>,
    pub camera: Option<CameraAsset>,
    pub active_camera: bool,
    pub audio_source: Option<AudioSourceAsset>,
    pub physics_collision: Option<PhysicsCollisionAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceableObjectDescriptor {
    pub id: String,
    pub name: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceableObjectRecord {
    pub descriptor: PlaceableObjectDescriptor,
    pub object: WorldObjectAsset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformAsset {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl Default for TransformAsset {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshRendererAsset {
    pub primitive: MeshPrimitiveAsset,
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpriteRendererAsset {
    pub sprite: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeshPrimitiveAsset {
    Cube { size: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraAsset {
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub up: [f32; 3],
    pub perspective: bool,
    pub ortho_size: [f32; 2],
    pub near: f32,
    pub far: f32,
    pub fov_radians: f32,
    pub viewport_size: [u32; 2],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSourceAsset {
    pub source: Option<String>,
    pub volume: f32,
    pub looped: bool,
    pub play_on_awake: bool,
    pub spatial: bool,
    pub min_distance: f32,
    pub max_distance: f32,
}

impl Default for CameraAsset {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 5.0],
            target: [0.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            perspective: true,
            ortho_size: [128.0, 72.0],
            near: 0.1,
            far: 1000.0,
            fov_radians: 75.0_f32.to_radians(),
            viewport_size: [1280, 720],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsCollisionAsset {
    pub size: [f32; 2],
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilemapAsset {
    pub width: u32,
    pub height: u32,
    pub tile_size: [u32; 2],
    pub offset: [i32; 2],
    pub layers: Vec<TilemapLayerAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilemapLayerAsset {
    pub name: String,
    pub visible: bool,
    pub opacity: f32,
}

pub fn create_empty_world() -> World {
    let mut world = World::default();

    let mut camera = Object::new();
    camera.name = "Main Camera".to_string();
    camera.add_component(Camera::default());
    camera.add_component(ActiveCamera);
    world.objects.push(camera);

    world
}

pub fn save_world(path: impl AsRef<Path>, world: &World) -> Result<(), ProjectError> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }

    let asset = WorldAsset::from_world(world);
    let content = ron::ser::to_string_pretty(&asset, ron::ser::PrettyConfig::default())?;
    fs::write(path, content)?;
    Ok(())
}

pub fn load_world(path: impl AsRef<Path>) -> Result<World, ProjectError> {
    let content = fs::read_to_string(path.as_ref())?;
    let asset: WorldAsset = ron::from_str(&content)?;
    let project_root = project_root_for_world_path(path.as_ref());
    Ok(asset.into_world_with_project_root(project_root.as_deref()))
}

pub fn load_world_with_object_loader<F>(
    path: impl AsRef<Path>,
    object_loader: F,
) -> Result<World, ProjectError>
where
    F: Fn(&str) -> Option<WorldObjectAsset>,
{
    let content = fs::read_to_string(path.as_ref())?;
    let asset: WorldAsset = ron::from_str(&content)?;
    let project_root = project_root_for_world_path(path.as_ref());
    Ok(asset.into_world_with_object_loader(project_root.as_deref(), object_loader))
}

impl WorldAsset {
    pub fn from_world(world: &World) -> Self {
        Self {
            objects: world
                .objects
                .iter()
                .map(WorldObjectAsset::from_object)
                .collect(),
        }
    }

    pub fn into_world(self) -> World {
        self.into_world_with_project_root(None)
    }

    pub fn into_world_with_project_root(self, project_root: Option<&Path>) -> World {
        let mut world = World::default();
        world.objects = self
            .objects
            .into_iter()
            .map(|object| object.into_object(project_root))
            .collect();
        world
    }

    pub fn into_world_with_object_loader<F>(
        self,
        project_root: Option<&Path>,
        object_loader: F,
    ) -> World
    where
        F: Fn(&str) -> Option<WorldObjectAsset>,
    {
        let mut world = World::default();
        world.objects = self
            .objects
            .into_iter()
            .map(|object| object.into_object_with_object_loader(project_root, &object_loader))
            .collect();
        world
    }
}

impl WorldObjectAsset {
    pub fn from_object(object: &Object) -> Self {
        let transform = object
            .get_component::<Transform>()
            .cloned()
            .unwrap_or_else(Transform::default);

        Self {
            name: object.name.clone(),
            object_id: object
                .get_component::<ObjectDefinitionInstance>()
                .map(|instance| instance.object_id.clone()),
            transform: TransformAsset::from_transform(&transform),
            mesh_renderer: object
                .get_component::<MeshRenderer>()
                .and_then(MeshRendererAsset::from_component),
            sprite_renderer: object
                .get_component::<SpriteRenderer>()
                .map(SpriteRendererAsset::from_component),
            tilemap: object
                .get_component::<Tilemap>()
                .map(TilemapAsset::from_component),
            camera: object
                .get_component::<Camera>()
                .map(CameraAsset::from_component),
            active_camera: object.get_component::<ActiveCamera>().is_some(),
            audio_source: object
                .get_component::<AudioSource>()
                .map(AudioSourceAsset::from_component),
            physics_collision: object
                .get_component::<PhysicsCollision>()
                .map(PhysicsCollisionAsset::from_component),
        }
    }

    pub fn into_object(self, project_root: Option<&Path>) -> Object {
        let object_id = self.object_id.clone();
        let mut object = Object::new();
        object.name = self.name;
        object.add_component(self.transform.into_component());

        if let Some(mesh_renderer) = self.mesh_renderer {
            object.add_component(mesh_renderer.into_component());
        }
        if let Some(sprite_renderer) = self.sprite_renderer {
            object.add_component(sprite_renderer.into_component(project_root));
        }
        if let Some(tilemap) = self.tilemap {
            object.add_component(tilemap.into_component());
        }
        if let Some(camera) = self.camera {
            object.add_component(camera.into_component());
        }
        if self.active_camera {
            object.add_component(ActiveCamera);
        }
        if let Some(audio_source) = self.audio_source {
            object.add_component(audio_source.into_component(project_root));
        }
        if let Some(physics_collision) = self.physics_collision {
            object.add_component(physics_collision.into_component());
        }
        if let Some(object_id) = object_id {
            object.add_component(ObjectDefinitionInstance::new(object_id));
        }

        object
    }

    pub fn into_object_with_object_loader<F>(
        self,
        project_root: Option<&Path>,
        object_loader: F,
    ) -> Object
    where
        F: Fn(&str) -> Option<WorldObjectAsset>,
    {
        if let Some(object_id) = &self.object_id {
            if let Some(mut spawned) = object_loader(object_id) {
                spawned.object_id = Some(object_id.clone());
                spawned.name = if self.name.is_empty() {
                    spawned.name
                } else {
                    self.name
                };
                spawned.transform = self.transform;
                if self.mesh_renderer.is_some() {
                    spawned.mesh_renderer = self.mesh_renderer;
                }
                if self.sprite_renderer.is_some() {
                    spawned.sprite_renderer = self.sprite_renderer;
                }
                if self.tilemap.is_some() {
                    spawned.tilemap = self.tilemap;
                }
                if self.camera.is_some() {
                    spawned.camera = self.camera;
                }
                spawned.active_camera = self.active_camera;
                if self.audio_source.is_some() {
                    spawned.audio_source = self.audio_source;
                }
                if self.physics_collision.is_some() {
                    spawned.physics_collision = self.physics_collision;
                }
                return spawned.into_object(project_root);
            }
        }

        self.into_object(project_root)
    }
}

impl TransformAsset {
    fn from_transform(transform: &Transform) -> Self {
        Self {
            position: transform.position.to_array(),
            rotation: transform.rotation.to_array(),
            scale: transform.scale.to_array(),
        }
    }

    fn into_component(self) -> Transform {
        Transform {
            position: Vec3::from_array(self.position),
            rotation: Quat::from_array(self.rotation),
            scale: Vec3::from_array(self.scale),
            previous_position: Vec3::from_array(self.position),
            previous_rotation: Quat::from_array(self.rotation),
        }
    }
}

impl MeshRendererAsset {
    fn from_component(component: &MeshRenderer) -> Option<Self> {
        let primitive = infer_mesh_primitive(&component.mesh)?;
        Some(Self {
            primitive,
            color: component.color,
        })
    }

    fn into_component(self) -> MeshRenderer {
        let mesh = match self.primitive {
            MeshPrimitiveAsset::Cube { size } => Mesh::cube(size),
        };

        MeshRenderer {
            mesh,
            color: self.color,
        }
    }
}

impl SpriteRendererAsset {
    fn from_component(component: &SpriteRenderer) -> Self {
        Self {
            sprite: component.texture_path.clone(),
        }
    }

    fn into_component(self, project_root: Option<&Path>) -> SpriteRenderer {
        let mut sprite = SpriteRenderer::default();
        if let Some(path) = self.sprite {
            if let Some(project_root) = project_root {
                if let Some(handle) = load_texture_handle(project_root, &path) {
                    sprite.set_texture(Some(handle), Some(path));
                } else {
                    sprite.texture_path = Some(path);
                }
            } else {
                sprite.texture_path = Some(path);
            }
        }
        sprite
    }
}

impl CameraAsset {
    fn from_component(camera: &Camera) -> Self {
        Self {
            position: camera.position.to_array(),
            target: camera.target.to_array(),
            up: camera.up.to_array(),
            perspective: matches!(camera.projection, ProjectionType::Perspective),
            ortho_size: camera.ortho_size.to_array(),
            near: camera.near,
            far: camera.far,
            fov_radians: camera.fov,
            viewport_size: [camera.viewport_size.0, camera.viewport_size.1],
        }
    }

    fn into_component(self) -> Camera {
        if self.perspective {
            Camera::new_perspective(
                Vec3::from_array(self.position),
                Vec3::from_array(self.target),
                Vec3::from_array(self.up),
                self.fov_radians,
                self.near,
                self.far,
                (self.viewport_size[0], self.viewport_size[1]),
            )
        } else {
            let mut camera = Camera::new_ortho(
                self.ortho_size[0] * 10.0,
                self.ortho_size[1] * 10.0,
                (self.viewport_size[0], self.viewport_size[1]),
            );
            camera.position = Vec3::from_array(self.position);
            camera.target = Vec3::from_array(self.target);
            camera.up = Vec3::from_array(self.up);
            camera.near = self.near;
            camera.far = self.far;
            camera
        }
    }
}

impl PhysicsCollisionAsset {
    fn from_component(collision: &PhysicsCollision) -> Self {
        Self {
            size: collision.size.to_array(),
            enabled: collision.enabled,
        }
    }

    fn into_component(self) -> PhysicsCollision {
        PhysicsCollision {
            size: Vec2::from_array(self.size),
            enabled: self.enabled,
        }
    }
}

impl AudioSourceAsset {
    fn from_component(audio: &AudioSource) -> Self {
        Self {
            source: audio.source_path.clone(),
            volume: audio.volume,
            looped: audio.looped,
            play_on_awake: audio.play_on_awake,
            spatial: audio.spatial,
            min_distance: audio.min_distance,
            max_distance: audio.max_distance,
        }
    }

    fn into_component(self, project_root: Option<&Path>) -> AudioSource {
        let mut audio = if self.spatial {
            AudioSource::new3d()
        } else {
            AudioSource::new2d()
        };
        if let Some(path) = self.source {
            if let Some(project_root) = project_root {
                if let Ok(asset) =
                    AudioAsset::from_file(project_root.to_string_lossy().as_ref(), &path)
                {
                    audio.set_asset_with_path(Some(std::sync::Arc::new(asset)), Some(path));
                } else {
                    audio.source_path = Some(path);
                }
            } else {
                audio.source_path = Some(path);
            }
        }
        audio.volume = self.volume;
        audio.looped = self.looped;
        audio.play_on_awake = self.play_on_awake;
        audio.min_distance = self.min_distance;
        audio.max_distance = self.max_distance;
        audio
    }
}

impl TilemapAsset {
    fn from_component(tilemap: &Tilemap) -> Self {
        Self {
            width: tilemap.width,
            height: tilemap.height,
            tile_size: [tilemap.tile_size.x as u32, tilemap.tile_size.y as u32],
            offset: [tilemap.offset.x, tilemap.offset.y],
            layers: tilemap
                .layers
                .iter()
                .map(TilemapLayerAsset::from_component)
                .collect(),
        }
    }

    fn into_component(self) -> Tilemap {
        let mut tilemap = Tilemap {
            width: self.width,
            height: self.height,
            tile_size: USizeVec2::new(self.tile_size[0] as usize, self.tile_size[1] as usize),
            offset: IVec2::new(self.offset[0], self.offset[1]),
            layers: Vec::new(),
        };
        for layer in self.layers {
            tilemap
                .layers
                .push(layer.into_component(self.width, self.height));
        }
        tilemap
    }
}

impl TilemapLayerAsset {
    fn from_component(layer: &TilemapLayer) -> Self {
        Self {
            name: layer.name.clone(),
            visible: layer.visible,
            opacity: layer.opacity,
        }
    }

    fn into_component(self, width: u32, height: u32) -> TilemapLayer {
        let mut layer = TilemapLayer::new(self.name, width, height);
        layer.visible = self.visible;
        layer.opacity = self.opacity;
        layer
    }
}

fn infer_mesh_primitive(mesh: &Mesh) -> Option<MeshPrimitiveAsset> {
    if mesh.vertices.is_empty() {
        return None;
    }

    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    for vertex in &mesh.vertices {
        min_x = min_x.min(vertex.position[0]);
        max_x = max_x.max(vertex.position[0]);
    }

    let size = (max_x - min_x).abs();
    Some(MeshPrimitiveAsset::Cube { size })
}

fn project_root_for_world_path(path: &Path) -> Option<PathBuf> {
    let manifest_path = find_project_manifest(path)?;
    load_project(manifest_path)
        .ok()
        .map(|project| project.root_dir)
}

fn load_texture_handle(project_root: &Path, relative_path: &str) -> Option<Handle<TextureAsset>> {
    let texture = TextureAsset::load(&project_root.join(relative_path)).ok()?;
    Some(Handle {
        inner: std::sync::Arc::new(texture),
    })
}
