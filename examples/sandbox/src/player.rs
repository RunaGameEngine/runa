use runa_core::components::sprite_renderer::SpriteRenderer;
use runa_core::ocs::script::Script;

pub struct Player {
    pub speed: f32,
}

impl Script for Player {
    fn construct(&self, _object: &mut runa_core::ocs::object::Object) {
        _object.add_component(SpriteRenderer::new());
        _object.add_component(SpriteRenderer::new());
        _object.add_component(SpriteRenderer::new());
    }
}
