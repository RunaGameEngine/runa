use runa_engine::runa_app::{RunaApp, RunaWindowConfig};
use runa_engine::runa_ecs;
use runa_engine::runa_core::components::{Camera, MeshRenderer, Mesh, Transform};
use runa_engine::runa_core::glam::{Quat, Vec3};
use runa_engine::system;

#[system]
fn rotate_cubes(world: &mut runa_ecs::World) {
    let dt = 1.0 / 60.0;
    for (_, transform) in world.query_mut::<runa_ecs::W<Transform>>() {
        transform.rotation *= Quat::from_rotation_y(0.5 * dt);
    }
}

fn main() {
    let mut world = runa_ecs::World::new();

    world.spawn((
        Transform {
            position: Vec3::new(-1.5, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            ..Transform::default()
        },
        MeshRenderer::new(Mesh::cube(1.0)),
    ));

    world.spawn((
        Transform {
            position: Vec3::new(1.5, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::new(-2.0, 2.0, 2.0),
            ..Transform::default()
        },
        MeshRenderer::new(Mesh::cube(1.0)),
    ));

    world.spawn((
        Camera::new_perspective(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::ZERO,
            Vec3::Y,
            75.0,
            0.1,
            1000.0,
        ),
    ));

    let config = RunaWindowConfig {
        title: "Runa 3D Sandbox - rotating cubes".to_string(),
        width: 1280,
        height: 720,
        fullscreen: false,
        vsync: false,
        show_fps_in_title: true,
        window_icon: None,
    };

    let _ = RunaApp::run_with_world(config, world);
}
