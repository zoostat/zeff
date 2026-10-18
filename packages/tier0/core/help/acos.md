# acos

## Summary
Inverse cosine

## Description
The angle in radians, between 0 and pi, whose cosine is `x`, as a Float. A cosine lies between -1
and 1, so `x` outside that range has no inverse cosine and gives `NA` with a warning (W0205); `NA`
gives `NA`. Given a column, `acos` applies row by row and returns a column of Floats.

## Arguments
- x: A value between -1 and 1, an Int or a Float, or a column of either.

## Value
A Float; a column of Floats when `x` is a column.

## Details
Within two units in the last place of the exact value on every target.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic, section 9.2 (recommended correctly rounded functions); the implementation is the pure-Rust port of the musl C library (the `libm` crate), compared with R and with exact decimal arithmetic in the test suite.

## See also
cos, asin, atan

## Examples
> acos(1)
0
> acos(-1)
3.14159
> acos(0.5)
1.0472
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 4)
> acos(wages["wage"] / 100)
Column<Float> · 3 values
1.33461
1.37963
1.2535
