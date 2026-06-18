use bevy::{DefaultPlugins, app::{App, FixedUpdate, Startup, Update}, asset::Assets, camera::Camera2d, color::Color, ecs::{component::Component, query::With, resource::Resource, system::{Commands, Query, Res, ResMut}}, math::{Vec3, primitives::{Circle, ToRing}}, mesh::{Mesh, Mesh2d}, sprite_render::{ColorMaterial, MeshMaterial2d}, transform::components::Transform};
use bevy::math::Vec2;

pub enum Shape2D {
    /// (radius)
    Circle(f32),
    /// (radius, thickness)
    HollowCircle(f32, f32),
}

#[derive(Component)]
pub struct RenderObject {
    pub pos: Vec2,
    pub shape: Shape2D,
    pub color: Color,
}

pub trait RenderEnv2D {
    fn physics_tick(&mut self) -> ();
    fn render_infos(&self) -> Vec<RenderObject>;
}

fn setup<R: RenderEnv2D + Resource>(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    render_env: Res<R>,
) {
    cmd.spawn(Camera2d);

    render_env.render_infos().into_iter()
    .map(|object| {
        (match object.shape {
            Shape2D::Circle(radius) => meshes.add(Circle::new(radius)),
            Shape2D::HollowCircle(radius, thickness) => meshes.add(Circle::new(radius).to_ring(thickness)),
        }, object)
    }).for_each(|(mesh, object)| {
        cmd.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(object.color)),
            Transform::from_xyz(object.pos.x, object.pos.y, 0.0),
            object
        ));
    });
}

fn update_object_infos<R: RenderEnv2D + Resource>(
    render_env: Res<R>,
    mut objects: Query<(&mut RenderObject, &mut Transform), With<Mesh2d>>
) {
    for ((mut object, mut transform), new_object) in objects.iter_mut().zip(render_env.render_infos().into_iter()) {
        transform.translation = Vec3::new(new_object.pos.x, new_object.pos.y, 0.0);

        // ignore shape and color changes for now
        *object = new_object;
    }
}

pub fn start_render<R: RenderEnv2D + Resource>(r: R) {
    let mut bevy_app = App::new();
    bevy_app
        .add_plugins(DefaultPlugins)
        .insert_resource(r)
        .add_systems(
            Startup, setup::<R>
        )
        .add_systems(
            FixedUpdate,
            |mut render: ResMut<R>| {
                render.physics_tick();
            }
        )
        .add_systems(
            FixedUpdate,
            update_object_infos::<R>
        );
    bevy_app.run();
}