# tgplot

`tgplot` plots numeric text columns in the terminal through `gnuplot`.

It is intended for commands such as:

```bash
tgplot --in text.txt using 1 2
tgplot --in a.txt b.txt using 1 2
tgplot --in text.txt using 2
tgplot --in a.txt using 1 2 --in b.txt using 1 2
tgplot --in a.txt --in b.txt using 2
tgplot --in text.txt using 1 2 --format x '%Y-%m-%dT%H:%M:%S'
tgplot --in text.txt using 1 2 --format y '%.5f'
tgplot using 1 2 < text.txt
tgplot using 2 < text.txt
tgplot using 1 2 --in text.txt --style points
tgplot --in text.txt using 1 2 --range x 0 10 y -10 10
tgplot --in text.txt using 1 2 --logscale y
tgplot --in text.txt using 1 2 --label x Time y Flux
tgplot --in text.txt using 1 2 --set 'set samples 400'
```

By default it uses a Unicode terminal plot:

```gnuplot
set term block braille ansi
```

## Options

- `--in FILE...`
- `using X Y`
- `--title TEXT`
- `--label x|y TEXT`
- `--format x|y FORMAT`
- `--range x|y MIN MAX`
- `--logscale x|y|xy`
- `--style lines|points|linespoints`
- `--key yes|no`
- `--grid yes|no`
- `--comments MARK...`
- `--set CMD`
- `--width N`
- `--height N`
- `--dumb`

## Notes

- Input is split on whitespace.
- Empty lines and lines starting with `#` are ignored.
- `X` and `Y` are 1-based column numbers.
- `using Y` plots a single column against the row index `1, 2, 3, ...`.
- Comment lines starting with `#` are ignored by default.
- `--comments # ! %` changes the ignore markers and can accept multiple markers.
- Repeating `--in ... using ...` adds independent plot series.
- `--in a.txt --in b.txt using 2` applies the same `using` clause to both files.
- `--in a.txt b.txt using 2` is the shorter equivalent.
- Options may appear before or after `using X Y`.
- `--label x 'Time' y 'Flux'` can set both axis labels in one `--label`.
- `--format x '%H:%M:%S' y '%e'` can set both axis formats in one `--format`.
- `--range x 0 10 y -1 1` can set both axis ranges in one `--range`.
- `--format x '%H:%M:%S'` or `--format x '%Y-%m-%dT%H:%M:%S'` enables a time-formatted x axis.
- `--format y '%.5f'` keeps numeric formatting on y.
- Time-like `--format x|y ...` values are passed to `gnuplot` as `set xdata/ydata time`, `set timefmt`, and `set format x/y`.
- `--set CMD` can be repeated for raw `gnuplot` commands that do not yet have dedicated CLI options.
- `gnuplot` must be installed and available on `PATH`.
