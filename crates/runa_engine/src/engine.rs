use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use runa_core::{
    components::{
        ui::CanvasSpace, ActiveCamera, AudioListener, AudioSource, Camera, Collider2D, Component,
        CursorInteractable, DirectionalLight, MeshRenderer, ObjectDefinitionInstance,
        PhysicsCollision, PointLight, SerializedTypeStorage, Sorting, SpriteAnimator,
        SpriteRenderer, Tilemap, TilemapLayer, TilemapRenderer, Transform, UiRenderer,
    },
    glam::USizeVec2,
    ocs::{Object, Script, World},
    registry::{
        ArchetypeKey, ArchetypeMetadata, ObjectDef, ObjectDefMetadata, RunaArchetype,
        RunaComponentType, RunaScriptType, RuntimeRegistry, TypeMetadata,
    },
};

pub trait RunaTypeRegistration {
    fn register(engine: &mut Engine) -> TypeMetadata;
}

pub struct Engine {
    runtime_registry: RuntimeRegistry,
}

impl Engine {
    pub fn new() -> Self {
        let mut engine = Self {
            runtime_registry: RuntimeRegistry::new(),
        };
        engine.register_builtin_types();
        engine
    }

    pub fn runtime_registry(&self) -> &RuntimeRegistry {
        &self.runtime_registry
    }

    pub fn runtime_registry_mut(&mut self) -> &mut RuntimeRegistry {
        &mut self.runtime_registry
    }

    pub fn register_component<T: Component + 'static>(&mut self) -> TypeMetadata {
        self.runtime_registry.register_component::<T>()
    }

    pub fn register_component_factory<T, F>(&mut self, factory: F) -> TypeMetadata
    where
        T: Component + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.runtime_registry
            .register_component_factory::<T, F>(factory)
    }

    pub fn register_component_named_factory<T, F>(
        &mut self,
        type_name: &'static str,
        factory: F,
    ) -> TypeMetadata
    where
        T: Component + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.runtime_registry
            .register_component_named_factory::<T, F>(type_name, factory)
    }

    pub fn register_default_component<T>(&mut self) -> TypeMetadata
    where
        T: Component + Default + 'static,
    {
        self.register_component_factory::<T, _>(T::default)
    }

    pub fn register_component_named<T: Component + 'static>(
        &mut self,
        type_name: &'static str,
    ) -> TypeMetadata {
        self.runtime_registry
            .register_component_named::<T>(type_name)
    }

    pub fn register_script<T: Script + 'static>(&mut self) -> TypeMetadata {
        self.runtime_registry.register_script::<T>()
    }

    pub fn register_script_factory<T, F>(&mut self, factory: F) -> TypeMetadata
    where
        T: Script + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.runtime_registry
            .register_script_factory::<T, F>(factory)
    }

    pub fn register_script_named_factory<T, F>(
        &mut self,
        type_name: &'static str,
        factory: F,
    ) -> TypeMetadata
    where
        T: Script + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.runtime_registry
            .register_script_named_factory::<T, F>(type_name, factory)
    }

    pub fn register_default_script<T>(&mut self) -> TypeMetadata
    where
        T: Script + Default + 'static,
    {
        self.register_script_factory::<T, _>(T::default)
    }

    pub fn register<T: RunaTypeRegistration>(&mut self) -> TypeMetadata {
        T::register(self)
    }

    pub fn register_script_named<T: Script + 'static>(
        &mut self,
        type_name: &'static str,
    ) -> TypeMetadata {
        self.runtime_registry.register_script_named::<T>(type_name)
    }

    pub fn register_derived_component<T: RunaComponentType>(&mut self) -> TypeMetadata {
        self.register_component_named::<T>(T::runa_component_type_name())
    }

    pub fn register_derived_script<T: RunaScriptType>(&mut self) -> TypeMetadata {
        self.register_script_named::<T>(T::runa_script_type_name())
    }

    pub fn register_archetype<T>(&mut self) -> ArchetypeMetadata
    where
        T: RunaArchetype,
    {
        self.runtime_registry.register_archetype::<T>()
    }

    pub fn register_object_def<T>(&mut self) -> ObjectDefMetadata
    where
        T: ObjectDef,
    {
        self.runtime_registry.register_object_def::<T>()
    }

    pub fn register_archetype_named<F>(
        &mut self,
        name: impl Into<Arc<str>>,
        factory: F,
    ) -> ArchetypeMetadata
    where
        F: Fn() -> Object + Send + Sync + 'static,
    {
        self.runtime_registry
            .register_archetype_named(name, factory)
    }

    pub fn create_world(&self) -> Rc<RefCell<World>> {
        // Создаём новый мир
        let mut world = World::default();

        // Настраиваем runtime_registry
        world.set_runtime_registry(Arc::new(self.runtime_registry.clone()));

        // Оборачиваем World в Rc<RefCell>
        Rc::new(RefCell::new(world))
    }

    pub fn spawn_archetype<T: RunaArchetype>(&self, world: &mut World) -> u64 {
        T::create(world)
    }

    pub fn spawn_def<T: ObjectDef>(&self, world: &mut World) -> u64 {
        world.spawn_def::<T>()
    }

    pub fn spawn_archetype_by_key(&self, world: &mut World, key: &ArchetypeKey) -> Option<u64> {
        self.runtime_registry.spawn_archetype_by_key(world, key)
    }

    pub fn spawn_archetype_by_name(&self, world: &mut World, name: &str) -> Option<u64> {
        self.runtime_registry.spawn_archetype_by_name(world, name)
    }

    pub fn spawn_def_by_name(&self, world: &mut World, name: &str) -> Option<u64> {
        self.runtime_registry.spawn_object_def_by_name(world, name)
    }

    fn register_builtin_types(&mut self) {
        self.runtime_registry
            .register_builtin_component::<Transform>();
        self.runtime_registry
            .register_builtin_component_factory::<Camera, _>(Camera::default);
        self.runtime_registry
            .register_builtin_component_factory::<ActiveCamera, _>(|| ActiveCamera);
        self.runtime_registry
            .register_builtin_component_factory::<SpriteRenderer, _>(SpriteRenderer::default);
        self.runtime_registry
            .register_builtin_component_factory::<SpriteAnimator, _>(SpriteAnimator::default);
        self.runtime_registry
            .register_builtin_component_factory::<Sorting, _>(Sorting::default);
        self.runtime_registry
            .register_builtin_component_factory::<Collider2D, _>(Collider2D::default);
        self.runtime_registry
            .register_builtin_component_factory::<UiRenderer, _>(|| {
                UiRenderer::new(CanvasSpace::Screen)
            });
        self.runtime_registry
            .register_builtin_component_factory::<AudioListener, _>(AudioListener::default);
        self.runtime_registry
            .register_builtin_component_factory::<AudioSource, _>(AudioSource::new2d);
        self.runtime_registry
            .register_builtin_component_factory::<CursorInteractable, _>(
                CursorInteractable::default,
            );
        self.runtime_registry
            .register_builtin_component_factory::<MeshRenderer, _>(|| {
                MeshRenderer::new(runa_core::components::Mesh::cube(1.0))
            });
        self.runtime_registry
            .register_builtin_component_factory::<DirectionalLight, _>(DirectionalLight::default);
        self.runtime_registry
            .register_builtin_component_factory::<PointLight, _>(PointLight::default);
        self.runtime_registry
            .register_builtin_component::<ObjectDefinitionInstance>();
        self.runtime_registry
            .register_builtin_component::<SerializedTypeStorage>();
        self.runtime_registry
            .register_builtin_component_factory::<PhysicsCollision, _>(PhysicsCollision::default);
        self.runtime_registry
            .register_builtin_component_factory::<Tilemap, _>(|| {
                let mut tilemap = Tilemap::centered(8, 8, USizeVec2::new(32, 32));
                tilemap.add_layer(TilemapLayer::new("Base".to_string(), 8, 8));
                tilemap
            });
        self.runtime_registry
            .register_builtin_component_object_factory::<TilemapRenderer, _>(|object| {
                let mut changed = false;
                if !object.has_component::<Tilemap>() {
                    let mut tilemap = Tilemap::centered(8, 8, USizeVec2::new(32, 32));
                    tilemap.add_layer(TilemapLayer::new("Base".to_string(), 8, 8));
                    object.add_component(tilemap);
                    changed = true;
                }
                if !object.has_component::<TilemapRenderer>() {
                    object.add_component(TilemapRenderer::new());
                    changed = true;
                }
                changed
            });
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
