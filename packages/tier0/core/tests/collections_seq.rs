//! `seq`, `any` and `all` against R (testing handbook, section 1.1; release 0.5.0, step 13):
//! R's `seq(from, to, by)` to the bit over a grid of fractional steps and ends from 1e7 to 1e12,
//! and R's `any` and `all` with `NA`, from `tests/reference/r/collections_seq.R`. The values of
//! `seq` are read back from 17 significant digits as the exact doubles R made.
//!
//! Where Zeff departs from R, the test names it: `seq(5, 1)` is an error here (the catalogue's
//! default step is 1; R counts down by itself) and a zero step is always an error.

use serde_json::Value as Json;
use zeff_core::collections::{SeqError, seq_floats, seq_ints};
use zeff_testkit::{root, run};

fn r_reference() -> Json {
    let path = root().join("tests/reference/r/collections_seq.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap()
}

fn number(value: &Json) -> f64 {
    value.as_str().unwrap().parse().unwrap()
}

#[test]
fn seq_agrees_with_r_to_the_bit() {
    let reference = r_reference();
    let mut checked = 0;
    for case in reference["seq"].as_array().unwrap() {
        let (from, to, by) = (
            number(&case["from"]),
            number(&case["to"]),
            number(&case["by"]),
        );
        let expected: Vec<f64> = case["values"]
            .as_array()
            .unwrap()
            .iter()
            .map(number)
            .collect();
        assert_eq!(expected.len() as u64, case["n"].as_u64().unwrap());
        let found = seq_floats(from, to, by)
            .unwrap_or_else(|e| panic!("seq({from}, {to}, {by}) failed: {e:?}"));
        assert_eq!(
            found.len(),
            expected.len(),
            "seq({from:e}, {to:e}, {by:e}) has {} values; R has {}",
            found.len(),
            expected.len()
        );
        for (i, (f, e)) in found.iter().zip(&expected).enumerate() {
            assert_eq!(
                f.to_bits(),
                e.to_bits(),
                "seq({from:e}, {to:e}, {by:e})[{}]: {f:e} against R's {e:e}",
                i + 1
            );
        }
        checked += 1;
    }
    assert!(checked >= 40, "{checked} cases");
}

#[test]
fn any_and_all_agree_with_r_on_the_truth_table() {
    let reference = r_reference();
    for case in reference["logic"].as_array().unwrap() {
        let items: Vec<&str> = case["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let list = format!("[{}]", items.join(", "));
        // An empty list has no type of its own; it is checked where a parameter types it.
        let out = run(&format!("any({list})\nall({list})\n"));
        assert_eq!(out.exit_code, 0, "{list}\n{}", out.output.stderr);
        let lines: Vec<&str> = out.output.stdout.lines().collect();
        assert_eq!(lines[0], case["any"].as_str().unwrap(), "any({list})");
        assert_eq!(lines[1], case["all"].as_str().unwrap(), "all({list})");
    }
}

#[test]
fn where_zeff_departs_from_r_it_says_so() {
    // R counts down by itself; Zeff's default step is 1, so the same call is an error.
    assert_eq!(seq_floats(5.0, 1.0, 1.0), Err(SeqError::WrongSign));
    assert_eq!(seq_ints(5, 1, 1), Err(SeqError::WrongSign));
    assert_eq!(seq_ints(5, 1, -1), Ok(vec![5, 4, 3, 2, 1]));
    // R returns `from` for seq(1, 1, by = 0); Zeff refuses a zero step whatever the range.
    assert_eq!(seq_floats(1.0, 1.0, 0.0), Err(SeqError::ZeroStep));
    assert_eq!(seq_ints(1, 1, 0), Err(SeqError::ZeroStep));
}
