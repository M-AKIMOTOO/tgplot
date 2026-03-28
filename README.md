# tgplot

`tgplot` plots numeric text columns in the terminal through `gnuplot`.

It is intended for commands such as:

```bash
tgplot < text.txt
tgplot --in text.txt --columns 1 2
tgplot --in a.txt b.txt --columns 1 2
tgplot --in text.txt --columns 2
tgplot --in a.txt --columns 1 2 --in b.txt --columns 1 2
tgplot --in a.txt --in b.txt --columns 2
tgplot --in text.txt --columns 1 2 --format x '%Y-%m-%dT%H:%M:%S'
tgplot --in text.txt --columns 1 2 --format y '%.5f'
tgplot --columns 1 2 < text.txt
tgplot --columns 2 < text.txt
tgplot --columns 1 2 --in text.txt --style points
tgplot --in data.csv --columns 1 2 --delimiter ','
tgplot --in text.txt --columns 1 2 --range x 0 10 y -10 10
tgplot --in text.txt --columns 1 2 --logscale y
tgplot --in text.txt --columns 1 2 --label x Time y Flux
tgplot --in text.txt --columns 1 2 --set 'set samples 400'
```

By default it uses a Unicode terminal plot:

```gnuplot
set term block braille ansi
```

## Options

- `--in FILE...`
- `--comments MARK...`
- `--delimiter TXT`
- `--columns N...`
- `--title TEXT`
- `--label x|y TEXT`
- `--format x|y FORMAT`
- `--range x|y MIN MAX`
- `--logscale x|y|xy`
- `--style lines|points|linespoints`
- `--key yes|no`
- `--grid yes|no`
- `--layout width|height N`
- `--dumb`
- `--set CMD`

## Notes

- By default, `tgplot` uses the current terminal size and `--layout` overrides it.
- Input is split on whitespace by default.
- `--delimiter ','` lets `tgplot` read csv-like input.
- Empty lines and lines containing `#` are ignored.
- `X` and `Y` are 1-based column numbers.
- If `--columns` is omitted, `tgplot` inspects the first plottable row.
- With two or more columns, omitted `--columns` behaves like `--columns 1 2`.
- With one column, omitted `--columns` behaves like `--columns 1` against the row index.
- `--columns Y` plots a single column against the row index `1, 2, 3, ...`.
- Lines containing `#` are ignored by default.
- `--comments # ! %` changes the ignore markers and can accept multiple markers.
- Repeating `--in ... --columns ...` adds independent plot series.
- `--in a.txt --in b.txt --columns 2` applies the same `--columns` clause to both files.
- `--in a.txt b.txt --columns 2` is the shorter equivalent.
- Options may appear before or after `--columns X Y`.
- `--layout width 100` and `--layout width 100 height 50` are both accepted.
- `--label x 'Time' y 'Flux'` can set both axis labels in one `--label`.
- `--format x '%H:%M:%S' y '%e'` can set both axis formats in one `--format`.
- `--range x 0 10 y -1 1` can set both axis ranges in one `--range`.
- `--format x '%H:%M:%S'` or `--format x '%Y-%m-%dT%H:%M:%S'` enables a time-formatted x axis.
- `--format y '%.5f'` keeps numeric formatting on y.
- Time-like `--format x|y ...` values are passed to `gnuplot` as `set xdata/ydata time`, `set timefmt`, and `set format x/y`.
- `--set CMD` can be repeated for raw `gnuplot` commands that do not yet have dedicated CLI options.
- If `gnuplot` does not support the `block` terminal, `tgplot` falls back to `dumb ansi`; use `--dumb` to select that mode explicitly.
- `gnuplot` must be installed and available on `PATH`.
