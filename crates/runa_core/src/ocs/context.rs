use crate::ocs::{object::Object, world::World};
/// Not in use yet
pub struct Context<'a> {
    pub object: &'a mut Object,
    pub world: &'a World,
}
