# Zeff Testing Handbook

Version of this document: 17 October 2026 · Companion to the Zeff Specification (sections 11.1 and 13)

This handbook sets out how Zeff is tested: the kinds of test kept, the routine on each change and each release, how every bug is handled, and what a new function needs before it counts as done. Every tool named is free. It is written for one builder and scales to a team unchanged.

---

## 1. Kinds of test

| Kind | What it catches | Tool | When it runs |
| --- | --- | --- | --- |
| Unit tests | Logic errors in kernel and package code | `cargo test` | Every push |
| Reference tests | Numbers that differ from R or Stata | Stored expected outputs (JSON) produced by scripts in `tests/reference/` | Every push |
| Truth-recovery tests | Estimators that match R but are both wrong; confidence intervals that do not cover | Monte Carlo: simulate data with known parameters; check that bias is near zero and that 95% intervals cover about 95% of the time | Nightly |
| Property tests | Bugs on inputs nobody thought of | `proptest`: scaling the outcome scales the coefficients; rescaling a regressor by 1e6 changes no t-statistic (column equilibration); reordering rows changes nothing; a variable constant within groups is absorbed by the fixed effects; a duplicate regressor is dropped with a note; every verb's output order matches its canonical order | Every push (short runs), nightly (long runs) |
| Edge-case datasets | Crashes and silent errors at the boundaries | Fixed files: all `NA`, one row, zero variance, perfect separation, singleton clusters and fixed-effect groups, exact collinearity, extreme weights, empty groups, Unicode labels and awkward column names; opaque columns (list, record, binary) carried through every row verb, refused by every other with a type error, and round-tripped through Parquet with their metadata (Zeff Specification, section 4.3) | Every push |
| Snapshot tests | Drift in messages, printed output and tables | `insta` | Every push |
| Help tests | A help page that is incomplete, names a parameter the function lacks, lists a diagnostic the code never raises (or omits one it does), or whose example prints something else | The build's help check and the conformance suite's example runs (Zeff Specification, section 9.1) | Every push |
| Script tests | Breakage of the whole language | `zeff run` on the sample script, the specification's examples and the replication projects, compared with stored output | Every push |
| Cross-platform identity | Results that differ between machines | Run the suite on WebAssembly, on two x86-64 machines of different SIMD levels (one without AVX-512) and on ARM64, and compare result hashes bit for bit, the SVG and PDF bytes of the sample figures (Zeff Specification, section 7.6), and the bytes of every Parquet file `save` writes (section 8.7); runs from the first commit of 0.1.0 (section 14) | Every push |
| Storage round trips | A frame that comes back different from Parquet or Arrow IPC, metadata lost on the way, a Parquet file whose bytes differ between targets, or a cached copy that differs from a fresh parse | Every frame of the suite saved and imported again, with its metadata store, panel declaration and opaque columns compared; the cached Arrow copy of every edge-case file compared with a fresh parse; pyarrow and R `arrow` read the Zeff Parquet profile's keys in CI (section 8.7) | Every push |
| Fuzzing | Crashes in the parser, checker and CSV import | `cargo-fuzz` | Nightly |
| Mutation testing | Tests that pass even when the code is wrong | `cargo-mutants` on estimator and variance code | Weekly |
| Coverage | Code no test reaches | `cargo-llvm-cov`; target 80% or more in the kernel and the estimators | Weekly |
| Benchmarks | Slowdowns | `criterion` and the benchmark suite (Zeff Specification, section 8.3, point 10); a case that slows by more than 5% fails | Nightly |
| Bundle and start-up (from 0.1.0) | The browser engine growing past its budget or starting too slowly | The compressed WebAssembly bundle size and the cold-start time of the browser path, against the thresholds of the risk register (R4) and the targets of the Zemp Specification, section 14 | Every push |
| Dialog and code parity (from Zeff 0.8.0) | Zemp dialogs that disagree with typed code | Playwright: build each dialog, run it, compare its result with the typed statement | Nightly |
| Edition builds (from Zemp 0.2) | A shared interface module that depends on a point-and-click module, so that Zemp Lite no longer builds; a project that runs or shows differently in the two editions | The build check of the Zemp Specification, section 3; the script tests run through both builds, and the Lite build is checked to contain no dialog, guide or builder module | Every push |
| Installation and command line (from 0.9.0) | An installer that fails on a clean machine, a launcher that picks the wrong engine, a signature that is not checked, a replication package that does not verify | Clean virtual machines for each channel (script, Homebrew, winget, Scoop, archive, offline bundle, container, each as it ships) on the four targets, and from 0.11.0 `zeff app install lite \| zemp` on each desktop target; `zeff selftest` on every official binary; a `zeff export replication` → `zeff verify` round trip of the replication projects; a source build compared with the official build; a tampered archive refused with Z0711 (Zeff Specification, section 2.1) | Before each release; `selftest` and the round trip every push |

### 1.1 Reference outputs

- **R is the main reference**, because it is free. Each method has an R script in `tests/reference/` that writes its expected values as JSON. Package versions are pinned with `renv.lock`, and the pinned versions are named in the release notes (Zeff Specification, section 13, conformance protocol).
- **Python's `statistics` module is the reference for 0.1.0** (`mean`, `sd`, `describe`, `summary`), because it computes in exact rational arithmetic; it stays as a second reference for the descriptive reducers once the R scripts exist from 0.2.0. Its version is pinned the same way.
- **Stata is not free.** Its expected outputs are generated once, on an institutional licence, and committed as JSON; the build never needs Stata. They are regenerated only when a Zeff release adopts a new reference version.
- **What is compared:** estimates, standard errors, test statistics, p-values, confidence limits, log-likelihoods and observation counts, within the tolerance of the function's accuracy class.

### 1.2 Public datasets with known answers

| Dataset | Used for |
| --- | --- |
| Wooldridge textbook data (`wooldridge` package) | Regression, panel, IV and binary-outcome tests |
| Card (1995) | IV with weak-instrument diagnostics |
| `mpdta` (`did` package) | Difference-in-differences (from 1.2) |
| Political Democracy, Holzinger–Swineford (`lavaan`) | CFA and SEM (from 1.3) |
| The `datasets` package's synthetic files (Zeff Specification, section 8.5) | Truth-recovery tests for every estimator, with the parameters their generating scripts document; the scripts are re-run on every build and must reproduce the shipped files bit for bit |

### 1.3 Infrastructure

GitHub Actions, which is free for public repositories, runs all four targets (the two x86-64 SIMD levels are chosen by runner label, or one of them is a self-hosted runner). WebAssembly runs under Node and in headless Chromium, so both the engine and the browser path are tested. Nightly and weekly jobs run on a schedule; their results are posted to the repository.

---

## 2. The routine

### 2.1 On every change (under 5 minutes)

1. Format, lint and type-check: `cargo fmt --check`, `cargo clippy`, `zeff fmt --check`.
2. Unit, reference, edge-case, snapshot and short property tests.
3. Script tests and the four-target identity check.
4. Merge only when everything is green.

### 2.2 Nightly

Truth-recovery Monte Carlo, long property runs, fuzzing, benchmarks and, from 0.8.0, dialog and code parity.

### 2.3 Weekly

Mutation testing and coverage, with a short note of any surviving mutants or newly uncovered code.

### 2.4 Before each release

1. Freeze features: only fixes from here on.
2. Run the full suite on all four targets, including the weekly mutation and coverage runs.
3. Re-run every replication project and the sample script, and read the printed output by eye.
4. From 0.8.0, run the reader test: give students and referees short scripts and ask what each model does; fix what they misread.
5. Install from every channel on clean machines, run `zeff selftest` on each official binary and `zeff verify` on each replication project's exported package (Zeff Specification, section 2.1).
6. Update the known-issues list and the release notes, naming the reference versions used; tag the release with its major.minor.patch number and its name (Zeff Specification, section 14).

---

## 3. Handling each bug

1. **Reproduce** the report, and shrink it with `zeff report-bug` (from 0.8.0; by hand before then).
2. **Classify** it:

| Severity | Meaning | Target |
| --- | --- | --- |
| S1 | Wrong result returned without an error | Fix first; a patch release (from 0.9.0; the next minor release before then); add to the known-issues list |
| S2 | Crash or internal error (Z9xxx) | Fix in the next patch release (from 0.9.0; the next minor release before then) |
| S3 | Wrong or misleading message | Next release |
| S4 | Slow | Next release, if it breaks a benchmark target |
| S5 | Cosmetic | When convenient |

3. **Write the failing test first**, from the shrunk reproducer, and see it fail.
4. **Fix** the bug and confirm the test passes on all four targets.
5. **Look for siblings**: the same mistake in similar code, such as another estimator using the same variance layer.
6. **Release**: for an S1 bug, publish a patch release (the third version number, as in 0.9.1; Zeff Specification, section 14) from 0.9.0, when the launcher keeps engines side by side, and ship the fix in the next minor release before then; add the bug to the known-issues list with the affected versions, functions and options, so `zeff audit` (also from 0.9.0) can flag affected runs, and describe it in the release notes.

---

## 4. Definition of done for a new function

- [ ] Matches its reference within its accuracy class on at least two datasets
- [ ] Passes the edge-case datasets, with the expected value, warning or error defined for each
- [ ] Has at least one property test, and a truth-recovery test if it is an estimator
- [ ] Has self-checks on its result, if it fits a model (Zeff Specification, section 11.1)
- [ ] Has snapshot tests for its printed output and every message it can give
- [ ] Gives identical results on WebAssembly, both x86-64 SIMD levels and ARM64
- [ ] Has the complete help page of the help contract (Zeff Specification, section 9.1): summary, description, every argument, value, details and references if it is a statistical method, see also, and at least one example that runs and prints what the page says; the build refuses the function otherwise
- [ ] Has a benchmark, if it is in the performance table
- [ ] Has its row in zeff_functions.csv updated: release, status, accuracy class, and at least one research-goal purpose tag beyond `general` if it is a statistical method (Zemp Specification, section 11)

---

## 5. Budget

Testing takes 30–40% of development time. For a statistics engine this is not overhead: it is what makes the numbers worth trusting.
