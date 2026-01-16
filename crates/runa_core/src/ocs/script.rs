use crate::ocs::{context::Context, object::Object};

/// Impl to script object
pub trait Script: 'static {
    fn construct(&self, _object: &mut Object) {}
    fn start(&mut self, _ctx: &mut Context) {}
    fn update(&mut self, _ctx: &mut Context, _dt: f32) {}
}
