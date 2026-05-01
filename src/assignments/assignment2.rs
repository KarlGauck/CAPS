use std::{f64, usize};
use std::iter::zip;
use crate::utils;

fn mean_anomaly(time: f64, phase: f64, period: f64) -> f64 {
    2. * std::f64::consts::PI * (time - phase) / period
}

fn fixed_point_iteration<T, F>(precision: T, starting_value: T, process: F) -> (T, usize) where F: Fn(&T, usize) -> T, T: num_traits::Signed + Clone + PartialOrd {
    let mut accumulator = starting_value;
    let mut error: T = T::zero();
    let mut iteration = 0;
    loop {
        let next_value = process(&accumulator, iteration);
        error = (next_value.clone() - accumulator).abs();
        accumulator = next_value;
        iteration += 1;

        if error < precision {
            break;
        }
    }
    (accumulator, iteration)
}

struct Orbit {
    eccentricity: f64,
    semimajor_axis: f64,
}


// Distribute a number of points equally over [0, 2pi] for the mean anomalies, then approximate the eccentric anomalies using fixedpoint method
fn calculate_orbit_basic_fixpoint(orbit: &Orbit, points_on_orbit: usize, precision: f64) -> (Vec<f64>, Vec<usize>) {
    let get_initial_value = |orbit: &Orbit, mean_anomaly: &f64| -> f64 {
        let ecc_threshold = 0.8;
        if (orbit.eccentricity < ecc_threshold) {
            *mean_anomaly
        } else {
            f64::consts::PI
        }
    };

    let mean_anomalies: Vec<f64> = (0..points_on_orbit).map(|x| ((x as f64)/(points_on_orbit as f64)) * 2. * f64::consts::PI).collect();
    let eccentric_anomalies: Vec<(f64, usize)> = mean_anomalies.iter().map(|mean_anomaly|
        fixed_point_iteration(precision, get_initial_value(&orbit, mean_anomaly), |old_eccentric_anomaly, _iterations| {
            mean_anomaly + orbit.eccentricity * f64::sin(*old_eccentric_anomaly)
        })
    ).collect();

    eccentric_anomalies.into_iter().unzip()
}



// Calculate the actual positions from the approximated eccentric anomalies
fn calculate_orbit_positions(orbit: &Orbit, eccentric_anomalies: Vec<f64>) -> Vec<(f64, f64)> {
    let true_anomalies: Vec<f64> = eccentric_anomalies.iter().map(|ecc_anomaly| {
        2.0*f64::atan2(f64::sqrt(1.+orbit.eccentricity) * f64::sin(*ecc_anomaly/2.0), f64::sqrt(1.-orbit.eccentricity) * f64::cos(*ecc_anomaly/2.0))
    }).collect();


    let sun_radii: Vec<f64> = true_anomalies.iter().map(|true_anomaly| {
        orbit.semimajor_axis*(1.0 + orbit.eccentricity.powf(2.0)) / (1.0 + orbit.eccentricity * f64::cos(*true_anomaly))
    }).collect();


    let points: Vec<(f64, f64)> = sun_radii.iter().zip(true_anomalies).map(|(sun_radius, true_anomaly)| {
        (sun_radius * f64::cos(true_anomaly), sun_radius * f64::sin(true_anomaly))
    }).collect();

    points
}


pub fn ex1() {
    let orbits = vec!(
        Orbit { eccentricity: 0.205, semimajor_axis: 0.39 },   // Mercury
        Orbit { eccentricity: 0.967, semimajor_axis: 17.8 },   // Halley's comet
    );

    for orbit in orbits {
        let precisions = vec!(0.1, 0.01, 0.001);
        let points_on_orbit = 256;

        let orbit_results: Vec<(Vec<(f64, f64)>, String)> = precisions.iter().map(|precision| {
            let fixpoint_result = calculate_orbit_basic_fixpoint(&orbit, points_on_orbit, *precision);
            let eccentric_anomalies = fixpoint_result.0;
            let points = calculate_orbit_positions(
                &orbit, eccentric_anomalies);

            (points, format!("precision: {precision}, iterations: {:?}", fixpoint_result.1.iter().copied().reduce(usize::max)))
        }).collect();

        utils::plotting::line_graph(
            orbit_results
        , false, "Orbit", "x in AU", "y in AU", "orbit.png")
    }
}