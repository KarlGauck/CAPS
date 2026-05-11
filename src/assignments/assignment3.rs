use std::fmt::Debug;
use num_traits::Float;
use crate::utils;
use crate::utils::plotting::PlotConfig;

fn classic_solution<T: Float + Debug>(a: T, b: T, c: T) -> Option<(T, T)>{
    let sqrt_arg: T = b.powi(2) - T::from(4.0).unwrap() * a * c;
    if sqrt_arg < T::zero()
        { None }
    else {
        Some(
            (
                -b + sqrt_arg.sqrt() / T::from(2.0).unwrap()*a,
                -b - sqrt_arg.sqrt() / T::from(2.0).unwrap()*a
            )
        )
    }
}

pub fn ex1() {
    let max_n = 10;
    let float_prec = (1..=max_n).map(|n| classic_solution::<f32>(1.0, 1.0, 10.0.powi(-n)).unwrap().1 as f64);
    let double_prec = (1..=max_n).map(|n| classic_solution::<f64>(1.0, 1.0, 10.0.powi(-n)).unwrap().1);

    let error: Vec<f64> = float_prec.zip(double_prec).map(|(a, b)| (a-b).abs()).collect();
    let line: Vec<(f64, f64)> = (1..=max_n).map(|x| x as f64).zip(error).collect();

    println!("{:?}", line.iter().map(|(x, y)| *y).collect::<Vec<f64>>());

    utils::plotting::line_graph(
        vec!(
            (line, "Error".to_string())
        ),
        PlotConfig::default().title("Error f32 vs f64").x_label("n").y_label("error").logarithmic_y(true),
        "solutions/03/img/quadratic_eq_error.png"
    );
}