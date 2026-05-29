use ndarray::prelude::*;
use plotters::coord::Shift;
use plotters::coord::ranged1d::ValueFormatter;
use plotters::prelude::*;
use plotters::style::full_palette::{GREY, ORANGE, PURPLE};
use std::ops::Range;
use std::path::Path;

pub struct PlotConfig {
    title: String,
    x_label: String,
    y_label: String,
    y_min_0: bool,
    logarithmic_x: bool,
    logarithmic_y: bool,
    point_size: i64,
}

impl PlotConfig {
    pub fn default() -> Self {
        Self {
            title: "Plot".to_string(),
            x_label: "x".to_string(),
            y_label: "y".to_string(),
            y_min_0: false,
            logarithmic_x: false,
            logarithmic_y: false,
            point_size: 1,
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

    pub fn logarithmic_x(mut self, logarithmic_x: bool) -> Self {
        self.logarithmic_x = logarithmic_x;
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

    pub fn y_min_0(mut self, y_min_0: bool) -> Self {
        self.y_min_0 = y_min_0;
        self
    }
}

fn make_log(p: (f64, f64), log_x: bool, log_y: bool) -> (f64, f64) {
    if p.0 == 0.0 || p.1 == 0.0 {
        return (p.0, p.1);
    }
    (
        if log_x { p.0.log10() } else { p.0 },
        if log_y { p.1.log10() } else { p.1 },
    )
}

pub fn line_graph(lines: Vec<(Vec<(f64, f64)>, String)>, config: PlotConfig, path: &str) {
    // create img dir
    let path_prefix = Path::new(path).parent().unwrap();
    std::fs::create_dir_all(path_prefix).unwrap();

    let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let lines = lines
        .into_iter()
        .map(|(vec, s)| {
            (
                vec.into_iter()
                    .map(|p| make_log(p, config.logarithmic_x, config.logarithmic_y))
                    .collect(),
                s,
            )
        })
        .collect::<Vec<(Vec<(f64, f64)>, String)>>();

    let flat_lines: Vec<_> = lines.iter().map(|e| e.0.clone()).flatten().collect();

    let max_x = flat_lines.iter().map(|e| e.0).reduce(f64::max).unwrap();
    let min_x = flat_lines.iter().map(|e| e.0).reduce(f64::min).unwrap();
    let mut max_y = flat_lines.iter().map(|e| e.1).reduce(f64::max).unwrap();
    let mut min_y = flat_lines.iter().map(|e| e.1).reduce(f64::min).unwrap();

    min_y = if config.y_min_0 && config.logarithmic_y {
        f64::EPSILON.log10()
    } else if config.y_min_0 {
        f64::EPSILON
    } else {
        min_y
    };

    if max_y < min_y {
        let tmp = max_y;
        max_y = min_y;
        min_y = tmp;
    }

    let x_axis = min_x..max_x;
    let y_axis = min_y..max_y;

    let mut binding = ChartBuilder::on(&root);

    let chart = binding
        .x_label_area_size(40)
        .y_label_area_size(60)
        .margin(20)
        .caption(config.title, ("sans-serif", 25).into_font());

    macro_rules! build_and_draw {
        ($x:expr, $y:expr) => {{
            let mut c = chart.build_cartesian_2d($x, $y).unwrap();
            draw_stuff(
                &mut c,
                lines,
                config.x_label.as_str(),
                config.y_label.as_str(),
                config.point_size,
                config.logarithmic_x,
                config.logarithmic_y,
            )
        }};
    }

    // match (config.logarithmic_x, config.logarithmic_y) {
    //     (true,  true)  => build_and_draw!(x_axis.log_scale(), y_axis.log_scale()),
    //     (true,  false) => build_and_draw!(x_axis.log_scale(), y_axis),
    //     (false, true)  => build_and_draw!(x_axis,             y_axis.log_scale()),
    //     (false, false) => build_and_draw!(x_axis,             y_axis),
    // }

    build_and_draw!(x_axis, y_axis);

    root.present().unwrap();
}

fn log_label(v: f64) -> String {
    format!("1e{}", v as i32)
}

const COLORS: &[RGBColor] = &[RED, BLUE, GREEN, MAGENTA, ORANGE, GREY, PURPLE];

fn draw_stuff<'a, DB, XT, YT>(
    chart: &mut ChartContext<'a, DB, Cartesian2d<XT, YT>>,
    lines: Vec<(Vec<(f64, f64)>, String)>,
    x_label: &str,
    y_label: &str,
    point_size: i64,
    format_log_x: bool,
    format_log_y: bool,
) where
    DB: DrawingBackend + 'a,
    XT: Ranged<ValueType = f64> + ValueFormatter<f64>,
    YT: Ranged<ValueType = f64> + ValueFormatter<f64>,
{
    let mut mesh = chart.configure_mesh();
    mesh
        .x_desc(x_label)
        .y_desc(y_label)
        // .x_labels(10)
        // .y_labels(10)
        ;

    if format_log_x {
        mesh.x_label_formatter(&|v| log_label(*v));
    }
    if format_log_y {
        mesh.y_label_formatter(&|v| log_label(*v));
    }

    mesh.draw().unwrap();

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
