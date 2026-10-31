# Zeff and Zemp: Open Decisions, Options and Recommendations

Date: 17 October 2026 · Source: Zeff Specification section 15, Zemp Specification section 17, the Zemp business model (zemp_business_model.md) and the risk register (zeff_risk_register.md) · The decisions are yours; each entry gives the options, what each costs and buys, a recommendation, and the release by which the choice has to be made.

**Status (16 October 2026, 19:54).** The "now" group was accepted and applied: #11 (the specification is published as two volumes, `zeff_language_reference.md` and `zeff_engine_specification.md`, generated from the one source; the source was removed on 17 October, and the volumes, now `zeff_language_reference.md` and `zeff_engine_specification.md`, are edited directly), #12–#14 and #22 (`posthoc` with factorial `anova(type:)` and `dist.tukey`, `iterate`, `event`, `iv.check` adopted; 1.0 is 256 functions, 257 since `g.plot()` joined on 17 October), #15–#16 (power functions and `survey()` deferred to 1.9), #19 (`factorial_design`, `fractional_design`, `response_surface`, `repeated_anova`, `tost` added as 1.9 rows), #21 (Apache 2.0 decided). The remaining 21 decisions stay open with their deadlines. Added 16 October 2026, 20:55: the command line and installation (zeff_cli_install.md) were accepted and applied with their recommendations (launcher plus engines on demand, the installer script with package managers listed first, the append-only console, baseline x86-64 builds, PyPI and conda after 1.0); one new decision, #31 (names), is open, so 22 remain. Added 17 October 2026: storage formats were decided (Engine and Project Specification, section 8.7: Arrow IPC for the caches, deterministic Parquet for kept files, the Zeff Parquet profile, BLAKE3 hashes), and one decision they leave open, #32 (a file of several frames and results), so 23 remain.

**Status (17 October 2026).** The second audit (zeff_audit_2026-10-17b.md) was applied: the release once planned as 2.0 is 1.11, so #1, #8, #10 and #29 now say 1.11; the interface revamp's five decisions, already in Zemp section 17, are #33; the business model's conflicts with the Zemp Specification are recorded here rather than settled, as options under #25 (open core), #27 (a Zemp server) and #28 (metered AI), and as #34 to #37 (long-term support, pricing and quotas, student verification, trademarks). 28 decisions are open; #4 closes at 0.5.0 if its recommendation stands.

**Status (17 October 2026, third audit).** The third audit (zeff_audit_2026-10-17c.md) was applied: #33 now records the dark-theme decision of 17 October and keeps its other four questions; #23 notes that the command line and `zeff app install` need no operating-system signing, so its deadline is tied to the setup-file decision; #8's option A reads 1.11, not 2.0; and three decisions were added from the Zemp Specification's section 17 and the risk register: #38 (setup files), #39 (Zemp Lite's paid features, web address and desktop preview) and #40 (a specification freeze while 0.1.0 to 0.3.0 are built). 31 decisions are open. Next deadlines: #40 at 0.1.0; #30 by 0.4.0; #4, #5 and #17 by 0.5.0.

**Status (18 October 2026, at the close of release 0.1.0).** The owner took the five decisions due at this release, each as recommended: #40 the specification stays frozen until 0.3.0, changed only to correct errors found while coding; #41 Z0301, Z0303 and Z0304; #42 Z0712 to Z0715, an unknown key in `zeff.toml` being an error; #43 Z0103, Z0105, W0101 and Z0401 to Z0409; #44 W0201 and W0204, with the specification's W0205 used for an undefined standard deviation. The codes are written into section 11 of the Language Reference when the freeze ends (0.3.0). #45 is due at 0.2.0.

**Status (18 October 2026, release 0.2.0, step 8).** The owner took #45 as recommended: a warning printed under a result shows no code until the pragma of 0.7.0; the code is carried in `--format json` and in the explained-code pages.

**Status (19 October 2026, at the close of release 0.2.0).** No decision fell due at this release: #45 was taken at step 8. #46 (the four provisional codes Z0221, Z0222, Z0410 and W0102, raised during 0.2.0) is due at 0.3.0, and the freeze of #40 holds until 0.3.0.

**Status (22 October 2026, at the close of release 0.3.0).** The owner took #46 as recommended (Z0221, Z0222, Z0410 and W0102) and ended the freeze of #40. The codes of #41 to #44 and #46, with Z0413 and Z0111, are in section 11 of the Language Reference. Decisions #47 to #53 are due by 0.4.0.

**Status (23 October 2026, release 0.4.0 started).** The owner said "go" for release 0.4.0. The summary table gains the rows of #47 to #53, whose deadlines are those of their entries (#47 to #50 and #52 at 0.4.0, with #30; #51 at 0.6.0; #53 at 1.1). The decisions due are put to the owner at the top of `docs/releases/0.4.0.md` with the register's recommendations, and each answer is recorded here when it is given.

**Status (23 October 2026, release 0.4.0, after step 1).** The owner took #30, #47, #48, #49, #50 and #52, each as recommended; #51 and #53 wait for their own deadlines (0.6.0 and 1.1). Each entry records its decision.

**Status (23 October 2026, release 0.4.0, step 7).** #54 raised while formulas were built: `..` binds more loosely than `+` and does not chain, so one formula holds one bare range and a second is written in brackets; the engine reads `(z1..z3)` as the range, and the question is whether the grammar should change. Due at 0.6.0.

**Status (28 October 2026, a review of requested capabilities).** The owner asked for three families of capability for later releases: psychometric and other latent-variable models, Bayesian models written in the manner of Stan and BUGS, and machine learning from stacking to deep and reinforcement learning. Each was checked against both volumes and the catalogue, and what remains to decide is recorded as #55 to #66. The owner took #55 the same day: results stay bit-identical on every machine, and large or pretrained models reach a result only by routes that keep that promise. The rest are open with their deadlines: #59 and #64 by 1.0.0; #61, #65 and #66 by 1.1; #56, #57 and #58 (in part) by 1.3; #62 by 1.5; #60 by 1.8; #63 and the rest of #58 by 1.11. Section 14.1 of the Engine and Project Specification lists the new goals.

**Status (30 October 2026, at the close of release 0.4.0).** The six decisions due at 0.4.0 (#30, #47 to #50 and #52) were taken on 23 October and are built; none remains due. Due by 0.5.0: #4 (variadic parameters), #5 (structural equality), #13 (`iterate`) and #17 (panels at 0.6.0 before tests and plots at 0.7.0), to be put to the owner when release 0.5.0 opens.

**Status (30 October 2026, at the opening of release 0.5.0).** The four decisions due by 0.5.0 (#4, #5, #13 and #17) were taken on 30 October 2026, each as recommended: no variadic parameters in `fn`; `equal(a, b)` added to `core` (catalogue row 529); `iterate` adopted with the catalogue's default `max: Int = 1000`, "required" read as required to be finite; panels at 0.6.0 before tests and plots at 0.7.0. None is due before 0.6.0, where #14, #51 and #54 fall due.

## Summary

| # | Decision | Recommendation | Decide by |
| --- | --- | --- | --- |
| 1 | User-defined types | A `type` declaration (named record + interfaces) at 1.11; interfaces fixed now | 1.3 |
| 2 | Multivariate outcomes | A list of formulas, `sur([f1, f2])`, no grammar change | after 1.0 |
| 3 | Fixing a parameter in a model block | `@`, as Mplus; reserve the token now | 1.3 |
| 4 | Variadic parameters | No: lists instead; variadics stay built-in only | decided 30 October 2026 |
| 5 | Structural equality | Add `equal(a, b)` to `core` at 0.5.0 | decided 30 October 2026 |
| 6 | Locale collation, time zones | Opt-in `collation:` with a pinned CLDR version; `to_timezone` with a pinned tz database; after 1.0 | 1.x |
| 7 | Grapheme-aware text | `unit: graphemes` argument; default stays code points | 1.x |
| 8 | Zeff-source packages before 1.11 | Yes, in 1.6 with the manifest and lockfile; components at 1.11 | 1.3 |
| 9 | Registry hosting | GitHub Pages and Releases with a signed index, mirrored on Codeberg | 1.5 |
| 10 | `synthesize` | A package through the registry, not the engine | after 1.11 |
| 11 | Split the specification | Yes, now, before coding starts | now |
| 12 | `posthoc` and factorial `anova(type:)` | Adopt, 0.7.0 | 0.7.0 |
| 13 | `iterate` | Adopt, 0.5.0 | decided 30 October 2026 |
| 14 | `event()` | Adopt, 0.6.0 | 0.6.0 |
| 15 | Power functions | Defer to 1.9 as one Design release | 0.7.0 |
| 16 | `survey()` in 1.0 | Defer to 1.9 | 0.7.0 |
| 17 | Panels (0.6.0) before tests and plots (0.7.0) | Keep the order | decided 30 October 2026 |
| 18 | Swap 1.2 DiD and 1.3 SEM | Decide at the 0.8.0 alpha from what its users ask for; DiD first otherwise | 0.8.0 |
| 19 | Design methods for 1.9 | Add designed experiments, repeated-measures ANOVA and TOST to the catalogue now | now |
| 20 | Bayesian estimation | `prior:` arguments taking `Distribution` values; `reg`, `logistic`, `mixed` first | 1.9 |
| 21 | Licence | Apache 2.0 (decided 16 October 2026) | decided |
| 22 | `iv.check` | Adopt, 0.10.0 | 0.10.0 |
| 23 | Code signing (for setup files of the Zemp desktop apps only; the command line and `zeff app install` need none) | Free Windows signing for open source plus the Apple Developer Program | 0.11.0, with #38 |
| 24 | Zemp interface framework | Svelte 5, after a two-week spike against Solid at 0.6.0 | 0.7.0 |
| 25 | Zemp licence | Apache 2.0, as Zeff; a commercial path through services, not the licence. The business model assumes open core instead: decide the two together | Zemp 0.2 alpha |
| 26 | Start Zemp at 0.7.0 | Keep; the framework spike starts at 0.6.0 | 0.6.0 |
| 27 | Hosting Zemp web; sync; a Zemp server | A static host that sets cross-origin isolation headers; sync through the user's own folder sync, no server. The business model's paid sync, server compute and self-hosting need a server and a change to Zemp section 13: decide the two together | Zemp 0.2 alpha |
| 28 | AI assistance model | User's own key to a remote model, a local model as the private option, both off by default; whether Zemp also sells metered access of its own (the business model's Pro tier) is part of the same decision | Zemp 1.11 |
| 29 | Zemp version coupling | Follow Zeff's minors until 1.11 | 1.0 |
| 30 | Performance targets | Put the two stress cases in the benchmark suite now; set targets at first measurement, tighten each release | decided 23 October 2026 |
| 31 | Names: domain and GitHub organisation | `zeff-lang.org` with the `zeff-lang` organisation; the installer URL and signing-key list depend on it | before 0.9.0 |
| 32 | A file of several frames and fitted results | A zip of Arrow files and JSON result descriptions, at 1.5 with the heavy models; never a binary format of Zeff's own | 1.5 |
| 33 | Interface revamp: density, icons, desktop menus, tablet and phone (the dark theme was decided on 17 October 2026) | As zemp_gui_revamp.md section 4 recommends for the four: one density with a larger-type setting; no icons; native menus mirroring the palette; no tablet or phone in 1.0 | Zemp 0.2 alpha |
| 34 | Long-term-support releases for institutions | None: exact pins and engines downloadable for ever already keep results stable; sell support, not a release line | Zemp Institution tier |
| 35 | Pricing levels and AI usage quotas | Set from the first year of running costs, not before Pro exists | Zemp Pro |
| 36 | Student verification for free Pro | Institutional email, with a manual route for institutions without one | Zemp Pro |
| 37 | Trademark filing for Zeff and Zemp | File in the home jurisdiction before the public alpha; Apache 2.0 gives others no right to the names | Zemp 0.2 alpha |
| 38 | Setup files for Zeff, Zemp Lite and Zemp | None until #23 is decided: `zeff app install` and the package managers cover the desktop apps; plan setup files with #23 | 0.11.0 |
| 39 | Zemp Lite: paid features and AI in Lite; the web address; a desktop preview before 0.5 | Free-tier only through 1.0, revisited with #27 and #28; Lite web at a path on Zemp web's origin; no earlier desktop preview | Zemp 0.2 alpha (the address); the rest with #27 and #28 |
| 40 | A specification freeze while 0.1.0 to 0.3.0 are built | Freeze the normative text except for bugs found while coding; record further design work here with a deadline release (risk register, R12) | decided 18 October 2026; ended 22 October 2026 |
| 41 | Diagnostic codes for three syntax conditions the specification does not number | Z0301 something the grammar does not allow at that place (an unexpected character, an unclosed quoted name, an unexpected token); Z0303 an unmatched closing bracket; Z0304 a construct the grammar has but this release does not run | decided 18 October 2026 |
| 42 | Diagnostic codes for two conditions of the project file | Z0712 `zeff.toml` is not valid TOML; Z0713 `zeff.toml` has an unknown key, a missing key or a value of the wrong kind | decided 18 October 2026 |
| 43 | Diagnostic codes for the checker's own messages | Z0103 no such function; Z0105 a `let` reuses a `core` name; W0101 a `let` hides a package's function; Z0401 an argument of the wrong type; Z0404 unknown argument; Z0405 an argument given twice; Z0406 a required argument missing; Z0407 too many arguments; Z0408 a positional argument after a named one; Z0409 a bad date format | decided 18 October 2026 |
| 44 | Diagnostic codes for the descriptive reducers' warnings | W0201 missing values skipped, with the count; W0204 infinite values of both signs, so the result is NA | decided 18 October 2026 |
| 45 | Whether a warning printed under a result shows its code | Follow section 10.2 for now: `warning: 211 rows ... were skipped (line 21)` without the code; the run record keeps it | decided 18 October 2026 |
| 46 | Diagnostic codes for four conditions of expressions and calls | Z0221 an operator applied to types it does not take (a Bool in arithmetic, `!` on a number, `&&` on numbers, a comparison across types, an index of the wrong kind); Z0222 a mapping key written or computed twice; Z0410 two options of one call that exclude each other (`round` with `digits:` and `sig:`); W0102 a name in a frame method that is both a variable and a binding | decided 22 October 2026 |
| 47 | A fixed reduction shape for every sum | B: N independent accumulators combined in a fixed order, N fixed by a micro-benchmark and recorded once | decided 23 October 2026 |
| 48 | Blocked kernels from the start, threads when proven | B: every kernel blocked from now and run one block after another; threads at 1.1; the browser single-threaded until proven | decided 23 October 2026 (the shape; threads at 1.1) |
| 49 | When fused passes arrive | B: the block and selection interface now, fusion itself at 0.6.0 | decided 23 October 2026 |
| 50 | Compact column storage | B: each column in its narrowest honest form, Floats always 64-bit | decided 23 October 2026 |
| 51 | Reading files in parallel by chunk | B: chunked parsing with boundaries fixed by the bytes alone, in the reader rewrite of 0.11.0 at the latest | 0.6.0 |
| 52 | A benchmark suite run at every release | B: a small suite from 0.4.0 as a new section of the testing handbook, reporting rather than failing | decided 23 October 2026 |
| 53 | Options rather than functions: HAC standard errors and weights on the descriptives | The `se:` variants with the panel models of 1.4; `weights:` on the descriptives at 1.1 as frequency weights | 1.1 |
| 54 | A second range of variables in one formula | A: the grammar stands; a second range is written in brackets, `(z1..z3)`, which the checker reads as the range | 0.6.0 |
| 55 | Large and pretrained models under the determinism contract | B: results stay bit-identical; such models reach a result only through a package under the contract, a frozen file whose hash the run records, or `external`; no second accuracy class | decided 28 October 2026 (the wording by 1.0.0, the build by 1.11) |
| 56 | Constraints and groups in model blocks | A: a repeated label means equal; `@` fixes (#3); `==` lines between labelled parameters; `group:` with one label per group written as a list | 1.3 |
| 57 | Latent functions brought into 1.3, 1.6 and 1.7 | A: 1.3 gains `invariance`, `latent_growth`, `factor_scores` and four validity measures; 1.6 the EFA family; 1.7 `lpa`, covariates and distal outcomes, mixture regression | 1.3 |
| 58 | Latent-variable releases after 1.11 | Multilevel latent models, latent interactions, item response, mixtures over time, composite models, in that order; the 1.3 and 1.4 likelihoods written so that groups and levels can follow | 1.3 (what stays open); 1.11 (the order) |
| 59 | Bayesian models of the user's own, in the manner of Stan and BUGS | No new reserved word: `model { }` lines with a distribution on the right of `~`, or a log-density written as a function | 1.0.0 (the reserved word); 1.10 (the design) |
| 60 | Reverse-mode differentiation | Reverse mode beside forward mode at 1.8, every sum in a fixed order, so gradients of many parameters cost one pass | 1.8 |
| 61 | Results that carry draws | A result may hold bootstrap replicates or posterior draws, and every derived quantity follows the result's own kind of uncertainty | 1.1 |
| 62 | Prediction after 1.5: stacking and conformal prediction | `spec.stack` and `conformal` as roadmap rows once named; `tune` and `folds` keep out-of-fold predictions from 1.5 | 1.5 |
| 63 | Neural networks, pretrained models and sequential decisions | Feed-forward networks built in; larger networks and pretrained weights through packages under #55; Markov decision processes and off-policy evaluation in `decide` | 1.11 |
| 64 | A new value of an option counts as an addition | Say so in section 14, Version numbers, beside new functions and options | 1.0.0 |
| 65 | Calling Zeff from R and Python | Thin R and Python packages over the command line and the versioned result formats, after 1.0 on request | 1.1 |
| 66 | A published speed comparison with Stata MP and R | At 1.1, when threads arrive, on the owner's machine; reported in the release notes, never a gate | 1.1 |
| 67 | The default `level:` of `ci()` on a fit made at another level | The fit's level, so that `ci()` agrees with the summary; `level:` written overrides it | 0.6.0 |
| 68 | A parameter of several types in a Zeff-written built-in | Allow the registry's `A \| B` in the `fn` line of a built-in's file, checked once per type, so that `fill_missing` and `n_missing` can be rewritten in Zeff | 0.7.0 |
| 69 | Access to the enclosing frame from a Zeff-written built-in | An implicit `Frame` parameter the checker supplies to a built-in that declares it, so that `row_mean` and `row_sum` can be rewritten in Zeff | 0.7.0 |

---

## Language

### 1. User-defined types

*Question.* How a package author's `NewsvendorFit` gets `report()`, `.plot()` and a dialog.

| Option | For | Against |
| --- | --- | --- |
| A. A `type` declaration: a named record that implements the result and estimator interfaces (Rust's structs and traits, no inheritance) | Fits the existing interface design; one mechanism for first- and third-party code; checkable at check time | Needs the interfaces to be stable first; a new declaration form |
| B. No user types: records only; new result types come from Rust components | Nothing to design | Zeff-source packages could never return a proper result; every method author needs Rust |
| C. Classes with inheritance | Familiar to Python users | Contradicts "no classes" (section 1.5); dispatch at run time breaks static checking |

*Recommendation.* A, at 1.11, with the interfaces frozen by 1.3 once `latent`'s result types show what they need. `type` is already reserved.

### 2. Multivariate outcomes

| Option | For | Against |
| --- | --- | --- |
| A. `[y1, y2] ~ x` in the formula grammar | Reads naturally | A list on the left of `~` changes the formula grammar and every function that takes a formula |
| B. A list of formulas: `sur([y1 ~ x, y2 ~ x + z])`, `manova([y1, y2], ~ group)` | No grammar change; each equation may differ, which SUR needs | Two spellings for one idea |
| C. Not supported | — | SUR and MANOVA are standard in management research |

*Recommendation.* B, after 1.0; A can become sugar later if wanted.

### 3. Fixing a parameter in a model block

| Option | For | Against |
| --- | --- | --- |
| A. `x1 @ 1` (Mplus) | `*` keeps its one meaning; `@` is unused | One more token |
| B. `1*x1` (lavaan) | Familiar to lavaan users | `*` would mean three things |
| C. `fix: {x1: 1}` on `cfa` and `sem` | No token | Separates the constraint from the equation it belongs to |

*Recommendation.* A, added to the grammar at 1.3 with the model block; add `@` to the reserved punctuation now.

*Note (28 October 2026).* #56 builds the rest of the model block's constraints, equalities and groups, on this token.

### 4. Variadic parameters

| Option | For | Against |
| --- | --- | --- |
| A. `...` in `fn` | `coalesce` writable in Zeff | Type checking of variadic calls; a second calling convention |
| B. Lists: `coalesce([a, b])` for user functions; variadics only in built-ins | Nothing new; lists already type-check | Built-ins and user functions differ slightly |

*Recommendation.* B; close the item at 0.5.0.

*Decided 30 October 2026, as recommended (option B).* A user function that takes several values takes a list; `coalesce`, `min`, `max`, `print` and `vars` stay the only variadic functions, built in.

### 5. Structural equality

| Option | For | Against |
| --- | --- | --- |
| A. `equal(a, b)` in `core`, using the identity relation (`NA` equal to `NA`), returning Bool | `assert(equal(xs, [1, 2]))` makes project tests possible; small | One more function in 1.0 |
| B. `==` on compound values | No new name | Three-valued logic on lists is confusing; the error on `==` is deliberate |
| C. None | — | Tests of user-written functions cannot compare results |

*Recommendation.* A, at 0.5.0 with `fn` and `assert`.

*Decided 30 October 2026, as recommended (option A).* `equal(a, b)` in `core`, by the identity relation of section 3.7 of the Language Reference (`NA` equal to `NA`, `-0.0` to `0.0`, lists item by item, records field by field, mappings entry by entry, `false` across types), returning `Bool`; catalogue row 529, release 0.5.0, exact, written in Rust.

### 6. Locale-aware collation and time zones

| Option | For | Against |
| --- | --- | --- |
| A. ICU collation by default | What users expect from Excel | Locale data of several megabytes in the browser; results differ by locale, against the determinism contract |
| B. Code-point order by default; opt-in `collation: "de"` with the CLDR version recorded in the run record; `to_timezone` with the tz database version recorded | Deterministic by default and when opted in; cost paid only by those who need it | Accented and non-Latin text sorted by code point surprises until the option is used |
| C. Never | Simplest | Reports in many languages need it |

*Recommendation.* B, after 1.0 (1.7 or later).

### 7. Grapheme-aware text functions

| Option | For | Against |
| --- | --- | --- |
| A. Switch `len` and `slice` to grapheme clusters | `len("e\u{301}")` is 1, as a reader sees it | Breaks parity with Python, R and Stata; Unicode segmentation rules change by version |
| B. A `unit: graphemes` argument; default stays code points | Both available; default predictable | One more argument |

*Recommendation.* B, in a 1.x release.

### 8. Zeff-source packages before 1.11

| Option | For | Against |
| --- | --- | --- |
| A. Zeff-source packages in 1.6, with the manifest and `zeff.lock`; WebAssembly components at 1.11 | Others can write methods a year earlier; no sandbox needed, the language is the sandbox | Manifest and lockfile work moves forward from 1.11 |
| B. Everything at 1.11 | One ecosystem release | About three years with no third-party methods, at a minor release every three months after 1.0 |
| C. In 1.1 | Earliest | 1.1 is already the engine's heaviest release |

*Recommendation.* A, 1.6; decide by 1.3 so the manifest is designed with the latent package's types in view.

### 9. Registry hosting

| Option | For | Against |
| --- | --- | --- |
| A. GitHub Pages for the index, GitHub Releases for archives | Free, familiar, reliable | One vendor |
| B. Own server | Full control | Cost and maintenance against the "everything free" rule |
| C. Codeberg | Non-profit, free | Smaller, slower |
| D. A on GitHub with a Codeberg mirror; the index signed, so the host is never trusted | Free, no single point of failure, trust comes from the signature | Two places to publish |

*Recommendation.* D.

### 10. `synthesize`

| Option | For | Against |
| --- | --- | --- |
| A. In the engine | Convenient | Privacy guarantees of synthetic data are a research field; a wrong promise is worse than none |
| B. A package through the registry | Can be validated separately and improved without engine releases | Later |

*Recommendation.* B, after 1.11.

### 11. Splitting the specification

| Option | For | Against |
| --- | --- | --- |
| A. Now: a Language Reference (sections 3 to 5, 7.1 to 7.2, 11, glossary) and an Engine and Project Specification | An implementer reads the language in one pass; a referee sees what binds; the independent reviewer's main structural finding | A day's work; two documents to keep consistent (the audit script already checks cross-references) |
| B. At 1.0.0 | No churn during the build | The build is when the split pays |
| C. Never, mark normative sections | Least work | The interleaving remains |

*Recommendation.* A, before 0.2.0 starts.

## Functions proposed for 1.0

The plan requires these to be adopted or dropped by 0.7.0; deciding now removes ten rows of uncertainty from the catalogue.

### 12. `posthoc` and factorial `anova(type:)`

| Option | For | Against |
| --- | --- | --- |
| Adopt | An `anova` without Tukey HSD and type-3 tables is incomplete for experimental data | `dist.tukey` (the studentized range) must be added |
| Defer | — | The 0.7.0 tests release would ship half an ANOVA |

*Recommendation.* Adopt, 0.7.0.

### 13. `iterate`

*Recommendation.* Adopt, 0.5.0: without it no EM-style or fixed-point algorithm can be written, and it is a bounded loop with a convergence test. Keep `max:` required.

*Decided 30 October 2026, as recommended.* Adopted at 0.5.0 with the catalogue's usage, `max: Int = 1000`: "required" means required to be finite, which a default is; W0805 when the loop stops at `max:` without converging, and the number of passes printed as a note.

### 14. `event()`

*Recommendation.* Adopt, 0.6.0: event-study leads and lags are the standard two-way fixed-effects check, cost little (indicator columns from a time and a treatment date), and Zemp's DiD recipe needs them.

### 15. Power and sample size

| Option | For | Against |
| --- | --- | --- |
| In 1.0 | Experimenters expect power early | Non-central t and F are new distribution code; five functions on a 1.0 that was cut to fit |
| In 1.9 | One coherent Design release with randomisation and survey designs | A year's wait |

*Recommendation.* 1.9.

### 16. `survey()` in 1.0

*Recommendation.* Defer to 1.9. Sampling weights with robust errors cover the common case in 1.0; design-based variance for every statistic and model is a release of its own.

## Build and release

### 17. Panels before tests and plots

*Recommendation.* Keep: `fe:` demeaning and the panel machinery are Zeff's differentiator and the hardest engine piece after the formula engine; facets in the plot grammar need the grouped computation of 0.6.0.

*Decided 30 October 2026, as recommended.* The order stands: 0.6.0 Panels, 0.7.0 Tests, plots and the language server.

### 18. Swap 1.2 (DiD) and 1.3 (latent variables)

*Recommendation.* Decide at the 0.8.0 alpha with a rule written down now: whichever family the alpha users ask for most goes first; DiD first if the alpha does not settle it.

### 19. Design methods for 1.9

*Recommendation.* Add to the catalogue now as 1.9 rows: designed experiments (factorial and fractional designs, blocking, response surfaces), repeated-measures ANOVA and equivalence tests (TOST). Questionnaire pre-tests are item analysis, already in `measure`.

### 20. Bayesian estimation

| Option | For | Against |
| --- | --- | --- |
| A. Priors inside `model { }` | One place for the model | `~` already means a regression path; a prior needs another operator |
| B. `prior: {educ: dist.normal(0, 1)}` arguments on the fitting functions, reusing `Distribution` values | No new syntax; priors are values; works for `reg`, `logistic`, `mixed` and `sem` alike | Long argument lists for big models |
| C. A `bayes { }` block | Separates the two | A third block form |

*Recommendation.* B; `reg`, `logistic` and `mixed` first, `sem` after. Decide at 1.9.

*Note (28 October 2026).* #59 asks how a model that is not built in is written, and #60 what differentiation such models need.

### 21. Licence

| Option | For | Against |
| --- | --- | --- |
| Apache 2.0 | Patent grant; the Rust ecosystem's norm; compatible with permissive dependencies | Slightly longer than MIT |
| MIT | Shortest | No patent grant |
| MPL 2.0 | File-level copyleft keeps improvements open | Rarer in Rust; some institutions hesitate |
| GPL | Strong copyleft | Blocks the permissive-only dependency rule in reverse for others; discourages packages |

*Recommendation.* Apache 2.0, decided 16 October 2026.

### 22. `iv.check`

*Recommendation.* Adopt, 0.10.0: the d-separation machinery is there, and an exclusion-restriction check against the graph is something no mainstream package offers.

### 23. Code signing

| Option | For | Against |
| --- | --- | --- |
| A. Pay for both | Clean installs everywhere | Apple Developer Program yearly, a Windows certificate yearly |
| B. Unsigned builds with checksums | Free | Windows SmartScreen and macOS Gatekeeper warnings stop most users at the first install |
| C. A free signing service for open-source Windows builds, plus the Apple Developer Program | One modest yearly cost; clean installs | Depends on acceptance by the service |
| D. Package managers only (winget, Homebrew) | Fewer warnings on Windows | Homebrew casks still need notarisation on macOS; students rarely use package managers |

*Recommendation.* C, decided by 0.11.0 together with #38. Since 17 October 2026 installation is from the command line only: the `zeff` command, `zeff app install lite | zemp` and the package managers verify every download against the launcher's built-in key and need no operating-system signing, so the 0.5 desktop beta ships without it; the certificate and the Apple membership are bought only when setup files (#38) are decided.

## Zemp

### 24. Interface framework

| Option | For | Against |
| --- | --- | --- |
| Svelte 5 | Small bundles, compiled, fine-grained reactivity suits a data grid and plot pane | Smaller ecosystem than React |
| SolidJS | Fastest fine-grained updates | Smaller still |
| React | Largest ecosystem; CodeMirror and grid integrations | Heaviest; re-render model fights a live data grid |
| Web components without a framework | No dependency | Everything written by hand |

*Recommendation.* Svelte 5, after a two-week spike against Solid at 0.6.0 on the two hard panes: a 100,000-row data grid and the redraw-to-fit plot viewer.

### 25. Zemp's licence

| Option | For | Against |
| --- | --- | --- |
| Apache 2.0 | Same as Zeff; contributors and institutions understand it; a commercial path remains through hosting, support and training | No licence-based revenue |
| AGPL with dual licensing | Keeps a paid licence possible | Deters institutional adoption; contributor agreements needed |
| Source-available (BSL) | Protects a future product | Not open source; against the project's framing |
| Open core: the app under Apache 2.0, the paid modules (cloud sync, version history, collaboration, admin console) under their own licence, as the business model assumes | A product to sell; the free app stays open and complete for analysis | Two licences to administer; contributors cannot touch the paid modules; the line between app and module must be drawn once and kept |

*Recommendation.* Apache 2.0, decided before the alpha is public. The business model (17 October 2026) assumes open core, which Zemp section 16 now records as the alternative; decide this item and the business model together, since "a commercial path through services" and "paid modules" cannot both be true.

### 26. Start Zemp at 0.7.0

*Recommendation.* Keep: the plot renderer and language server Zemp needs exist at 0.7.0; the framework spike (24) starts at 0.6.0 so no time is lost.

### 27. Hosting Zemp web, and sync

| Option | For | Against |
| --- | --- | --- |
| GitHub Pages | Free | Cannot set the cross-origin isolation headers threads need from 1.1 |
| A static host with custom headers (Cloudflare Pages, Netlify) | Free tier, headers, fast | A vendor to depend on; bandwidth limits |
| Sync: none (export a zip) · the user's own folder sync (a project folder in OneDrive or Dropbox on the desktop; the browser's folder access on Chromium) · an own sync server | Folder sync keeps the privacy promise with no server | A sync server costs money and holds users' data |

| A Zemp server: paid cloud sync and encrypted backup, server compute for heavy jobs, a self-hosted server for institutions, as the business model proposes | The Pro and Institution tiers exist; heavy jobs leave the laptop | Contradicts Zemp section 13 as written ("sends nothing to a server") and section 10; running costs; the privacy promise must be rewritten as opt-in, with what is sent, how it is encrypted and where it is held |

*Recommendation.* A static host that sets the headers; sync through the user's own folder sync, no Zemp server. If the business model's paid tiers are adopted, the server option replaces this recommendation, local-only stays the default, and Zemp section 13 is rewritten before the alpha is public.

### 28. AI assistance model

*Recommendation.* Both, off by default: a remote model with the user's own key for quality, a local model as the private option for restricted data; only code, variable names and summaries are sent, never rows, as section 12 of the Zemp Specification already says. A third option, metered access to a model Zemp pays for, is what the business model's Pro tier sells; it can sit beside the two above (own key and local model free, metered access paid) and is decided with #35. Decide by Zemp 1.11, when AI assistance arrives.

### 29. Zemp version coupling

*Recommendation.* Follow Zeff's minor numbers by convention until 1.11, then revisit; it is cheap and users read one number.

### 30. Performance targets

*Recommendation.* Add the two stress cases (double lasso on a 2 GB file; Callaway–Sant'Anna on 4 million panel rows) to the benchmark suite now; set each target at the first measurement and tighten it with each release, rather than guessing numbers before the engine exists.

*Decided 23 October 2026, as recommended.* The two stress cases join the benchmark suite when their methods exist (1.2 and 1.5); at 0.4.0 the suite of #52 records the first measurements of `reg`, and the targets are set from them and tightened at each release.

### 31. Names: the domain and the GitHub organisation

The installer script, the documentation, the registry index (#9) and the signed key list the launcher trusts all live under one host name, and the GitHub organisation holds the repositories, the release archives and the Action (Zeff Specification, section 2.1). Both must be fixed before the launcher ships in 0.9.0, because a key list URL cannot move without a launcher update.

| Option | For | Against |
| --- | --- | --- |
| A. `zeff.dev` and the organisation `zeff-lang` | Short; the `.dev` zone is HTTPS-only | A yearly fee; `zeff.dev` may be taken |
| B. `zeff-lang.org` and `zeff-lang` | Matches the organisation; `.org` reads as non-commercial, as `rust-lang.org` does | Longer to type |
| C. GitHub Pages only (`zeff-lang.github.io`) | No cost, no administration | A host name the project does not own; moving later breaks every installer line already published |

*Recommendation.* B, registered now and pointed at GitHub Pages, so that nothing breaks if the hosting changes.

### 32. A file of several frames and fitted results

`save` writes one frame, and in 1.0 nothing else needs saving: the result cache and `zeff run` reproduce every fit exactly, and `fit.estimates()` keeps what a table or plot needs. The heavy models of 1.5 (forests, cross-validated lasso) make re-running slow enough that a saved fit may be worth having. Whatever is chosen, it is not a binary format of Zeff's own: that would tie replication packages to one program, as `.RData` does (Engine and Project Specification, section 8.7).

| Option | For | Against |
| --- | --- | --- |
| A. A zip of Arrow IPC files and JSON result descriptions, in the versioned forms of section 8.3, point 5; `bundle("output/study.zip", frames: [wages], results: [fit])` | Every part readable by other tools as it is; versioned already; a replication package is the same thing with scripts added | A second way to keep a result beside the cache; reading one back needs `unbundle` and a stable result schema across engines |
| B. Nothing: re-run from the cache, export `estimates()` | No new machinery | A forest fitted in an hour is lost when the cache is cleared or the project moves |
| C. Results saved as Parquet tables alone (`fit.estimates().save(...)`, `fit.predict` on new data later) | Already possible in 1.0 | A saved table cannot predict, test or be compared |

*Recommendation.* A, designed at 1.5 when the first heavy estimator exists and its result schema is known; B until then.

### 33. Interface revamp decisions

Of the five decisions of zemp_gui_revamp.md, section 4, the dark theme was decided on 17 October 2026 with Zemp Lite's look, against the proposal's recommendation: a dark theme beside the light one in both editions, switched from the top bar and remembered per user, with exports always in the light print theme (Zemp Specification, sections 5 and 17). Four remain, already listed in Zemp section 17: density, icons, desktop menus, and tablet and phone. The proposal's recommendations are one density, with a larger-type setting that scales everything by 1.15; no icons in 1.0 (Zemp Lite's top bar of word controls already follows this); native menus that mirror the command palette one for one; no tablet or phone in 1.0, and the browser build says so below 1200 px.

*Recommendation.* Accept the four as proposed, before the 0.2 alpha, and merge the proposal's design language into the Zemp Specification with the dark theme added to it.

## Business model

Added 17 October 2026 from zemp_business_model.md. Each conflicts with, or is absent from, the Zemp Specification as written.

### 34. Long-term-support releases for institutions

The Institution tier offers "long-term-support releases, so results stay stable across a project's life". The cadence table of the Zeff Specification (section 14) issues patches for the current and the previous minor release only, and reproducibility is already guaranteed by exact pins and by every engine staying downloadable.

| Option | For | Against |
| --- | --- | --- |
| A. No LTS line: exact pins keep results stable; institutions buy support, not releases | Nothing to maintain; the support policy stands | "LTS" is a word procurement offices look for |
| B. One LTS minor every two years, patched for four | What institutions expect from a vendor | Two patch lines to test on four targets; backports of wrong-result fixes to an old minor |

*Recommendation.* A; if a pilot institution asks for B, cost it then.

### 35. Pricing levels and AI usage quotas

*Recommendation.* Set them from the first year's running costs (hosting, compute, model calls), not before Pro exists; keep the free tier's rules fixed in the business model ("lines never to cross") whatever the prices.

### 36. Student verification for the free Pro tier

| Option | For | Against |
| --- | --- | --- |
| A. Institutional email | Free; immediate | Not every institution has one; alumni addresses linger |
| B. A verification service | Covers more cases | A fee per check; a vendor holding students' documents |
| C. Honour system with a yearly renewal | Nothing to build | Abuse |

*Recommendation.* A, with a manual route for institutions without one; decide when Pro exists.

### 37. Trademark filing

Apache 2.0 gives others no right to the names Zeff and Zemp, but only a registered mark lets the project stop a rival app from using them.

*Recommendation.* File in the home jurisdiction before the public alpha (Zemp 0.2), and in the main markets once the first institutional customer exists; decide the jurisdiction with #31 (domain names), since the two are checked together.

## Installation, Zemp Lite and the build

Added 17 October 2026 by the third audit, from the Zemp Specification's section 17 and the risk register (zeff_risk_register.md).

### 38. Setup files for Zeff, Zemp Lite and Zemp

Installation was kept to the command line on 17 October 2026: the `zeff` launcher comes from the installer script, a package manager, an archive or the offline bundle, and the desktop apps from `zeff app install lite | zemp` or a package manager (Zeff Specification, section 2.1; Zemp Specification, section 9). Setup files (Windows `.exe` or `.msi`, macOS `.dmg` or `.pkg`, Linux `.deb`, `.rpm` or AppImage) were deferred, because unsigned ones meet SmartScreen and Gatekeeper warnings and signed ones need #23.

| Option | For | Against |
| --- | --- | --- |
| A. None until #23 is decided; the command line and the package managers are the only routes | Nothing to build or sign; one install path to test | Users who have never opened a terminal must read one page of instructions |
| B. Unsigned setup files with published checksums | Familiar double-click installs | Every install starts with a warning that most users will not click through |
| C. Signed setup files from the 0.5 desktop beta | Clean installs for everyone | Needs #23's certificate and membership from 0.11.0 |

*Recommendation.* A, revisited with #23 by 0.11.0; if the desktop beta's users ask for setup files, C.

### 39. Zemp Lite: paid features, the web address and a desktop preview

Three questions left open when Zemp Lite was decided (Zemp Specification, section 17): whether AI assistance and the paid tiers' features (cloud sync, version history) are offered in Lite as well as in Zemp; the address of Zemp Lite web; and whether a Lite desktop preview ships before the 0.5 desktop beta.

| Question | Options | Recommendation |
| --- | --- | --- |
| Paid features and AI in Lite | Lite free-tier only · Lite gains the same Pro features as Zemp · Lite gains AI assistance only | Free-tier only through 1.0, as the business model's fourth line says; the features do not exist before #27 and #28 are decided, so decide then, and prefer the same Pro features in both editions if any, since a project moves between them unchanged |
| The web address | A path on Zemp web's origin (`…/lite`) · A subdomain · A separate site | A path on the same origin, with its own manifest scope for the offline app: the browser's private file system that both editions share (Zemp Specification, section 10) is per origin, so a subdomain or a separate site would break "Open in Zemp" |
| A Lite desktop preview before 0.5 | None · A preview at the 0.2 alpha | None: it would need the Tauri packaging and `zeff app install` of 0.11.0 three releases early, and the Lite web preview at 0.2 serves the same purpose |

*Recommendation.* As the table says; decide the address before the 0.2 alpha, the rest with #27 and #28.

### 40. A specification freeze while 0.1.0 to 0.3.0 are built

The risk register's R12 recommends freezing the specification while the first three releases are built, so that the code can push back on the text and the specification does not become a second project to maintain.

| Option | For | Against |
| --- | --- | --- |
| A. Freeze the normative text until 0.3.0 except for bugs found while coding; new design work is recorded here as decisions with a deadline release | The first three releases are built against a fixed target; every change is a traceable decision | A good idea found while coding waits a release or two |
| B. Keep editing freely | Nothing changes | The documents changed many times on 16 and 17 October; without a freeze the engine chases the text |

*Recommendation.* A, from the first commit of 0.1.0; this register is reviewed at each of the three releases together with the risk register.

*Decided 18 October 2026.* Option A: the freeze holds until 0.3.0.

*Ended 22 October 2026.* At the close of release 0.3.0 the owner ended the freeze, as recommended: the only edits still waiting on it were the diagnostic codes of #41 to #44 and #46, now in section 11 of the Language Reference. From 0.4.0 a change to the normative text is an ordinary design decision, still logged in `docs/spec_changes.md` with its date, section and reason.

### 41. Diagnostic codes for three syntax conditions the specification does not number

Found while writing the lexer and parser of 0.1.0 (docs/releases/0.1.0.md, steps 6, 7 and 12). Section 3 of the Language Reference numbers the syntax errors it describes (Z0302 to Z0322), but three conditions arise that it gives no code to. The engine uses the numbers below provisionally, from the unused Z0301, Z0303 and Z0304, so that the messages have a code; the specification text is not edited (decision #40).

| Condition | Proposed code | Message in 0.1.0 |
| --- | --- | --- |
| Something the grammar does not allow at that place: a character that is not part of it (for example `#`, `$`, `@`, a letter outside ASCII outside backticks), a backtick name that is empty or never closed, or a token the parser did not expect (`expected an expression, found `)``) | Z0301 | `unexpected character`, or `expected X, found Y`, with a help line where the intent is known (`#` starts no comment; use `//`; separate arguments with commas) |
| A closing bracket with no matching opener | Z0303 | `unexpected closing )` |
| A construct the grammar has but this release does not run (`fn`, `for`, `use`, `case`, `if`, `model`, `dag`) | Z0304 | `not available in this release`, raised by the checker (step 12 of 0.1.0; ends when 0.5.0 and later releases run them) |

*Recommendation.* Accept the three numbers and add them to section 11 of the Language Reference at the next minor release, after the freeze (0.3.0). Deadline: 0.1.0 (the numbers appear in printed messages from this release).

*Decided 18 October 2026.* Accepted: Z0301, Z0303 and Z0304.

### 42. Diagnostic codes for two conditions of the project file

Found while writing the project file reader of 0.1.0 (docs/releases/0.1.0.md, step 8). Section 2 of the Engine and Project Specification says what `zeff.toml` holds and gives codes to its editions (Z0706), paths (Z0701, Z0702, Z0707), the lock (Z0703), the project's absence (Z0709) and the pinned engine (Z0710), but none to a project file that cannot be read or is wrong. The numbers Z0704 and Z0705 are taken (a step over the memory budget, a cancelled step; Language Reference section 5.6), so the engine uses the next free ones, provisionally.

| Condition | Proposed code | Exit code |
| --- | --- | --- |
| `zeff.toml` is not valid TOML | Z0712 | 1 |
| `zeff.toml` is valid TOML but wrong for Zeff: an unknown key or table (a setting that would otherwise be ignored silently), a missing `name`, `zeff` or `edition`, a value of the wrong kind, a pin that is not one exact version, a size that is not a size, a data root without a `path`, or a non-empty `[packages]` before 1.0 | Z0713 | 1 |
| A script named on the command line (`zeff check scripts/x.zeff`) that the project does not have | Z0714 | 1 |
| `import` of a path inside the project at which there is no file, or a file that cannot be read | Z0715 | 1 |

*Recommendation.* Accept both numbers and add them to the Z07xx row of section 11 of the Language Reference after the freeze (0.3.0). Unknown keys are errors, not warnings, so that a misspelt setting such as `netwrk` cannot be ignored without anyone noticing. Deadline: 0.1.0.

*Decided 18 October 2026.* Accepted as proposed: Z0712 to Z0715, an unknown key an error.

### 43. Diagnostic codes for the checker's own messages

Found while writing the typed IR and `zeff check` of 0.1.0 (docs/releases/0.1.0.md, step 12). Section 11 of the Language Reference numbers the areas (Z01xx names, Z02xx types, Z04xx arguments) and gives codes to many conditions, but not to these, which the checker meets at once. The specification's own text settles two of them without a number: "an unknown name or a repeated argument is an error" and "once one is named, the rest must be" (section 3.1); "a `let` cannot reuse a name from `core`" and "a binding may shadow a name from any other package, with a warning" (section 3.6). The engine uses the free numbers below provisionally.

| Condition | Proposed code |
| --- | --- |
| A call names no function of Zeff (`wages.maen(wage)`), with "did you mean" | Z0103 |
| A `let` reuses a name of `core` (`let print = 1`) | Z0105 |
| A `let` hides a function of another package (`let mean = 3`); a warning | W0101 |
| An argument, a receiver or a definition of the wrong type (`wages.mean("wage")`, `5.mean(x)`) | Z0401 |
| An argument name the function does not have, a definition given to a function that takes options, or a receiver written as an option | Z0404 |
| An argument or a definition given twice | Z0405 |
| A required argument missing | Z0406 |
| More positional arguments than the function takes | Z0407 |
| A positional argument after a named one | Z0408 |
| `Date("...")` or `DateTime("...")` with a format that is not one, or of the other kind | Z0409 |

Unknown bare names stay Z0101, as section 3.7 says ("elsewhere it is an unknown name (Z0101)"), with the wording "nothing is called `x` here" outside a frame method and "no variable `x` in `wages`" inside one.

*Recommendation.* Accept the numbers and add them to section 11 after the freeze (0.3.0). Deadline: 0.1.0.

*Decided 18 October 2026.* Accepted: Z0103, Z0105, W0101 and Z0401 to Z0409.

### 44. Diagnostic codes for the descriptive reducers' warnings

Found while registering `mean` and `sd` for 0.1.0 (docs/releases/0.1.0.md, steps 12 and 20). Section 4.1 of the Language Reference says that statistics skip `NA` with a warning giving the count, that `Inf` and `-Inf` together give `NA` with a warning, and that a sum of all-`NA` input is `NA`, but gives these warnings no number. W0202 and W0203 are taken by the conversions of `to`.

| Condition | Proposed code |
| --- | --- |
| Missing values skipped by a reducer, with the count (also an all-missing column, whose result is `NA`) | W0201 |
| `Inf` and `-Inf` both present, so the result is `NA` | W0204 |

*Recommendation.* Accept both and add them to section 11 after the freeze (0.3.0). Deadline: 0.1.0.

*Decided 18 October 2026.* Accepted: W0201 and W0204; an undefined standard deviation (one value, or an infinite value) uses W0205, the specification's code for a result that is NA from 0/0 or Inf − Inf.

### 45. Whether a warning printed under a result shows its code

Found while writing the interpreter of 0.1.0 (docs/releases/0.1.0.md, step 13). Section 10.2 prints warnings and notes under the output they belong to as `warning: 12 rows gave −Inf from log(sales) where sales = 0 (line 9)` and `note: 211 values of wage read as NA from na_values -99 (line 5)`, without a code, while section 11 numbers every warning and the pragma `// zeff: allow(W0203)` of section 11.1 needs the code. The engine follows section 10.2.

| Option | For | Against |
| --- | --- | --- |
| A. As section 10.2 prints it, without the code | Reads as a sentence; the code is in the run record and in `--format json` | A user who wants to allow the warning must look its code up |
| B. `warning[W0201]: 12 rows ... (line 9)` | The code is at hand for `allow(CODE)` and `zeff explain` | Heavier; departs from the printed layout of section 10.2 |

*Recommendation.* B, when the pragma arrives (0.7.0); A until then. Deadline: 0.2.0, when `zeff explain` pages begin.

*Decided 18 October 2026.* Option A until the pragma arrives (0.7.0): a warning under a result prints without its code, as section 10.2 shows; the code is in `--format json` from 0.2.0 and in the explained-code pages. Errors keep their code.

### 46. Diagnostic codes for four conditions of expressions and calls

Found while writing the operators of 0.2.0 (docs/releases/0.2.0.md, step 2). Section 3.7 settles the conditions in words but gives them no number: "Bool is not a number: `female + 1` with a Bool `female` is an error that suggests `to(female, Int)`"; comparison "only between values of one type"; a list indexed by something that is not an Int; a mapping looked up by a key of another type; "keys are unique". The engine uses the free numbers below provisionally, in the Z02xx range of types and values; the specification text is not edited (decision #40).

| Condition | Proposed code |
| --- | --- |
| An operator applied to types it does not take: a Bool or `Nothing` in arithmetic (text has its own Z0215), `-` on text, `!` on a number, `&&` or `||` on numbers, a comparison across types (`1 == "1"`, `true < 2`), `..` with a Float end, `[ ]` on a number, a list indexed by text, a mapping key of another type | Z0221 |
| A mapping key written twice in a literal (check time) or computed twice (run time) | Z0222 |
| Two options of one call that exclude each other, given together: `round(x, digits: 2, sig: 3)` (found at step 11; section 3.7 gives `round` both options and no rule for the pair). Section 6 already uses Z0410 for one such pair, `se: classical` with `weight_type: sampling`; the page generalises that use | Z0410 |
| A bare name inside a frame method's argument that is both a variable of the frame and a binding: the variable is used, with a warning (found at step 15; section 3.7, "Name lookup", asks for the warning and gives it no code) | W0102 |

*Recommendation.* Accept all four and add them to section 11 with the codes of #41 to #44 after the freeze (0.3.0). Deadline: 0.3.0.

*Decided 22 October 2026.* Accepted as proposed: Z0221, Z0222, Z0410 and W0102. They are in section 11 of the Language Reference with the codes of #41 to #44 and the two numbered with the distributions (Z0413, Z0111).

## Performance

### 47. A fixed reduction shape for every sum: independent accumulators in a fixed order

Raised on 21 October 2026 in a review of the performance plan. Bit-identity depends on the order of the additions, not on how many are done at once. If every sum over rows is defined as N independent running totals, row i going to total i mod N, the totals combined in one fixed order, and no fused multiply-add, then a scalar loop, SSE2, AVX2, AVX-512, NEON and simd128 give the same bits, and a native build may use the widest vectors it finds. Section 8.3, point 3 asks for "fixed blocking and reduction order"; this decision names the shape.

| Option | For | Against |
| --- | --- | --- |
| A. Keep today's order in each kernel, no named shape | No change to any result | Vector width is limited to what each loop happens to allow; the order is a convention kept in each kernel, not a rule |
| B. N independent accumulators (N = 8 proposed), combined in a fixed order, for every reducer | One rule for all kernels; wide vectors allowed natively; the cross-target check is the proof | N becomes part of the numerical definition; sums change in the last digits against earlier versions, and the snapshots and the R comparisons of the stats functions already shipped must be re-read |

The gain should be measured before the rule is adopted: a micro-benchmark of one sum and one cross-product, scalar against 8 accumulators, on the arm64 and x86 targets. The wide x86 levels matter less than the shape itself; the owner's machine is arm64 (128-bit).

*Recommendation.* B, with N fixed by that measurement and recorded once. Changes to the shipped functions' last digits are logged in the notes of the release that makes them. Deadline: 0.4.0, before the first release whose cost depends on a long sum.

*Decided 23 October 2026, as recommended (option B).* N is set by the micro-benchmark of step 2 of release 0.4.0 (a sum and a dot product, scalar against N lanes, on the owner's ARM64 machine and the x86-64 runner) and recorded once in `docs/decisions.md`; every kernel of `zeff-kernels` is written to the shape from its first line; the reducers shipped at 0.1.0 to 0.3.0 move to it at 0.4.0 only if the measurement shows a gain of a fifth or more, since the move changes the last digits of the published conformance cases and the samples' output.

### 48. Blocked kernels from the start, threads when proven

Raised on 21 October 2026 in a review of the performance plan. Section 14 puts threads in 1.1. A deterministic parallel design exists: rows in fixed-size blocks (the size independent of the number of threads), one partial result per block, the partials combined in a fixed order, so 1 thread or 64 give the same bits. Writing the kernels this way costs little; adding threads to kernels written as one long loop means rewriting them.

| Option | For | Against |
| --- | --- | --- |
| A. Single-threaded kernels now, rewrite for threads in 1.1 | Nothing new to test | The rewrite touches every kernel and risks the results already validated |
| B. Kernels blocked from now, the blocks run one after another; threads switched on later | The switch cannot change a bit; no scheduling to debug in early releases; the block structure is also what #47 and #49 need | A small cost in code shape now |
| C. Threads from 0.1.0 | The largest speed-up, native | Brings forward work the build order puts later; browser threads need cross-origin isolation and a nightly toolchain; a service-worker shim is fragile on first load |

Two conditions in any case: random draws use one stream per draw, derived from the seed and the draw's number, so the thread count never changes a result; and a test runs each blocked kernel at 1 and at several threads and demands equal bits.

*Recommendation.* B for the engine and the native build; the browser build stays single-threaded until threads there are shown to work. Deadline: 0.4.0 for the shape; 1.1 for the threads.

*Decided 23 October 2026, as recommended (option B).* The kernels consume row blocks of a fixed size, run one after another; a test runs each kernel at several block sizes and demands equal bits; threads arrive at 1.1 for the engine and the native build, and the browser build stays single-threaded until threads there are shown to work.

### 49. When fused passes arrive: 0.2.0 or 0.6.0

Raised on 21 October 2026 in a review of the performance plan. Large-data kernels are limited by memory traffic. Processing rows in cache-sized blocks across all the columns a statement uses, and carrying `where` as a selection of rows instead of copying them, can speed up the frame verbs and the regression pipeline by a factor of two or three. Section 14 places fused expressions at 0.6.0.

Two corrections to the claim as first made. Fixed-effects demeaning needs several sweeps, so only the last sweep can feed the cross-products directly. And accumulating X′X directly favours the normal equations, which are less stable than a QR decomposition; section 6 decides which the regression uses.

| Option | For | Against |
| --- | --- | --- |
| A. As planned, fused expressions in 0.6.0 | Verbs are first written plainly and validated | Verbs may need rewriting once |
| B. The block and selection interface in the verbs from now; fusion itself at 0.6.0 | The later change is a swap inside the interface | A small cost in the verbs now |
| C. Fusion at 0.2.0 | Earliest gain | Brings work forward; complicates the verbs while they are being validated |

Whichever is chosen, every fused result is tested against the unfused one for equal bits.

*Recommendation.* B. Deadline: 0.4.0.

*Decided 23 October 2026, as recommended (option B).* The block interface of #48 is the one the verbs adopt at 0.6.0 with the fused passes; nothing in the verbs changes at 0.4.0.

### 50. Compact column storage: narrow types, dictionary codes and a validity mask

Raised on 21 October 2026 in a review of the performance plan. Every pass over a column costs in proportion to its bytes. Section 4.3 of the Language Reference fixes the column types and the Arrow metadata; this decision concerns only how each is held in memory, which no script can observe.

| Option | For | Against |
| --- | --- | --- |
| A. Every numeric column as 64-bit floats; text as text; missing values as a sentinel checked on every row | Simplest kernels | Ten million dates or categories weigh 80 MB each; every reducer tests for the sentinel on every row |
| B. Each column in its narrowest honest form: dates as day counts (Int32), categories as small integer codes with a sorted dictionary, Bool as one bit per row, missing values in a separate validity bitmap; floats stay 64-bit | Three to ten times fewer bytes on the common columns; grouping by a categorical variable is a counting pass over the codes, with the groups in key order from the sorted dictionary; the validity bitmap lets a kernel skip whole blocks with no missing values | More kinds of column for each kernel to accept; the dictionary must stay sorted after `append` and `recode` |

Floats never change width: no reduced precision on any result-bearing path. The arithmetic is defined over the values, so the storage cannot change a bit.

*Recommendation.* B, settled before the panel verbs of 0.6.0, since `aggregate`, `panel` and the joins are written against the column interface. Deadline: 0.4.0.

*Decided 23 October 2026, as recommended (option B).* The column store already holds each column in this form (Arrow dictionary codes for a category, Int32 days for a date, one bit per row for a Bool, a validity bitmap; Int stays Int64 and Float 64-bit, as section 3.8 of the Language Reference says); the decision confirms it, and the model matrix of 0.4.0 reads a category's codes directly.

### 51. Reading files in parallel by chunk, with deterministic boundaries

Raised on 21 October 2026 in a review of the performance plan. Parsing a CSV is often the slowest step of a session and sits outside every kernel. Splitting the file at fixed byte offsets, moving each split to the next newline (with quoted newlines handled by a fixed rule), parsing the chunks independently and concatenating them in order gives the same frame from the same bytes whatever the number of threads, so it belongs with the thread pool of 1.1 and the Arrow cache of section 8.7 rather than with the determinism contract.

| Option | For | Against |
| --- | --- | --- |
| A. One pass, one thread, as now | Nothing to prove | Import of a large file is minutes, and the cache only helps the second time |
| B. Chunked parsing from the start, chunks run one after another until 1.1; the chunk boundaries and the row order fixed by the bytes alone | The switch to threads in 1.1 cannot change the frame; type inference is done once over the whole file, not per chunk, so a chunk cannot infer a type that another contradicts | A chunked reader is more code; quoted newlines need a scan to settle the boundaries |

Type inference and the `na_values` rules of section 6.1 are applied to the whole column after the chunks are joined, so a chunk never decides a type.

*Recommendation.* B, in the reader rewrite of 0.11.0 at the latest; earlier if the 0.6.0 panel work shows import to be the bottleneck. Deadline: 0.6.0.

### 52. A benchmark suite run at every release

Raised on 21 October 2026 in a review of the performance plan. The testing handbook has no benchmark. Without one, a speed-up is a guess and a slow-down goes unseen until a user reports it; section 14 names a public benchmark suite only at 1.0.0.

| Option | For | Against |
| --- | --- | --- |
| A. The public benchmark suite at 1.0.0, nothing before | No work now | Ten releases of kernels written without measurement |
| B. A small suite from 0.4.0: `import` of a CSV, `aggregate` by a categorical variable, `merge` on sorted keys, one `reg` with and one without `fe:`, at three sizes (10⁴, 10⁶, 10⁷ rows), run on the arm64 and x86 native targets and in the browser build, with the times recorded in each release's notes | Every change is measured; the 1.0.0 public suite grows out of it | Times vary with the machine; a threshold that fails CI would be noisy, so the suite reports and the release notes judge |

The suite measures time only; a result it produces is also checked for equal bits against the stored snapshot, so a benchmark can never be made faster by a numerical change.

*Recommendation.* B, as a new section of the testing handbook, added after the freeze (0.3.0) and run from 0.4.0. Deadline: 0.4.0.

*Decided 23 October 2026, as recommended (option B).* The suite is step 21 of release 0.4.0, written into a new section of the testing handbook and run from 0.4.0; it reports its times, and the release notes judge them.

### 53. Options rather than functions: HAC standard errors and weights on the descriptives

Raised on 21 October 2026 with the catalogue additions of that date (rows 506 to 524). Two of the suggested additions are options on existing functions, so they cannot be catalogue rows, and both would extend a usage frozen at 1.0.

* `se: newey_west` (with `lags: Int`) and `se: driscoll_kraay` on `reg`, `ivreg` and the panel models: autocorrelation-robust standard errors for time series and panels. Conley spatial errors are already in section 14.1; these are their simpler cousins and share the variance layer's kernel form.
* A `weights: Var` option on `mean`, `sd`, `variance`, `median`, `quantile`, `correlation` and `tabulate`, for frequency weights, before the survey package of 1.9 brings design weights.

An option with a default that keeps earlier behaviour is an addition a minor release may make after 1.0 (section 14, Version numbers), so neither needs the freeze lifted; the question is only which release and whether the two kinds of weight (frequency now, design at 1.9) share the one option name.

*Recommendation.* The `se:` variants with the panel models of 1.4; `weights:` on the descriptives at 1.1, as frequency weights, with `survey()` of 1.9 overriding them by design weights where a design is declared. Deadline: 1.1.

### 54. A second range of variables in one formula

Raised on 23 October 2026 at step 7 of release 0.4.0, when formulas became values. Section 3.3 puts `..` at level 6, more loosely bound than `+` and `-` at level 5, and section 3.4 lets no range chain (Z0314). A formula's right side is one expression tree, so `y ~ d + x1..x9 + z` parses as `(d + x1)..(x9 + z)`; the checker reads the range as joining the nearest names on each side, which gives `d`, `x1..x9` and `z`, as section 7.1 intends. But `y ~ x1..x9 + z1..z5` parses as `x1..(x9 + z1)..z5`, a chained range, and is refused by the parser before any formula rule applies. The engine therefore reads a bracketed range, `(z1..z3)`, as the range itself, so that a second range can be written; a bare second range stays Z0314.

| Option | For | Against |
| --- | --- | --- |
| A. The grammar stands: one bare range per formula, further ranges in brackets `(z1..z3)`, documented in section 7.1's table | No change to section 3.3 or the grammar; `2..n + 1` keeps reading `2..(n + 1)` everywhere | A formula with several ranges needs brackets, and the error for two bare ranges speaks of chaining |
| B. `..` binds more tightly than `+` and `-` (between levels 4 and 5), inside formulas and out | `y ~ x1..x9 + z1..z5` reads as two ranges with no brackets | A grammar change before 1.0: `2..n + 1` becomes `(2..n) + 1`, an error, and every script with a range in arithmetic changes meaning |
| C. `..` binds more tightly only inside a formula | Formulas read as R users expect; arithmetic is untouched | The one operator whose precedence depends on context, which section 3.3 avoids on purpose |

*Recommendation.* A: the grammar is the one thing a user learns once, and brackets for a second range are a small price. Deadline: 0.6.0, when `fe:` takes `Terms` and the grammar of terms is used in two places.

## Capabilities for later releases

Added 28 October 2026. The owner asked for three families of capability: psychometric and other latent-variable models; Bayesian models written in the manner of Stan, BUGS and JAGS; and machine learning from stacking to deep and reinforcement learning, with an architecture that lets packages grow towards them. Each request was checked against both volumes and the catalogue. Much is already planned: the `model { }` grammar (section 7.4 of the Language Reference), `weights:` with `weight_type:` on every model, `Matrix` and `Vector` with `*` as the matrix product, the result and estimator interfaces shared by built-in and third-party packages (section 8.3, point 6), the leakage-safe learning of section 7.5 with `spec.*`, `learn` and `tune`, the bandits and Q-learning of the `decide` package, and the catalogued latent, measurement and learning functions. What remains to decide is below.

### 55. Large and pretrained models under the determinism contract

Large neural networks are trained on graphics processors, whose results differ between machines, and pretrained ("foundation") models arrive as files of weights trained elsewhere. The question was whether Zeff should admit results that are not bit-identical, in a second accuracy class, or keep the contract of section 8.3, point 3 for every result and give such models routes that respect it.

| Option | For | Against |
| --- | --- | --- |
| A. A second accuracy class for device-dependent results (graphics processors, native libraries), labelled in the run record | The quickest path to large models inside Zeff | Breaks the promise every other result keeps; a replication package could no longer be checked bit for bit |
| B. The contract holds for every result; large and pretrained models reach a result by three routes that respect it | Nothing Zeff reports changes from machine to machine; packages can still grow towards these models | Large networks train slowly on the processor alone; training on graphics processors happens outside Zeff |

The three routes of option B:

| Route | What runs where | Bit-identical |
| --- | --- | --- |
| 1. A package | Zeff-source and WebAssembly component packages (section 8.4) fit and run models on the processor under the contract: tree ensembles, feed-forward, convolutional, sequence and graph networks of moderate size, Gaussian processes | Yes, and the validated badge is open to them |
| 2. A frozen file | A model trained anywhere (on graphics processors, in Python, by a third party) is saved as a file of weights, read from the project or a data root like any data file, its hash recorded in the run record (section 2) and carried into replication packages; Zeff runs the model through route 1 | The training is outside Zeff and outside the promise; everything Zeff computes from the file is bit-identical |
| 3. Outside Zeff | The training itself, and anything that cannot be frozen, runs in R or Python through `external` (from 1.1; section 8.3, point 8), whose results the run record already marks as outside Zeff's validation and bit-identical guarantees; a call to a hosted model goes through `fetch` (section 8.6), whose responses are cached, hashed and replayed, so a re-run reproduces them | No for the outside step itself, and the run record says so |

*Decided 28 October 2026 by the owner.* Option B: results stay bit-identical on every machine, and extensions keep that promise. The manifest's `effects = "none"` stays its only value (section 8.4); no package reaches a graphics processor, the network, the clock or the file system; and the optional GPU kernels of section 14.1 stay admitted only if they meet the contract on every target.

The engineering the decision needs, to be written into sections 8.3 and 8.4 (the wording by 1.0.0, when the extension interfaces are published; the build by 1.11, when components ship):

* Section 8.4 says components are bit-identical "by construction". WebAssembly makes that true only under three rules the host enforces, which the text should state: every NaN a component returns is made canonical by the host, since WebAssembly leaves a NaN's bit pattern open; the relaxed-SIMD instructions, whose results may differ between processors, are refused when a component is loaded; and shared-memory threads inside a component are refused, parallelism coming from the host instead.
* The component interface gives a component the kernel's random streams, so that its draws follow the rule of section 8.3, point 3, and a fixed-block parallel map: the host runs the component's function on fixed row blocks over the thread pool of 1.1 and combines the partial results in a fixed order, so a component is as parallel as a built-in package without threads of its own.
* A file of weights is read whole as a `Bytes` value (section 4.3 of the Language Reference) and passed to the component as an argument; a component never reads a file itself. The name of the function that reads a file whole is for the owner when components are built.
* A row that is a tensor (an image, a sequence) is a fixed-size list column carrying Arrow's canonical fixed-shape tensor extension in its metadata, as embeddings are already fixed-size lists; no tensor type enters the language.

### 56. Constraints and groups in model blocks

Most of the latent-variable models on the owner's list that Zeff lacks need no estimator of their own, only a way to say that parameters are fixed, equal, or free by group: bifactor and higher-order CFA; the unmeasured latent method factor, marker-variable and CFA-marker tests of common method variance; multitrait–multimethod and CT-C(M−1) models; longitudinal invariance; random-intercept cross-lagged panel models; STARTS and ALT models; latent state–trait models; latent change-score models; piecewise and latent-basis growth; and multi-group SEM. Section 7.4 of the Language Reference has labels (`x as a`) and defined parameters (`:=`), and #3 recommends `@` for fixing; nothing yet says how parameters are made equal or how a model differs between groups. Each form below is rejected as an error by the 1.0 grammar or checker, so adding it at 1.3 breaks no script.

| Option | For | Against |
| --- | --- | --- |
| A. In the block, as lavaan does: a label repeated means equal (`y1 ~ x as a` and `y2 ~ x as a`); `@` fixes (`g ~~ s1 @ 0` for an orthogonal bifactor); lines such as `a == b` and `a > 0` constrain labelled or defined parameters; `group:` on `cfa` and `sem`, a label written as a list giving one label per group (`x as [a1, a2]`), and `group_equal: [loadings, intercepts, residuals]` for the usual invariance steps | Every model of the list is a few readable lines; lavaan users recognise it; the model stays in one place | A repeated label carries meaning without a mark, so the editor and the printed summary must show the equalities it implies |
| B. Arguments on the fitting functions: `fix: {...}`, `equal: [[a, b]]`, `group:` | No new form inside the block | Constraints sit far from the paths they constrain; bifactor and RI-CLPM models need long argument lists |
| C. Group sections inside the block, as Mplus writes them | Familiar to Mplus users | A second structure inside the block, and a larger grammar |

*Recommendation.* A, with hover and the printed summary listing every equality a repeated label implies. Each model of the list above gets a checked help example (section 9.1) fitting it with these lines, so that the common-method-variance tests and RI-CLPM are templates rather than functions. Deadline: 1.3, with #3.

### 57. Latent functions brought into 1.3, 1.6 and 1.7

The catalogue holds several latent-variable functions as "after 1.0", with no release. Some cost little once their release's machinery exists, and management and survey research reports them in almost every paper.

| Release | Brought in | Why there |
| --- | --- | --- |
| 1.3 | `invariance` (configural to strict, and partial invariance by freeing parameters), `latent_growth`, `factor_scores`, `composite_reliability`, `ave`, `fornell_larcker`, `htmt` | `invariance` and `latent_growth` are model blocks with #56's constraints and groups; the other five are closed forms on a fitted CFA |
| 1.6 | `efa`, `kmo`, `sphericity_test`, `parallel_analysis`; invariance for ordered items; bifactor-ESEM and set-ESEM | Rotation (geomin, target) and polychoric correlations are 1.6's engine work; set-ESEM is `efa()` blocks with #56's constraints |
| 1.7 | `lpa`; covariates and distal outcomes by the three-step and BCH methods, as options of `lca` and `lpa`; latent class and mixture regression | Each reuses 1.7's EM with many random starts and class-label alignment |

| Option | For | Against |
| --- | --- | --- |
| A. As the table says | Each family arrives complete enough for a published paper | 1.3 becomes the largest method release, and its checklist has more steps |
| B. The releases stay as they are; these functions wait for later rows | Smaller releases | Users of 1.3 compute composite reliability, AVE and HTMT by hand, and test invariance without `invariance` |

*Recommendation.* A. If #18 puts latent variables before difference-in-differences, the widened release moves with it. Deadline: 1.3.

### 58. Latent-variable releases after 1.11

With #56 and #57, the remaining models of the owner's list need engine work of their own. They fall into five families, which section 14.1 lists.

| Family | Models | Engine work |
| --- | --- | --- |
| Multilevel latent models | Multilevel CFA; two-level SEM; multilevel mediation (1-1-1, 2-1-1, 2-2-1) and cross-level moderated mediation with random slopes; the multilevel latent covariate model; multilevel latent classes | A two-level likelihood on the machinery of 1.3 and 1.4 |
| Latent interactions | LMS and QML; latent moderated mediation; latent quadratic effects; product indicators with double-mean-centring as a helper | Numerical integration inside the likelihood |
| Item response | `irt` (Rasch, 2PL, 3PL, graded response), partial credit and generalised partial credit, nominal, multidimensional and bifactor models, explanatory IRT; `dif`; `alignment` | Marginal maximum likelihood with quadrature; the EM of 1.7 |
| Mixtures over time | `lta`; `growth_mixture` and LCGA; factor mixture models; hidden Markov and Markov-switching models; latent class logit | The EM of 1.7, with forward–backward recursions |
| Composite models | PLS-SEM, PLSc, GSCA, confirmatory composite analysis | Closed-form iterations; possibly a package rather than the engine (section 8.4's own example is a `pls_sem` package) |

After the Bayesian release (1.10; #59): Bayesian CFA with small-variance priors, Bayesian approximate invariance, Bayesian SEM and dynamic SEM. Not scheduled, and left to packages or a later review: moderated nonlinear factor analysis, MIIV-2SLS, continuous-time SEM, cognitive diagnosis models, unfolding models (GGUM), Thurstonian IRT, ideal-point models, hybrid choice (ICLV) models, interactive fixed effects and IPCA, and FAVAR.

*Recommendation.* The order of the table, which puts first what management and organisational research asks for most. Two things are needed earlier, so that no door closes: the 1.3 likelihood is written so that a group, and later a level, is an index over which parameters may differ; and the mixed models of 1.4 expose their random-effect structures to the `latent` package. Deadline: 1.3 for those two; 1.11 for the order.

### 59. Bayesian models of the user's own, in the manner of Stan and BUGS

Release 1.10 fits the built-in models and `mle` objectives by NUTS, with priors given as arguments (#20). Stan, BUGS and JAGS go further: the user writes any model, its parameters, priors and likelihood, and the sampler fits it. How such a model is written decides whether a word must be reserved before 1.0, since a reserved word added after 1.0 needs a new edition (section 3.1 of the Language Reference).

| Option | For | Against |
| --- | --- | --- |
| A. Extend `model { }`: a line with a distribution on the right of `~` is a sampling statement, as in BUGS (`y ~ dist.normal(mu, sigma)`, `mu ~ dist.normal(0, 10)`), beside the paths and factors the block already holds | No new word or block; structural and Bayesian models read alike; priors in model blocks are already the plan of section 14.1 | `~` is read by the type of its right side, terms or a distribution, which the checker must explain plainly |
| B. A log-density written as a Zeff function and passed to a fitting function, as Stan's `target +=` accumulates one | No grammar at all; any model the language can compute | Less readable for the common hierarchical models |
| C. A new block word, such as `bayes { }` | A clean separation from structural models | Must be reserved before 1.0, or wait for a new edition; a third block form |

Whatever the form, the release that brings it adds: parameters with constraints (positive, bounded, simplex, correlation matrix) mapped to an unconstrained space with the Jacobian; discrete parameters by marginalisation, as Stan does; split R-hat, effective sample sizes, divergences, and trace and rank plots; posterior predictive checks; LOO and WAIC; and results that carry their draws (#61). Chains run in parallel on the random streams of section 8.3, so a fit is bit-identical whatever the number of threads.

*Recommendation.* A for models that read as statistical models, B for anything else; no new reserved word. Deadline: 1.0.0 for the reserved word; the design at 1.10, with #20.

### 60. Reverse-mode differentiation

Section 8.3, point 9 gives user objectives forward-mode automatic differentiation at 1.8. Forward mode costs one pass per parameter for a gradient: fine for a likelihood with ten parameters, impractical for NUTS on a user's model with thousands of parameters (1.10 samples `mle` objectives; #59 goes further) and for neural networks (#63). Reverse mode gives the whole gradient in one backward pass. Built-in estimators keep their analytic derivatives either way.

| Option | For | Against |
| --- | --- | --- |
| A. Forward mode only, as written | Simplest; no record of the computation to keep | Gradients of many parameters are too slow for 1.10 and for the families after it |
| B. Reverse mode beside forward mode, both on the typed IR, the backward pass summing in a fixed order; Hessians as forward-over-reverse | One pass for a gradient whatever the number of parameters; bit-identical, since the order of every sum is fixed | More engine work at 1.8 |

*Recommendation.* B. An engineering choice, written into section 8.3, point 9 and the 1.8 row when 1.8 starts, unless the owner objects. Deadline: 1.8.

### 61. Results that carry draws

From 1.1 a fit may have bootstrap replicates (`se: bootstrap`), from 1.10 posterior draws, and multiple imputation (section 14.1) pools several fits. In 1.0 a result's uncertainty is its variance matrix, and `derive` uses the delta method (section 15). If the result interface keeps only that, every later kind of result needs an interface of its own.

*Recommendation.* An engineering choice: from 1.1 a result may carry draws (bootstrap replicates, posterior draws, or the fits of several imputations), and `ci`, `derive`, `test`, `report` and `plot` ask the result for its kind of uncertainty, the variance matrix and the delta method for an asymptotic fit, the draws themselves for a bootstrap or posterior fit. Which interval the draws give (percentile or another) is a statistical definition, settled with `se: bootstrap` at 1.1. Nothing printed by 1.0 changes. The interface is fixed before third-party packages implement it (1.6 if #8 is taken, otherwise 1.11). Deadline: 1.1.

### 62. Prediction after 1.5: stacking and conformal prediction

The catalogue already holds most of the prediction methods the owner listed: `lasso`, `tree`, `random_forest`, `boosting` (gradient-boosted trees as XGBoost and LightGBM fit them), `svm`, `knn`, `naive_bayes`, `neural_net`, `calibration`, `shap`, `partial_dependence`, `causal_forest`, `meta_learner` (S-, T-, X-, R- and DR-learners), `blp` and `gates`, with double machine learning in 1.5. Two common methods are missing, and both depend on what 1.5 keeps from its folds.

| Method | What it is | Name proposed |
| --- | --- | --- |
| Stacking | A meta-model fitted to the out-of-fold predictions of several model specifications, under the leakage rules of section 7.5 | `spec.stack(models: [...], meta: spec.reg())`, a specification like the others |
| Conformal prediction | Distribution-free prediction intervals and sets around any `LearnFit`, split and cross-conformal | `fit.conformal(level: 0.9)`, calibrated on held-out rows |
| Ordered boosting | CatBoost's handling of categories and row order | An option of `spec.boosting` |

*Recommendation.* Add `spec.stack` and `conformal` to the catalogue as rows after 1.0 once the owner approves the names, and have `tune` and `folds` keep every row's out-of-fold predictions from 1.5, so that stacking needs nothing new from them. Tabular foundation models (TabPFN and its kind) are pretrained weights and take route 2 of #55. Deadline: 1.5.

### 63. Neural networks, pretrained models and sequential decisions

#55 decides the routes; this entry proposes which models Zeff builds itself and which it leaves to packages.

| Kind | Built in | Through packages (#55, routes 1 and 2) | Outside Zeff (#55, route 3) |
| --- | --- | --- | --- |
| Neural networks | `neural_net` (feed-forward), with deep ensembles (several seeds) as an option, after #60 | Convolutional, recurrent and state-space sequence models; graph neural networks; transformers of moderate size; variational autoencoders and normalising flows | Training of large language, vision and diffusion models; fine-tuning, including LoRA |
| Pretrained models | None | Inference from frozen weights: tabular and time-series foundation models, embedding models | Calls to a hosted model through `fetch`, replayed from its cache |
| Sequential decisions | `bandit`, `q_learning`, `a_learning`, `policy_tree` and `ewm` of the `decide` package (catalogued); proposed: Markov decision processes by value and policy iteration, and off-policy evaluation | Deep reinforcement learning on the processor | Environments and agents in other software, through `external` |

*Recommendation.* As the table says. Deadline: 1.11, when component packages ship and the line between built in and packaged must be drawn.

### 64. A new value of an option counts as an addition

Section 14, Version numbers lists what a minor release may add after 1.0: "new functions, packages, options with defaults that keep earlier behaviour, formats and diagnostics". Many later rows add a value to an option that already exists (`se: newey_west` in #53, `estimator: wlsmv` at 1.6), which the list does not name; and the sentence after it, that an addition never changes "whether it runs", could be read as forbidding a script that failed before from running after.

*Recommendation.* An engineering correction of the wording: add new values of an option to the list, and say that an addition never stops a script that runs. Deadline: 1.0.0.

### 65. Calling Zeff from R and Python

Section 8.3, point 8 keeps interoperability at the data boundary: no shared memory and no embedded interpreter. Many users would still like to run a Zeff project from R or Python and read its results there.

| Option | For | Against |
| --- | --- | --- |
| A. Thin R and Python packages that run `zeff run` on a project and read the frames and results it writes, in the versioned formats of section 8.3, point 5 | No engine change; the same results as the command line; a way into Zeff for R and Python users | Two more packages to maintain, outside the engine's own releases |
| B. Nothing: users run `zeff` and read the files themselves | No work | A barrier for the users most likely to try Zeff |

*Recommendation.* A, after 1.0 and on request, as the PyPI and conda packages of the command line are (section 2.1). Deadline: 1.1.

### 66. A published speed comparison with Stata MP and R

The owner asked whether Zeff will be faster than Stata MP on eight cores. The benchmark suite (#52) times Zeff alone; section 8.3, point 10 compares the stress cases with the fastest R and Python tools; nothing compares Zeff with Stata. Until 1.1 Zeff runs each statement on one thread (section 1.4), so a comparison before then would measure the wrong thing.

*Recommendation.* At 1.1, when threads arrive, run the suite's cases in Zeff, in Stata MP on eight cores and in R (fixest and data.table) on the owner's machine, and publish the times in the 1.1 release notes whichever way they fall, adding the two stress cases of #30 as their methods arrive (1.2 and 1.5); the comparison reports and never gates a release. It needs the owner's Stata MP licence. Deadline: 1.1.

### 67. The default `level:` of `ci()` on a fit made at another level

Raised on 30 October 2026 at step 4 of release 0.5.0, when the owner compared `a.ci(educ)` with the summary of `a`, a fit made with `level: 0.9`: the summary prints 90% intervals and `ci()` returns 95% ones, since the catalogue's usage is `fit.ci(term: Term?, level: Float = 0.95)` and the default does not read the fit. `report` and `summary` use the fit's level; `ci()` alone does not.

| Option | For | Against |
| --- | --- | --- |
| A. The default is the fit's level: `ci()` on a fit made at 90% gives 90% intervals, and `level:` written out overrides it | Agrees with the summary and with R's `confint`, whose default follows nothing but 0.95 either, yet never contradicts the printed table; one rule for every interval of a fit | The usage line reads `level: Float = the fit's level`, a default that is not a literal; `ci()` on two fits at different levels gives intervals at different levels |
| B. Keep 0.95 as the catalogue says | Nothing changes; the number is on the page | A fit printed at 90% returns 95% intervals unless the level is repeated |

*Recommendation.* A, at 0.6.0, with the catalogue's usage corrected to `level: Float = the fit's level` and the page saying so; the same rule for `predict`'s intervals when they arrive. Deadline: 0.6.0.

### 68. A parameter of several types in a Zeff-written built-in

Raised on 31 October 2026 at step 15 of release 0.5.0. A built-in written in Zeff declares its parameters with the grammar's `type`, one type each; the loader checks a scalar parameter in its column forms, which covers `clamp(x: Float, ...)` on a column but not a parameter that takes a value of any scalar type (`fill_missing(x, value)`: Bool, Int, Float or Str, joined) or a list or a column (`n_missing(x)`). Both were planned in Zeff and are written in Rust at 0.5.0, their `layer` corrected (logged); `summary`, planned for a rewrite in Zeff at 0.5.0, meets the same limit with its `vars` and `stats:` parameters. A Rust built-in writes `x: Bool | Int | Float | Str` in its registry signature and the checker already understands it.

| Option | For | Against |
| --- | --- | --- |
| A. Allow `A \| B` in the `fn` line of a built-in's file only (not in a project's `fn`), the body checked once per type as it is per column form, the result typed by `Join` or `Same` where the signature says so | One signature language for Rust and Zeff built-ins; `fill_missing` and `n_missing` become the one-line Zeff bodies the catalogue planned; the body is still checked before a script runs | The `fn` grammar differs between a built-in's file and a project's; the forms multiply (each union doubles or triples them) |
| B. Several homes, one `fn` per type in one file (`n_missing` on a list and `n_missing` on a column), the checker choosing by the first argument as section 8.1, rule 6, chooses between packages | The grammar stays one; each body is simple | Two bodies to keep in step for one function; rule 6 chooses between packages, not within one, so the registry and the checker need a new case |
| C. Keep them in Rust | Nothing to build | The count of Zeff-written built-ins stays below the plan and the language cannot express two of its own simplest functions |

*Recommendation.* A, at 0.7.0 with `zeff test` and `zeff fmt`, when the `fn` line of a built-in is next touched; the catalogue's `layer` returns to Zeff for both rows then.

### 69. Access to the enclosing frame from a Zeff-written built-in

Raised on 31 October 2026 at step 17 of release 0.5.0. `row_mean([q1, q2, q3])` inside `add` reads three variables of the frame whose method encloses the call; a Rust body reads them from the call's context, but a function written in Zeff sees its parameters and other functions only (section 5.3, "Scope"), and a `Vars` parameter is a list of names with no frame to look them up in. Both row functions were planned in Zeff and are written in Rust at 0.5.0, their `layer` corrected (logged). The same limit will meet every Zeff-written column function that takes variables rather than a column.

| Option | For | Against |
| --- | --- | --- |
| A. An implicit parameter: a built-in's `fn` line may declare `data: Frame` as its first parameter marked as the enclosing frame (a spelling to choose), which the checker supplies from the frame method the call stands in, Z0401 outside one as `n()` gives | The body is ordinary Zeff over a frame it was given, `data.add(...)` and the reducers included; the registry's usage line stays `row_mean(vars, ...)` | A new piece of signature grammar for built-ins only; the checker must know which frame (the group's rows under `by:`, as the Rust context gets) |
| B. Change the usage to a frame method, `data.row_mean(vars)`, so that the frame is an ordinary argument | No new grammar; the body is ordinary Zeff | `wages.add(score = wages.row_mean([q1, q2]))` repeats the frame and breaks under `by:`; the catalogue's usage changes |
| C. Keep them in Rust | Nothing to build | Every column function over variables stays Rust |

*Recommendation.* A, at 0.7.0 with open decision #68, when the `fn` line of a built-in is next touched; the catalogue's `layer` returns to Zeff for both rows then.
