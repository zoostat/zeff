# Known issues

The limits and rough edges of the engine as released, kept current from the pre-release routine of
each release (testing handbook, section 2.4) and copied into the release notes. An entry names the
release that introduced it and, where one is planned, the release that removes it. A wrong result
is never listed here as accepted: it is fixed or the function is withdrawn.

## As of 0.1.0

- **Printed tables count width in characters.** An East Asian wide character takes two cells in a
  terminal, so a frame or table holding such text misaligns by one cell per character. Values are
  never cut. A Unicode width table is a dependency decision for a later release.
- **`summary` prints fixed three-decimal statistics.** Values below 0.0005 in magnitude print as
  `0.000` and very large values print every digit before the point (section 10.2's form). `mean`
  and `sd` on their own print six significant digits and are the way to read such columns.
- **Dates are read from CSV only in ISO form or with a declared format.** A column that looks like
  dates in another form is read as text, with a note proposing the `types:` entry.
- **No `by:` on the statistics yet.** `mean`, `sd` and `summary` work on the whole frame; grouping
  arrives with `aggregate` (0.6.0).
- **A line beginning with `-` is refused (Z0305).** `-7` on its own line is not a statement; write
  `print(-7)` or `let x = -7`. Expressions over values arrive with 0.2.0.
- **`import` reads CSV only**, UTF-8, with `delim:`, `header:`, `skip:`, `na_values:`, `types:`
  and `names:`. `thousands:`, `encoding:`, compressed files and the other formats arrive from
  0.2.0 onwards, as the catalogue says.
- **Each project pins the exact engine version** (`zeff = "0.1.0"`). An engine of another
  version refuses the project with exit code 4; there is no launcher yet to pick the pinned engine
  (0.9.0).
- **The function catalogue's status column** is updated at each release close, not per function,
  so between releases it lags the registry; `zeff help <package>` shows what is shipped.
