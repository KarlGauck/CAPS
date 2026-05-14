use core::error;
use std::fmt::Debug;
use num_traits::{Float, float};
use crate::utils;
use crate::utils::plotting::PlotConfig;
use hyperprec::f512;

// 1.a
fn classic_solution<T: Float + Debug>(a: T, b: T, c: T) -> Option<(T, T)> {
    let sqrt_arg: T = b.powi(2) - T::from(4.0).unwrap() * a * c;
    if sqrt_arg < T::zero()
        { None }
    else {
        Some(
            (
                (-b + sqrt_arg.sqrt()) / (T::from(2.0).unwrap()*a),
                (-b - sqrt_arg.sqrt()) / (T::from(2.0).unwrap()*a)
            )
        )
    }
}

// Vieta
fn stable_solution<T: Float + Debug>(a: T, b: T, c: T) -> Option<T> {
    let sqrt_arg = b.powi(2) - T::from(4.0).unwrap() * a * c;
    if sqrt_arg < T::zero() { None }
    else {
        Some(T::from(-2.0).unwrap() * c / (b + sqrt_arg.sqrt()))
    }
}
fn other_stable_solution(a: f512, b: f512, c: f512) -> Option<f512> {
    let sqrt_arg = b.pow(f512::from_f64(2.0)) - f512::from(4.0) * a * c;
    if sqrt_arg < f512::ZERO { None }
    else {
        Some(f512::from(-2.0) * c / (b + sqrt_arg.sqrt()))
    }
}

// 1.d)

// Return the numerator of the first root of a 2nd degree polynomial
fn midnight_numerator<T: Float>(a: T, b: T, c: T) -> Option<T> {
    let sqrt_arg = b*b - T::from(4.0).unwrap() * a * c;
    if sqrt_arg < T::zero() { None } else {
        Some(-b + sqrt_arg.sqrt())
    }
}
fn final_stable_solution<T: Float + Debug>(a: T, b: T, c: T) -> Option<(T, T)> {
    if b > T::zero() {
        // num is 2nd root
        if let Some(num) = midnight_numerator(a, -b, c) {
            Some((
                - T::from(2.0).unwrap() * c / num,
                num
            ))
        } else { None }
    } else {
        // num is 1st root
        if let Some(num) = midnight_numerator(a, b, c) {
            Some((
                num,
                T::from(2.0).unwrap() * c / num
            ))
        } else { None }
    }
}

pub fn ex1() {
    let max_n = 25;
    let min_n = 1;

    let a = 1.0;
    let b = 1.0;

    let float_prec = (min_n..=max_n).map(|n| classic_solution::<f32>(a, b, 10.0.powi(-n)).unwrap().0);
    let double_prec = (min_n..=max_n).map(|n| classic_solution::<f64>(a.into(), b.into(), 10.0.powi(-n)).unwrap().0);
    // let stable = (min_n..=max_n).map(|n| stable_solution::<f64>(1.0, 1.0, 10.0.powi(-n)).unwrap());
    let stable = 
        (min_n..=max_n)
            .map(|n| other_stable_solution(a.into(), b.into(), f512::from_f64(10.0)
            .pow(f512::from_f64(-n as f64))).unwrap())
            .collect::<Vec<f512>>();

    // let (error_p, error_m): (Vec<f64>, Vec<f64>) = float_prec.zip(double_prec).map(|(a, b)| ((a.0 as f64 - b.0).abs(), (a.1 as f64 - b.1).abs())).unzip();
    let error_double = double_prec.zip(stable.iter()).map(|(a, &b)| (f512::from_f64(a) - b).abs().to_f64());
    let error_single = float_prec.zip(stable.iter()).map(|(a, &b)| (f512::from_f64(a as f64) - b).abs().to_f64());

    let line_double: Vec<(f64, f64)> = (min_n..=max_n).map(|n: i32| 10.0.powi(-n) as f64).zip(error_double).collect();
    let line_single: Vec<(f64, f64)> = (min_n..=max_n).map(|n: i32| 10.0.powi(-n) as f64).zip(error_single).collect();
    // let line_m: Vec<(f64, f64)> = (1..=max_n).map(|n: i32| 10.0.powi(-n) as f64).zip(error_m).collect();

    let prec_line_double = (min_n..=max_n)
        .map(|n| 10.0.powi(-n) as f64)
        .map(|x| (x, x.next_up() - x))
        .map(|(x, prec)| (x as f64, prec as f64))
        .collect::<Vec<(f64, f64)>>();

    let prec_line_single = (min_n..=max_n)
        .map(|n| 10.0.powi(-n) as f32)
        .map(|x| (x, x.next_up() - x))
        .map(|(x, prec)| (x as f64, prec as f64))
        .collect::<Vec<(f64, f64)>>();

    println!("{:?}", line_double.iter().map(|(_, y)| *y).collect::<Vec<f64>>());

    utils::plotting::line_graph(
        vec!(
            (line_double, "Error f64".to_string()),
            (line_single, "Error f32".to_string()),
            (prec_line_double, "f64 Machine Epsilon".to_string()),
            (prec_line_single, "f32 Machine Epsilon".to_string()),
        ),
        PlotConfig::default()
            .title("Error stable vs f32 and f64")
            .x_label("c")
            .y_label("error")
            .logarithmic_y(true)
            .logarithmic_x(true),
        "solutions/03/img/quadratic_eq_error.png"
    );
}