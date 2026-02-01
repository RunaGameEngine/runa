use crate::{input::InputState, ocs::object::Object};

/// Impl to scripting object
pub trait Script: 'static {
    fn construct(&self, _object: &mut Object) {}
    fn start(&mut self, _object: &mut Object) {}
    fn update(&mut self, _object: &mut Object, _dt: f32) {}
    fn input(&mut self, _object: &mut Object, _input: &InputState) {}
}
