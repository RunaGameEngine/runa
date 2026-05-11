use runa_engine::{
    runa_core::{
        components::{SerializedFieldAccess, Transform},
        ocs::{Object, ObjectBuilder, Script, ScriptContext, World},
        RegisteredTypeKind, RegistrationSource,
    },
    Engine, ObjectDef, RunaArchetype, RunaComponent, RunaObjectDef, RunaScript,
};

#[derive(RunaArchetype)]
#[runa(name = "player")]
struct PlayerArchetype;

impl PlayerArchetype {
    pub fn create(world: &mut World) -> runa_engine::runa_core::ocs::ObjectId {
        world.spawn(Object::new("Player"))
    }
}

#[derive(RunaArchetype)]
struct BossEnemy;

impl BossEnemy {
    pub fn create(world: &mut World) -> runa_engine::runa_core::ocs::ObjectId {
        world.spawn(Object::new("Boss Enemy"))
    }
}

#[derive(Default, RunaComponent)]
struct Health {
    #[runa_editable]
    max: i32,
    #[runa_runtime]
    current: i32,
}

#[derive(Default, RunaScript)]
struct PlayerController;

impl Script for PlayerController {
    fn update(&mut self, _ctx: &mut ScriptContext, _dt: f32) {}
}

#[test]
fn engine_registers_builtin_types_automatically() {
    let engine = Engine::new();
    let transform = engine
        .runtime_registry()
        .types()
        .get_component::<Transform>()
        .copied()
        .expect("Transform should be registered by Engine::new()");

    assert_eq!(transform.kind(), RegisteredTypeKind::Component);
    assert_eq!(transform.source(), RegistrationSource::BuiltIn);
    assert!(!engine
        .runtime_registry()
        .types()
        .registered_builtin_types()
        .is_empty());
}

#[test]
fn engine_register_generic_registers_user_types() {
    let mut engine = Engine::new();
    let health = engine.register::<Health>();
    let controller = engine.register::<PlayerController>();

    assert_eq!(health.kind(), RegisteredTypeKind::Component);
    assert_eq!(controller.kind(), RegisteredTypeKind::Script);
    assert_eq!(health.source(), RegistrationSource::User);
    assert_eq!(controller.source(), RegistrationSource::User);
}

#[test]
fn typed_archetypes_register_and_spawn() {
    let mut engine = Engine::new();
    let player_metadata = engine.register_archetype::<PlayerArchetype>();
    let boss_metadata = engine.register_archetype::<BossEnemy>();
    let world_rc = engine.create_world();

    {
        let mut world = world_rc.borrow_mut();

        let player_id = world.spawn_archetype::<PlayerArchetype>();
        let boss_id = world.spawn_archetype::<BossEnemy>();
        let player_by_name = world.spawn_archetype_by_name("player");
        let boss_by_key = world.spawn_archetype_by_key(boss_metadata.key());

        assert_eq!(PlayerArchetype::archetype_name(), "player");
        assert_eq!(PlayerArchetype::archetype_key().as_str(), "player");
        assert_eq!(BossEnemy::archetype_key().as_str(), "boss_enemy");
        assert_eq!(player_metadata.key().as_str(), "player");
        assert_eq!(boss_metadata.name(), "boss_enemy");
        assert!(world.object(player_id).is_some());
        assert!(world.object(boss_id).is_some());
        assert!(player_by_name.is_some());
        assert!(boss_by_key.is_some());
    }
}

#[derive(RunaObjectDef)]
#[runa(name = "defined_player")]
struct DefinedPlayer;

impl ObjectDef for DefinedPlayer {
    fn build(object: &mut ObjectBuilder) {
        object.name("Defined Player").with(Health {
            max: 100,
            current: 100,
        });
    }
}

#[test]
fn object_defs_spawn_by_type_and_name_with_component_access() {
    let mut engine = Engine::new();
    let metadata = engine.register_object_def::<DefinedPlayer>();
    let world_rc = engine.create_world();

    let by_type = world_rc.borrow_mut().spawn_def::<DefinedPlayer>();
    let by_name = world_rc.borrow_mut()
        .spawn("defined_player")
        .expect("definition should spawn by name");

    assert_eq!(metadata.key().as_str(), "defined_player");
    assert_eq!(DefinedPlayer::object_def_name(), "defined_player");
    assert_eq!(world_rc.borrow().get::<Health>(by_type).unwrap().max, 100);
    world_rc.borrow_mut().get_mut::<Health>(by_name).unwrap().max = 150;
    assert_eq!(world_rc.borrow().get::<Health>(by_name).unwrap().max, 150);

    let fields = world_rc.borrow().get::<Health>(by_type).unwrap().serialized_fields();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].name, "max");
}

