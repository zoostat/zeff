//! The elementary functions and the rounding and division rules of Language Reference section
//! 3.7, behind one pure-Rust implementation on every target (Engine and Project Specification,
//! section 8.3, point 3): `exp`, `log`, `sin` and the rest come from the `libm` crate, a port of
//! musl's maths library, never from the platform's C library, whose results differ between
//! systems. `sqrt`, `floor`, `ceil` and `trunc` are exact IEEE operations and come from Rust's own
//! intrinsics, which every target implements identically.
//!
//! Rounding is decided on the exact binary value, as R does: `round(2.675, 2)` is `2.67` because
//! the number stored for 2.675 lies below the tie. A tie is detected exactly from the bits, so
//! `round(2.5)` is `2` half to even and `3` half away from zero.

/// How a tie is broken: to the even neighbour, as R, or away from zero, as Stata and Excel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundingMode {
    Even,
    Away,
}

pub fn exp(x: f64) -> f64 {
    libm::exp(x)
}

/// The natural logarithm; `-Inf` at zero and not-a-number below it, for the caller's warnings.
pub fn log(x: f64) -> f64 {
    libm::log(x)
}

/// The logarithm to `base`: `log(x) / log(base)`, with the exact answers for bases 2 and 10.
pub fn log_base(x: f64, base: f64) -> f64 {
    if base == 2.0 {
        libm::log2(x)
    } else if base == 10.0 {
        libm::log10(x)
    } else {
        libm::log(x) / libm::log(base)
    }
}

pub fn log2(x: f64) -> f64 {
    libm::log2(x)
}

pub fn log10(x: f64) -> f64 {
    libm::log10(x)
}

pub fn log1p(x: f64) -> f64 {
    libm::log1p(x)
}

pub fn expm1(x: f64) -> f64 {
    libm::expm1(x)
}

pub fn sin(x: f64) -> f64 {
    libm::sin(x)
}

pub fn cos(x: f64) -> f64 {
    libm::cos(x)
}

pub fn tan(x: f64) -> f64 {
    libm::tan(x)
}

pub fn asin(x: f64) -> f64 {
    libm::asin(x)
}

pub fn acos(x: f64) -> f64 {
    libm::acos(x)
}

pub fn atan(x: f64) -> f64 {
    libm::atan(x)
}

pub fn atan2(y: f64, x: f64) -> f64 {
    libm::atan2(y, x)
}

/// The square root: IEEE's own, correctly rounded on every target.
pub fn sqrt(x: f64) -> f64 {
    x.sqrt()
}

/// -1, 0 or 1 (not-a-number for not-a-number).
pub fn sign(x: f64) -> f64 {
    if x.is_nan() {
        x
    } else if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    }
}

/// Whether `x * 10^digits` is exactly halfway between two integers. With `x = M * 2^E` for an odd
/// `M`, `2 * x * 10^digits = M * 5^digits * 2^(E + 1 + digits)` is an odd integer exactly when
/// `E + 1 + digits == 0`; for a negative `digits`, `x` must be an odd multiple of `5 * 10^(-digits - 1)`.
fn is_tie(x: f64, digits: i32) -> bool {
    if !x.is_finite() || x == 0.0 {
        return false;
    }
    let bits = x.abs().to_bits();
    let exponent_field = ((bits >> 52) & 0x7ff) as i32;
    let mantissa = bits & ((1u64 << 52) - 1);
    let (m, e) = if exponent_field == 0 {
        (mantissa, -1074)
    } else {
        (mantissa | (1u64 << 52), exponent_field - 1075)
    };
    let shift = m.trailing_zeros() as i32;
    let e = e + shift;
    if digits >= 0 {
        e + 1 + digits == 0
    } else {
        // An integer x (e >= 0) whose double is an odd multiple of 10^(-digits):
        // 2x = (2k + 1) * 10^(-digits).
        if e < 0 || x.abs() >= 9.0e15 {
            return false;
        }
        let value = (x.abs() as u64) * 2;
        let unit = 10u64.pow((-digits) as u32);
        value.is_multiple_of(unit) && (value / unit) % 2 == 1
    }
}

/// `x` rounded to `digits` decimal places (a negative count rounds to tens, hundreds, ...), the
/// tie broken by `mode`. The result is the double nearest to the decimal chosen, as R gives it.
pub fn round(x: f64, digits: i32, mode: RoundingMode) -> f64 {
    if !x.is_finite() || x == 0.0 {
        return x;
    }
    let digits = digits.clamp(-15, 15);
    if digits >= 0 {
        // Rust formats the exact binary value, correctly rounded, ties to even.
        let text = format!("{x:.*}", digits as usize);
        let even: f64 = text.parse().unwrap_or(x);
        if mode == RoundingMode::Away && is_tie(x, digits) && even.abs() < x.abs() {
            return away_from_zero(&text, x.is_sign_negative());
        }
        if even == 0.0 {
            return if x.is_sign_negative() { -0.0 } else { 0.0 };
        }
        even
    } else {
        let unit = 10f64.powi(-digits);
        let scaled = x / unit;
        let rounded = match mode {
            RoundingMode::Even => scaled.round_ties_even(),
            RoundingMode::Away => {
                if is_tie(x, digits) {
                    scaled.round()
                } else {
                    scaled.round_ties_even()
                }
            }
        };
        rounded * unit
    }
}

/// The decimal `text` (already rounded to even, which went toward zero at a tie) moved one unit
/// of its last place away from zero.
fn away_from_zero(text: &str, negative: bool) -> f64 {
    let digits: Vec<u8> = text.bytes().filter(u8::is_ascii_digit).collect();
    let mut incremented = digits.clone();
    let mut i = incremented.len();
    loop {
        if i == 0 {
            incremented.insert(0, b'1');
            break;
        }
        i -= 1;
        if incremented[i] == b'9' {
            incremented[i] = b'0';
        } else {
            incremented[i] += 1;
            break;
        }
    }
    // Put the decimal point back where it was, counted from the right.
    let decimals = text.rsplit_once('.').map_or(0, |(_, frac)| frac.len());
    let mut body = String::from_utf8(incremented).unwrap_or_default();
    if decimals > 0 {
        let point = body.len() - decimals;
        body.insert(point, '.');
    }
    let value: f64 = body.parse().unwrap_or(0.0);
    if negative { -value } else { value }
}

/// `x` rounded to `significant` significant digits (at least one), the tie broken by `mode`.
pub fn round_significant(x: f64, significant: u32, mode: RoundingMode) -> f64 {
    if !x.is_finite() || x == 0.0 {
        return x;
    }
    let significant = significant.max(1) as i32;
    // The decimal exponent of the leading digit, read from the shortest exact form.
    let scientific = format!("{x:e}");
    let exponent: i32 = scientific
        .rsplit_once('e')
        .and_then(|(_, e)| e.parse().ok())
        .unwrap_or(0);
    round(x, significant - 1 - exponent, mode)
}

/// Floored integer division, as Python and R: the quotient rounds toward negative infinity.
/// `None` for a zero divisor (Z0212) or the one overflow, `i64::MIN / -1` (Z0211).
pub fn div_int(a: i64, b: i64) -> Option<i64> {
    if b == 0 {
        return None;
    }
    let q = a.checked_div(b)?;
    if a.checked_rem(b)? != 0 && (a < 0) != (b < 0) {
        q.checked_sub(1)
    } else {
        Some(q)
    }
}

/// The remainder of floored division, with the sign of `b`. `None` for a zero divisor.
pub fn mod_int(a: i64, b: i64) -> Option<i64> {
    if b == 0 {
        return None;
    }
    if b == -1 {
        // Every integer is a multiple of -1; the remainder is 0 even for i64::MIN, where the
        // quotient would overflow.
        return Some(0);
    }
    let r = a.checked_rem(b)?;
    if r != 0 && (r < 0) != (b < 0) {
        r.checked_add(b)
    } else {
        Some(r)
    }
}

/// Floored division of Floats: `floor(a / b)`, following `/` at zero (section 3.7).
pub fn div_float(a: f64, b: f64) -> f64 {
    (a / b).floor()
}

/// The Float remainder with the sign of `b`: `a - b * floor(a / b)`.
pub fn mod_float(a: f64, b: f64) -> f64 {
    let r = a - b * (a / b).floor();
    // A rounding can leave a result at `b` itself; the remainder lies in [0, b) or (b, 0].
    if b.is_finite() && r != 0.0 && (r.abs() >= b.abs() || (r < 0.0) != (b < 0.0)) {
        0.0
    } else {
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounding_decides_on_the_exact_value_and_breaks_ties_as_asked() {
        let even = RoundingMode::Even;
        let away = RoundingMode::Away;
        assert_eq!(round(0.5, 0, even), 0.0);
        assert_eq!(round(1.5, 0, even), 2.0);
        assert_eq!(round(2.5, 0, even), 2.0);
        assert_eq!(round(-2.5, 0, even), -2.0);
        assert_eq!(round(0.5, 0, away), 1.0);
        assert_eq!(round(2.5, 0, away), 3.0);
        assert_eq!(round(-2.5, 0, away), -3.0);
        // 2.675 is stored below the tie, so both modes give 2.67, as R does.
        assert_eq!(round(2.675, 2, even), 2.67);
        assert_eq!(round(2.675, 2, away), 2.67);
        // 0.125 is exact: a true tie at two decimals.
        assert_eq!(round(0.125, 2, even), 0.12);
        assert_eq!(round(0.125, 2, away), 0.13);
        assert_eq!(round(-0.125, 2, away), -0.13);
        assert_eq!(round(1.005, 2, even), 1.0, "1.005 is stored below the tie");
        assert_eq!(round(123.456, 1, even), 123.5);
        assert_eq!(round(123.456, -1, even), 120.0);
        assert_eq!(round(1250.0, -2, even), 1200.0);
        assert_eq!(round(1250.0, -2, away), 1300.0);
        assert_eq!(round(1350.0, -2, even), 1400.0);
        assert_eq!(round(-0.4, 0, even), -0.0);
        assert!(round(-0.4, 0, even).is_sign_negative());
        assert_eq!(round(9.995, 2, even), 9.99, "stored below the tie");
        assert_eq!(round(f64::INFINITY, 2, even), f64::INFINITY);
        assert!(round(f64::NAN, 2, even).is_nan());
        assert_eq!(round(0.0, 3, even), 0.0);
        assert_eq!(round(1e300, 2, even), 1e300);
        assert_eq!(
            round(123456.789, 3, even),
            123456.789,
            "already at three decimals"
        );
        assert_eq!(round(0.995, 2, away), 0.99, "0.995 is stored below the tie");
    }

    #[test]
    fn significant_digits() {
        let even = RoundingMode::Even;
        assert_eq!(round_significant(123456.0, 3, even), 123000.0);
        assert_eq!(round_significant(0.00123456, 3, even), 0.00123);
        assert_eq!(round_significant(1.25, 2, even), 1.2);
        assert_eq!(round_significant(1.25, 2, RoundingMode::Away), 1.3);
        assert_eq!(round_significant(9.99, 1, even), 10.0);
        assert_eq!(round_significant(-987.654, 2, even), -990.0);
        assert_eq!(round_significant(5.0, 0, even), 5.0, "at least one digit");
    }

    #[test]
    fn floored_division_and_remainder_as_python_and_r() {
        assert_eq!(div_int(7, 2), Some(3));
        assert_eq!(div_int(-7, 2), Some(-4));
        assert_eq!(div_int(7, -2), Some(-4));
        assert_eq!(div_int(-7, -2), Some(3));
        assert_eq!(div_int(6, 3), Some(2));
        assert_eq!(div_int(-6, 3), Some(-2));
        assert_eq!(mod_int(7, 2), Some(1));
        assert_eq!(mod_int(-7, 2), Some(1));
        assert_eq!(mod_int(7, -2), Some(-1));
        assert_eq!(mod_int(-7, -2), Some(-1));
        assert_eq!(mod_int(6, 3), Some(0));
        assert_eq!(div_int(1, 0), None);
        assert_eq!(mod_int(1, 0), None);
        assert_eq!(div_int(i64::MIN, -1), None, "the one overflow");
        assert_eq!(mod_int(i64::MIN, -1), Some(0));
        for (a, b) in [
            (7, 2),
            (-7, 2),
            (7, -2),
            (-7, -2),
            (0, 5),
            (5, 5),
            (i64::MAX, 7),
            (i64::MIN, 7),
        ] {
            let (q, r) = (div_int(a, b).unwrap(), mod_int(a, b).unwrap());
            assert_eq!(
                b.wrapping_mul(q).wrapping_add(r),
                a,
                "a = b*q + r for {a}, {b}"
            );
            assert!(
                r == 0 || (r < 0) == (b < 0),
                "the remainder takes the sign of b"
            );
        }
        assert_eq!(div_float(7.5, 2.0), 3.0);
        assert_eq!(div_float(-7.5, 2.0), -4.0);
        assert_eq!(mod_float(7.5, 2.0), 1.5);
        assert_eq!(mod_float(-7.5, 2.0), 0.5);
        assert_eq!(mod_float(7.5, -2.0), -0.5);
        assert_eq!(div_float(1.0, 0.0), f64::INFINITY, "follows `/` at zero");
        assert!(mod_float(1.0, 0.0).is_nan());
    }

    #[test]
    fn the_elementary_functions_give_their_exact_answers() {
        assert_eq!(exp(0.0), 1.0);
        assert_eq!(log(1.0), 0.0);
        assert_eq!(log(0.0), f64::NEG_INFINITY);
        assert!(log(-1.0).is_nan());
        assert_eq!(log2(1024.0), 10.0);
        assert_eq!(log10(1000.0), 3.0);
        assert_eq!(log_base(8.0, 2.0), 3.0);
        assert_eq!(log_base(100.0, 10.0), 2.0);
        assert!((log_base(27.0, 3.0) - 3.0).abs() < 1e-14);
        assert_eq!(log1p(0.0), 0.0);
        assert_eq!(expm1(0.0), 0.0);
        assert_eq!(sin(0.0), 0.0);
        assert_eq!(cos(0.0), 1.0);
        assert_eq!(tan(0.0), 0.0);
        assert_eq!(asin(1.0), std::f64::consts::FRAC_PI_2);
        assert_eq!(acos(1.0), 0.0);
        assert_eq!(atan(0.0), 0.0);
        assert_eq!(atan2(0.0, 1.0), 0.0);
        assert_eq!(atan2(1.0, 0.0), std::f64::consts::FRAC_PI_2);
        assert_eq!(sqrt(144.0), 12.0);
        assert!(sqrt(-1.0).is_nan());
        assert_eq!(sign(-3.5), -1.0);
        assert_eq!(sign(0.0), 0.0);
        assert_eq!(sign(2.0), 1.0);
        assert!(sign(f64::NAN).is_nan());
    }
}
