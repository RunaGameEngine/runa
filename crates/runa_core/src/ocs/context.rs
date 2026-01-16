use crate::ocs::{object::Object, world::World};

pub struct Context<'a> {
    pub object: &'a mut Object,
    pub world: &'a mut World,
}
