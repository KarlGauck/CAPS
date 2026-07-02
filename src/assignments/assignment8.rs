use bevy::ecs::resource::Resource;

use crate::utils::plotting;
use crate::utils::plotting::PlotConfig;
use crate::utils::render_fluid_1d::{RenderEnv1D, start_render_1d};

const GRID_SIZE: usize = 400;
const INTERVAL: (f64, f64) = (-1.0, 1.0);

fn index_to_position(index: i32) -> f64 {
    let ratio = ((index % (GRID_SIZE as i32)) as f64) / GRID_SIZE as f64;
    INTERVAL.1 * ratio + INTERVAL.0 * (1.0 - ratio)
}

fn initial_velocities() -> Vec<f64> {
    vec![0; GRID_SIZE]
        .iter()
        .enumerate()
        .map(|(i, _)| {
            if f64::abs(index_to_position(i as i32)) < (1.0 / 3.0) {
                1.0
            } else {
                0.0
            }
        })
        .collect()
}

fn dx() -> f64 {
    (INTERVAL.1 - INTERVAL.0) / GRID_SIZE as f64
}

fn periodic_index(index: i32) -> usize {
    let res = (index + (GRID_SIZE as i32)) % (GRID_SIZE as i32);
    res as usize
}

fn central_differene(velocities: &[f64]) -> Vec<f64> {
    vec![0; velocities.len()]
        .iter()
        .enumerate()
        .map(|(i, _)| {
            (velocities[periodic_index(i as i32 + 1)] - velocities[periodic_index(i as i32 - 1)])
                / (2.0 * dx())
        })
        .collect()
}

fn solve_ftcs(alpha: f64, dt: f64, grid: Vec<f64>) -> Vec<f64> {
    let central_diff = central_differene(&grid);
    let adt = alpha * dt;
    grid.into_iter()
        .zip(central_diff.iter())
        .map(|(a, b)| a - adt * (*b))
        .collect()
}

fn solve_upwind(alpha: f64, dt: f64, grid: Vec<f64>) -> Vec<f64> {
    let sigma = alpha * dt / dx();

    let derivative: Vec<f64> = vec![0; GRID_SIZE]
        .iter()
        .enumerate()
        .map(|(i, _)| grid[periodic_index(i as i32)] - grid[periodic_index(i as i32 - 1)])
        .collect();

    grid.into_iter()
        .zip(derivative.iter())
        .map(|(a, b)| a - sigma * (*b))
        .collect()
}

fn solve_lax_wendroff(alpha: f64, dt: f64, grid: Vec<f64>) -> Vec<f64> {
    let sigma = alpha * dt / dx();

    let comp1: Vec<f64> = vec![0; grid.len()]
        .iter()
        .enumerate()
        .map(|(i, _)| {
            (grid[periodic_index(i as i32 + 1)] - grid[periodic_index(i as i32 - 1)]) / 2.0
        })
        .collect();

    let comp2: Vec<f64> = vec![0; grid.len()]
        .iter()
        .enumerate()
        .map(|(i, _)| {
            (grid[periodic_index(i as i32 + 1)] - 2.0 * grid[periodic_index(i as i32)]
                + grid[periodic_index(i as i32 - 1)])
                / 2.0
        })
        .collect();

    grid.into_iter()
        .zip(comp1)
        .zip(comp2)
        .map(|((a, b), c)| a - sigma * b + sigma.powi(2) * c)
        .collect()
}

pub fn ex1() {
    let sigma = 0.8;
    let alpha = 1.0;
    let dt = sigma * dx() / alpha;

    let velocities = initial_velocities();
    let mut grids = vec![velocities.clone(), velocities.clone(), velocities.clone()];

    let mut t = 0.0;
    let final_t = 4.0;
    while t < final_t {
        t += dt;

        grids = vec![
            solve_ftcs(alpha, dt, grids[0].clone()),
            solve_upwind(alpha, dt, grids[1].clone()),
            solve_lax_wendroff(alpha, dt, grids[2].clone()),
        ]
    }

    let interval = vec![0.0; GRID_SIZE]
        .iter()
        .enumerate()
        .map(|(i, _)| index_to_position(i as i32))
        .collect::<Vec<f64>>();

    let lines: Vec<(Vec<(f64, f64)>, String)> = vec![
        (
            interval
                .clone()
                .into_iter()
                .zip(grids[0].clone())
                .collect::<Vec<(f64, f64)>>(),
            "FTCS".to_string(),
        ),
        (
            interval
                .clone()
                .into_iter()
                .zip(grids[1].clone())
                .collect::<Vec<(f64, f64)>>(),
            "Upwind".to_string(),
        ),
        (
            interval
                .clone()
                .into_iter()
                .zip(grids[2].clone())
                .collect::<Vec<(f64, f64)>>(),
            "Lax Wendroff".to_string(),
        ),
    ];

    plotting::line_graph(
        lines,
        PlotConfig::default()
            .title("1D Linear Advection")
            .x_label("X value")
            .y_label("Velocity"),
        "solutions/08/img/velocities.png",
    );

    let lines_stable: Vec<(Vec<(f64, f64)>, String)> = vec![
        (
            interval
                .clone()
                .into_iter()
                .zip(grids[1].clone())
                .collect::<Vec<(f64, f64)>>(),
            "Upwind".to_string(),
        ),
        (
            interval
                .clone()
                .into_iter()
                .zip(grids[2].clone())
                .collect::<Vec<(f64, f64)>>(),
            "Lax Wendroff".to_string(),
        ),
    ];

    plotting::line_graph(
        lines_stable,
        PlotConfig::default()
            .title("1D Linear Advection")
            .x_label("X value")
            .y_label("Velocity"),
        "solutions/08/img/velocities_stable.png",
    )
}

pub enum Solver {
    FTCS,
    Upwind,
    LaxWendroff,
}

#[derive(Resource)]
struct FluidSim {
    grid: Vec<f64>,
    alpha: f64,
    dt: f64,
    solver: Solver,
}

impl FluidSim {
    fn new(solver: Solver) -> Self {
        let alpha = 1.0;
        let sigma = 0.8;
        let dt = sigma * dx() / alpha;
        Self {
            grid: initial_velocities(),
            alpha,
            dt,
            solver,
        }
    }
}

impl RenderEnv1D for FluidSim {
    fn velocities(&self) -> Vec<f32> {
        self.grid.iter().map(|&v| v as f32).collect()
    }
    fn tick(&mut self) {
        self.grid = match self.solver {
            Solver::FTCS => solve_ftcs(self.alpha, self.dt, self.grid.clone()),
            Solver::Upwind => solve_upwind(self.alpha, self.dt, self.grid.clone()),
            Solver::LaxWendroff => solve_lax_wendroff(self.alpha, self.dt, self.grid.clone()),
        }
    }
}

pub fn render(solver: Solver) {
    start_render_1d(FluidSim::new(solver));
}
