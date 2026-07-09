use crate::components::{Collider2D, Component, Transform};
use crate::ocs::{ScriptContext, World};
use glam::Vec2;
use std::any::TypeId;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

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
    pending_children: Vec<Object>,
    pub(crate) components: Vec<(TypeId, Box<dyn Component>)>,
    world: Option<Weak<RefCell<World>>>,
}

impl Object {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            parent: None,
            children: Vec::new(),
            pending_children: Vec::new(),
            components: vec![(TypeId::of::<Transform>(), Box::new(Transform::default()))],
            world: None,
        }
    }

    pub fn component_type_ids(&self) -> Vec<TypeId> {
        self.components.iter().map(|(tid, _)| *tid).collect()
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

    pub(crate) fn drain_pending_children(&mut self) -> Vec<Object> {
        std::mem::take(&mut self.pending_children)
    }

    pub fn set_world(&mut self, world: Rc<RefCell<World>>) {
        self.world = Some(Rc::downgrade(&world));
    }

    pub fn get_world(&self) -> Option<Rc<RefCell<World>>> {
        self.world.as_ref()?.upgrade()
    }

    pub(crate) fn set_id(&mut self, id: ObjectId) {
        self.id = Some(id);
    }

    pub fn with<T: Component>(mut self, part: T) -> Self {
        self.add_component(part);
        self
    }

    pub fn with_child(mut self, child: Object) -> Self {
        self.pending_children.push(child);
        self
    }

    pub fn add_child(&mut self, child: Object) {
        self.pending_children.push(child);
    }

    pub fn find_in_world_by_name(&self, name: &str) -> Option<ObjectId> {
        self.get_world()?.borrow().find_by_name(name)
    }

    pub fn find_in_world_by_path(&self, path: &str) -> Option<ObjectId> {
        self.get_world()?.borrow().find_by_path(path)
    }

    pub fn add_component<T: Component>(&mut self, component: T) -> &mut Object {
        let type_id = TypeId::of::<T>();
        if let Some(pos) = self.components.iter().position(|(tid, _)| *tid == type_id) {
            self.components[pos] = (type_id, Box::new(component));
        } else {
            self.components.push((type_id, Box::new(component)));
        }
        self
    }

    pub fn get_component<T: 'static>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.components
            .iter()
            .find(|(tid, _)| *tid == type_id)
            .and_then(|(_, c)| c.as_any().downcast_ref())
    }

    pub fn get_component_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.components
            .iter_mut()
            .find(|(tid, _)| *tid == type_id)
            .and_then(|(_, c)| c.as_any_mut().downcast_mut())
    }

    /// Recursively search this object and its children for a component of type T.
    /// Uses an explicit world reference to avoid RefCell double-borrow.
    pub fn get_component_in_children<'a, T: 'static>(&'a self, world: &'a World) -> Option<&'a T> {
        if let Some(component) = self.get_component::<T>() {
            return Some(component);
        }
        for child_id in &self.children {
            if let Some(child) = world.object(*child_id) {
                if let Some(component) = child.get_component_in_children::<T>(world) {
                    return Some(component);
                }
            }
        }
        None
    }

    /// Recursively search this object and its parents for a component of type T.
    /// Uses an explicit world reference to avoid RefCell double-borrow.
    pub fn get_component_in_parent<'a, T: 'static>(&'a self, world: &'a World) -> Option<&'a T> {
        if let Some(component) = self.get_component::<T>() {
            return Some(component);
        }
        if let Some(parent_id) = self.parent {
            if let Some(parent) = world.object(parent_id) {
                return parent.get_component_in_parent::<T>(world);
            }
        }
        None
    }

    pub fn has_component<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.components.iter().any(|(tid, _)| *tid == type_id)
    }

    pub fn has_component_type_id(&self, type_id: TypeId) -> bool {
        self.components.iter().any(|(tid, _)| *tid == type_id)
    }

    pub fn remove_component_type_id(&mut self, type_id: TypeId) -> bool {
        if type_id == TypeId::of::<Transform>() {
            return false;
        }
        let Some(pos) = self.components.iter().position(|(tid, _)| *tid == type_id) else {
            return false;
        };
        self.components.swap_remove(pos);
        true
    }

    pub(crate) fn run_start(&mut self, world: *mut World) {
        let world = unsafe { &mut *world };
        for i in (0..self.components.len()).rev() {
            let (type_id, mut component) = self.components.swap_remove(i);
            let mut ctx = ScriptContext::new(self, world);
            component.on_start(&mut ctx);
            self.components.push((type_id, component));
        }
    }

    pub(crate) fn run_update(&mut self, world: *mut World, dt: f32) {
        let world = unsafe { &mut *world };
        for i in (0..self.components.len()).rev() {
            let (type_id, mut component) = self.components.swap_remove(i);
            let mut ctx = ScriptContext::new(self, world);
            component.on_update(&mut ctx, dt);
            self.components.push((type_id, component));
        }
    }

    pub(crate) fn run_late_update(&mut self, world: *mut World, dt: f32) {
        let world = unsafe { &mut *world };
        for i in (0..self.components.len()).rev() {
            let (type_id, mut component) = self.components.swap_remove(i);
            let mut ctx = ScriptContext::new(self, world);
            component.on_late_update(&mut ctx, dt);
            self.components.push((type_id, component));
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
