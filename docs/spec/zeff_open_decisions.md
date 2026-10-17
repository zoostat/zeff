# Zeff and Zemp: Open Decisions, Options and Recommendations

Date: 17 October 2026 · Source: Zeff Specification section 15, Zemp Specification section 17, the Zemp business model (zemp_business_model.md) and the risk register (zeff_risk_register.md) · The decisions are yours; each entry gives the options, what each costs and buys, a recommendation, and the release by which the choice has to be made.

**Status (16 October 2026, 19:54).** The "now" group was accepted and applied: #11 (the specification is published as two volumes, `zeff_language_reference.md` and `zeff_engine_specification.md`, generated from the one source; the source was removed on 17 October, and the volumes, now `zeff_language_reference.md` and `zeff_engine_specification.md`, are edited directly), #12–#14 and #22 (`posthoc` with factorial `anova(type:)` and `dist.tukey`, `iterate`, `event`, `iv.check` adopted; 1.0 is 256 functions, 257 since `g.plot()` joined on 17 October), #15–#16 (power functions and `survey()` deferred to 1.9), #19 (`factorial_design`, `fractional_design`, `response_surface`, `repeated_anova`, `tost` added as 1.9 rows), #21 (Apache 2.0 decided). The remaining 21 decisions stay open with their deadlines. Added 16 October 2026, 20:55: the command line and installation (zeff_cli_install.md) were accepted and applied with their recommendations (launcher plus engines on demand, the installer script with package managers listed first, the append-only console, baseline x86-64 builds, PyPI and conda after 1.0); one new decision, #31 (names), is open, so 22 remain. Added 17 October 2026: storage formats were decided (Engine and Project Specification, section 8.7: Arrow IPC for the caches, deterministic Parquet for kept files, the Zeff Parquet profile, BLAKE3 hashes), and one decision they leave open, #32 (a file of several frames and results), so 23 remain.

**Status (17 October 2026).** The second audit (zeff_audit_2026-10-17b.md) was applied: the release once planned as 2.0 is 1.11, so #1, #8, #10 and #29 now say 1.11; the interface revamp's five decisions, already in Zemp section 17, are #33; the business model's conflicts with the Zemp Specification are recorded here rather than settled, as options under #25 (open core), #27 (a Zemp server) and #28 (metered AI), and as #34 to #37 (long-term support, pricing and quotas, student verification, trademarks). 28 decisions are open; #4 closes at 0.5.0 if its recommendation stands.

**Status (17 October 2026, third audit).** The third audit (zeff_audit_2026-10-17c.md) was applied: #33 now records the dark-theme decision of 17 October and keeps its other four questions; #23 notes that the command line and `zeff app install` need no operating-system signing, so its deadline is tied to the setup-file decision; #8's option A reads 1.11, not 2.0; and three decisions were added from the Zemp Specification's section 17 and the risk register: #38 (setup files), #39 (Zemp Lite's paid features, web address and desktop preview) and #40 (a specification freeze while 0.1.0 to 0.3.0 are built). 31 decisions are open. Next deadlines: #40 at 0.1.0; #30 by 0.4.0; #4, #5 and #17 by 0.5.0.

## Summary

| # | Decision | Recommendation | Decide by |
| --- | --- | --- | --- |
| 1 | User-defined types | A `type` declaration (named record + interfaces) at 1.11; interfaces fixed now | 1.3 |
| 2 | Multivariate outcomes | A list of formulas, `sur([f1, f2])`, no grammar change | after 1.0 |
| 3 | Fixing a parameter in a model block | `@`, as Mplus; reserve the token now | 1.3 |
| 4 | Variadic parameters | No: lists instead; variadics stay built-in only | 0.5.0 |
| 5 | Structural equality | Add `equal(a, b)` to `core` at 0.5.0 | 0.5.0 |
| 6 | Locale collation, time zones | Opt-in `collation:` with a pinned CLDR version; `to_timezone` with a pinned tz database; after 1.0 | 1.x |
| 7 | Grapheme-aware text | `unit: graphemes` argument; default stays code points | 1.x |
| 8 | Zeff-source packages before 1.11 | Yes, in 1.6 with the manifest and lockfile; components at 1.11 | 1.3 |
| 9 | Registry hosting | GitHub Pages and Releases with a signed index, mirrored on Codeberg | 1.5 |
| 10 | `synthesize` | A package through the registry, not the engine | after 1.11 |
| 11 | Split the specification | Yes, now, before coding starts | now |
| 12 | `posthoc` and factorial `anova(type:)` | Adopt, 0.7.0 | 0.7.0 |
| 13 | `iterate` | Adopt, 0.5.0 | 0.5.0 |
| 14 | `event()` | Adopt, 0.6.0 | 0.6.0 |
| 15 | Power functions | Defer to 1.9 as one Design release | 0.7.0 |
| 16 | `survey()` in 1.0 | Defer to 1.9 | 0.7.0 |
| 17 | Panels (0.6.0) before tests and plots (0.7.0) | Keep the order | 0.5.0 |
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
| 30 | Performance targets | Put the two stress cases in the benchmark suite now; set targets at first measurement, tighten each release | 0.4.0 |
| 31 | Names: domain and GitHub organisation | `zeff-lang.org` with the `zeff-lang` organisation; the installer URL and signing-key list depend on it | before 0.9.0 |
| 32 | A file of several frames and fitted results | A zip of Arrow files and JSON result descriptions, at 1.5 with the heavy models; never a binary format of Zeff's own | 1.5 |
| 33 | Interface revamp: density, icons, desktop menus, tablet and phone (the dark theme was decided on 17 October 2026) | As zemp_gui_revamp.md section 4 recommends for the four: one density with a larger-type setting; no icons; native menus mirroring the palette; no tablet or phone in 1.0 | Zemp 0.2 alpha |
| 34 | Long-term-support releases for institutions | None: exact pins and engines downloadable for ever already keep results stable; sell support, not a release line | Zemp Institution tier |
| 35 | Pricing levels and AI usage quotas | Set from the first year of running costs, not before Pro exists | Zemp Pro |
| 36 | Student verification for free Pro | Institutional email, with a manual route for institutions without one | Zemp Pro |
| 37 | Trademark filing for Zeff and Zemp | File in the home jurisdiction before the public alpha; Apache 2.0 gives others no right to the names | Zemp 0.2 alpha |
| 38 | Setup files for Zeff, Zemp Lite and Zemp | None until #23 is decided: `zeff app install` and the package managers cover the desktop apps; plan setup files with #23 | 0.11.0 |
| 39 | Zemp Lite: paid features and AI in Lite; the web address; a desktop preview before 0.5 | Free-tier only through 1.0, revisited with #27 and #28; Lite web at a path on Zemp web's origin; no earlier desktop preview | Zemp 0.2 alpha (the address); the rest with #27 and #28 |
| 40 | A specification freeze while 0.1.0 to 0.3.0 are built | Freeze the normative text except for bugs found while coding; record further design work here with a deadline release (risk register, R12) | 0.1.0 |
| 41 | Diagnostic codes for three syntax conditions the specification does not number | Z0301 something the grammar does not allow at that place (an unexpected character, an unclosed quoted name, an unexpected token); Z0303 an unmatched closing bracket; Z0304 a construct the grammar has but this release does not run | 0.1.0 |
| 42 | Diagnostic codes for two conditions of the project file | Z0712 `zeff.toml` is not valid TOML; Z0713 `zeff.toml` has an unknown key, a missing key or a value of the wrong kind | 0.1.0 |

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

### 4. Variadic parameters

| Option | For | Against |
| --- | --- | --- |
| A. `...` in `fn` | `coalesce` writable in Zeff | Type checking of variadic calls; a second calling convention |
| B. Lists: `coalesce([a, b])` for user functions; variadics only in built-ins | Nothing new; lists already type-check | Built-ins and user functions differ slightly |

*Recommendation.* B; close the item at 0.5.0.

### 5. Structural equality

| Option | For | Against |
| --- | --- | --- |
| A. `equal(a, b)` in `core`, using the identity relation (`NA` equal to `NA`), returning Bool | `assert(equal(xs, [1, 2]))` makes project tests possible; small | One more function in 1.0 |
| B. `==` on compound values | No new name | Three-valued logic on lists is confusing; the error on `==` is deliberate |
| C. None | — | Tests of user-written functions cannot compare results |

*Recommendation.* A, at 0.5.0 with `fn` and `assert`.

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

### 41. Diagnostic codes for three syntax conditions the specification does not number

Found while writing the lexer and parser of 0.1.0 (docs/releases/0.1.0.md, steps 6, 7 and 12). Section 3 of the Language Reference numbers the syntax errors it describes (Z0302 to Z0322), but three conditions arise that it gives no code to. The engine uses the numbers below provisionally, from the unused Z0301, Z0303 and Z0304, so that the messages have a code; the specification text is not edited (decision #40).

| Condition | Proposed code | Message in 0.1.0 |
| --- | --- | --- |
| Something the grammar does not allow at that place: a character that is not part of it (for example `#`, `$`, `@`, a letter outside ASCII outside backticks), a backtick name that is empty or never closed, or a token the parser did not expect (`expected an expression, found `)``) | Z0301 | `unexpected character`, or `expected X, found Y`, with a help line where the intent is known (`#` starts no comment; use `//`; separate arguments with commas) |
| A closing bracket with no matching opener | Z0303 | `unexpected closing )` |
| A construct the grammar has but this release does not run (`fn`, `for`, `use`, `case`, `if`, `model`, `dag`) | Z0304 | `not available in this release`, raised by the checker (step 12 of 0.1.0; ends when 0.5.0 and later releases run them) |

*Recommendation.* Accept the three numbers and add them to section 11 of the Language Reference at the next minor release, after the freeze (0.3.0). Deadline: 0.1.0 (the numbers appear in printed messages from this release).

### 42. Diagnostic codes for two conditions of the project file

Found while writing the project file reader of 0.1.0 (docs/releases/0.1.0.md, step 8). Section 2 of the Engine and Project Specification says what `zeff.toml` holds and gives codes to its editions (Z0706), paths (Z0701, Z0702, Z0707), the lock (Z0703), the project's absence (Z0709) and the pinned engine (Z0710), but none to a project file that cannot be read or is wrong. The numbers Z0704 and Z0705 are taken (a step over the memory budget, a cancelled step; Language Reference section 5.6), so the engine uses the next free ones, provisionally.

| Condition | Proposed code | Exit code |
| --- | --- | --- |
| `zeff.toml` is not valid TOML | Z0712 | 1 |
| `zeff.toml` is valid TOML but wrong for Zeff: an unknown key or table (a setting that would otherwise be ignored silently), a missing `name`, `zeff` or `edition`, a value of the wrong kind, a pin that is not one exact version, a size that is not a size, a data root without a `path`, or a non-empty `[packages]` before 1.0 | Z0713 | 1 |

*Recommendation.* Accept both numbers and add them to the Z07xx row of section 11 of the Language Reference after the freeze (0.3.0). Unknown keys are errors, not warnings, so that a misspelt setting such as `netwrk` cannot be ignored without anyone noticing. Deadline: 0.1.0.
