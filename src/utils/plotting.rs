use plotters::prelude::*;
use std::ops::Range;
use ndarray::prelude::*;
use plotters::coord::ranged1d::ValueFormatter;
use plotters::coord::Shift;


pub struct PlotConfig {
    title: String,
    x_label: String,
    y_label: String,
    logarithmic_y: bool,
    point_size: i64
}

impl PlotConfig {
    pub fn default() -> Self {
        Self {
            title: "Plot".to_string(),
            x_label: "x".to_string(),
            y_label: "y".to_string(),
            logarithmic_y: false,
            point_size: 1
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn x_label(mut self, x_label: &str) -> Self {
        self.x_label = x_label.to_string();
        self
    }

    pub fn y_label(mut self, y_label: &str) -> Self {
        self.y_label = y_label.to_string();
        self
    }

    pub fn logarithmic_y(mut self, logarithmic_y: bool) -> Self {
        self.logarithmic_y = logarithmic_y;
        self
    }

    pub fn point_size(mut self, point_size: i64) -> Self {
        self.point_size = point_size;
        self
    }
}


pub fn line_graph(lines: Vec<(Vec<(f64, f64)>, String)>, config: PlotConfig, path: &str) {
    let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let flat_lines: Vec<_> = lines.iter().map(|e|e.0.clone()).flatten().collect();

    let max_x = flat_lines.iter().map(|e|e.0).reduce(f64::max).unwrap();
    let min_x = flat_lines.iter().map(|e|e.0).reduce(f64::min).unwrap();
    let max_y = flat_lines.iter().map(|e|e.1).reduce(f64::max).unwrap();
    let min_y = flat_lines.iter().map(|e|e.1).reduce(f64::min).unwrap();

    let x_axis = min_x..max_x;
    let y_axis = min_y..max_y;

    let mut binding = ChartBuilder::on(&root);
    let chart = binding
        .x_label_area_size(40)
        .y_label_area_size(40)
        .caption(config.title, ("sans-serif", 25).into_font());
    if config.logarithmic_y {
        let mut chart = chart.build_cartesian_2d(x_axis, y_axis.log_scale()).unwrap();
        draw_stuff(&mut chart, lines, config.x_label.as_str(), config.y_label.as_str(), config.point_size)
    } else {
        let mut chart = chart.build_cartesian_2d(x_axis, y_axis).unwrap();
        draw_stuff(&mut chart, lines, config.x_label.as_str(), config.y_label.as_str(), config.point_size)
    };

    root.present().unwrap();
}


const COLORS: &[RGBColor] = &[RED, BLUE, GREEN, MAGENTA];

fn draw_stuff<'a, DB, XT, YT>(
    chart: &mut ChartContext<'a, DB, Cartesian2d<XT, YT>>,
    lines: Vec<(Vec<(f64, f64)>, String)>,
    x_label: &str,
    y_label: &str,
    point_size: i64,
) where
    DB: DrawingBackend + 'a,
    XT: Ranged<ValueType = f64> + ValueFormatter<f64>,
    YT: Ranged<ValueType = f64> + ValueFormatter<f64>,
{
    chart
        .configure_mesh()
        .x_desc(x_label)
        .y_desc(y_label)
        .draw()
        .unwrap();

    for (i, (points, label)) in lines.into_iter().enumerate() {
        let color = COLORS[i % COLORS.len()];
        chart
            .draw_series(LineSeries::new(points, &color).point_size(point_size as u32))
            .unwrap()
            .label(label)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));
    }

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()
        .unwrap();
}
