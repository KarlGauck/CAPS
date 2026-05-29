use crate::utils;
use crate::utils::plotting::PlotConfig;
use num_traits::Float;
use std::fmt::Debug;

// ---------------------------------------------------------------------------
// Quadratic solver implementations
// ---------------------------------------------------------------------------

// x₂ via reine Vieta-Formel:  x₂ = -2c / (b - sqrt(b²-4ac))
// Instabil für b > 0 und kleines c:
// Nenner  b - sqrt(b²-4ac) ≈ b - b = 0  (Auslöschung)
fn vieta_x2<T: Float>(a: T, b: T, c: T) -> Option<T> {
    let disc = b * b - T::from(4.0).unwrap() * a * c;
    if disc < T::zero() {
        return None;
    }
    let denom = b - disc.sqrt();
    if denom == T::zero() {
        return None;
    }
    Some(T::from(-2.0).unwrap() * c / denom)
}

// x₂ via stabile Formel (bestversion):
// Für b >= 0: x₂ = (-b - sqrt(b²-4ac)) / 2a   (klassisch, kein Auslöschungsproblem)
// Für b < 0:  x₂ = -2c / (b - sqrt(b²-4ac))   (Vieta, da klassisch x₂ instabil wäre)
fn stable_x2<T: Float + Debug>(a: T, b: T, c: T) -> Option<T> {
    let disc = b * b - T::from(4.0).unwrap() * a * c;
    if disc < T::zero() {
        return None;
    }
    let two_a = T::from(2.0).unwrap() * a;
    if b >= T::zero() {
        // klassische Formel für x₂ ist stabil (b und sqrt haben gleiches Vorzeichen)
        Some((-b - disc.sqrt()) / two_a)
    } else {
        // Vieta für x₂ ist stabil (Nenner b - sqrt hat großen Betrag)
        let denom = b - disc.sqrt();
        Some(T::from(-2.0).unwrap() * c / denom)
    }
}

// ---------------------------------------------------------------------------
// Ex 1  –  Residuum |f(x₂)| = |x₂² + x₂ + c|  für a=b=1, c=10⁻ⁿ
// Zeigt Auslöschung im Nenner der Vieta-Formel für x₂
// ---------------------------------------------------------------------------

pub fn ex1() {
    let max_n = 19;
    let min_n = 1;

    let eval = |x: f64, c: f64| x * x + x + c;

    // Vieta x₂ — instabil (wie version2 in Python)
    let res_vieta: Vec<(f64, f64)> = (min_n..=max_n)
        .filter_map(|n| {
            let c = 10.0f64.powi(-n);
            let x2 = vieta_x2::<f64>(1.0, 1.0, c)?;
            let r = eval(x2, c).abs();
            if r > 0.0 { Some((c, r)) } else { None }
        })
        .collect();

    // Stabile Formel x₂ — (wie bestversion in Python)
    let res_stable: Vec<(f64, f64)> = (min_n..=max_n)
        .filter_map(|n| {
            let c = 10.0f64.powi(-n);
            let x2 = stable_x2::<f64>(1.0, 1.0, c)?;
            let r = eval(x2, c).abs();
            if r > 0.0 { Some((c, r)) } else { None }
        })
        .collect();

    utils::plotting::line_graph(
        vec![
            (
                res_vieta,
                "Vieta x₂ = -2c/(b - sqrt(b²-4ac))  — Auslöschung im Nenner".to_string(),
            ),
            (
                res_stable,
                "Stabil x₂ = (-b - sqrt(b²-4ac))/2a  — kein Auslöschungsproblem".to_string(),
            ),
        ],
        PlotConfig::default()
            .title("Residuum |x₂² + x₂ + c|  –  große Wurzel von  x² + x + c = 0")
            .x_label("c  (Parameter)")
            .y_label("|f(x₂)|  (Residuum)")
            .logarithmic_x(true)
            .logarithmic_y(true),
        "solutions/03/img/residual_x2_cancellation.png",
    );
}
