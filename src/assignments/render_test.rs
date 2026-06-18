use bevy::ecs::resource::Resource;
use bevy::math::Vec2;
use bevy::color::Color;

use crate::utils::render::*;

#[derive(Resource)]
pub struct BounceScene {
    circle_pos: Vec2,
    hollow_pos: Vec2,
    circle_vel: f32,
    hollow_vel: f32,
    time: f32,
}

impl BounceScene {
    pub fn new() -> Self {
        Self {
            circle_pos: Vec2::new(0.0, 100.0),
            hollow_pos: Vec2::new(0.0, -100.0),
            circle_vel: 200.0,
            hollow_vel: -300.0,
            time: 0.0,
        }
    }
}

impl RenderEnv2D for BounceScene {
    fn physics_tick(&mut self) {
        let dt = 1.0 / 60.0;
        self.time += dt;

        self.circle_pos.x += self.circle_vel * dt;
        self.hollow_pos.x += self.hollow_vel * dt;

        if self.circle_pos.x.abs() > 500.0 {
            self.circle_vel = -self.circle_vel;
        }
        if self.hollow_pos.x.abs() > 500.0 {
            self.hollow_vel = -self.hollow_vel;
        }
    }

    fn render_infos(&self) -> Vec<RenderObject> {
        vec![
            RenderObject {
                pos: self.circle_pos,
                shape: Shape2D::Circle(50.0),
                color: Color::srgb(0.2, 0.6, 1.0),
            },
            RenderObject {
                pos: self.hollow_pos,
                shape: Shape2D::HollowCircle(50.0, 10.0),
                color: Color::srgb(1.0, 0.4, 0.2),
            },
        ]
    }
}

pub fn run() {
    start_render(BounceScene::new());
}