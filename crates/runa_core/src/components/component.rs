use crate::ocs::ScriptContext;
use serde::{Deserialize, Serialize};
use std::any::Any;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedField {
    pub name: String,
    pub value: SerializedFieldValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializedFieldValue {
    Bool(bool),
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    String(String),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    ObjectRef(String),
}

/// Core trait for all ECS components.
///
/// Any `Send + Sync + 'static` type can be a component — just derive `Component`.
///
/// # Lifecycle
/// - `on_start()`: called once when the entity enters the world
/// - `on_update()`: called every frame
/// - `on_late_update()`: called after all regular updates
pub trait Component: Any + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn on_start(&mut self, _ctx: &mut ScriptContext) {}
    fn on_update(&mut self, _ctx: &mut ScriptContext, _dt: f32) {}
    fn on_late_update(&mut self, _ctx: &mut ScriptContext, _dt: f32) {}
}
