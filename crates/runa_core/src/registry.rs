use std::any::{type_name, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    components::{Component, ObjectDefinitionInstance},
    ocs::{Object, ObjectBuilder, ObjectId, Script, World},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisteredTypeKind {
    Component,
    Script,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistrationSource {
    BuiltIn,
    User,
}

#[derive(Debug, Clone, Copy)]
pub struct TypeMetadata {
    type_id: TypeId,
    type_name: &'static str,
    kind: RegisteredTypeKind,
    source: RegistrationSource,
}

impl TypeMetadata {
    pub fn type_id(self) -> TypeId {
        self.type_id
    }

    pub fn type_name(self) -> &'static str {
        self.type_name
    }

    pub fn kind(self) -> RegisteredTypeKind {
        self.kind
    }

    pub fn source(self) -> RegistrationSource {
        self.source
    }
}

type TypeObjectFactory = Arc<dyn Fn(&mut Object) -> bool + Send + Sync>;

#[derive(Clone)]
struct TypeRegistration {
    metadata: TypeMetadata,
    object_factory: Option<TypeObjectFactory>,
}

#[derive(Default, Clone)]
pub struct TypeRegistry {
    types_by_id: HashMap<TypeId, TypeRegistration>,
    types_by_name: HashMap<&'static str, TypeId>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_component<T: Component + 'static>(&mut self) -> TypeMetadata {
        self.register::<T>(RegisteredTypeKind::Component, RegistrationSource::User)
    }

    pub fn register_component_named<T: Component + 'static>(
        &mut self,
        type_name: &'static str,
    ) -> TypeMetadata {
        self.register_named::<T>(
            RegisteredTypeKind::Component,
            RegistrationSource::User,
            type_name,
        )
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
        self.register_factory::<T, F>(
            RegisteredTypeKind::Component,
            RegistrationSource::User,
            type_name,
            factory,
        )
    }

    pub fn register_builtin_component<T: Component + 'static>(&mut self) -> TypeMetadata {
        self.register::<T>(RegisteredTypeKind::Component, RegistrationSource::BuiltIn)
    }

    pub fn register_builtin_component_named<T: Component + 'static>(
        &mut self,
        type_name: &'static str,
    ) -> TypeMetadata {
        self.register_named::<T>(
            RegisteredTypeKind::Component,
            RegistrationSource::BuiltIn,
            type_name,
        )
    }

    pub fn register_script<T: Script + 'static>(&mut self) -> TypeMetadata {
        self.register::<T>(RegisteredTypeKind::Script, RegistrationSource::User)
    }

    pub fn register_script_named<T: Script + 'static>(
        &mut self,
        type_name: &'static str,
    ) -> TypeMetadata {
        self.register_named::<T>(
            RegisteredTypeKind::Script,
            RegistrationSource::User,
            type_name,
        )
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
        self.register_factory::<T, F>(
            RegisteredTypeKind::Script,
            RegistrationSource::User,
            type_name,
            factory,
        )
    }

    pub fn register_builtin_script<T: Script + 'static>(&mut self) -> TypeMetadata {
        self.register::<T>(RegisteredTypeKind::Script, RegistrationSource::BuiltIn)
    }

    pub fn register_builtin_script_named<T: Script + 'static>(
        &mut self,
        type_name: &'static str,
    ) -> TypeMetadata {
        self.register_named::<T>(
            RegisteredTypeKind::Script,
            RegistrationSource::BuiltIn,
            type_name,
        )
    }

    pub fn get_by_id(&self, type_id: TypeId) -> Option<&TypeMetadata> {
        self.types_by_id.get(&type_id).map(|entry| &entry.metadata)
    }

    pub fn get_by_name(&self, type_name: &str) -> Option<&TypeMetadata> {
        let type_id = self.types_by_name.get(type_name)?;
        self.types_by_id.get(type_id).map(|entry| &entry.metadata)
    }

    pub fn get_component<T: Component + 'static>(&self) -> Option<&TypeMetadata> {
        self.get_by_id(TypeId::of::<T>())
            .filter(|metadata| metadata.kind == RegisteredTypeKind::Component)
    }

    pub fn get_script<T: Script + 'static>(&self) -> Option<&TypeMetadata> {
        self.get_by_id(TypeId::of::<T>())
            .filter(|metadata| metadata.kind == RegisteredTypeKind::Script)
    }

    pub fn registered_types(&self) -> Vec<TypeMetadata> {
        self.types_by_id
            .values()
            .map(|entry| entry.metadata)
            .collect()
    }

    pub fn registered_builtin_types(&self) -> Vec<TypeMetadata> {
        self.types_by_id
            .values()
            .map(|entry| entry.metadata)
            .filter(|metadata| metadata.source == RegistrationSource::BuiltIn)
            .collect()
    }

    pub fn registered_user_types(&self) -> Vec<TypeMetadata> {
        self.types_by_id
            .values()
            .map(|entry| entry.metadata)
            .filter(|metadata| metadata.source == RegistrationSource::User)
            .collect()
    }

    pub fn has_object_factory(&self, type_id: TypeId) -> bool {
        self.types_by_id
            .get(&type_id)
            .and_then(|entry| entry.object_factory.as_ref())
            .is_some()
    }

    pub fn add_to_object(&self, object: &mut Object, type_id: TypeId) -> bool {
        let Some(factory) = self
            .types_by_id
            .get(&type_id)
            .and_then(|entry| entry.object_factory.as_ref())
        else {
            return false;
        };
        factory(object)
    }

    fn register_object_factory<T, F>(
        &mut self,
        kind: RegisteredTypeKind,
        source: RegistrationSource,
        type_name: &'static str,
        factory: F,
    ) -> TypeMetadata
    where
        T: 'static,
        F: Fn(&mut Object) -> bool + Send + Sync + 'static,
    {
        let metadata = self.register_named::<T>(kind, source, type_name);
        if let Some(entry) = self.types_by_id.get_mut(&metadata.type_id) {
            entry.object_factory = Some(Arc::new(factory));
        }
        metadata
    }

    fn register<T: 'static>(
        &mut self,
        kind: RegisteredTypeKind,
        source: RegistrationSource,
    ) -> TypeMetadata {
        self.register_named::<T>(kind, source, type_name::<T>())
    }

    fn register_named<T: 'static>(
        &mut self,
        kind: RegisteredTypeKind,
        source: RegistrationSource,
        type_name: &'static str,
    ) -> TypeMetadata {
        let metadata = TypeMetadata {
            type_id: TypeId::of::<T>(),
            type_name,
            kind,
            source,
        };

        self.types_by_name
            .insert(metadata.type_name, metadata.type_id);
        self.types_by_id.insert(
            metadata.type_id,
            TypeRegistration {
                metadata,
                object_factory: None,
            },
        );
        metadata
    }

    fn register_factory<T, F>(
        &mut self,
        kind: RegisteredTypeKind,
        source: RegistrationSource,
        type_name: &'static str,
        factory: F,
    ) -> TypeMetadata
    where
        T: Component + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        let metadata = self.register_named::<T>(kind, source, type_name);
        let object_factory: TypeObjectFactory = Arc::new(move |object| {
            if object.has_component::<T>() {
                return false;
            }
            object.add_component(factory());
            true
        });

        if let Some(entry) = self.types_by_id.get_mut(&metadata.type_id) {
            entry.object_factory = Some(object_factory);
        }

        metadata
    }
}

pub trait RunaComponentType: Component + 'static {
    fn runa_component_type_name() -> &'static str {
        type_name::<Self>()
    }

    fn register(registry: &mut TypeRegistry) -> TypeMetadata
    where
        Self: Sized,
    {
        registry.register_component_named::<Self>(Self::runa_component_type_name())
    }
}

pub trait RunaScriptType: Script + 'static {
    fn runa_script_type_name() -> &'static str {
        type_name::<Self>()
    }

    fn register(registry: &mut TypeRegistry) -> TypeMetadata
    where
        Self: Sized,
    {
        registry.register_script_named::<Self>(Self::runa_script_type_name())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArchetypeKey(Arc<str>);

impl ArchetypeKey {
    pub fn new(value: impl Into<Arc<str>>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ArchetypeKey {
    fn from(value: &str) -> Self {
        Self(Arc::<str>::from(value))
    }
}

impl From<String> for ArchetypeKey {
    fn from(value: String) -> Self {
        Self(Arc::<str>::from(value))
    }
}

impl From<Arc<str>> for ArchetypeKey {
    fn from(value: Arc<str>) -> Self {
        Self(value)
    }
}

pub type ObjectDefKey = ArchetypeKey;
pub type ObjectDefMetadata = ArchetypeMetadata;
pub type ObjectDefRegistry = ArchetypeRegistry;

pub trait ObjectDefName: Sized + 'static {
    fn key() -> ArchetypeKey;
}

pub trait ObjectDef: ObjectDefName {
    fn build(object: &mut ObjectBuilder);

    fn create_object() -> Object {
        let mut builder = ObjectBuilder::new();
        Self::build(&mut builder);
        let mut object = builder.build();
        if object
            .get_component::<ObjectDefinitionInstance>()
            .is_none()
        {
            object.add_component(ObjectDefinitionInstance::new(
                <Self as ObjectDefName>::key().as_str(),
            ));
        }
        object
    }
}

pub trait RunaArchetype: Sized + 'static {
    fn key() -> ArchetypeKey;
    fn create(world: &mut World) -> ObjectId;
}

type ArchetypeFactory = Arc<dyn Fn(&mut World) -> ObjectId + Send + Sync>;

#[derive(Clone)]
pub struct ArchetypeMetadata {
    key: ArchetypeKey,
    name: Arc<str>,
    source: RegistrationSource,
}

impl ArchetypeMetadata {
    pub fn key(&self) -> &ArchetypeKey {
        &self.key
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source(&self) -> RegistrationSource {
        self.source
    }
}

#[derive(Clone)]
struct ArchetypeRegistration {
    metadata: ArchetypeMetadata,
    factory: ArchetypeFactory,
}

#[derive(Default, Clone)]
pub struct ArchetypeRegistry {
    archetypes_by_key: HashMap<ArchetypeKey, ArchetypeRegistration>,
    keys_by_name: HashMap<Arc<str>, ArchetypeKey>,
}

impl ArchetypeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<T>(&mut self) -> ArchetypeMetadata
    where
        T: RunaArchetype,
    {
        let key = T::key();
        let name = Arc::<str>::from(key.as_str());
        self.register_factory(key, name, RegistrationSource::User, T::create)
    }

    pub fn register_def<T>(&mut self) -> ArchetypeMetadata
    where
        T: ObjectDef,
    {
        let key = <T as ObjectDefName>::key();
        let name = Arc::<str>::from(key.as_str());
        self.register_factory(key, name, RegistrationSource::User, |world| {
            world.spawn_object(T::create_object())
        })
    }

    pub fn register_named<F>(&mut self, name: impl Into<Arc<str>>, factory: F) -> ArchetypeMetadata
    where
        F: Fn() -> Object + Send + Sync + 'static,
    {
        let name = name.into();
        let key = ArchetypeKey::from(name.clone());
        let instance_name = name.clone();
        self.register_factory(key, name, RegistrationSource::User, move |world| {
            let mut object = factory();
            if object.get_component::<ObjectDefinitionInstance>().is_none() {
                object.add_component(ObjectDefinitionInstance::new(instance_name.as_ref()));
            }
            world.spawn_object(object)
        })
    }

    fn register_factory<F>(
        &mut self,
        key: ArchetypeKey,
        name: Arc<str>,
        source: RegistrationSource,
        factory: F,
    ) -> ArchetypeMetadata
    where
        F: Fn(&mut World) -> ObjectId + Send + Sync + 'static,
    {
        let metadata = ArchetypeMetadata {
            key: key.clone(),
            name: name.clone(),
            source,
        };
        let registration = ArchetypeRegistration {
            metadata: metadata.clone(),
            factory: Arc::new(factory),
        };

        self.keys_by_name.insert(name, key.clone());
        self.archetypes_by_key.insert(key, registration);
        metadata
    }

    pub fn contains_key(&self, key: &ArchetypeKey) -> bool {
        self.archetypes_by_key.contains_key(key)
    }

    pub fn contains_name(&self, name: &str) -> bool {
        self.keys_by_name.contains_key(name)
    }

    pub fn metadata_by_key(&self, key: &ArchetypeKey) -> Option<&ArchetypeMetadata> {
        self.archetypes_by_key.get(key).map(|entry| &entry.metadata)
    }

    pub fn metadata_by_name(&self, name: &str) -> Option<&ArchetypeMetadata> {
        let key = self.keys_by_name.get(name)?;
        self.metadata_by_key(key)
    }

    pub fn spawn_by_key(&self, world: &mut World, key: &ArchetypeKey) -> Option<ObjectId> {
        let factory = &self.archetypes_by_key.get(key)?.factory;
        Some(factory(world))
    }

    pub fn spawn_by_name(&self, world: &mut World, name: &str) -> Option<ObjectId> {
        let key = self.keys_by_name.get(name)?;
        self.spawn_by_key(world, key)
    }

    pub fn metadata(&self) -> Vec<ArchetypeMetadata> {
        self.archetypes_by_key
            .values()
            .map(|entry| entry.metadata.clone())
            .collect()
    }

    pub fn registered_user_archetypes(&self) -> Vec<ArchetypeMetadata> {
        self.archetypes_by_key
            .values()
            .filter(|entry| entry.metadata.source == RegistrationSource::User)
            .map(|entry| entry.metadata.clone())
            .collect()
    }

    pub fn registered_builtin_archetypes(&self) -> Vec<ArchetypeMetadata> {
        self.archetypes_by_key
            .values()
            .filter(|entry| entry.metadata.source == RegistrationSource::BuiltIn)
            .map(|entry| entry.metadata.clone())
            .collect()
    }
}

#[derive(Default, Clone)]
pub struct RuntimeRegistry {
    types: TypeRegistry,
    archetypes: ArchetypeRegistry,
}

impl RuntimeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn types(&self) -> &TypeRegistry {
        &self.types
    }

    pub fn types_mut(&mut self) -> &mut TypeRegistry {
        &mut self.types
    }

    pub fn archetypes(&self) -> &ArchetypeRegistry {
        &self.archetypes
    }

    pub fn archetypes_mut(&mut self) -> &mut ArchetypeRegistry {
        &mut self.archetypes
    }

    pub fn register_component<T: Component + 'static>(&mut self) -> TypeMetadata {
        self.types.register_component::<T>()
    }

    pub fn register_component_factory<T, F>(&mut self, factory: F) -> TypeMetadata
    where
        T: Component + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.types.register_factory::<T, F>(
            RegisteredTypeKind::Component,
            RegistrationSource::User,
            type_name::<T>(),
            factory,
        )
    }

    pub fn register_component_object_factory<T, F>(&mut self, factory: F) -> TypeMetadata
    where
        T: Component + 'static,
        F: Fn(&mut Object) -> bool + Send + Sync + 'static,
    {
        self.types.register_object_factory::<T, F>(
            RegisteredTypeKind::Component,
            RegistrationSource::User,
            type_name::<T>(),
            factory,
        )
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
        self.types
            .register_component_named_factory::<T, F>(type_name, factory)
    }

    pub fn register_builtin_component<T: Component + 'static>(&mut self) -> TypeMetadata {
        self.types.register_builtin_component::<T>()
    }

    pub fn register_builtin_component_factory<T, F>(&mut self, factory: F) -> TypeMetadata
    where
        T: Component + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.types.register_factory::<T, F>(
            RegisteredTypeKind::Component,
            RegistrationSource::BuiltIn,
            type_name::<T>(),
            factory,
        )
    }

    pub fn register_builtin_component_object_factory<T, F>(&mut self, factory: F) -> TypeMetadata
    where
        T: Component + 'static,
        F: Fn(&mut Object) -> bool + Send + Sync + 'static,
    {
        self.types.register_object_factory::<T, F>(
            RegisteredTypeKind::Component,
            RegistrationSource::BuiltIn,
            type_name::<T>(),
            factory,
        )
    }

    pub fn register_component_named<T: Component + 'static>(
        &mut self,
        type_name: &'static str,
    ) -> TypeMetadata {
        self.types.register_component_named::<T>(type_name)
    }

    pub fn register_script<T: Script + 'static>(&mut self) -> TypeMetadata {
        self.types.register_script::<T>()
    }

    pub fn register_script_factory<T, F>(&mut self, factory: F) -> TypeMetadata
    where
        T: Script + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.types.register_factory::<T, F>(
            RegisteredTypeKind::Script,
            RegistrationSource::User,
            type_name::<T>(),
            factory,
        )
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
        self.types
            .register_script_named_factory::<T, F>(type_name, factory)
    }

    pub fn register_builtin_script<T: Script + 'static>(&mut self) -> TypeMetadata {
        self.types.register_builtin_script::<T>()
    }

    pub fn register_builtin_script_factory<T, F>(&mut self, factory: F) -> TypeMetadata
    where
        T: Script + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.types.register_factory::<T, F>(
            RegisteredTypeKind::Script,
            RegistrationSource::BuiltIn,
            type_name::<T>(),
            factory,
        )
    }

    pub fn register_script_named<T: Script + 'static>(
        &mut self,
        type_name: &'static str,
    ) -> TypeMetadata {
        self.types.register_script_named::<T>(type_name)
    }

    pub fn register_archetype<T>(&mut self) -> ArchetypeMetadata
    where
        T: RunaArchetype,
    {
        self.archetypes.register::<T>()
    }

    pub fn register_object_def<T>(&mut self) -> ObjectDefMetadata
    where
        T: ObjectDef,
    {
        self.archetypes.register_def::<T>()
    }

    pub fn register_archetype_named<F>(
        &mut self,
        name: impl Into<Arc<str>>,
        factory: F,
    ) -> ArchetypeMetadata
    where
        F: Fn() -> Object + Send + Sync + 'static,
    {
        self.archetypes.register_named(name, factory)
    }

    pub fn register_object_def_named<F>(
        &mut self,
        name: impl Into<Arc<str>>,
        factory: F,
    ) -> ObjectDefMetadata
    where
        F: Fn() -> Object + Send + Sync + 'static,
    {
        self.archetypes.register_named(name, factory)
    }

    pub fn spawn_archetype_by_key(
        &self,
        world: &mut World,
        key: &ArchetypeKey,
    ) -> Option<ObjectId> {
        self.archetypes.spawn_by_key(world, key)
    }

    pub fn spawn_archetype_by_name(&self, world: &mut World, name: &str) -> Option<ObjectId> {
        self.archetypes.spawn_by_name(world, name)
    }

    pub fn spawn_object_def_by_key(
        &self,
        world: &mut World,
        key: &ObjectDefKey,
    ) -> Option<ObjectId> {
        self.archetypes.spawn_by_key(world, key)
    }

    pub fn spawn_object_def_by_name(&self, world: &mut World, name: &str) -> Option<ObjectId> {
        self.archetypes.spawn_by_name(world, name)
    }

    pub fn add_type_to_object(&self, object: &mut Object, type_id: TypeId) -> bool {
        self.types.add_to_object(object, type_id)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ArchetypeKey, RegisteredTypeKind, RegistrationSource, RunaArchetype, RuntimeRegistry,
        TypeRegistry,
    };
    use crate::{
        components::{SerializedFieldAccess, Transform},
        ocs::{Object, Script, ScriptContext, World},
    };

    struct TestScript;
    impl SerializedFieldAccess for TestScript {}

    impl Script for TestScript {
        fn update(&mut self, _ctx: &mut ScriptContext, _dt: f32) {}
    }

    #[test]
    fn registry_tracks_component_and_script_metadata() {
        let mut registry = TypeRegistry::new();
        let transform = registry.register_component::<Transform>();
        let script = registry.register_script::<TestScript>();

        assert_eq!(transform.kind(), RegisteredTypeKind::Component);
        assert_eq!(script.kind(), RegisteredTypeKind::Script);
        assert_eq!(transform.source(), RegistrationSource::User);
        assert_eq!(script.source(), RegistrationSource::User);
        assert!(registry.get_component::<Transform>().is_some());
        assert!(registry.get_script::<TestScript>().is_some());
        assert!(registry.get_by_name(transform.type_name()).is_some());
        assert!(registry.get_by_name(script.type_name()).is_some());
    }

    #[test]
    fn registry_tracks_builtin_type_origin() {
        let mut registry = TypeRegistry::new();
        let transform = registry.register_builtin_component::<Transform>();

        assert_eq!(transform.kind(), RegisteredTypeKind::Component);
        assert_eq!(transform.source(), RegistrationSource::BuiltIn);
        assert_eq!(registry.registered_builtin_types().len(), 1);
        assert_eq!(registry.registered_user_types().len(), 0);
    }

    #[test]
    fn runtime_registry_spawns_registered_archetype() {
        let mut registry = RuntimeRegistry::new();
        registry.register_archetype_named("test", || Object::new("Test"));

        let mut world = crate::World::default();
        let object_id = registry.spawn_archetype_by_name(&mut world, "test");

        assert!(object_id.is_some());
        assert_eq!(world.query::<Transform>().len(), 1);
    }

    struct TypedTestArchetype;

    impl RunaArchetype for TypedTestArchetype {
        fn key() -> ArchetypeKey {
            ArchetypeKey::new("typed_test")
        }

        fn create(world: &mut World) -> crate::ocs::ObjectId {
            world.spawn(Object::new("TypedTest"))
        }
    }

    #[test]
    fn runtime_registry_tracks_typed_archetypes_by_key_and_name() {
        let mut registry = RuntimeRegistry::new();
        let metadata = registry.register_archetype::<TypedTestArchetype>();
        let mut world = crate::World::default();

        let object_id = registry.spawn_archetype_by_key(&mut world, metadata.key());

        assert!(object_id.is_some());
        assert_eq!(metadata.key().as_str(), "typed_test");
        assert_eq!(metadata.name(), "typed_test");
        assert_eq!(metadata.source(), RegistrationSource::User);
        assert_eq!(world.query::<Transform>().len(), 1);
        assert!(registry.archetypes().contains_name("typed_test"));
    }
}
