use crate::components::transform::Transform;
use runa_asset::handle::Handle;
use runa_asset::texture::TextureAsset;
use runa_render_api::queue::RenderQueue;

pub struct Sprite {
    pub transform: Transform,
    pub texture: Handle<TextureAsset>, // CPU-side handle
}

pub struct CoreRenderer {
    pub queue: RenderQueue,
}

impl CoreRenderer {
    pub fn new() -> Self {
        Self {
            queue: RenderQueue::new(),
        }
    }

    pub fn submit_sprite(&mut self, sprite: &Sprite, transform: &Transform) {
        self.queue
            .draw_sprite(sprite.texture.clone(), transform.matrix());
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }
}
