#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Config {
    pub(crate) series: Vec<SeriesSpec>,
    pub(crate) title: Option<String>,
    pub(crate) xlabel: Option<String>,
    pub(crate) ylabel: Option<String>,
    pub(crate) xformat: Option<String>,
    pub(crate) yformat: Option<String>,
    pub(crate) xrange: Option<AxisRange>,
    pub(crate) yrange: Option<AxisRange>,
    pub(crate) logscale: LogScale,
    pub(crate) style: PlotStyle,
    pub(crate) show_key: bool,
    pub(crate) show_grid: bool,
    pub(crate) comment_markers: Vec<String>,
    pub(crate) extra_set_commands: Vec<String>,
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) dumb: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SeriesSpec {
    pub(crate) input: Option<String>,
    pub(crate) x_column: Option<usize>,
    pub(crate) y_column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SeriesData {
    pub(crate) label: String,
    pub(crate) points: Vec<PlotPoint>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PlotPoint {
    pub(crate) x: AxisValue,
    pub(crate) y: AxisValue,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum AxisValue {
    Number(f64),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AxisRange {
    pub(crate) min: String,
    pub(crate) max: String,
}

#[derive(Debug)]
pub(crate) enum CliAction {
    Help,
    Detail,
    Plot(Config),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LogScale {
    None,
    X,
    Y,
    XY,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlotStyle {
    Lines,
    Points,
    LinesPoints,
}

impl PlotStyle {
    pub(crate) fn gnuplot_name(self) -> &'static str {
        match self {
            PlotStyle::Lines => "lines",
            PlotStyle::Points => "points",
            PlotStyle::LinesPoints => "linespoints",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AxisValueKind {
    Number,
    Time,
}

pub(crate) fn axis_value_kind(format: Option<&str>) -> AxisValueKind {
    if format.is_some_and(looks_like_time_format) {
        AxisValueKind::Time
    } else {
        AxisValueKind::Number
    }
}

fn looks_like_time_format(format: &str) -> bool {
    let bytes = format.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'%' {
            let spec = bytes[i + 1] as char;
            if matches!(
                spec,
                'Y' | 'y'
                    | 'm'
                    | 'b'
                    | 'B'
                    | 'd'
                    | 'H'
                    | 'I'
                    | 'M'
                    | 'S'
                    | 'T'
                    | 'a'
                    | 'A'
                    | 'j'
                    | 'p'
            ) {
                return true;
            }
        }
        i += 1;
    }
    false
}
