use crate::cli::parse_args;
use crate::data::{parse_points, series_label};
use crate::gnuplot::write_gnuplot_script;
use crate::model::{
    AxisRange, AxisValue, AxisValueKind, CliAction, Config, LogScale, PlotPoint, PlotStyle,
    SeriesData, SeriesSpec, axis_value_kind,
};

fn unwrap_plot(action: CliAction) -> Config {
    match action {
        CliAction::Plot(config) => config,
        CliAction::Help => panic!("expected plot config, got help"),
        CliAction::Detail => panic!("expected plot config, got detail"),
    }
}

#[test]
fn parse_args_supports_requested_shape() {
    let config = unwrap_plot(
        parse_args([
            "--in".to_string(),
            "text.txt".to_string(),
            "using".to_string(),
            "1".to_string(),
            "2".to_string(),
        ])
        .unwrap(),
    );

    assert_eq!(
        config,
        Config {
            series: vec![SeriesSpec {
                input: Some("text.txt".to_string()),
                x_column: Some(1),
                y_column: 2,
            }],
            title: None,
            xlabel: None,
            ylabel: None,
            xformat: None,
            yformat: None,
            xrange: None,
            yrange: None,
            logscale: LogScale::None,
            style: PlotStyle::Lines,
            show_key: false,
            show_grid: true,
            comment_markers: vec!["#".to_string()],
            extra_set_commands: Vec::new(),
            width: config.width,
            height: config.height,
            dumb: false,
        }
    );
}

#[test]
fn parse_args_accepts_options_after_using_clause() {
    let config = unwrap_plot(
        parse_args([
            "--in".to_string(),
            "text.txt".to_string(),
            "using".to_string(),
            "1".to_string(),
            "2".to_string(),
            "--style".to_string(),
            "points".to_string(),
        ])
        .unwrap(),
    );

    assert_eq!(config.series.len(), 1);
    assert_eq!(config.series[0].input, Some("text.txt".to_string()));
    assert_eq!(config.series[0].x_column, Some(1));
    assert_eq!(config.series[0].y_column, 2);
    assert_eq!(config.style, PlotStyle::Points);
}

#[test]
fn parse_args_accepts_options_before_and_after_using_clause() {
    let config = unwrap_plot(
        parse_args([
            "--title".to_string(),
            "demo".to_string(),
            "--in".to_string(),
            "text.txt".to_string(),
            "using".to_string(),
            "2".to_string(),
            "4".to_string(),
            "--grid".to_string(),
            "n".to_string(),
        ])
        .unwrap(),
    );

    assert_eq!(config.title, Some("demo".to_string()));
    assert_eq!(config.series[0].input, Some("text.txt".to_string()));
    assert_eq!(config.series[0].x_column, Some(2));
    assert_eq!(config.series[0].y_column, 4);
    assert!(!config.show_grid);
}

#[test]
fn parse_args_supports_gnuplot_style_axis_options() {
    let config = unwrap_plot(
        parse_args([
            "--range".to_string(),
            "x".to_string(),
            "02:05:00".to_string(),
            "02:15:00".to_string(),
            "y".to_string(),
            "-10".to_string(),
            "10".to_string(),
            "--label".to_string(),
            "x".to_string(),
            "Time".to_string(),
            "y".to_string(),
            "Flux".to_string(),
            "--format".to_string(),
            "x".to_string(),
            "%H:%M:%S".to_string(),
            "--format".to_string(),
            "y".to_string(),
            "%.5f".to_string(),
            "--logscale".to_string(),
            "x".to_string(),
            "--set".to_string(),
            "set samples 400".to_string(),
            "using".to_string(),
            "1".to_string(),
            "2".to_string(),
        ])
        .unwrap(),
    );

    assert_eq!(
        config.xrange,
        Some(AxisRange {
            min: "02:05:00".to_string(),
            max: "02:15:00".to_string()
        })
    );
    assert_eq!(
        config.yrange,
        Some(AxisRange {
            min: "-10".to_string(),
            max: "10".to_string()
        })
    );
    assert_eq!(config.xlabel, Some("Time".to_string()));
    assert_eq!(config.ylabel, Some("Flux".to_string()));
    assert_eq!(config.xformat, Some("%H:%M:%S".to_string()));
    assert_eq!(config.yformat, Some("%.5f".to_string()));
    assert_eq!(config.logscale, LogScale::X);
    assert_eq!(
        config.extra_set_commands,
        vec!["set samples 400".to_string()]
    );
}

#[test]
fn parse_args_accepts_both_axis_formats_in_one_format_clause() {
    let config = unwrap_plot(
        parse_args([
            "--format".to_string(),
            "x".to_string(),
            "%H:%M:%S".to_string(),
            "y".to_string(),
            "%e".to_string(),
            "using".to_string(),
            "1".to_string(),
            "2".to_string(),
        ])
        .unwrap(),
    );

    assert_eq!(config.xformat, Some("%H:%M:%S".to_string()));
    assert_eq!(config.yformat, Some("%e".to_string()));
}

#[test]
fn parse_args_accepts_both_axis_labels_in_one_label_clause() {
    let config = unwrap_plot(
        parse_args([
            "--label".to_string(),
            "x".to_string(),
            "Time".to_string(),
            "y".to_string(),
            "Flux".to_string(),
            "using".to_string(),
            "1".to_string(),
            "2".to_string(),
        ])
        .unwrap(),
    );

    assert_eq!(config.xlabel, Some("Time".to_string()));
    assert_eq!(config.ylabel, Some("Flux".to_string()));
}

#[test]
fn parse_args_accepts_both_axis_ranges_in_one_range_clause() {
    let config = unwrap_plot(
        parse_args([
            "--range".to_string(),
            "x".to_string(),
            "0".to_string(),
            "10".to_string(),
            "y".to_string(),
            "-1".to_string(),
            "1".to_string(),
            "using".to_string(),
            "1".to_string(),
            "2".to_string(),
        ])
        .unwrap(),
    );

    assert_eq!(
        config.xrange,
        Some(AxisRange {
            min: "0".to_string(),
            max: "10".to_string()
        })
    );
    assert_eq!(
        config.yrange,
        Some(AxisRange {
            min: "-1".to_string(),
            max: "1".to_string()
        })
    );
}

#[test]
fn parse_args_accepts_single_using_column() {
    let config = unwrap_plot(
        parse_args([
            "--in".to_string(),
            "text.txt".to_string(),
            "using".to_string(),
            "2".to_string(),
        ])
        .unwrap(),
    );

    assert_eq!(config.series[0].input, Some("text.txt".to_string()));
    assert_eq!(config.series[0].x_column, None);
    assert_eq!(config.series[0].y_column, 2);
}

#[test]
fn parse_args_accepts_multiple_comment_markers() {
    let config = unwrap_plot(
        parse_args([
            "--comments".to_string(),
            "#".to_string(),
            "!".to_string(),
            "%".to_string(),
            "using".to_string(),
            "1".to_string(),
            "2".to_string(),
        ])
        .unwrap(),
    );

    assert_eq!(
        config.comment_markers,
        vec!["#".to_string(), "!".to_string(), "%".to_string()]
    );
}

#[test]
fn parse_args_accepts_key_and_grid_toggle_values() {
    let config = unwrap_plot(
        parse_args([
            "--key".to_string(),
            "yes".to_string(),
            "--grid".to_string(),
            "n".to_string(),
            "using".to_string(),
            "1".to_string(),
            "2".to_string(),
        ])
        .unwrap(),
    );

    assert!(config.show_key);
    assert!(!config.show_grid);
}

#[test]
fn parse_args_supports_multiple_independent_input_series() {
    let config = unwrap_plot(
        parse_args([
            "--in".to_string(),
            "a.txt".to_string(),
            "using".to_string(),
            "1".to_string(),
            "2".to_string(),
            "--in".to_string(),
            "b.txt".to_string(),
            "using".to_string(),
            "1".to_string(),
            "3".to_string(),
        ])
        .unwrap(),
    );

    assert_eq!(
        config.series,
        vec![
            SeriesSpec {
                input: Some("a.txt".to_string()),
                x_column: Some(1),
                y_column: 2,
            },
            SeriesSpec {
                input: Some("b.txt".to_string()),
                x_column: Some(1),
                y_column: 3,
            },
        ]
    );
    assert!(config.show_key);
}

#[test]
fn parse_args_applies_single_using_clause_to_multiple_inputs() {
    let config = unwrap_plot(
        parse_args([
            "--in".to_string(),
            "a.txt".to_string(),
            "b.txt".to_string(),
            "using".to_string(),
            "2".to_string(),
        ])
        .unwrap(),
    );

    assert_eq!(
        config.series,
        vec![
            SeriesSpec {
                input: Some("a.txt".to_string()),
                x_column: None,
                y_column: 2,
            },
            SeriesSpec {
                input: Some("b.txt".to_string()),
                x_column: None,
                y_column: 2,
            },
        ]
    );
    assert!(config.show_key);
}

#[test]
fn parse_args_still_accepts_repeated_in_flags() {
    let config = unwrap_plot(
        parse_args([
            "--in".to_string(),
            "a.txt".to_string(),
            "--in".to_string(),
            "b.txt".to_string(),
            "using".to_string(),
            "2".to_string(),
        ])
        .unwrap(),
    );

    assert_eq!(config.series.len(), 2);
    assert_eq!(config.series[0].input, Some("a.txt".to_string()));
    assert_eq!(config.series[1].input, Some("b.txt".to_string()));
}

#[test]
fn parse_points_skips_comments() {
    let points = parse_points(
        "# comment\n1 2\n\n3 4\n",
        Some(1),
        2,
        AxisValueKind::Number,
        AxisValueKind::Number,
        &["#".to_string()],
    )
    .unwrap();
    assert_eq!(
        points,
        vec![
            PlotPoint {
                x: AxisValue::Number(1.0),
                y: AxisValue::Number(2.0)
            },
            PlotPoint {
                x: AxisValue::Number(3.0),
                y: AxisValue::Number(4.0)
            }
        ]
    );
}

#[test]
fn parse_points_skips_lines_containing_comment_markers() {
    let points = parse_points(
        "1 2\n12 # 34\n3 4\n",
        Some(1),
        2,
        AxisValueKind::Number,
        AxisValueKind::Number,
        &["#".to_string()],
    )
    .unwrap();
    assert_eq!(
        points,
        vec![
            PlotPoint {
                x: AxisValue::Number(1.0),
                y: AxisValue::Number(2.0)
            },
            PlotPoint {
                x: AxisValue::Number(3.0),
                y: AxisValue::Number(4.0)
            }
        ]
    );
}

#[test]
fn parse_points_uses_row_index_for_single_column_mode() {
    let points = parse_points(
        "# comment\n10\n\n20\n",
        None,
        1,
        AxisValueKind::Number,
        AxisValueKind::Number,
        &["#".to_string()],
    )
    .unwrap();
    assert_eq!(
        points,
        vec![
            PlotPoint {
                x: AxisValue::Number(1.0),
                y: AxisValue::Number(10.0)
            },
            PlotPoint {
                x: AxisValue::Number(2.0),
                y: AxisValue::Number(20.0)
            }
        ]
    );
}

#[test]
fn parse_points_accepts_time_values_on_x_axis() {
    let points = parse_points(
        "02:00:00 1.5\n03:00:00 2.5\n",
        Some(1),
        2,
        AxisValueKind::Time,
        AxisValueKind::Number,
        &["#".to_string()],
    )
    .unwrap();
    assert_eq!(
        points,
        vec![
            PlotPoint {
                x: AxisValue::Text("02:00:00".to_string()),
                y: AxisValue::Number(1.5)
            },
            PlotPoint {
                x: AxisValue::Text("03:00:00".to_string()),
                y: AxisValue::Number(2.5)
            }
        ]
    );
}

#[test]
fn gnuplot_script_uses_block_terminal_by_default() {
    let config = Config {
        series: vec![
            SeriesSpec {
                input: Some("a.txt".to_string()),
                x_column: Some(1),
                y_column: 2,
            },
            SeriesSpec {
                input: Some("b.txt".to_string()),
                x_column: None,
                y_column: 1,
            },
        ],
        title: Some("demo".to_string()),
        xlabel: Some("Time".to_string()),
        ylabel: Some("Flux".to_string()),
        xformat: Some("%H:%M:%S".to_string()),
        yformat: Some("%.2f".to_string()),
        xrange: Some(AxisRange {
            min: "02:05:00".to_string(),
            max: "02:15:00".to_string(),
        }),
        yrange: Some(AxisRange {
            min: "-1".to_string(),
            max: "1".to_string(),
        }),
        logscale: LogScale::X,
        style: PlotStyle::LinesPoints,
        show_key: true,
        show_grid: false,
        comment_markers: vec!["#".to_string()],
        extra_set_commands: vec!["set samples 400".to_string()],
        width: 80,
        height: 24,
        dumb: false,
    };

    let mut script = Vec::new();
    let series = vec![
        SeriesData {
            label: "a.txt [1:2]".to_string(),
            points: vec![
                PlotPoint {
                    x: AxisValue::Text("02:00:00".to_string()),
                    y: AxisValue::Number(1.0),
                },
                PlotPoint {
                    x: AxisValue::Text("03:00:00".to_string()),
                    y: AxisValue::Number(0.0),
                },
            ],
        },
        SeriesData {
            label: "b.txt [1]".to_string(),
            points: vec![
                PlotPoint {
                    x: AxisValue::Text("02:00:00".to_string()),
                    y: AxisValue::Number(2.0),
                },
                PlotPoint {
                    x: AxisValue::Text("03:00:00".to_string()),
                    y: AxisValue::Number(3.0),
                },
            ],
        },
    ];
    write_gnuplot_script(&mut script, &config, &series).unwrap();
    let script = String::from_utf8(script).unwrap();

    assert!(script.contains("set term block braille ansi size 80,24"));
    assert!(script.contains("set key"));
    assert!(script.contains("unset grid"));
    assert!(script.contains("set title 'demo'"));
    assert!(script.contains("set xlabel 'Time'"));
    assert!(script.contains("set ylabel 'Flux'"));
    assert!(script.contains("set xdata time"));
    assert!(script.contains("set timefmt '%H:%M:%S'"));
    assert!(script.contains("set format x '%H:%M:%S'"));
    assert!(script.contains("set format y '%.2f'"));
    assert!(script.contains("set xrange ['02:05:00':'02:15:00']"));
    assert!(script.contains("set yrange [-1:1]"));
    assert!(script.contains("set logscale x"));
    assert!(script.contains("set samples 400"));
    assert!(script.contains("plot '-' using 1:2 with linespoints title 'a.txt [1:2]', '-' using 1:2 with linespoints title 'b.txt [1]'"));
}

#[test]
fn axis_value_kind_detects_time_formats() {
    assert_eq!(axis_value_kind(Some("%H:%M:%S")), AxisValueKind::Time);
    assert_eq!(
        axis_value_kind(Some("%Y-%m-%dT%H:%M:%S")),
        AxisValueKind::Time
    );
    assert_eq!(axis_value_kind(Some("%.5f")), AxisValueKind::Number);
    assert_eq!(axis_value_kind(Some("%e")), AxisValueKind::Number);
}

#[test]
fn numeric_xrange_still_requires_numeric_bounds() {
    let error = parse_args([
        "--range".to_string(),
        "x".to_string(),
        "02:05:00".to_string(),
        "02:15:00".to_string(),
        "using".to_string(),
        "1".to_string(),
        "2".to_string(),
    ])
    .unwrap_err();

    assert!(error.contains("invalid value for --xrange"));
}

#[test]
fn parse_points_skips_multiple_comment_markers() {
    let points = parse_points(
        "! skip\n1 % 2\n3 4\n",
        Some(1),
        2,
        AxisValueKind::Number,
        AxisValueKind::Number,
        &["#".to_string(), "!".to_string(), "%".to_string()],
    )
    .unwrap();

    assert_eq!(
        points,
        vec![PlotPoint {
            x: AxisValue::Number(3.0),
            y: AxisValue::Number(4.0)
        }]
    );
}

#[test]
fn series_label_includes_columns() {
    assert_eq!(
        series_label(&SeriesSpec {
            input: Some("a.txt".to_string()),
            x_column: Some(1),
            y_column: 2,
        }),
        "a.txt [1:2]"
    );
    assert_eq!(
        series_label(&SeriesSpec {
            input: None,
            x_column: None,
            y_column: 3,
        }),
        "stdin [3]"
    );
}
