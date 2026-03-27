use std::io::{self, Write};
use std::process::{Command, Stdio};

use crate::model::{AxisValue, AxisValueKind, Config, LogScale, SeriesData, axis_value_kind};

pub(crate) fn run_gnuplot(config: &Config, series: &[SeriesData]) -> Result<(), String> {
    let mut child = Command::new("gnuplot")
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| format!("failed to start gnuplot: {error}"))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "failed to open gnuplot stdin".to_string())?;

    write_gnuplot_script(&mut stdin, config, series)?;
    drop(stdin);

    let status = child
        .wait()
        .map_err(|error| format!("failed to wait for gnuplot: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("gnuplot exited with status {status}"))
    }
}

pub(crate) fn write_gnuplot_script<W: Write>(
    mut out: W,
    config: &Config,
    series: &[SeriesData],
) -> Result<(), String> {
    let x_kind = axis_value_kind(config.xformat.as_deref());
    let y_kind = axis_value_kind(config.yformat.as_deref());
    let timefmt = match (x_kind, y_kind) {
        (AxisValueKind::Time, AxisValueKind::Time) => {
            if config.xformat != config.yformat {
                return Err(
                    "time-formatted x and y axes currently require the same --format".to_string(),
                );
            }
            config.xformat.clone()
        }
        (AxisValueKind::Time, AxisValueKind::Number) => config.xformat.clone(),
        (AxisValueKind::Number, AxisValueKind::Time) => config.yformat.clone(),
        (AxisValueKind::Number, AxisValueKind::Number) => None,
    };

    if config.dumb {
        writeln!(
            out,
            "set term dumb ansi size {},{}",
            config.width, config.height
        )
        .map_err(io_error)?;
    } else {
        writeln!(
            out,
            "set term block braille ansi size {},{}",
            config.width, config.height
        )
        .map_err(io_error)?;
    }
    if config.show_key {
        writeln!(out, "set key").map_err(io_error)?;
    } else {
        writeln!(out, "unset key").map_err(io_error)?;
    }
    if config.show_grid {
        writeln!(out, "set grid").map_err(io_error)?;
    } else {
        writeln!(out, "unset grid").map_err(io_error)?;
    }
    if let Some(title) = &config.title {
        writeln!(out, "set title '{}'", escape_gnuplot(title)).map_err(io_error)?;
    }
    if let Some(xlabel) = &config.xlabel {
        writeln!(out, "set xlabel '{}'", escape_gnuplot(xlabel)).map_err(io_error)?;
    }
    if let Some(ylabel) = &config.ylabel {
        writeln!(out, "set ylabel '{}'", escape_gnuplot(ylabel)).map_err(io_error)?;
    }
    if x_kind == AxisValueKind::Time {
        writeln!(out, "set xdata time").map_err(io_error)?;
    }
    if y_kind == AxisValueKind::Time {
        writeln!(out, "set ydata time").map_err(io_error)?;
    }
    if let Some(timefmt) = &timefmt {
        writeln!(out, "set timefmt '{}'", escape_gnuplot(timefmt)).map_err(io_error)?;
    }
    if let Some(xformat) = &config.xformat {
        writeln!(out, "set format x '{}'", escape_gnuplot(xformat)).map_err(io_error)?;
    }
    if let Some(yformat) = &config.yformat {
        writeln!(out, "set format y '{}'", escape_gnuplot(yformat)).map_err(io_error)?;
    }
    if let Some(xrange) = &config.xrange {
        writeln!(
            out,
            "set xrange [{}:{}]",
            format_range_bound(&xrange.min, x_kind),
            format_range_bound(&xrange.max, x_kind)
        )
        .map_err(io_error)?;
    }
    if let Some(yrange) = &config.yrange {
        writeln!(
            out,
            "set yrange [{}:{}]",
            format_range_bound(&yrange.min, y_kind),
            format_range_bound(&yrange.max, y_kind)
        )
        .map_err(io_error)?;
    }
    match config.logscale {
        LogScale::None => {}
        LogScale::X => {
            writeln!(out, "set logscale x").map_err(io_error)?;
        }
        LogScale::Y => {
            writeln!(out, "set logscale y").map_err(io_error)?;
        }
        LogScale::XY => {
            writeln!(out, "set logscale xy").map_err(io_error)?;
        }
    }
    for command in &config.extra_set_commands {
        writeln!(out, "{command}").map_err(io_error)?;
    }
    write!(out, "plot ").map_err(io_error)?;
    for (index, item) in series.iter().enumerate() {
        if index > 0 {
            write!(out, ", ").map_err(io_error)?;
        }
        write!(
            out,
            "'-' using 1:2 with {} title '{}'",
            config.style.gnuplot_name(),
            escape_gnuplot(&item.label)
        )
        .map_err(io_error)?;
    }
    writeln!(out).map_err(io_error)?;
    for item in series {
        for point in &item.points {
            write_axis_value(&mut out, &point.x).map_err(io_error)?;
            write!(out, " ").map_err(io_error)?;
            write_axis_value(&mut out, &point.y).map_err(io_error)?;
            writeln!(out).map_err(io_error)?;
        }
        writeln!(out, "e").map_err(io_error)?;
    }
    Ok(())
}

fn write_axis_value<W: Write>(mut out: W, value: &AxisValue) -> io::Result<()> {
    match value {
        AxisValue::Number(number) => write!(out, "{number}"),
        AxisValue::Text(text) => write!(out, "{text}"),
    }
}

fn format_range_bound(value: &str, kind: AxisValueKind) -> String {
    if value == "*" {
        return "*".to_string();
    }
    match kind {
        AxisValueKind::Number => value.to_string(),
        AxisValueKind::Time => format!("'{}'", escape_gnuplot(value)),
    }
}

fn escape_gnuplot(text: &str) -> String {
    text.replace('\\', "\\\\").replace('\'', "\\'")
}

fn io_error(error: io::Error) -> String {
    format!("failed to write gnuplot input: {error}")
}
