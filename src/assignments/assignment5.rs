use crate::utils::plotting::PlotConfig;

fn step_2(
    f_a: &dyn Fn(f64, f64, f64) -> f64, // f_a(t, y_a, y_b)
    f_b: &dyn Fn(f64, f64, f64) -> f64, // f_b(t, y_a, y_b)
    y_a: f64,
    y_b: f64,
    step_size: f64,
    t: f64,
) -> (f64, f64, f64) {
    let t_next = t + step_size;

    let f_a_euler = f_a(t, y_a, y_b);
    let f_b_euler = f_b(t, y_a, y_b);

    let y_a_next = y_a + step_size * f_a_euler;
    let y_b_next = y_b + step_size * f_b_euler;

    (
        y_a + 0.5 * step_size * (f_a_euler + f_a(t_next, y_a_next, y_b_next)),
        y_b + 0.5 * step_size * (f_b_euler + f_b(t_next, y_a_next, y_b_next)),
        t_next,
    )
}

fn analytic_w(n: f64, xi: f64) -> f64 {
    if n == 0.0 {
        1.0 - xi * xi / 6.0
    } else if n == 1.0 {
        if xi.abs() < 1e-12 { 1.0 } else { xi.sin() / xi }
    } else if n == 5.0 {
        1.0 / (1.0 + xi * xi / 3.0).sqrt()
    } else {
        f64::NAN
    }
}

fn solve_lane_emden(n: f64, step_size: f64, steps: usize) -> Vec<(f64, f64)> {
    let mut xi = step_size;
    let mut w = analytic_w(0.0, xi);
    let mut z = -xi / 3.0;

    let mut res = vec![(xi, w)];

    for _ in 0..steps {
        (w, z, xi) = step_2(
            &|_, _, z| z,
            &|xi: f64, w: f64, z: f64| -((2.0 / xi * z) + w.powf(n)),
            w,
            z,
            step_size,
            xi,
        );

        res.push((xi, w));
    }

    res
}

fn plot_lane_emden(n: f64, step_sizes: &[f64], xi_max: f64, title: &str, path: &str) {
    let mut series: Vec<(Vec<(f64, f64)>, String)> = step_sizes
        .iter()
        .map(|&h| {
            let steps = (xi_max / h).round() as usize;
            (solve_lane_emden(n, h, steps), format!("Heun, d xi = {h}"))
        })
        .collect();

    let analytic: Vec<(f64, f64)> = (0..=500)
        .map(|i| {
            let xi = xi_max * i as f64 / 500.0;
            (xi, analytic_w(n, xi))
        })
        .collect();
    series.push((analytic, "analytical".to_string()));

    let mut error_series: Vec<(Vec<(f64, f64)>, String)> = step_sizes
        .iter()
        .map(|&h| {
            let steps = (xi_max / h).round() as usize;
            let errors: Vec<(f64, f64)> = solve_lane_emden(n, h, steps)
                .into_iter()
                .map(|(xi, w)| (xi, (w - analytic_w(n, xi)).abs().max(1e-16)))
                .collect();
            (errors, format!("Heun error, d xi = {h}"))
        })
        .collect();

    series.append(&mut error_series);

    crate::utils::plotting::line_graph(
        series,
        PlotConfig::default()
            .title(title)
            .x_label("xi")
            .y_label("w(xi)")
            .point_size(1),
        path,
    );
}

pub fn ex21() {
    let step_sizes = [0.5, 0.1, 0.05, 0.01];

    plot_lane_emden(
        0.0,
        &step_sizes,
        2.5,
        "Lane-Emden n = 0",
        "solutions/05/img/lane_emden_n0.png",
    );
    plot_lane_emden(
        1.0,
        &step_sizes,
        3.2,
        "Lane-Emden n = 1",
        "solutions/05/img/lane_emden_n1.png",
    );
    plot_lane_emden(
        5.0,
        &step_sizes,
        10.0,
        "Lane-Emden n = 5",
        "solutions/05/img/lane_emden_n5.png",
    );
}

// ==========================================
// ==========================================

fn solve_lane_emden_to_surface(n: f64, step_size: f64, xi_max: f64) -> (Vec<(f64, f64)>, f64, f64) {
    let mut xi = step_size;
    let mut w = analytic_w(0.0, xi);
    let mut z = -xi / 3.0;

    let mut res = vec![(xi, w)];

    loop {
        let (w_next, z_next, xi_next) = step_2(
            &|_, _, z| z,
            &|xi: f64, w: f64, z: f64| -((2.0 / xi * z) + w.max(0.0).powf(n)),
            w,
            z,
            step_size,
            xi,
        );

        if w_next <= 0.0 || xi_next > xi_max {
            let t = w / (w - w_next);
            let xi_1 = xi + t * (xi_next - xi);
            let z_1 = z + t * (z_next - z);
            res.push((xi_1, 0.0));
            return (res, xi_1, z_1);
        }

        xi = xi_next;
        w = w_next;
        z = z_next;
        res.push((xi, w));
    }
}

fn lane_emden_mass(xi_1: f64, z_1: f64) -> f64 {
    -xi_1 * xi_1 * z_1
}

pub fn ex2_2a() {
    let step_size = 1e-4;
    let xi_max = 20.0;

    let gammas = [(3.0, 0.5), (5.0 / 3.0, 1.5), (7.0 / 5.0, 2.5)];

    let density_series: Vec<(Vec<(f64, f64)>, String)> = gammas
        .iter()
        .map(|&(gamma, n)| {
            let (profile, xi_1, z_1) = solve_lane_emden_to_surface(n, step_size, xi_max);
            let mass = lane_emden_mass(xi_1, z_1);

            println!("gamma = {gamma:.3} (n = {n}): xi_1 = {xi_1:.5}, M* = {mass:.5}");

            let density: Vec<(f64, f64)> = profile
                .into_iter()
                .map(|(xi, w)| (xi, w.max(0.0).powf(n)))
                .collect();

            (density, format!("gamma = {gamma:.3} (n = {n})"))
        })
        .collect();

    crate::utils::plotting::line_graph(
        density_series,
        PlotConfig::default()
            .title("Lane-Emden density profiles")
            .x_label("xi")
            .y_label("rho / rho_c = w^n")
            .point_size(1),
        "solutions/05/img/lane_emden_density_profiles.png",
    );
}

pub fn ex2_2b() {
    let n = 3.0;
    let step_size = 1e-4;
    let xi_max = 20.0;

    let (_, xi_1, z_1) = solve_lane_emden_to_surface(n, step_size, xi_max);
    let mass_star = lane_emden_mass(xi_1, z_1);

    #[allow(non_snake_case)]
    let G = 6.674e-8; // Gravitational constant
    let k_b = 1.380649e-16; // Boltzmann constant
    let m_h = 1.6726e-24; // hydrogen mass

    let r_sun = 6.96e10;
    let rho_bar_sun = 1.41;
    let mu = 0.62;

    // (12)
    let rho_c = rho_bar_sun * xi_1.powi(3) / (3.0 * mass_star);

    // (8)
    let alpha = r_sun / xi_1;

    // (8) for K
    #[allow(non_snake_case)]
    let K = alpha * alpha * 4.0 * std::f64::consts::PI * G * rho_c.powf((n - 1.0) / n) / (n + 1.0);

    // pressure in center
    let p_c = K * rho_c.powf((n + 1.0) / n);

    // ideal gas
    #[allow(non_snake_case)]
    let T_c = p_c * mu * m_h / (rho_c * k_b);

    println!("T_c = {T_c:.4e} K (literature: ~1.57e7 K)");
}
