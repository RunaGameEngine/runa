use crate::ocs::ScriptContext;
use serde::{Deserialize, Serialize};
use std::any::Any;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentRuntimeKind {
    Component,
    Script,
}

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
    /// Reference to another object by name (resolved at load time).
    /// Similar to Unity's GameObject reference in serialized data.
    ObjectRef(String),
}

pub trait SerializedFieldAccess {
    fn serialized_fields(&self) -> Vec<SerializedField> {
        Vec::new()
    }

    fn set_serialized_field(&mut self, _field_name: &str, _value: SerializedFieldValue) -> bool {
        false
    }
}

pub trait Component: Any + SerializedFieldAccess {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn runtime_kind(&self) -> ComponentRuntimeKind {
        ComponentRuntimeKind::Component
    }

    fn runtime_type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn on_start(&mut self, _ctx: &mut ScriptContext) {}

    fn on_update(&mut self, _ctx: &mut ScriptContext, _dt: f32) {}

    fn on_late_update(&mut self, _ctx: &mut ScriptContext, _dt: f32) {}
}
