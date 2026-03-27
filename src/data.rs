use std::fs;
use std::io::{self, IsTerminal, Read};

use crate::cli::HELP;
use crate::model::{
    AxisValue, AxisValueKind, Config, PlotPoint, SeriesData, SeriesSpec, axis_value_kind,
};

pub(crate) fn load_series(config: &Config) -> Result<Vec<SeriesData>, String> {
    let mut stdin_data = None;
    let mut series = Vec::with_capacity(config.series.len());
    let x_kind = axis_value_kind(config.xformat.as_deref());
    let y_kind = axis_value_kind(config.yformat.as_deref());

    for spec in &config.series {
        let data = match spec.input.as_deref() {
            Some(path) => read_input(Some(path))?,
            None => {
                if stdin_data.is_none() {
                    stdin_data = Some(read_input(None)?);
                }
                stdin_data
                    .as_ref()
                    .ok_or_else(|| "failed to reuse stdin data".to_string())?
                    .clone()
            }
        };
        let points = parse_points(
            &data,
            spec.x_column,
            spec.y_column,
            x_kind,
            y_kind,
            &config.comment_markers,
        )?;
        if points.is_empty() {
            return Err(format!(
                "series {} has no plottable rows",
                series_label(spec)
            ));
        }
        series.push(SeriesData {
            label: series_label(spec),
            points,
        });
    }

    Ok(series)
}

fn read_input(input: Option<&str>) -> Result<String, String> {
    match input {
        Some("-") => read_stdin(),
        Some(path) => {
            fs::read_to_string(path).map_err(|error| format!("failed to read {path}: {error}"))
        }
        None => read_stdin(),
    }
}

fn read_stdin() -> Result<String, String> {
    let stdin = io::stdin();
    if stdin.is_terminal() {
        return Err(format!("missing input\n\n{HELP}"));
    }

    let mut data = String::new();
    stdin
        .lock()
        .read_to_string(&mut data)
        .map_err(|error| format!("failed to read stdin: {error}"))?;
    Ok(data)
}

pub(crate) fn parse_points(
    data: &str,
    x_column: Option<usize>,
    y_column: usize,
    x_kind: AxisValueKind,
    y_kind: AxisValueKind,
    comment_markers: &[String],
) -> Result<Vec<PlotPoint>, String> {
    let mut points = Vec::new();
    let mut point_index = 0usize;

    for (line_no, raw) in data.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || is_comment_line(line, comment_markers) {
            continue;
        }

        let columns: Vec<&str> = line.split_whitespace().collect();
        let y = columns
            .get(y_column - 1)
            .ok_or_else(|| format!("line {} does not have column {}", line_no + 1, y_column))?;

        let x = match x_column {
            Some(column) => {
                let x = columns.get(column - 1).ok_or_else(|| {
                    format!("line {} does not have column {}", line_no + 1, column)
                })?;
                parse_axis_value(x, x_kind, line_no + 1, 'X')?
            }
            None => AxisValue::Number((point_index + 1) as f64),
        };
        let y = parse_axis_value(y, y_kind, line_no + 1, 'Y')?;

        points.push(PlotPoint { x, y });
        point_index += 1;
    }

    Ok(points)
}

fn is_comment_line(line: &str, comment_markers: &[String]) -> bool {
    comment_markers
        .iter()
        .any(|marker| line.starts_with(marker))
}

fn parse_axis_value(
    value: &str,
    kind: AxisValueKind,
    line_no: usize,
    axis: char,
) -> Result<AxisValue, String> {
    match kind {
        AxisValueKind::Number => value
            .parse::<f64>()
            .map(AxisValue::Number)
            .map_err(|_| format!("line {} has non-numeric {} value: {}", line_no, axis, value)),
        AxisValueKind::Time => Ok(AxisValue::Text(value.to_string())),
    }
}

pub(crate) fn series_label(spec: &SeriesSpec) -> String {
    let input = spec.input.as_deref().unwrap_or("stdin");
    match spec.x_column {
        Some(x) => format!("{input} [{x}:{y}]", y = spec.y_column),
        None => format!("{input} [{y}]", y = spec.y_column),
    }
}
