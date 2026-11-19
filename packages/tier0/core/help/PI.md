# PI

## Summary
The constant pi

## Description
The ratio of a circle's circumference to its diameter, 3.141592653589793, as the nearest Float. It is
a constant of `core`: written bare, without parentheses, and it cannot be rebound (Z0105). R has `pi`;
Stata has `_pi`.

## Arguments


## Value
A Float, 3.141592653589793.

## Details
The Float nearest to pi; it prints to six significant digits as `3.14159`.

## Coming from R or Stata
| PI | pi | _pi or c(pi) |

In R `pi` is an ordinary name and can be given another value; here `PI` cannot be rebound.

## References
IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic.

## See also
E, sin

## Examples
// The constant.
> PI
3.14159

// The circumference of a circle of radius one.
> 2 * PI
6.28319

// Ninety degrees in radians.
> 90 * PI / 180
1.5708
