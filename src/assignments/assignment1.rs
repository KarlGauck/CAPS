use std::cmp::max;
use std::env::join_paths;
use std::fmt::Debug;
use std::time::{Duration, Instant};
use num_traits::{Float, NumCast};

use crate::utils::plotting;


pub fn ex4() {
    const MAX_ITERATIONS: usize = 25;
    let mut current = f64::sqrt(8.);

    let mut data = [0.; MAX_ITERATIONS];

    for i in 0..MAX_ITERATIONS {
        let n = 2+i;

        current = pi_iteration(current, n as i32);
        data[i] = current;
    }

    let data0 = data;

    let mut current = f64::sqrt(8.);
    for i in 0..MAX_ITERATIONS {
        let n = 2+i;

        current = pi_iteration_kahan(current, n as i32);
        data[i] = current;
    }

    // plotting::line_graph(
    //     vec!(
    //         (0..MAX_ITERATIONS).map(|x| x as f64).zip(data.map(|x| f64::abs(x - std::f64::consts::PI))).collect(),
    //         (0..MAX_ITERATIONS).map(|x| x as f64).zip(data0.map(|x| f64::abs(x - std::f64::consts::PI))).collect(),
    //         (0..MAX_ITERATIONS).map(|x| x as f64).zip((0..MAX_ITERATIONS).map(|x| f64::EPSILON)).collect()
    //     ),
    //     true,
    //     "PI Approximation Error",
    //     "iterations",
    //     "error",
    //     "pi_error.png"
    // )
}

fn pi_iteration(an: f64, n: i32) -> f64 {
    let p2n: f64 = 2.0.powi(n);
    p2n * f64::sqrt(2. - 2.*f64::sqrt(1. - (an/p2n).powf(2.)))
}

fn pi_iteration_kahan(an: f64, n: i32) -> f64 {
    let p2n: f64 = 2.0.powi(n);
    let zn = 2.*(an/(p2n*2.)).powf(2.)/(1. + f64::sqrt(1. - (an/p2n).powf(2.)));
    p2n * f64::sqrt(4.*zn)
}


pub fn ex3() {
    let a = 1e30f32;
    let b = 0.0f32;
    let res = f32::sqrt(a.powf(2.0) + b.powf(2.0));
    println!("res = {}", res);

    let scale = 1e20f32;
    let sum = (a/scale).powf(2.0) + (b/scale).powf(2.0);
    println!("sum = {}", sum);
    let sqrt = f32::sqrt(sum);
    println!("sqrt = {}", sqrt);
    let res = scale * sqrt;
    println!("res = {}", res);
}


// Determine machine epsilon
pub fn ex2() {
    let eps32 = machineepsilon::<f32>(0.001f32);
    let eps64 = machineepsilon::<f64>(0.001f64);

    println!("single precision: {}  double precision: {}   (measured)", eps32, eps64);
    println!("single precision: {}  double precision: {}   (rust def)", f32::EPSILON, f64::EPSILON);
}

fn machineepsilon<T: Float>(start: T) -> T {
    let mut small_end = T::zero();
    let mut big_end = start;

    let mut last_eps  = T::zero();
    let mut current_eps = T::one();

    while last_eps != current_eps {
        last_eps = current_eps;
        current_eps = (small_end + big_end)/T::from(2.0f64).unwrap();

        let test = (T::one()+current_eps) == T::one();

        if test {
            small_end = current_eps;
        } else {
            big_end = current_eps;
        }

    };

    big_end
}


// Plot different kinds of floatingpoint errors
pub fn ex1(path: &str) {
    let ks = [1e3, 1e6, 1e7, 1e8];

    // (double precision, reversed)
    let configurations = vec!(
        (false, false),
        (false, true),
        (true, false),
        (true, true),
    );

    let mut rel_error_lines = Vec::new();
    let mut duration_lines = Vec::new();
    let mut abs_lines = Vec::new();

    let ksx = ks.map(|x| x);

    for (is_f64, is_reversed) in configurations {
        let (relative_error, duration, a) = if is_f64 {
            sum_helper::<f64>(is_reversed)
        } else {
            let (e, d, a) = sum_helper::<f32>(is_reversed);
            (e.map(|x| x as f64), d, a)
        };

        let label = format!("{}, {}", if is_f64 {"f64"} else {"f32"}, if is_reversed {"reversed"} else {"forwards"});

        rel_error_lines.push((ksx.iter().cloned().zip(relative_error).collect::<Vec<(f64, f64)>>(), label.clone()));
        duration_lines.push((ksx.iter().cloned().zip(duration.map(|d| d.as_millis() as f64)).collect::<Vec<(f64, f64)>>(), label.clone()));
        abs_lines.push((ksx.iter().cloned().zip(a).collect(), label.clone()));
    }

    let rel_error_path = format!("{path}/Prec-relError.png");
    let duration_path = format!("{path}/Prec-durations.png");
    let abs_path = format!("{path}/Prec-abs.png");
    println!("Saving to {rel_error_path}, {duration_path}, {abs_path}");

    plotting::line_graph(
        rel_error_lines,
        true,
        "Relative error",
        "Iterations",
        "Error",
        &rel_error_path
    );

    plotting::line_graph(
        duration_lines,
        false,
        "Duration",
        "Iterations",
        "Milliseconds",
        &duration_path
    );

    plotting::line_graph(
        abs_lines,
        false,
        "Values",
        "Iterations",
        "Value",
        &abs_path
    );

}

fn sum_helper<T: Float + Debug>(reverse: bool) -> ([f64; 4], [Duration; 4], [f64; 4]) {
    let sinf: f64 = std::f64::consts::PI.powf(2.0) / 6.0;
    let ks = [1000, 100000, 10000000, 100000000];

    let mut relative_error = [0.0; 4];
    let mut duration = [Duration::new(0, 0); 4];
    let mut abs = [0.0; 4];

    for (index, &k) in ks.iter().enumerate() {
        let vec: Vec<T> = if reverse {
            (1..k).rev().map(|x| NumCast::from(x).unwrap()).collect()
        } else {
            (1..k).map(|x| NumCast::from(x).unwrap()).collect()
        };

        let time = Instant::now();
        let sum = sum::<T>(&vec);

        duration[index] = time.elapsed();
        relative_error[index] = Float::abs(sum.to_f64().unwrap() - sinf) / sinf;
        abs[index] = NumCast::from(sum).unwrap();
    }

    (relative_error, duration, abs)
}

pub fn sum<T: Float>(list: &Vec<T>) -> T {
    let mut sum: T = T::zero();
    for &i in list {
        sum = sum + T::one()/(i*i)
    }
    sum
}