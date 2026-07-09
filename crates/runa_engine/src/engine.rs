use std::cell::RefCell;
use std::rc::Rc;

use runa_core::ocs::World;

pub struct Engine;

impl Engine {
    pub fn new() -> Self {
        Self
    }

    pub fn create_world() -> Rc<RefCell<World>> {
        let world = World::default();
        Rc::new(RefCell::new(world))
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
