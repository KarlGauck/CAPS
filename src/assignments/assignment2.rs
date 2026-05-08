use std::{f64, usize};
use crate::utils;
use chrono;
use chrono::{DateTime, Datelike, NaiveDate};
use crate::utils::plotting::PlotConfig;

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

struct PlanetPositionJ2000 {
    semimajor_axis: f64,
    eccentricity: f64,
    phi0: f64,
    mean_longitude: f64,
}


fn calculate_starting_mean_anomalies(points_on_orbit: usize) -> Vec<f64> {
    (0..points_on_orbit).map(|x| ((x as f64)/(points_on_orbit as f64)) * 2. * f64::consts::PI).collect()
}

fn get_initial_eccentricity(orbit: &Orbit, mean_anomaly: &f64) -> f64 {
    let ecc_threshold = 0.8;
    if (orbit.eccentricity < ecc_threshold) {
        *mean_anomaly
    } else {
        f64::consts::PI
    }
}


// Distribute a number of points equally over [0, 2pi] for the mean anomalies, then approximate the eccentric anomalies using fixedpoint method
fn calculate_orbit_basic_fixpoint(orbit: &Orbit, mean_anomalies: &Vec<f64>, precision: f64) -> (Vec<f64>, Vec<usize>) {

    let eccentric_anomalies: Vec<(f64, usize)> = mean_anomalies.iter().map(|mean_anomaly|
        fixed_point_iteration(precision, get_initial_eccentricity(&orbit, mean_anomaly), |old_eccentric_anomaly, _iterations| {
            // Default fixedpoint iteration
            mean_anomaly + orbit.eccentricity * f64::sin(*old_eccentric_anomaly)
        })
    ).collect();

    eccentric_anomalies.into_iter().unzip()
}

fn calculate_orbit_newton_raphson(orbit: &Orbit, mean_anomalies: &Vec<f64>, precision: f64) -> (Vec<f64>, Vec<usize>) {
    let eccentric_anomalies: Vec<(f64, usize)> = mean_anomalies.iter().map(|mean_anomaly|
        fixed_point_iteration(precision, get_initial_eccentricity(&orbit, mean_anomaly), |old_eccentric_anomaly, _iterations| {
            // Newton Raphson
            let g = old_eccentric_anomaly - orbit.eccentricity * old_eccentric_anomaly.sin() - mean_anomaly;
            let dg = 1.0 - orbit.eccentricity * old_eccentric_anomaly.cos();
            old_eccentric_anomaly - g/dg
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


const EARTH_SEMIMAJOR_AXIS: f64 = 1.0;
const EARTH_PERIOD: f64 = 1.0;

fn orbital_period(semimajor_axis: f64) -> f64{
    semimajor_axis.sqrt().powf(3.0)
}


pub fn ex1() {
    let orbits = vec!(
        Orbit { eccentricity: 0.205, semimajor_axis: 0.39 },   // Mercury
        Orbit { eccentricity: 0.967, semimajor_axis: 17.8 },   // Halley's comet
    );

    let precision = 1e-9;

    let points_on_orbit = 256;
    let mean_anomalies = calculate_starting_mean_anomalies(points_on_orbit);

    for orbit in orbits {

        let default_result = calculate_orbit_basic_fixpoint(&orbit, &mean_anomalies, precision);
        let default = calculate_orbit_positions(&orbit, default_result.0);

        let newton_result = calculate_orbit_newton_raphson(&orbit, &mean_anomalies, precision);
        let newton = calculate_orbit_positions(&orbit, newton_result.0);

        utils::plotting::line_graph(
            vec!(
                (default.iter().map(|(x, y)| (*x, y + 5.0)).collect(), format!("Fixepoint:  {:?} Iterations", default_result.1.iter().copied().reduce(usize::max).unwrap())),
                (newton, format!("Newton Raphson:  {:?} Iterations", newton_result.1.iter().copied().reduce(usize::max).unwrap()))
            ),
            PlotConfig::default().title("Orbit (slight y-offset for better visibility)").x_label("x in AU").y_label("y in AU"),
            "orbit.png"
        )
    }


    let starting_date = NaiveDate::from_ymd_opt(1985, 1, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2024, 5, 29).unwrap();
    let j2000 = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();

    let points_per_year = 200;
    let point_ratio = (points_per_year as f64) / 365.0;
    let time_duration_days = (end_date - starting_date).num_days().abs();
    let num_points = ((time_duration_days as f64) * point_ratio) as i64;

    println!("Num points: {:?}", num_points);



    let planets = vec!{
        PlanetPositionJ2000 { semimajor_axis: EARTH_SEMIMAJOR_AXIS, eccentricity: 0.0167, phi0: 102.95, mean_longitude: 100.46 }, // earth
        PlanetPositionJ2000 { semimajor_axis: 1.524, eccentricity: 0.0934, phi0: 336.04, mean_longitude: 355.45 }  // mars
    };

    let orbits: Vec<Vec<(f64, f64)>> = planets.iter().map(|planet| {
        let orbit = Orbit {
            semimajor_axis: planet.semimajor_axis,
            eccentricity: planet.eccentricity,
        };

        let orbital_period = orbital_period(planet.semimajor_axis);

        let j2000_diff = (starting_date - j2000).num_days().abs();
        let starting_mean_anomaly = planet.mean_longitude - planet.phi0 + (j2000_diff as f64) * 2.0 * f64::consts::PI / orbital_period;


        let anomalies: Vec<f64> = (0..num_points).map( |point| {
            2.0 * f64::consts::PI * (point as f64 / points_per_year as f64) /orbital_period + starting_mean_anomaly
        }).collect();

        let positions = calculate_orbit_positions(&orbit, calculate_orbit_newton_raphson(&orbit, &anomalies, precision).0);
        println!("positions: {:?}", positions.len());
        positions
    }).collect();

    let distances = orbits[0].iter().zip(&orbits[1]).map(|((x1, y1), (x2, y2))| f64::sqrt((x2-x1).powf(2.0) + (y2-y1).powf(2.0))).collect::<Vec<_>>();
    let plot = ((0..num_points).map(|x| (starting_date.year() as f64) + ((x as f64) / (points_per_year as f64))).zip(distances).collect::<Vec<_>>(), "Distance".to_string());

    utils::plotting::line_graph(
        vec!(plot),
        PlotConfig::default().title("Mercury / Earth distance").x_label("year").y_label("distance in AU").point_size(1),
        "distance.png"
    );

}