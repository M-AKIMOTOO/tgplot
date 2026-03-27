use std::env;

use crate::model::{
    AxisRange, AxisValueKind, CliAction, Config, LogScale, PlotStyle, SeriesSpec, axis_value_kind,
};

pub(crate) const HELP: &str = "\
tgplot - plot text columns in the terminal using gnuplot

Usage:
  tgplot --in FILE using X Y
  tgplot --in FILE... using X Y
  tgplot --in FILE using Y
  tgplot --in A using X Y --in B using X Y
  tgplot --in A --in B using Y
  tgplot using X Y < data.txt
  tgplot using Y < data.txt

Options:
  --in FILE...      Read one or more input files. Use - for stdin
  --title TEXT      Set the plot title
  --label AXIS TXT... Set axis labels for x or y
  --format AXIS FMT... Set axis format for x or y; accepts x/y pairs and time-like formats enable time data
  --range AXIS MIN MAX... Set axis ranges for x or y
  --logscale AXIS   Use logarithmic axes: x, y, or xy
  --style STYLE     Plot style: lines, points, linespoints (default: lines)
  --key VALUE       Legend on/off: yes/no, y/n, on/off, true/false
  --grid VALUE      Grid on/off: yes/no, y/n, on/off, true/false
  --comments MARK... Ignore lines starting with these markers (default: #)
  --set CMD         Pass a raw gnuplot command, e.g. --set 'set samples 400'
  --detail          Show the full README-based guide
  --width N         Terminal width in characters
  --height N        Terminal height in characters
  --dumb            Use gnuplot dumb terminal instead of block braille
  -h, --help        Show this help

Use --detail for examples and the full README-based guide.
";

pub(crate) fn parse_args<I>(args: I) -> Result<CliAction, String>
where
    I: IntoIterator<Item = String>,
{
    let args: Vec<String> = args.into_iter().collect();
    if args.is_empty() {
        return Ok(CliAction::Help);
    }

    let mut series = Vec::new();
    let mut pending_inputs = Vec::new();
    let mut title = None;
    let mut xlabel = None;
    let mut ylabel = None;
    let mut xformat = None;
    let mut yformat = None;
    let mut xrange = None;
    let mut yrange = None;
    let mut logscale = LogScale::None;
    let mut style = PlotStyle::Lines;
    let mut show_key = false;
    let mut show_grid = true;
    let mut comment_markers = vec!["#".to_string()];
    let mut extra_set_commands = Vec::new();
    let mut width = env_size("COLUMNS", 100);
    let mut height = env_size("LINES", 30).saturating_sub(2).max(10);
    let mut dumb = false;
    let mut key_explicit = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => return Ok(CliAction::Help),
            "--detail" => return Ok(CliAction::Detail),
            "--in" => {
                let mut consumed = 0usize;
                while let Some(value) = args.get(i + 1 + consumed) {
                    if value == "using" || value.starts_with('-') {
                        break;
                    }
                    pending_inputs.push(value.clone());
                    consumed += 1;
                }
                if consumed == 0 {
                    return Err("missing value for --in".to_string());
                }
                i += consumed;
            }
            "--title" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| "missing value for --title".to_string())?;
                title = Some(value.clone());
            }
            "--xlabel" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| "missing value for --xlabel".to_string())?;
                xlabel = Some(value.clone());
            }
            "--ylabel" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| "missing value for --ylabel".to_string())?;
                ylabel = Some(value.clone());
            }
            "--label" => {
                let mut consumed = 0usize;
                loop {
                    let axis_index = i + 1 + consumed;
                    let Some(axis) = args.get(axis_index) else {
                        break;
                    };
                    if axis == "using" || axis.starts_with('-') {
                        break;
                    }
                    let value = args
                        .get(axis_index + 1)
                        .ok_or_else(|| format!("missing label text for --label axis: {axis}"))?;
                    match axis.as_str() {
                        "x" => xlabel = Some(value.clone()),
                        "y" => ylabel = Some(value.clone()),
                        _ => {
                            return Err(format!("invalid --label axis: {axis} (expected x or y)"));
                        }
                    }
                    consumed += 2;
                }
                if consumed == 0 {
                    return Err("missing axis for --label".to_string());
                }
                i += consumed;
            }
            "--format" => {
                let mut consumed = 0usize;
                loop {
                    let axis_index = i + 1 + consumed;
                    let Some(axis) = args.get(axis_index) else {
                        break;
                    };
                    if axis == "using" || axis.starts_with('-') {
                        break;
                    }
                    let format = args.get(axis_index + 1).ok_or_else(|| {
                        format!("missing format string for --format axis: {axis}")
                    })?;
                    match axis.as_str() {
                        "x" => xformat = Some(format.clone()),
                        "y" => yformat = Some(format.clone()),
                        _ => {
                            return Err(format!("invalid --format axis: {axis} (expected x or y)"));
                        }
                    }
                    consumed += 2;
                }
                if consumed == 0 {
                    return Err("missing axis for --format".to_string());
                }
                i += consumed;
            }
            "--xrange" => {
                let (range, consumed) = parse_axis_range(&args, i + 1, "--xrange")?;
                xrange = Some(range);
                i += consumed;
            }
            "--yrange" => {
                let (range, consumed) = parse_axis_range(&args, i + 1, "--yrange")?;
                yrange = Some(range);
                i += consumed;
            }
            "--range" => {
                let mut consumed = 0usize;
                loop {
                    let axis_index = i + 1 + consumed;
                    let Some(axis) = args.get(axis_index) else {
                        break;
                    };
                    if axis == "using" || axis.starts_with('-') {
                        break;
                    }
                    let (range, range_consumed) =
                        parse_axis_range(&args, axis_index + 1, "--range")?;
                    match axis.as_str() {
                        "x" => xrange = Some(range),
                        "y" => yrange = Some(range),
                        _ => {
                            return Err(format!("invalid --range axis: {axis} (expected x or y)"));
                        }
                    }
                    consumed += 1 + range_consumed;
                }
                if consumed == 0 {
                    return Err("missing axis for --range".to_string());
                }
                i += consumed;
            }
            "--logscale" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| "missing value for --logscale".to_string())?;
                logscale = parse_logscale(value)?;
            }
            "--style" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| "missing value for --style".to_string())?;
                style = parse_style(value)?;
            }
            "--key" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| "missing value for --key".to_string())?;
                show_key = parse_toggle(value, "--key")?;
                key_explicit = true;
            }
            "--grid" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| "missing value for --grid".to_string())?;
                show_grid = parse_toggle(value, "--grid")?;
            }
            "--comments" => {
                let mut consumed = 0usize;
                let mut markers = Vec::new();
                while let Some(value) = args.get(i + 1 + consumed) {
                    if value == "using" || value.starts_with('-') {
                        break;
                    }
                    markers.push(value.clone());
                    consumed += 1;
                }
                if consumed == 0 {
                    return Err("missing marker for --comments".to_string());
                }
                comment_markers = markers;
                i += consumed;
            }
            "--set" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| "missing value for --set".to_string())?;
                extra_set_commands.push(value.clone());
            }
            "--width" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| "missing value for --width".to_string())?;
                width = parse_positive_usize(value, "--width")?;
            }
            "--height" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| "missing value for --height".to_string())?;
                height = parse_positive_usize(value, "--height")?;
            }
            "--dumb" => {
                dumb = true;
            }
            "using" => {
                let first = args
                    .get(i + 1)
                    .ok_or_else(|| format!("missing column after using\n\n{HELP}"))?;
                let second = args.get(i + 2);
                let inputs = if pending_inputs.is_empty() {
                    vec![None]
                } else {
                    pending_inputs.drain(..).map(Some).collect()
                };
                if let Some(value) = second {
                    if !value.starts_with('-') {
                        let x_column = Some(parse_positive_usize(first, "X column")?);
                        let y_column = parse_positive_usize(value, "Y column")?;
                        for input in inputs {
                            series.push(SeriesSpec {
                                input,
                                x_column,
                                y_column,
                            });
                        }
                        i += 2;
                    } else {
                        let y_column = parse_positive_usize(first, "Y column")?;
                        for input in inputs {
                            series.push(SeriesSpec {
                                input,
                                x_column: None,
                                y_column,
                            });
                        }
                        i += 1;
                    }
                } else {
                    let y_column = parse_positive_usize(first, "Y column")?;
                    for input in inputs {
                        series.push(SeriesSpec {
                            input,
                            x_column: None,
                            y_column,
                        });
                    }
                    i += 1;
                }
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown option: {other}\n\n{HELP}"));
            }
            other => {
                return Err(format!("unexpected argument: {other}\n\n{HELP}"));
            }
        }
        i += 1;
    }

    if !pending_inputs.is_empty() {
        return Err(format!("missing using clause for --in\n\n{HELP}"));
    }
    if series.is_empty() {
        return Err(format!("missing using X Y clause\n\n{HELP}"));
    }
    if axis_value_kind(xformat.as_deref()) == AxisValueKind::Number {
        if let Some(xrange) = &xrange {
            validate_numeric_range_bound(&xrange.min, "--xrange")?;
            validate_numeric_range_bound(&xrange.max, "--xrange")?;
        }
    }
    if axis_value_kind(yformat.as_deref()) == AxisValueKind::Number {
        if let Some(yrange) = &yrange {
            validate_numeric_range_bound(&yrange.min, "--yrange")?;
            validate_numeric_range_bound(&yrange.max, "--yrange")?;
        }
    }
    if !key_explicit && series.len() > 1 {
        show_key = true;
    }

    Ok(CliAction::Plot(Config {
        series,
        title,
        xlabel,
        ylabel,
        xformat,
        yformat,
        xrange,
        yrange,
        logscale,
        style,
        show_key,
        show_grid,
        comment_markers,
        extra_set_commands,
        width,
        height,
        dumb,
    }))
}

fn parse_style(value: &str) -> Result<PlotStyle, String> {
    match value {
        "lines" => Ok(PlotStyle::Lines),
        "points" => Ok(PlotStyle::Points),
        "linespoints" => Ok(PlotStyle::LinesPoints),
        _ => Err(format!(
            "invalid --style: {value} (expected lines, points, or linespoints)"
        )),
    }
}

fn parse_positive_usize(value: &str, label: &str) -> Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| format!("invalid {label}: {value}"))?;
    if parsed == 0 {
        Err(format!("{label} must be greater than 0"))
    } else {
        Ok(parsed)
    }
}

fn parse_logscale(value: &str) -> Result<LogScale, String> {
    match value {
        "x" => Ok(LogScale::X),
        "y" => Ok(LogScale::Y),
        "xy" | "yx" => Ok(LogScale::XY),
        _ => Err(format!(
            "invalid --logscale: {value} (expected x, y, or xy)"
        )),
    }
}

fn parse_toggle(value: &str, flag: &str) -> Result<bool, String> {
    match value {
        "yes" | "y" | "on" | "true" => Ok(true),
        "no" | "n" | "off" | "false" => Ok(false),
        _ => Err(format!(
            "invalid value for {flag}: {value} (expected yes/no, y/n, on/off, or true/false)"
        )),
    }
}

fn parse_axis_range(
    args: &[String],
    start: usize,
    label: &str,
) -> Result<(AxisRange, usize), String> {
    let min = args
        .get(start)
        .ok_or_else(|| format!("missing MIN value for {label}"))?;
    let max = args
        .get(start + 1)
        .ok_or_else(|| format!("missing MAX value for {label}"))?;
    Ok((
        AxisRange {
            min: min.clone(),
            max: max.clone(),
        },
        2,
    ))
}

fn validate_numeric_range_bound(value: &str, label: &str) -> Result<(), String> {
    if value == "*" {
        return Ok(());
    }
    value
        .parse::<f64>()
        .map(|_| ())
        .map_err(|_| format!("invalid value for {label}: {value}"))
}

fn env_size(name: &str, fallback: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|&value| value > 0)
        .unwrap_or(fallback)
}
