use std::env::join_paths;
use std::fmt::Debug;
use std::time::{Duration, Instant};
use num_traits::{Float, NumCast};

use crate::utils::plotting;


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
pub fn ex1() {
    let ks = [1000, 100000, 10000000, 100000000];

    // (double precision, reversed)
    let configurations = vec!(
        (false, false),
        (false, true),
        (true, false),
        (true, true),
    );

    for (isF64, reversed) in configurations {
        let (relative_error, duration) = if isF64 {
            sumHelper::<f64>(reversed)
        } else {
            let (e, d) = sumHelper::<f32>(reversed);
            (e.map(|x| x as f64), d)
        };

        plotting::line_graph(
            vec!(
                ks.map(|x| x as f64).iter().copied().zip(relative_error).collect::<Vec<(f64, f64)>>()
            ),
            "Relative error",
            "Iterations",
            "log_2(Error)",
            format!("{}{}Prec-relError.png", if reversed {"rev"} else {"not-rev"}, if isF64 {"double"} else {"single"}).as_str()
        );

        plotting::line_graph(
            vec!(
                ks.map(|x| x as f64).iter().copied().zip(duration.map(|d| d.as_millis() as f64)).collect::<Vec<(f64, f64)>>()
            ),
            "Duration",
            "Iterations",
            "Milliseconds",
            format!("{}{}Prec-durations.png", if reversed {"rev"} else {"not-rev"}, if isF64 {"double"} else {"single"}).as_str()
        );
    }
}

fn sumHelper<T: Float + Debug>(reverse: bool) -> ([f64; 4], [Duration; 4]) {
    let SINF: f64 = std::f64::consts::PI.powf(2.0) / 6.0;
    let ks = [1000, 100000, 10000000, 100000000];

    let mut relativeError = [0.0; 4];
    let mut duration = [Duration::new(0, 0); 4];

    for (index, &k) in ks.iter().enumerate() {
        let vec: Vec<T> = if reverse {
            (1..k).rev().map(|x| NumCast::from(x).unwrap()).collect()
        } else {
            (1..k).map(|x| NumCast::from(x).unwrap()).collect()
        };

        let time = Instant::now();
        let sum = sum::<T>(&vec);

        duration[index] = time.elapsed();
        relativeError[index] = Float::abs(sum.to_f64().unwrap() - SINF) / SINF;
    }

    relativeError = relativeError.map(|x| x.log2());

    (relativeError, duration)
}

pub fn sum<T: Float>(list: &Vec<T>) -> T {
    let mut sum: T = T::zero();
    for &i in list {
        sum = sum + T::one()/(i*i)
    }
    sum
}