use crate::components::{Collider2D, Transform};
use crate::ocs::{Script, World};
use glam::Vec2;
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub struct Object {
    pub name: String,
    pub transform: Transform,
    components: HashMap<TypeId, Box<dyn Any>>,
    pub script: Option<Box<dyn Script>>,
    world_ptr: *mut World,
}

impl Object {
    pub fn new() -> Self {
        Self {
            name: String::default(),
            transform: Transform::default(),
            components: HashMap::new(),
            script: None,
            world_ptr: std::ptr::null_mut(),
        }
    }

    /// Set the world pointer for this object (called when object is added to world)
    pub fn set_world(&mut self, world: &mut World) {
        self.world_ptr = world as *mut World;
    }

    /// Get mutable reference to the world
    ///
    /// # Safety
    /// This is safe to call as long as the object is part of a world and
    /// no other mutable borrows of the world exist at the same time.
    pub fn get_world_mut(&mut self) -> Option<&mut World> {
        if self.world_ptr.is_null() {
            None
        } else {
            Some(unsafe { &mut *self.world_ptr })
        }
    }

    pub fn get_transform(&self) -> &Transform {
        &self.transform
    }

    pub fn get_transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    /// Adding component to object. Only one per object!
    pub fn add_component<T: 'static>(&mut self, component: T) -> &mut Object {
        let type_id = TypeId::of::<T>();
        assert!(
            !self.components.contains_key(&type_id),
            "Component already exists {type_id:?}"
        );
        self.components.insert(type_id, Box::new(component));
        self
    }

    /// To get component by Type if it exist
    pub fn get_component<T: 'static>(&self) -> Option<&T> {
        // let type_id = TypeId::of::<T>();
        // assert!(
        //     !self.components.contains_key(&type_id),
        //     "Component not exists"
        // );
        self.components
            .get(&TypeId::of::<T>())
            .and_then(|c| c.downcast_ref())
    }

    pub fn get_component_mut<T: 'static>(&mut self) -> Option<&mut T> {
        // let type_id = TypeId::of::<T>();
        // assert!(
        //     !self.components.contains_key(&type_id),
        //     "Component not exists"
        // );
        self.components
            .get_mut(&TypeId::of::<T>())
            .and_then(|c| c.downcast_mut())
    }

    pub fn set_script(&mut self, script: Box<dyn Script>) {
        self.script = Some(script);
    }

    pub fn is_colliding_2d(&mut self) -> bool {
        let center = self
            .get_component::<Transform>()
            .map(|transform| transform.position.truncate())
            .unwrap_or(Vec2::ZERO);
        self.would_collide_2d_at(center)
    }

    pub fn would_collide_2d_at(&mut self, center: Vec2) -> bool {
        let Some(collider) = self.get_component::<Collider2D>().copied() else {
            return false;
        };

        let self_ptr = self as *const Object;
        self.get_world_mut()
            .map(|world| world.overlaps_collider_2d(center, &collider, Some(self_ptr)))
            .unwrap_or(false)
    }
}
