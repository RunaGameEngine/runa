use std::sync::Arc;

use crate::components::Transform;
use runa_asset::Handle;
use runa_asset::TextureAsset;
use runa_render_api::RenderQueue;

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
        self.queue.push_sprite(
            Arc::from(sprite.texture.clone()),
            transform.position,
            transform.rotation,
            transform.scale,
        );
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }
}
