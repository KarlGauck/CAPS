use std::collections::VecDeque;
 
use bevy::asset::{Handle, RenderAssetUsages};
use bevy::color::Alpha;
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::math::Vec2;
use bevy::{
    DefaultPlugins,
    app::{App, FixedUpdate, Startup, Update},
    asset::Assets,
    camera::Camera2d,
    color::Color,
    ecs::{
        component::Component,
        query::With,
        resource::Resource,
        system::{Commands, Query, Res, ResMut},
    },
    math::{
        Vec3,
        primitives::{Circle, ToRing},
    },
    mesh::{Mesh, Mesh2d},
    sprite_render::{ColorMaterial, MeshMaterial2d},
    transform::components::Transform,
};

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

#[derive(Component)]
pub struct Trail {
    pub positions: VecDeque<Vec2>,
    pub max_length: usize,
    pub width: f32,
}

fn build_trail_mesh(positions: &[Vec2], width: f32) -> Mesh {
    let mut verts: Vec<[f32; 3]> = vec![];
    let mut uvs: Vec<[f32; 2]> = vec![];
    let mut indices: Vec<u32> = vec![];

    for (i, window) in positions.windows(2).enumerate() {
        let dir = (window[1] - window[0]).normalize_or_zero();
        let perp = Vec2::new(-dir.y, dir.x);
        let t = i as f32 / positions.len() as f32;
        let w = width * (1.0 - t); // taper

        let i0 = (i * 2) as u32;
        verts.push((window[0] + perp * w).extend(0.0).into());
        verts.push((window[0] - perp * w).extend(0.0).into());
        uvs.push([0.0, t]); uvs.push([1.0, t]);

        if i > 0 {
            indices.extend([i0-2, i0-1, i0, i0-1, i0+1, i0]);
        }
    }

    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verts);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::mesh::Indices::U32(indices));
    mesh
}

fn update_trails(
    mut meshes: ResMut<Assets<Mesh>>,
    objects: Query<&Transform, With<RenderObject>>,
    mut trails: Query<(&mut Trail, &Mesh2d)>,
) {
    for (transform, (mut trail, mesh_handle)) in objects.iter().zip(trails.iter_mut()) {
        trail.positions.push_front(transform.translation.truncate());
        if trail.positions.len() > trail.max_length {
            trail.positions.pop_back();
        }
        let parent_pos = transform.translation.truncate();
        let local_positions: Vec<Vec2> = trail.positions.iter()
            .map(|p| *p - parent_pos)
            .collect();
        if let Some(mesh) = meshes.get_mut(mesh_handle) {
            *mesh = build_trail_mesh(&local_positions, trail.width);
        }
    }
}

fn setup<R: RenderEnv2D + Resource>(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    render_env: Res<R>,
) {
    cmd.spawn(Camera2d);

    // generate all meshes beforehand
    let objects: Vec<(Handle<Mesh>, Handle<Mesh>, RenderObject)> = render_env
        .render_infos()
        .into_iter()
        .map(|object| {
            let mesh = match object.shape {
                Shape2D::Circle(radius) => meshes.add(Circle::new(radius)),
                Shape2D::HollowCircle(radius, thickness) => {
                    meshes.add(Circle::new(radius).to_ring(thickness))
                }
            };
            let trail_mesh = meshes.add(build_trail_mesh(&[], 5.0));
            (mesh, trail_mesh, object)
        })
        .collect();

        for (mesh, trail_mesh, object) in objects {
            let color = object.color.clone();
            cmd.spawn((
                Mesh2d(mesh.clone()),
                MeshMaterial2d(materials.add(object.color)),
                Transform::from_xyz(object.pos.x, object.pos.y, 0.0),
                object,
            )).with_children(|parent| {
                parent.spawn((
                    Mesh2d(trail_mesh),
                    // MeshMaterial2d(materials.add(Color::srgba(1.0, 1.0, 1.0, 0.5))),
                    MeshMaterial2d(materials.add(color.with_alpha(0.5f32))),
                    Transform::default(),
                    Trail { positions: VecDeque::new(), max_length: 30, width: 5.0 },
                ));
            });
        }
}

fn update_object_infos<R: RenderEnv2D + Resource>(
    render_env: Res<R>,
    mut objects: Query<(&mut RenderObject, &mut Transform), With<Mesh2d>>,
) {
    for ((mut object, mut transform), new_object) in objects
        .iter_mut()
        .zip(render_env.render_infos().into_iter())
    {
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
        .add_systems(Startup, setup::<R>)
        .add_systems(
            FixedUpdate,
            |mut render: ResMut<R>, keyboard_input: Res<ButtonInput<KeyCode>>| {
                let count = if keyboard_input.pressed(KeyCode::Space) {
                    100
                } else {
                    1
                };
                for _ in 0..count {
                    render.physics_tick();
                }
            },
        )
        .add_systems(Update, (update_object_infos::<R>, update_trails));
    bevy_app.run();
}
