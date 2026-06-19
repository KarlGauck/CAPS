use bevy::color::Color;
use bevy::math::Vec2;
use bevy::prelude::Resource;
use num_traits::ToPrimitive;
use crate::utils::vec4;
use crate::utils::vec4::Vec4;
use crate::utils::plotting;
use crate::utils::plotting::PlotConfig;
use crate::utils::render::{start_render, RenderEnv2D, RenderObject, Shape2D};

struct RC3BSystem {
    mu1: f64,
    mu2: f64,
    angular_velocity: f64
}




impl RC3BSystem {
    fn initial_ydot(&mut self, jacobi_constant: f64, initial_x: f64) -> f64 {
        let r1 = r1(initial_x, 0.0, self.mu2);
        let r2 = r2(initial_x, 0.0, self.mu1);
        (self.angular_velocity.powi(2) * initial_x.powi(2) + 2.0*(self.mu1/r1 + self.mu2/r2) - jacobi_constant).sqrt()
    }
}

fn r1(x: f64, y: f64, mu2: f64) -> f64 {
    ((x+mu2).powi(2) + y.powi(2)).sqrt()
}

fn r2(x: f64, y: f64, mu1: f64) -> f64 {
    ((x-mu1).powi(2) + y.powi(2)).sqrt()
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
        2.0*system.angular_velocity*ydot + system.angular_velocity.powi(2)*x - (system.mu1*((x + system.mu2)/r1.powi(3)) + system.mu2*((x - system.mu1)/r2.powi(3))),
        (system.angular_velocity.powi(2) - (system.mu1 / r1.powi(3)) - (system.mu2 / r2.powi(3))) * y - 2.0*system.angular_velocity*xdot
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

    current + (delta_t / 6.0) * (k1 + 2.0*k2 + 2.0*k3 + k4)
}

pub fn ex2() {
    let mu2: f64 = 10.0f64.powi(-3);
    let jacobi = 3.03;
    let delta_t = 10.0f64.powi(-3);

    let stepcount= 5 * 10i32.pow(4);

    let mut system = RC3BSystem {
        mu1: 1.0-mu2,
        mu2,
        angular_velocity: 1.0
    };

    let initial_x = vec!(0.21, 0.24, 0.26, 0.27, 0.4, 0.5, 0.6, 0.8);
    let mut positions:  Vec<(Vec<(f64, f64)>, String)> = vec!();
    for (particle_index, x0) in initial_x.iter().enumerate() {
        let ydot0 = system.initial_ydot(jacobi, *x0);

        let mut current = Vec4::new(*x0, 0.0, 0.0, ydot0);

        let mut line: Vec<(f64, f64)> = vec!();

        for step in 0..stepcount {
            current = runge_kutta_step(current, delta_t, &system);

            if step % 1 == 0 {
                line.push((current.x0, current.x1));
            }
        }

        positions.push((line, format!("initial x: {x0}")))
    }

    plotting::line_graph(
        positions,
        PlotConfig::default().point_size(0),
        "solutions/06/img/paths.png"
    )

}

pub fn render_path() {
    let mu2: f64 = 10.0f64.powi(-3);
    let jacobi = 3.03;
    let x0 = 0.4;

    let mut system = RC3BSystem {
        mu1: 1.0-mu2,
        mu2,
        angular_velocity: 1.0
    };

    let y0dot = system.initial_ydot(jacobi, x0);

    let current = Vec4::new(x0, 0.0, 0.0, y0dot);

    let renderer = RK4Renderer {
        system,
        current
    };

    start_render(renderer);
}


#[derive(Resource)]
struct RK4Renderer {
    system: RC3BSystem,
    current: Vec4
}

impl RenderEnv2D for RK4Renderer {
    fn physics_tick(&mut self) {
        let dt = 1.0 / 60.0;
        self.current = runge_kutta_step(self.current, dt, &self.system);
    }

    fn render_infos(&self) -> Vec<RenderObject> {
        let render_scale = 300.0f32;

        vec![
            RenderObject {
                pos: Vec2 {x: -0.5*render_scale, y: 0.0*render_scale},
                shape: Shape2D::Circle(15.0),
                color: Color::srgb(0.2, 0.0, 1.0),
            },
            RenderObject {
                pos: Vec2 {x: 0.5*render_scale, y: 0.0*render_scale},
                shape: Shape2D::Circle(10.0),
                color: Color::srgb(0.2, 0.0, 1.0),
            },
            RenderObject {
                pos: Vec2 {x: self.current.x0.to_f32().unwrap() * render_scale, y: self.current.x1.to_f32().unwrap() * render_scale},
                shape: Shape2D::Circle(5.0),
                color: Color::srgb(0.2, 0.6, 1.0),
            },
        ]
    }
}