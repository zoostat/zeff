# Sources of the NIST reference problems

The thirty-one files of this folder are the linear-regression, univariate and analysis-of-variance problems of the Statistical Reference Datasets (StRD) of the National Institute of Standards and Technology, Information Technology Laboratory. Each holds its data and the values NIST certifies for them, computed independently of any statistical package, to fifteen significant digits. The conformance suite reads them as they are and records the log relative error of each certified value (`crates/zeff-conformance/src/nist.rs`; open decision #153).

| Files | Source | Terms |
| --- | --- | --- |
| `lls/Norris.dat`, `Pontius.dat`, `NoInt1.dat`, `NoInt2.dat`, `Filip.dat`, `Longley.dat`, `Wampler1.dat` to `Wampler5.dat` | `https://www.itl.nist.gov/div898/strd/lls/data/LINKS/DATA/<name>.dat`, fetched on 23 November 2026 | Works of the United States government, in the public domain |
| `univ/PiDigits.dat`, `Lottery.dat`, `Lew.dat`, `Mavro.dat`, `Michelso.dat`, `NumAcc1.dat` to `NumAcc4.dat` | `https://www.itl.nist.gov/div898/strd/univ/data/<name>.dat`, fetched on 23 November 2026 | As above |
| `anova/SiRstv.dat`, `SmLs01.dat` to `SmLs09.dat`, `AtmWtAg.dat` | `https://www.itl.nist.gov/div898/strd/anova/<name>.dat`, fetched on 23 November 2026 | As above |

The files are byte for byte what the Institute serves, but for their line ends: served with a carriage return and a line feed, kept here with the line feed alone, so that the line numbers each file gives for its certified values and its data are unchanged. Nothing in them is edited; a certified value is never corrected here. The references each problem rests on are named in its own file.
