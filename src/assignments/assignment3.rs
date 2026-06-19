use crate::utils;
use crate::utils::plotting::PlotConfig;
use core::f64;
use hyperprec::f512;
use num_traits::Float;
use std::fmt::Debug;

// 1.a
fn classic_solution<T: Float + Debug>(a: T, b: T, c: T) -> Option<(T, T)> {
    let sqrt_arg: T = b.powi(2) - T::from(4.0).unwrap() * a * c;
    if sqrt_arg < T::zero() {
        None
    } else {
        Some((
            (-b + sqrt_arg.sqrt()) / (T::from(2.0).unwrap() * a),
            (-b - sqrt_arg.sqrt()) / (T::from(2.0).unwrap() * a),
        ))
    }
}

// Vieta
// fn stable_solution<T: Float + Debug>(a: T, b: T, c: T) -> Option<T> {
//     let sqrt_arg = b.powi(2) - T::from(4.0).unwrap() * a * c;
//     if sqrt_arg < T::zero() {
//         None
//     } else {
//         Some(T::from(-2.0).unwrap() * c / (b + sqrt_arg.sqrt()))
//     }
// }
fn other_stable_solution(a: f512, b: f512, c: f512) -> Option<f512> {
    let sqrt_arg = b.pow(f512::from_f64(2.0)) - f512::from(4.0) * a * c;
    if sqrt_arg < f512::ZERO {
        None
    } else {
        Some(f512::from(-2.0) * c / (b + sqrt_arg.sqrt()))
    }
}

// 1.d)

// Return the numerator of the first root of a 2nd degree polynomial
fn midnight_numerator<T: Float>(a: T, b: T, c: T) -> Option<T> {
    let sqrt_arg = b * b - T::from(4.0).unwrap() * a * c;
    if sqrt_arg < T::zero() {
        None
    } else {
        Some(-b + sqrt_arg.sqrt())
    }
}
#[allow(unused)]
fn final_stable_solution<T: Float + Debug>(a: T, b: T, c: T) -> Option<(T, T)> {
    if b > T::zero() {
        // num is 2nd root
        midnight_numerator(a, -b, c).map(|num| (-T::from(2.0).unwrap() * c / num, num))
    } else {
        // num is 1st root
        midnight_numerator(a, b, c).map(|num| (num, T::from(2.0).unwrap() * c / num))
    }
}

pub fn ex1() {
    let max_n = 25;
    let min_n = 1;

    let a = 1.0;
    let b = 1.0;

    let float_prec =
        (min_n..=max_n).map(|n| classic_solution::<f32>(a, b, 10.0.powi(-n)).unwrap().0);
    let double_prec = (min_n..=max_n).map(|n| {
        classic_solution::<f64>(a.into(), b.into(), 10.0.powi(-n))
            .unwrap()
            .0
    });

    let double_2nd = (min_n..=max_n).map(|n| {
        classic_solution::<f64>(a.into(), b.into(), 10.0.powi(-n))
            .unwrap()
            .1
    });

    let double_prec = double_prec.collect::<Vec<f64>>();
    println!("double value 1st {:?}", &double_prec);
    println!("double value 2nd {:?}", double_2nd.collect::<Vec<f64>>());

    let double_prec = double_prec.into_iter();

    // let stable = (min_n..=max_n).map(|n| stable_solution::<f64>(1.0, 1.0, 10.0.powi(-n)).unwrap());
    let stable = (min_n..=max_n)
        .map(|n| {
            other_stable_solution(
                a.into(),
                b.into(),
                f512::from_f64(10.0).pow(f512::from_f64(-n as f64)),
            )
            .unwrap()
        })
        .collect::<Vec<f512>>();

    // let (error_p, error_m): (Vec<f64>, Vec<f64>) = float_prec.zip(double_prec).map(|(a, b)| ((a.0 as f64 - b.0).abs(), (a.1 as f64 - b.1).abs())).unzip();
    let error_double = double_prec
        .zip(stable.iter())
        .map(|(a, &b)| (f512::from_f64(a) - b).abs().to_f64());
    let error_single = float_prec
        .zip(stable.iter())
        .map(|(a, &b)| (f512::from_f64(a as f64) - b).abs().to_f64());

    let line_double: Vec<(f64, f64)> = (min_n..=max_n)
        .map(|n: i32| 10.0.powi(-n))
        .zip(error_double)
        .collect();
    let line_single: Vec<(f64, f64)> = (min_n..=max_n)
        .map(|n: i32| 10.0.powi(-n))
        .zip(error_single)
        .collect();
    // let line_m: Vec<(f64, f64)> = (1..=max_n).map(|n: i32| 10.0.powi(-n) as f64).zip(error_m).collect();

    let prec_line_double = (min_n..=max_n)
        .map(|n| 10.0.powi(-n))
        .map(|x: f64| (x, x.next_up() - x))
        .map(|(x, prec)| (x, prec))
        .collect::<Vec<(f64, f64)>>();

    let prec_line_single = (min_n..=max_n)
        .map(|n| 10.0.powi(-n) as f32)
        .map(|x| (x, x.next_up() - x))
        .map(|(x, prec)| (x as f64, prec as f64))
        .collect::<Vec<(f64, f64)>>();

    println!(
        "double error {:?}",
        line_double.iter().map(|(_, y)| *y).collect::<Vec<f64>>()
    );

    utils::plotting::line_graph(
        vec![
            (line_double, "Error f64".to_string()),
            (line_single, "Error f32".to_string()),
            (prec_line_double, "f64 Machine Epsilon".to_string()),
            (prec_line_single, "f32 Machine Epsilon".to_string()),
        ],
        PlotConfig::default()
            .title("Error stable vs f32 and f64")
            .x_label("c")
            .y_label("error")
            .logarithmic_y(true)
            .logarithmic_x(true),
        "solutions/03/img/quadratic_eq_error.png",
    );
}

// ------------------------------------------
// Ex 2
// ------------------------------------------

type Real = f64;
const ONE: Real = 1.0;
const ZERO: Real = 0.0;
const PI: Real = f64::consts::PI;

fn runge(x: Real) -> Real {
    ONE / (ONE + x * x)
}

fn cheb_at(k: i32, n: i32) -> Real {
    let a = Real::from(-5);
    let b = Real::from(5);
    (b - a) / Real::from(2) * (Real::from(n - k) / Real::from(n) * PI).cos()
        + (b + a) / Real::from(2)
}
fn node_at(k: i32, n: i32) -> Real {
    // Real::from(- 5.0) + Real::from(10.0) / Real::from(n) * Real::from(k)
    cheb_at(k, n)
}

fn newton(x: Real, j: i32, n: i32) -> Real {
    if j == 0 {
        ONE
    } else {
        (0..j).fold(ONE, |acc, i| acc * (x - node_at(i, n)))
    }
}

const TICK_COUNT: i32 = 250;
const MIN_X: f64 = -5.0;
const MAX_X: f64 = 5.0;

fn make_range() -> impl Iterator<Item = f64> {
    let step_size = (MAX_X - MIN_X) / (TICK_COUNT as f64);

    (0..=TICK_COUNT).map(move |n| MIN_X + step_size * n as f64)
}

fn make_pol_line(n: i32) -> Vec<(f64, f64)> {
    let mut c = vec![ZERO; (n + 1) as usize];

    for i in 0..=n {
        c[i as usize] = runge(node_at(i, n));
    }
    for k in 1..=n {
        for i in (k..=n).rev() {
            let iu = i as usize;
            c[iu] = (c[iu] - c[iu - 1]) / (node_at(i, n) - node_at(i - k, n));
        }
    }

    let pol = |x: Real| (0..=n).map(|i| c[i as usize] * newton(x, i, n)).sum();

    // Plotting

    make_range()
        .map(|x| (x, pol(Real::from(x))))
        .collect::<Vec<(f64, f64)>>()
}

#[allow(non_snake_case)]
fn make_W_line(n: i32) -> Vec<(f64, f64)> {
    let error = |x: Real| (0..=n).fold(ONE, |acc, i| acc * (x - node_at(i, n)));

    make_range().map(|x| (x, error(x))).collect()
}

pub fn ex2() {
    let pol12_line = make_pol_line(12);
    let pol20_line = make_pol_line(20);

    let pol12_w_line = make_W_line(12);
    let pol20_w_line = make_W_line(20);

    let runge_line = make_range()
        .map(|x| (x, runge(Real::from(x))))
        .collect::<Vec<(f64, f64)>>();

    let error12_line: Vec<(f64, f64)> = pol12_line
        .iter()
        .zip(runge_line.iter())
        .map(|((x, p), (_, r))| (*x, (r - p).abs()))
        .collect();

    let error20_line: Vec<(f64, f64)> = pol20_line
        .iter()
        .zip(runge_line.iter())
        .map(|((x, p), (_, r))| (*x, (r - p).abs()))
        .collect();

    utils::plotting::line_graph(
        vec![
            (pol12_line, "Polynomial n=12".to_owned()),
            (pol20_line, "Polynomial n=20".to_owned()),
            (pol12_w_line, "W n=12".to_owned()),
            (pol20_w_line, "W n=20".to_owned()),
            (error12_line, "abs err n=12".to_owned()),
            (error20_line, "abs err n=20".to_owned()),
            (runge_line, "Runge".to_owned()),
        ],
        PlotConfig::default()
            .title(&"Polynomial of Runge function with error on [-5,5] and chebyshev".to_string())
            .x_label("x")
            .y_label("y"),
        "solutions/03/img/pol_error_w_ch.png",
    );
}
