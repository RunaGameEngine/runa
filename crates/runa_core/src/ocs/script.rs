use crate::components::{Collider2D, Component};
use crate::ocs::{Object, ObjectHandle, ObjectId, ScriptCommands, World};
use glam::Vec2;

pub struct ScriptContext<'a> {
    object: &'a mut Object,
    world: &'a mut World,
}

impl<'a> ScriptContext<'a> {
    pub(crate) fn new(object: &'a mut Object, world: &'a mut World) -> Self {
        Self { object, world }
    }

    pub fn id(&self) -> Option<ObjectId> {
        self.object.id()
    }

    pub fn name(&self) -> &str {
        &self.object.name
    }

    pub fn set_name(&mut self, name: impl Into<String>) {
        self.object.name = name.into();
    }

    pub fn get_component<T: 'static>(&self) -> Option<&T> {
        self.object.get_component::<T>()
    }

    pub fn get_component_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.object.get_component_mut::<T>()
    }

    pub fn add_component<T: Component>(&mut self, component: T) -> &mut Self {
        self.object.add_component(component);
        self
    }

    pub fn handle(&self) -> Option<ObjectHandle> {
        self.object.handle()
    }

    pub fn object(&self) -> &Object {
        self.object
    }

    pub fn object_mut(&mut self) -> &mut Object {
        self.object
    }

    pub fn world(&self) -> &World {
        self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        self.world
    }

    pub fn commands(&mut self) -> ScriptCommands<'_> {
        ScriptCommands::new(self.world)
    }

    pub fn get_object(&self, id: ObjectId) -> Option<&Object> {
        if self.object.id() == Some(id) {
            Some(&*self.object)
        } else {
            self.world.object(id)
        }
    }

    pub fn find_by_name(&self, name: &str) -> Option<ObjectId> {
        self.world.find_by_name(name)
    }

    pub fn find_by_path(&self, path: &str) -> Option<ObjectId> {
        self.world.find_by_path(path)
    }

    pub fn get_component_in_children<T: 'static>(&self) -> Option<&T> {
        self.object.get_component_in_children(self.world)
    }

    pub fn get_component_in_parent<T: 'static>(&self) -> Option<&T> {
        self.object.get_component_in_parent(self.world)
    }

    pub fn find_first_with<T: 'static>(&self) -> Option<ObjectId> {
        self.world.find_first_with::<T>()
    }

    pub fn find_all_with<T: 'static>(&self) -> Vec<ObjectId> {
        self.world.find_all_with::<T>()
    }

    pub fn colliding_2d(&mut self, world: &World) -> bool {
        self.object.colliding_2d(world)
    }

    pub fn would_collide_2d_at(&mut self, world: &World, center: Vec2) -> bool {
        self.object.would_collide_2d_at(world, center)
    }

    pub fn overlaps_collider_2d(&self, center: Vec2, collider: &Collider2D) -> bool {
        let self_ptr = self.object as *const Object;
        self.world
            .overlaps_collider_2d(center, collider, Some(self_ptr))
    }

    pub fn emit_event<E: 'static>(&self, event: E) {
        self.world.events.borrow_mut().emit(event);
    }

    pub fn subscribe_to_event<E: 'static>(&self, callback: impl Fn(&E) + 'static) {
        self.world.events.borrow_mut().subscribe(callback);
    }
}

/// Script component that adds custom behavior to an object.
///
/// Scripts are attachable behavior components.
/// They follow a deterministic lifecycle and operate on an already-composed object.
pub trait Script: Send + Sync + 'static {
    fn start(&mut self, _ctx: &mut ScriptContext) {}
    fn update(&mut self, _ctx: &mut ScriptContext, _dt: f32) {}
    fn late_update(&mut self, _ctx: &mut ScriptContext, _dt: f32) {}
}

impl<T: Script> Component for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn on_start(&mut self, ctx: &mut ScriptContext) {
        Script::start(self, ctx);
    }

    fn on_update(&mut self, ctx: &mut ScriptContext, dt: f32) {
        Script::update(self, ctx, dt);
    }

    fn on_late_update(&mut self, ctx: &mut ScriptContext, dt: f32) {
        Script::late_update(self, ctx, dt);
    }
}
