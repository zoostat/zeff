# Sources of the test data

Every file in this folder, where it comes from and under what terms. A file copied from outside the project is listed with its source and licence before it is committed.

| File | Source | Terms |
| --- | --- | --- |
| `wage1.csv`, `wage2.csv` | Data of J. M. Wooldridge, *Introductory Econometrics: A Modern Approach*, as distributed in the R package `wooldridge` 1.4-7; written by `tests/reference/r/regression.R` | The package is under GPL-3; the data are the textbook's, used here as reference inputs only, never linked into the engine |
| `wagepan.csv` | As above; written by `tests/reference/r/fixed_effects.R` | As above |
| `mroz.csv` | As above: the labour-force data of T. A. Mroz (1987), the eight variables of the textbook's example 17.1; written by `tests/reference/r/binary.R` | As above |
| `card.csv` | The extract of the National Longitudinal Survey of Young Men used by D. Card, "Using geographic variation in college proximity to estimate the return to schooling" (1995), as distributed in the R package `wooldridge` 1.4-7; written by `tests/reference/r/iv.R`, which adds `region`, the region of residence in 1966 as one variable (1 to 9) made from the extract's nine indicators `reg661` to `reg669` | As above |
| `constant.csv`, `offset.csv`, `two.csv` | Made for the tests of the descriptive functions | The project's own (Apache-2.0) |
| `event_panel.csv` | Made for the tests of `event` | The project's own (Apache-2.0) |
| `kernels_cases.csv`, `maths_inputs.csv`, `special_cases.csv`, `tukey_cases.csv` | Inputs chosen for the reference tests of the numeric kernels | The project's own (Apache-2.0) |
| `small.csv`, `wide.csv` | Made for the command-line, engine and conformance tests | The project's own (Apache-2.0) |
