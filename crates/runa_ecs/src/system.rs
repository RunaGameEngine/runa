use crate::World;

pub trait System: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn run(&mut self, world: &mut World);
}

pub struct FunctionSystem {
    name: &'static str,
    func: fn(&mut World),
}

impl FunctionSystem {
    pub fn new(name: &'static str, func: fn(&mut World)) -> Self {
        Self { name, func }
    }
}

impl System for FunctionSystem {
    fn name(&self) -> &'static str { self.name }
    fn run(&mut self, world: &mut World) { (self.func)(world) }
}

// ─── Auto-registration via inventory ─────────────────────────

pub struct SystemDescriptor {
    pub name: &'static str,
    pub func: fn(&mut World),
}

inventory::collect!(SystemDescriptor);

// ─── SystemStage ─────────────────────────────────────────────

pub struct SystemStage {
    pub name: &'static str,
    pub systems: Vec<Box<dyn System>>,
}

impl SystemStage {
    pub fn new(name: &'static str) -> Self {
        Self { name, systems: Vec::new() }
    }

    pub fn add_system(&mut self, system: impl System) -> &mut Self {
        self.systems.push(Box::new(system));
        self
    }

    pub fn run(&mut self, world: &mut World) {
        for sys in &mut self.systems {
            sys.run(world);
        }
    }
}

// ─── Scheduler ───────────────────────────────────────────────

pub struct Scheduler {
    pub stages: Vec<SystemStage>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    pub fn add_stage(&mut self, stage: SystemStage) -> &mut Self {
        self.stages.push(stage);
        self
    }

    pub fn collect_registered_systems(&mut self, stage_name: &'static str) -> &mut Self {
        let mut stage = SystemStage::new(stage_name);
        for desc in inventory::iter::<SystemDescriptor> {
            stage.add_system(FunctionSystem::new(desc.name, desc.func));
        }
        self.add_stage(stage);
        self
    }

    pub fn run(&mut self, world: &mut World) {
        for stage in &mut self.stages {
            stage.run(world);
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self { Self::new() }
}
