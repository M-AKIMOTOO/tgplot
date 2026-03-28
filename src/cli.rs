use std::env;

use crossterm::terminal;

use crate::model::{
    AxisRange, AxisValueKind, CliAction, Config, LogScale, PlotStyle, SeriesSpec, axis_value_kind,
};

pub(crate) const HELP: &str = "\
tgplot - plot text columns in the terminal using gnuplot

Usage:
  tgplot
  tgplot --in FILE --columns X Y
  tgplot --in FILE... --columns X Y
  tgplot --in FILE --columns Y
  tgplot --in A --columns X Y --in B --columns X Y
  tgplot --in A --in B --columns Y
  tgplot --columns X Y < data.txt
  tgplot --columns Y < data.txt

Options:
  --in FILE...      Read one or more input files. Use - for stdin
  --comments MARK... Ignore lines containing these markers (default: #)
  --delimiter TXT   Split input on this delimiter instead of whitespace
  --columns N...    Plot Y against row index, X Y, or X Y1 Y2 ... for multiple series
  --title TEXT      Set the plot title
  --label AXIS TXT... Set axis labels for x or y
  --format AXIS FMT... Set axis format for x or y; accepts x/y pairs and time-like formats enable time data
  --range AXIS MIN MAX... Set axis ranges for x or y
  --logscale AXIS   Use logarithmic axes: x, y, or xy
  --style STYLE     Plot style: lines, points, linespoints (default: lines)
  --key VALUE       Legend on/off: yes/no, y/n, on/off, true/false
  --grid VALUE      Grid on/off: yes/no, y/n, on/off, true/false
  --layout KEY N... Set layout values such as width/height (defaults to the current terminal size)
  --dumb            Use gnuplot dumb terminal instead of block braille
  --set CMD         Pass a raw gnuplot command, e.g. --set 'set samples 400'
  --detail          Show the full README-based guide
  -h, --help        Show this help

Use --detail for examples and the full README-based guide.
";

const LONG_OPTIONS: &[&str] = &[
    "--help",
    "--detail",
    "--in",
    "--title",
    "--label",
    "--format",
    "--range",
    "--logscale",
    "--style",
    "--key",
    "--grid",
    "--delimiter",
    "--comments",
    "--columns",
    "--layout",
    "--set",
    "--dumb",
];

pub(crate) fn parse_args<I>(args: I) -> Result<CliAction, String>
where
    I: IntoIterator<Item = String>,
{
    let args: Vec<String> = args.into_iter().collect();

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
    let mut delimiter = None;
    let mut comment_markers = vec!["#".to_string()];
    let mut extra_set_commands = Vec::new();
    let (mut width, mut height) = default_layout();
    let mut dumb = false;
    let mut key_explicit = false;

    let mut i = 0;
    while i < args.len() {
        let option = resolve_long_option(&args[i])?;
        match option.as_str() {
            "-h" | "--help" => return Ok(CliAction::Help),
            "--detail" => return Ok(CliAction::Detail),
            "--in" => {
                let mut consumed = 0usize;
                while let Some(value) = args.get(i + 1 + consumed) {
                    if value.starts_with('-') {
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
            "--label" => {
                let mut consumed = 0usize;
                loop {
                    let axis_index = i + 1 + consumed;
                    let Some(axis) = args.get(axis_index) else {
                        break;
                    };
                    if axis.starts_with('-') {
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
                    if axis.starts_with('-') {
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
            "--range" => {
                let mut consumed = 0usize;
                loop {
                    let axis_index = i + 1 + consumed;
                    let Some(axis) = args.get(axis_index) else {
                        break;
                    };
                    if axis.starts_with('-') {
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
            "--delimiter" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| "missing value for --delimiter".to_string())?;
                if value.is_empty() {
                    return Err("empty value for --delimiter".to_string());
                }
                delimiter = Some(value.clone());
            }
            "--comments" => {
                let mut consumed = 0usize;
                let mut markers = Vec::new();
                while let Some(value) = args.get(i + 1 + consumed) {
                    if value.starts_with('-') {
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
            "--layout" => {
                let mut consumed = 0usize;
                loop {
                    let key_index = i + 1 + consumed;
                    let Some(key) = args.get(key_index) else {
                        break;
                    };
                    if key.starts_with('-') {
                        break;
                    }
                    let value = args
                        .get(key_index + 1)
                        .ok_or_else(|| format!("missing value for --layout key: {key}"))?;
                    match key.as_str() {
                        "width" => width = parse_positive_usize(value, "--layout width")?,
                        "height" => height = parse_positive_usize(value, "--layout height")?,
                        _ => {
                            return Err(format!(
                                "invalid --layout key: {key} (expected width or height)"
                            ));
                        }
                    }
                    consumed += 2;
                }
                if consumed == 0 {
                    return Err("missing key for --layout".to_string());
                }
                i += consumed;
            }
            "--dumb" => {
                dumb = true;
            }
            "--columns" => {
                let mut consumed = 0usize;
                let mut values = Vec::new();
                while let Some(value) = args.get(i + 1 + consumed) {
                    if value.starts_with('-') {
                        break;
                    }
                    values.push(value.clone());
                    consumed += 1;
                }
                if values.is_empty() {
                    return Err(format!("missing column after --columns\n\n{HELP}"));
                }
                let inputs = if pending_inputs.is_empty() {
                    vec![None]
                } else {
                    pending_inputs.drain(..).map(Some).collect()
                };
                if values.len() == 1 {
                    let y_column = parse_positive_usize(&values[0], "Y column")?;
                    for input in inputs {
                        series.push(SeriesSpec {
                            input,
                            x_column: None,
                            y_column,
                        });
                    }
                } else {
                    let x_column = Some(parse_positive_usize(&values[0], "X column")?);
                    let y_columns = values[1..]
                        .iter()
                        .map(|value| parse_positive_usize(value, "Y column"))
                        .collect::<Result<Vec<_>, _>>()?;
                    for input in inputs {
                        for &y_column in &y_columns {
                            series.push(SeriesSpec {
                                input: input.clone(),
                                x_column,
                                y_column,
                            });
                        }
                    }
                }
                i += consumed;
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
        for input in pending_inputs.drain(..) {
            series.push(SeriesSpec::auto(Some(input)));
        }
    }
    if series.is_empty() {
        series.push(SeriesSpec::auto(None));
    }
    if axis_value_kind(xformat.as_deref()) == AxisValueKind::Number {
        if let Some(xrange) = &xrange {
            validate_numeric_range_bound(&xrange.min, "--range x")?;
            validate_numeric_range_bound(&xrange.max, "--range x")?;
        }
    }
    if axis_value_kind(yformat.as_deref()) == AxisValueKind::Number {
        if let Some(yrange) = &yrange {
            validate_numeric_range_bound(&yrange.min, "--range y")?;
            validate_numeric_range_bound(&yrange.max, "--range y")?;
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
        delimiter,
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

fn default_layout() -> (usize, usize) {
    if let Ok((width, height)) = terminal::size() {
        let width = usize::from(width).max(1);
        let height = usize::from(height).saturating_sub(2).max(10);
        return (width, height);
    }

    (
        env_size("COLUMNS", 100),
        env_size("LINES", 30).saturating_sub(2).max(10),
    )
}

fn resolve_long_option(option: &str) -> Result<String, String> {
    if !option.starts_with("--") {
        return Ok(option.to_string());
    }
    if LONG_OPTIONS.contains(&option) {
        return Ok(option.to_string());
    }

    let matches: Vec<&str> = LONG_OPTIONS
        .iter()
        .copied()
        .filter(|candidate| candidate.starts_with(option))
        .collect();
    match matches.len() {
        0 => Ok(option.to_string()),
        1 => Ok(matches[0].to_string()),
        _ => Err(format!(
            "ambiguous option: {option} (could be {})\n\n{HELP}",
            matches.join(", ")
        )),
    }
}
