use crate::ocs::{Object, ObjectId, World};

pub enum WorldCommand {
    Despawn(ObjectId),
    Spawn(Object),
}

pub struct ScriptCommands<'a> {
    world: &'a mut World,
}

impl<'a> ScriptCommands<'a> {
    pub(crate) fn new(world: &'a mut World) -> Self {
        Self { world }
    }

    pub fn despawn(&mut self, object_id: ObjectId) {
        self.world.queue_command(WorldCommand::Despawn(object_id));
    }

    pub fn spawn(&mut self, object: Object) {
        self.world.queue_command(WorldCommand::Spawn(object));
    }
}
