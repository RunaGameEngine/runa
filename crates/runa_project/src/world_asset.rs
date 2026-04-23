use std::fs;
use std::path::{Path, PathBuf};

use runa_asset::{AudioAsset, Handle, TextureAsset};
use runa_core::components::{
    ActiveCamera, AudioSource, Camera, Mesh, MeshRenderer, ObjectDefinitionInstance,
    PhysicsCollision, ProjectionType, SerializedField, SerializedTypeEntry, SerializedTypeKind,
    SerializedTypeStorage, SpriteRenderer, Tilemap, TilemapLayer, TilemapRenderer, Transform,
    DEFAULT_SPRITE_PIXELS_PER_UNIT,
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
    pub serialized_components: Vec<SerializedObjectTypeAsset>,
    pub serialized_scripts: Vec<SerializedObjectTypeAsset>,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProjectRegisteredTypeKind {
    Component,
    Script,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProjectRegistrationSource {
    BuiltIn,
    User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRegisteredTypeRecord {
    pub type_name: String,
    pub kind: ProjectRegisteredTypeKind,
    pub source: ProjectRegistrationSource,
    pub editor_addable: bool,
    pub default_fields: Vec<SerializedField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadataSnapshot {
    pub object_records: Vec<PlaceableObjectRecord>,
    pub registered_types: Vec<ProjectRegisteredTypeRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedObjectTypeAsset {
    pub type_name: String,
    pub fields: Vec<SerializedField>,
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
    #[serde(default = "default_sprite_pixels_per_unit")]
    pub pixels_per_unit: f32,
}

fn default_sprite_pixels_per_unit() -> f32 {
    DEFAULT_SPRITE_PIXELS_PER_UNIT
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeshPrimitiveAsset {
    Cube { size: f32 },
    Quad { width: f32, height: f32 },
    Plane { width: f32, depth: f32 },
    Pyramid { width: f32, height: f32, depth: f32 },
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
            ortho_size: [320.0, 180.0],
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

    let mut camera = Object::new("Main Camera");
    camera.add_component(Camera::default());
    camera.add_component(ActiveCamera);
    world.spawn(camera);

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

pub fn load_world_with_runtime_registry(
    path: impl AsRef<Path>,
    runtime_registry: &runa_core::registry::RuntimeRegistry,
) -> Result<World, ProjectError> {
    let content = fs::read_to_string(path.as_ref())?;
    let asset: WorldAsset = ron::from_str(&content)?;
    let project_root = project_root_for_world_path(path.as_ref());
    Ok(asset.into_world_with_runtime_registry(project_root.as_deref(), runtime_registry))
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
                .query::<Transform>()
                .into_iter()
                .filter_map(|object_id| world.get(object_id))
                .map(WorldObjectAsset::from_object)
                .collect(),
        }
    }

    pub fn into_world(self) -> World {
        self.into_world_with_project_root(None)
    }

    pub fn into_world_with_project_root(self, project_root: Option<&Path>) -> World {
        let mut world = World::default();
        for object in self.objects.into_iter().map(|object| object.into_object(project_root)) {
            world.spawn(object);
        }
        world
    }

    pub fn into_world_with_runtime_registry(
        self,
        project_root: Option<&Path>,
        runtime_registry: &runa_core::registry::RuntimeRegistry,
    ) -> World {
        let mut world = World::default();
        for object in self
            .objects
            .into_iter()
            .map(|object| object.into_object_with_runtime_registry(project_root, Some(runtime_registry)))
        {
            world.spawn(object);
        }
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
        for object in self
            .objects
            .into_iter()
            .map(|object| object.into_object_with_object_loader(project_root, &object_loader))
        {
            world.spawn(object);
        }
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
            serialized_components: collect_serialized_type_assets(object, SerializedTypeKind::Component),
            serialized_scripts: collect_serialized_type_assets(object, SerializedTypeKind::Script),
        }
    }

    pub fn into_object(self, project_root: Option<&Path>) -> Object {
        self.into_object_with_runtime_registry(project_root, None)
    }

    pub fn into_object_with_runtime_registry(
        self,
        project_root: Option<&Path>,
        runtime_registry: Option<&runa_core::registry::RuntimeRegistry>,
    ) -> Object {
        let object_id = self.object_id.clone();
        let mut object = Object::new(self.name);
        object.add_component(self.transform.into_component());

        if let Some(mesh_renderer) = self.mesh_renderer {
            object.add_component(mesh_renderer.into_component());
        }
        if let Some(sprite_renderer) = self.sprite_renderer {
            object.add_component(sprite_renderer.into_component(project_root));
        }
        if let Some(tilemap) = self.tilemap {
            object.add_component(tilemap.into_component());
            if object.get_component::<TilemapRenderer>().is_none() {
                object.add_component(TilemapRenderer::new());
            }
        }
        if let Some(camera) = self.camera {
            let transform = object
                .get_component::<Transform>()
                .cloned()
                .unwrap_or_else(Transform::default);
            object.add_component(camera.into_component_with_transform(&transform));
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

        apply_serialized_type_assets(
            &mut object,
            runtime_registry,
            SerializedTypeKind::Component,
            self.serialized_components,
        );
        apply_serialized_type_assets(
            &mut object,
            runtime_registry,
            SerializedTypeKind::Script,
            self.serialized_scripts,
        );

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
                if !self.serialized_components.is_empty() {
                    spawned.serialized_components = self.serialized_components;
                }
                if !self.serialized_scripts.is_empty() {
                    spawned.serialized_scripts = self.serialized_scripts;
                }
                return spawned.into_object_with_runtime_registry(project_root, None);
            }
        }

        self.into_object_with_runtime_registry(project_root, None)
    }
}

fn collect_serialized_type_assets(
    object: &Object,
    kind: SerializedTypeKind,
) -> Vec<SerializedObjectTypeAsset> {
    let mut assets = Vec::new();
    for info in object.component_infos() {
        let matches_kind = match kind {
            SerializedTypeKind::Component => {
                info.kind() == runa_core::components::ComponentRuntimeKind::Component
            }
            SerializedTypeKind::Script => {
                info.kind() == runa_core::components::ComponentRuntimeKind::Script
            }
        };
        if !matches_kind || is_builtin_serialized_type(info.type_id()) {
            continue;
        }

        if let Some(fields) = object.with_component_by_type_id(info.type_id(), |component| {
            component.serialized_fields()
        }) {
            assets.push(SerializedObjectTypeAsset {
                type_name: info.type_name().to_string(),
                fields,
            });
        }
    }

    if let Some(storage) = object.get_component::<SerializedTypeStorage>() {
        for entry in storage.entries_of_kind(kind) {
            assets.push(SerializedObjectTypeAsset {
                type_name: entry.type_name.clone(),
                fields: entry.fields.clone(),
            });
        }
    }

    dedup_serialized_assets(assets)
}

fn dedup_serialized_assets(
    assets: Vec<SerializedObjectTypeAsset>,
) -> Vec<SerializedObjectTypeAsset> {
    let mut deduped: Vec<SerializedObjectTypeAsset> = Vec::new();
    for asset in assets {
        if let Some(existing) = deduped
            .iter_mut()
            .find(|existing| existing.type_name == asset.type_name)
        {
            *existing = asset;
        } else {
            deduped.push(asset);
        }
    }
    deduped
}

fn apply_serialized_type_assets(
    object: &mut Object,
    runtime_registry: Option<&runa_core::registry::RuntimeRegistry>,
    kind: SerializedTypeKind,
    assets: Vec<SerializedObjectTypeAsset>,
) {
    for asset in assets {
        if let Some(registry) = runtime_registry {
            if let Some(metadata) = registry.types().get_by_name(&asset.type_name) {
                let type_id = metadata.type_id();
                let has_runtime_instance =
                    object.with_component_by_type_id(type_id, |_| ()).is_some();
                if has_runtime_instance || registry.add_type_to_object(object, type_id) {
                    for field in &asset.fields {
                        let _ = object.with_component_mut_by_type_id(type_id, |component| {
                            component.set_serialized_field(&field.name, field.value.clone())
                        });
                    }
                    continue;
                }
            }
        }

        let mut storage = object
            .get_component::<SerializedTypeStorage>()
            .cloned()
            .unwrap_or_default();
        storage.upsert(SerializedTypeEntry {
            type_name: asset.type_name,
            kind,
            fields: asset.fields,
        });
        if object.get_component::<SerializedTypeStorage>().is_some() {
            object.remove_component_type_id(std::any::TypeId::of::<SerializedTypeStorage>());
        }
        object.add_component(storage);
    }
}

fn is_builtin_serialized_type(type_id: std::any::TypeId) -> bool {
    use std::any::TypeId;

    [
        TypeId::of::<Transform>(),
        TypeId::of::<MeshRenderer>(),
        TypeId::of::<SpriteRenderer>(),
        TypeId::of::<Tilemap>(),
        TypeId::of::<TilemapRenderer>(),
        TypeId::of::<Camera>(),
        TypeId::of::<ActiveCamera>(),
        TypeId::of::<AudioSource>(),
        TypeId::of::<PhysicsCollision>(),
        TypeId::of::<ObjectDefinitionInstance>(),
        TypeId::of::<SerializedTypeStorage>(),
    ]
    .contains(&type_id)
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
            MeshPrimitiveAsset::Quad { width, height } => Mesh::quad(width, height),
            MeshPrimitiveAsset::Plane { width, depth } => Mesh::plane(width, depth),
            MeshPrimitiveAsset::Pyramid { width, height, depth } => {
                Mesh::pyramid(width, height, depth)
            }
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
            pixels_per_unit: component.pixels_per_unit,
        }
    }

    fn into_component(self, project_root: Option<&Path>) -> SpriteRenderer {
        let mut sprite = SpriteRenderer::default();
        sprite.pixels_per_unit = self.pixels_per_unit.max(f32::EPSILON);
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

    fn into_component_with_transform(self, transform: &Transform) -> Camera {
        let inverse_rotation = transform.rotation.inverse();
        let local_position = inverse_rotation * (Vec3::from_array(self.position) - transform.position);
        let local_target = inverse_rotation * (Vec3::from_array(self.target) - transform.position);
        let local_up = inverse_rotation * Vec3::from_array(self.up);

        if self.perspective {
            let mut camera = Camera::new_perspective(
                local_position,
                local_target,
                local_up,
                self.fov_radians,
                self.near,
                self.far,
            );
            camera.resize(self.viewport_size[0], self.viewport_size[1]);
            camera
        } else {
            let mut camera = Camera::new_ortho(self.ortho_size[0], self.ortho_size[1]);
            camera.position = local_position;
            camera.target = local_target;
            camera.up = local_up;
            camera.near = self.near;
            camera.far = self.far;
            camera.resize(self.viewport_size[0], self.viewport_size[1]);
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
    let hint = mesh.primitive_hint?;
    let (min, max) = mesh_bounds(mesh)?;
    let size = max - min;
    match hint {
        runa_core::components::BuiltinMeshPrimitive::Cube => {
            Some(MeshPrimitiveAsset::Cube { size: size.x.abs().max(size.y.abs()).max(size.z.abs()) })
        }
        runa_core::components::BuiltinMeshPrimitive::Quad => Some(MeshPrimitiveAsset::Quad {
            width: size.x.abs(),
            height: size.y.abs(),
        }),
        runa_core::components::BuiltinMeshPrimitive::Plane => Some(MeshPrimitiveAsset::Plane {
            width: size.x.abs(),
            depth: size.z.abs(),
        }),
        runa_core::components::BuiltinMeshPrimitive::Pyramid => Some(MeshPrimitiveAsset::Pyramid {
            width: size.x.abs(),
            height: size.y.abs(),
            depth: size.z.abs(),
        }),
    }
}

fn mesh_bounds(mesh: &Mesh) -> Option<(Vec3, Vec3)> {
    let first = mesh.vertices.first()?;
    let mut min = Vec3::from_array(first.position);
    let mut max = Vec3::from_array(first.position);
    for vertex in &mesh.vertices {
        let p = Vec3::from_array(vertex.position);
        min = min.min(p);
        max = max.max(p);
    }
    Some((min, max))
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

#[cfg(test)]
mod tests {
    use super::{apply_serialized_type_assets, SerializedObjectTypeAsset, WorldObjectAsset};
    use runa_core::components::{
        SerializedField, SerializedFieldAccess, SerializedFieldValue, SerializedTypeKind,
        SerializedTypeStorage,
    };
    use runa_core::ocs::{Object, Script, ScriptContext};
    use runa_core::registry::RuntimeRegistry;

    #[derive(Clone)]
    struct TestScript {
        speed: f32,
    }

    impl SerializedFieldAccess for TestScript {
        fn serialized_fields(&self) -> Vec<SerializedField> {
            vec![SerializedField {
                name: "speed".to_string(),
                value: SerializedFieldValue::F32(self.speed),
            }]
        }

        fn set_serialized_field(&mut self, field_name: &str, value: SerializedFieldValue) -> bool {
            match (field_name, value) {
                ("speed", SerializedFieldValue::F32(speed)) => {
                    self.speed = speed;
                    true
                }
                _ => false,
            }
        }
    }

    impl Script for TestScript {
        fn update(&mut self, _ctx: &mut ScriptContext, _dt: f32) {}
    }

    #[test]
    fn serialized_script_fields_apply_to_existing_runtime_instance() {
        let mut registry = RuntimeRegistry::new();
        let metadata = registry.register_script_named_factory::<TestScript, _>(
            std::any::type_name::<TestScript>(),
            || TestScript { speed: 1.0 },
        );

        let mut object = Object::new("Runtime Script");
        assert!(registry.add_type_to_object(&mut object, metadata.type_id()));

        apply_serialized_type_assets(
            &mut object,
            Some(&registry),
            SerializedTypeKind::Script,
            vec![SerializedObjectTypeAsset {
                type_name: std::any::type_name::<TestScript>().to_string(),
                fields: vec![SerializedField {
                    name: "speed".to_string(),
                    value: SerializedFieldValue::F32(9.5),
                }],
            }],
        );

        let applied_speed = object
            .get_component::<TestScript>()
            .map(|script| script.speed)
            .unwrap_or_default();
        assert!((applied_speed - 9.5).abs() < f32::EPSILON);
    }

    #[test]
    fn archetype_override_preserves_serialized_script_asset_data() {
        let mut base_object = Object::new("Base");
        base_object.add_component(TestScript { speed: 1.0 });
        let base_asset = WorldObjectAsset::from_object(&base_object);

        let mut override_asset = WorldObjectAsset::from_object(&Object::new("Override"));
        override_asset.object_id = Some("test_script_archetype".to_string());
        override_asset.serialized_scripts = vec![SerializedObjectTypeAsset {
            type_name: std::any::type_name::<TestScript>().to_string(),
            fields: vec![SerializedField {
                name: "speed".to_string(),
                value: SerializedFieldValue::F32(4.0),
            }],
        }];

        let object = override_asset.into_object_with_object_loader(None, |_object_id| {
            Some(base_asset.clone())
        });

        let serialized_speed = object
            .get_component::<SerializedTypeStorage>()
            .and_then(|storage| {
                storage
                    .entries_of_kind(SerializedTypeKind::Script)
                    .find(|entry| entry.type_name == std::any::type_name::<TestScript>())
            })
            .and_then(|entry| entry.fields.first())
            .and_then(|field| match &field.value {
                SerializedFieldValue::F32(value) => Some(*value),
                _ => None,
            })
            .unwrap_or_default();

        assert!((serialized_speed - 4.0).abs() < f32::EPSILON);
    }
}
