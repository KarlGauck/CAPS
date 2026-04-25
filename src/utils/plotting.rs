use plotters::prelude::*;
use std::ops::Range;
use ndarray::prelude::*;
use plotters::coord::ranged1d::ValueFormatter;
use plotters::coord::Shift;

pub fn line_graph(lines: Vec<Vec<(f64, f64)>>, logarithmic_y: bool, title: &str, x_label: &str, y_label: &str, path: &str) {
    let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();
    root.fill(&WHITE).unwrap();


    let max_x = lines.iter().flatten().copied().map(|(x, y)| x).reduce(f64::max).unwrap();
    let min_x = lines.iter().flatten().copied().map(|(x, y)| x).reduce(f64::min).unwrap();
    let max_y = lines.iter().flatten().copied().map(|(x, y)| y).reduce(f64::max).unwrap();
    let min_y = lines.iter().flatten().copied().map(|(x, y)| y).reduce(f64::min).unwrap();
    let x_axis = min_x..max_x;
    let y_axis = min_y..max_y;

    let mut binding = ChartBuilder::on(&root);
    let chart = binding
        .x_label_area_size(40)
        .y_label_area_size(40)
        .caption(title, ("sans-serif", 25).into_font());
    if logarithmic_y {
        let mut chart = chart.build_cartesian_2d(x_axis, y_axis.log_scale()).unwrap();
        draw_stuff(&mut chart, lines, x_label, y_label)
    } else {
        let mut chart = chart.build_cartesian_2d(x_axis, y_axis).unwrap();
        draw_stuff(&mut chart, lines, x_label, y_label)
    };

    root.present().unwrap();
}

fn draw_stuff<DB, XT, YT>(
    chart: &mut ChartContext<DB, Cartesian2d<XT, YT>>,
    lines: Vec<Vec<(f64, f64)>>,
    x_label: &str,
    y_label: &str
) where
    DB: DrawingBackend,
    XT: Ranged<ValueType = f64> + ValueFormatter<f64>,
    YT: Ranged<ValueType = f64> + ValueFormatter<f64>,
{
    chart.configure_mesh()
        .x_desc(x_label)
        .y_desc(y_label).draw().unwrap();

    for points in lines {
        chart.draw_series(LineSeries::new(points, &RED).point_size(2)).unwrap();
    }
}
