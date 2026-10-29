# Zeff and Zemp: Risk Register

Status: 30 October 2026 (reviewed at the close of 0.4.0). Companion to the Engine and Project Specification (`zeff_engine_specification.md`), the Zemp Specification (`zemp_specification.md`) and the testing handbook (`zeff_testing.md`). The register is reviewed at every minor release: each risk's rating is updated, retired risks are kept with the release that closed them, and a new risk is added the first time its early warning is seen.

Likelihood and impact are rated High, Medium or Low. The owner is the project lead until the project has more maintainers.

## Summary

| ID | Risk | Likelihood | Impact | First test |
| --- | --- | --- | --- | --- |
| R1 | Scope outruns the time available | Low | High | 0.1.0 to 0.3.0 cadence |
| R2 | Bit-identical results across targets prove hard to keep | Low | Medium | 0.1.0 CI |
| R3 | Matching R and Stata on edge cases takes far longer than the easy cases | Medium | Medium | 0.4.0 |
| R4 | Browser build limits: memory, download size, threads, C code | Low | High | 0.1.0 bundle size; 1.1 ReadStat |
| R5 | Language tooling (checker, messages, language server) is underestimated | Medium | High | 0.5.0 to 0.7.0 |
| R6 | Dependency churn, above all `arrow-rs` | Medium | Medium | Every upgrade |
| R7 | Rust learning curve and slow builds | Low | Medium | 0.1.0 to 0.2.0 |
| R8 | Byte-identical plot export (SVG, PDF, fonts) | Medium | Low | 0.7.0 |
| R9 | CI and reference costs (ARM runners, second x86 machine, Stata licence) | Low | Low | 0.1.0 CI |
| R10 | Adoption: researchers stay with R and Stata | High | High | Zemp Lite preview, 0.8.0 |
| R11 | Momentum: a long solo build with no users | Medium | High | Any release that slips twice |
| R12 | The specification grows faster than the code | Low | Medium | Now |

## Risks

### R1 · Scope outruns the time available

257 functions for 1.0 over eleven minor releases at six to eight weeks each, and a 505-function roadmap, assume more than one experienced Rust developer. Alongside teaching and research the schedule may stretch two to three times.

- **Early warning:** 0.1.0 or 0.2.0 takes more than twice its planned cadence.
- **Mitigation:** ship thin vertical slices (each release runs one more real analysis end to end); move functions between releases rather than delaying a release; treat the 1.0 function list as the ceiling, not the floor, and cut to the seven hard engine pieces first if needed (section 14 of the Engine and Project Specification).
- **Fallback:** a smaller 1.0 (regression with `fe:`, IV, logit and probit, the toolkit) with the rest moved to 1.x.

### R2 · Bit-identical results across targets

The determinism contract (Engine and Project Specification, section 8.3, point 3) needs Zeff's own elementary functions and linear algebra, a fixed reduction order, and no fused or reordered floating-point operations on WebAssembly, x86-64 and ARM64. Breakages are subtle and appear late if not watched.

- **Early warning:** any cross-target hash mismatch in CI.
- **Mitigation:** run the four-target identity check from the first commit while the code is small (it is in the 0.1.0 row of section 14); forbid FMA and fast-math in the build profile; keep accuracy (agreement with R and Stata within the tolerance of a function's class) separate from determinism (identical bits across machines), so a determinism slip is a tracked bug, not a release blocker for a correct result.
- **Fallback:** for a function whose bit-identity cannot be kept, publish it as "identical within 1 ulp" in its accuracy line until fixed.

### R3 · Matching R and Stata on edge cases

Easy data matches quickly; collinearity, singleton fixed effects, perfect separation, clustered-SE degrees of freedom and cases where R and Stata disagree take weeks.

- **Early warning:** a model release's reference tests are still failing two weeks before its planned date.
- **Mitigation:** the accuracy classes and edge-case datasets of the testing handbook; a fixed time allowance for edge cases in every model release; when references disagree, follow the documented definition and record which reference matches (section 13); reference grids at ordinary parameters miss faults at extreme ones (seen at 0.3.0), so every reference file carries parameters from 1e7 to 1e12 beside its working grid, and a fuzz target feeds extreme parameters through the engine.

### R4 · Browser build limits

32-bit WebAssembly caps memory at 4 GB, cannot memory-map files, needs cross-origin isolation headers for threads, and is harder to debug. Arrow and Parquet enlarge the download, which threatens the start-up targets (Zemp Lite web 2 s, Zemp web 3 s). ReadStat (SPSS and SAS) is C and must be compiled for the browser.

- **Early warning:** the compressed engine bundle passes 5 MB before 0.7.0; a cold start misses its target in CI.
- **Mitigation:** track bundle size and start time in CI from 0.1.0 (in the 0.1.0 row of section 14); build Arrow and Parquet with only the features used; load packages on first use (rule 7 of section 8.1); prove the C-to-WebAssembly path for ReadStat with a small spike before 1.1 depends on it.

### R5 · Language tooling is underestimated

The parser, typed IR, checker, readable diagnostics with fixes, formatter and language server are each sizeable; users judge a language by its error messages.

- **Early warning:** diagnostics in 0.5.0 are written ad hoc rather than from the diagnostics-as-data design.
- **Mitigation:** diagnostics as data from 0.1.0 (already in the 0.1.0 row); snapshot tests of every message (`insta`); a language server built on the same IR as the checker, never a second parser.

### R6 · Dependency churn

`arrow-rs` and `parquet` ship breaking releases often; other crates change less but can still break builds or results.

- **Early warning:** an upgrade changes a result hash or needs more than a day of work.
- **Mitigation:** pin exact versions in `Cargo.lock`; upgrade deliberately, one dependency at a time, with the full suite and the cross-target check; keep Arrow behind a thin internal layer so most crates never touch its API directly.

### R7 · Rust learning curve and slow builds

The borrow checker, trait design and long compile times slow the first months.

- **Early warning:** a clean build of the workspace passes ten minutes, or routine changes take days.
- **Mitigation:** the tiered workspace keeps crates small so incremental builds stay fast; `sccache` or the CI cache for builds; simple ownership designs first (clone, then optimise where benchmarks show a need).

### R8 · Byte-identical plot export

SVG and PDF that are identical byte for byte on every machine need embedded fonts, fixed number formatting and a renderer of Zeff's own (Language Reference, section 7.6).

- **Early warning:** sample-figure hashes differ across targets in 0.7.0 CI.
- **Mitigation:** compare bytes from the first plot; fall back to comparing rendered pixels within a tolerance for PDF if byte identity proves impractical, and say so in the specification.

### R9 · CI and reference costs

Bit-identity CI needs ARM64 runners and a second x86-64 machine of a different SIMD level from 0.1.0; Stata reference outputs need a licence once.

- **Early warning:** CI minutes or runner availability limit how often the full suite runs.
- **Mitigation:** free GitHub Actions minutes for a public repository; one self-hosted runner for the second x86 level; Stata outputs generated once and committed as JSON (testing handbook, section 1.1).

### R10 · Adoption

Researchers rarely leave R or Stata; third-party packages arrive only at 1.11.

- **Early warning:** the Zemp Lite web preview (with Zemp 0.2, Zeff 0.8.0) draws few repeat users.
- **Mitigation:** lead with what R and Stata lack (causal-graph checks, bit-identical replication, `zeff verify`, a free app); the reader test of Zeff syntax at the 0.2 alpha; interoperability through Parquet so Zeff can sit beside R and Stata rather than replace them.

### R11 · Momentum

A long solo build with no public users until the Zemp Lite web preview at 0.8.0 is where projects often stall.

- **Early warning:** two releases in a row slip by more than a cadence.
- **Mitigation:** make 0.1.0 work end to end soon; show each release to a few colleagues; the Zemp Lite web preview as the first public milestone; publish progress openly once the repository is public.

### R12 · The specification grows faster than the code

The two specification volumes, catalogue and companion documents changed many times on 16 and 17 October 2026; a large specification becomes a second project to maintain.

- **Early warning:** more specification edits than commits in a month once coding starts.
- **Mitigation (recommended; open decision #40 of `zeff_open_decisions.md`, to be taken at 0.1.0):** freeze the specification for 0.1.0 to 0.3.0 except for bugs found while coding; let the code push back on the specification; record further design work as open decisions with a deadline release rather than editing the normative text.

## The first test of the register

0.1.0 (ten functions, the tiered workspace, four-target CI, the help build-check and the first five commands) tests R1, R2, R4, R6 and R7 within weeks. Its review is the register's first scheduled update.

## Review at 0.1.0 (18 October 2026)

The release's 25 steps were built and tagged on 18 October 2026, within the cadence, so R1's early warning was not seen; its rating stands until 0.2.0 and 0.3.0 give more points. R2: the four-target identity check ran from the first commit and no run showed a cross-target difference, the kernel's elementary paths are pure Rust and the build profile forbids fused operations and fast maths; likelihood lowered to Medium. R4: the compressed engine bundle is 355 kB against the 5 MB warning level and the cold start 21 ms against 1,000 ms, measured on every build; likelihood lowered to Low (the impact stays High: threads and ReadStat are untested). R6: no upgrade yet; Arrow is pinned at 60.0.0 behind the column store, Parquet is not linked; unchanged. R7: a clean release build takes 23 s and the tiered workspace keeps incremental builds short; likelihood lowered to Low. R9: the free GitHub-hosted runners covered all four targets, ARM64 included; likelihood and impact lowered to Low. R12: the freeze of open decision #40 held, with two corrections logged in `docs/spec_changes.md` and five design questions recorded as open decisions rather than edits; likelihood lowered to Medium. No early warning of any risk was seen and no new risk is added.

## Review at 0.2.0 (19 October 2026)

The release's 21 steps were built and tagged on 19 October 2026, the second release in a row within its cadence, so R1's early warning (a release taking more than twice its planned time) was not seen at either of the two releases it names; likelihood lowered to Medium, the impact unchanged, since the model releases from 0.4.0 are the heavier ones. R3: the R harness of this release matched the descriptive statistics within 1e-12 and the nine quantile definitions to the bit, and the edge cases that cost time were platform ones (R's `%%` on Floats and the elementary functions differing between macOS and Linux, met with tolerances and hex-float inputs), not statistical; the risk is first tested by the model releases, so the rating stands. R4: the compressed bundle grew from 355 kB to 1.10 MB because the two datasets are embedded, against the 5 MB warning level, and the cold start from 21 ms to 132 ms against 1,000 ms; rating unchanged, with a note that each further embedded dataset costs its compressed size and that the budget is to be checked before any dataset is added. R5: 69 codes, each from the diagnostics-as-data path with a snapshot and an explained page; `--format json` carries them to an editor; the language server is still to come, so the rating stands. R6: no upgrade in the release; 15 crates added (126 to 141 in the lock file), each pinned and recorded with its licence, none changing a result; unchanged. R7: a clean release build takes 60 s (23 s at 0.1.0) against the ten-minute warning, with the engine six times the functions; unchanged at Low. R12: the freeze of #40 held, with one normative correction and one catalogue correction logged in `docs/spec_changes.md` and one design question (#46) recorded as an open decision; unchanged. No early warning of any risk was seen and no new risk is added.

## Review at 0.3.0 (22 October 2026)

The release's 15 steps were built between 19 and 22 October 2026 and tagged on 22 October, the third release in a row within its cadence, so R1's early warning was not seen; rating unchanged, since the model releases from 0.4.0 are the heavier ones. R2: the identity check of 81 cases (15 new, among them the special functions, the raw random stream and draws) showed no cross-target difference on any push, and the nightly stochastic figures are identical to the last digit on macOS and Linux; the rating stays at Medium until the reduction shape and the blocked kernels of open decisions #47 and #48, which change the order of every sum, have been built and proven at 0.4.0. R3: the R harness matched every printed value of 35 parameter sets and four composites at six digits and 22 correlation pairs within 1e-12, and the kernel holds the exact decimal reference within 1e-12; but a review before the tag found two wrong results at parameters of 1e7 and above that the reference grids at ordinary parameters could not see (`docs/decisions.md`, 22 October 2026), so the mitigation gains the rule that every reference file carries parameters from 1e7 to 1e12 and a fuzz target for extreme parameters; the rating stands, the risk being first tested by the model releases. R4: the compressed bundle is 1,151 kB (1,101 kB at 0.2.0) against the 5 MB warning level and the cold start 140 ms (132 ms) against 1,000 ms; unchanged. R5: 71 codes, each from the diagnostics-as-data path with a snapshot and an explained page, the four provisional codes settled by #46; unchanged. R6: no third-party crate added or upgraded in the release (the lock file grew by the two new workspace packages); unchanged. R7: a clean release build takes 62 s (60 s at 0.2.0) against the ten-minute warning; unchanged at Low. R9: the first early warning of this risk was seen: the weekly mutation run of the numeric kernel (1,879 mutants at this release, each running the workspace's tests) exceeded a runner's three-hour limit and was cancelled once, and each of the four shards that replaced it ran for more than three hours; the job now runs twelve shards. The repository is public, so the minutes are free, and the limit is the number of concurrent runners; the rating stays at Low, to be raised if a shard exceeds its limit again. R12: the freeze of #40 held through the release (one catalogue correction, 23 roadmap rows and the table of numbered codes in section 11, each logged in `docs/spec_changes.md`; seven design questions recorded as open decisions #47 to #53) and ended at its close, as the decision said; from 0.4.0 every change to the normative text is a logged design decision, and the next review tests whether the edits stay fewer than the commits. No new risk is added.

## Review at 0.4.0 (30 October 2026)

The release's 27 steps (22 and the six sub-steps of the pre-release routine) were built between 23 and 30 October 2026 and tagged on 30 October, the fourth release within its cadence and the first model release, which the earlier reviews had named as the heavier kind; R1's early warning was not seen at any of the four, so likelihood lowered to Low, the impact unchanged. R2: the identity check of 107 cases (26 new, among them the kernel's reductions and factorisations, the model matrices and the regression fits) showed no cross-target difference on any push; the reduction shape of #47 and the blocked kernels of #48, which change the order of every sum and which the 0.3.0 review held the rating at Medium for, are built and proven three ways (identical digests on the five targets, the nightly stochastic figures identical to 0.3.0's to the last digit, the mutation run's outcomes identical mutant by mutant between the x86-64 runner and the ARM64 development machine); likelihood lowered to Low. R3, at its first test: the R harness matched 36 regression fits and the inference block within 1e-10 and the model matrices to the bit, and the edge cases did cost, as the risk says: the formula fuzz target found thirteen engine faults during the release, each fixed with a test within it, none a wrong result on ordinary data; the early warning (reference tests still failing two weeks before the planned date) was not seen, and the release took a week; likelihood lowered to Medium, with the mitigation's extreme-parameter rule and the fuzz targets now the working defence. R4: the compressed bundle is 1,337 kB (1,151 kB at 0.3.0) against the 5 MB warning level, and the cold start 639 ms (140 ms) against 1,000 ms, of which the module's load is 17 ms and the first call 620 ms; the rating is unchanged, with a watch item: the first call's rise is to be understood before the next release adds to the bundle, since another such rise would reach the warning level. R5: 94 codes, each from the diagnostics-as-data path with a snapshot and an explained page; unchanged. R6: no third-party crate upgraded in the release; `criterion` 0.8.2 joined as a development dependency of the kernel and the benchmark tool, with 25 crates none of which is linked into the engine; `arrow` stays at 60.0.0; unchanged. R7: a clean release build of the engine takes 67 s (62 s for the workspace at 0.3.0) against the ten-minute warning; unchanged at Low. R9: the weekly mutation run, now 24 shards over four crates, spent 2,994 runner-minutes in 3 hours 45 minutes of wall time, every shard within its 340-minute limit (the longest 224 minutes), and the free GitHub-hosted runners carried every job of the release, ARM64 included; unchanged at Low, the shard limit still the figure to watch. R12: the first review since the freeze of #40 ended: 19 logged changes to the normative text against 62 commits in the release, each a correction found while coding or a code's row, and eleven design questions recorded as open decisions (#54 to #65) rather than edits; the edits stayed fewer than the commits, as the last review asked, so likelihood lowered to Low. No new risk is added.
