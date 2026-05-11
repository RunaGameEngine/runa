use crate::components::{Collider2D, Component, Transform};
use crate::ocs::{ScriptContext, World};
use crate::registry::RuntimeRegistry;
use glam::Vec2;
use std::any::TypeId;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::Arc;

pub type ObjectId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectHandle {
    id: ObjectId,
}

impl ObjectHandle {
    pub fn new(id: ObjectId) -> Self {
        Self { id }
    }

    pub fn id(self) -> ObjectId {
        self.id
    }
}

pub struct Object {
    id: Option<ObjectId>,
    pub name: String,
    parent: Option<ObjectId>,
    children: Vec<ObjectId>,
    components: HashMap<TypeId, Box<dyn Component>>,
    world: Option<Weak<RefCell<World>>>,
}

pub struct ObjectBuilder {
    object: Object,
}

#[derive(Debug, Clone, Copy)]
pub struct ObjectComponentInfo {
    type_id: TypeId,
    type_name: &'static str,
    kind: crate::components::ComponentRuntimeKind,
}

impl ObjectComponentInfo {
    pub fn type_id(self) -> TypeId {
        self.type_id
    }

    pub fn type_name(self) -> &'static str {
        self.type_name
    }

    pub fn kind(self) -> crate::components::ComponentRuntimeKind {
        self.kind
    }
}

impl ObjectBuilder {
    pub fn new() -> Self {
        Self {
            object: Object::new(""),
        }
    }

    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.object.name = name.into();
        self
    }

    pub fn with<T: Component>(&mut self, component: T) -> &mut Self {
        self.object.add_component(component);
        self
    }

    pub fn build(self) -> Object {
        self.object
    }
}

impl Default for ObjectBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Object {
    pub fn new(name: impl Into<String>) -> Self {
        let mut components: HashMap<TypeId, Box<dyn Component>> = HashMap::new();
        components.insert(TypeId::of::<Transform>(), Box::new(Transform::default()));

        Self {
            id: None,
            name: name.into(),
            parent: None,
            children: Vec::new(),
            components,
            world: None,
        }
    }

    pub fn component_type_ids(&self) -> Vec<TypeId> {
        self.components
            .keys()
            .copied()
            .collect()
    }

    pub fn empty() -> Self {
        Self::new("")
    }

    pub fn id(&self) -> Option<ObjectId> {
        self.id
    }

    pub fn handle(&self) -> Option<ObjectHandle> {
        self.id.map(ObjectHandle::new)
    }

    pub fn parent(&self) -> Option<ObjectId> {
        self.parent
    }

    pub fn children(&self) -> &[ObjectId] {
        &self.children
    }

    pub(crate) fn set_parent_id(&mut self, parent: Option<ObjectId>) {
        self.parent = parent;
    }

    pub(crate) fn add_child_id(&mut self, child: ObjectId) {
        if !self.children.contains(&child) {
            self.children.push(child);
        }
    }

    pub(crate) fn remove_child_id(&mut self, child: ObjectId) {
        self.children.retain(|candidate| *candidate != child);
    }

    pub(crate) fn clear_children(&mut self) {
        self.children.clear();
    }

    pub fn set_world(&mut self, world: Rc<RefCell<World>>) {
        self.world = Some(Rc::downgrade(&world));
    }

    pub fn get_world(&self) -> Option<Rc<RefCell<World>>> {
        self.world.as_ref()?.upgrade() // None, if World killed
    }

    pub(crate) fn set_id(&mut self, id: ObjectId) {
        self.id = Some(id);
    }

    pub fn with<T: Component>(mut self, part: T) -> Self {
        self.add_component(part);
        self
    }

    pub fn runtime_registry(&self) -> Option<Arc<RuntimeRegistry>> {
        self.get_world()
            .and_then(|world_rc| world_rc.borrow().runtime_registry_arc())
    }

    /// Add a component to the object. Only one component of a given type is allowed.
    pub fn add_component<T: Component>(&mut self, component: T) -> &mut Object {
        let type_id = TypeId::of::<T>();
        if type_id == TypeId::of::<Transform>() {
            self.components.insert(type_id, Box::new(component));
            return self;
        }

        assert!(
            !self.components.contains_key(&type_id),
            "Component already exists {type_id:?}"
        );
        self.components.insert(type_id, Box::new(component));
        self
    }

    pub fn get_component<T: 'static>(&self) -> Option<&T> {
        self.components
            .get(&TypeId::of::<T>())
            .and_then(|c| c.as_any().downcast_ref())
    }

    pub fn get_component_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.components
            .get_mut(&TypeId::of::<T>())
            .and_then(|c| c.as_any_mut().downcast_mut())
    }

    pub fn has_component<T: 'static>(&self) -> bool {
        self.get_component::<T>().is_some()
    }

    pub fn has_component_type_id(&self, type_id: TypeId) -> bool {
        self.components.contains_key(&type_id)
    }

    pub fn with_component_mut_by_type_id<R>(
        &mut self,
        type_id: TypeId,
        apply: impl FnOnce(&mut dyn Component) -> R,
    ) -> Option<R> {
        let component = self.components.get_mut(&type_id)?;
        Some(apply(component.as_mut()))
    }

    pub fn with_component_by_type_id<R>(
        &self,
        type_id: TypeId,
        apply: impl FnOnce(&dyn Component) -> R,
    ) -> Option<R> {
        let component = self.components.get(&type_id)?;
        Some(apply(component.as_ref()))
    }

    pub fn remove_component_type_id(&mut self, type_id: TypeId) -> bool {
        if type_id == TypeId::of::<Transform>() {
            return false;
        }

        self.components.remove(&type_id).is_some()
    }

    pub fn component_infos(&self) -> Vec<ObjectComponentInfo> {
        self.components
            .iter()
            .map(|(type_id, component)| ObjectComponentInfo {
                type_id: *type_id,
                type_name: component.runtime_type_name(),
                kind: component.runtime_kind(),
            })
            .collect()
    }

    pub(crate) fn run_start(&mut self, world: *mut World) {
        let world = unsafe { &mut *world };
        let component_ids: Vec<TypeId> = self.components.keys().copied().collect();
        for type_id in component_ids {
            let Some(mut component) = self.components.remove(&type_id) else {
                continue;
            };
            let mut ctx = ScriptContext::new(self, world);
            component.on_start(&mut ctx);
            self.components.insert(type_id, component);
        }
    }

    pub(crate) fn run_update(&mut self, world: *mut World, dt: f32) {
        let world = unsafe { &mut *world };
        let component_ids: Vec<TypeId> = self.components.keys().copied().collect();
        for type_id in component_ids {
            let Some(mut component) = self.components.remove(&type_id) else {
                continue;
            };
            let mut ctx = ScriptContext::new(self, world);
            component.on_update(&mut ctx, dt);
            self.components.insert(type_id, component);
        }
    }

    pub(crate) fn run_late_update(&mut self, world: *mut World, dt: f32) {
        let world = unsafe { &mut *world };
        let component_ids: Vec<TypeId> = self.components.keys().copied().collect();
        for type_id in component_ids {
            let Some(mut component) = self.components.remove(&type_id) else {
                continue;
            };
            let mut ctx = ScriptContext::new(self, world);
            component.on_late_update(&mut ctx, dt);
            self.components.insert(type_id, component);
        }
    }

    pub fn colliding_2d(&mut self, world: &World) -> bool {
        let center = self
            .get_component::<Transform>()
            .map(|transform| transform.position.truncate())
            .unwrap_or(Vec2::ZERO);
        self.would_collide_2d_at(world, center)
    }

    pub fn would_collide_2d_at(&self, world: &World, center: Vec2) -> bool {
        let Some(collider) = self.get_component::<Collider2D>().copied() else {
            return false;
        };

        let self_ptr = self as *const Object;
        world.overlaps_collider_2d(center, &collider, Some(self_ptr))
    }
}
