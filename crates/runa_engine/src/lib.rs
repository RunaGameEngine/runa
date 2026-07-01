mod engine;
pub mod prelude;

pub use runa_app;
pub use runa_asset;
pub use runa_core;

pub use engine::Engine;
pub use runa_core::codefirst::{Bundle, QueryMut, QueryRef};
pub use runa_core::Color;
pub use runa_macros::Component;

/// Create a `SpriteRenderer` from a path relative to the crate root.
///
/// Loads the texture at compile time (validates the file exists) and
/// stores both the handle and the path — no duplicate path strings needed.
///
/// ```ignore
/// use runa_engine::sprite;
///
/// let spr = sprite!("assets/art/Tester1.png");
/// ```
#[macro_export]
macro_rules! sprite {
    ($path:literal) => {
        $crate::runa_core::components::SpriteRenderer::new(Some(
            $crate::runa_asset::load_image!($path),
        ))
    };
}
