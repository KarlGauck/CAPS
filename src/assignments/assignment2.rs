

fn mean_anomaly(time: f64, phase: f64, period: f64) -> f64 {
    2. * std::f64::consts::PI * (time - phase) / period
}

fn fixed_point_iteration<T, F>(iterations: usize, starting_value: T, process: F) -> T where F: Fn(&T, usize) -> T {
    (0..iterations).fold(starting_value, |acc, i| process(&acc, i))
}



struct Orbit {
    eccentricity: f64,
    semimajor_axis: f64,
}



fn calculate_orbit(orbit: Orbit, points_on_orbit: usize, iterations: usize) {
    let get_initial_value = |orbit: &Orbit, mean_anomaly: &f64| -> f64 {
        if (orbit.eccentricity < 0.8) {
            *mean_anomaly
        } else {
            std::f64::consts::PI
        }
    };

    let mean_anomalies: Vec<f64> = (0..points_on_orbit).map(|x| ((x as f64)/(points_on_orbit as f64)) * 2. * std::f64::consts::PI).collect();
    let eccentric_anomalies = mean_anomalies.iter().map(|mean_anomaly|
        fixed_point_iteration(iterations, get_initial_value(&orbit, mean_anomaly), |old_eccentric_anomaly, _iterations| {
            mean_anomaly + orbit.eccentricity * f64::sin(*old_eccentric_anomaly)
        })
    );
}





pub fn ex1() {

}