use crate::ocs::script::Script;
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub struct Object {
    components: HashMap<TypeId, Box<dyn Any>>,
    pub script: Option<Box<dyn Script>>,
}

impl Object {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            script: None,
        }
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
}
