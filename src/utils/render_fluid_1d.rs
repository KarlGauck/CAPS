use bevy::{
    DefaultPlugins,
    app::{App, FixedUpdate, PluginGroup, Startup, Update},
    asset::{Assets, RenderAssetUsages},
    camera::Camera2d,
    color::Color,
    ecs::{
        resource::Resource,
        system::{Commands, Query, Res, ResMut},
    },
    math::{Vec2, Vec3},
    mesh::{Mesh, Mesh2d},
    render::render_resource::PrimitiveTopology,
    sprite_render::{ColorMaterial, MeshMaterial2d},
    transform::components::Transform,
    window::{Window, WindowPlugin},
};

// ---------------------------------------------------------------------------
// Public interface
// ---------------------------------------------------------------------------

/// The only thing the simulation needs to provide.
pub trait RenderEnv1D: Resource {
    /// Velocity u[i] for each grid cell, left→right.
    fn velocities(&self) -> Vec<f32>;
    fn tick(&mut self);
}

// ---------------------------------------------------------------------------
// Derived quantities
// ---------------------------------------------------------------------------

struct FluidFields {
    velocity:   Vec<f32>,
    pressure:   Vec<f32>,
    density:    Vec<f32>,
    divergence: Vec<f32>,
    flux:       Vec<f32>,
    n:          usize, // source of truth for the size
}

impl FluidFields {
    fn from_velocities(u: Vec<f32>) -> Self {
        let n = u.len();
        if n == 0 {
            return Self { velocity: vec![], pressure: vec![], density: vec![],
                          divergence: vec![], flux: vec![], n: 0 };
        }
        let dx = 1.0_f32;

        // Divergence: du/dx via central differences (forward/backward at edges)
        let divergence: Vec<f32> = (0..n).map(|i| {
            let u_r = if i + 1 < n { u[i + 1] } else { u[i] };
            let u_l = if i > 0   { u[i - 1] } else { u[i] };
            let denom = if i == 0 || i == n - 1 { dx } else { 2.0 * dx };
            (u_r - u_l) / denom
        }).collect();

        // Pressure: -ρ ∇·u  (incompressible, ρ=1).  We integrate from the left
        // so the mean is approximately zero.
        let mut pressure: Vec<f32> = divergence.iter().map(|d| -d).collect();
        let mean_p: f32 = pressure.iter().sum::<f32>() / n as f32;
        pressure.iter_mut().for_each(|p| *p -= mean_p);

        // Density: simple passive scalar advected by u (Euler step, dt=1, ρ₀=1).
        // ρ_new[i] = ρ[i] - dt * u[i] * dρ/dx   with ρ initialised to 1.
        // Since we don't carry state between frames we instead derive a
        // quasi-density from the running integral of -divergence, which gives
        // a coherent spatial pattern.
        let mut density = vec![1.0_f32; n];
        let mut acc = 0.0_f32;
        for i in 0..n {
            acc -= divergence[i] * dx;
            density[i] = 1.0 + acc * 0.1;
        }

        // Flux: J[i] = u[i] * ρ[i]
        let flux: Vec<f32> = u.iter().zip(density.iter()).map(|(ui, ri)| ui * ri).collect();

        Self { velocity: u, pressure, density, divergence, flux, n }
    }

    /// Normalise a field to [-1, 1] for display.
    fn normalise(v: &[f32]) -> Vec<f32> {
        let max = v.iter().cloned().fold(0.0_f32, |a, x| a.max(x.abs()));
        if max < 1e-9 { return vec![0.0; v.len()]; }
        v.iter().map(|x| x / max).collect()
    }
}

// ---------------------------------------------------------------------------
// Mesh helpers
// ---------------------------------------------------------------------------

/// Build a filled area chart mesh.
/// `values` ∈ [-1, 1] mapped to y ∈ [0, height] above `baseline_y`.
fn build_area_mesh(
    values:     &[f32],
    baseline_y: f32,
    height:     f32,
    total_w:    f32,
) -> Mesh {
    let n = values.len();
    if n < 2 {
        return empty_mesh();
    }
    let cell_w = total_w / n as f32;

    let mut verts:   Vec<[f32; 3]> = Vec::with_capacity(n * 2);
    let mut uvs:     Vec<[f32; 2]> = Vec::with_capacity(n * 2);
    let mut indices: Vec<u32>      = Vec::with_capacity((n - 1) * 6);

    for (i, &v) in values.iter().enumerate() {
        let x  = -total_w / 2.0 + (i as f32 + 0.5) * cell_w;
        let y  = baseline_y + v.max(0.0) * height;
        let t  = i as f32 / (n - 1) as f32;

        verts.push([x, baseline_y, 0.0]);
        verts.push([x, y,          0.0]);
        uvs.push([t, 0.0]);
        uvs.push([t, 1.0]);

        if i > 0 {
            let b = (i as u32) * 2;
            indices.extend([b-2, b-1, b, b-1, b+1, b]);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verts);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0,     uvs);
    mesh.insert_indices(bevy::mesh::Indices::U32(indices));
    mesh
}

/// Build a polyline mesh (thin strip).
fn build_line_mesh(
    values:     &[f32],
    baseline_y: f32,
    height:     f32,
    total_w:    f32,
    thickness:  f32,
) -> Mesh {
    let n = values.len();
    if n < 2 { return empty_mesh(); }
    let cell_w = total_w / n as f32;

    let pts: Vec<Vec2> = values.iter().enumerate().map(|(i, &v)| {
        let x = -total_w / 2.0 + (i as f32 + 0.5) * cell_w;
        let y = baseline_y + v * height;
        Vec2::new(x, y)
    }).collect();

    let mut verts:   Vec<[f32; 3]> = Vec::with_capacity(n * 2);
    let mut uvs:     Vec<[f32; 2]> = Vec::with_capacity(n * 2);
    let mut indices: Vec<u32>      = Vec::with_capacity((n - 1) * 6);

    for (i, &p) in pts.iter().enumerate() {
        let dir = if i + 1 < n { (pts[i+1] - p).normalize_or_zero() }
                  else          { (p - pts[i-1]).normalize_or_zero() };
        let perp = Vec2::new(-dir.y, dir.x) * thickness * 0.5;
        let t = i as f32 / (n - 1) as f32;

        verts.push((p + perp).extend(0.0).into());
        verts.push((p - perp).extend(0.0).into());
        uvs.push([t, 0.0]);
        uvs.push([t, 1.0]);

        if i > 0 {
            let b = (i as u32) * 2;
            indices.extend([b-2, b-1, b, b-1, b+1, b]);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verts);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0,     uvs);
    mesh.insert_indices(bevy::mesh::Indices::U32(indices));
    mesh
}

/// Dashed horizontal line at y.
fn build_dash_mesh(y: f32, total_w: f32, dash: f32, gap: f32) -> Mesh {
    let h = 1.5_f32;
    let mut verts:   Vec<[f32; 3]> = vec![];
    let mut uvs:     Vec<[f32; 2]> = vec![];
    let mut indices: Vec<u32>      = vec![];

    let mut x = -total_w / 2.0;
    while x < total_w / 2.0 {
        let x1 = (x + dash).min(total_w / 2.0);
        let b = verts.len() as u32;
        verts.extend([[x, y-h, 0.0], [x, y+h, 0.0], [x1, y+h, 0.0], [x1, y-h, 0.0]]);
        uvs.extend([[0.,0.],[0.,1.],[1.,1.],[1.,0.]]);
        indices.extend([b, b+1, b+2, b, b+2, b+3]);
        x += dash + gap;
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verts);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0,     uvs);
    mesh.insert_indices(bevy::mesh::Indices::U32(indices));
    mesh
}

fn empty_mesh() -> Mesh {
    let mut m = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32;3]>::new());
    m.insert_attribute(Mesh::ATTRIBUTE_UV_0,     Vec::<[f32;2]>::new());
    m.insert_indices(bevy::mesh::Indices::U32(vec![]));
    m
}

// ---------------------------------------------------------------------------
// ECS resources & components
// ---------------------------------------------------------------------------

#[derive(Resource)]
struct VisualizerConfig {
    total_w:    f32,   // horizontal span in world units
    total_h:    f32,   // maximum chart height in world units
    baseline_y: f32,   // y of the zero line
}

impl Default for VisualizerConfig {
    fn default() -> Self {
        Self { total_w: 1200.0, total_h: 180.0, baseline_y: -20.0 }
    }
}

/// Marker for each layer mesh so we can update them.
#[derive(bevy::ecs::component::Component)]
enum Layer {
    VelocityFill,
    VelocityLine,
    PressureLine,
    DensityLine,
    DivergenceLine,
    FluxLine,
    ZeroDash,
}

// ---------------------------------------------------------------------------
// Layer palette
// ---------------------------------------------------------------------------

const COL_VELOCITY_FILL: Color = Color::srgba(0.75, 0.90, 1.00, 0.18);
const COL_VELOCITY_LINE: Color = Color::srgba(0.90, 0.97, 1.00, 0.95);
const COL_PRESSURE:      Color = Color::srgba(1.00, 0.55, 0.20, 0.85);
const COL_DENSITY:       Color = Color::srgba(0.40, 0.90, 0.70, 0.80);
const COL_DIVERGENCE:    Color = Color::srgba(0.80, 0.40, 1.00, 0.75);
const COL_FLUX:          Color = Color::srgba(1.00, 0.85, 0.20, 0.70);
const COL_DASH:          Color = Color::srgba(1.00, 1.00, 1.00, 0.30);

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

fn setup<R: RenderEnv1D>(
    mut cmd:       Commands,
    mut meshes:    ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    cfg:           Res<VisualizerConfig>,
    env:           Res<R>,
) {
    cmd.spawn(Camera2d);

    let fields = FluidFields::from_velocities(env.velocities());

    macro_rules! area {
        ($layer:expr, $vals:expr, $col:expr) => {{
            let norm = FluidFields::normalise($vals);
            let m    = build_area_mesh(&norm, cfg.baseline_y, cfg.total_h, cfg.total_w);
            cmd.spawn((
                Mesh2d(meshes.add(m)),
                MeshMaterial2d(materials.add($col)),
                Transform::from_xyz(0.0, 0.0, layer_z(&$layer)),
                $layer,
            ));
        }};
    }
    macro_rules! line {
        ($layer:expr, $vals:expr, $col:expr) => {{
            let norm = FluidFields::normalise($vals);
            let m    = build_line_mesh(&norm, cfg.baseline_y, cfg.total_h, cfg.total_w, 2.5);
            cmd.spawn((
                Mesh2d(meshes.add(m)),
                MeshMaterial2d(materials.add($col)),
                Transform::from_xyz(0.0, 0.0, layer_z(&$layer)),
                $layer,
            ));
        }};
    }

    area!(Layer::VelocityFill, &fields.velocity,   COL_VELOCITY_FILL);
    line!(Layer::VelocityLine, &fields.velocity,   COL_VELOCITY_LINE);
    line!(Layer::PressureLine, &fields.pressure,   COL_PRESSURE);
    line!(Layer::DensityLine,  &fields.density,    COL_DENSITY);
    line!(Layer::DivergenceLine, &fields.divergence, COL_DIVERGENCE);
    line!(Layer::FluxLine,     &fields.flux,       COL_FLUX);

    // Zero-line dash
    let dash_m = build_dash_mesh(cfg.baseline_y, cfg.total_w, 8.0, 6.0);
    cmd.spawn((
        Mesh2d(meshes.add(dash_m)),
        MeshMaterial2d(materials.add(COL_DASH)),
        Transform::from_xyz(0.0, 0.0, 0.1),
        Layer::ZeroDash,
    ));
}

fn layer_z(l: &Layer) -> f32 {
    match l {
        Layer::VelocityFill  => 0.2,
        Layer::VelocityLine  => 0.5,
        Layer::PressureLine  => 0.6,
        Layer::DensityLine   => 0.7,
        Layer::DivergenceLine=> 0.8,
        Layer::FluxLine      => 0.9,
        Layer::ZeroDash      => 0.1,
    }
}

// ---------------------------------------------------------------------------
// Update
// ---------------------------------------------------------------------------

fn update_layers<R: RenderEnv1D>(
    env:        Res<R>,
    cfg:        Res<VisualizerConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    layers:     Query<(&Layer, &Mesh2d)>,
) {
    let fields = FluidFields::from_velocities(env.velocities());

    for (layer, mesh_handle) in layers.iter() {
        let new_mesh = match layer {
            Layer::VelocityFill => {
                let n = FluidFields::normalise(&fields.velocity);
                build_area_mesh(&n, cfg.baseline_y, cfg.total_h, cfg.total_w)
            }
            Layer::VelocityLine => {
                let n = FluidFields::normalise(&fields.velocity);
                build_line_mesh(&n, cfg.baseline_y, cfg.total_h, cfg.total_w, 2.5)
            }
            Layer::PressureLine => {
                let n = FluidFields::normalise(&fields.pressure);
                build_line_mesh(&n, cfg.baseline_y, cfg.total_h, cfg.total_w, 2.5)
            }
            Layer::DensityLine => {
                let n = FluidFields::normalise(&fields.density);
                build_line_mesh(&n, cfg.baseline_y, cfg.total_h, cfg.total_w, 2.5)
            }
            Layer::DivergenceLine => {
                let n = FluidFields::normalise(&fields.divergence);
                build_line_mesh(&n, cfg.baseline_y, cfg.total_h, cfg.total_w, 2.5)
            }
            Layer::FluxLine => {
                let n = FluidFields::normalise(&fields.flux);
                build_line_mesh(&n, cfg.baseline_y, cfg.total_h, cfg.total_w, 2.5)
            }
            Layer::ZeroDash => continue,
            _ => continue,
        };

        if let Some(mesh) = meshes.get_mut(mesh_handle) {
            *mesh = new_mesh;
        }
    }
}

fn tick_simulation<R: RenderEnv1D>(
    mut env: ResMut<R>
) {
    env.tick();
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn start_render_1d<R: RenderEnv1D>(env: R) {
    App::new()
        .add_plugins((DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "1D Fluid Visualizer".into(),
                resolution: (1400, 500).into(),
                ..Default::default()
            }),
            ..Default::default()
        })))
        .insert_resource(env)
        .insert_resource(VisualizerConfig::default())
        .add_systems(Startup, setup::<R>)
        .add_systems(Update,  update_layers::<R>)
        .add_systems(FixedUpdate, tick_simulation::<R>)
        .run();
}