use crate::ocs::{Object, World};

/// Script component that adds custom behavior to an object.
///
/// Scripts are the primary way to implement game logic in Runa.
/// They follow a deterministic lifecycle and have access to the object's components.
///
/// # Lifecycle
/// 1. `construct()` - Called immediately after script creation (before object enters world)
/// 2. `start()` - Called once on the first tick after object is added to the world
/// 3. `update()` - Called every tick while object exists in the world
///
/// # Example
/// ```
/// struct Player {
///     speed: f32,
/// }
///
/// impl Script for Player {
///     fn construct(&self, object: &mut Object) {
///         // Initialize components before object enters world
///         object.add_component(Transform::new());
///     }
///
///     fn start(&mut self, object: &mut Object) {
///         // Access components after object is in world
///         println!("Player spawned at {:?}", object.get_component::<Transform>().unwrap().position);
///     }
///
///     fn update(&mut self, object: &mut Object, dt: f32, world: &mut World) {
///         // Game logic runs every tick
///         if Input::is_key_pressed(KeyCode::W) {
///             let transform = object.get_component_mut::<Transform>().unwrap();
///             transform.position.y -= self.speed * dt;
///         }
///     }
/// }
/// ```
pub trait Script: 'static {
    /// Called immediately after script creation, before the object is added to the world.
    ///
    /// Use this method to:
    /// - Initialize components that the object requires
    /// - Set up initial state that doesn't depend on world context
    /// - Configure object hierarchy (children/parents)
    ///
    /// Note: World systems and other objects are NOT accessible here.
    fn construct(&self, _object: &mut Object) {}

    /// Called once on the first tick after the object is added to the world.
    ///
    /// Use this method to:
    /// - Access other objects in the world
    /// - Query world state (e.g., find nearest enemy)
    /// - Start coroutines or timed events
    /// - Initialize physics/collision state
    ///
    /// This is the earliest point where the object is fully integrated into the simulation.
    fn start(&mut self, _object: &mut Object) {}

    /// Called every tick while the object exists in the world.
    ///
    /// Use this method for:
    /// - Input handling (`Input::is_key_pressed()`)
    /// - Movement and animation
    /// - AI behavior and decision making
    /// - Physics updates (use fixed timestep for determinism)
    /// - Audio playback via `world.play_sound()`
    ///
    /// Parameters:
    /// - `dt`: Delta time in seconds since last frame (use for frame-rate independent movement)
    /// - `world`: Mutable reference to the world for accessing systems and playing sounds
    fn update(&mut self, _object: &mut Object, _dt: f32, _world: &mut World) {}
}
