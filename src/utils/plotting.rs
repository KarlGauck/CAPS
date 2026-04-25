use plotters::prelude::*;
use std::ops::Range;
use ndarray::prelude::*;
use plotters::coord::Shift;

pub fn line_graph(lines: Vec<Vec<(f64, f64)>>, title: &str, x_label: &str, y_label: &str, path: &str) {
    let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();
    root.fill(&WHITE).unwrap();


    let max_x = lines.iter().flatten().copied().map(|(x, y)| x).reduce(f64::max).unwrap();
    let min_x = lines.iter().flatten().copied().map(|(x, y)| x).reduce(f64::min).unwrap();
    let max_y = lines.iter().flatten().copied().map(|(x, y)| y).reduce(f64::max).unwrap();
    let min_y = lines.iter().flatten().copied().map(|(x, y)| y).reduce(f64::min).unwrap();
    let x_axis = min_x..max_x;
    let y_axis = min_y..max_y;

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .caption(title, ("sans-serif", 25).into_font())
        .build_cartesian_2d(x_axis, y_axis).unwrap();

    chart.configure_mesh()
        .x_desc(x_label)
        .y_desc(y_label).draw().unwrap();

    for points in lines {
        chart.draw_series(LineSeries::new(points, &RED).point_size(2)).unwrap();
    }

    root.present().unwrap();
}