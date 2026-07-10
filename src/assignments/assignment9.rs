use crate::utils::plotting::*;

// ffmpeg -framerate 30 -i solutions/09/<EX>/rho_%05d.png -pix_fmt yuv420p movie.mp4

pub fn ex_a() {
    let n = 500usize;
    let l = 100.0;
    let dx = l / n as f64;
    let cs = 1.0f64;
    let courant = 0.5;
    let t_end = 100.0;

    let mut q1 = vec![0.0; n];
    let mut q2 = vec![0.0; n];

    // initial density profile
    for (i, q1_it) in q1.iter_mut().enumerate() {
        let x = (i as f64 + 0.5) * dx;
        *q1_it = 1.0 + 0.3 * (-(x - 50.0).powi(2) / 10.0).exp();
    }

    std::fs::create_dir_all("out").unwrap();
    let mut t = 0.0;
    let mut step = 0;

    while t < t_end {
        let rho_line: Vec<(f64, f64)> = (0..n).map(|i| ((i as f64 + 0.5) * dx, q1[i])).collect();
        line_graph(
            vec![(rho_line, "rho".to_string())],
            PlotConfig::default()
                .title(&format!("t = {:.2}", t))
                .x_label("x")
                .y_label("rho(x,t)")
                .max_x(l),
            &format!("solutions/09/a/rho_{:05}.png", step),
        );

        let vmax = (0..n).map(|i| (q2[i] / q1[i]).abs()).fold(0.0, f64::max) + cs;

        // timestep with courant condition
        let dt = courant * dx / vmax;

        // ghost cells with reflective boundary conditions
        // density stays the same, u gets inverted
        let g = |v: &Vec<f64>, i: isize, sign: f64| -> f64 {
            if i < 0 {
                sign * v[0]
            } else if i >= n as isize {
                sign * v[n - 1]
            } else {
                v[i as usize]
            }
        };

        // half steps
        let mut q1_h = q1.clone();
        let mut q2_h = q2.clone();
        for i in 0..n {
            let ul = q2[i] / q1[i];
            let q1_l = g(&q1, i as isize - 1, 1.0);
            let q2_l = g(&q2, i as isize - 1, -1.0);
            let q1_r = g(&q1, i as isize + 1, 1.0);
            let q2_r = g(&q2, i as isize + 1, -1.0);

            let u_l = q2_l / q1_l;
            let u_r = q2_r / q1_r;

            let flux = |ql: f64, qr: f64, ul: f64, ur: f64| -> f64 {
                let uf = 0.5 * (ul + ur);
                if uf >= 0.0 { ql * uf } else { qr * uf }
            };

            let flow_1_l = flux(q1_l, q1[i], u_l, ul);
            let flow_1_r = flux(q1[i], q1_r, ul, u_r);

            let flow_2_l = flux(q2_l, q2[i], u_l, ul);
            let flow_2_r = flux(q2[i], q2_r, ul, u_r);

            q1_h[i] = q1[i] - dt / dx * (flow_1_r - flow_1_l);
            q2_h[i] = q2[i] - dt / dx * (flow_2_r - flow_2_l);
        }
        q1 = q1_h;
        q2 = q2_h;

        // full n+1 step
        let p: Vec<f64> = q1.iter().map(|r| cs * cs * r).collect();
        for i in 0..n {
            let p_l = if i == 0 { p[0] } else { p[i - 1] };
            let p_r = if i == n - 1 { p[n - 1] } else { p[i + 1] };
            q2[i] -= dt / (2.0 * dx) * (p_r - p_l);
        }

        t += dt;
        step += 1;
    }
}

// ==========================================
// b)
// ==========================================

fn advect(
    q: &[f64],
    u_g: &dyn Fn(isize) -> f64,
    sign: f64,
    dt: f64,
    dx: f64,
    n: usize,
) -> Vec<f64> {
    let g = |i: isize| -> f64 {
        if i < 0 {
            sign * q[0]
        } else if i >= n as isize {
            sign * q[n - 1]
        } else {
            q[i as usize]
        }
    };
    q.iter().enumerate().map(|(i, _)| {
        let i = i as isize;
        let ql = g(i - 1);
        let qc = g(i);
        let qr = g(i + 1);
        let flux = |a: f64, b: f64, ua: f64, ub: f64| -> f64 {
            let uf = 0.5 * (ua + ub);
            if uf >= 0.0 { a * uf } else { b * uf }
        };
        let fl = flux(ql, qc, u_g(i - 1), u_g(i));
        let fr = flux(qc, qr, u_g(i), u_g(i + 1));
        qc - dt / dx * (fr - fl)
    }).collect()
}

pub fn ex_b() {
    let n = 500usize;
    let l = 100.0;
    let dx = l / n as f64;
    let courant = 0.5;
    let t_end = 40.0;
    let gamma = 1.4;

    let mut q1 = vec![0.0; n];
    let mut q2 = vec![0.0; n];
    let mut q3 = vec![0.0; n];
    for i in 0..n {
        let x = (i as f64 + 0.5) * dx;
        q1[i] = if x <= 50.0 { 2.0 } else { 1.0 };
        q3[i] = q1[i] * 1.0;
    }

    let mut t = 0.0;
    let mut step = 0;

    while t < t_end {
        let u: Vec<f64> = (0..n).map(|i| q2[i] / q1[i]).collect();
        let eps_tot: Vec<f64> = (0..n).map(|i| q3[i] / q1[i]).collect();
        let eps: Vec<f64> = (0..n).map(|i| eps_tot[i] - 0.5 * u[i] * u[i]).collect();
        let cs: Vec<f64> = eps
            .iter()
            .map(|&e| (gamma * (gamma - 1.0) * e).sqrt())
            .collect();

        let xs: Vec<f64> = (0..n).map(|i| (i as f64 + 0.5) * dx).collect();
        let plot = |dir: &str, vals: &Vec<f64>, name: &str, ylabel: &str| {
            let pts: Vec<(f64, f64)> = xs.iter().zip(vals).map(|(&x, &y)| (x, y)).collect();
            line_graph(
                vec![(pts, name.to_string())],
                PlotConfig::default()
                    .title(&format!("t = {:.2}", t))
                    .x_label("x")
                    .y_label(ylabel)
                    .max_x(l),
                &format!("{}/{}_{:05}.png", dir, name, step),
            );
        };
        plot("solutions/09/b/out_rho", &q1, "q1", "q1(x,t)");
        plot("solutions/09/b/out_u", &u, "u", "u(x,t)");
        plot("solutions/09/b/out_eps", &eps, "eps", "eps(x,t)");

        // Courant timestep from current max |u|+cs
        let vmax = (0..n).map(|i| u[i].abs() + cs[i]).fold(0.0, f64::max);
        let dt = courant * dx / vmax;

        let u_g = |i: isize| -> f64 {
            if i < 0 {
                u[0]
            } else if i >= n as isize {
                -u[n - 1]
            } else {
                u[i as usize]
            }
        };
        q1 = advect(&q1, &u_g, 1.0, dt, dx, n);
        q2 = advect(&q2, &u_g, -1.0, dt, dx, n);
        q3 = advect(&q3, &u_g, 1.0, dt, dx, n);

        let u2: Vec<f64> = (0..n).map(|i| q2[i] / q1[i]).collect();
        let eps_tot2: Vec<f64> = (0..n).map(|i| q3[i] / q1[i]).collect();
        let eps2: Vec<f64> = (0..n).map(|i| eps_tot2[i] - 0.5 * u2[i] * u2[i]).collect();
        let p: Vec<f64> = (0..n).map(|i| (gamma - 1.0) * q1[i] * eps2[i]).collect();
        let pu: Vec<f64> = (0..n).map(|i| p[i] * u2[i]).collect();

        let bp = |v: &Vec<f64>, i: isize| -> f64 {
            if i < 0 {
                v[0]
            } else if i >= n as isize {
                v[n - 1]
            } else {
                v[i as usize]
            }
        };

        let mut q2_h = q2.clone();
        let mut q3_h = q3.clone();
        for i in 0..n as isize {
            q2_h[i as usize] -= dt / (2.0 * dx) * (bp(&p, i + 1) - bp(&p, i - 1));
            q3_h[i as usize] -= dt / (2.0 * dx) * (bp(&pu, i + 1) - bp(&pu, i - 1));
        }
        q2 = q2_h;
        q3 = q3_h;

        t += dt;
        step += 1;
    }
}

// ==========================================
// c)
// ==========================================

pub fn ex_c() {
    let n = 500usize;
    let l = 100.0;
    let dx = l / n as f64;
    let courant = 0.5;
    let t_end = 40.0;
    let gamma = 1.4;

    let mut q1 = vec![0.0; n];
    let mut q2 = vec![0.0; n];
    let mut q3 = vec![0.0; n];
    for i in 0..n {
        let x = (i as f64 + 0.5) * dx;
        q1[i] = if x <= 50.0 { 2.0 } else { 1.0 };
        q3[i] = q1[i] * 1.0;
    }

    let xi = 3.0;

    let mut t = 0.0;
    let mut step = 0;

    while t < t_end {
        let u: Vec<f64> = (0..n).map(|i| q2[i] / q1[i]).collect();
        let eps_tot: Vec<f64> = (0..n).map(|i| q3[i] / q1[i]).collect();
        let eps: Vec<f64> = (0..n).map(|i| eps_tot[i] - 0.5 * u[i] * u[i]).collect();
        let cs: Vec<f64> = eps
            .iter()
            .map(|&e| (gamma * (gamma - 1.0) * e).sqrt())
            .collect();

        let xs: Vec<f64> = (0..n).map(|i| (i as f64 + 0.5) * dx).collect();
        let plot = |dir: &str, vals: &Vec<f64>, name: &str, ylabel: &str| {
            let pts: Vec<(f64, f64)> = xs.iter().zip(vals).map(|(&x, &y)| (x, y)).collect();
            line_graph(
                vec![(pts, name.to_string())],
                PlotConfig::default()
                    .title(&format!("t = {:.2}", t))
                    .x_label("x")
                    .y_label(ylabel)
                    .max_x(l),
                &format!("{}/{}_{:05}.png", dir, name, step),
            );
        };
        plot("solutions/09/c/out_rho", &q1, "q1", "q1(x,t)");
        plot("solutions/09/c/out_u", &u, "u", "u(x,t)");
        plot("solutions/09/c/out_eps", &eps, "eps", "eps(x,t)");

        let smax = (0..n).map(|i| u[i].abs() + cs[i]).fold(0.0, f64::max);
        let dt = courant * dx / smax;

        let u_g = |i: isize| -> f64 {
            if i < 0 {
                u[0]
            } else if i >= n as isize {
                -u[n - 1]
            } else {
                u[i as usize]
            }
        };
        q1 = advect(&q1, &u_g, 1.0, dt, dx, n);
        q2 = advect(&q2, &u_g, -1.0, dt, dx, n);
        q3 = advect(&q3, &u_g, 1.0, dt, dx, n);

        let u2: Vec<f64> = (0..n).map(|i| q2[i] / q1[i]).collect();
        let eps_tot2: Vec<f64> = (0..n).map(|i| q3[i] / q1[i]).collect();
        let eps2: Vec<f64> = (0..n).map(|i| eps_tot2[i] - 0.5 * u2[i] * u2[i]).collect();
        let p: Vec<f64> = (0..n).map(|i| (gamma - 1.0) * q1[i] * eps2[i]).collect();

        // Neumann Richtmyer artificial viscosity
        let u_r = |i: isize| -> f64 {
            if i < 0 {
                u2[0]
            } else if i >= n as isize {
                -u2[n - 1]
            } else {
                u2[i as usize]
            }
        };
        let q: Vec<f64> = (0..n as isize)
            .map(|i| {
                let du = u_r(i + 1) - u_r(i);
                if du < 0.0 {
                    xi * xi * q1[i as usize] * du * du
                } else {
                    0.0
                }
            })
            .collect();
        let p_eff: Vec<f64> = (0..n).map(|i| p[i] + q[i]).collect();
        let pu: Vec<f64> = (0..n).map(|i| p_eff[i] * u2[i]).collect();

        let bp = |v: &Vec<f64>, i: isize| -> f64 {
            if i < 0 {
                v[0]
            } else if i >= n as isize {
                v[n - 1]
            } else {
                v[i as usize]
            }
        };

        let mut q2_h = q2.clone();
        let mut q3_h = q3.clone();
        for i in 0..n as isize {
            q2_h[i as usize] -= dt / (2.0 * dx) * (bp(&p, i + 1) - bp(&p, i - 1));
            q3_h[i as usize] -= dt / (2.0 * dx) * (bp(&pu, i + 1) - bp(&pu, i - 1));
        }
        q2 = q2_h;
        q3 = q3_h;

        t += dt;
        step += 1;
    }
}
