use std::f32::consts::PI;

use crate::utils::plotting;
use crate::utils::plotting::PlotConfig;
use crate::utils::render::{RenderEnv2D, RenderObject, Shape2D, start_render};
use crate::utils::vec4::Vec4;
use bevy::color::Color;
use bevy::math::Vec2;
use bevy::prelude::Resource;

struct RC3BSystem {
    mu1: f64,
    mu2: f64,
    angular_velocity: f64,
}

impl RC3BSystem {
    fn initial_ydot(&mut self, jacobi_constant: f64, initial_x: f64) -> f64 {
        let r1 = r1(initial_x, 0.0, self.mu2);
        let r2 = r2(initial_x, 0.0, self.mu1);
        (self.angular_velocity.powi(2) * initial_x.powi(2) + 2.0 * (self.mu1 / r1 + self.mu2 / r2)
            - jacobi_constant)
            .sqrt()
    }
}

fn r1(x: f64, y: f64, mu2: f64) -> f64 {
    ((x + mu2).powi(2) + y.powi(2)).sqrt()
}

fn r2(x: f64, y: f64, mu1: f64) -> f64 {
    ((x - mu1).powi(2) + y.powi(2)).sqrt()
}

// The derivative function (does not depend on t explicitly here)
// State vector is y from the initial value problem to avoid collisions with the y-coordinate of our solution :)
fn f(state_vector: Vec4, system: &RC3BSystem) -> Vec4 {
    let x = state_vector.x0;
    let y = state_vector.x1;
    let xdot = state_vector.x2;
    let ydot = state_vector.x3;

    let r1 = r1(x, y, system.mu2);
    let r2 = r2(x, y, system.mu1);

    Vec4::new(
        xdot,
        ydot,
        2.0 * system.angular_velocity * ydot + system.angular_velocity.powi(2) * x
            - (system.mu1 * ((x + system.mu2) / r1.powi(3))
                + system.mu2 * ((x - system.mu1) / r2.powi(3))),
        (system.angular_velocity.powi(2) - (system.mu1 / r1.powi(3)) - (system.mu2 / r2.powi(3)))
            * y
            - 2.0 * system.angular_velocity * xdot,
    )

    // returns
    // x0: x derivative
    // x1: y derivative
    // x2: x double derivative
    // x3: y double derivative
}

fn runge_kutta_step(current: Vec4, delta_t: f64, system: &RC3BSystem) -> Vec4 {
    // x0: x position
    // x1: y position
    // x2: x derivative
    // x3: y derivative

    let k1 = f(current, system);
    let k2 = f(current + k1 * (delta_t / 2.0), system);
    let k3 = f(current + k2 * (delta_t / 2.0), system);
    let k4 = f(current + k3 * delta_t, system);

    current + (delta_t / 6.0) * (k1 + 2.0 * k2 + 2.0 * k3 + k4)
}

pub fn ex2() {
    let mu2: f64 = 1.0e-3;
    let jacobi = 3.03;
    let delta_t = 1.0e-3;

    let stepcount = 5 * 1e6 as i32;

    let mut system = RC3BSystem {
        mu1: 1.0 - mu2,
        mu2,
        angular_velocity: 1.0,
    };

    let initial_x = [0.21, 0.24, 0.26, 0.27, 0.4, 0.5, 0.6, 0.8];
    let mut positions: Vec<(Vec<(f64, f64)>, String)> = vec![];
    let mut poincare_points = vec![];
    for x0 in initial_x.iter() {
        let ydot0 = system.initial_ydot(jacobi, *x0);

        let mut current = Vec4::new(*x0, 0.0, 0.0, ydot0);

        let mut line: Vec<(f64, f64)> = vec![];
        let mut inner_poincare = vec![];

        for step in 0..stepcount {
            let prev_y = current.x1;
            current = runge_kutta_step(current, delta_t, &system);

            // only print first 5 * 10^5 steps, after that the x0=0.5 path diverges and the others can't be seen anymore
            if step < 5e5 as i32 && prev_y < 0.0 && current.x1 >= 0.0 && current.x3 > 0.0 {
                inner_poincare.push((current.x0, current.x2))
            }

            line.push((current.x0, current.x1));
        }

        poincare_points.push((inner_poincare, format!("initial x: {x0}")));
        positions.push((line, format!("initial x: {x0}")))
    }

    plotting::line_graph(
        poincare_points,
        PlotConfig::default()
            .max_x(1.7)
            .scatter_plot(true)
            .x_label("dx/dt")
            .y_label("x")
            .title("Poincare sections"),
        "solutions/06/img/poincare.png",
    );

    for (max_steps, skips) in [(5e4 as usize, 1), (5e5 as usize, 10), (5e6 as usize, 500)] {
        let processed_pos = positions
            .iter()
            .map(|(line, label)| {
                (
                    line.iter()
                        .take(max_steps)
                        .step_by(skips)
                        .cloned()
                        .collect::<Vec<_>>(),
                    label.clone(),
                )
            })
            .collect::<Vec<_>>();

        plotting::line_graph(
            processed_pos,
            PlotConfig::default()
                .point_size(0)
                .x_label("x")
                .y_label("y")
                .title(format!("Particle paths for different initial values, {max_steps} steps, every {skips}th point").as_str()),
            format!("solutions/06/img/paths_{max_steps}_{skips}.png").as_str(),
        )
    }
}

pub fn render_path() {
    let mu2: f64 = 1.0e-3;
    // let jacobi = 3.03;
    let jacobi = 4.0;

    // ==========================================
    // Adjust this value for the initial value
    let x0 = 0.5;
    // ==========================================

    let mut system = RC3BSystem {
        mu1: 1.0 - mu2,
        mu2,
        angular_velocity: 1.0,
    };

    let y0dot = system.initial_ydot(jacobi, x0);

    let current = Vec4::new(x0, 0.0, 0.0, y0dot);

    let renderer = RK4Renderer { system, current, time: 0.0f32 };

    start_render(renderer);
}

#[derive(Resource)]
struct RK4Renderer {
    system: RC3BSystem,
    current: Vec4,
    time: f32,
}

impl RenderEnv2D for RK4Renderer {
    fn physics_tick(&mut self) {
        let dt = 1.0 / 64.0;
        self.current = runge_kutta_step(self.current, dt, &self.system);

        self.time += dt as f32;
    }

    fn render_infos(&self) -> Vec<RenderObject> {
        let render_scale = 300.0f32;
        let time_scale = 0.1f32;
        let rotation = Vec2::from_angle(self.time * 2f32 * PI * time_scale);

        let rotated = rotation.rotate(Vec2::new(
            self.current.x0 as f32 * render_scale,
            self.current.x1 as f32 * render_scale,
        ));

        vec![
            RenderObject {
                pos: rotation.rotate(Vec2 {
                    x: -0.5 * render_scale,
                    y: 0.0 * render_scale,
                }),
                shape: Shape2D::Circle(15.0),
                color: Color::srgb(0.2, 0.0, 1.0),
            },
            RenderObject {
                pos: rotation.rotate(Vec2 {
                    x: 0.5 * render_scale,
                    y: 0.0 * render_scale,
                }),
                shape: Shape2D::Circle(10.0),
                color: Color::srgb(0.2, 0.0, 1.0),
            },
            RenderObject {
                pos: rotated,
                shape: Shape2D::Circle(5.0),
                color: Color::srgb(0.8, 0.1, 0.3),
            },
        ]
    }
}
