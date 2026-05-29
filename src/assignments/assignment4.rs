use crate::utils::plotting::PlotConfig;
use std::f64::consts::PI;

// ==========================================
// Functions
// ==========================================

fn si(t: f64) -> f64 {
    // division by 0 guard
    if t.abs() < 1e-20 { 1.0 } else { t.sin() / t }
}

fn c(t: f64) -> f64 {
    (PI / 2.0 * t * t).cos()
}

// ==========================================
// Integration methods
// ==========================================

/// Trapezoid rule with n subintervals.
fn trapez_n(f: &dyn Fn(f64) -> f64, a: f64, b: f64, n: usize) -> f64 {
    let h = (b - a) / n as f64;
    // TOOD: rewrite as fold
    let mut sum = (f(a) + f(b)) / 2.0;
    for i in 1..n {
        sum += f(a + i as f64 * h);
    }
    sum * h
}

/// Simpson's rule on a single interval [a, b].
fn simpson_single(f: &dyn Fn(f64) -> f64, a: f64, b: f64) -> f64 {
    let mid = (a + b) / 2.0;
    (b - a) / 6.0 * (f(a) + 4.0 * f(mid) + f(b))
}

/// Adaptive Simpson's method
/// Return (result, fun evaluations).
fn adaptive_simpson(f: &dyn Fn(f64) -> f64, a: f64, b: f64, eps: f64) -> (f64, usize) {
    let mut evals: usize = 3;
    let whole = simpson_single(f, a, b);

    fn recurse(
        f: &dyn Fn(f64) -> f64,
        a: f64,
        b: f64,
        eps: f64,
        acc: f64,
        depth: usize,
        evals: &mut usize,
    ) -> f64 {
        // implemented according to [[https://en.wikipedia.org/wiki/Adaptive_Simpson's_method]]
        let mid = (a + b) / 2.0;
        let left = simpson_single(f, a, mid);
        let right = simpson_single(f, mid, b);
        *evals += 4;
        let delta = left + right - acc;
        if depth == 0 || delta.abs() <= 15.0 * eps {
            return left + right + delta / 15.0;
        }
        recurse(f, a, mid, eps / 2.0, left, depth - 1, evals)
            + recurse(f, mid, b, eps / 2.0, right, depth - 1, evals)
    }

    let result = recurse(f, a, b, eps, whole, 50, &mut evals);
    (result, evals)
}

// "reference" via adaptive Simpson with small tolerance
fn reference_value(f: &dyn Fn(f64) -> f64, a: f64, b: f64) -> f64 {
    adaptive_simpson(f, a, b, 1e-13).0
}

/// Trapezoid: double N until |I_2N - I_N| < eps.
/// Return (N, |error|)
fn trapez_convergence(
    f: &dyn Fn(f64) -> f64,
    a: f64,
    b: f64,
    exact: f64,
    eps: f64,
) -> Vec<(f64, f64)> {
    let mut data = Vec::new();
    let mut n: usize = 1;
    let mut prev: Option<f64> = None;
    loop {
        let approx = trapez_n(f, a, b, n);
        let err = (approx - exact).abs();
        data.push((n as f64, err));
        if let Some(p) = prev {
            if (approx - p).abs() < eps {
                break;
            }
        }
        prev = Some(approx);
        n *= 2;
        if n > 1 << 26 {
            break;
        }
    }
    data
}

/// Adaptive Simpson convergence
/// Return (N, |error|)
fn adaptive_convergence(
    f: &dyn Fn(f64) -> f64,
    a: f64,
    b: f64,
    exact: f64,
    target_eps: f64,
) -> Vec<(f64, f64)> {
    let mut data = Vec::new();
    // We have to use a manual eps as proxy for the steps, as the implementation
    // of simpson exits not after a set amount of steps, but a set epsilon
    // TODO: make adaptive_simpson take a step parameter
    let mut eps = 1e-1_f64;
    while eps >= target_eps {
        let (approx, steps) = adaptive_simpson(f, a, b, eps);
        let err = (approx - exact).abs().max(1e-16);
        data.push((steps as f64, err));
        eps *= 0.25;
    }
    data
}

// ==========================================
// Plotting
// ==========================================

fn plot_convergence(
    trapezoid_data: &[(f64, f64)],
    simpson_data: &[(f64, f64)],
    eps: f64,
    title: &str,
    path: &str,
) {
    let all_x: Vec<f64> = trapezoid_data
        .iter()
        .chain(simpson_data.iter())
        .map(|(x, _)| *x)
        .collect();
    let x_min = all_x.iter().cloned().fold(f64::INFINITY, f64::min);
    let x_max = all_x.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let precision_line = vec![(x_min, eps), (x_max, eps)];

    crate::utils::plotting::line_graph(
        vec![
            (trapezoid_data.to_vec(), "Trapezoid".to_string()),
            (simpson_data.to_vec(), "Adaptive Simpson".to_string()),
            (precision_line, format!("eps = {:.0e}", eps)),
        ],
        PlotConfig::default()
            .title(title)
            .x_label("Number of steps")
            .y_label("Absolute error")
            .logarithmic_y(true)
            .point_size(3),
        path,
    );
}

// ── Public entry point ────────────────────────────────────────────────────────

pub fn ex2() {
    let eps = 1e-8_f64;

    // Si(1)
    let si_exact = reference_value(&si, 0.0, 1.0);
    let (si_val, si_steps) = adaptive_simpson(&si, 0.0, 1.0, eps);
    println!("Si(1) reference        = {:.12}", si_exact);
    println!(
        "Si(1) adaptive Simpson = {:.12}  (steps = {})",
        si_val, si_steps
    );

    let si_trap = trapez_convergence(&si, 0.0, 1.0, si_exact, eps);
    let si_adapt = adaptive_convergence(&si, 0.0, 1.0, si_exact, eps);
    println!(
        "Si(1) trapezoid converged at N = {}",
        si_trap.last().unwrap().0 as usize
    );

    plot_convergence(
        &si_trap,
        &si_adapt,
        eps,
        "Si(1) Convergence of Trapezoid vs Adaptive Simpson",
        "solutions/04/img/si1_convergence.png",
    );

    // C(5)
    let c5_exact = reference_value(&c, 0.0, 5.0);
    let (c5_val, c5_steps) = adaptive_simpson(&c, 0.0, 5.0, eps);
    println!("\nC(5) reference         = {:.12}", c5_exact);
    println!(
        "C(5) adaptive Simpson  = {:.12}  (steps = {})",
        c5_val, c5_steps
    );

    let c5_trap = trapez_convergence(&c, 0.0, 5.0, c5_exact, eps);
    let c5_adapt = adaptive_convergence(&c, 0.0, 5.0, c5_exact, eps);
    println!(
        "C(5) trapezoid converged at N = {}",
        c5_trap.last().unwrap().0 as usize
    );

    plot_convergence(
        &c5_trap,
        &c5_adapt,
        eps,
        "C(5) Convergence of Trapezoid vs Adaptive Simpson",
        "solutions/04/img/c5_convergence.png",
    );
}
