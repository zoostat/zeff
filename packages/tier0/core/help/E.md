# E

## Summary
The constant e

## Description
The base of the natural logarithm, 2.718281828459045, as the nearest Float. It is a constant of
`core`: written bare, without parentheses, and it cannot be rebound (Z0105). R has no constant for it
and writes `exp(1)`.

## Arguments


## Value
A Float, 2.718281828459045.

## Details
The Float nearest to e; it prints to six significant digits as `2.71828`.

## Coming from R or Stata
| E | exp(1) | exp(1) |

Neither R nor Stata has a name for it.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic.

## See also
PI, exp

## Examples
// The constant.
> E
2.71828

// Its square.
> E ^ 2
7.38906

// Its natural logarithm is one.
> log(E)
1
