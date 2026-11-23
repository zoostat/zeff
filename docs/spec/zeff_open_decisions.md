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

**Status (1 November 2026, at the close of release 0.5.0).** The four decisions due at 0.5.0 (#4, #5, #13 and #17) were taken on 30 October and are built (`equal` as catalogue row 529; `iterate`; no `...` in `fn`; the order of 0.6.0 and 0.7.0 kept). Six were raised during the release: #67 (the default level of `ci()`, due 0.6.0) and #68 to #72, the pieces the language lacks for the functions planned in Zeff (a parameter of several types, the enclosing frame, a note and a coded warning from a body, the fitted design, a table constructor; #72 due 0.6.0, the rest 0.7.0). Due by 0.6.0: #14 (`event()`), #26 (Zemp's framework spike), #51 (chunked reading), #54 (a second range in a formula), #67 and #72, to be put to the owner when release 0.6.0 opens.

**Status (1 November 2026, after the release).** One decision raised from the owner's requests for the Zemp interface, recorded without any change to the normative text: #73, typst as the typesetting engine of exports and of the report document, with a live preview in both editions. Deadline 0.8.0; the spike is proposed for 0.7.0.

**Status (1 November 2026, at the opening of release 0.6.0).** The six decisions due by 0.6.0 (#14, #26, #51, #54, #67 and #72) were taken on 1 November 2026, each as recommended: `event()` adopted; Zemp starts at 0.7.0 with the framework spike run during 0.6.0; chunked reading in the reader rewrite of 0.11.0 unless the 0.6.0 benchmarks show `import` to be the bottleneck; the grammar stands, a second range in a formula written in brackets; `ci()` defaults to the fit's level (catalogue row 163 corrected); `table(title:, columns:)` added to `core` (catalogue row 530), with the rewrite of `summary` waiting for #70. None is due before 0.7.0, where #68 to #71, #76 and #77 fall due.

**Status (1 November 2026, later the same day).** Five more decisions from the owner's interface requests, recorded without any change to the normative text: #74 (a Help pane), #75 (the data viewer and the environment browser), #76 (traceback, run to here and the loop inspector), #77 (schema-aware completion and extract function) and #78 (Git history and a diff of results). Deadlines 0.8.0, 0.8.0, 0.7.0, 0.7.0 and 1.1. Each is drawn on the owner's mockup canvas of Zemp Studio and Zemp Lite, outside the repositories.

**Status (5 November 2026, the owner's proposal of 4 November).** The owner's proposal of 4 November 2026 for shorter and clearer scripts (kept outside the repository) was checked against both volumes, the catalogue, this register and the code, and the owner asked for its core set to be recorded here, with no change to the normative text: #80 (`each(...)` as the one marker for "separately", and tests on several variables), #81 (model sets: `nested` and `each` terms in a formula, `RegFits`, and one frame of estimates for a set), #82 (what `size:` means for a table in `export`), #83 (the sentence of section 3.7 on bare words, beside the option records of section 3.1) and #84 (reserving `each` and `peek` before `core` freezes). Deadlines 0.7.0, 0.8.0, 0.11.0, 0.7.0 and 0.11.0. The proposal's other items are additions a minor release may make before or after 1.0 and are not recorded; each is raised here if the owner adopts it. Nothing enters release 0.6.0.

**Status (6 November 2026, at the close of release 0.6.0).** The six decisions due at 0.6.0 (#14, #26, #51, #54, #67 and #72) were taken on 1 November and are built or stand (`event` as catalogue row 252; Zemp at 0.7.0; the chunked reader at 0.11.0, its condition read at the pre-release routine, where `import` was found not to be the bottleneck of panel work; one bare range in a formula; `ci()` at the fit's level; `table` as row 530); none remains due. Raised since the close of 0.5.0: #73 to #78 (the owner's requests for the Zemp interface), #79 (arithmetic over lists, or a map, in a Zeff-written built-in, raised at step 10) and #80 to #84 (the core set of the proposal of 4 November). The summary rows of #12, #15 and #16 now read "decided 16 October 2026", as the status paragraph of that day has recorded them since. Due by 0.7.0, to be put to the owner when that release opens: #24 (the Zemp interface framework; the owner said on 6 November that its two-week trial, placed in a Zemp session during 0.6.0 by #26, has not yet been run, so the trial comes before the decision), #68 to #71 and #79 (the pieces the Zeff-written built-ins lack), #76 and #77 (the traceback and schema-aware completion), #80 (`each`, with the tests or after them) and #83 (the sentence of section 3.7 on bare words). #73's trial of the typesetting engine is planned beside the plot renderer of 0.7.0, its decision due at 0.8.0.

**Status (6 November 2026, at the opening of release 0.7.0).** The owner said "go" to the checklist of `docs/releases/0.7.0.md` and took its eight questions as recommended. Decided: #68, #69, #70 and #79 (option B: `map`, `filter` and `reduce` over a function value), built at steps 3 and 3b; #71, built with the result plots at step 30; #76 (option A: the traceback at 0.7.0, the inspector and **Run to here** with Zemp's alpha); #77 (option A: completion from the schema pass at 0.7.0, extract function at 1.1); #80 (option B: the rule adopted, built in the first later release with room, its text written then with the owner's two statistical choices; the word kept under #84); #83 (option A, the row of section 3.7 corrected). The adopted pieces bring six names (`note`, `name_of`, `map`, `filter` and `reduce` in `core`, `design` in `results`); their catalogue rows and the counts of both volumes follow in the steps that build them. #24 stays open: its two-week trial is run in a Zemp session during this release and the decision taken before the close. #73's trial runs as a tool-only step beside the plot renderer (step 22b), its decision due at 0.8.0. Raised at this opening: #85 (the Games–Howell comparison in `posthoc`), #86 (the release of folding, snippets and the statement ranges behind **Run**; formatting through the language server is built at 0.7.0) and #87 (a variable name in an option record, found while correcting section 3.7).

**Status (6 November 2026, later the same day).** #31's domain was decided by the owner: `zeff-lang.dev`. A look-up in the public DNS that day found `zeff.dev`, `zeff.org`, `zeff.com`, `zeff.io` and `zeff.app` registered by others, and no record for `zeff-lang.dev`, `zeff-lang.org`, `zeff-lang.io` or `zefflang.org`; the GitHub names `zeff-lang` and `zefflang` were free. The organisation stays open, due before 0.9.0, with `zeff-lang` recommended; the installer line of section 2.1 of the Engine and Project Specification now names the host.

**Status (7 November 2026, release 0.7.0 under way).** Steps 2 to 4.4 of 0.7.0 applied #45, #68, #69, #70 and #79 (each entry says how); #89 is raised at step 4.4 and #90 at step 5b, both with deadline 0.8.0.

**Status (7 November 2026, a study of R packages).** The owner asked for 135 R packages to be studied against the catalogue (the study and its plan are kept outside the repository), then asked which functions should join 1.0, and decided the same day: the nine items of #88, all as proposed. Eight functions join 1.0, five of them from the roadmap (`ar_test` at 0.8.0; `margins`, `roc` and `confusion` at 0.9.0; `mcnemar_test` at 0.10.0) and three new (`dist.weibull`, `dist.logistic` and `dist.hypergeometric` at 0.10.0, catalogue rows 531 to 533), and `merge` gains `join: semi | anti` at 0.10.0. 1.0 is 267 functions, with 266 on the roadmap and 533 in all. Three further additions were offered and declined; the study's proposals for after 1.0 are not recorded here and stay with the owner.

**Status (8 November 2026, release 0.7.0, before step 7).** The owner raised #91 (hypothesis and decision lines in every printed test, with `alternative:` on the four directional tests), to be decided before step 7 builds `TestResult`; the entry records his recommendation (A) and the two gaps it exposes (`shapiro_test` above 5,000 observations; the printed name of an unlabelled group). Decided the same day: A, with α fixed at 0.05 where no `level:` exists; the help examples, the testing handbook, the catalogue and the Language Reference are updated the same day; the two gaps were settled the same day (`shapiro_test` refuses more than 5,000 observations, as R does; an unlabelled group prints as `female=0` and `female=1`).

**Status (9 November 2026, release 0.7.0, after step 12).** The owner asked how a test's figures are read from a bound result; the check found that the fields the Language Reference says a `TestResult` carries (`statistic`, `df`, `p_value`, `estimate`, `effect_size`, `h0`, `h1`, `alternative`) cannot be read by name, that `df` and the effect size's value cannot be reached at all, and that a one-value column has no way to a number. Recorded as #94 at the owner's request, due at 0.7.0, since step 7 built the type and the fix belongs with it. Decided the same day: B, as recommended; step 12b of the release builds it.

**Status (9 November 2026, release 0.7.0, after step 13).** The owner asked for a form of `print` that does what R's `cat` does; today a one-line sentence needs `join` and `to(x, Str)`. Recorded as #95, due at 0.8.0, and decided the same day: A, `print` itself puts plain values side by side on one line with `sep:`, built as an `[engine]` step of 0.8.0.

**Status (9 November 2026, release 0.7.0, after step 13, later).** The owner asked what must be added before 1.0. The engine check found that a `DateTime` cannot measure a difference shorter than a day, give its hour as a number or be reduced by `min` and `max`, and that no `Date` can be built from year and month columns. Recorded as #96 at the owner's request, due at 0.8.0, with the recommendation to add both bundles there as `[function]` steps. Amended the same day at the owner's suggestion: the differences become one function, `time_between(start, end, unit:)`, replacing `days_between` and `months_between`.

**Status (9 November 2026, release 0.7.0, after step 14).** The owner set the rule that one function with a bare-word option stands where names would differ only by a unit, a statistic or a method, and asked that the four families failing it be recorded. Recorded and decided as #97: the rule applies to every future addition, and the merges of `add_days` and `add_months`, `quantile_group` into `bin`, `fill_forward` into `fill_missing`, and the three variance tests into `variance_test` are due at 0.8.0.

**Status (10 November 2026, release 0.7.0, after step 16).** The owner asked whether Types I, II and III sums of squares were planned for the factorial `anova`; the text had types 2 and 3 only. Recorded as #98 and decided the same day: the function covers every type R and Stata give, so `type: 1 | 2 | 3 = 2`, built with the factorial form at step 17, whose line in `docs/releases/0.7.0.md` says so; catalogue row 171 is amended at that step.

**Status (10 November 2026, release 0.7.0, the review of #99).** The owner made the sentence of #98 a rule for every function and asked for each to be reviewed. The 278 functions due by 1.0 were held against R (argument lists read from the installed library) and Stata (from its manuals as remembered; Stata is not installed, so nothing was run). About 70 functions lack something R or Stata gives; the gaps are listed in #99 in three classes, each with the option that would close it. Nothing is scheduled yet. Two questions wait on the owner before steps 17 and 19 are built: the reading of the rule, and whether the further methods of `posthoc` and `p_adjust`, the effect sizes and covariates of `anova` and the options of `histogram` and `density` enter release 0.7.0.

**Status (10 November 2026, release 0.7.0, question 2 of #99).** The owner ruled on question 2 of #99: "Yes, add them". The routine gaps of the steps not yet built enter release 0.7.0: a covariate in the factorial `anova` (step 17), `effect: eta | omega | epsilon` on both forms of `anova` (new step 17b), four further methods of `p_adjust` (new step 18b), seven further methods of `posthoc` (step 19), and the bin rule, scale and start of `histogram` with the kernel of `density` (steps 28 and 29). The release gains no function, only options; its checklist gains two steps. The one point the ruling left, how a negative omega or epsilon squared is printed, the owner settled the same day: as it is, with a note. The owner also confirmed the reading of the rule (question 1): "Yes, as read". Questions 3 and 4 of #99 stay open, due at 0.8.0.

**Status (12 November 2026, release 0.7.0, question 2 of #110).** Asked "Add `triangle_down` and `star` as the seventh and eighth point shapes?", the owner answered "Yes, add both". Question 2 of #110 is decided as option D: eight shapes for categories, the six of step 20d in their order, then `triangle_down` and `star`; each may also be named as a constant, solid or with `_open`. Built as step 20f of 0.7.0, before step 21. The other four questions of #110 stay open with their deadline.

**Status (12 November 2026, release 0.7.0, #126).** Asked "Open decision #126: draw the text of an SVG figure as letter outlines?", the owner answered "Yes, outlines". Decided as recommended: an exported SVG draws its text as outlines of the embedded fonts, so that a figure looks the same in every program whether or not IBM Plex is installed there, and keeps the words in the file for search; text with the font embedded, as an option of `export`, waits until after 1.0. Built at step 21c of 0.7.0; the sentence of section 7.6 of the Language Reference is amended in that step.

**Status (13 November 2026, release 0.7.0, #134).** The owner asked what stops a researcher from changing the code or the data inside a replication package when `zeff verify` still reports *identical*. Recorded as #134, due at 0.11.0, the release that builds replication packages, with no change to the normative text.

**Status (13 November 2026, release 0.7.0, #135).** The owner asked for a function that gives citation details, and chose its name, `cite`, over R's `citation`. Recorded as #135, due at 0.10.0, the release of the citation file and the DOI of #116, with no change to the normative text; its release is open.

**Status (13 November 2026, release 0.7.0, #135 decided).** Asked to choose the release of `cite`, the owner answered "Option A". `cite` and its three formats arrive at 0.10.0 with the citation file and the DOI, and `zeff cite` at 0.11.0 with the run records it reads.

**Status (13 November 2026, release 0.7.0, #100, #101, #102 and #105 decided).** Shown the four decisions due at this release, the owner answered "A, but B for #102". #100, #101 and #105 are option A and are built as steps 38d, 38e and 38f of the release's checklist; #102 is option B, nothing at 1.0, the question returning at 1.1 with the less common gaps of #99.

**Status (13 November 2026, release 0.7.0, #141).** The owner asked for HTML output of Quarto's quality, HTML slides, reports written in Markdown and a package for them, and named it `publish`. Recorded as #141, due at 0.8.0 with #73 and #124, whose writing format it amends; the Markdown format and the name are the owner's rulings, the release and the file extension are open. No normative text is changed.

**Status (13 November 2026, release 0.7.0, #141 decided).** Asked to settle #141 as recommended, the owner answered "Yes, A with .zmd". `publish` is created at 0.10.0 with `report()` and `cite()` in it; the Markdown documents, `render` and `zeff render` follow after 1.0, HTML first; the file extension is `.zmd`. #124 and #135 carry notes of what #141 changes in them.

**Status (13 November 2026, release 0.7.0, #142).** Shown the sample report of #141 drawn as HTML, the owner asked that its figures be PNG and not sit on a white background in the dark theme. Recorded as #142, due with the HTML documents of #141, after 1.0; that the figures of an HTML document are PNG is the owner's ruling. No normative text is changed.

**Status (13 November 2026, release 0.7.0, #142 decided).** Asked to settle #142 as recommended, the owner answered "Yes, option A". `export(background:)`, `theme(mode:)` and the light and dark PNG of each figure in an HTML document are built with the HTML documents of #141, after 1.0; the two options are amended into the catalogue when that release is planned. No normative text is changed now.

**Status (13 November 2026, release 0.7.0, #144).** Asked whether Zeff can handle a terabyte of data, the owner asked that the gap found be recorded: nothing in the specification reduces a file larger than memory before it is loaded. Recorded as #144, to decide by 1.1 and to build at 1.7 with `stream`. No normative text is changed.

**Status (13 November 2026, release 0.7.0, #144 decided).** Asked to settle #144 as recommended, the owner answered "Yes, option A". The six parts are built with `stream` at 1.7; the lazy chains of 1.1 are built to run row group by row group so that 1.7 adds to them. No normative text is changed now.

**Status (13 November 2026, release 0.7.0, #27, hosting ruled).** Asked whether the website could go on GitHub instead of Cloudflare now that Zemp Studio is deferred, the owner was told that the website already goes on GitHub Pages and that Zemp Lite web still needs a host that sets the cross-origin isolation headers, from 0.8.0 rather than 1.1; asked to choose between Netlify and Cloudflare, he answered "Yes, record it" to Cloudflare Pages. Sync and a Zemp server stay open. No normative text is changed.

**Status (13 November 2026, release 0.7.0, #27, address ruled).** The owner ruled that Zemp Lite web is served at `lite.zeff-lang.dev`. No normative text is changed.

**Status (14 November 2026, release 0.7.0, #146).** While the website and Zemp Lite web were drawn, the owner ruled on their look: the logo, the tagline, the fonts, one type scale, a light/dark button and the logo watermark on every page, purple buttons, realistic pages, and the Panes layout of Zemp Lite with its output, data and variables viewers. Recorded as #146, decided by those rulings; the Zemp Specification, section 5, is amended when Zemp 0.2 alpha (0.8.0) is planned. No normative text is changed now.

**Status (15 November 2026, at the opening of release 0.8.0).** The owner said "go" to the checklist of `docs/releases/0.8.0.md` and took its nine questions as recommended, under his standing instruction of 12 November 2026 (each his to overturn). Decided: #86, #87, #89 (with the argument's name), #90, #108 and #147, each option A, built at steps 8b, 4, 7a, 7b, 6 and 5; #73 option A in steps (`.typ` export at step 21a, the library measured as a dependency at step 21b, PDF through it decided from the figures); #81 option B of the checklist (the design adopted for 1.0, built after 1.0 as an addition; block G builds it at 0.8.0 only if the owner asks for it in 1.0); #92 option B (the tests stay in Rust); #93 option B deferred (the noncentral t with the `power` family, in the release of #113's plan that builds it); the second halves of #130 and #131 built at step 8a, those of #132 and #136 not built before 1.0; question 2 of #109 not built before 1.0 (an addition placed with the grammar extensions of 1.2 under #122). #18 is decided at the close of this release from what the alpha's users ask for, DiD first otherwise. #74 and #75 are the alpha's, built in Zemp sessions; #116's first layer is the alpha's web host's, and the launcher's count waits for 0.9.0. The decisions of 0.7.0 that placed work here are carried by the checklist: #103 (step 2 and block F), #96 (steps 8d to 8f), #97 and question 4 of #99 (steps 8g to 8k), #113, #120 and #122 (step 8c). Also taken on the recommendation: `ivmodel` and `ivDiag` join the R reference library as reference tooling and the Card data come from `wooldridge`; the IV examples run on a new synthetic dataset, `staff`, an encouragement design; the definitions of question 3 of the checklist, as fixest, ivreg2 and Stata have them; the pages of `core`, `text` and `time` are revised at this release, `data`, `datasets`, `dist` and `stats` at 0.9.0, `results`, `regress` and `plot` at 0.10.0. No normative text is changed at the opening; section 9.1 is amended at step 2 under #103.

**Status (15 November 2026, release 0.8.0, step 8c1: the plan of #113 applied).** The catalogue, section 14 of the Engine Specification, the release numbers written in the two volumes and section 15 of the Zemp Specification follow the 29 releases of #113, without third-party packages before 2.0.0 (#123); `docs/spec_changes.md` of this date has the account. The entries below that are still open and give a release after 1.0 as their deadline, or name one in their recommendation, were written against the plan before #113 (#1, #3, #20, #28, #53, #56 to #63, #65, #66 and #78): each number there reads as the release the plan now gives its subject (1.2 is 1.19, its plot layers 1.2 and PPML 1.7; 1.3 is 1.12; 1.4 is 1.8 and 1.11; 1.5 is 1.4, 1.14, 1.18 and 1.20; 1.6 is 1.12 and 1.25; 1.7 is 1.13; 1.8 is 1.15; 1.9 and 1.10 are 1.17; 1.11 is 1.15, and 2.0.0 for packages; 1.1 stays), and each is reviewed, with its deadline, when the release that carries its subject is planned. One is rewritten now because the Engine Specification repeats it: #32, due at 1.14 with the heavy models. The owner's table also spells five methods differently from their catalogue rows without #113 listing them as merges (`polychoric`, `meta_3level`, `alignment`, `effect_convert` beside `effect_size`, and `spec.logistic`): the rows stay, and each is the owner's to settle when its release is planned (`docs/decisions.md` of this date).

**Status (15 November 2026, the owner's ruling on Zemp's repositories).** #148: Zemp Lite is developed in a public repository under Apache 2.0 and Zemp Studio, when its work starts, in a separate private one under a proprietary licence. It replaces the repository and licence parts of #125, and with them #25; sections 1, 2, 3, 15, 16 and 17 of the Zemp Specification are amended.

**Status (16 November 2026, the article for the Journal of Statistical Software).** The owner's draft of an article on Zeff for the Journal of Statistical Software (kept outside the repository) was checked against the journal's published rules for authors and its mission statement, and against both volumes, the catalogue and this register. The owner asked for the changes that give the article its best chance to be recorded here: #151 to #161, in the new group "Raised for the journal article" at the end of this register. No normative text is changed. Four of them (#151, #156, #157 and the choice of study in #155) reverse or extend a ruling of the owner or set the article's own course, so they wait for his word and are not taken under the standing instruction of 12 November 2026; the others are taken as recommended at their deadlines unless he says otherwise. Asked the same day which changes to the release plan would follow, the owner asked for two more to be recorded: #162 (a release 0.12.0 for the bridges to R and Python, before the names freeze) and #163 (Poisson regression with fixed effects, the wild cluster bootstrap and difference-in-differences brought forward to 1.2 and 1.3). Both change the plan the owner set, so both wait for his word.

**Status (19 November 2026, the review against R on outside datasets).** Thirteen scripts on twenty datasets the owner supplied were run on the commit of step 20a and compared with R: 688 of 698 figures agree, and every difference is explained. The engine's faults are steps 20d to 20j of the 0.8.0 checklist. The design questions are recorded as #165 to #171, in a new group "Raised by the review against R": the length of each text value (#165), a row-wise count of missing values (#166), a row number (#167), one value out of a column (#168), widening a guessed type on import (#169), six smaller refusals (#170) and six printed layouts (#171). Deadline 0.9.0 for each; each is taken as recommended under the standing instruction of 12 November 2026 unless the owner says otherwise. No normative text is changed.

**Status (17 November 2026, the owner's question on polynomial regression).** #164 recorded at the owner's request: R's `poly` and Stata's `orthpoly`, orthogonal polynomials of a continuous variable, have no counterpart in Zeff, where powers are written in the formula. Recommended as a formula term at 1.2; deadline 1.1. No normative text is changed.

**Status (20 November 2026, at the close of release 0.8.0).** Built at this release, as decided: #86, #87, #89, #90, #95, #96, #97 with question 4 of #99, #108 and #147, the second halves of #130 and #131, the first third of the pages of #103 (`core`, `text` and `time`), and the plan of releases of #113, #120 and #122 in the catalogue and the three volumes. Decided during the release: #73's last step on the figures of steps 21b and 21c (the engine does not link Typst; `export` writes `.typ`), which leaves #172 for the Zemp release that builds the export of a script with its output; #150, option A, taken on the recommendation at step 13 (one form of the two-stage standard errors at 1.0, the N − k of `reg`); #173, option A now and C after 1.0. The owner decided on 20 November 2026 to skip the reader test of the testing handbook (section 2.4, item 4) at this release; the handbook is unchanged and the test is owed at 0.9.0. Due at 0.8.0 and still the owner's: #18, which was to be settled by what the alpha's users ask for; the alpha has not yet been in a user's hands, so the rule written into the entry keeps difference-in-differences first until he says otherwise. #74 and #75 are the alpha's and are settled in the Zemp Lite repository at its release 0.2. Due at 0.9.0, each taken as recommended under the standing instruction of 12 November 2026 unless the owner says otherwise, and put to him at "Start release 0.9.0": #85, #149, #153, #161 and #165 to #171; and, his own word, #151 (the licence and the journal's rule) and the organisation of #31. No normative text is changed.

**Status (20 November 2026, after the release of 0.8.0).** Three answers of the owner, given in the session that keeps the plan of releases and passed on to the Zeff session with his words. #18 is closed: "Yes, close it", as settled by the plan of #113, under which difference-in-differences (1.9) already comes before the latent-variable models (1.12); no change to the plan. #151 (the licence) and the organisation of #31 are moved from 0.9.0 to 0.11.0: "I want to skip #151 and #31 now and want to move it to 0.11.0". For #31 the launcher of 0.9.0 is to trust addresses under the decided domain only; for #151 an outside contribution offered before 0.11.0 brings the question back to him first. Each entry carries its paragraph. No normative text is changed.

**Status (20 November 2026, the notes of Zemp Lite entered).** On the owner's word ("Yes, set it up", to the question whether the register should be brought up to date before Zeff 0.9.0 and Zemp Lite 0.2 are started), the notes the Zemp Lite sessions wrote between 15 and 20 November 2026 are entered as #174 to #197, in a new group "Raised from Zemp Lite". One records a ruling he has made: #174, the margin editor adopted on 18 November 2026 as the editor of Zemp Lite from the 0.2 alpha ("Yes, adopt it"), with what was ruled the same day. The others are proposals, each his to decide, with the deadline its note proposed: what the editor asks of the engine (#175) and leaves open (#176); the proposals from his review of notebook features (#177 to #182); finding a fault (#183 to #187); comparing fits in the engine (#188, #189); five refinements of the diagram plan of #121 (#190 to #194); `margins` through a transformed term and two messages an R user meets (#195, #196); and the plots of a run handed to an interface (#197). Four entries gain a paragraph: #76 (how **Run to here** is read in the margin editor), #146 (the amendment covers the paragraph "The look" of section 5), #158 (two findings from the shipped data) and #164 (the names of a `poly` term's columns, and the rows and weights its basis is orthogonal on). Due at the opening of Zeff 0.9.0 among them: #175 in part, #178, #180, #184, #185, #186, #195, #196 and #197; due at the Zemp 0.2 alpha: #176 in part, #177, #181, #183 and #187. Nothing is settled here that the owner must decide, and no normative text is changed.

**Status (21 November 2026, at the opening of release 0.9.0).** The owner said "go" to the checklist of `docs/releases/0.9.0.md` without changing its seven answers, so each recommendation is taken under his standing instruction of 12 November 2026 (asked again as question 1 and carried over), his to overturn. Decided: #85, #149, #153, #161, #165 (the name `n_chars`), #166, #167 (the name `row_number`), #168, #169, #170 and #171 (each layout as recommended), each option A or as its table recommends, built at steps 29 to 37; from Zemp Lite, #175 (C5, C6 and C22 built here, C16 with #197, C23 to C27 placed at 0.11.0), #178 (with the comment `// # Title`), #180, #181, #184, #185, #186, #195 and #196, each option A, and #197, option A, the owner's priority; the three definitions of `margins` left by #88 for this opening, written into #88. #112, decided on 11 November 2026, is applied to the volumes at step 2. Not due here: #151 and the organisation of #31 (0.11.0, moved by the owner on 20 November 2026); the launcher trusts only addresses under `zeff-lang.dev`, so the organisation does not matter to it (`docs/decisions.md`, 20 November 2026). Also taken on the recommendation: `marginaleffects`, `rstatix` and `pROC` for the R reference library, which a session cannot install (the references are written in base R until the owner installs them); nothing hosted on `zeff-lang.dev` before 1.0, engines installed at 0.9.x from a downloaded archive or a mirror. No normative text is changed at the opening.

**Status (21 November 2026, the owner's answers from Zemp Lite 0.2).** At step 1 of Zemp Lite's release 0.2 the owner answered the seventeen questions of its checklist, "Go, as recommended"; the Zemp Lite session, which may not edit this repository, wrote them up in the shared notes, and they are entered here from the Zeff session: #27 (its server part closed until the paid tiers), #37 (open until the trademark applications are filed), #74 and #75 (option A, in the margin editor's form), #76 (the reading of 20 November confirmed), #164 (the addendum's two points), #176 (R2, R8, R9), #177, #178 (the form of the comment), #181, #183, #184 to #186 (option A each, placed at 0.11.0 by the opening of Zeff 0.9.0), #187, #195 and #196 (confirmed). Each entry carries a dated paragraph. Also recorded from the same answer: the reader test is one test for Zemp 0.2 and Zeff 0.9.0, run on the hosted alpha with five to eight readers the owner chooses (step 45 of Zeff 0.9.0); and the alpha's example is the margin editor's `wages` script with Zeff's sample `instruments` on `staff`. The amendments of the Zemp Specification that follow from #146, #174 and these answers (sections 2, 5, 7, 8, 15 and 17) and of section 12 of the Engine Specification (R9) are made with them and logged in `docs/spec_changes.md`.

**Status (21 November 2026, release 0.9.0, step 7b).** #198 is new: the small-sample factor of robust and clustered errors on a `glm` fit, where R's `sandwich` left to itself, Stata, and the variance layer differ. Taken as recommended (A, the variance layer's factors for every model) under the owner's standing instruction of 12 November 2026, his to overturn before 1.0.

**Status (21 November 2026, release 0.9.0, step 7c).** #199 is new: what `weight_type: analytic` means on a binary model. Taken as recommended (A, refused) under the owner's standing instruction, his to overturn before 1.0.

**Status (21 November 2026, release 0.9.0, step 8a).** #200 is new: what `predict` and `residuals` give a `glm` fit when `type:` is left out, where R and Stata differ. Taken as recommended (A, the scale of the outcome) under the owner's standing instruction, his to overturn before 1.0.

**Status (21 November 2026, release 0.9.0, step 8c).** #201 is new, in two parts: what the `R²` row of `report` shows for a `glm` fit (McFadden's, marked; taken as recommended under the standing instruction), and which standardization `summary(standardized: true)` gives a `glm` fit (refused until the owner chooses; open, deadline 1.0).

**Status (21 November 2026, release 0.9.0, step 11a).** #202 is new: five things the three definitions of `margins` in #88 leave unsaid, met when it was built for `reg`. Each is taken as recommended under the owner's standing instruction of 12 November 2026 and is his to overturn before 1.0: the average over the rows is weighted by the fit's weights; with no variable named `margins` gives the adjusted prediction (built at step 11b); after `reg` the p-values and intervals are from Student's t on the fit's degrees of freedom; the printed table, shown to him at this step; under `type: mem` a category is held at the shares of its levels (built at step 11b). Rows Z0414 and Z0417 of section 11.1 of the Language Reference are extended to `margins` (`docs/spec_changes.md`).

**Status (22 November 2026, release 0.9.0, step 15).** #205 is new: the printed form of `confusion` and how a script reads its shares, taken as recommended under the owner's instruction of 22 November 2026 and his to overturn before 1.0.

**Status (22 November 2026, release 0.9.0, step 14).** #204 is new: three things #112 leaves unsaid of `curve`, met when it was built, each taken as recommended under the owner's instruction of 22 November 2026 ("immediately start next step when a step is completed. I am approving and also take your recommended decision if necessary") and his to overturn before 1.0: `type:` is required; the result is read by `statistic()` (the area), `estimates()` (the points) and `plot()` (the figure); under weights a row counts as its weight.

**Status (22 November 2026, a ruling of the owner's from Zemp Lite).** #203 is new and decided: on 21 November 2026, trying the hosted alpha, the owner proposed one layout of three panes for every place of Zemp Lite in the place of the margin editor's two layouts, approved a mockup drawn twice to his corrections ("looks fine") and placed it in a release of its own: "Yes, record it as 0.2.1 and reader test, your checks by hand etc will move to 0.2.1". The Zemp Lite session, which may not edit this repository, wrote it up in the shared notes; asked in the Zeff session on 22 November 2026 whether to enter it, the owner answered "Yes, enter it", and it is entered here, his words of the 21st as that note gives them. It revises #174 (the margin becomes the Notes pane; a run unfolds every output) and #176, R2 (Panes is removed as a layout), and brings #178's outline and **Run section** to Zemp Lite 0.2.1 in the interface's own reading of the comment. With it the reader test (one test for Zemp and Zeff 0.9.0; step 45 of Zeff 0.9.0) and the acceptance checks left for the hand at Zemp Lite 0.2 move to the hosted 0.2.1; the test's scripts are unchanged. Section 15 of the Zemp Specification gains the row of 0.2.1, and sections 5, 7 and 8 a sentence each (`docs/spec_changes.md`).

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
| 8 | Zeff-source packages before 1.11 | Yes, in 1.6 with the manifest and lockfile; components at 1.11 | decided 11 November 2026: none before 2.0.0 (#123) |
| 9 | Registry hosting | GitHub Pages and Releases with a signed index, mirrored on Codeberg | when 2.0.0 is planned (#123) |
| 10 | `synthesize` | A package through the registry, not the engine | when 2.0.0 is planned (#123) |
| 11 | Split the specification | Yes, now, before coding starts | now |
| 12 | `posthoc` and factorial `anova(type:)` | Adopt, 0.7.0 | decided 16 October 2026 |
| 13 | `iterate` | Adopt, 0.5.0 | decided 30 October 2026 |
| 14 | `event()` | Adopt, 0.6.0 | decided 1 November 2026 |
| 15 | Power functions | Defer to 1.9 as one Design release | decided 16 October 2026 |
| 16 | `survey()` in 1.0 | Defer to 1.9 | decided 16 October 2026 |
| 17 | Panels (0.6.0) before tests and plots (0.7.0) | Keep the order | decided 30 October 2026 |
| 18 | Swap 1.2 DiD and 1.3 SEM | Decide at the 0.8.0 alpha from what its users ask for; DiD first otherwise | decided 20 November 2026 (closed as settled by #113; no change to the plan of releases) |
| 19 | Design methods for 1.9 | Add designed experiments, repeated-measures ANOVA and TOST to the catalogue now | now |
| 20 | Bayesian estimation | `prior:` arguments taking `Distribution` values; `reg`, `logistic`, `mixed` first | 1.9 |
| 21 | Licence | Apache 2.0 (decided 16 October 2026) | decided |
| 22 | `iv.check` | Adopt, 0.10.0 | 0.10.0 |
| 23 | Code signing (for setup files of the Zemp desktop apps only; the command line and `zeff app install` need none) | Free Windows signing for open source plus the Apple Developer Program | 0.11.0, with #38 |
| 24 | Zemp interface framework | Svelte 5, after a two-week spike against Solid at 0.6.0 | decided 11 November 2026 (Svelte 5, without the spike) |
| 25 | Zemp licence | Apache 2.0, as Zeff; a commercial path through services, not the licence. The business model assumes open core instead: decide the two together | settled by #125 (12 November 2026), and again by #148 (15 November 2026), which replaces it: Lite public under Apache 2.0, Studio private and proprietary |
| 26 | Start Zemp at 0.7.0 | Keep; the framework spike starts at 0.6.0 | decided 1 November 2026 |
| 27 | Hosting Zemp web; sync; a Zemp server | Hosting ruled 13 November 2026: Cloudflare Pages for Zemp web at `lite.zeff-lang.dev`, the website on GitHub Pages at `zeff-lang.dev`. Sync through the user's own folder sync, no server. The business model's paid sync, server compute and self-hosting need a server and a change to Zemp section 13: decide the two together | the server part decided 21 November 2026, as recommended (the owner's "Go, as recommended" in the Zemp Lite session): no Zemp server before 1.0; returns with the paid tiers (#35, #36) |
| 28 | AI assistance model | User's own key to a remote model, a local model as the private option, both off by default; whether Zemp also sells metered access of its own (the business model's Pro tier) is part of the same decision | Zemp 1.11 |
| 29 | Zemp version coupling | Follow Zeff's minors until 1.11 | 1.0 |
| 30 | Performance targets | Put the two stress cases in the benchmark suite now; set targets at first measurement, tighten each release | decided 23 October 2026 |
| 31 | Names: domain and GitHub organisation | The domain `zeff-lang.dev` (decided 6 November 2026); the organisation `zeff-lang` as recommended, still to be confirmed | organisation by 0.11.0 (moved from 0.9.0 by the owner on 20 November 2026) |
| 32 | A file of several frames and fitted results | A zip of Arrow files and JSON result descriptions, at 1.14 with the heavy models (1.5 before the plan of #113); never a binary format of Zeff's own | 1.14 |
| 33 | Interface revamp: density, icons, desktop menus, tablet and phone (the dark theme was decided on 17 October 2026) | As zemp_gui_revamp.md section 4 recommends for the four: one density with a larger-type setting; no icons; native menus mirroring the palette; no tablet or phone in 1.0 | decided 11 November 2026 (as recommended) |
| 34 | Long-term-support releases for institutions | None: exact pins and engines downloadable for ever already keep results stable; sell support, not a release line | Zemp Institution tier |
| 35 | Pricing levels and AI usage quotas | Set from the first year of running costs, not before Pro exists | Zemp Pro |
| 36 | Student verification for free Pro | Institutional email, with a manual route for institutions without one | Zemp Pro |
| 37 | Trademark filing for Zeff and Zemp | File in the home jurisdiction before the public alpha; Apache 2.0 gives others no right to the names | decided 21 November 2026, as recommended (the owner's "Go, as recommended" in the Zemp Lite session): the alpha built and hosted, its address announced once applications for "Zeff" and "Zemp" are filed in the owner's home jurisdiction; open until he says they are filed |
| 38 | Setup files for Zeff, Zemp Lite and Zemp Studio | None until #23 is decided: `zeff app install` and the package managers cover the desktop apps; plan setup files with #23 | 0.11.0 |
| 39 | Zemp Lite: paid features and AI in Lite; the web address; a desktop preview before 0.5 | Free-tier only through 1.0, revisited with #27 and #28; Lite web at a path on Zemp Studio web's origin; no earlier desktop preview | decided 11 November 2026 (as recommended) |
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
| 51 | Reading files in parallel by chunk | B: chunked parsing with boundaries fixed by the bytes alone, in the reader rewrite of 0.11.0 at the latest | decided 1 November 2026 |
| 52 | A benchmark suite run at every release | B: a small suite from 0.4.0 as a new section of the testing handbook, reporting rather than failing | decided 23 October 2026 |
| 53 | Options rather than functions: HAC standard errors and weights on the descriptives | The `se:` variants with the panel models of 1.4; `weights:` on the descriptives at 1.1 as frequency weights | 1.1 |
| 54 | A second range of variables in one formula | A: the grammar stands; a second range is written in brackets, `(z1..z3)`, which the checker reads as the range | decided 1 November 2026 |
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
| 67 | The default `level:` of `ci()` on a fit made at another level | The fit's level, so that `ci()` agrees with the summary; `level:` written overrides it | decided 1 November 2026 |
| 68 | A parameter of several types in a Zeff-written built-in | Allow the registry's `A \| B` in the `fn` line of a built-in's file, checked once per type, so that `fill_missing` and `n_missing` can be rewritten in Zeff | decided 6 November 2026 |
| 69 | Access to the enclosing frame from a Zeff-written built-in | An implicit `Frame` parameter the checker supplies to a built-in that declares it, so that `row_mean` and `row_sum` can be rewritten in Zeff | decided 6 November 2026; applied 7 November 2026 (0.7.0, step 3.3) |
| 70 | A note or a coded warning from a Zeff-written built-in | A `note(message)` function beside `warn`, a `code:` option on both, and the name of a `Var` as text (`name_of(v)`), so that `drop_missing` can be rewritten in Zeff with its note and `skewness` can raise W0205 | decided 6 November 2026 |
| 71 | The fitted design through the result interface | A `design()` method giving the kept design as a frame of the rows used, so that `vif` and later diagnostics can be written in Zeff | decided 6 November 2026 |
| 72 | A table constructor for Zeff-written functions | `table(title, columns = [...])` in `core`, with `format` for numbers, so that `summary` can be rewritten in Zeff as section 14 planned | decided 1 November 2026; `summary` kept in Rust at 0.7.0, step 4.5 |
| 73 | Typst: the typesetting engine of exports and of the report document, with a live preview | Adopt, after the licence, identity and bundle checks: a spike at 0.7.0 beside the plot renderer; `.typ` exports and Lite's export of script with output as PDF at 0.8.0; the report document a typst document with executed Zeff chunks | decided 15 November 2026 (option A in steps, on the recommendation); `.typ` written by `export` since 0.8.0 (step 21a); the library measured at steps 21b and 21c and, on 19 November 2026, **not linked into the engine**: the bytes agree on every target, but 14 of its 48 sets of hyphenation patterns are not under a permissive licence and cannot be left out, and it adds 26 MB to the engine and 7.2 MB compressed to the browser, where the bundle's limit is 5 MB; what follows for Zemp is #172 |
| 74 | A Help pane in the Viewer | A fourth tab beside Data, Results and Plots that follows the cursor to the function's page and opens a code's explained page from the Log; in both editions | decided 21 November 2026, as recommended (the owner's "Go, as recommended" in the Zemp Lite session) (option A, in the margin editor's form); at the Zemp 0.2 alpha |
| 75 | The data viewer and the environment browser, made excellent | Frozen columns, a statistics header per column, search, a live row count under a filter; Objects as a tree with memory per object; Lite's viewer read-only | decided 21 November 2026, as recommended (the owner's "Go, as recommended" in the Zemp Lite session) (option A, in the Data place); at the Zemp 0.2 alpha |
| 76 | Traceback, run to here and the loop inspector | The full call chain of an error through module functions; run to a line; one row per loop pass and per `iterate` state from the run record; no breakpoints or stepping | decided 6 November 2026; its reading in the margin editor confirmed 21 November 2026 (R15) |
| 77 | Schema-aware completion and extract function | Completion from the schema pass (the columns of the frame at that point in the chain) at 0.7.0; extract function with inferred parameters, a page skeleton and the same-bytes check at 1.1 | decided 6 November 2026 |
| 79 | Arithmetic over lists, or a map, in a Zeff-written built-in | A `map(list, f)` over a function value, or `+`, `-`, `*` and `/` lifted over a list as they are over a column, so that `quantile_group` can build its probabilities i/n exactly and be rewritten in Zeff | decided 6 November 2026; applied 7 November 2026 (0.7.0, steps 3b and 4.4) |
| 78 | Git history, the diff of a script and a diff of results | History and script diffs in the Git panel, and a table of every output of two commits' run records, identical to the bit or traced to the statement that changed; merges and pull requests left to the system terminal | 1.1 |
| 80 | `each(...)`: one marker for "separately", and tests on several variables | Adopt the rule and write its text before 0.7.0 opens; build it with the tests at 0.7.0 only if that release's checklist has room, otherwise in the first release that has, the word kept under #84; the default adjustment and the default rows are the owner's | decided 6 November 2026 (option B); 0.11.0 (the word) |
| 81 | Model sets: `nested` and `each` terms in a formula, `RegFits`, and one frame of estimates for a set | Adopt for 1.0: `reg` at 0.8.0 and each other estimator in the release after it ships; the whole moves to a later minor release if 0.8.0 runs late; the words of the rows rule are the owner's | decided 15 November 2026 (option B of the checklist of 0.8.0, on the recommendation): the design adopted for 1.0, built after 1.0 as an addition unless the owner asks for it in 1.0 |
| 82 | What `size:` means for a table in `export` | Refuse `size:` on a table with a named diagnostic at 1.0; table sizes with an exact splitting rule follow as an addition, written with the typesetting engine of #73 | 0.11.0 |
| 83 | The sentence of section 3.7 on bare words, beside the option records of section 3.1 | Amend section 3.7 to say what the engine has done since 0.5.0: in a record literal, a bare name that a parameter accepts as a word is that word | decided 6 November 2026 |
| 84 | Reserving `each` and `peek` before `core` freezes | At 0.11.0, add whichever word has not shipped as a `core` function to the words reserved for later use (section 3.6); `peek` only if the owner wants `.peek()` | 0.11.0 |
| 85 | The Games–Howell comparison in `posthoc` | Add `games_howell` to `method:` in the first release after 0.7.0 with room; until then Tukey's HSD after Welch's ANOVA prints a note | decided 21 November 2026 (option A, on the recommendation); built at 0.9.0, step 29 |
| 86 | Folding, snippets and the statement ranges behind **Run** in the language server | At 0.8.0, with Zemp's alpha; formatting through the server is built at 0.7.0 | decided 15 November 2026 (option A); built at 0.8.0, step 8b |
| 87 | A variable name in an option record | Accept a `Vars` of exactly one variable where a parameter takes a `Var`, so that `{ se: robust, cluster: vars(firm) }` can be named once and spread | decided 15 November 2026 (option A); built at 0.8.0, step 4 |
| 205 | `confusion`: the printed table and how its shares are read | Recommended A: the observed outcome in the rows and the predicted in the columns, with totals; five shares under it as proportions; `estimates()` gives them and `statistic(name)` one; a row at the cutoff is predicted 1; rows counted by their weights | decided at 0.9.0, step 15, under the standing instruction (A); the owner's to overturn before 1.0 |
| 206 | What `zeff audit` reads before run records have their schema | Recommended A: from 0.9.0 `zeff run` keeps a small record of each run in `.zeff/runs/` (the engine, the day, the script, the built-in functions it calls with the options written), in a provisional form that the versioned schema of 0.11.0 replaces; `zeff audit` reads it, and is answered by the newest engine installed, whose list knows the most | decided at 0.9.0, step 22, under the standing instruction (A); the owner's to overturn before 0.11.0, when the schema is fixed |
| 207 | Which machines the archives of 0.9.0 are built for | Recommended A: the three that continuous integration builds and proves on every push, Linux x86-64 and ARM64 (static) and macOS on Apple silicon; Windows and Intel Macs when the tests run there, by 1.0.0 with the installers | decided at 0.9.0, step 23, under the standing instruction (A); the owner's to overturn at any release |
| 208 | Zemp Lite 0.2.2: a project saved to the computer, and the actions on files, folders and projects | **Export project…** as a zip and the opening of one, persistent storage and the reminder to export (the parts of #117 that need no run record), and the actions a project lacked on screen: a script made, files and folders added, renamed, moved, duplicated, deleted and saved, a project renamed, duplicated and deleted; **Open folder…** placed with the 0.5 desktop beta | decided by the owner on 22 November 2026 ("Yes, into 0.2.2"; "Add file actions, folder actions, project actions"), entered on his word in the Zeff session the same day |
| 209 | What the Variables pane shows for each kind of variable, beyond a number, a category and a date (Zemp) | Recommended A: long text shown by its length (a histogram of `n_chars`), its empty and missing rows, five values in full and the values that repeat; text of few values counted as a category is, with a line naming `category()`; a variable whose every value differs said to look like an identifier, with no chart; an ordered category in its levels' order with the cumulative share and the median level; a number on an interval scale as any number, with its unit; an opaque column counted and its first values shown as printed; every figure the engine's | open; raised by the owner on 23 November 2026; due at the plan of Zemp 0.3, on Zeff 0.9.0 |
| 210 | Symbolic data: interval-, set-, histogram- and distribution-valued variables (Billard and Diday) | Recommended A: a package after 1.0 whose column types carry symbolic values through the interface of section 4.3, rule 1 (intervals, sets of numbers and of categories, histograms, quantiles and distributions, modal categories), made first by aggregating classical rows, with its descriptive statistics, regression, clustering and principal components; nothing before 1.0 | open; raised by the owner on 23 November 2026; due when the roadmap after 1.0 is drawn, at 1.0.0 |
| 211 | The Help place redesigned: a home page of methods, a page for each method, moving about, and search as one types (Zemp) | Recommended A: in two stages. Stage 1 (Zemp alone, on the engine it has): **Home**, **Back** and **Forward** with a trail of where one is, the left pane listing the pages around the one open, and search that suggests from the second character (names, titles, R and Stata names, codes). Stage 2 (the engine and Zemp): a methods index held in the engine, each method with a few sentences, a minimal example on a shipped dataset and the output the engine printed, checked at every build; the home page lists the methods by area, as a statistics package's features page does | the design decided on 23 November 2026 (the owner, on the mockup: "Yes as drawn, except …"): Home is the **Index of methods**, the left pane **Topics** with five sections (methods, the language, packages, errors and warnings, using Zemp Lite), a page for each package, search in the bar's field; stage 1 placed at Zemp Lite 0.2.3, on Zeff 0.8.0, by the owner on 23 November 2026 ("Yes, enter 0.2.3"), with Home an **Index of functions** until the methods arrive; stage 2 recommended at Zeff 0.10.0, shown by Zemp 0.4, to be put to the owner at the opening of Zeff 0.10.0; the examples run in the page with a **Run** button from Zemp 0.3, by the owner on 23 November 2026 ("record it for 3.0"), each example running by itself when its page opens as recommended; Home a landing page of soft panels (Zeff, Zemp Lite, then the topics), the Index of functions removed, by the owner on 24 November 2026 (item 14); from Zemp Lite 0.2.3, by the owner's standing rule of 24 November 2026, every release's Help agrees with the engine it carries: the code of Zemp's own pages run at the build, the names they use checked against that engine, the pictures made again; at 0.2.3, by the owner on 24 November 2026, **On this page** the right pane in place of **Related functions**, and the search field at the right end of the bar (item 13) |
| 212 | A small estimate in a fit's own table | Recommended A: the rule #171 gave `report` in every table of estimates: two significant digits where the table's decimals would print a number that is not zero as zeros, its standard error and interval to the same decimals; B: as now, and the page's advice to rescale the variable; C: a note under the table naming the terms below its precision | open; deadline 0.10.0 |
| 213 | R's names for Zeff's functions, `factor(x)` in a formula first | Recommended A: where a call names no function and the name is the R or Stata equivalent the catalogue records for one, Z0103 names the Zeff function (`factor` is `category`); B: `factor` alone, inside a formula; C: as now | open; deadline 0.10.0 |
| 214 | NIST's Filip problem and the tolerance of the collinearity rule | Recommended A: as now at 1.0, the tolerance of section 7.2 fixed at 1e-7 as R's default is, Filip a known issue said in the notes and the article; B: an option of `reg` to lower it, after 1.0; C: a lower fixed tolerance | open; deadline 0.10.0 |
| 215 | A name with two functions in the help: `histogram`, `boxplot`, `density`, `plot`, `min`, `max`, `append`, `quantile`, `summary` | **Decided (24 November 2026, the owner: "Yes, option A")**: one page for each function, told apart by what it is called on ("histogram · of a frame", "histogram · of a plot"); the bare name opens a short chooser of the two; search, Topics and a package's page list the two apart; hover and F1 unchanged (already the one in use); in Zemp Lite 0.2.3 for the Help place, and in the engine at 0.10.0 for `zeff help` | Zemp Lite 0.2.3; Zeff 0.10.0 |
| 204 | `curve`: `type:` required, how the result is read, weights | Recommended A for each: `type:` has no default; the area by `statistic()`, the points by `estimates()`, the figure by `plot()`; rows counted by their weights, frequency or sampling | decided at 0.9.0, step 14, under the standing instruction (A); the owner's to overturn before 1.0 |
| 203 | One layout of three panes on every page (Zemp Lite) | Every place (Script, Data, Plots, Help) has a left pane, the place in the middle and a right pane, each pane opened and closed by its own link; the Panes layout, the Margin link and the Objects panel are removed; Run unfolds every output; revises #174 and #176 (R2) | decided 21 November 2026 (the owner's ruling, given in the Zemp Lite session, passed on in the shared notes and confirmed by him in the Zeff session on 22 November 2026); Zemp Lite 0.2.1, on Zeff 0.8.0 |
| 202 | `margins`: the average under weights, the call with no variable, the reference distribution, the printed table, a category under `type: mem` | Recommended A for each: the average weighted by the fit's weights; with no variable named, the adjusted prediction; Student's t on the fit's degrees of freedom after `reg`; the table of step 11a; a category held at the shares of its levels | (a), (c) and (d) decided at 0.9.0, step 11a, under the standing instruction (A), (d) shown to the owner; (b) and (e) built at step 11b (22 November 2026); the owner's to overturn before 1.0 |
| 201 | A `glm` fit in `report` and under `summary(standardized:)` | Recommended A for each: the `R²` row of `report` shows McFadden's R² marked `(McFadden)`; `standardized: true` is refused on a `glm` fit until a standardization is chosen | the first decided at 0.9.0, step 8c, under the standing instruction (A); the second open, deadline 1.0 |
| 200 | What `predict` and `residuals` give a `glm` fit unless told | Recommended A: the scale of the outcome for both, `type: response`: a probability from `predict`, the outcome less the fitted probability from `residuals`; R's defaults (`link`, `deviance`) one word away | decided at 0.9.0, step 8a, under the standing instruction (A); the owner's to overturn before 1.0 |
| 199 | Analytic weights on a binary model | Recommended A: `weight_type: analytic` is refused on `glm` with `family: binomial` (Z0410), as Stata refuses `aweight` on `logit`; frequency and sampling weights are taken | decided at 0.9.0, step 7c, under the standing instruction (A); the owner's to overturn before 1.0 |
| 198 | The small-sample factor of robust and clustered errors on a likelihood fit | Recommended A: the factors of the variance layer for every model, HC1 N/(N − k) and CR1 G/(G − 1) · (N − 1)/(N − k), as section 8.3, point 2, of the Engine Specification says | decided at 0.9.0, step 7b, under the standing instruction (A); the owner's to overturn before 1.0 |
| 197 | The plots of a run handed to an interface | Recommended A: an engine step at 0.9.0 by which the browser build keeps the plots of a run, lays one out at a size and theme without a new run, and hands over the scene (C16 of #175) | decided 21 November 2026 (option A, the owner's priority as recommended); built at 0.9.0, step 25 |
| 196 | Two messages R users meet: `I()` and `poly` in a formula, and a function of a later release | Recommended A: inside a formula, the help of Z0103 for `I(...)` shows the term without it and for `poly(x, k)` writes the powers out; a catalogued name of a later release says its release | decided 21 November 2026 (option A, on the recommendation); built at 0.9.0, step 35 |
| 195 | Slopes and turning points: `margins` through a transformed term | Recommended A: `margins` differentiates through every term that is a function of the variable asked for (`exper` and `exper^2` today; `poly` and `spline` when they arrive) | decided 21 November 2026 (option A, on the recommendation); built at 0.9.0, steps 10 to 12 |
| 194 | Statistical guard rails, written into the diagram rules | Adopt rule 1 (no edge suggested by the data, on a causal graph or an SEM diagram) and rule 5 (stars only beside an estimate in `path_plot`); rules 2 to 4 stay where they are | open; deadline 0.10.0 for rule 1, 1.12 for rule 5 |
| 193 | Dragged positions in Zemp Lite: Copy as Zeff | Recommended A: **Copy as Zeff** (#114) extends to dragged positions in every diagram viewer, as `layout:` or `at:` on the clipboard | open; deadline: Zemp 0.4, with Zeff 0.10.0 |
| 192 | Testing layouts against known figures | Recommended A, or B if nothing EPL is wanted even in tooling: a suite of reference figures measured on every change, Graphviz's `dot` a yardstick in the tests alone; the licence question is the owner's | open; the owner's word on Graphviz; deadline 0.10.0 |
| 191 | One graph form under every diagram | Recommended A: every diagram lowered to one graph form (nodes, edges, groups, ports, stable names) that layout, the viewer, the table view and the exports read; fixed at 0.10.0 | open; deadline 0.10.0 |
| 190 | Building the layout engine in stages | Recommended A: the layout engine of #121 in three stages, each with the release that first needs it (0.10.0 for causal graphs, 1.2 for flowcharts, 1.12 for SEM) | open; deadline 0.10.0 |
| 189 | A coefficient plot of several fits | Recommended A: `coef_plot([fits])` in `plot`, one row per term with each fit's estimate and interval; fits of different outcomes or estimators refused; the name before 1.0 | open; deadline 0.10.0 |
| 188 | `report()` says when its columns cannot be compared | Recommended A: a note in the table's footer for each column that differs from the one before it in outcome, estimator or rows; nothing when the columns agree | open; deadline 0.10.0 |
| 187 | Where the time and memory of a run went | Recommended A: two lists in the run record, the statements by their time and the bindings by their memory, read from the engine (C26) | decided 21 November 2026, as recommended (the owner's "Go, as recommended" in the Zemp Lite session) (option A, R19); at the Zemp 0.2 alpha, the interface's own timing until the engine reports each statement's |
| 186 | The console line inside a pass or a call | Recommended A, with #185: while a pass or a call is open, the console line runs in its names and can change nothing; needs C25 | decided 21 November 2026 (option A); C25 placed at 0.11.0 |
| 185 | Opening a pass or a call, by replay | Recommended A: the engine runs one call or one pass again from the arguments the run gave it, bit for bit, and shows each `let` of the body for that call; no breakpoints (#76); needs C24 | decided 21 November 2026 (option A); C24 placed at 0.11.0 |
| 184 | Searching the passes of a block, and drawing a field across them | Recommended A: find the first pass that failed, warned or kept no rows; for `iterate`, each field's change and one field drawn against the pass; needs C23 and C27 of #175 | decided 21 November 2026 (option A); C23 and C27 placed at 0.11.0 |
| 183 | The error's trail in the margin | Recommended A: the failing statement's output lists each link of the chain as a link, each file opening with a note on the lines the chain names; nothing run again | decided 21 November 2026, as recommended (the owner's "Go, as recommended" in the Zemp Lite session) (option A, R14); at the Zemp 0.2 alpha, the pass added with C22 at 0.9.0 |
| 182 | Small conveniences, none before 1.0 | Keep as roadmap, each weighed after Zemp Lite 1.0: moving a statement, hiding one output, a heading as formatted text, a point's values on hover, the time a statement last ran | open; deadline 1.1 |
| 181 | Run from here to the end | Recommended A, as an amendment of #76: **Run from here** as a command and a gutter action beside **Run to here** | decided 21 November 2026 (option A, as an amendment of #76); a command of Zemp, at the Zemp 0.2 alpha |
| 180 | Run from a fresh session | Recommended A: a ⌘K command that drops the console's statements and runs the script from its first line, the cache serving unchanged statements | decided 21 November 2026 (option A); a command of Zemp, no engine step |
| 179 | The last run's output shown when a project is opened | Recommended A: on opening, the last run's output from the run record and the cache, each block labelled with when it ran, the stale rules applying at once | open; deadline 0.11.0 |
| 178 | Sections in a script | Recommended A: a comment written as a heading (`// # Data`) starts a section, reported by the language server as outline and folding, with **Run section**; the form of the comment is the owner's choice | decided 21 November 2026 (option A, the comment `// # Title`, on the recommendation); built at 0.9.0, step 28 |
| 177 | Zemp Lite is not a notebook: the statement is the cell | Recommended A: no cells; the script stays one plain text file and the statement is the unit that is run, shown, folded and marked stale | decided 21 November 2026, as recommended (the owner's "Go, as recommended" in the Zemp Lite session) (option A) |
| 176 | Six questions the margin editor leaves open | Recommended per item: Panes kept one release (R2); the keys as the document has them (R8); ⌘↵ runs the statement with the stale statements it reads (R9); tables as cells when the engine can (R10); the HTML article until the PDF (R12); a paper's chunks in its own session (R13) | R2, R8 and R9 decided 21 November 2026, as recommended (the owner's "Go, as recommended" in the Zemp Lite session); R10 with C7; R12 and R13 with the documents of #141; R2 revised on 21 November 2026 by #203: Panes is removed as a layout at Zemp Lite 0.2.1 |
| 175 | What the margin editor asks of the engine: the contract, C1 to C27 | Recommended A: each of the 27 items placed at an engine release as a step with its tests; C5 and C6 (a result's type and dimensions, its print at a width) first, for the 0.2 alpha | decided 21 November 2026 (option A): C5, C6 and C22 built at 0.9.0 (steps 26 and 27), C16 with #197 (step 25), C23 to C27 placed at 0.11.0, the rest at the releases they rest on |
| 174 | The editor of Zemp Lite: the margin editor | The margin editor replaces the Inline layout of section 5 as the editor of Zemp Lite, from the 0.2 alpha: a note beside every statement, the full result unfolded under it at the reader's asking; places Script, Data, Plots and Paper; the quiet look | decided 18 November 2026 (option A; the owner's ruling, recorded from the Zemp Lite repository); built from the Zemp 0.2 alpha; revised on 21 November 2026 by #203 (its margin becomes the Notes pane of one layout of three panes, from Zemp Lite 0.2.1) |
| 173 | Probabilities below the smallest double: a log scale for the tails | Recommended A now and C after 1.0: the limit is stated on the page of `dist.truncated` and in the known issues; `log: true` on `pdf` and `cdf`, used inside a truncation, is an addition placed in the plan of releases after 1.0 | decided 20 November 2026 (A, on the recommendation under the standing instruction); C open, deadline: the plan of releases after 1.0 |
| 172 | Typst inside Zemp, for the PDF of a script with its output | Recommended A: a module of its own, fetched the first time a PDF is asked for, built with the hyphenation patterns under permissive licences only; until then the `.typ` export and the Typst the user has installed | open; deadline: the Zemp release that builds **Export script with output** |
| 171 | Six printed layouts the review found | Recommended A for each: small coefficients in `report` by significant digits, sign kept; `labels:` applied to the parts of an interaction; thousands separators in every df column; facet titles "kids = 0"; long group names wrapped in the proportion test's hypotheses; the standard error of a t-test's difference in its estimates | decided 21 November 2026 (each layout as recommended); built at 0.9.0, step 34 |
| 170 | Six smaller refusals with workarounds | Recommended per item: `equal` on columns; `category` on a list; a count from `duplicates`; `attempt` of a call to `error`; `map` over a list holding a failed value; `chi2_test(x, p:, levels:)` naming categories never observed | decided 21 November 2026 (each item as recommended); built at 0.9.0, step 33 |
| 169 | Widening a guessed Int to Float on import | Recommended A: a later value with a decimal widens the column to Float with a note; only text in a numeric column stops the import | decided 21 November 2026 (option A, on the recommendation); built at 0.9.0, step 32 |
| 168 | One value out of a column | Recommended A: `first`, `last` and `x[i]` take a column, and `to(column, List)` gives a List | decided 21 November 2026 (option A, on the recommendation); built at 0.9.0, step 32 |
| 167 | A row number | Recommended A: `row_number()` in `data`, within `by:` groups in `add`, as Stata's `_n` | decided 21 November 2026 (option A, the name `row_number`, on the recommendation); built at 0.9.0, step 31 |
| 166 | A row-wise count of missing values | Recommended A: `row(vars, stat: n_missing)`, an option of `row` | decided 21 November 2026 (option A, on the recommendation); built at 0.9.0, step 31 |
| 165 | The length of each text value in a column | Recommended A: a text function giving each value's length in code points, `n_chars(s)`, with a note when `len` of a column is spread over the rows of `add` | decided 21 November 2026 (option A, the name `n_chars`, on the recommendation); built at 0.9.0, step 31 |
| 164 | Orthogonal polynomials of a continuous variable in a formula | Recommended A: a formula term `poly(x, degree: Int, raw: Bool = false)` in `regress` at 1.2, orthogonal by default as R's `poly`, predictions on the fit's own basis; B: at 0.9.0; C: raw powers only | open; deadline 1.1; the addendum's points decided 21 November 2026 (point 1, option C; point 2, the rows the fit uses, unweighted) |
| 163 | Poisson regression with fixed effects, the wild cluster bootstrap and difference-in-differences before the plan of #113 places them | Recommended A: Poisson with fixed effects and the wild cluster bootstrap at 1.2, two-period and staggered difference-in-differences at 1.3, the releases of #113 from 1.2 on moved two places later; B: Poisson and the two-period design only; C: #113 stands | open; the owner's word; deadline 1.1 |
| 162 | A release 0.12.0 for the bridges to R and Python, before the names freeze | Recommended A: 0.12.0 "Bridges" with the R package, the Python wheel, the browser playground and the translation page (moved from 1.0.0 by #152 and #161); B: all of it in 1.0.0 | open; the owner's word; deadline 0.10.0 |
| 161 | Aids for the article's referees: a playground, a translation page, an archive and a citation file | Recommended A: a citation file and an archived copy of each release with its own identifier from 0.9.0; the playground with the article's examples and the R and Stata translation page, generated from the help pages, on the website at 1.0.0 | decided 21 November 2026 (option A, on the recommendation); the citation file at 0.9.0, step 37; the archive linked by the owner |
| 160 | `sample_flow` before 1.0 | Recommended B: the printed table of `data.sample_flow()` and `fit.sample_flow()` at 0.11.0, the drawing at 1.2 as planned | open; deadline 0.10.0 |
| 159 | Candidate instruments from a causal graph | Recommended A: `g.instruments(exposure:, outcome:)` at 0.10.0, the instruments and conditional instruments with their conditioning sets, held to dagitty | open; deadline 0.10.0 |
| 158 | Testing a causal graph's implied independencies against the data | Recommended A: an option `data:` on `g.implied_independencies()` at 0.10.0 that tests each one, held to dagitty's local tests | open; deadline 0.10.0 |
| 157 | Packages written in Zeff during 1.x, beside the ruling of #123 | Recommended B: Zeff-source packages at 1.6 as #8 once recommended, WebAssembly components at 2.0; A: #123 stands | open; the owner's word; deadline 1.1 |
| 156 | When the article is submitted | Recommended A: a preprint at 1.0.0 and the submission at 1.1; B: submit at 1.0.0; C: wait for a wider range of methods | open; the owner's word; deadline 1.0.0 |
| 155 | The article's case study and its replication project | Recommended A: one published study with openly licensed data replicated end to end, and the article itself a project that `zeff verify` re-runs within an hour; the study is the owner's choice | open; the study is the owner's word; deadline 0.11.0 |
| 154 | Independent checking of the numerical core | Recommended A: the owner reads the reference scripts of the estimators and runs the Stata references himself by 0.10.0; an outside audit of the kernels and the variance layer, published, before submission | open; deadline 1.0.0 |
| 153 | The NIST Statistical Reference Datasets in the validation suite | Recommended A: the linear-regression, univariate and analysis-of-variance problems at 0.9.0, accuracy as the log relative error, with R's and Stata's beside it | decided 21 November 2026 (option A, on the recommendation); built at 0.9.0, step 36 |
| 152 | A home in a standard repository: an R package on CRAN and a Python wheel on PyPI | Recommended A: both at 1.0.0, the R package running a script or project and returning frames and results | open; deadline 0.10.0 |
| 151 | The licence and the journal's rule of GPL compatibility | Recommended A: dual licence, MIT or Apache 2.0 at the user's choice, from 0.9.0, after a word with the editors | open; the owner's word; deadline 0.11.0 (moved from 0.9.0 by the owner on 20 November 2026) |
| 150 | The small-sample form of the two-stage standard errors, and Stata's large-sample default | Recommended A: one form at 1.0, the N − k of `reg` with the t table (R's `ivreg`, fixest, Stata's `ivregress 2sls, small`), the page saying how Stata's default differs; the large-sample form an option after 1.0. B: an option on `ivreg` now | decided 16 November 2026 (option A, on the recommendation under the owner's standing instruction); built at step 13 of 0.8.0 |
| 149 | A warning repeated word for word in the passes of a loop | Recommended A: said once with the count of passes, as a note is from step 8o of 0.8.0; B: every pass, as now | decided 21 November 2026 (option A, on the recommendation); built at 0.9.0, step 30 |
| 148 | Zemp Lite in a public repository under Apache 2.0; Zemp Studio in a private one | The owner's ruling: Zemp Lite is developed in the public repository `zemp-lite` under Apache 2.0, as Zeff; Zemp Studio, when its work starts, gets a separate private repository and a proprietary licence and builds on Lite; Lite never depends on Studio's modules | decided 15 November 2026 (owner's ruling); the Zemp Specification amended the same day; replaces the repository and licence parts of #125, and with them #25 |
| 147 | Inside a function's body, a parameter where a term of a fit is expected | A: inside a function's body a bound name that holds a `Var` or a `Str` is the term it names, one rule with #140; the help line of Z0414 names `name_of` until it is built | decided 15 November 2026 (option A); built at 0.8.0, step 5 |
| 146 | The look of the website and of Zemp Lite web | The owner's rulings of 13 and 14 November 2026: logo D4, the tagline, Atkinson Hyperlegible Next with JetBrains Mono and Newsreader, one type scale, a light/dark button and the D4 watermark on every page, violet buttons, realistic pages; Zemp Lite's Panes with a tabbed left panel, data tabbed with the script, output under the script, a modern data and variables viewer, and a code text size control | decided 14 November 2026 (owner's rulings); Zemp section 5 amended at Zemp 0.2 alpha |
| 145 | Where the missing count and the label of a variable come from, in completion and hover | A: from the shipped dataset, or from the last run, marked as such | decided 13 November 2026 (option A, taken on the recommendation under the owner's standing instruction, which he may overturn); built at step 35d of 0.7.0 |
| 144 | Reducing a file larger than memory before it is loaded | `where`, `select`, `add`, `aggregate`, `tabulate` and `save` on a `stream`, in one pass by row group, at 1.7; a step that needs every row at once refused with a message | decided 13 November 2026 (option A); built at 1.7 |
| 143 | How a step's memory is estimated before the step starts | A: three rules that lean high (a file five times its size on disk, a list built by a loop, the frames a statement reads once more) | decided 13 November 2026 (option A, taken on the recommendation under the owner's standing instruction, which he may overturn); built at step 33c of 0.7.0, rule 2 amended at step 38g |
| 142 | Figures of an HTML document in the light and the dark theme | `export(background: white | none = white)` for a transparent PNG, SVG or PDF; a dark form of each figure theme, `theme(mode: light | dark = light)`; `render` writes each figure of an HTML document twice, as PNG, light and dark, on no background, and the page shows the one its theme asks for | Decided 13 November 2026 (option A), with the HTML documents of #141 after 1.0 |
| 141 | A `publish` package: reports written in Markdown, with HTML of Quarto's quality and HTML slides | Markdown in Quarto's dialect, Typst underneath for PDF; HTML, HTML slides, PDF and Word from one document, each self-contained and identical on every machine; `publish` in tier 1 after `plot` with `render`, `zeff render`, `report()` moved from `results` and `cite()`; `pulldown-cmark`, a LaTeX-to-MathML converter, Hayagriva, reveal.js and Typst, no Pandoc; the package and its names at 0.10.0, the documents after 1.0, HTML first; `.zmd` | Markdown and the name `publish` ruled 13 November 2026; decided 13 November 2026 (option A, `.zmd`): `publish` at 0.10.0, the documents after 1.0 |
| 140 | Inside a function's body, which comes first: the function's own name or the frame's variable? | A: the function's own names come first inside its body; a script is unchanged | decided 13 November 2026 (option A, taken on the recommendation under the owner's standing instruction, which he may overturn); built at step 38c of 0.7.0 |
| 139 | How the "Warnings and notes" of a built-in written in Zeff is narrowed | A: the codes a body cannot raise are taken off its generated list | built 13 November 2026 (steps 32e and 32f of 0.7.0) |
| 138 | What `zeff test` runs, and what a failed `assert` does under it | A on the four questions: a script is a test when its text calls `assert`; a failed `assert` is kept and the run goes on | decided 13 November 2026 (option A, taken on the recommendation under the owner's standing instruction, which he may overturn); built at step 32a of 0.7.0 |
| 137 | The layout `zeff fmt` writes where section 3.5 is silent | The thirteen rulings of the entry, each the owner's to overturn before the 1.0 freeze of the style | built at step 31 of 0.7.0; open to the owner until 1.0 |
| 136 | `vif` in Zeff: what the language lacks to print it as it prints now | A for 0.7.0: `vif` stays in Rust; whether Zeff-written tables carry lines under their grids is the question for 0.8.0 | decided 13 November 2026 for 0.7.0 (option A, taken on the recommendation under the owner's standing instruction, which he may overturn); the rewrite and the lines under Zeff-written grids not before 1.0 (15 November 2026) |
| 135 | A function that gives citation details: `cite` | `cite()` for Zeff at the pinned engine version, `fit.cite()` for the methods and options a fit used, `cite(t_test)` for a function's references and `cite(wages)` for a dataset, with `format: text | bibtex | ris`; `zeff cite` for a project's whole bibliography; the references of the help pages kept as structured entries. At 0.10.0, the command at 0.11.0 | decided 13 November 2026 (option A, named `cite`); `cite` at 0.10.0, `zeff cite` at 0.11.0 |
| 134 | Making a manipulated analysis harder to hide in a replication package | Six measures: a stamp of the scripts before the data are read, for a preregistration; an optional run history whose records are chained by hash; the raw data checked against the checksums their archive publishes; signed and dated run records; specification curves; and each run record naming the Git commit it ran from, with any uncommitted changes. The fields of the run record at 0.11.0, the commands after 1.0 | 0.11.0 |
| 133 | `by:` on the shorthand plots | A: every shorthand takes `by:` and draws one panel a group | decided 13 November 2026 (option A, taken on the recommendation under the owner's standing instruction, which he may overturn); built at steps 29a to 29d of 0.7.0 |
| 132 | Polar coordinates: where the circle starts, which way it turns, and which variable is the angle | A for 0.7.0, as built (from the top, clockwise, `x` the angle); three arguments at 0.8.0 if the owner wants them | decided 13 November 2026 for 0.7.0 (option A, taken on the recommendation under the owner's standing instruction, which he may overturn); the arguments not before 1.0 (15 November 2026), placed with the grammar extensions of 1.2 |
| 131 | The titles of the legends of size, shape and fill | B at 0.8.0: three more arguments of `labels`, an addition that breaks no script | decided 12 November 2026 (taken on the recommendation under the owner's standing instruction, which he may overturn); built at 0.8.0, step 8a |
| 130 | The notes of a plot built in a chain: call by call, or for the plot the statement ends with | A for 0.7.0 (a note raised earlier in the chain is still printed); B at 0.8.0, in a step of its own | decided 12 November 2026 (taken on the recommendation under the owner's standing instruction, which he may overturn); B built at 0.8.0, step 8a |
| 129 | How an axis of dates chooses its breaks and writes them | A: ISO labels by default, with a pattern through `labels:` | decided 12 November 2026 (option A, taken on the recommendation under the owner's standing instruction, which he may overturn); built at steps 25c1 and 25c2 of 0.7.0 |
| 128 | Which variables make an axis of categories | C: a Str and a Category make one; a labelled number keeps a linear axis with its labels at the breaks | decided 12 November 2026 (option C, taken on the recommendation under the owner's standing instruction, which he may overturn); built at steps 24a and 25 of 0.7.0 |
| 127 | A name bound to a number, where an option takes a number or a variable | B: a bare name is a variable of the frame or, failing that, a bound value, with W0102 where both exist | decided 12 November 2026 (option B, taken on the recommendation under the owner's standing instruction, which he may overturn); built at step 23f of 0.7.0 |
| 126 | How the text of an exported SVG figure is written: as outlines, or as text with the font embedded | Outlines by default at 0.7.0, so that a figure looks the same in every program whether or not IBM Plex is installed, with the words kept in the file for search and for readers of the screen; text with the font embedded as an option of `export` after 1.0 | decided 12 November 2026 (outlines); built at step 21c of 0.7.0 |
| 125 | Zemp Lite only, with Zemp Studio later | Build and release Zemp Lite alone; start Zemp Studio after Zeff 2.0.0, or decide that at a review at the close of 1.5; Zemp in a private repository from its first commit, Lite free of charge; the Document settings dialog of #124 in Lite; the Studio menu plan of 12 November kept for when Studio starts | decided 12 November 2026 (as recommended); the Zemp Specification amended the same day; settles #25; its repository and licence parts replaced by #148 (15 November 2026) |
| 124 | The report document: a Typst file with a settings file, simple chunks, dialogs, and a Quarto-like default | The report is plain Typst text with a YAML settings file beside it, plain ``` chunks and `` `= value` `` inline values; Zemp Studio writes the settings through a nine-tab dialog and tables through a table builder, with no Insert menu; the default template looks like a Quarto PDF; nine enhancements (live numbers, notes from the fits, method citations and others); amends the report-document part of #73 | decided 12 November 2026 (option A without the table builder); built with the report document of #73 |
| 123 | No third-party packages before 2.0.0 | No package of either kind, no registry and no package commands before 2.0.0, the release that ships them set when 2.0.0 is planned; a project's own functions in `functions/` unaffected; how packages keep Zeff's principles (names, shared arguments, reporting, duplicates) settled then | decided 11 November 2026 |
| 122 | Placing the graphics and diagram proposals on the releases of #113 | The capabilities of the two proposals become the plan, each placed by the rules of #113 (basic, advanced, research; econometrics first) with its method's release where it is a result plot; the exact release confirmed when each release is planned; catalogue rows added in the documentation step of 0.8.0 | decided 11 November 2026 (option A); catalogue rows at the documentation step of 0.8.0; the graphics rows added at 0.8.0, step 8c2, and the diagram rows at step 8c3 (15 November 2026) |
| 121 | Diagrams: one layout engine, positions in the script for export, SEM path diagrams and the model builder | Zeff's own deterministic layered layout engine at 0.10.0 for `g.plot()` (no Graphviz or ELK); dragged positions are view state and exports use only positions the script holds; `m.plot()` and `fit.path_plot()` with SEM at 1.12; identification checks at check time; `diagram { }`, `sample_flow` and `reason:` at 1.2; `zeff graph` at 1.1; one canvas for every builder in Zemp | decided 11 November 2026 (option A); the positions rule written into the Language Reference, section 7.3, and the Zemp Specification, sections 5 and 6, the same day; the layout engine at 0.10.0; the rows after 1.0 added to the catalogue at 0.8.0, step 8c3 (15 November 2026) |
| 120 | Result plots: one `_plot` method per kind with a `kind:` option, and no further plots in 1.0 | Derived plots are `_plot` methods on results, one per kind of figure, each with a `kind:` option (`residual_plot(kind: fitted \| qq \| scale_location)`), as rule 6 and #97 together require; the proposal's four 1.0 candidates wait for 1.2, the ROC curve being `fit.curve(type: roc).plot()` (#112) | decided 11 November 2026 (option A); the convention fixed at step 30 of 0.7.0, the table of #113 renamed at 0.8.0; applied to the catalogue and to the owner's table at 0.8.0, step 8c2 (15 November 2026) |
| 119 | Names reserved before the 1.0 freeze for diagrams and rich text | `diagram` joins the words reserved for later use, and `Diagram` and `RichText` are reserved as type names in `core`, so that the `diagram { }` block and `rich()` labels can arrive after 1.0 without breaking a script | decided 11 November 2026 (option A); built at 0.11.0 with #84 |
| 118 | The plot renderer's fixed points before step 21 of 0.7.0 | The closed set of drawing primitives (with curves, text boxes and element names), row references for the viewer only, reserved fields for animation, 3D and maps, rasterised layers, scale limits as a zoom, notes off exported figures, labelling, shape and line-type orders, interval statements; and whether ticks stay on the rule built at step 20 or follow R's extended algorithm | decided 11 November 2026 (option A, the tick rule of step 20 kept); each rule carried by the 0.7.0 step that builds it |
| 117 | Opening a shared zip project in Zemp, one file from a zip of several, and keeping browser projects safe | **Open a replication package…** on the welcome screen of both editions, unpacking a zip into a new project and offering **Verify** as `zeff verify` does; `import(path, file:)` for one file of a zip that holds several; Zemp web asking the browser for persistent storage and reminding the user to export | decided 11 November 2026 (option A); built at 0.11.0 |
| 116 | Knowing how many use Zeff and Zemp, and who: counts, optional registration and citations, without a required login | Three layers: anonymous counts of downloads, engine fetches and web visits (no personal data, no cookies); an optional registration for wrong-result alerts that tells who uses the tools; a citation file and a DOI per release; an account only for paid features if they ever exist; Zeff and Zemp always usable without one | decided 11 November 2026 (option A); layer 1 at 0.8.0, layer 3 at 0.10.0, layer 2 at 1.0 |
| 115 | Zemp Lite's look: colour scheme, syntax colours, type and the screens of the mockups | One set of colour tokens for the light and dark themes, seven syntax colours, IBM Plex Sans and Mono, every text colour at least 4.5:1 against its ground, and the screens drawn in the owner's mockups (welcome, Inline, Panes, Git panel, data grid, variable view) as the reference for the interface | decided 11 November 2026 (option A); written into the Zemp Specification, section 5, the same day with #24 |
| 114 | "Copy as Zeff" in Zemp Lite's viewers | Lite's viewers offer **Copy as Zeff**, which puts the statement equivalent to a sort, filter or selection on the clipboard and never touches the script, beside Zemp Studio's **Add to script**; or Lite stays without it, as section 2 of the Zemp Specification reads now | decided 11 November 2026 (option C: sorts, filters and variable changes, no selections) |
| 113 | The plan of releases after 1.0, every model placed by level, and the names of the model functions | Replace the after-1.0 table of section 14 by 29 releases: basic models first (1.1–1.5), then advanced (1.6–1.17), then research grade (1.18–1.29), and within each level econometrics, psychometrics, machine learning, then the rest; outputs matched against R or Mplus only, with Stata-only methods held in 1.29; one function per kind of model, with about 25 roadmap names merged | decided 11 November 2026 (option A); catalogue, section 14 and the Zemp release table amended at 0.8.0, step 8c1 (15 November 2026) |
| 112 | The names of the binary models and of the ROC curve before the 1.0 freeze: `glm` and `curve` | At 0.9.0, `glm(y ~ x, family: binomial, link: logit \| probit)` in place of `logistic` and `probit`, and `fit.curve(type: roc)` in place of `fit.roc()`; the work is unchanged, other families, links and curves arrive later as words; answers question 4 of #99 with another name, and leaves its question 3 to the owner | decided 11 November 2026 (option A); applied as the first step of 0.9.0 |
| 111 | `p_adjust` inside `add`: adjusted p-values as a column, and families with `by:` | `p_adjust` given a column of a frame returns a column, as `rank` does, so that `t.add(p_holm = p_adjust(p))` works and `by:` adjusts each family on its own; given a list it returns a list as now | decided 11 November 2026 (option A); built at step 18c of 0.7.0 |
| 110 | The sizes and shapes of a plot: where a size starts, the six shapes and their names, sizes of 0 and below, `group` and `label` before a mark reads them, and shapes of equal area | As built at step 20d: (1) a size is laid out by area from 0, the largest value 18 points across; (2) decided 12 November 2026, option D: eight shapes in a fixed order, the six of step 20d then `triangle_down` and `star`, `asterisk` for `NA`, names with `_open`, repeating from the ninth category with a note (built at step 20f); (3) a size below 0 is dropped with a note and a size of 0 is kept and drawn with no size, with a note; (4) `group` and `label` are held and printed, and the plot says no layer draws them; (5) from step 21 the shapes of one size are drawn with equal area | decided: question 2 by the owner on 12 November 2026, questions 1, 3, 4 and 5 as recommended on 12 November 2026 |
| 109 | The colours of a plot: more categories than the palette has colours, a level with no row, and the names of colours | As built at step 20c: (1) the eight colours repeat from the ninth category and a note says so; (2) a level with no row among the marks drawn has no colour and no legend entry; (3) a colour is one of the 148 names of CSS or `#` and hexadecimal digits, R's own names not being accepted | decided 12 November 2026 (as built, each as recommended); the argument of `scale` to keep every level not before 1.0 (15 November 2026), placed with the grammar extensions of 1.2 |
| 108 | A numeric function of `core` given a text or category column | Refuse it when the statement runs, as R and Stata do and as the engine already refuses `log("a")` on one value; until decided `log(industry)` in `add` or in a mapping gives `NA` on every row without a message | decided 15 November 2026 (option A); built at 0.8.0, step 6, listed as a breaking change |
| 107 | What an expression mapped to an aesthetic may call | For 0.7.0 the operators and the row-by-row functions of `core` implemented in the engine, as built at step 20b; the reducers of `data` (`wage - mean(wage)`), the functions of `time` and `text`, and `logit`, `inv_logit`, `clamp` and `fill_missing` are refused with advice to compute the variable with `add` first; widening later breaks no script | decided 11 November 2026 (option A); widening put to the owner again at step 25 |
| 106 | A layer that overrides a mapping of its plot | Each mark takes `x:` and `y:`, and the other aesthetics it draws, for its own layer, as ggplot2's `geom_point(aes())` and Observable Plot's marks do, so that observed points and a fitted line of another variable share one plot; until decided a mark takes the catalogued options only and such an argument is refused as unknown (Z0404) | decided 12 November 2026 (option A, on the recommendation under the owner's standing instruction); built at step 23a of 0.7.0 |
| 105 | A confidence level on `posthoc`, and intervals for its step-wise methods | `level: Float = 0.95` on `posthoc`, as every other test of `stats` that prints an interval and as R's `TukeyHSD(conf.level = )` and Stata's `pwcompare, level()`; the step-wise methods keep a table without intervals, the help page saying that `emmeans` prints Bonferroni's there; until decided the intervals are at 95% | decided 13 November 2026 (option A); step 38f of 0.7.0 |
| 104 | `posthoc` after a factorial `anova`: which means are compared, and of which term | The estimated marginal means of one term, as R's `emmeans` and Stata's `pwcompare`, the term named by a new option, proposed `term:`, required when the model has several factors; built as step 19b of 0.7.0; until decided `posthoc` after an `anova` given a formula is refused | decided 11 November 2026 (option A, the option named `term:`); built at step 19b of 0.7.0 |
| 103 | The quality of the help pages, and a reference website | The owner's ruling: before 1.0 every function has a help page of the quality of the sample written for `add` (the commonest forms first, missing values under their own heading, a caption on every example, a common mistake with its real diagnostic, the R and Stata equivalents), and later a website carries every page in the manner of the tidyverse reference sites; open are the parts required of each kind of function, the timetable for the existing pages, and the release of the website | decided 11 November 2026 (option A); 0.8.0 (the amendment of section 9.1); 1.0.0 (the website's name and address) |
| 102 | The largest of several studentized ranges: R's `nranges` on `dist.tukey` | One more parameter, proposed `ranges: Int = 1`, for the largest of that many independent ranges on one estimate of the standard deviation, as `ptukey(q, nmeans, df, nranges)` and `qtukey` give it; Stata has no equivalent; `posthoc` does not need it; until decided the help page says it is not offered | decided 13 November 2026 (option B): not in 1.0; back at 1.1 with the less common gaps of #99 |
| 101 | Partial omega squared as Stata's `estat esize` computes it | One more word on `effect:` of `anova`, proposed `omega_f`, for the figure Stata's `estat esize, omega` and R's `F_to_omega2` compute from F; `omega` stays the ruled formula of R's `omega_squared`; until decided the help page states the difference | decided 13 November 2026 (option A, the word `omega_f`); step 38e of 0.7.0 |
| 100 | One term of the factorial `anova` table read as a value | `p_values("dose")`, `statistic("dose")`, `df("dose")` and `effect_size("dose")` read the row of that term, as the same methods read a term of a fit; without a term they stay the whole model's; until decided a term named there is an error and `estimates()` gives every row | decided 13 November 2026 (option A); step 38d of 0.7.0 |
| 99 | Every function covers the alternatives R and Stata give: the review and its gaps | The rule as read; the routine gaps of `posthoc`, `p_adjust`, `anova` and the plots into the open steps of 0.7.0; the other routine gaps before 1.0 in the release that owns each function; the less common ones at 1.1 unless their function is being built anyway; differences of convention answered in the help pages | questions 1 and 2 decided 10 November 2026, question 3 on 11 November 2026 (after 1.0); question 4 answered for `probit` by #112 and decided for `running` and `row` on 11 November 2026; built at 0.8.0 (the `running` and `row` merges) |
| 98 | `anova` with every type of sums of squares that R and Stata give | `type: 1 \| 2 \| 3 = 2` on the factorial form: the sequential table of R's `aov` and Stata's `sequential` joins types 2 and 3; built at step 17 of 0.7.0, the default unchanged | decided 10 November 2026 |
| 97 | One function with an option rather than a family of names | The rule for every addition; at 0.8.0 `add_time(d, n, unit:)` replaces `add_days` and `add_months`, `bin(x, n: \| breaks:)` absorbs `quantile_group`, `fill_missing(x, value \| method:)` absorbs `fill_forward`, and `variance_test(var, group:, method: f \| levene \| bartlett)` replaces the two planned variance tests | decided 9 November 2026 |
| 96 | Computing with a `DateTime`, and a `Date` from its parts | At 0.8.0: `hour`, `minute`, `second`; `time_between(start, end, unit: second \| ... \| year = day)` replacing `days_between` and `months_between`; `min` and `max` on date columns; `datetime(x, epoch:)`; `date(year, month, day)`; `start_of(d, month \| quarter \| ...)` | decided 11 November 2026 (option A as amended); built at 0.8.0 |
| 95 | `print` with several plain values on one line | `print` itself prints plain values side by side on one line, separated by `sep:` (a space unless given), and anything larger as its own block; `print("mean =", x)` reads as R's `cat`; built at 0.8.0 | decided 9 November 2026 |
| 94 | Reading a `TestResult`'s fields as values | Methods in the style of the result interface, `fit.statistic()`, `fit.df()`, `fit.effect_size()`, `fit.h0()`, `fit.h1()`, `fit.alternative()`; `estimates()` gains a `df` column and a row for the effect size; the Language Reference's sentence says the fields are read by these methods | decided 9 November 2026 |
| 93 | The interval of an effect size: noncentral t or large-sample normal | `t_test` prints the large-sample interval of Hedges and Olkin for d and g; R's `effectsize` uses the noncentral t by default, which needs a noncentral t distribution the engine does not have; decide whether `dist` gains it and the tests move to it | decided 15 November 2026 (option B, deferred): the noncentral t with the `power` family, in the release of #113's plan that builds it |
| 92 | A `TestResult` built from a body written in Zeff | The nine tests of 0.7.0 were planned in Zeff; a body has no way to build a `TestResult` (its hypotheses, table, estimate, effect and decision) nor to raise Z0411 and Z0412 (#89), so `t_test` ships in Rust; a constructor `test_result(...)` in `core` or `results`, with #89, would let the tests move | decided 15 November 2026 (option B): the tests stay in Rust; no constructor before 1.0 |
| 91 | Hypothesis and decision lines in test output | Every `TestResult` prints an `H0:` and `H1:` block (population-level sentences with standard symbols) and a `decision at α` line beside the p-value; `alternative:` added to the four directional tests; α fixed at 0.05 where no `level:` exists, revisited with #15 | decided 8 November 2026 |
| 90 | Variable ranges in a Zeff-written built-in | A `Vars` argument's ranges, `[id..z]`, resolved against the frame before a body sees them, so that `for v in vars` and `name_of(v)` meet plain names | decided 15 November 2026 (option A); built at 0.8.0, step 7b |
| 89 | A coded error from a Zeff-written built-in | `error(message, code:)` in a built-in's file, so that a body refuses an argument with the code a Rust body gives (Z0401, Z0406) rather than Z0001 | decided 15 November 2026 (option A, with the argument's name); built at 0.8.0, step 7a |
| 88 | Functions brought into 1.0 after the study of R packages | Nine items: `ar_test` (0.8.0); `margins`, `roc`, `confusion` (0.9.0); `dist.weibull`, `dist.logistic`, `dist.hypergeometric`, `mcnemar_test` and `merge(join: semi \| anti)` (0.10.0) | decided 7 November 2026 |

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

**Settled by #123 (11 November 2026).** The owner decided that no third-party package of either kind ships before 2.0.0, so Zeff-source packages do not come at 1.6, 1.1 or 1.11.

### 9. Registry hosting

| Option | For | Against |
| --- | --- | --- |
| A. GitHub Pages for the index, GitHub Releases for archives | Free, familiar, reliable | One vendor |
| B. Own server | Full control | Cost and maintenance against the "everything free" rule |
| C. Codeberg | Non-profit, free | Smaller, slower |
| D. A on GitHub with a Codeberg mirror; the index signed, so the host is never trusted | Free, no single point of failure, trust comes from the signature | Two places to publish |

*Recommendation.* D.

**Deadline moved by #123 (11 November 2026).** With no third-party packages before 2.0.0, the host is chosen when 2.0.0 is planned.

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

*Decided 1 November 2026, as recommended.* Adopted at 0.6.0 with the definition of `docs/releases/0.6.0.md`, question 2: `treat` is the period a unit is first treated (`NA` for a never-treated unit, which is a control), the relative period is `time − treat`, one indicator per relative period present except `ref:`, the terms named `event=-3`, `event=0`, `event=2` as category levels are.

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

*Decided 20 November 2026: closed, as settled by #113.* The question was put to the owner after the release of 0.8.0, in the session that keeps the plan of releases: the plan he decided on 11 November 2026 (#113) has overtaken the swap. Under it 1.2 is basic econometrics and 1.3 basic psychometrics; the two-by-two difference-in-differences is at 1.9 and confirmatory factor analysis and structural equation models at 1.12, so difference-in-differences already comes before the latent-variable models, which is what this entry's own rule would have chosen with no alpha user yet to ask. Asked "Close #18 as settled by the plan of 11 November?", he answered: "Yes, close it". Nothing in section 14 of the Engine Specification or in the catalogue changes. (#163, which would move difference-in-differences earlier still, is a separate question and stays open.)

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

**The owner's decision (11 November 2026).** "Svelte 5." Zemp's interface, in both editions, is written in Svelte 5 with TypeScript, bundled by Vite and packaged on the desktop by Tauri (Zemp Specification, section 3). The spike against SolidJS planned for 0.6.0 was not held and is not needed: the two demanding panes, the 100,000-row data grid and the redraw-to-fit plot viewer, are custom-built in either framework, and CodeMirror 6, the editor, is independent of both. Svelte is under the MIT licence, which the dependency rule allows; Zemp's code lives in its own repository, where the version is pinned and recorded with its licence when the interface work starts. The look the interface follows is #115. The Zemp Specification's section 3 names the framework and its section 17 moves the item to the decided list, logged in `docs/spec_changes.md`.

### 25. Zemp's licence

| Option | For | Against |
| --- | --- | --- |
| Apache 2.0 | Same as Zeff; contributors and institutions understand it; a commercial path remains through hosting, support and training | No licence-based revenue |
| AGPL with dual licensing | Keeps a paid licence possible | Deters institutional adoption; contributor agreements needed |
| Source-available (BSL) | Protects a future product | Not open source; against the project's framing |
| Open core: the app under Apache 2.0, the paid modules (cloud sync, version history, collaboration, admin console) under their own licence, as the business model assumes | A product to sell; the free app stays open and complete for analysis | Two licences to administer; contributors cannot touch the paid modules; the line between app and module must be drawn once and kept |

*Recommendation.* Apache 2.0, decided before the alpha is public. The business model (17 October 2026) assumes open core, which Zemp section 16 now records as the alternative; decide this item and the business model together, since "a commercial path through services" and "paid modules" cannot both be true.

**Settled by #125 (12 November 2026).** None of the four options was taken: Zemp is developed in a private repository from its first commit, Zemp Lite is free of charge but not open source at least until 1.0, and Zemp Studio, the paid edition once its work starts, has a proprietary licence. Section 16 of the Zemp Specification says so.

### 26. Start Zemp at 0.7.0

*Recommendation.* Keep: the plot renderer and language server Zemp needs exist at 0.7.0; the framework spike (24) starts at 0.6.0 so no time is lost.

*Decided 1 November 2026, as recommended.* Zemp's interface work starts at 0.7.0; the framework spike of #24 (Svelte 5 against SolidJS on the data grid and the plot viewer) runs in a Zemp session during release 0.6.0, outside the Zeff checklist.

### 27. Hosting Zemp web, and sync

| Option | For | Against |
| --- | --- | --- |
| GitHub Pages | Free | Cannot set the cross-origin isolation headers threads need from 1.1 |
| A static host with custom headers (Cloudflare Pages, Netlify) | Free tier, headers, fast | A vendor to depend on; bandwidth limits |
| Sync: none (export a zip) · the user's own folder sync (a project folder in OneDrive or Dropbox on the desktop; the browser's folder access on Chromium) · an own sync server | Folder sync keeps the privacy promise with no server | A sync server costs money and holds users' data |

| A Zemp server: paid cloud sync and encrypted backup, server compute for heavy jobs, a self-hosted server for institutions, as the business model proposes | The Pro and Institution tiers exist; heavy jobs leave the laptop | Contradicts Zemp section 13 as written ("sends nothing to a server") and section 10; running costs; the privacy promise must be rewritten as opt-in, with what is sent, how it is encrypted and where it is held |

*Recommendation.* A static host that sets the headers; sync through the user's own folder sync, no Zemp server. If the business model's paid tiers are adopted, the server option replaces this recommendation, local-only stays the default, and Zemp section 13 is rewritten before the alpha is public.

**The need comes sooner than the table says (13 November 2026).** The table gives the headers as needed "from 1.1", for threads. The browser path built at step 33e1 of release 0.7.0 already needs them: a run is cancelled through four bytes of memory shared between the page and the engine's worker (a `SharedArrayBuffer`, `docs/decisions.md`, 13 November 2026), and a browser gives shared memory only to a page that is cross-origin isolated. Zemp Lite web therefore needs the headers from its preview at 0.8.0. The alternatives on GitHub Pages are a service worker that adds the headers itself, which the register calls fragile on first load (#48, option C), or a cancel that stops the worker and starts a new one, which loses the run's results each time.

**The owner's ruling on hosting (13 November 2026).** Asked to choose between Netlify and Cloudflare for Zemp Lite web, the owner answered "Yes, record it" to the recommendation: Cloudflare Pages, whose free plan sets the headers through a headers file and does not cap traffic, which matters because every visit downloads the engine (2,535 kB compressed at step 33e3); the domain may be held at Cloudflare at cost, as recommended for `zeff-lang.dev` (`docs/decisions.md`, 6 November 2026). Zemp web is served at `lite.zeff-lang.dev` (the name ruled below); the website, the installer and the key list stay on GitHub Pages at 1.0, from their own repository. Cloudflare Pages accepts files of at most 25 MiB; the engine's WebAssembly file is 13.4 MB before compression at step 33e3, and if it nears the limit it is split so that packages load when a script uses them. The owner creates the account, with two-step sign-in, and points the address; the plan and its limits are checked again before the account is opened. Sync and a Zemp server stay open, due at Zemp 0.2 alpha.

**The address (13 November 2026).** Told that a path such as `zeff-lang.dev/lite` cannot be served by a second host while the name `zeff-lang.dev` points at GitHub Pages, since a name in the DNS points at one host for all its paths, the owner ruled "lite.zeff-lang.dev is fine, record it": Zemp Lite web is served at `lite.zeff-lang.dev`, a name of its own pointed at Cloudflare Pages, and the website links to it. The owner adds the DNS record when the preview is prepared for 0.8.0.

**The owner's answer (21 November 2026, given in the Zemp Lite session at step 1 of its release 0.2 and passed on in the shared notes; entered from a Zeff session, which alone edits this register).** His words to the seventeen questions of that checklist, each put with a recommendation: "Go, as recommended." The server part: no Zemp server before 1.0, so no paid sync, server compute or self-hosted server; the web editions send nothing to any server but the host's cookie-free count of visits (#116), and section 13 of the Zemp Specification stands as written. With hosting (13 November) and folder sync already ruled, the entry is closed until the paid tiers (#35, #36) bring the question back.

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

*Domain decided 6 November 2026: `zeff-lang.dev`,* option A's zone with option B's name, since `zeff.dev` was found registered by others. The owner preferred `.dev` to `.org` and to `.io`: `.dev` is a generic zone that serves only HTTPS, which suits a host carrying the installer and the signed key list, while `.io` is the country code of the British Indian Ocean Territory, whose future is uncertain since the 2025 agreement to transfer the territory, a risk a key-list address that cannot move should not take. The name is registered by the owner and pointed at GitHub Pages, as the recommendation says, so that nothing breaks if the hosting changes; `zeff-lang.org` may be held later to forward to it. The organisation is still open, with `zeff-lang` recommended (free on GitHub on 6 November 2026), due before 0.9.0.

*Organisation moved to 0.11.0 on 20 November 2026.* The owner's words, after the release of 0.8.0: "I want to skip #151 and #31 now and want to move it to 0.11.0". The domain stays as decided; the organisation is decided by 0.11.0. This entry says the organisation must be fixed before the launcher ships at 0.9.0, because the address of the key list cannot move without a launcher update. The engineering answer, recorded in `docs/decisions.md` under the same date for the session that plans 0.9.0: every address the launcher of 0.9.0 is built to trust (the key list, the index of engines, and through that index the downloads) is under the decided domain `zeff-lang.dev`, and none is a path of a GitHub organisation; the organisation, and any later move of the repository, then breaks no installed launcher. If the planning of 0.9.0 finds that this cannot hold, the question comes back to the owner before the launcher's step, not after it.

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

**The owner's decision (11 November 2026).** "As recommended." The four are accepted as proposed. (1) One density, with a **Larger type** setting that scales the whole interface by 1.15, remembered per user, beside Ctrl + and Ctrl −. (2) No icons in 1.0: every control is a word, the Git control included (#115). (3) On the desktop, native menus that mirror the ⌘K command palette one for one, so that no command lives in only one of them. (4) No tablet or phone layout in 1.0; below 1200 pixels of width the browser editions say so. The design language the recommendation asked to merge into the Zemp Specification is the look decided under #115 the same day, written into its section 5; this decision adds a paragraph there and closes the item of its section 17, logged in `docs/spec_changes.md`.

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

**The owner's answer (21 November 2026, given in the Zemp Lite session at step 1 of its release 0.2 and passed on in the shared notes; entered from a Zeff session, which alone edits this register).** His words to the seventeen questions of that checklist, each put with a recommendation: "Go, as recommended." The alpha is built and hosted; its address is announced only once applications for "Zeff" and "Zemp" are filed in the owner's home jurisdiction, which is his to do. The entry stays open until he says they are filed.

### 38. Setup files for Zeff, Zemp Lite and Zemp Studio

Installation was kept to the command line on 17 October 2026: the `zeff` launcher comes from the installer script, a package manager, an archive or the offline bundle, and the desktop apps from `zeff app install lite | zemp` or a package manager (Zeff Specification, section 2.1; Zemp Specification, section 9). Setup files (Windows `.exe` or `.msi`, macOS `.dmg` or `.pkg`, Linux `.deb`, `.rpm` or AppImage) were deferred, because unsigned ones meet SmartScreen and Gatekeeper warnings and signed ones need #23.

| Option | For | Against |
| --- | --- | --- |
| A. None until #23 is decided; the command line and the package managers are the only routes | Nothing to build or sign; one install path to test | Users who have never opened a terminal must read one page of instructions |
| B. Unsigned setup files with published checksums | Familiar double-click installs | Every install starts with a warning that most users will not click through |
| C. Signed setup files from the 0.5 desktop beta | Clean installs for everyone | Needs #23's certificate and membership from 0.11.0 |

*Recommendation.* A, revisited with #23 by 0.11.0; if the desktop beta's users ask for setup files, C.

### 39. Zemp Lite: paid features, the web address and a desktop preview

Three questions left open when Zemp Lite was decided (Zemp Specification, section 17): whether AI assistance and the paid tiers' features (cloud sync, version history) are offered in Lite as well as in Zemp Studio; the address of Zemp Lite web; and whether a Lite desktop preview ships before the 0.5 desktop beta.

| Question | Options | Recommendation |
| --- | --- | --- |
| Paid features and AI in Lite | Lite free-tier only · Lite gains the same Pro features as Zemp Studio · Lite gains AI assistance only | Free-tier only through 1.0, as the business model's fourth line says; the features do not exist before #27 and #28 are decided, so decide then, and prefer the same Pro features in both editions if any, since a project moves between them unchanged |
| The web address | A path on Zemp Studio web's origin (`…/lite`) · A subdomain · A separate site | A path on the same origin, with its own manifest scope for the offline app: the browser's private file system that both editions share (Zemp Specification, section 10) is per origin, so a subdomain or a separate site would break "Open in Zemp" |
| A Lite desktop preview before 0.5 | None · A preview at the 0.2 alpha | None: it would need the Tauri packaging and `zeff app install` of 0.11.0 three releases early, and the Lite web preview at 0.2 serves the same purpose |

*Recommendation.* As the table says; decide the address before the 0.2 alpha, the rest with #27 and #28.

**The owner's decision (11 November 2026).** "Yes, as recommended." (1) Zemp Lite is free-tier only through 1.0; whether it gains AI assistance or the paid tiers' features is decided with #27 and #28, preferring the same features in both editions if any. (2) Zemp Lite web lives at a path on Zemp Studio web's origin (`…/lite`), installable as its own offline app with its own manifest scope, so that both editions share the browser's private file system and **Open in Zemp** moves a project across; the domain itself waits on #27 (hosting) and #37 (trademarks). (3) No Lite desktop preview before the 0.5 desktop beta; the Lite web preview of the 0.2 alpha serves. The Zemp Specification's sections 2, 10 and 17 are amended accordingly, logged in `docs/spec_changes.md`.

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

*Applied 7 November 2026 (release 0.7.0, step 2b).* Option B from 0.7.0, the pragma having arrived at step 2: a warning or a note that carries a code prints it under the result, `warning[W0204]: 12 rows gave −Inf from log(sales) where sales = 0 (line 9)`, and a message without a code (`warn(message)`, the notes of a format or a seed) prints as before. The printed examples of sections 5.4 and 10.2 follow (logged in `docs/spec_changes.md`).

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

*Decided 1 November 2026, as recommended (option B).* Chunked parsing with boundaries fixed by the bytes alone arrives with the reader rewrite of 0.11.0; the `import` case of the 0.6.0 benchmarks is read at the pre-release routine, and the work is brought forward only if that case shows the parser to be the bottleneck of the panel work.

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

*Decided 1 November 2026, as recommended (option A).* The grammar stands: one bare range per formula, a further range in brackets, `(z1..z3)`, which the checker reads as the range; the form is documented in the table of section 7.1 at the `fe:` step of release 0.6.0.

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

*Decided 1 November 2026, as recommended (option A).* `ci()` defaults to the level the fit was made at and `level:` written out overrides it; catalogue row 163 reads `level: Float = the fit's level`; `predict`'s intervals follow the same rule when they arrive. Built at release 0.6.0.

### 68. A parameter of several types in a Zeff-written built-in

Raised on 31 October 2026 at step 15 of release 0.5.0. A built-in written in Zeff declares its parameters with the grammar's `type`, one type each; the loader checks a scalar parameter in its column forms, which covers `clamp(x: Float, ...)` on a column but not a parameter that takes a value of any scalar type (`fill_missing(x, value)`: Bool, Int, Float or Str, joined) or a list or a column (`n_missing(x)`). Both were planned in Zeff and are written in Rust at 0.5.0, their `layer` corrected (logged); `summary`, planned for a rewrite in Zeff at 0.5.0, meets the same limit with its `vars` and `stats:` parameters. A Rust built-in writes `x: Bool | Int | Float | Str` in its registry signature and the checker already understands it.

| Option | For | Against |
| --- | --- | --- |
| A. Allow `A \| B` in the `fn` line of a built-in's file only (not in a project's `fn`), the body checked once per type as it is per column form, the result typed by `Join` or `Same` where the signature says so | One signature language for Rust and Zeff built-ins; `fill_missing` and `n_missing` become the one-line Zeff bodies the catalogue planned; the body is still checked before a script runs | The `fn` grammar differs between a built-in's file and a project's; the forms multiply (each union doubles or triples them) |
| B. Several homes, one `fn` per type in one file (`n_missing` on a list and `n_missing` on a column), the checker choosing by the first argument as section 8.1, rule 6, chooses between packages | The grammar stays one; each body is simple | Two bodies to keep in step for one function; rule 6 chooses between packages, not within one, so the registry and the checker need a new case |
| C. Keep them in Rust | Nothing to build | The count of Zeff-written built-ins stays below the plan and the language cannot express two of its own simplest functions |

*Recommendation.* A, at 0.7.0 with `zeff test` and `zeff fmt`, when the `fn` line of a built-in is next touched; the catalogue's `layer` returns to Zeff for both rows then.

*Decided 6 November 2026, as recommended (option A).* Built at step 3 of `docs/releases/0.7.0.md`; `fill_missing` and `n_missing` return to Zeff at step 4.

*Applied 7 November 2026 (release 0.7.0, step 3.2).* In the `fn` line of a built-in's file a parameter may be declared with several types, `x: Bool | Int | Float | Str` or `x: List | Column`, and the result typed `Join` or `Same`; the checker reads the file once per choice of one type per such parameter (a choice whose types do not join is left out when the result is `Join`), each choice once per column form as before, at most 256 forms in all (Z0323 beyond), and the interpreter runs the form of the arguments' types, the exact one first and then one that accepts them. A project function declaring several types, or `Join`, is refused (Z0101). Section 9.1 of the Engine and Project Specification and the grammar of section 3.4 of the Language Reference record it (logged). `fill_missing` and `n_missing` return to Zeff at step 4, with the conformance case the piece needs.

### 69. Access to the enclosing frame from a Zeff-written built-in

Raised on 31 October 2026 at step 17 of release 0.5.0. `row_mean([q1, q2, q3])` inside `add` reads three variables of the frame whose method encloses the call; a Rust body reads them from the call's context, but a function written in Zeff sees its parameters and other functions only (section 5.3, "Scope"), and a `Vars` parameter is a list of names with no frame to look them up in. Both row functions were planned in Zeff and are written in Rust at 0.5.0, their `layer` corrected (logged). The same limit will meet every Zeff-written column function that takes variables rather than a column.

| Option | For | Against |
| --- | --- | --- |
| A. An implicit parameter: a built-in's `fn` line may declare `data: Frame` as its first parameter marked as the enclosing frame (a spelling to choose), which the checker supplies from the frame method the call stands in, Z0401 outside one as `n()` gives | The body is ordinary Zeff over a frame it was given, `data.add(...)` and the reducers included; the registry's usage line stays `row_mean(vars, ...)` | A new piece of signature grammar for built-ins only; the checker must know which frame (the group's rows under `by:`, as the Rust context gets) |
| B. Change the usage to a frame method, `data.row_mean(vars)`, so that the frame is an ordinary argument | No new grammar; the body is ordinary Zeff | `wages.add(score = wages.row_mean([q1, q2]))` repeats the frame and breaks under `by:`; the catalogue's usage changes |
| C. Keep them in Rust | Nothing to build | Every column function over variables stays Rust |

*Recommendation.* A, at 0.7.0 with open decision #68, when the `fn` line of a built-in is next touched; the catalogue's `layer` returns to Zeff for both rows then.

*Decided 6 November 2026, as recommended (option A).* Built at step 3; `row_mean`, `row_sum` and `diff` return to Zeff at step 4 where the piece expresses them.

*Applied 7 November 2026 (release 0.7.0, step 3.3).* The spelling is `data: Frame = enclosing`, the first parameter of the built-in's own `fn` line, typed `Frame`; the usage line and the page leave it out, and the engine binds it at the call to the frame of the frame method the call stands in, the group's rows under `by:`, before the arguments. A call that stands in no frame method is refused as `n()` is (Z0401, "no frame here"), and a project function marking a parameter is refused (Z0101) with a help naming the rule. Section 9.1 of the Engine and Project Specification and the grammar of section 3.4 of the Language Reference record it (logged). The conformance case `control/enclosing` digests `n()`, `row_mean` and `row_sum` wherever a frame method gives them a frame.

*Step 4.2 (7 November 2026).* The three functions stay in Rust, the piece notwithstanding. `row_mean` and `row_sum` reduce each row with the kernel's compensated sum and mean (section 8.3, point 3: one reduction order), which `+` over columns, the only row-wise arithmetic a body has, does not reproduce bit for bit (the `rows` case holds 1e12 beside 1.5 and 2.5); `diff` is `x - lag(x, n)`, and `lag` reads the panel of the frame around the call, which a body written in Zeff does not pass on to the functions it calls (section 5.3, "Scope"). The piece stands for the Zeff-written functions over a frame's variables that need no kernel reduction and call nothing that reads the frame for itself; the catalogue rows record the reasons.

### 70. A note or a coded warning from a Zeff-written built-in

Raised on 31 October 2026 at step 18 of release 0.5.0, widened at step 20: `skewness` and `kurtosis`, written in Zeff, give `NA` for too few values with a warning that the owner's answer to question 5 numbers W0205, and `warn(message)` carries no code, so the warning prints the same under a result (a warning under a result shows no code, #45) but carries no code in `--format json`. `drop_missing` reports the rows it dropped in a note that names the variables, `note: 228 rows with NA in educ or exper dropped`. A body written in Zeff can print, `warn` and `error`, but `warn` prints a warning where section 4.1 asks for a note, and a `Var` has no text form, so the variables' names cannot be written into the message. The function was planned in Zeff and is written in Rust at 0.5.0, its `layer` corrected (logged).

| Option | For | Against |
| --- | --- | --- |
| A. Add `note(message)` to `core` beside `warn`, a `code:` option on both (`warn("...", code: W0205)`), and `name_of(v)` giving a `Var`'s name as text, all callable from a function | Two small functions; the note is then ordinary Zeff, and a project's own functions gain the same voice | Two catalogue rows to add and freeze in `core` before 1.0; a note from a project function is a new thing for Zemp to show |
| B. Keep such functions in Rust | Nothing to build | Every Zeff-written verb that reports a count stays Rust |

*Recommendation.* A, at 0.7.0 with #68 and #69; the catalogue's `layer` returns to Zeff for row 121 then.

*Decided 6 November 2026, as recommended (option A).* `note(message)`, `code:` on `warn` and `note`, and `name_of(v)` are built at step 3, their catalogue rows added there; `drop_missing`, `skewness` with W0205 and `summary` on `table` (#72) follow at step 4.

*Applied 7 November 2026 (release 0.7.0, step 3.1).* `note(message)` and `name_of(v)` are rows 534 and 535 of `core`; `warn` and `note` take `code:`, a warning's code written bare (`code: W0205`) or as text, which the engine checks against its pages when the statement runs (Z0401 for a code no warning has, or an error's code). A coded message from `warn` or `note` is one the pragma of section 11.1 allows and `--deny-warnings` refuses, as the engine's own; the page of a built-in written in Zeff declares the codes its body gives the two. Section 5.4 of the Language Reference and the "Warnings and notes" row of section 9.1 of the Engine and Project Specification say so (logged); the counts of both volumes and of the Zemp Specification follow. At step 4.3 the warnings of `skewness` and `kurtosis` for too few values carry W0205, as the owner's answer numbered them. At step 5b, `join` in hand, `drop_missing` still stays in Rust: a variable range reaches a body as its written token (#90) and the errors its page declares would be Z0001 from a body (#89).

### 71. The fitted design through the result interface

Raised on 31 October 2026 at step 21 of release 0.5.0. `vif` regresses each column of the design a fit used on the other columns; a script has the fit's estimates, variances, residuals, fitted values and leverages through the result interface (section 8.1, rule 5) but not the design, so the function was planned in Zeff and is written in Rust at 0.5.0 on the Gram matrix the fit now keeps, its `layer` corrected (logged). Later diagnostics in Zeff (partial residual plots, added-variable plots, leverage against the regressors) meet the same wall.

| Option | For | Against |
| --- | --- | --- |
| A. A `design()` method on every fit, giving the kept design as a frame of the rows used, the intercept and the dummy columns among them under the term names of section 7.2 | One method; every diagnostic over the design becomes ordinary Zeff over a frame, `reg` included | The frame can be large (the rows used by every term); it is derived from the fit's data, which the fit does not keep today, so either the data or the design is stored with the fit |
| B. Keep the Gram matrix and write such diagnostics in Rust | Nothing stored beyond k × k numbers | Every design diagnostic stays Rust |

*Recommendation.* A, at 0.7.0 with the plot grammar, which needs the design for its result plots; the catalogue's `layer` returns to Zeff for row 248 then.

*Decided 6 November 2026, as recommended (option A).* `design()` is built with the result plots at step 30, its catalogue row added there; `vif` returns to Zeff in the same step.

### 72. A table constructor for Zeff-written functions

Raised on 31 October 2026 at step 22 of release 0.5.0, where `summary` was to be rewritten in Zeff. `summary` returns a `Table` (section 3.8: a title, columns with their alignment, rows of text, lines under the grid), with counts written with thousands separators and statistics to three decimals, a group column under `by:` and one warning naming every variable's missing rows. The language has no value that builds a `Table`: `aggregate` (0.6.0) is the first function to return one from data a script shapes, no function turns a number into text with a chosen number of decimals before the `text` package, and a function raises no coded warning (#70). The Rust body stays and the catalogue's `layer` is corrected (logged).

| Option | For | Against |
| --- | --- | --- |
| A. A constructor in `core`, `table(title: Str, columns: Record)` taking lists of text or numbers (numbers printed as the display rules print them, or through `format(x, decimals:)` from `text`), with `by:`-style grouping left to `aggregate` | `summary` and every tabulating function a project writes become ordinary Zeff; the constructor is a few lines over the `Table` the engine already prints | A catalogue row in `core` before 1.0; `format` belongs to `text`, which `core` may not call, so the numbers' formatting is the display rules' until `text` ships |
| B. `aggregate` at 0.6.0 as the only way to a table, `summary` rewritten over it | No new function | `summary`'s title, its warning and its `stats:` words are not `aggregate`'s shape; the rewrite becomes a different function |
| C. `summary` stays Rust | Nothing to build | The one function section 14 named for the rewrite never moves |

*Recommendation.* A at 0.6.0 beside `aggregate`, the rewrite of `summary` following in the same release; the catalogue's `layer` returns to Zeff for row 153 then.

*Decided 1 November 2026, as recommended (option A), with one correction.* `table(title: Str, columns: Record)` joins `core` at 0.6.0 (catalogue row 530), taking a record whose fields are lists or columns of one length each, numbers printed by the display rules or through `format` by the caller. The rewrite of `summary` does not follow in 0.6.0: `summary` prints one combined warning naming every variable's missing rows, which a Zeff body cannot raise until #70 (0.7.0) is taken; the rewrite is scheduled with #70, and row 153 stays Rust until then. Built at release 0.6.0, step 16.

*Step 4.5 of 0.7.0 (7 November 2026).* `summary` stays in Rust, `table` and the pieces of #70 notwithstanding: its counts print with thousands separators and its statistics to three decimals, which need `format` (`text`, step 6); its one warning names every variable with missing rows, which needs `join` (step 5); its title carries the frame's binding name, which no function of the language gives a body; and it takes `weights:` with `weight_type:`, whose weighted mean and standard deviation are the reducers' kernels. The owner may schedule the rewrite after `text` ships, with a frame-name accessor as a piece of its own, or leave `summary` in Rust for 1.0.

### 73. Typst: the typesetting engine of exports and of the report document, with a live preview

Raised on 1 November 2026, from the owner's request for first-class typst with a live preview. Section 8 of the Zemp Specification plans a report document at Zemp 1.11, "text and Zeff chunks that re-run from the project", exporting to Word, PDF and HTML, and names no engine for the PDF; today's exports of tables go to Word and LaTeX, and the plot renderer of 0.7.0 will write SVG and PDF of its own. Typst is a typesetting system written in Rust under the Apache-2.0 licence: it compiles a document to PDF in milliseconds and incrementally, runs in the browser as WebAssembly, embeds its fonts, and sets the same bytes on every machine when the fonts are fixed, which is the promise Zeff already makes for tables and plots (risk register, R8). It has cross-references, numbered figures and tables, equations and bibliographies (BibTeX and its own Hayagriva files) of its own. A LaTeX installation on the user's machine is the alternative every statistical tool relies on; it cannot be bundled, differs between machines, is absent from most students' computers, and is hundreds of megabytes.

| Option | For | Against |
| --- | --- | --- |
| A. Typst as the engine of every PDF Zeff or Zemp sets, and the report document a typst document. `zeff export` writes `.typ` beside `.tex` and `.docx`, and a PDF set by typst from it; the plot renderer's SVG is placed in typst pages; Zemp Studio and Zemp Lite preview a `.typ` file of the project live, page by page, as it is typed; the report document is a `.typ` file in the project's `report/` folder whose fenced Zeff chunks the engine runs in order as one script, their tables and plots placed as typst content, with chunk options as attributes (`echo`, `output`, `label`), cross-references and citations typst's own, parameters as a record at the head of the document, and the content-addressed cache of the Engine and Project Specification, section 8.3, point 4, keeping a chunk's output until its code or inputs change; the document re-runs under the run record, so a report is a replication package | One engine, in Rust, under a permissive licence, deterministic to the byte with embedded fonts; the live preview is typst's incremental compiler; the browser and the desktop produce the same PDF; most of the report document already exists in the engine (the cache, the run record, the exports) and in typst (references, bibliographies, numbering), so what is built is the chunk runner, the preview and the placing of tables and plots | A dependency of some size: typst as WebAssembly is several megabytes before fonts, so the browser loads it on first use (section 8.1, rule 7) and the bundle budget of R4 is checked at the spike; a new file kind in projects (`.typ`) and the fonts to embed and license; the whole-document Word export is not typst's and stays as the table and plot exports until a converter exists |
| B. Typst as an export target only, `.typ` beside `.tex`, with the report document's engine decided at 1.11 | A small step; users of typst get their format | No preview, no PDF of Zeff's own; the decision only moves |
| C. LaTeX through an installation on the user's machine | Nothing to bundle; what journals have long expected | Not deterministic, not available in the browser, not installed on most machines |

*Recommendation.* A, taken in steps, each behind the standard checks for a dependency (licence, one dependency at a time, the cross-target identity of the PDF bytes with the fonts embedded, the bundle's size): a spike at 0.7.0 beside the plot renderer, whose SVG is what typst would place; `.typ` and PDF exports of tables and plots, the live preview of a project's `.typ` files, and Lite's **Export script with output** (section 5, "Planned for Inline") generated as a typst document and set by typst, at 0.8.0, which proves the chunk model before any authoring; the report document itself built as section 15 schedules it at 1.11, or pulled to an earlier release by a logged change to section 15 when the 0.8.0 slice has shown its cost. Deadline: 0.8.0, with the spike's result reported at 0.7.0.

*The spike of 0.7.0 (12 November 2026, step 22b4).* Run as a tool only with Typst 0.15.0, nothing linked into the engine; the findings are in `docs/decisions.md` under "12 November 2026: the trial of the typesetting engine, as a tool only". In short: Typst and the 298 crates of its library are under permissive licences, the fonts and hyphenation patterns it bundles still to be read; with the fonts and the date fixed two runs give the same bytes, and without a date they do not; an SVG of Zeff with its text as outlines is placed as lines at its own size and its words stay searchable; a table has no path into Typst until `export` writes `.typ`. Not measured: the bytes across machines, and what the library adds to the size of the engine. The decision stays due at 0.8.0.

*Decided 15 November 2026, at the opening of release 0.8.0, on the recommendation (option A in steps), the owner's to overturn.* `export` writes `.typ` beside `.tex` and `.docx` at step 21a of `docs/releases/0.8.0.md`, a pure text writer with no dependency; the typst library is measured as a dependency at step 21b (the licences of the fonts and hyphenation patterns it bundles, the PDF bytes across the four targets, the size it adds to the engine and to the browser bundle against R4's limit), and whether the engine sets PDF through it is decided from those figures and recorded here; nothing is linked unless the figures allow and the owner agrees. Lite's export of a script with its output as PDF belongs to the alpha, in Zemp.

*Measured and decided, 19 November 2026 (steps 21b and 21c of release 0.8.0).* The library (`typst` and `typst-pdf` 0.15.1) was linked into a measuring tool outside the workspace, `tools/typst-trial`, and into nothing else; the figures are in `docs/decisions.md` under that date. In short. The bytes: one fixed document, set with the engine's three fonts, a fixed date and a fixed creator, is the same 42,327 bytes on the five targets of the identity jobs (Linux on x86-64 at two levels, Linux on ARM64, macOS on ARM64, and the browser's target under Node). The licences: the 298 crates are permissive, and none of Typst's own fonts need be linked (one of them is under the GPL 3 with exceptions; they are off unless asked for); but the hyphenation patterns inside `hypher` are compiled in for all 48 languages, and 13 of them are under the LaTeX Project Public License alone and the Hungarian under the MPL, the GPL or the LGPL, none of which this project's rule for dependencies allows, and a crate that uses the library cannot leave a language out. The size: about 26 MB added to an engine of 17.5 MB, and 7.2 MB compressed in the browser, where the whole bundle is 2.76 MB today and may not pass 5 MB. **Decision, on the figures and under the rule of this entry ("nothing is linked unless the figures allow and the owner agrees"): the figures do not allow it, and the engine does not link Typst.** `export` writes `.typ` (step 21a), which a Typst the user has installed sets as PDF, with the engine's fonts if they are given to it. Whether Zemp carries Typst as a module of its own is #172. The owner may overturn this.

### 74. A Help pane in the Viewer

Raised on 1 November 2026 from the owner's requests for the Zemp interface. Section 5 of the Zemp Specification reaches help through hover, completion, `:help` in the console and ⌘K, and gives it no pane; a Stata or SPSS user expects a help window that stays open. Every page already exists in the engine (section 9.1's contract), and every code has an explained page, so a pane costs the view and nothing else.

| Option | For | Against |
| --- | --- | --- |
| A. A **Help** tab in the Viewer beside Data, Results and Plots, in both editions: it follows the cursor to the page of the function under it (a switch stops the following), opens a code's explained page when a Log entry is clicked, and pops out like the other viewers | The page under the cursor is one glance away; the Log's codes become readable without leaving the window; the same pane serves a project's own functions, whose pages the `///` lines give | One more tab; following the cursor must be cheap (the language server already knows the function under the cursor) |
| B. Hover and ⌘K only, as written | Nothing to build | The user who reads help at length has nowhere to keep it |

*Recommendation.* A, at the 0.2 alpha (Zeff 0.8.0), when the language server exists to say what is under the cursor. Deadline: 0.8.0.

**The owner's answer (21 November 2026, given in the Zemp Lite session at step 1 of its release 0.2 and passed on in the shared notes; entered from a Zeff session, which alone edits this register).** His words to the seventeen questions of that checklist, each put with a recommendation: "Go, as recommended." Option A in the margin editor's form: a Help panel at the side of the Script place, hidden until asked for (F1, ⌘K, **Explain** on a warning), following the function under the cursor unless switched off, showing a code's explained page; in Panes, a Help tab. At the Zemp 0.2 alpha.

**The Help page set in type (22 November 2026; passed on in the shared notes by the Zemp Lite session, which records the owner's choice "C, formatted" on a mockup kept outside the repositories).** From Zemp Lite 0.2.1 the Help place shows a function's page and a code's explained page set in type (headings, prose in the app's type and code in the code's, each argument with its type, codes as coloured labels, names and codes as links, the usage and the examples as code) and no longer as the terminal prints it; every word stays the engine's, and Zeff 0.8.0 is unchanged. Nothing normative changes: section 5 of the Zemp Specification says what the Help place shows and how it is opened, not how a page is set. It supersedes Zemp Lite's choice of the same morning that a page be shown character for character, which ran past the middle of the place in a narrow window. Two things follow for the engine. First, the interface takes the print of `zeff help` and `zeff explain` apart by its layout (the first line, the headings in capitals at the first column, the body four characters in, the rows of ARGUMENTS and of WARNINGS AND NOTES, `(shared)` at a line's right edge, backticks, the asterisks of a title in REFERENCES, `> ` before a statement of an example), and a test of Zemp Lite holds every page of the engine to a known shape: the layout of the print is therefore something an interface depends on, recorded in `docs/decisions.md` of this day, and a change to it is announced to Zemp Lite before it ships. Second, a structured form of a help page, as `zeff explain --format json` gives a code's page in parts, would let the interface drop its reader, say which names in the prose are functions, and set a package's list in type; it is a candidate for an entry of its own at the planning of the release that rebuilds help (0.11.0, "Help built from `///`"), and is not raised here.

### 75. The data viewer and the environment browser, made excellent

Raised on 1 November 2026. Section 5 gives the Data viewer sorting and filtering as view state, **Add to script**, and the variable view, and the Objects panel every binding with type, size, memory and stale marks; the owner asked for a strong live view of objects and an excellent data viewer. What is missing is the detail that makes a grid usable on 18,000 rows and a panel usable on a long session.

| Option | For | Against |
| --- | --- | --- |
| A. The grid gains frozen columns, a statistics header on every column (type, missing count, a small distribution, drawn from the schema and the kernel's summaries), search with matches highlighted, a live row count under a filter, copy of a range, and in Zemp Studio the statement **Add to script** will write shown before it is inserted; the Objects panel becomes a tree (a frame opens to its panel declaration and its columns, a list to its items with failed ones marked, a record to its fields) with memory per object and a filter box. Zemp Lite has the same grid without **Add to script**, the variable view read-only, and Objects filling the column Methods has in Studio (section 2) | Each item is a known gap of data grids in statistical tools; the statistics header reuses what `describe` computes; the tree is the Objects panel's existing data shown one level deeper | A virtualised grid with frozen columns is the hardest pane of the interface, which is why the framework trial of #24 is run on it |
| B. The viewer and panel as written | Nothing to build | A grid without frozen keys or column statistics sends users back to Stata's browser |

*Recommendation.* A; the grid is the subject of #24's trial at 0.6.0 and ships at the 0.2 alpha. Deadline: 0.8.0.

**The owner's answer (21 November 2026, given in the Zemp Lite session at step 1 of its release 0.2 and passed on in the shared notes; entered from a Zeff session, which alone edits this register).** His words to the seventeen questions of that checklist, each put with a recommendation: "Go, as recommended." Option A in the Data place of the margin editor: the grid drawn as scrolled, the first column frozen, a statistics header per column asked of the engine, search with matches marked, a live row count under a filter, copying a range, variables shown and hidden; read-only with **Copy as Zeff** (#114). The Objects tree is replaced by Data's list of frames, each by the statement that made it. At the Zemp 0.2 alpha.

### 76. Traceback, run to here and the loop inspector

Raised on 1 November 2026 from the owner's request for tracebacks and step debugging. A Zeff run is a sequence of statements whose bindings never change (Language Reference, section 3.1), so running to a line and reading every binding is what a debugger does elsewhere by stepping, and the Inline layout already shows each statement's output under it. Three things are missing. An error inside a module function names the module's line (section 5, "A module function runs under its own file") but not the chain that led there; nothing shows a loop pass by pass; and running up to a line is done by selecting and running.

| Option | For | Against |
| --- | --- | --- |
| A. A traceback on every error raised inside a function: the script's line, each module function's line with the values that made it fail, and the function that raised the code, with the explained page one click away; **Run to here** as a command and a gutter action, which runs the statements before the line from a fresh session or only those not yet run, since bindings never change; a loop inspector beside the Log with one row per pass of a `for` loop (the loop variable, the rows after a `where`, the result, the time) and one row per `iterate` pass (the state and what `done` compared), read from the run record, which keeps the pass as the random stream's key already does (Engine and Project Specification, section 8.3) | The interpreter knows the chain and the pass; the inspector costs nothing when closed; the three together answer the questions a debugger answers in this language | The run record grows by one line per pass, bounded by `max:`; a long `for` loop needs the inspector to fold |
| B. A breakpoint-and-step debugger in the interpreter | What programmers expect | Nothing changes while stepping, so each step shows what Objects already shows; a second execution mode in the interpreter, under the determinism contract, for little gain |
| C. The error's file and line only, as written | Nothing to build | A failure three calls deep is found by hand |

*Recommendation.* A, the traceback with the language server at 0.7.0 and the inspector and **Run to here** at the 0.2 alpha; B rejected. Deadline: 0.7.0.

*Decided 6 November 2026, as recommended (option A).* The traceback through module functions is built with the engine worker at step 33; **Run to here** and the loop inspector belong to Zemp's 0.2 alpha.

*A reading of this decision for the margin editor (entered 20 November 2026; R15 of the editor's document, #174).* The decision makes **Run to here** "a command and a gutter action". In the margin editor the gutter action is the words *Run to here* shown on resting over a statement's line number, beside the command in ⌘K: a word and not an icon, in keeping with the quiet look. This is how the decision is read, not a new choice; the owner may prefer ⌘K alone. **Run from here**, its partner, is proposed as #181, and what builds on the traceback and the inspector as #183 to #187. Nothing here changes what was decided: no breakpoints and no stepping.

**The owner's answer (21 November 2026, given in the Zemp Lite session at step 1 of its release 0.2 and passed on in the shared notes; entered from a Zeff session, which alone edits this register).** His words to the seventeen questions of that checklist, each put with a recommendation: "Go, as recommended." The reading of 20 November 2026 is confirmed (R15 of the margin editor's document): *Run to here* is shown on resting over a statement's line number, beside the command in ⌘K.

### 77. Schema-aware completion and extract function

Raised on 1 November 2026 from the owner's request for deeper code intelligence. The language server's table (Engine and Project Specification, section 10) has completion, signature help, hover, diagnostics and formatting at 1.0, and quick fixes, navigation, rename and inlay hints at 1.1. Two things matter more than the rest in this language. The schema pass of 0.6.0 gives the checker the columns of every frame at every point in a chain, so completion can offer them; and functions in `functions/` are how a project grows, so turning a block of a script into one should be a command.

| Option | For | Against |
| --- | --- | --- |
| A. Schema-aware completion at 0.7.0: after `frame.` or inside a formula or a column argument, the columns of that frame at that point, with type, missing count and label, and a line saying what the frame holds after the statement; **Extract function** at 1.1 beside rename: the selected statements become a `fn` in `functions/` (or in the script), its parameters inferred from the bindings the block reads but does not define, its return type from the last statement, the page skeleton Z0319 requires written, the call rewritten, and the check that the script prints the same bytes before and after; **Find references** and **Move to module** with it | Completion that knows the data is what no R or Stata editor has; extraction with inferred parameters is mechanical in a language without mutation; the same-bytes check uses the identity machinery the engine already has | Extraction needs the typed IR of the whole project, which the language server holds from 0.7.0; a block that reads a loop variable or a frame's column by bare name needs a rule, written with the feature |
| B. The language server's table as written | Nothing to add | The two features that fit the language best are left to third-party editors, which cannot build them |

*Recommendation.* A, completion at 0.7.0 and extraction at 1.1, each a row of the language server's table when adopted. Deadline: 0.7.0.

*Decided 6 November 2026, as recommended (option A).* Completion from the schema pass is built at step 35; extract function stays at 1.1.

### 78. Git history, the diff of a script and a diff of results

Raised on 1 November 2026 from the owner's request for more advanced Git and project features. Section 5's Git panel (1.1 on the desktop) has the branch, the changed files, a message, Commit, Pull and Push. The run record of every commit holds every output (Language Reference, section 11), and the engine is bit-identical, so two commits can be compared in numbers, which no other statistical tool can do.

| Option | For | Against |
| --- | --- | --- |
| A. The panel gains the history of the project, the diff of a script between two commits, and a **Results diff**: a table of every output of the two commits' run records, each identical to the bit or different, a difference traced to the statement that changed in the script diff, and a difference with no code change flagged, since it means a different engine, seed or input file; merges, rebases and pull requests stay with the system terminal, which the bottom bar opens in the project folder | History and a script diff are what the Git installed on the computer already gives; the results diff is the run record read twice and is the panel's one feature of Zeff's own; the terminal keeps the hard Git out of the interface | The run records of past commits must be kept or recomputed (`zeff verify` recomputes one); a results diff across engine versions is a comparison, not a diff, and the table says so |
| B. The panel as written, everything else in the terminal | Nothing to add | A researcher asks "what changed in my numbers since last week" and the tool, which knows, does not answer |

*Recommendation.* A at 1.1 with the Git panel, the results diff reading the run records the project keeps. Deadline: 1.1.

### 79. Arithmetic over lists, or a map, in a Zeff-written built-in

Raised on 2 November 2026 at step 10 of release 0.6.0. `quantile_group(x, n:, type:)` was planned in Zeff as `to(bin(x, breaks: the quantiles at i/n), Int)`, and `quantile` takes its probabilities as a list; but a body written in Zeff cannot make the list 1/n to (n − 1)/n: `seq(1, n - 1)` is a list of Ints, `/` does not apply to a list (Z0221), no function maps over a list, and `seq(1 / n, 1 - 1 / n, step: 1 / n)` reaches each probability as `1/n + i × 1/n`, within a rounding of i/n but not the value R's `quantile` is given, so the groups could differ from R's at a cut point. The function is written in Rust at 0.6.0, its `layer` corrected (logged). The same limit will meet every Zeff-written function that computes with a list of numbers.

| Option | For | Against |
| --- | --- | --- |
| A. Lift the arithmetic operators over lists as they are lifted over columns, so that `seq(1, n - 1) / n` is a list of Floats | No new function; reads as the mathematics | A list is a value of the language, not data, and `[1, 2] + [3]` needs a rule for lengths; a list of Str or Bool under `+` needs a message |
| B. A `map(list, f)` over a function value (section 3.7, "Functions as values"), `map(seq(1, n - 1), fn(i) i / n)`, with `filter` and `reduce` beside it | General: every computation over a list becomes expressible; a project's own functions gain the same | A function literal in an expression is new grammar, and the three functions are three catalogue rows to freeze in `core` before 1.0 |
| C. Keep such functions in Rust | Nothing to build | Every Zeff-written function over a list of numbers stays Rust |

*Recommendation.* B, at 0.7.0 with #68 to #70, when the `fn` line of a built-in is next touched; the catalogue's `layer` returns to Zeff for row 131 then. Deadline: 0.7.0.

*Decided 6 November 2026, as recommended (option B).* A function literal in an expression, with `map`, `filter` and `reduce` in `core`, is built at step 3b, the grammar and the three catalogue rows added there; `quantile_group` returns to Zeff at step 4.

*Applied 7 November 2026 (release 0.7.0, step 3b).* A function literal, `fn(i) i / n`, is accepted where a parameter takes a function: its parameters take the function type's types unless written (which must then agree), its body is one expression over them and the bindings in sight, whose values the literal takes with it, and a literal anywhere else is Z0205. `map(list, f)`, `filter(list, f)` and `reduce(list, f, start)` are rows 536 to 538 of `core`, typed by the list's items at each call (`Fn(T) -> R` giving `List<R>`; `Fn(T) -> Bool`; `Fn(S, T) -> S` with `S` the type of `start`), run by the interpreter as `iterate` is; `map(seq(1, n - 1), fn(i) i / n)` gives the probabilities i/n exactly. The grammar of section 3.4 and the rows of sections 3.7 and 5.3 of the Language Reference, and the counts of both volumes, record it (logged).

*Step 4.4 (7 November 2026).* `quantile_group` is written in Zeff: `frame(value = x).quantile(value, p: map(seq(1, n - 1), fn(i) i / n), type: type)` gives the cut points, the same bits as the kernel's, and each row's group is one more than the number of cut points below its value, folded with `reduce`; every result and every conformance digest is unchanged. A pragma inside the body silences the reducer's W0201 for the rows with `NA`, which the Rust form did not raise; the file's pragmas now travel with the body. The function's argument errors (`n:` missing, below 1, `type:` outside 1 to 9) are raised by `error` and so print as Z0001 at the call, where the Rust form gave Z0406 and Z0401; that is open decision #89.

---

## Shorter and clearer scripts

Added 5 November 2026. The owner's proposal of 4 November 2026 gathers ideas for making scripts shorter, clearer and harder to get wrong without hidden state, each held to four checks: it contradicts no written rule, reverses no recorded decision, respects the tier rules and gives no word a second meaning. The proposal was checked against both volumes, the catalogue, this register and the code on 5 November. Its citations hold, and its two set terms parse under the grammar of section 3.4 as it stands; its counts predate the 259 functions of section 14, and it was written before #41 to #79. The owner asked for the core set to be recorded: the two items that need a `core` name or serve the brevity criterion of section 13 (#80 and #81), the two gaps in the text the proposal found (#82 and #83), and the word reservation that takes the deadline off the first two (#84). The other items (plots of several variables, several outcomes in one model, updating a fit, models by group, `highlight:`, lists of variables in `label` and `recode`, the check-time `case` warning, adaptive digits and the footnote sentence as options, idiom hints and the rest) are additions a minor release may make before or after 1.0; none is recorded until the owner adopts it. No entry here changes the normative text.

### 80. `each(...)`: one marker for "separately", and tests on several variables

Raised on 5 November 2026 from the owner's proposal (its item P7a). Running one test on several variables takes a loop, `[for v in vars(lwage, hours, educ) { wages.t_test(v, group: female) }]`, and then `p_adjust` over the collected p-values, which come back detached from the tests. A plain list cannot carry the meaning, because `[a, b]` means "together" wherever it appears (`drop_missing`, `correlation`, `by:`). The proposal adds one rule beside the lifting rule of section 3.7: a `Var` parameter of a built-in function that receives `each(a, b, c)` runs the function once per variable and returns a set.

```
wages.t_test(each(lwage, hours, educ), group: female)
```

For the tests of 0.7.0 the set is a `TestResults`, declared in `stats`: one table with one row per variable in the order written, N on every row, the raw and the adjusted p-value both printed (`p_adjust: holm | bonferroni | bh | by | none = holm`, the methods and the default of catalogue row 178), and `missing: pairwise | listwise = pairwise` as in `correlation`. One `each` per call; `each` in a `Vars` parameter, or given to a project function (which is not lifted, section 3.7), is an error. The grammar does not change. `each` would be a `core` name, so it cannot be bound once it exists (section 3.6), and it cannot join `core` after 1.0 without breaking a script that binds it; hence #84.

What the text must settle before it is built, since section 1 allows no unspecified behaviour: whether `each(...)` is a value a `let` may bind or exists only where it is written in an argument, as a bare word does; which functions accept it (the proposal names the one-variable forms of `t_test`, `wilcoxon_test`, `proportion_test`, `anova`, `kruskal_test`, `shapiro_test` and `ks_test`, the first variable of `chi2_test` and `fisher_test`, and the `group:` of a test; not the paired forms); how it meets `by:`, which statistics already take (section 6); what `p_values()` gives on a set, raw or adjusted; and the places that name `TestResult` alone (the naming table of section 3.6, the types of section 4, the join rule of section 3.7).

Two choices are the owner's as a statistician: whether the adjusted p-value is printed by default when the family is simply the variables of one call (R and Stata print the raw value only), and whether each test uses its own complete rows by default.

| Option | For | Against |
| --- | --- | --- |
| A. Adopt, built at 0.7.0 with the tests | The tests ship once, with their final printed forms and help pages; the same rule then serves model sets (#81) and, later, plots; the target analysis is as short as in rstatix or Stata | 0.7.0 is already the largest release (60 functions, the plot renderer, the language server) with #68 to #71, #76, #77 and #79 due; a new result type brings its page, its snapshots on four targets and its reference comparison (R's tests one at a time with `p.adjust`); Zemp's test dialogs gain a form for several variables before Zemp Studio 1.0 |
| B. Adopt the rule now, build it after the tests, in whichever minor release has room, before or after 1.0 | 0.7.0's row stands; a new form of an argument is an addition and breaks no script; the word is kept by #84 | Until then the loop; the tests' pages and dialogs are visited twice |
| C. Not adopted | Nothing to build | The loop and the detached p-values stay, and section 13's brevity comparison is made with them |

*Recommendation.* Adopt the rule, and write its text, with the points above settled, before release 0.7.0 opens. Build it at 0.7.0 only if that release's checklist shows room once its own row is planned (option A); otherwise in the first release that has (option B), the word kept under #84. Deadline: 0.7.0 for the choice between A and B; 0.11.0 for the word.

*Decided 6 November 2026 (option B).* The rule is adopted and is not built at 0.7.0, the largest release of the plan: the tests ship in their one-variable forms, and `each` is built in the first later release with room. Its text is written then, with the points listed above and the owner's two statistical choices (the adjusted p-value by default; each test's own complete rows). The word is kept under #84.

### 81. Model sets: `nested` and `each` terms in a formula, `RegFits`, and one frame of estimates for a set

Raised on 5 November 2026 from the owner's proposal (its items P1 and G2). A family of related models names the data once per model, and whether the models are nested or rivals is known only to the author. fixest writes a table of nested models in one call (`csw`, `sw`) and Stata has `nestreg`; section 13 asks that a target analysis be no longer in Zeff than in R or Stata.

```
let m = wages.reg(lwage ~ educ + nested([exper, exper2], female, tenure), se: robust)
let r = wages.reg(lwage ~ educ + each(exper, age), se: robust)
report(m, stats: [n_obs, r_squared])
m.compare()
```

`nested(a, [b, c], d)` fits the base with `a`, then with `b` and `c` added, then with `d` added; `each(a, b)` fits the base with `a` and the base with `b`, separately. The base is the rest of the formula; an argument is a variable or a list of variables added as one block, and a transformed term is added as a column first, because operators inside a call in a formula are arithmetic (section 7.2). `base_model: true` adds the base alone as the first member. One set term per formula, on the right of `~`; the call's options apply to every member. The call returns a `RegFits`, declared in `regress` as `LcaFits` is in `latent`, which the checker knows from the formula. `report` and `coef_plot` read a set through the result interface of section 8.1, rule 5, which gains a label on each member naming the terms it added; `estimates()` on a set gives one frame with a `model` column, from which any figure can be built; `compare()` on a nested set gives one test per added block by the rules of catalogue row 249 as they stand, and is refused on a set of rivals, as any non-nested comparison is. `nested` is a formula term of `regress`, beside `interact` and `event`; `each` is the word of #80. The grammar does not change. A set of outcomes on the left of `~` is a separate addition and is not part of this entry.

What the text must settle before it is built:

1. *The rows.* The proposal fits every member of a nested or rival set on the rows complete for the largest member by default (as Stata's `nestreg`; fixest's default is each model on its own rows), under `missing: listwise | pairwise`. In `correlation`, `pairwise` means the rows complete for each pair of variables; here it would mean each member on its own rows, a second meaning of the word. The rule, its two words and its default are the owner's; the "Rows" rule of section 7.2 then gains the member that drops a row for a variable it does not use.
2. *What a `RegFits` is.* No rule lets one type stand where another is expected (section 3.7, "No implicit conversion" and "Joining types"), so "behaves as a `List<RegFit>`" needs writing out: either a result type on which indexing and `for` are defined, or a `List<RegFit>` whose members carry their labels and the kind of set.
3. *A member that fails.* The whole call stops, naming the member, or the set holds a `Failed` member, as a list may (section 3.7).
4. *A block that adds nothing.* When `fe:` absorbs a block or it is dropped as collinear, the member is kept with a note; the test of that block in `compare()` has no degrees of freedom and needs a stated result.
5. *The names.* `nested`, `each`, `base_model:` (not `base:`, which `base()` uses) and `RegFits`.

| Option | For | Against |
| --- | --- | --- |
| A. Adopt for 1.0: `reg` at 0.8.0, and `ivreg`, `logistic` and `probit` each in the release after it ships | A table of nested models is one statement; the set knows how it was built, so `compare()` runs only where a valid nested test exists; nothing in the grammar or in an existing rule changes | 0.8.0's row is two functions around one hard engine piece (two-stage least squares with the weak-instrument statistics); sets add a kind of formula term, a result type, labelled columns in `report`, comparison over a set and reference comparisons against R's `anova()`, `nestreg` and fixest; the regression dialog gains a control before Zemp Studio 1.0 |
| B. Adopt the design before 1.0 and build it after, as an addition | The release plan stands; a formula term of `regress` and a new type are additions, and only the word `each` has a deadline (#84) | 1.0 ships with the list of fits written out, `report([m1, m2, m3])`, and its brevity comparison is made without sets |
| C. Not adopted | Nothing to build | The relation between the models stays in the author's head |

*Recommendation.* A, bounded: `reg` alone at 0.8.0, the other estimators following; if 0.8.0 runs late the whole moves to a later minor release, which costs nothing in compatibility. Points 2 to 5 are engineering and are settled in the text when it is written; point 1 is the owner's. Deadline: 0.8.0.

*Decided 15 November 2026, at the opening of release 0.8.0, on the recommendation of that checklist (option B), the owner's to overturn.* Under the owner's instruction of 11 November 2026 that nothing is added before 1.0 unless necessary, the design is adopted for 1.0 (the words `nested`, `each`, `base_model:` and `RegFits` kept; `each` stays reserved under #84) and built after 1.0 as an addition, which breaks no script. Point 1, the rows rule, is recommended as every member fitted on the rows complete for the largest member, `rows: common | own = common`, and stays the owner's when the text is written; points 2 to 5 are engineering. Block G of the 0.8.0 checklist builds the sets at 0.8.0 only if the owner asks for them in 1.0.

### 82. What `size:` means for a table in `export`

Raised on 5 November 2026 from the owner's proposal (its item T5). Catalogue row 165 gives `x.export(path, size: one_column | two_column | slide)` for "a result, table or plot", and section 7.6 of the Language Reference defines the three sizes for figures only. Section 1 allows no unspecified behaviour, so what `size:` does to a Word or LaTeX table must be said by 1.0. The engine exports tables without `size:` today; the option arrives with the plots of 0.7.0.

| Option | For | Against |
| --- | --- | --- |
| A. Define it: for a table, `size:` sets the font size and the total width; a table too wide for its size splits into panels with the row labels repeated, or turns the page, with a warning, by a rule stated exactly so that two machines never split a table differently | A table made for a journal column fits it; one option for figures and tables | The widths of typeset text enter a rule that must be bit-identical on every target, and they depend on the typesetting engine, which #73 decides at 0.8.0 |
| B. In 1.0, `size:` on a table is refused with a named diagnostic whose help says tables take their width from the document; table sizes follow as an addition | One diagnostic closes the gap; a later release may make the call work, which an addition may do (#64) | A table still has to be fitted to its column by hand in Word or LaTeX |

*Recommendation.* B at 0.11.0, and A afterwards, written once #73 is decided and its layout can state the splitting rule exactly. Deadline: 0.11.0.

### 83. The sentence of section 3.7 on bare words, beside the option records of section 3.1

Raised on 5 November 2026 from the owner's proposal. Section 3.1 ("Definitions") and section 3.2 write option records with bare words, `{se: robust}` and `{ se: robust, cluster: firm }`; section 3.7 ("Words and types") says that a bare word "exists only as an argument whose parameter lists its allowed values" and is an unknown name anywhere else (Z0101). The two cannot both hold. The engine has followed section 3.1 since step 4 of release 0.5.0 (`docs/decisions.md`, 30 October 2026, "Option records hold words"): inside a record literal, a bare name that is no binding and that some parameter accepts as a word is that word, checked against the parameter's list when the record is spread into a call; a name no parameter accepts stays Z0101, so `{se: robst}` is caught where it is written. The sentence of section 3.7 was not amended then.

| Option | For | Against |
| --- | --- | --- |
| A. Amend the row of section 3.7 to say what the engine does, and what a variable name in such a record is (`cluster: firm` in section 3.2), the change logged in `docs/spec_changes.md` | The text agrees with itself and with the engine; options can be named once and spread, `wages.reg(f, ..opts)`; no behaviour changes | None found |
| B. Keep section 3.7 and remove the word from the examples of sections 3.1 and 3.2 | The rule on bare words keeps one clause | An option that takes a word could no longer travel in a record, and the tests of 0.5.0 that spread one would be withdrawn |

*Recommendation.* A, a correction of the text with no change to the engine. Deadline: 0.7.0.

*Decided 6 November 2026, as recommended (option A).* The row "Words and types" of section 3.7 is corrected (logged in `docs/spec_changes.md`). Checking the engine for the correction showed what a variable name in such a record is: a bare name that is neither a binding nor a word is refused (Z0101), since a record literal has no frame to look a variable up in; a record carries a variable only through a binding that holds a `Var`, such as the variable of a loop over `vars(...)`. The example of section 3.2, `{ se: robust, cluster: firm }`, therefore reads as a record only where `firm` is such a binding; whether a variable should be writable in an option record is #87.

### 84. Reserving `each` and `peek` before `core` freezes

Raised on 5 November 2026 while the owner's proposal was checked. `core` freezes at 1.0 (Engine and Project Specification, section 8.1, rule 1) and a `let` cannot reuse a `core` name (section 3.6), so a word added to `core` after 1.0 breaks every 1.x script that binds it. #80 and #81 need `each` as a `core` name. The proposal's `.peek()` (its item L1: print the receiver through the display interface and pass it on unchanged, so that a chain can be looked into without being broken) would be a `core` name too; it is not among the items the owner adopted on 5 November. Section 3.6 already keeps a list of words "reserved for later use" (`type`, `while`, `match` and five more), so that a later release can give them meaning without a new edition.

| Option | For | Against |
| --- | --- | --- |
| A. Each word joins `core` as a function by 0.11.0, which means its feature is built by then | No reserved word that does nothing | The deadline of a word becomes the deadline of a feature, in releases that are already full |
| B. At 0.11.0, a word whose feature has not shipped joins the words reserved for later use; the feature then arrives in any minor release, before or after 1.0 | The deadline is met by one line of the text and the lexer; no script can bind the word in the meantime, so making it a function later breaks nothing | The 26 reserved words of sections 3.1 and 3.4 become 27 or 28; until the feature ships, a column named `each` or `peek` is written in backticks; a word reserved and never used is a small cost that stays |
| C. Neither: a feature that misses 1.0 takes another spelling or waits for an edition | Nothing to do now | `each` is the word both #80 and #81 are designed around |

*Recommendation.* B, as the fallback at 0.11.0 for whichever word has not shipped by then: `each` if #80 or #81 is adopted, `peek` only if the owner wants `.peek()`. Deadline: 0.11.0.

---

## Raised at the opening of release 0.7.0

Added 6 November 2026, from the planning of `docs/releases/0.7.0.md` and its first step.

### 85. The Games–Howell comparison in `posthoc`

Raised on 6 November 2026. Catalogue row 195 gives `fit.posthoc(method: tukey | bonferroni | holm = tukey)`, and `anova` is Welch's by default (row 171). Tukey's honest significant difference assumes equal variances, so after the default ANOVA the default comparison rests on an assumption the ANOVA did not make. At 0.7.0 the owner's ruling stands: `tukey` is Tukey's HSD on the pooled variance, as R's `TukeyHSD`, with a note that the ANOVA did not assume equal variances, and `bonferroni` and `holm` use pairwise Welch tests. The Games–Howell comparison is the unequal-variance counterpart of Tukey's (the studentized range of `dist.tukey` with Welch's degrees of freedom for each pair), and is what SPSS and the `rstatix` package of R offer beside it.

| Option | For | Against |
| --- | --- | --- |
| A. Add `games_howell` to `method:`, the default staying `tukey` | Completes the pair of defaults; needs no new distribution | A fourth word in a frozen list; one more reference comparison (`rstatix::games_howell_test`, a package the reference library does not hold) |
| B. Add it and make it the default after a Welch ANOVA | The default comparison then matches the default ANOVA | The default would depend on an earlier argument, and R's and Stata's users expect Tukey |
| C. Not added | Nothing to build | The note stays as the only guard |

*Recommendation.* A, in the first release after 0.7.0 with room. Deadline: 0.9.0, so that the word is settled well before the names freeze.

**The owner's decision (21 November 2026, at the opening of release 0.9.0).** The owner said "go" to the checklist of `docs/releases/0.9.0.md` without changing its answers, so its recommendation is taken under his standing instruction of 12 November 2026, his to overturn. Option A: `games_howell` joins `method:` of `posthoc`, the default staying `tukey`; built at step 29.

**Status (23 November 2026).** Built at step 29 of Zeff 0.9.0, as decided. `method: games_howell` follows the one-way form of `anova` and takes each group's own variance whichever F was computed; after an `anova` of a formula, whose means have one residual variance, it is refused in words. The note that `tukey` prints after Welch's `anova` now names it. The reference is the formula in base R, checked against `rstatix::games_howell_test` where that package is installed; the package is not added to the reference library. Deadline for the owner to overturn: 1.0, when the names freeze.

### 86. Folding, snippets and the statement ranges behind **Run** in the language server

Raised on 6 November 2026. Section 12 of the Engine and Project Specification says 1.0 ships completion, signature help, hover, live diagnostics, formatting, highlighting, running and folding, with snippets in its table; the row of 0.7.0 in section 14 names completion, signature help, hover and live diagnostics, and no later row names the rest. Formatting through the server is `zeff fmt` and is built at 0.7.0 by the owner's word of 6 November. Folding, snippets and the ranges of the current statement or chain, which **Run** needs, have no release.

| Option | For | Against |
| --- | --- | --- |
| A. At 0.8.0, with Zemp's alpha, which is the first interface to use them | Built when something reads them; 0.7.0 keeps its row | 0.8.0's row gains engine work it does not name |
| B. At 0.7.0, with the rest of the server | The server is finished in one release | 0.7.0 is already the largest release of the plan |
| C. At 1.0.0 | Nothing moves | Zemp's alpha and beta would run statements by ranges of their own |

*Recommendation.* A. Deadline: 0.8.0.

*Decided 15 November 2026, as recommended (option A).* Built at step 8b of `docs/releases/0.8.0.md`: folding ranges, a fixed list of snippets, and the range of the statement or chain at a place, which Zemp's **Run** reads.

### 87. A variable name in an option record

Raised on 6 November 2026, from the correction of section 3.7 under #83. Options can be named once and spread, `wages.reg(f, ..opts)`, and a word travels in the record (`{ se: robust }`). A variable does not: `let opts = { se: robust, cluster: firm }` is refused (Z0101), because a record literal stands outside any frame's method and has no frame to look `firm` up in (section 3.7), and `vars(firm)` is a `Vars`, which a `Var` parameter such as `cluster:` refuses (Z0401). Today a record carries a variable only through a binding that holds a `Var`, such as the variable of a loop over `vars(...)`. The most common option record of applied work, robust errors clustered by one variable, therefore cannot be written once and reused, and the example of section 3.2 shows exactly that record.

| Option | For | Against |
| --- | --- | --- |
| A. A `Vars` of exactly one variable is accepted where a parameter takes a `Var`, directly or through a spread, so the record is written `{ se: robust, cluster: vars(firm) }` | Small; explicit; `vars(...)` is already how a variable is named outside a frame; a misspelt word in a record is still caught where it is written | The record of section 3.2 gains `vars( )`; a `Vars` of two variables given to a `Var` needs its own message |
| B. Inside a record literal, a bare name that is neither a binding nor a word is kept as a variable name and checked against the frame when the record is spread | The record reads as section 3.2 writes it | A misspelt word (`{ se: robst }`) is no longer caught where it is written but at the spread, as an unknown variable; the rule that a variable is named only inside a frame's method gains an exception |
| C. Keep the refusal; correct the example of section 3.2 | Nothing to build | Options with a variable are repeated at every call |

*Recommendation.* A. Deadline: 0.8.0, when instrumental-variable models bring more calls that share their options.

*Decided 15 November 2026, as recommended (option A).* Built at step 4 of `docs/releases/0.8.0.md`; a `Vars` of two variables given to a `Var` parameter gets a message of its own; the example of section 3.2 is checked at the step and corrected if it needs `vars( )`, logged.

---

## Raised from the study of R packages

Added 7 November 2026.

### 88. Functions brought into 1.0 after the study of R packages

Raised on 7 November 2026. The owner asked for 135 R packages, from `data.table` and `fixest` to `sf` and `rugarch`, to be set against the catalogue. By their main verdict 25 are answered by 1.0, 34 by a dated release, 32 by the undated roadmap, 22 are gaps and 22 belong outside the engine. Almost everything missing can wait, because after 1.0 a new function is an addition that breaks no script (section 14, Version numbers). The question was which functions a user of 1.0 would miss on the first day, where the machinery exists by the release named and no new engine piece is needed.

| Item | Release | Why before 1.0 |
| --- | --- | --- |
| `margins` (catalogue row 260), for `reg`, `logistic` and `probit` | 0.9.0 | With `logistic` and `probit` and no marginal effects, the quantity applied researchers report could be had only one coefficient at a time through `derive`; Stata's `margins` and R's `marginaleffects` are the references |
| `roc` and `confusion` (rows 515 and 516) | 0.9.0 | The area under the curve and the classification table are standard output after a binary model (Stata's `lroc` and `estat classification`); both are small and exact |
| `ar_test` (row 256) | 0.8.0 | `first_stage` reports that instruments are weak, and 1.0 offered no inference that stays valid when they are; the Anderson–Rubin test and confidence set are that inference |
| `dist.weibull`, `dist.logistic`, `dist.hypergeometric` (new rows 531 to 533) | 0.10.0 | `dist` had no roadmap row, and 1.0's design goal rests on simulation; all three are closed forms on the special functions of 0.3.0, and `fisher_test` computes the hypergeometric already |
| `mcnemar_test` (row 508) | 0.10.0 | `t_test` has a paired form and proportions had none |
| `merge(join: semi \| anti)` (row 115, a new value of an option) | 0.10.0 | "The rows that found no match", as a frame, is an everyday step of cleaning |

| Option | For | Against |
| --- | --- | --- |
| A. The three of 0.9.0 only | The strongest case, at the least cost: three rows and about three steps | The weak-instrument remedy and the small additions wait for 1.x |
| B. All nine items | 1.0 answers what its own models raise: effects after a binary model, inference under weak instruments, paired proportions, unmatched rows | Eight more functions and about six steps before 1.0, against risk R1 (scope) |
| C. None | The plan of 16 October stands untouched | A logit without marginal effects in a release named "Binary outcomes" |

*Recommendation.* A as the minimum, B as the owner's choice.

*Decided 7 November 2026.* Option B: all nine items. The catalogue, sections 8.1 (rule 5), 8.2, 9.2, 14 and 15 of the Engine and Project Specification and section 11 of the Zemp Specification record it; 1.0 is 267 functions, with 266 on the roadmap and 533 in all. No release before 0.8.0 changes.

*Left for the releases that build them.* For `margins`, when 0.9.0 opens: the effect of a category and of a 0/1 regressor (a difference between predictions, as Stata, or a derivative), whether the standard errors treat the sample's regressors as fixed (Stata's default) or as sampled, and what `at:` holds the other regressors at. For `ar_test`, when 0.8.0 opens: the form of the test under robust and clustered errors and with more than one instrumented regressor. For the three distributions the argument names follow R's meaning, `dist.weibull(shape, scale)`, `dist.logistic(location, scale)` and `dist.hypergeometric(successes, failures, draws)`, a ruling the owner may overturn before 0.10.0.

*The definitions of `margins`, decided 21 November 2026 at the opening of release 0.9.0* (question 2 of `docs/releases/0.9.0.md`, taken on the recommendation under the owner's standing instruction when he said "go"). (a) The effect of a Bool or 0/1 regressor, and of each level of a category against its base, is the difference between the predictions at the two values, as Stata's `margins` and R's `marginaleffects` give it, not a derivative. (b) The standard errors are by the delta method with the sample's regressors held fixed, as both give them by default. (c) `type: ame`, the default, averages the effect over the rows as they are; `type: mem` takes it at the means of the regressors; a variable named in `at:` is set to the values given and every other keeps its observed values (ame) or its mean (mem). (d) The derivative is taken through every term that is a function of the variable (#195, option A). `logistic` and `probit` are `glm` since #112.

*Offered the same day and declined.* Odds ratios as an option on the results of `logistic`; two-way clustering, `cluster:` taking two variables, before 1.0 (the Language Reference, section 6, keeps it for after 1.0); and `kappa` at 0.10.0. Each stays where the specification had it. The Games–Howell comparison is #85, still due at 0.9.0.

*Not decided here.* The study's proposals for after 1.0 (survival analysis, additive models, time-series tests and volatility models, a `spatial` package, networks, about eighty candidate functions and an order of ten method releases after 1.11) are not recorded in this register; they wait for the owner's word.

### 89. A coded error from a Zeff-written built-in

Raised on 7 November 2026 at step 4.4 of release 0.7.0. `quantile_group`, written in Zeff, refuses `n:` missing, `n:` below 1 and `type:` outside 1 to 9 with `error(message)`, which prints as `error[Z0001]` at the call; the Rust form gave Z0406 (a required argument missing) and Z0401 (an argument of the wrong kind), the codes the pages of the other functions use for the same mistakes, with a label on the offending argument. `winsorize` refuses a share outside 0 to 0.5 the same way since 0.6.0. Open decision #70 gave `warn` and `note` a `code:` and left `error` without one, since an error of the script is Z0001 by definition (section 5.4); a built-in's body is the engine's, not the script's, and its errors stand for the engine's.

| Option | For | Against |
| --- | --- | --- |
| A. `error(message, code:)` in a built-in's file only, the code one of the engine's error codes with an explained page, checked at the build as the warning codes are; a project's `error` stays Z0001 | The pages of Zeff-written functions list the codes the Rust ones do, and `--format json` carries them; nothing changes for a script | One more rule that holds in a built-in's file and not in a project; the label still points at the call, not at the argument, unless `error` also takes the argument's name |
| B. Argument checks in the `fn` line: a parameter's allowed range or list written in the signature (`n: Int >= 1`), refused by the checker and the interpreter with Z0401 | The check is declared, not programmed, and the message is the engine's own with the label on the argument | New grammar for signatures, and only the simple checks are expressible |
| C. Leave it: a Zeff-written built-in's argument errors are Z0001 | Nothing to build | The codes differ from the Rust functions' for the same mistakes, which the known-issues list must keep saying |

*Recommendation.* A, at 0.8.0 (`zeff explain` and `zeff report-bug` arrive there, and both read codes), with the argument's name as an optional third piece so that the label can point at it. Deadline: 0.8.0.

*Decided 15 November 2026, as recommended (option A), with the argument's name as an optional third piece.* Built at step 7a of `docs/releases/0.8.0.md`; `quantile_group` and `winsorize` give Z0401 and Z0406 as their Rust forms did, and their pages list the codes.

### 90. Variable ranges in a Zeff-written built-in

Raised on 7 November 2026 at step 5b of release 0.7.0, where `drop_missing` was to move to Zeff with `join` writing its note. A `Vars` argument reaches a body as the names written, a range among them as its token, `id..z`; a Rust body resolves the token against the frame (`resolve_vars`, Z0401 for a range that runs backwards), but a body written in Zeff has no function that does, and `split` (step 6) with a search of `data.variables()` would be a reimplementation in every such body. `drop_missing`, `select`, `drop`, `row_mean` and every verb over several variables meets the same token.

| Option | For | Against |
| --- | --- | --- |
| A. The interpreter resolves a `Vars` argument against the frame before binding it to a Zeff body's parameter, when the function is a frame method or declares the enclosing frame: ranges expanded by position, each name once, Z0101 and Z0401 as the Rust bodies give them | Every body sees plain names, `for v in vars` and `name_of(v)` work, and the messages are the engine's, naming the frame | A `Vars` parameter means two things, written names in a script's function and resolved names in a built-in's; a built-in that wants the token (none today) could not have it |
| B. A function `variables_of(data, vars)` giving the resolved list of names as texts, called by the body | No change to what a parameter means | Every body must call it first, and the backwards-range error is still the body's to raise, as Z0001 until #89 |
| C. Leave it: a verb over variables stays in Rust | Nothing to build | `drop_missing` and its kind never move |

*Recommendation.* A, at 0.8.0 with #89, since both are what `drop_missing` needs and both touch the loader of Zeff-written built-ins. Deadline: 0.8.0.

*Decided 15 November 2026, as recommended (option A).* Built at step 7b of `docs/releases/0.8.0.md`, with #89; `drop_missing` moves to its Zeff body there if the two pieces suffice, and otherwise the reason is recorded.

### 91. Hypothesis and decision lines in test output

Raised on 8 November 2026 by the owner, before step 7 of release 0.7.0 builds `TestResult` and the first test prints. Every hypothesis test Zeff prints would state the hypothesis it tests and the one it accepts when the null is rejected, in words and, where a standard symbolic form exists, in symbols, with a decision line beside the p-value. The reader of a log, a Zemp card or a pasted result should never have to infer the null from the name of the test, and a student should never read "the data are normal" from a test that cannot say so. Among mainstream packages, Stata prints H0 and all three alternatives for `ttest`; R prints the alternative only; SPSS prints a null and a decision for its nonparametric tests; SAS, Minitab's normality tests and the Python libraries print neither.

**The rules proposed** ("test" means every function whose result is a `TestResult`).

- R1, position: the hypothesis block follows the header line and precedes the data table, indented two spaces as the tables are.
- R2, both hypotheses, H0 first: exactly two lines, `H0:` and `H1:`, written with the digit, not a subscript, so the block is identical in a terminal, a log file and a Zemp card.
- R3, population, never sample: every hypothesis is a statement about the population or the data-generating distribution. "Population mean", "population proportion", "drawn from", "in the population" or "distribution" appears in every H0; a hypothesis never says a variable "is normal", "has mean 0" or "is independent of" without that qualifier.
- R4, words, then symbols: each line gives a sentence, then, after at least two spaces, the symbolic form where one is standard; every symbol is defined by the sentence beside it. Tests without a standard symbolic form (Shapiro–Wilk, Kolmogorov–Smirnov, Kruskal–Wallis, Fisher) print the sentence only.
- R5, names from the call: the variable's name, the labels of `group:` in label order, and the names of `a` and `b` in the paired form; the symbols use the same labels, as `μ(male) − μ(female)`.
- R6, sign convention: a difference is taken in the order of the printed estimate, first group label minus second, or `a − b` in the paired form, so the H0 line, the `difference (…)` line and the sign of the statistic agree.
- R7, H1 follows the alternative tested: H1 states the alternative the p-value was computed for and no other; never the three alternatives together as Stata prints them. The alternative is `≠` unless the call says `alternative: less` or `alternative: greater`, when H1 uses `<` or `>` and the sentence says "is less than" or "is greater than". Tests with no directional form print their fixed H1.
- R8, the decision line: after the statistic line, `decision at α = <alpha>: reject H0 (p <value>)` or `decision at α = <alpha>: fail to reject H0 (p <value>)`, the p-value printed as the statistic line prints it. The words "accept H0", "H0 is true", "the data are normal", "the means are equal" and "no effect" never appear. α is `1 − level` where the test takes `level:`; where it does not, α is 0.05 (the second question below).
- R9, one block per test, none per row: a test that prints several rows each with its own p-value (the factorial `anova` table) prints no block and no decision line; one line above the table states the family, `each row tests H0: the population effect of the term is zero, against H1: it is not`.
- R10, printed layout: the block is part of the printed layout fixed within an edition (Language Reference, section 3.7, "Stability"); adopted before 0.7.0 ships, so no shipped layout changes, and every example that shows a test result is updated at the same time, since `zeff test` compares output exactly.
- R11, the result value: `TestResult` gains `h0: Str` and `h1: Str`, holding the sentences, and `alternative: two_sided | less | greater`; `summary()` prints them as the block does, Zemp shows them in a card's hover text, and `report()` may put them in a table note.

**The hypotheses by test.** The sentences are the normative text, with `<var>`, `<a>`, `<b>`, `<g1>`, `<g2>`, `<mu>`, `<p>` and `<dist>` filled from the call; symbols for the two-sided case, R7 giving the one-sided forms.

| Function and form | H0 | H1 (two-sided) | Symbols | Directional |
| --- | --- | --- | --- | --- |
| `t_test(var, mu:)` | the population mean of `<var>` equals `<mu>` | the population mean of `<var>` differs from `<mu>` | `μ = <mu>` · `μ ≠ <mu>` | yes |
| `t_test(var, group:)` | the population means of `<var>` are equal in `<g1>` and `<g2>` | the population means of `<var>` differ between `<g1>` and `<g2>` | `μ(<g1>) − μ(<g2>) = 0` · `≠ 0` | yes |
| `t_test(a, b, paired: true)` | the population mean of the paired differences `<a> − <b>` is 0 | the population mean of the paired differences `<a> − <b>` is not 0 | `μ_d = 0` · `μ_d ≠ 0` | yes |
| `wilcoxon_test(var, mu:)` | the distribution of `<var>` is symmetric about `<mu>` | the distribution of `<var>` is not symmetric about `<mu>`; values tend to lie on one side of it | none | yes |
| `wilcoxon_test(var, group:)` | the distribution of `<var>` is the same in `<g1>` and `<g2>` | values of `<var>` in `<g1>` tend to be larger or smaller than in `<g2>` | none | yes |
| `wilcoxon_test(a, b, paired: true)` | the distribution of the paired differences `<a> − <b>` is symmetric about 0 | the paired differences `<a> − <b>` tend to lie on one side of 0 | none | yes |
| `anova(var, group:)`, one-way | the population means of `<var>` are equal in all `<k>` groups of `<group>` | at least one group of `<group>` has a different population mean of `<var>` | `μ₁ = μ₂ = … = μ_k` · `not all equal` | no |
| `anova(y ~ a * b)`, factorial | one line above the table (R9); no block | | | no |
| `kruskal_test(var, group:)` | the distribution of `<var>` is the same in all `<k>` groups of `<group>` | at least one group of `<group>` tends to give larger or smaller values of `<var>` | none | no |
| `chi2_test(a, b)` | `<a>` and `<b>` are independent in the population | `<a>` and `<b>` are associated in the population | none | no |
| `chi2_test(var, p:)` | the population proportions of `<var>` equal the given `p` | at least one population proportion of `<var>` differs from the given `p` | `π = p` · `π ≠ p` | no |
| `fisher_test(a, b)` | `<a>` and `<b>` are independent in the population | `<a>` and `<b>` are associated in the population | 2 × 2 only: `OR = 1` · `OR ≠ 1` | 2 × 2 only |
| `proportion_test(var, p:)` | the population proportion of `<var>` equals `<p>` | the population proportion of `<var>` differs from `<p>` | `π = <p>` · `π ≠ <p>` | yes |
| `proportion_test(var, group:)` | the population proportions of `<var>` are equal in `<g1>` and `<g2>` | the population proportions of `<var>` differ between `<g1>` and `<g2>` | `π(<g1>) − π(<g2>) = 0` · `≠ 0` | yes |
| `shapiro_test(var)` | `<var>` was drawn from a normally distributed population | `<var>` was not drawn from a normally distributed population | none | no |
| `ks_test(var, dist)` | `<var>` was drawn from the `<dist>` distribution | `<var>` was not drawn from the `<dist>` distribution | none | no |
| `ks_test(var, dist, estimated: true)` | `<var>` was drawn from a `<dist family>` distribution with unspecified parameters | `<var>` was not drawn from any `<dist family>` distribution | none | no |

The rank-sum and signed-rank sentences avoid "median" and "location shift" on purpose: the rank-sum test is a test of stochastic dominance under its general null and tests a shift in location only under the extra assumption of a common shape, which Zeff does not check (R's "true location shift is equal to 0" assumes it without saying so). Welch's ANOVA tests the same H0 as the pooled test, so the sentence is the same and the header names the variant. `ks_test` with `estimated: true` is the Lilliefors form, whose H0 is about the family. π is the population proportion, never the sample proportion `p̂` or the argument `p:`.

**Two additions to the catalogue the rules need, both within 0.7.0.** First, `alternative: two_sided | less | greater = two_sided`, shared, on `t_test`, `wilcoxon_test`, `proportion_test` and `fisher_test` (2 × 2 only; a new error code on a larger table): `less` means the first-named quantity is less than the second or than the reference value in the convention of R6, so `t_test(lwage, group: female, alternative: less)` tests `μ(<g1>) − μ(<g2>) < 0`; the p-value, the one-sided interval (as R gives it) and H1 follow it. No catalogue row carries the argument today. Second, α for the six tests that print no interval and take no `level:` (`shapiro_test`, `ks_test`, `chi2_test`, `fisher_test`, `anova`, `kruskal_test`):

| Option | For | Against |
| --- | --- | --- |
| A. α fixed at 0.05 in the decision line; no new argument | Nothing added; 0.05 is what every reader expects | A reader working at 0.01 recomputes by eye |
| B. A shared `alpha: Float = 0.05` on every test, `level:` derived as `1 − alpha` where an interval prints | One name for the threshold everywhere | Two arguments that must agree on `t_test` and `proportion_test`; a planned signature changes |
| C. `level:` on the six tests as well, used only for the decision line | A name already in the table | "Confidence level" is an odd name for a test that prints no interval |

The owner recommends A for 0.7.0, revisited with #15 (power and sample size, 1.9), where α becomes an argument in its own right and one convention can serve tests and power functions together. Whether to print the decision line at all: the American Statistical Association's 2016 statement and APA's seventh edition both discourage reporting a test as a bare reject/retain decision; the line is kept because Zeff's first users are students taught the decision rule, and the p-value prints beside the verdict so that neither displaces the other. A project setting `decision: false` could suppress the line; it is not proposed for 1.0 and is recorded here so that it is not lost.

**Two gaps the proposal exposes, for the owner.** (a) `shapiro_test` is catalogued "for 3 to 5,000 observations", and what happens above 5,000 is stated nowhere: an error as R's `shapiro.test` gives, or the first 5,000 non-missing rows with a note (the proposal's example takes the latter and cites Z0415, which is already the code of a restriction `test` cannot use, so a new code is needed either way). (b) R5 names groups by their labels; the shipped `female` is a 0/1 integer with no value labels, so the printed form of an unlabelled group variable (`female=0` and `female=1`, as a category's indicator columns are named (`docs/decisions.md`, 23 October 2026), or the values alone) must be fixed before the first snapshot.

**The example** (illustrative figures, consistent with the example of the Language Reference, section 10.2):

```
t_test · Welch two-sample · lwage by female
  H0: the population means of lwage are equal in male and female        μ(male) − μ(female) = 0
  H1: the population means of lwage differ between male and female      μ(male) − μ(female) ≠ 0

  group      N      mean      sd
  male    9,604    3.214   0.512
  female  8,597    3.061   0.498

  difference (male − female) = 0.153   95% CI [0.138, 0.168]
  t = 20.42   df = 18,074.2   p < 0.001
  Hedges' g = 0.30   95% CI [0.27, 0.33]
  decision at α = 0.05: reject H0 (p < 0.001)
```

```
shapiro_test · lwage
  H0: lwage was drawn from a normally distributed population
  H1: lwage was not drawn from a normally distributed population

  N = 2,836   (35 rows dropped for NA in lwage)
  W = 0.9997   p = 0.928
  decision at α = 0.05: fail to reject H0 (p = 0.928)
```

| Option | For | Against |
| --- | --- | --- |
| A. Both hypotheses, the one alternative tested, and a decision line; α = 1 − level, or 0.05 where there is no `level:` | Self-describing output for students and logs; the null is never phrased at sample level; a one-sided test shows exactly what was tested | Four lines longer per test; a decision line some referees consider dichotomous |
| B. The alternative only, as R | The shortest addition | The null, under which the p-value is computed, is still left to the reader |
| C. H0 and all three alternatives with three p-values, as Stata | Familiar to Stata users | Prints two tests nobody asked for, against the rule of printing what was computed |
| D. No hypothesis text, as SAS and the Python libraries | Nothing to maintain | The gap this decision exists to close |

**If A is adopted, before step 7 of 0.7.0:** `alternative:` added to the catalogue rows of `t_test`, `wilcoxon_test`, `fisher_test` and `proportion_test`, a new error code for `alternative:` on a Fisher table larger than 2 × 2, and the over-5,000 behaviour of `shapiro_test` stated in its row; the block and the decision line inserted into the `t_test` example of the Language Reference, section 10.2, and the `h0`, `h1` and `alternative` fields added to `TestResult` in section 7; the `t_test` page of the help examples updated (the example, `alternative:` under ARGUMENTS, one sentence in DESCRIPTION); one golden-output case per row of the table above in the testing handbook, with one `less` and one `greater` case; each change logged in `docs/spec_changes.md` as a correction found while coding, since the layout is adopted before any test ships.

*Recommendation.* The owner's: A, with α as option A of the second question, adopted before step 7 so that no shipped layout changes; α revisited with #15 at 1.9. Deadline: 0.7.0, before step 7.

**The owner's decision (8 November 2026).** A, as recommended, with α fixed at 0.05 where a test takes no `level:`. Applied the same day, before step 7: the `t_test` page of the help examples (the block, the decision line, `alternative:` and the three fields of the value) and the definition of done of the testing handbook (a snapshot of the block for every form of the table, with one `less` and one `greater` case); the catalogue rows 169, 170, 174, 175 and 176, the Language Reference (the shared-arguments table and the `Result` row of section 7; the example of section 10.2) and the log in `docs/spec_changes.md`, the same day. The two refusals (`alternative:` on a Fisher table larger than 2 × 2; `shapiro_test` above 5,000) receive their codes at steps 13 and 14, as the release's ruling on provisional codes says. Layout rulings taken in applying it: the `H0:` and `H1:` lines are indented two spaces as the tables are (R1) and the decision line stands flush with the statistic lines it follows; the one-sided interval prints as R gives it, `[0.136, Inf)`. Of the two gaps, the owner decided the first on 8 November 2026: `shapiro_test` refuses more than 5,000 non-missing observations, as R does, with a message that names `ks_test` and `qq_plot` (the first 5,000 rows would have made the result depend on row order); the code is given in the catalogue row. The second he decided the same day, as recommended: a group variable without value labels prints its groups as `female=0` and `female=1`, the form a category's indicator columns already take, in the sentences and in the symbols (`μ(female=0) − μ(female=1) = 0`).

### 92. A `TestResult` built from a body written in Zeff

Raised on 8 November 2026 at step 9 of release 0.7.0, where `t_test` was to be written in Zeff (the checklist's rows for steps 9, 10 and 12 say the same of `proportion_test`, `chi2_test`, `kruskal_test` and `p_adjust`). A body written in Zeff can compute every statistic a test needs (`mean`, `sd`, `dist.t`), but it has no way to make a `TestResult`: the value's hypotheses, header lines, group table, estimate and effect size with their intervals, statistic, degrees of freedom, p-value, level, α and the count of rows dropped are set by Rust constructors alone, and the two coded errors every test shares (Z0411, Z0412) cannot be raised from Zeff until #89. `t_test` therefore ships in Rust, its catalogue row saying so, and the shared pieces of section 6 are Rust helpers (`stats::testing`) every test calls.

| Option | For | Against |
| --- | --- | --- |
| A. A constructor `test_result(method:, title:, h0:, h1:, symbols:, statistic:, df:, p_value:, estimate:, ci:, effect:, n:, dropped:, level:)` in `results`, callable from a built-in's file only, with #89 for the coded errors | Each test's body becomes a readable Zeff file, as the specification wants; the printed layout stays the engine's | A wide signature to keep in step with `TestResult`; a body still needs the weighted kernels of #72 |
| B. Tests stay in Rust; the Zeff-written plan applies to functions that return plain values | Nothing to build; the nine tests of 0.7.0 ship on time | Section 9.1's count of Zeff-written functions falls by the tests' number |

*Recommendation.* B for 0.7.0, so that the tests ship; A decided at 0.8.0 together with #89 and #72, since the three pieces are what a test written in Zeff needs. Deadline: 0.8.0.

*Decided 15 November 2026 (option B).* Under the owner's instruction of 11 November 2026 that nothing is added before 1.0 unless necessary, the tests stay in Rust and no constructor of a `TestResult` is added to the language before 1.0; the catalogue's `layer` column says Rust for them. The question returns after 1.0 if a function planned in Zeff needs it.

### 93. The interval of an effect size: noncentral t or large-sample normal

Raised on 8 November 2026 at step 9 of release 0.7.0. The help example of `t_test` prints Hedges' g with a 95% interval and names R's `effectsize` package as the reference. `effectsize` computes that interval from the noncentral t distribution by default (the pivot of Steiger and Fouladi); the engine's `dist` has no noncentral t, which is a numerically delicate piece (Lenth's algorithm AS 243, a series of incomplete beta terms) and an `[engine]` step of its own. `t_test` prints the large-sample interval of Hedges and Olkin (1985), g ± z · √((n₁ + n₂)/(n₁ n₂) + d²/(2(n₁ + n₂))) · J, which agrees with the noncentral interval to the second decimal for samples above a few dozen and differs in the third decimal for small ones; its page says so.

| Option | For | Against |
| --- | --- | --- |
| A. Keep the large-sample interval, as the page documents | Nothing to build; exact in base R for the reference | Not what `effectsize` prints for small samples |
| B. Add the noncentral t to `dist` (`dist.t(df:, ncp:)`) and compute the interval by the pivot | Matches `effectsize`'s default; `power.t_test` (1.9) needs the same distribution | An engine step of a few days; a reference without base R |

*Recommendation.* A at 0.7.0; B at 0.8.0 or with `power` at 1.9, whichever comes first, the interval switching then with a note in the release notes. Deadline: 0.8.0.

*Decided 15 November 2026 (option B, deferred).* The large-sample interval stands, as the page of `t_test` documents; the noncentral t (and the noncentrality of `dist.chi2`, `dist.f` and `dist.beta`, section 4 of #99) is built with the `power` family in the release of #113's plan that builds it, the intervals switching then with a note in the release notes. Nothing is built at 0.8.0.

### 94. Reading a `TestResult`'s fields as values

Raised on 9 November 2026 after step 12 of release 0.7.0, when the owner asked how a test's statistic is read from a bound result. The types table of section 3 of the Language Reference says a `TestResult` "carries `statistic`, `df`, `p_value`, `estimate`, `ci`, `effect_size` and `n_obs`, and, from open decision #91, `h0` and `h1` and `alternative`", and the Result row of the same table names the result interface as the way every result is read. The engine as built at step 7 gives these routes and no other:

- `fit.p_values()` gives the p-value and `fit.n_obs()` the rows used, as numbers; `fit.ci()` gives the intervals as a table, the effect size's among them.
- `fit.estimates()` gives one frame with the columns `term`, `estimate`, `std_error`, `statistic`, `p_value`, `ci_lower` and `ci_upper`, one row for the estimate (`difference (male − female)`, `proportion (Wilson)`, `chi2`). The statistic is read from it as `fit.estimates().max(statistic)`, the frame aggregates being the only way from a one-value column to a number: `col[1]` is refused (Z0221, a Column is not a list) and `first(col)` too (Z0401), and neither message names the aggregate.
- `fit.statistic`, `fit.df`, `fit.p_value` and the rest are refused with Z0213, "reads a record field, but this is a TestResult", whose help suggests `.statistic()`, which does not exist either.
- The degrees of freedom are printed and nowhere else: no column of `estimates()`, no method. The effect size's value is printed and appears in `ci()` as an interval, but has no row in `estimates()`. `h0`, `h1` and `alternative` are printed only.

So a script that gathers several tests' statistics, degrees of freedom or effect sizes into a table of its own, or writes them into a sentence with `format` and `join`, cannot be written today, although the text promises the values. The regression results have no such gap: `coef`, `std_errors`, `p_values` and `ci` take a term and give numbers.

| Option | For | Against |
| --- | --- | --- |
| A. Field reads, `fit.statistic`, `fit.df`, `fit.effect_size`, `fit.h0`, as the types table reads | The shortest form; what the sentence of section 3 leads a reader to write | A `TestResult` is not a record, and no other result is read this way (`m.coef(educ)`, not `m.coef`); the grammar's rule that `.name` without parentheses reads a record field would gain an exception for one type |
| B. Methods in the style of the result interface: `fit.statistic()`, `fit.df()`, `fit.effect_size()`, `fit.h0()`, `fit.h1()`, `fit.alternative()`, beside the `p_values()` and `n_obs()` that exist; `estimates()` gains a `df` column and a row for the effect size (`NA` where a test has none) | One way to read every result; the Z0213 help already points to the method form; `df()` and `effect_size()` are what `posthoc` (step 19) and `report` need internally | Six method rows in the catalogue and six help pages, each with its tests; the sentence of section 3 is reworded to say the fields are read by these methods (a correction of the text, logged) |
| C. Widen `estimates()` only: a `df` column and an effect-size row, and `first` or `[1]` allowed on a column | Smallest change to the language | `fit.estimates().max(df)` is a roundabout way to a number a statistician reads from every printout; `h0`, `h1` and `alternative` stay unreachable |

*Recommendation.* B, at 0.7.0 as an `[engine]` step beside step 7's type: the methods are small, the catalogue gains six rows in `results` (the interface is `results`', not `stats`'), and `estimates()` gains its column and row under B as under C. The sentence of section 3 is corrected to name the methods and logged in `docs/spec_changes.md`; the help of Z0213 on a result names them too. Deadline: 0.7.0.

**The owner's decision (9 November 2026).** B, as recommended. Recorded the same day as step 12b of `docs/releases/0.7.0.md`, an `[engine]` step beside step 7's type: the six methods in `results`, the `df` column and the effect-size row of `estimates()`, the catalogue rows, the help pages, the correction of the sentence of section 3 with its log in `docs/spec_changes.md`, and the help of Z0213 on a result. Nothing is built until the step is taken.

### 95. `print` with several plain values on one line

Raised on 9 November 2026 after step 13 of release 0.7.0, when the owner asked for a form of `print` that does what R's `cat` does. Today `print(x, ...)` prints each value through the display interface, one after another, each on its own lines, so `print("mean =", x)` puts the label and the number on two lines, and the one-line sentence a report or a loop prints needs `join` and `to(x, Str)`: `print(join(["t = ", format(t, digits: 2), " on ", to(n, Str), " df"]))`. Section 3.3 of the Language Reference closes interpolation inside strings ("there is no interpolation, `format` and `join` build text"), so the gap is in the printing statement, not in the text functions.

| Option | For | Against |
| --- | --- | --- |
| A. `print` itself: plain values (an Int, a Float, a Str, a Bool, a Date, a DateTime or `NA`) that stand side by side share one line, separated by `sep:`, a space unless given; any other value (a frame, a list, a column, a result, a table) ends the line and prints as its own block as it does now; the line always ends, so no `"\n"` housekeeping; numbers keep the six-digit rule of section 3.8 and `format` shapes them when more is wanted | `print("mean =", x)`, `print("done:", i, "of", 10, "passes")` and `print("n", "mean", "sd", sep: "\t")` read as `cat` does and as Python, Julia and Stata's `display` print; one printing word, and every single-argument call is unchanged | `print(1, 2)` changes from two lines to `1 2`; the `print` help page, two test files, a snapshot that refuses `sep:` as an unknown option and the catalogue's usage column change with it |
| B. A second function, `say(x, ..., sep: Str = " ")`, that prints plain values on one line and refuses anything larger (Z0401), with `print` unchanged | `print` keeps its single meaning, one value per block | Two printing words, the pair of R's `print` and `cat` that the question came from, and every author must learn which to reach for |
| C. Interpolation inside strings, `"mean = {x}"` | The shortest form to write | Closed by section 3.3; a second small language inside strings; how `{x}` rounds a number reopens what `format` settles |

*Recommendation.* A, at 0.8.0 as an `[engine]` step in `core`'s display, with a space as the default separator and a one-value `Column` counted as a block, not as a plain value, so that a column and a number never print alike. The cost is small while there are no outside scripts and grows with every script that prints two values on two lines. The sentence "Several values print one after another, each on its own lines" of the `print` help page and the usage `print(x, ...)` of the catalogue are corrected with the step. Deadline: 0.8.0.

**The owner's decision (9 November 2026).** A, as recommended: a space as the default separator, a one-value column printed as a block. Built as an `[engine]` step of release 0.8.0 when its checklist is written; nothing changes in 0.7.0.

### 96. Computing with a `DateTime`, and a `Date` from its parts

Raised on 9 November 2026 after step 13 of release 0.7.0, when the owner asked what must still be added before 1.0. A check of the engine as built shows that the two date types ship in 1.0 with less than a script needs to use them.

*A `DateTime` cannot be computed with.* `import` creates the type and `datetime` parses it; the engine compares two values, takes the date part with `to(dt, Date)` and writes the hour as text with `format(dt, pattern: "%H")`, and can do nothing else. `DateTime − DateTime` is an error in 1.0 by section 3.7 of the Language Reference (it gives a `Duration` after 1.0); `days_between` is the only difference and gives 0 for two events on the same day; `min` and `max` refuse a `Date` or `DateTime` column ("needs a Bool, Int or Float"), although their catalogue rows say only "smallest non-missing value"; and no function gives the hour, minute or second as a number. A script with arrivals and departures cannot compute a waiting time, a service time, or the first and last event of a day, which is the use of the type.

*A `Date` cannot be built from numbers.* Panel and time-series files carry `year`, `quarter` or `month` as numbers. `date` parses text only, and `join` builds text from a list, not from columns, so there is no route from those columns to a `Date`; without one there is no date axis on a `line` plot and no `panel(time:)` from components. Grouping by month or quarter works only on formatted text, which sorts but does not plot.

| Option | For | Against |
| --- | --- | --- |
| A. Add both bundles at 0.8.0 as `[function]` steps on the date kernel: (1) `hour(dt)`, `minute(dt)`, `second(dt)` as Ints; `seconds_between(start, end)`, `minutes_between(start, end)` and `hours_between(start, end)` beside `days_between`, whole units truncated towards zero as `days_between` counts; `min` and `max` on a `Date` or `DateTime` column, in `aggregate` too; `datetime(x, epoch: seconds \| milliseconds)` as a third form of `datetime` for logs carrying Unix time; (2) `date(year, month, day: Int = 1)` as a second form of `date`, Ints or Int columns, an impossible date giving `NA` with a warning as `date` does; `start_of(d, month \| quarter \| year \| week \| day \| hour)` on a `Date` or `DateTime`, returning the same type, with `week` starting on Monday as `weekday` counts | The types shipped in 1.0 become usable for the data they were made for; every piece is a few lines on the kernel of 0.6.0 with an R check against `lubridate` and `difftime`; section 3.7's rule on `−` is untouched, since the `_between` functions are what it points to | Nine catalogue rows and two widened usage lines before 1.0; `start_of` with a bare word is a new shape among the date functions |
| B. Wait for `Duration` and `to_timezone` after 1.0 and add the time-of-day functions with them | One design for every length of time | 1.0 ships a `DateTime` that cannot measure a wait; the `Date` constructor has nothing to wait for |
| C. Bundle (2) only at 0.8.0, bundle (1) after 1.0 with `Duration` | The smaller change | The same hole in `DateTime` at 1.0 |

*Recommendation.* A, at 0.8.0, as `[function]` steps in `time` (and the `min` and `max` extension in `data`), each with its help page, its R reference and its catalogue row, with `hour(dt)` and `date(2026, 3, 1)` as the definition-of-done examples. The `_between` names follow `days_between`; `start_of` follows `lubridate::floor_date` in meaning and `weekday`'s Monday rule; `epoch:` reads an Int or Float column and refuses text (Z0401). Deadline: 0.8.0.

**Amended (9 November 2026).** The owner asked whether the differences could be one function with a unit rather than a family of names, and they can. Option A and the recommendation are amended so that `seconds_between`, `minutes_between` and `hours_between` are not added and the two functions that shipped at 0.6.0, `days_between` and `months_between`, are replaced by one:

`time_between(start, end, unit: second | minute | hour | day | week | month | year = day)`

Whole units truncated towards zero, as `days_between` counts, so 23 hours is 0 days and 90 seconds is 1 minute; `month` and `year` keep the clamping rule of `months_between` and `add_months` (31 January to 28 February is one month, a year is twelve such months); `second`, `minute` and `hour` need a `DateTime` on both sides and are an error (Z0401) on two dates; the other units take either type; `NA` gives `NA`; columns row by row. The unit words are singular, as in `start_of(d, month)`, so the two functions share one vocabulary, and the default is fixed at `day` and written in the usage line, never chosen from the data as R's `difftime(units = "auto")` chooses. When `Duration` arrives after 1.0, `time_between` keeps giving an Int, so no script changes type, and section 3.7's rule on `DateTime − DateTime` is untouched. Removing `days_between` and `months_between` is a breaking change, allowed in a minor release before 1.0 (section 14 of the Engine and Project Specification), to be listed in the 0.8.0 release notes with the rewrite: `days_between(a, b)` becomes `time_between(a, b)` and `months_between(a, b)` becomes `time_between(a, b, unit: month)`. The catalogue loses two rows and gains one, and bundle (1) is otherwise as raised.

**The owner's decision (11 November 2026).** "A, as amended." Both bundles enter release 0.8.0 as `[function]` steps in `time` (and the `min` and `max` extension in `data`), each with its help page, its R reference (`lubridate`, `difftime`) and its catalogue row: `hour`, `minute` and `second`; `time_between(start, end, unit:)` replacing `days_between` and `months_between`, with the rewrite in the release notes and the old names refused with a diagnostic that names the new call; `min` and `max` on `Date` and `DateTime` columns; `datetime(x, epoch:)`; `date(year, month, day)`; and `start_of(d, unit)`. The owner's instruction of the same day, that nothing be added before 1.0 unless necessary, is met: the two date types ship in 1.0 and are not usable for their data without these. The functions of 1.0 come to 276 with this decision and the merges of #97, #99 and #112. The "Start release 0.8.0" plan lists the steps.

### 97. One function with an option rather than a family of names

Raised on 9 November 2026 after step 14 of release 0.7.0, when the owner, having turned the `_between` family of #96 into one `time_between(start, end, unit:)`, asked whether other families could be reduced the same way, and then set the rule for every future addition. The tests already follow it: `t_test` carries the one-sample, two-group and paired forms, `chi2_test` independence and goodness of fit, `anova` the one-way and factorial tables, `proportion_test` one and two proportions. Four families of column and date functions do not.

**The rule.** When several functions do one thing that differs only by a unit, a statistic or a method, the catalogue has one function with a bare-word option (`unit: month`, `method: levene`, `stat: sum`) and not one name per case. A function keeps its own name when the names are distinct words every reader knows and merging them would hide the meaning behind an option: `year`, `month` and `day`; `floor`, `ceil`, `truncate` and `round`; `starts_with`, `ends_with` and `contains`; `head` and `tail`; `first` and `last`; `today` and `now`; `row_mean` and `row_sum`. A proposed catalogue row is judged against this rule before it is added, and a family that fails it is recorded here as a merge.

| Today | One function | Notes |
| --- | --- | --- |
| `add_days(d, n)`, `add_months(d, n)` (both 0.6.0) | `add_time(d, n: Int, unit: day \| week \| month \| year = day)` | The mirror of `time_between`, with the same unit words; `month` and `year` keep the clamping of `add_months`; `week` is seven days. Rewrite: `add_days(d, n)` → `add_time(d, n)`; `add_months(d, n)` → `add_time(d, n, unit: month)` |
| `quantile_group(x, n: Int, type: Int = 7)`, `bin(x, breaks: List, labels: List, closed: right \| left = right)` (both 0.6.0) | `bin(x, n: Int \| breaks: List, labels: List, closed: right \| left = right, type: Int = 7)` | Both cut a number into a category, by sample quantiles or by breaks; exactly one of `n:` and `breaks:` is given (Z0410 otherwise). With `n:` the result is a Category whose levels are the quantile groups in order, labelled `1` to `n` unless `labels:` names them, tied values sharing a group as `quantile_group` has it; the group number is `to(g, Int)`. The usage then agrees with `recipe.bin(var, n:)` of 1.5. Rewrite: `quantile_group(x, 4)` → `bin(x, n: 4)` |
| `fill_forward(x)` (0.6.0, `data`), `fill_missing(x, value)` (0.5.0, `core`), and the `fill_backward` proposed on 9 November | `fill_missing(x, value)` or `fill_missing(x, method: forward \| backward)` | One word for filling `NA`, whatever fills it; `value` and `method:` exclude each other (Z0410). The merged function lives in `data`, so that `method:` keeps `fill_forward`'s rule of filling within panel units or `by:` groups; the `core` row goes. Rewrite: `fill_forward(x)` → `fill_missing(x, method: forward)` |
| the `var_test` proposed on 9 November, `levene_test(var, group:, center: median \| mean = median)`, `bartlett_test(var, group:)` (both after 1.0) | `variance_test(var, group: Var, method: f \| levene \| bartlett = levene, center: median \| mean = median)` | Three tests of one hypothesis, equal variances across groups; `f` is the ratio of two variances with its interval (R `var.test`, Stata `sdtest`) and is refused for more than two groups (Z0401); `center:` applies to `levene` only (Brown–Forsythe when centred on the median). Nothing has shipped, so no rewrite |

Not merged, with the reason: `running_sum` and `running_product` (a `running(x, sum \| product)` reads worse than the names, and the family is two until `running_max` and `running_min` are wanted); `log2` and `log10` beside `log(x, base:)` (redundant with an existing option, but universal names in R, Stata and Excel; left to a later decision); `shapiro_test`, `ks_test` and `jarque_bera_test` (`ks_test` compares with any distribution, so a `normality_test` would misdescribe it).

| Option | For | Against |
| --- | --- | --- |
| A. The four merges at 0.8.0, as `[function]` steps, each with the merged help page, the rewrite in the release notes, and the removal of the old rows | Six catalogue rows fewer; one vocabulary with `time_between` and `start_of`; dearer with every release that ships scripts using the old names | Four shipped functions change before 1.0, a breaking change section 14 allows in a minor release |
| B. The `variance_test` merge only, since nothing has shipped, and the other three left as they are | No breaking change | Three families stay against the rule, and `bin` keeps disagreeing with `recipe.bin` |

*Recommendation.* A, at 0.8.0, in the release that also brings `time_between`, `start_of` and the amendments of #96, so that the date and column vocabulary changes once. Deadline: 0.8.0.

**The owner's decision (9 November 2026).** The rule is adopted and applies to every future addition to the catalogue; the four merges are recorded for 0.8.0 as recommended.

### 98. `anova` with every type of sums of squares that R and Stata give

Raised on 10 November 2026 after step 16 of release 0.7.0, when the owner asked whether Types I, II and III sums of squares were planned for the factorial form of `anova`. Catalogue row 171 gives `data.anova(y ~ a * b, type: 2 | 3 = 2)`, as `car::Anova`, by the decision of #12; the sequential table appears nowhere in the text. What the two reference packages give:

| | Type 1, sequential | Type 2 | Type 3, partial |
| --- | --- | --- | --- |
| R, base | `aov()` and `anova(lm)`, the default | not offered directly | `drop1(fit, . ~ ., test = "F")` |
| R, `car::Anova` | not offered | the default | `type = 3`; the user must set sum contrasts first, and nothing warns when they are not set |
| Stata `anova` | the option `sequential` | not offered | the default ("partial SS"), whatever the coding of the factors |
| Zeff, row 171 | absent | the default | `type: 3` |

So a reader of base R cannot reproduce `summary(aov(y ~ a * b))` on unbalanced data, and a reader of Stata cannot reproduce `anova y a##b, sequential`; `compare()` on two nested regressions gives one sequential F at a time, not the table.

| Option | For | Against |
| --- | --- | --- |
| A. `type: 1 \| 2 \| 3 = 2`, built with the factorial form at step 17 of 0.7.0 | Every table R and Stata print can be reproduced; the sequential sums fall out of the same fit, so the cost is small; base R's `anova(lm)` becomes a reference that needs no added package | A table that depends on the order of the terms, which the printed result must say |
| B. Types 2 and 3 only, as row 171 has it | Nothing order-dependent in the language | The default table of base R cannot be reproduced on unbalanced data |

**Type 1 as it is defined.** The terms enter in the order of the formula, the main effects in the order written, then the two-way interactions in the order written, and so on, as R's `terms` orders them and `anova(lm)` reports them; each term's sum of squares is the fall in the residual sum of squares when it joins the terms above it, and F is its mean square over the residual mean square of the full model. Stata's `anova, sequential` agrees when its terms are written in that order. Partial eta squared is SS(term) / (SS(term) + SS(residual)) under every type. The line that closes the table names the type and, under type 1, says that each term is adjusted for the terms above it and that the order of the formula matters. On a balanced design the three types agree; with one factor each of them is the classical one-way table. The default stays `type: 2`, as #12 decided.

**Noted for step 17, not part of the ruling.** The catalogue says "type 3 with sum contrasts" without saying who sets them. The reading taken is Stata's: the engine applies sum contrasts itself under `type: 3`, whatever `category(x, contrast:)` says, so that the treatment-coded table `car::Anova` prints without warning cannot be obtained; the step records the reading in `docs/decisions.md`.

*Recommendation.* A, at step 17 of 0.7.0: the usage of catalogue row 171 becomes `type: 1 | 2 | 3 = 2` and its description names the sequential table, both amended at the step and logged in `docs/spec_changes.md`; the reference is `anova(lm(...))` with the terms in both orders on unbalanced data, beside `car::Anova`; the property tests add that the type 1 sums of squares add up to the model's and that the three types agree on a balanced design. Deadline: 0.7.0.

**The owner's decision (10 November 2026).** A: "the Zeff function must cover all alternatives provided by R and Stata". `anova` gives types 1, 2 and 3. Whether the same sentence is to be a rule for every catalogued function, with a review of each function's options against R and Stata, was asked of the owner the same day and is not settled by this entry.

**Amended (10 November 2026).** The owner answered the same day: "Yes, review all". The sentence is a rule for every catalogued function: whatever alternative R or Stata gives for a procedure, the Zeff function for that procedure gives too, as a value of an option on the one function (#97) and never as a sibling name. The review of every function due by 1.0 against the rule, and the gaps it found, are #99.

### 99. Every function covers the alternatives R and Stata give: the review and its gaps

Raised on 10 November 2026, the day the owner made the sentence of #98 a rule for every catalogued function and asked for each function to be reviewed against it ("Yes, review all"). The review covers the 278 functions due by 1.0. This entry holds the rule as it was read, what was checked, and every gap found, each with the option that would close it on the existing function (#97) and a proposed release. Nothing here is scheduled until the owner rules; the function-by-function record, including the functions with no gap, is kept outside the repository with the other studies.

**The rule as it was read.** An *alternative* is another way of doing the same procedure that base R, the R package named in the function's `equivalent` column, or official Stata gives: another statistic, another way to the p-value or the interval, another estimator of a variance, another definition, or another form of the same test (one sample or two, one side or two). Not counted: a whole procedure that already has its own catalogue row on the roadmap (`sign_test`, `friedman_test`, `bootstrap`, the `power.` family and the like); arguments that only arrange R's or Stata's own output; and commands that are user-written in Stata (`esttab`, `reghdfe`, `winsor2`, `weakiv`), which are listed where the catalogue itself names them as the equivalent and marked as such. The owner may read the rule more widely or more narrowly; question 1 below asks.

**What was checked, and how far.** R: the argument lists were read from the installed library (R 4.6.0; `car` 3.1.5, `sandwich` 3.1.1, `marginaleffects` 1.0.0, `psych` 2.6.5, `nortest`, `DescTools`, `e1071`, `moments`, `emmeans`, `multcomp`, `lmtest`). The packages `fixest`, `pROC`, `AER`, `ivreg`, `dagitty`, `effectsize` and `modelsummary` are not installed, so what is said of them is from their documentation as remembered. Stata is not installed on this machine: every statement about Stata is from its manuals as remembered and none was confirmed by a run; two are flagged where the version matters. Zeff: each gap on a function already built was confirmed by running the engine and seeing the option refused; functions not yet built were read from their catalogue rows.

**Classes.** A: a method or form that analysts reach for routinely, so that its absence would be noticed in ordinary work. B: given by R or Stata but used less often, or needing a new piece of machinery. C: a difference of convention or a convenience; the proposal is a sentence in the help page saying what R and Stata do, and no option.

**1. Release 0.7.0, steps not yet built (17, 19 and the plot steps 20 to 30).** These are cheapest to settle before the step is built.

| Function | What R and Stata give that Zeff lacks | Proposed on the one function | Class |
| --- | --- | --- | --- |
| `posthoc` (step 19) | Stata `pwcompare` and `pwmean`, `mcompare()`: `noadjust`, `bonferroni`, `sidak`, `scheffe`, `tukey`, `snk`, `duncan`, `dunnett`; `oneway` prints Bonferroni, Scheffé and Šidák. R `pairwise.t.test` takes every method of `p.adjust` | `method:` gains `sidak`, `scheffe`, `hochberg`, `hommel`, `bh`, `by`, `none` beside `tukey`, `bonferroni`, `holm`; all rest on `dist.f`, `dist.tukey` and `p_adjust` | A |
| `posthoc` | Student–Newman–Keuls and Duncan (Stata `snk`, `duncan`): step-down tests on the studentised range over a falling number of means | `method: snk \| duncan`, on `dist.tukey` of step 18 | B |
| `posthoc` | Dunnett's comparison of every group with one control (Stata `dunnett`; R `multcomp`) | `method: dunnett, control: <level>`; it needs Dunnett's distribution, a two-dimensional integral, as a new [engine] piece | B |
| `posthoc` | Pooled or separate standard deviations as a choice (R `pool.sd`); today the form of `anova` fixes it | `pooled: Bool`, the default following the `anova` it comes after | B |
| `posthoc` | Pairwise rank tests after `kruskal_test` (R `pairwise.wilcox.test`) | `posthoc` accepts the result of `kruskal_test` | B |
| `anova` (steps 16 and 17) | Omega squared and epsilon squared beside eta squared (Stata `estat esize, omega epsilon`; R `effectsize`) | `effect: eta \| omega \| epsilon = eta`, partial in the factorial form | A |
| `anova` | The interval of the effect size by the noncentral F (Stata `estat esize`) | Printed once `dist.f` takes `ncp:` (section 4 below, and #93) | A |
| `anova` | A numeric covariate among the terms, the analysis of covariance (R `aov` and `car::Anova` on any linear model; Stata `anova y a c.x`) | The formula of the factorial form accepts numeric terms; settled at step 17 | A |
| `anova` | The table on a heteroskedasticity-consistent covariance (`car::Anova(white.adjust:)`) | `se:` on the factorial form, with `reg`'s values | B |
| `anova` | Repeated measures and nested error terms (R `aov(y ~ a + Error(id/a))`; Stata `anova, repeated()` with the Greenhouse–Geisser and Huynh–Feldt corrections) | A within-subject form of its own; too large for 0.7.0, proposed as a roadmap row | B |
| `anova` | Bartlett's test printed beneath the one-way table (Stata `oneway`) | None; `variance_test(method: bartlett)` of #97 gives it | C |
| `p_adjust` | `hochberg` and `hommel` (R `p.adjust`), `sidak` (Stata), `none`, and `n:`, the number of comparisons when it exceeds the p-values given (R) | `method:` gains the four words at step 19, which needs them; `n: Int` | A, `n:` B |
| `histogram`, `p.histogram` | The rule for the number of bins (R: Sturges, Scott, Freedman–Diaconis; Stata: the smaller of √N and 10 log₁₀ N); the vertical scale (Stata `density`, `fraction`, `frequency`, `percent`; R `freq`); where the first bin starts (Stata `start()`) | `bins: Int \| sturges \| scott \| fd`, the default named in the help page; `scale: count \| density \| fraction \| percent = count`; `start: Float` | A |
| `density`, `p.density` | The kernel (R: seven, Gaussian by default; Stata: eight, Epanechnikov by default) and the bandwidth rule (R `nrd0`, `nrd`, `ucv`, `bcv`, `SJ`) | `kernel: gaussian \| epanechnikov \| rectangular \| triangular \| biweight \| cosine = gaussian`; `bandwidth: Float \| nrd0 \| nrd \| sj` | A, the rules B |
| `boxplot`, `p.boxplot` | The length of the whiskers (R `range = 1.5`); the quartiles are Tukey's hinges in R's `boxplot`, type 7 in `ggplot2`, and Stata's own percentiles | `whisker: Float = 1.5`; the quartile type stated in the help page | B |
| `qq_plot` | The plotting positions (R `ppoints`: (i − 3/8)/(n + 1/4) up to ten values, (i − 1/2)/n beyond; Stata i/(N + 1)) and the reference line (R through the quartiles; Stata through the mean and standard deviation) | `line: quartiles \| moments = quartiles`; the positions stated in the help page | B |
| `scatter` | A smooth fitted curve and the band round a fitted line (R `lowess`; Stata `lowess`, `qfit`, `lfitci`) | `fit: none \| linear \| quadratic \| lowess`, `band: Bool`; `p.smooth` (1.2) already plans `loess` | B |

**2. The tests already built in 0.7.0, and `tabulate`.** Proposed for 0.8.0, beside the merges of #97 and the ruling of #93.

| Function | What R and Stata give that Zeff lacks | Proposed on the one function | Class |
| --- | --- | --- | --- |
| `t_test` | Two variables as independent samples (R `t.test(x, y)`; Stata `ttest a == b, unpaired`); today two variables need `paired: true` | `t_test(a, b)` is the unpaired form | A |
| `t_test` | A hypothesised difference other than 0 with two groups (R `mu`) | `mu:` accepted in the two-group and paired forms | B |
| `t_test` | Glass's delta (Stata `esize, glass`) | `effect: d \| g \| glass = g` | B |
| `t_test` | Welch's (1947) degrees of freedom beside Satterthwaite's (Stata `welch`); the test from summary figures alone (Stata `ttesti`) | None; a sentence in the help page | C |
| `wilcoxon_test` | Zeros in the signed-rank test: R drops them, Stata keeps them and adjusts the variance, so the two p-values differ | `zeros: drop \| keep = drop` | B |
| `wilcoxon_test` | The probability that a value of one group exceeds a value of the other (Stata `ranksum, porder`) | Given as `effect_size()`, which is `NA` today | B |
| `wilcoxon_test` | The help page says Stata always approximates; Stata 16 and later are believed to give exact p-values (not confirmed) | The sentence checked and corrected | C |
| `kruskal_test` | H both with and without the correction for ties (Stata) | None; a sentence in the help page | C |
| `chi2_test` | The likelihood-ratio chi-squared (Stata `lrchi2`) | `method: pearson \| lr = pearson` | A |
| `chi2_test` | Cramér's V; gamma and Kendall's tau-b for ordered tables (Stata `V`, `gamma`, `taub`) | V as `effect_size()` and in the print; `measure: v \| gamma \| tau_b` | A for V, B |
| `chi2_test` | Expected counts and residuals of each cell (R `$expected`, `$residuals`, `$stdres`; Stata `expected`, `cchi2`) | `expected()` and `residuals(type: pearson \| standardized)` on the result | A |
| `chi2_test` | A p-value by simulation when the counts are small (R `simulate.p.value`) | `simulate: Int`, with `seed:` | B |
| `fisher_test` | The confidence level and a null odds ratio other than 1 (R `conf.level`, `or`) | `level:` (fixed at 95% today), `or: Float = 1` | A, `or:` B |
| `fisher_test` | The sample odds ratio with Woolf's or Cornfield's interval (Stata `cc`); a simulated p-value for large tables (R) | `ratio: conditional \| sample`; `simulate: Int` | B |
| `proportion_test` | The exact binomial test (R `binom.test`; Stata `bitest`); today `ci: exact` changes the interval and leaves the p-value the score test's | `exact: Bool = false`, the p-value and interval both exact | A |
| `proportion_test` | The Agresti–Coull and Jeffreys intervals (Stata `ci proportions`) | `ci:` gains `agresti` and `jeffreys` | B |
| `proportion_test` | More than two groups (R `prop.test`) | None; the message Z0412 points to `chi2_test`, which gives the same statistic | C |
| `correlation` | The interval of Pearson's r and a one-sided alternative (R `cor.test`) | `level:` and `alternative:`, the interval printed with `p_values: true` | A |
| `correlation` | P-values adjusted over the table (Stata `pwcorr, bonferroni sidak`) | `adjust:`, with the words of `p_adjust` | B |
| `correlation` | The exact p-value of the rank methods forced or refused, and their continuity correction (R `cor.test(exact:, continuity:)`; today R's own rule chooses); Kendall's tau-a (Stata `ktau`) | `exact: Bool`, `correct: Bool`; tau-a printed beside tau-b | B |
| `correlation`, `covariance` | Weights (Stata `correlate [aweight \| fweight]`; R `cov.wt`) | `weights:`, by the ruling of #53 | B |
| `covariance` | A rank-based covariance (R `cov(method:)`); the divisor n (R `cov.wt(method: "ML")`) | None | C |
| `shapiro_test` | The Shapiro–Francia test (Stata `sfrancia`; R `nortest`) | `method: wilk \| francia = wilk` | B |
| `ks_test` | The two-sample test (R `ks.test(x, y)`; Stata `ksmirnov, by()`) | `ks_test(var, group:)` | A |
| `ks_test` | One-sided alternatives (R `alternative`; Stata prints D⁺ and D⁻); the exact p-value forced or refused (R `exact`) | `alternative:`; `exact: Bool` | B |
| `variance_test` (#97, 0.8.0) | The Fligner–Killeen test (R `fligner.test`); the trimmed mean as the centre (Stata `robvar`, W₁₀) | `method:` gains `fligner`; `center: median \| mean \| trimmed` | B |
| `tabulate` | Missing values as a category (R `useNA`; Stata `missing`) | `missing: Bool = false` | A |
| `tabulate` | Row, column and cell percentages together (Stata `row column cell`); weights (Stata); three variables or more (R `table(a, b, c)`) | `percent:` takes a list; `weights:`; a third variable as layers | B |

**3. Functions of earlier releases: data, text and time.** Proposed for 0.8.0, where #97 already reopens `bin` and `fill_missing`.

| Function | What R and Stata give that Zeff lacks | Proposed on the one function | Class |
| --- | --- | --- | --- |
| `summary` | The median and quartiles, which R's own `summary` prints, and the figures of Stata's `summarize, detail` and `tabstat`; today `stats:` takes six words | `stats:` accepts the name of any reducer: `median`, `p25`, `p75`, `variance`, `skewness`, `kurtosis`, `sum`, `iqr`, and `se`, `cv` | A |
| `sort` | Rising on one key and falling on another (Stata `gsort +a -b`; R `order(a, -b)`) | `desc:` takes a list of the keys to reverse | A |
| `winsorize` | Different lower and upper cut-offs (`DescTools::Winsorize`, the named equivalent, takes any pair) | `at: Float \| [low, high]` | A |
| `to_long`, `to_wide` | Several stubs or value columns in one call (Stata `reshape long inc ue`; `tidyr`) | `stub:` and `values_from:` take a list | A |
| `merge` | Which frame each row came from (Stata `_merge`) | `indicator: Str` names a column of `left`, `right`, `both` | A |
| `merge` | The right join (R `all.y`); every pairing of matching rows (R `merge`, Stata `joinby`) | `join: right`; `relation: many_to_many` | B |
| `mean` | The trimmed mean (R `trim`); the geometric and harmonic means (Stata `ameans`) | `trim: Float = 0`; `type: arithmetic \| geometric \| harmonic` | B |
| `rank` | Ties `last` and `random` (R); ranking from the largest (Stata `field`) | `ties:` gains the two words; `desc: Bool = false` | B |
| `bin` | Equal-width bins by their number (R `cut(x, 4)`), beside the quantile groups of #97; the lowest value kept in the first bin (R `include.lowest`) | With `n:`, `method: quantile \| width`; `include_lowest: Bool` | B |
| `sample` | Unequal probabilities (R `prob`); sampling within groups (Stata `sample, by()`) | `weights:`; `by:` | B |
| `diff` | Second and higher differences (R `differences`; Stata `D2.`) | `order: Int = 1` | B |
| `running_sum`, `running_product` | The running maximum and minimum (R `cummax`, `cummin`) | By #97 one `running(x, stat: sum \| product \| max \| min)`; a fifth merge to add to that entry | B |
| `row_mean`, `row_sum` | The minimum, maximum, standard deviation, median and count of missing across variables (Stata `egen rowmin` and its family; R `pmin`, `pmax`) | By #97 one `row(vars, stat:)`; a sixth merge | B |
| `standardize` | Centring without scaling, or scaling by a given mean and standard deviation (R `scale(center:, scale:)`; Stata `egen std(), mean() sd()`) | `center: Bool = true`, `scale: Bool = true` | B |
| `round` | Rounding to a multiple (Stata `round(x, 0.5)`) | `to: Float` | B |
| `seq` | A sequence by its length (R `length.out`) | `n: Int`, in place of `step:` | B |
| `replace` | Only the first match, or the first n (R `sub`; Stata `subinstr(s, a, b, n)`) | `count: Int` | B |
| `trim` | One side only (R `trimws(which:)`; Stata `strltrim`, `strrtrim`) | `side: both \| left \| right = both` | B |
| `weekday` | The day the week starts on (`lubridate` `week_start`; Stata `dow()` counts from Sunday = 0) | `start: monday \| sunday = monday` | B |
| `week` | Weeks counted from 1 January in blocks of seven days (Stata `week()`, `lubridate::week()`), or from the first Sunday or Monday (R `%U`, `%W`) | `type: iso \| jan1 \| sunday \| monday = iso` | B |
| `quantile`, `iqr` | Nothing: types 1 to 9 include Stata's default (type 2) and its `altdef` (type 6) | A sentence in the help page naming them | C |
| `mad`, `skewness`, `kurtosis`, `category` | The centre of `mad` (R); the third skewness of `e1071`; Stata's contrast operators, which belong to `contrasts` on the roadmap | None | C |

**4. Distributions.** Proposed for 0.8.0, where #93 falls due.

| Function | What R and Stata give that Zeff lacks | Proposed on the one function | Class |
| --- | --- | --- | --- |
| `dist.t`, `dist.chi2`, `dist.f`, `dist.beta` | Noncentrality (R `ncp`; Stata `nt`, `nchi2`, `nF`, `nbeta`); the intervals of d, g and eta squared and the `power.` family of 1.9 all need it | `ncp: Float = 0`, an [engine] piece; it settles #93 for the noncentral interval | A |
| `quantile` of a distribution | The quantile of the upper tail (R `lower.tail = FALSE`; Stata `invttail`, `invchi2tail`, `invFtail`) | `upper: Bool = false`, as `cdf` has | A |
| `pdf`, `cdf` | The logarithm of the density and of the probability (R `log`, `log.p`) | `log: Bool = false` | B |
| `dist.gamma`, `dist.exponential` | The scale in place of the rate (R `scale`; Stata's functions take the scale) | `scale:` as the alternative to `rate:` | B |
| new families | Cauchy (R, Stata), geometric (R), the signed-rank and rank-sum distributions (R), Laplace, inverse Gaussian and the bivariate normal (Stata) | Whole functions, not options; Cauchy and geometric proposed for 0.10.0 beside the three of #88, the rest declined | B, C |

**5. Regression and its results.** `ivreg` at 0.8.0; the rest proposed for 0.9.0, which brings robust and clustered errors to the variance layer for every model.

| Function | What R and Stata give that Zeff lacks | Proposed on the one function | Class |
| --- | --- | --- | --- |
| `predict` | The standard error and the confidence and prediction intervals (R `predict(interval:, se.fit:)`; Stata `stdp`, `stdf`) | `interval: none \| confidence \| prediction = none`, `level:`; from 0.9.0 `type: response \| link` | A |
| `residuals` | Standardised and studentised residuals, and Pearson, deviance and working residuals for the likelihood models (R `residuals(type:)`, `rstandard`, `rstudent`; Stata `predict`) | `type: response \| standardized \| studentized \| pearson \| deviance \| working = response` | A |
| `influence` | DFFITS, DFBETAS, the covariance ratio and the standardised residual (R `influence.measures`, the named equivalent; Stata `dfits`, `dfbeta`, `covratio`); Stata's Welsch distance | Four more columns; Welsch declined | A, C |
| `vif` | The generalised factor of a term with several columns (`car::vif`, the named equivalent) | Printed for categorical and interaction terms | A |
| `reg` | Clustering on two or more variables (`fixest`, `sandwich::vcovCL`; Stata's `reghdfe`, user-written, and believed official from Stata 18, not confirmed) | `cluster:` takes a list | A |
| `reg` | `HC0`, and `HC4`, `HC4m`, `HC5` (`sandwich`, `car`); the small-sample cluster corrections CR2 and CR3 (`sandwich`) | `se:` gains `hc0`, `hc4`, `hc4m`, `hc5`; `cluster_type: cr1 \| cr2 \| cr3 = cr1` | `hc0` A, the rest B |
| `reg` | Newey–West and Driscoll–Kraay errors; bootstrap and jackknife errors (Stata `vce()`) | Already #53 (1.4) and the roadmap rows `bootstrap` and `jackknife`; no change | n/a |
| `report` | The t statistic, the p-value or the interval beneath the estimate (`esttab` `se \| t \| p \| ci`; `modelsummary` `statistic`); other star cut-offs; more rows of statistics | `below: se \| t \| p \| ci = se`; `stars:` takes a list of cut-offs; `stats:` gains `adj_r_squared`, `f`, `rmse`, `log_likelihood`, `n_clusters` | A, the stars B |
| `compare` | The likelihood-ratio test (Stata `lrtest`, the named equivalent; R `anova(test: "LRT")`); three models or more (R) | `method: f \| lr`, at 0.9.0 with the likelihood models; a list of fits | A, B |
| `test` | The chi-squared form of the Wald test (`car::linearHypothesis(test: "Chisq")`); a joint test of nonlinear restrictions (Stata `testnl`) | `statistic: f \| chi2`; nonlinear conditions accepted | B |
| `ci` | Profile-likelihood intervals for the likelihood models (R `confint`) beside Wald's (Stata) | `method: wald \| profile`, at 0.9.0 | A |
| `fit_measures` | Stata's count of parameters leaves out the error variance, so its AIC is R's less 2 and its BIC R's less ln N; AICc | None; a sentence in the help page | C |
| `ivreg` (0.8.0) | Limited-information maximum likelihood and GMM (Stata `ivregress liml \| gmm`) | `method: tsls \| liml \| gmm = tsls` | A |
| `first_stage` (0.8.0) | The list to hold against Stata's `estat firststage`, `estat overid` and `estat endogenous`: partial and Shea's R², Cragg–Donald with the Stock–Yogo values, Sargan, Basmann and Hansen's J, Durbin and Wu–Hausman | Confirmed item by item when the release is planned | A |
| `ar_test` (0.8.0) | Moreira's conditional likelihood-ratio test and the K test (`ivmodel`; Stata's `weakiv`, user-written) | `method: ar \| clr \| k = ar` | B |
| `logistic`, `probit` (0.9.0) | The complementary log-log link (R, Stata `cloglog`); the cauchit and log links (R) | By #97 one function: `logistic(link: logit \| probit \| cloglog \| cauchit \| log = logit)`, `probit` becoming a value of `link:`; a seventh merge | A, B |
| `logistic` | Odds ratios in place of coefficients (Stata `logistic`, `logit, or`); successes out of a number of trials (R `cbind(s, f) ~ x`; Stata `glm, family(binomial n)`) | `summary(odds_ratios: true)`; `trials: Var` | A, B |
| `margins` (0.9.0) | Elasticities (Stata `eyex`, `dyex`, `eydx`; `marginaleffects` `slope:`); predictive margins of a factor; margins within groups (Stata `over()`; `marginaleffects` `by:`) | `slope: dydx \| eyex \| eydx \| dyex = dydx`; `margins(vars)` of a factor gives its predictive margins; `by: Vars` | A |
| `roc` (0.9.0) | The interval of the area (DeLong, bootstrap in `pROC`; Bamber, Hanley in Stata's `roctab`); the comparison of two curves (`pROC::roc.test`; Stata `roccomp`) | `ci: delong \| bootstrap \| hanley`; `roc.compare(other)` | B |
| `confusion` (0.9.0) | The measures of Stata's `estat classification` and of `caret` | The printed list confirmed when the release is planned | B |

**6. Release 0.10.0.**

| Function | What R and Stata give that Zeff lacks | Proposed on the one function | Class |
| --- | --- | --- | --- |
| `mcnemar_test` | The exact p-value and the odds ratio with its interval (Stata `mcc`) | `exact: Bool = false`; the ratio printed | A |
| `reliability` | The standardised alpha (`psych`, Stata `std`); reverse-scored items (`psych` `check.keys`; Stata `reverse()`); the item statistics and alpha with each item dropped (Stata `item`) | `standardized: Bool`; `reverse: Vars`; the item table printed | A |
| `reliability` | The interval of alpha (`psych`); listwise in place of pairwise deletion (Stata `casewise`) | `level:`; `missing: pairwise \| listwise` | B |
| `adjustment_sets`, `implied_independencies` | Canonical sets and the direct effect (`dagitty` `type: "canonical"`, `effect: "direct"`); the kinds of independence listed (`dagitty` `type:`). Stata has no command for graphs | `type:` gains `canonical`; `effect: total \| direct = total`; `type: missing_edge \| basis_set \| all_pairs` | B |

**The count.** Of the 278 functions, about 70 have a gap of class A or B and about 200 have none: the arithmetic, text, date and list functions, the layers of the plot grammar and the result interface do one thing that R and Stata also do in one way. Class A comes to about 40 options, class B to about 55, class C to about a dozen sentences in help pages. As a rough estimate, to be made exact when each release is planned: class A adds two or three steps to each of 0.7.0, 0.8.0, 0.9.0 and 0.10.0 beyond what their functions need anyway; class B adds about as much again, and its three hard pieces are Dunnett's distribution, the repeated-measures form of `anova` and the small-sample cluster corrections.

**What the owner is asked.**

1. *The reading of the rule.* Whether "alternatives" is as read above, or wider (every procedure R or Stata has, whether or not Zeff has the function) or narrower (methods only, leaving out forms and displays).
2. *Release 0.7.0, before steps 17 to 19 are built.* Whether the class A items of section 1 enter the open steps: for `posthoc`, the seven further words of `method:` and with them the four of `p_adjust`; for `anova`, `effect: eta | omega | epsilon` and numeric covariates; for the plot steps, the bin rule and scale of `histogram` and the kernel of `density`.
3. *The rest.* Which of the options below.
4. *The three merges this review adds to #97* (`running`, `row`, and `probit` as a value of `link:`): whether they join the four due at 0.8.0, the third at 0.9.0.

| Option | For | Against |
| --- | --- | --- |
| A. Class A and class B both before 1.0, each in the release its section names; class C answered in the help pages | The rule holds at 1.0, when the names and options freeze; each option is built while its function is open | About twice the added work of B; three hard pieces before 1.0 |
| B. Class A before 1.0 as sectioned; class B at 1.1, except where a release is building the function anyway (`ivreg`, `logistic`, `margins`, `roc`, `reliability`, the plots); class C in the help pages | The routine methods are all there at 1.0; an option added at 1.1 breaks no script, since defaults do not change | The rule holds at 1.0 for routine work only |
| C. Everything, class C included, before 1.0 | The rule read to the letter | Options that change nothing an analyst reports (Welch's 1947 degrees of freedom, the Welsch distance); the longest road to 1.0 |

*Recommendation.* The rule as read; the class A items of section 1 into steps 17, 19 and 20 to 30 of 0.7.0 now, each step's checklist line amended as #98 amended step 17; then B. Every option that enters does so by an amendment of its catalogue row at the step that builds it, logged in `docs/spec_changes.md`, with its reference test against the R function or the Stata output named here; no default changes; each "Start release" plan from 0.8.0 on lists the items of this entry that fall to it, and the review is repeated for the functions of each later release when that release is planned. Deadline: 0.7.0 for questions 1 and 2, since steps 17 and 19 wait on them; 0.8.0 for questions 3 and 4.

**The owner's decision (10 November 2026), question 2.** "Yes, add them." The class A items of section 1 enter release 0.7.0 at the steps not yet built, and the checklist `docs/releases/0.7.0.md` was amended the same day:

| Step | What enters | Reference |
| --- | --- | --- |
| 17 | A numeric covariate among the terms of the factorial `anova` (the analysis of covariance) | `car::Anova` and `anova(lm(...))` on a model with a covariate |
| 17b (new) | `anova(effect: eta \| omega \| epsilon = eta)` on both forms, partial in the factorial form | The formulas on base R's `aov` table; epsilon squared of the one-way form against the adjusted R² of `lm` |
| 18b (new) | `p_adjust(method:)` gains `hochberg`, `hommel`, `sidak`, `none` | `p.adjust`; Šidák's 1 − (1 − p)^m by hand |
| 19 | `posthoc(method:)` gains `sidak`, `scheffe`, `hochberg`, `hommel`, `bh`, `by`, `none` beside `tukey`, `bonferroni`, `holm` | `pairwise.t.test` with each method, pooled and not; `DescTools::ScheffeTest`; `emmeans` with `adjust = "sidak"` |
| 28 | `p.histogram`: `bins: Int \| sturges \| scott \| fd = fd`, `scale: count \| density \| fraction \| percent = count`, `start: Float`. `p.density`: `kernel: gaussian \| epanechnikov \| rectangular \| triangular \| biweight \| cosine = gaussian` | `nclass.Sturges`, `nclass.scott`, `nclass.FD`, `hist` on the same breaks, `density` with each kernel |
| 29 | The shorthands `histogram` and `density` carry the same options | The scene of the grammar form, byte for byte |

The definitions, fixed here so that the steps build one thing. *Šidák:* the adjusted p-value is 1 − (1 − p)^m over the m comparisons, the single-step form Stata prints. *Scheffé:* a pair's t² divided by k − 1 is compared with F on k − 1 and the error degrees of freedom, on the pooled variance, so that after Welch's `anova` it carries the note `tukey` carries by question 4 (f) of the release. `hochberg`, `hommel`, `bh`, `by`, `sidak` and `none` adjust the pairwise tests exactly as `bonferroni` and `holm` do under that question: Welch tests after Welch's `anova`, pooled tests after `equal_var: true`; `none` leaves them as they are (Stata's `noadjust`). *Omega squared,* one-way: (SS between − (k − 1) MS within) / (SS total + MS within); partial, for an effect of the factorial table: (SS effect − df effect × MS error) / (SS effect + (N − df effect) × MS error). *Epsilon squared,* one-way: (SS between − (k − 1) MS within) / SS total, which is the adjusted R² of the one-way model; partial: (SS effect − df effect × MS error) / (SS effect + SS error). After Welch's `anova` the two are computed from the unweighted decomposition, as eta squared is today. *The kernels* are each scaled so that `bandwidth:` is the kernel's standard deviation, as R's `density` does, so one bandwidth smooths alike under every kernel; `cosine` is R's `cosine`, not its `optcosine`. *The bin rules* give a number of bins as R's `nclass.Sturges`, `nclass.scott` and `nclass.FD` do; the default stays Freedman–Diaconis, as step 28 already had it.

Each catalogue row (171, 178, 195, 229, 234, 472, 473) is amended at the step that builds its option and logged in `docs/spec_changes.md`; no default changes. Left out of 0.7.0 although of class A: the interval of the `anova` effect size, which waits for `ncp:` on `dist.f` (#93, 0.8.0). The class B and C items of section 1 are not added by this ruling and fall under question 3.

One point the ruling raises and does not settle: omega squared and epsilon squared are negative whenever F is below 1 (on the small example of the `anova` help page, whose classical F is 0.56, omega squared is −0.067 and epsilon squared −0.073). Whether Zeff prints the negative figure or 0 is the owner's to say before step 17b; what R's `effectsize` and Stata's `estat esize` print in that case could not be confirmed here, since neither is installed. Questions 1, 3 and 4 remain open, question 1 with its deadline of 0.7.0.

**The owner's decision (10 November 2026), the negative estimates.** "Print as it is." A negative omega squared or epsilon squared is printed as computed, never set to 0, and `effect_size()` returns the same figure; a note beneath the table says that F is below 1 and the estimate is therefore negative, to be read as no evidence of an effect. The reason: the two estimators correct the upward bias of eta squared, and setting their negative values to 0 brings a part of that bias back. The help page says so, and says that other programs may print 0 in this case. Built at step 17b.

**The owner's decision (10 November 2026), question 1.** "Yes, as read." The rule stands as this entry read it: an alternative is another way of doing the same procedure that base R, the R package named in the function's `equivalent` column, or official Stata gives (another statistic, another way to the p-value or the interval, another estimator of a variance, another definition, or another form of the same test). Not counted: a whole procedure with its own catalogue row on the roadmap, arguments that only arrange R's or Stata's own output, and user-written Stata commands, except where the catalogue names one as the equivalent. The three classes and the gap tables above are therefore the list to be scheduled; questions 3 and 4, when the remaining gaps are built and the three further merges, remain open with their deadline of 0.8.0.

**The owner's decision (11 November 2026), question 3.** "Wait until after 1.0." None of options A, B and C: no gap found by this review enters a release before 1.0 beyond the class A items of section 1 that question 2 put into release 0.7.0, which stand. The class A and B items of sections 2 to 6 and the class B items of section 1 are built after 1.0, each in the release after 1.0 that builds or reopens its function. Their places are set when the plan of releases after 1.0 is recorded (the owner's table of models by level and release, 11 November 2026). The class C items are sentences in the help pages, written when each page is revised under #103. Nothing the specification already assigns to a release before 1.0 is withdrawn: the 0.8.0 row's Kleibergen–Paap, Cragg–Donald and effective F, for instance, stay in 0.8.0, and the list for `first_stage` is the one that row names. For 0.9.0 this means `glm` (#112) with the logit and probit links only, `margins` with `type: ame | mem` only, and `curve(type: roc)` without an interval or a comparison test. No default changes; every later option is an added word or argument, so no script written at 1.0 breaks. The owner's instruction of the same day, that nothing be added before 1.0 unless necessary, is the reason.

Question 4: the merge of `probit` into `link:` is answered by #112. The two other merges it names, `running(x, stat:)` for `running_sum` and `running_product` and `row(vars, stat:)` for `row_mean` and `row_sum`, remain open with their deadline of 0.8.0; they are renames, not additions, and names freeze at 1.0.

**The owner's decision (11 November 2026), question 4.** "Yes, merge both." The fifth and sixth merges join the four of #97 at release 0.8.0, built the same way: a `[function]` step each, with the merged help page, the rewrite in the release notes and the removal of the old catalogue rows. `running_sum(x)` and `running_product(x)` (catalogue rows 128 and 129, shipped at 0.6.0) become `running(x, stat: sum | product)`; `row_mean(vars, min_valid:)` and `row_sum(vars)` (rows 136 and 137, shipped at 0.5.0) become `row(vars, stat: mean | sum, min_valid:)`. Under question 3, the merges carry only the statistics that ship today: the running maximum and minimum and the row minimum, maximum, standard deviation, median and count of missing are added words after 1.0. The old names are refused with a diagnostic that names the new call. All four questions of this entry are now settled.

### 100. One term of the factorial `anova` table read as a value

Raised on 10 November 2026 at step 17 of release 0.7.0, the factorial form of `anova`. Its result is the `TestResult` of the one-way form, as the step requires ("both forms share one result type"), with one F, one pair of degrees of freedom, one p-value and one partial eta squared for each term of the table. The methods of open decision #94 return one figure: on the factorial result `statistic()`, `df()` and `p_values()` give the whole model's F, its degrees of freedom and its p-value, the first row of the table, and `effect_size()` is `NA`, since no single effect size describes a table of several terms. The figures of one term are reached through `estimates()`, a frame with a row for each term (its F, its two degrees of freedom and its p-value) and a row for each partial eta squared. A script that wants one term's p-value therefore writes `t.estimates().where(term == "dose")` and takes a column from it. Naming the term on the method, as a fit's methods allow (`m.p_values(educ)`), is refused today: `p_values("dose")` with Z0414, whose message points to `estimates()`, and `statistic("dose")`, `df("dose")` and `effect_size("dose")` with Z0407, since those three methods take no argument. The question is whether the methods should take a term.

| Option | For | Against |
| --- | --- | --- |
| A. `p_values("dose")`, `statistic("dose")`, `df("dose")` and `effect_size("dose")` read that term's row; a bare name works as it does on a fit (`p_values(dose)`); an interaction is named as the table prints it (`"dose:diet"`); without a term the methods stay the whole model's, and a name that is no term of the table is Z0414 with the list of terms | The way a fit is already read; one line where the frame route needs three; no new function | `df("dose")` returns two numbers, as `df()` does; `statistic()` without a term keeps meaning the model's F, which a reader of the table may not expect |
| B. As A, and `p_values()` without a term returns every term's p-value under its name, as it does on a fit | The same reading as a fit in every case | Changes what step 17 ships for the bare call; the whole model's p-value then needs a name of its own |
| C. Nothing: `estimates()` is the way to a term | No change | A common need takes three lines; differs from a fit for no reason a user can see |

*Recommendation.* A, as a `[function]` step of 0.7.0 after step 17b, which adds `effect:` to the same result, so that the three effect sizes are read the same way. Deadline: 0.7.0.

*Note of 11 November 2026 (step 19).* The result of `posthoc` is a `TestResult` of several rows as well, one for each pair of groups. Step 19 follows what step 17 did: `statistic()`, `df()` and `p_values()` on it give the F of the `anova` it follows, with its degrees of freedom and its p-value, and the figures of one comparison are read from `estimates()`, which gives its difference, standard error, t, p-value and limits. Whatever is ruled here for a term of the factorial table should hold for a comparison too, named as the table prints it (`p_values("a − b")`); under option A the methods without a name would stay the F of the `anova`.

**The decision (13 November 2026).** The owner, shown the four decisions due at 0.7.0, answered "A, but B for #102". This one is A. `p_values("dose")`, `statistic("dose")`, `df("dose")` and `effect_size("dose")` read the row of the term named, a bare name working as on a fit and an interaction named as the table prints it; without a term the four stay the whole model's; a name that is no term of the table is Z0414 with the list of terms. By the note of 11 November the same holds for a comparison of `posthoc`, named as its table prints it. Built as step 38d of the checklist of 0.7.0; the usages of catalogue rows 539, 540 and 541 gain the term, logged in `docs/spec_changes.md` at that step.

**Built (14 November 2026, step 38d of release 0.7.0).** As ruled. Two things were settled in the building, both recorded in `docs/decisions.md` and the owner's to overturn: the minus sign of a comparison may be typed as the hyphen of a keyboard, with or without the spaces round it (`"low - mid"` for `low − mid`), since the sign the table prints is on no keyboard; and the row `model` of the factorial table is not a name, the whole model being what the four methods give without one. The other methods of a result (`std_errors`, `ci`, `coef`) still take no row of a test's result, and say which four do.

### 101. Partial omega squared as Stata's `estat esize` computes it

Raised on 10 November 2026 at step 17b of release 0.7.0, which adds `effect: eta | omega | epsilon` to `anova` under the ruling on question 2 of #99. The ruling fixed the formulas, and step 17b ships them as ruled: the partial omega squared of a term is (SS effect − df effect × MS error) / (SS effect + (N − df effect) × MS error), where N is the number of observations. This is the textbook definition (Olejnik and Algina 2003) and the figure of R's `effectsize::omega_squared()`. Reading the sources while writing the help page showed that Stata's `estat esize, omega`, after `anova` or `regress`, computes another figure, from the F of the term alone: (F − 1) / (F + (df error + 1) / df effect), which is (SS effect − df effect × MS error) / (SS effect + (df error + 1) × MS error). R gives the same figure in `effectsize::F_to_omega2()`, whose documentation calls it "only an approximation for complex designs". The two definitions agree in the one-way form, in a model of one term and on the row of the whole model, since there N − df effect = df error + 1. For a term of a model with several terms df error + 1 is the smaller number, so Stata's figure is the larger in size. On the example of Stata's own manual (`regress bwt smoke i.race`, 189 observations) Stata prints 0.0716 for `smoke` and 0.0806 for `race`, and the ruled formula gives 0.0709 and 0.0802. Eta squared and epsilon squared do not differ between the programs. The sentence of #98, made a rule in #99, asks for every alternative that R or Stata gives to be an option on the one function, and this is a second definition of the same measure that both programs give. The help page of `anova` states the difference from step 17b on; the question is whether Zeff should also compute Stata's figure.

| Option | For | Against |
| --- | --- | --- |
| A. One more word on `effect:` for the figure computed from F, proposed `effect: omega_f`; the word is the owner's to choose | The rule of #98 and #99 as written; a Stata user reproduces `estat esize, omega` for every term; one option, no new function (#97); the kernel already holds every quantity it needs | A fourth word whose figure differs from `omega` in the third decimal place and only for a term of a model with several terms; the name needs explaining on the help page |
| B. Nothing more: the ruled formula alone, with the difference stated on the help page as step 17b ships it | One definition, the textbook one; nothing to build | A Stata user cannot reproduce `estat esize, omega` for a term of a model with several terms; reads the rule more narrowly than the owner stated it |
| C. Replace the ruled formula by Stata's | Agreement with Stata everywhere | Departs from the ruling, from the textbook definition and from R's `omega_squared()`; R's own documentation calls the figure an approximation |

*Recommendation.* A, as a small `[function]` step of 0.7.0 after step 17b, because the rule asks for it and the cost is a few lines; the default stays `eta` and `omega` stays the ruled formula. Deadline: 0.7.0.

**The decision (13 November 2026).** The owner, shown the four decisions due at 0.7.0, answered "A, but B for #102". This one is A, with the word proposed: `effect: omega_f` gives the partial omega squared computed from F, as Stata's `estat esize, omega` and R's `F_to_omega2` give it; `omega` stays the ruled formula and the default stays `eta`. Built as step 38e of the checklist of 0.7.0; the usage of catalogue row 171 gains the word, logged in `docs/spec_changes.md` at that step.

**Built (14 November 2026, step 38e of release 0.7.0).** As ruled: `effect: omega_f`, printed as "partial omega squared from F" (in the one-way form "omega squared from F", where it is omega squared itself). A correction to the illustration above, found while building: the figures quoted from Stata's manual could not be reproduced. On R's copy of the same data (`MASS::birthwt`, 189 observations, `bwt ~ smoke + factor(race)`, type 2) the formula from F gives 0.0718 for `smoke` and 0.0802 for `race`, the ruled formula 0.0711 and 0.0798, and 0.0806 is the partial epsilon squared of `race`. The two formulas, which are what was ruled, are not in doubt, and both are held to R; but no run of Stata stands behind `omega_f`, since the reference library has no Stata output. A run of `regress bwt smoke i.race` and `estat esize, omega` in Stata would settle the figures; it is the owner's to make or to ask for.

### 102. The largest of several studentized ranges: R's `nranges` on `dist.tukey`

Raised on 11 November 2026 at step 18 of release 0.7.0, which builds `dist.tukey(groups: Int, df: Float)` as the catalogue's row 494 gives it. R's `ptukey(q, nmeans, df, nranges = 1)` and `qtukey` take one more argument: with `nranges` of c they give the distribution of the largest of c studentized ranges, each of `nmeans` means, the c ranges independent of one another and all divided by the same estimate of the standard deviation. In the notation of the help page, P(Q ≤ q) = ∫ [P(W ≤ qs)]^c f(s) ds. This is the distribution of Copenhaver and Holland (1988), for comparing the means within each of c rows of a two-way layout at one overall level (tests of simple effects). Stata's `tukeyprob(k, df, x)` and `invtukeyprob()` have no such argument, and nothing in the catalogue needs it: `posthoc` at step 19 uses one range. The rule of open decisions #98 and #99 (every alternative that R or Stata gives is an option) reaches it nevertheless, and #99 places a less common gap at 1.1 "unless their function is being built anyway", which is the case here. The catalogued usage has no such parameter, and the freeze leaves a new one to the owner. Step 18 ships the catalogued usage; its help page says that R's `nranges` is not offered and names this decision.

| Option | For | Against |
| --- | --- | --- |
| A. One more parameter on `dist.tukey`, proposed `ranges: Int = 1`, a whole number of 1 or more; the name is the owner's to choose | The rule of #98 and #99 as written; one option, no new function (#97); the kernel's outer integral gains a power, and its upper tail and its density the matching terms, while the method is fresh; the exact reference extends by a few lines | A parameter few users will meet; more cases in the reference, whose script already takes about three minutes of processor time in CI; one more `[engine]` step in a long release |
| B. Nothing in 1.0: the help page says it is not offered, and the question returns at 1.1 with the other less common gaps of #99 | Nothing to build now; the catalogued usage stands | Reads #99 more narrowly than it is written, since the function is being built now; an R script that uses `nranges` cannot be reproduced |
| C. Nothing at all | The smallest surface | Against the rule |

*Recommendation.* A, as a small `[engine]` step of 0.7.0 after step 19, because the rule asks for it and the kernel and its reference are fresh; the default of 1 leaves every value of step 18 unchanged. B is the choice if 0.7.0 should not grow further. Deadline: 0.7.0.

**The decision (13 November 2026).** The owner, shown the four decisions due at 0.7.0, answered "A, but B for #102". This one is B: `dist.tukey` keeps its catalogued usage in 1.0, its help page saying that R's `nranges` is not offered, and the question returns at 1.1 with the less common gaps of #99, so that 0.7.0 does not grow further.

### 103. The quality of the help pages, and a reference website

Raised on 11 November 2026 by the owner, after comparing the help page of `add` with the reference page of `dplyr::mutate()` and reading a sample page written for `add` in the same session. The owner's ruling in principle: before 1.0 every catalogued function has a help page of the quality of that sample, and later the project has a website carrying every function's help page in the manner of the tidyverse reference sites. The ruling stands. What is open is how the help contract of section 9.1 of the Engine Specification changes, when the existing pages are brought up to it, and when the website appears. Section 9.1 is frozen (#40), so its text changes only when this entry is decided.

Where the pages stand at commit 170ffa0: 209 pages and 9,619 lines; the median page has 41 lines, the shortest 22 (`h0`) and the longest 330 (`anova`); 94 pages carry three example statements or fewer and 12 carry seven or more; no example has a caption saying what it shows; 176 pages mention `NA` or missing values somewhere but none under a heading of its own; 170 mention R or Stata, mostly inside Details. The catalogue schedules 278 functions up to 1.0, so about 70 pages are still to be written and 209 to be revised. The sample page was roughly twice as long as today's page for `add`.

What the sample added to the parts of section 9.1:

- **In brief**, before the description: the two or three commonest forms of the call, each with one line saying what it does.
- **Missing values** under a heading of its own: what the function does with `NA`, and what it prints when it drops or produces one.
- **A caption on every example**, saying what the example shows, and examples on the example files that ship with the engine rather than on a file of three rows.
- **A common mistake**: the wrong call a newcomer is most likely to write, with the diagnostic the engine really prints and the corrected call.
- **Related tasks**: "to do this other thing, use that function", one line each, beside the bare list of See also.
- **Coming from R or Stata**: the equivalent calls, and the differences that would surprise a user of either.
- Details divided under short headings where the function has several distinct matters to explain (for `add`: what a definition may contain).

Questions for the owner:

1. Which of these parts are required, and of which functions. Proposed: a caption on every example and Missing values on every function that reads data; In brief on every function with more than one form of call; A common mistake wherever the function has a diagnostic of its own; Coming from R or Stata wherever the catalogue's `equivalent` column is filled, the equivalents generated from the catalogue and the differences written by hand; Related tasks and the headings inside Details where they help, not required.
2. The least an example section must show. Proposed: every argument appears in at least one example, and every example runs on the shipped example files.
3. The timetable, in the table below.
4. The website: its release, its address and its name. Proposed: generated from the same help files and the same tested examples as the engine's own help, so that the two cannot disagree; one page per function, an index by package and by task, and a search box; published with the documentation of 1.0.0.

| Option | For | Against |
| --- | --- | --- |
| A. Amend section 9.1 at the opening of 0.8.0. Every function written from 0.8.0 onwards meets the new contract in its own commit; the pages written before it are revised package by package as the closing steps of 0.8.0, 0.9.0 and 0.10.0, complete at 0.10.0, where the validation suite is published; the website is generated at 1.0.0 | No page is written twice after 0.8.0; the revision is spread over three releases that are small beside 0.7.0; the build check keeps every page to the contract from then on; the pages are complete before `core` freezes at 0.11.0 | The remaining functions of 0.7.0 are written to today's contract and revised later; three releases each carry extra steps |
| B. One pass over every page at 1.0.0, under "documentation" in the build order | Nothing changes until the functions are all in place; one consistent voice | About 280 pages in a single release that also carries packaging and the launch; every page written until then is written twice; faults such as the out-of-date sentence on `add` stay in the engine through four releases |
| C. Amend the contract now and write the rest of 0.7.0 to it | The 35 or so functions still to come in 0.7.0 are written once | Changes the definition of done in the middle of the largest release, with a concurrent step in hand; pulls a specification change forward of its decision |

*Recommendation.* A, with the proposals of questions 1, 2 and 4 as written. Engineering that follows, recorded in docs/decisions.md when the entry is decided: the build check of the help contract extended to the new headings; a way of writing a caption within the markup section 9.1 allows; the generator of the website. Found on the way and to be corrected as errors whatever is decided here: the page of `add` says that reducers inside a definition and `by:` "arrive later in this release", though both work; it has no example of either; and its last printed line, on an `add` of the `learn` package after 1.0, reads as though this function were unavailable. Deadline: 0.8.0 for questions 1 to 3; 1.0.0 for the address and name of the website.

**The owner's decision (11 November 2026).** "Yes, option A", with the proposals of questions 1, 2 and 4 as written. What follows:

- Section 9.1 of the Engine Specification is amended at the opening of release 0.8.0, as the first step of that release's checklist, and the change is logged in docs/spec_changes.md then. The new parts and who must carry them: a caption on every example, and Missing values on every function that reads data; In brief on every function with more than one form of call; A common mistake wherever the function has a diagnostic of its own; Coming from R or Stata wherever the catalogue's `equivalent` column is filled, the equivalents generated from the catalogue and the differences written by hand; Related tasks and headings inside Details where they help, not required. Every argument appears in at least one example, and every example runs on the example files that ship with the engine. The build check of the help contract enforces all of it from that step.
- Every function written from 0.8.0 onwards meets the amended contract in its own commit; the testing handbook's definition of done (section 4) already requires the complete help page there.
- The pages written before 0.8.0 are revised package by package as closing steps of the checklists of 0.8.0, 0.9.0 and 0.10.0, tagged `[function]`, and are complete at 0.10.0.
- The functions still to come in 0.7.0 are written to the contract as it stands today and revised with the others; nothing of this decision enters 0.7.0 except the correction of the errors on the page of `add` named above.
- The website is generated at 1.0.0 from the same help files and the same tested examples as the engine's own help, with one page per function, an index by package and by task, and a search box. Its name and its address remain the owner's to choose, by 1.0.0.

### 104. `posthoc` after a factorial `anova`: which means are compared, and of which term

Raised on 11 November 2026 at step 19 of release 0.7.0. The checklist's step 19 asked for `posthoc` "after either form of `anova`". After the one-way form the definition is not in doubt, and step 19 ships it. After the factorial form, `d.anova(y ~ a * b)`, R and Stata do not agree on what is compared, and no normative text says: the catalogue's row 195 reads "pairwise comparisons after an ANOVA", and question 4 (f) of the checklist speaks of the one-way form alone. Step 19 was therefore split. Until this entry is decided, `posthoc` after an `anova` given a formula is refused with Z0401, whose help names the one-way form. Three things are open.

*Which means.* (i) The estimated marginal means of the fitted model, also called least-squares means: for each level of the factor, the model's prediction averaged with equal weight over the levels of the other factors, a covariate held at its mean. R's `pairs(emmeans(fit, ~ a))` and Stata's `pwcompare a` after `anova` compare these; the standard error of a difference comes from the covariance matrix of the coefficients, on the residual degrees of freedom of the model. (ii) The means of R's `TukeyHSD(aov(y ~ a * b))`, taken from the sequential decomposition of `aov`: each term's means after the terms written before it have been removed, with a standard error from the residual mean square and the sizes of the groups. (iii) The raw cell means compared as one layout, as Stata's `pwmean y, over(a b)`. In a balanced design without a covariate (i) and (ii) agree. In an unbalanced design they do not, and (ii) depends on the order in which the terms are written. Checked in R 4.6.0 on 18 observations in a 3 × 2 layout with cells of 2 to 4: for one pair of levels `TukeyHSD` gives a difference of 1.214 (p = 0.044) from `y ~ a + b` and 1.349 (p = 0.025) from `y ~ b + a`, while `emmeans` gives 1.34 (p = 0.026) from either. R's own documentation claims sensible intervals from `TukeyHSD` only for mildly unbalanced designs.

*Which term.* A factorial table has several terms and a `TestResult` prints one table. `TukeyHSD` prints a table for every factor unless `which = ` names some; `emmeans` and `pwcompare` require the term to be named.

*An interaction.* With `a:b` named, `emmeans` and `pwcompare` compare every pair of cells. The comparisons of `a` within each level of `b` (simple effects, `emmeans(fit, ~ a | b)`) are a further form.

| Option | For | Against |
| --- | --- | --- |
| A. The estimated marginal means of one term, named by a new option, proposed `term:`: `posthoc(term: dose)` for a factor, `posthoc(term: "dose:diet")` for every pair of cells of an interaction; `term:` may be left out when the model has one factor and is required when it has several, the error listing them; every method of step 19, on the residual variance and degrees of freedom of the model, family-wise over the pairs of that term; the name of the option is the owner's | The definition on which R's `emmeans` and Stata's `pwcompare` agree; correct in an unbalanced design and with a covariate; independent of the order of the terms; `emmeans` is already in the reference library | Differs from R's `TukeyHSD` in an unbalanced design, which the help page must say; the kernel of step 17 must hand on its coefficients, their covariance matrix and the coding of the factors, so the step is `[engine]`; one more option on a frozen usage |
| B. The means of R's `TukeyHSD`, with `term:` as in A | The command the catalogue names as the equivalent; simple arithmetic on the table of means | Depends on the order of the terms in an unbalanced design; not what Stata gives; with a covariate the answer depends on where the covariate is written |
| C. No factorial form at 0.7.0: it arrives at 0.9.0 with `margins`, which needs the same machinery (predictions averaged over the levels of the other terms); the refusal of step 19 stands until then | The machinery is built once; 0.7.0, already the largest release, does not grow | The checklist promised both forms; a two-factor experiment has no comparisons before 0.9.0 except through a one-way `anova` on each factor, which ignores the other |

*Recommendation.* A, as step 19b of 0.7.0, because the definition is the one R's `emmeans` and Stata share and the checklist promised the form at this release; C is the choice if 0.7.0 should not grow by an `[engine]` step. Under A or B the simple effects join the less common gaps of #99 under its question 3. Deadline: 0.7.0.

**The owner's decision (11 November 2026).** "Yes, option A", after the samples of step 19 showed that R's `TukeyHSD` on the trial data of the step 17 samples gives a p-value of 0.072 for diet c against a under `gain ~ dose * diet` and 0.015 under `gain ~ diet * dose`, while `emmeans` gives 0.065 under either order. After a factorial `anova`, `posthoc` compares the estimated marginal means of one term, as R's `emmeans` and Stata's `pwcompare`; the term is named by the option `term:`, the name proposed here; every method of step 19 applies, on the residual variance and degrees of freedom of the model. Built as step 19b of 0.7.0. The simple effects (`emmeans(fit, ~ a | b)`) join the less common gaps of #99 under its question 3.

### 105. A confidence level on `posthoc`, and intervals for its step-wise methods

Raised on 11 November 2026 at step 19 of release 0.7.0. Two points on the intervals of `posthoc`, both met while it was built against the catalogued usage, which has `method:` and nothing else.

*The level.* R's `TukeyHSD(conf.level = 0.95)` and `DescTools::ScheffeTest(conf.level = )`, `emmeans` (`confint(level = )`) and Stata's `pwcompare, level(#)` and `pwmean, level(#)` give the intervals at any level. Every other test of `stats` that prints an interval takes `level: Float = 0.95`. Step 19 ships the intervals at 95%: `posthoc(level: 0.9)` is an unknown argument (Z0404), and `ci(level: 0.9)` on the result is refused with a message saying that `posthoc` gives its intervals at 95%. The rule of #98 and #99 reaches the gap, and the review of #99 did not list it.

*The step-wise methods.* Step 19 prints intervals with the single-step methods, `tukey`, `scheffe`, `bonferroni`, `sidak` and `none`, whose intervals match their p-values exactly. Holm's, Hochberg's, Hommel's and the two false-discovery methods adjust each p-value by its place among the others and have no simple intervals to match, so their table has none and `ci()` gives `NA`. R's `pairwise.t.test` prints no intervals under any method. R's `emmeans` prints Bonferroni's intervals when a step-wise adjustment is asked for, and says so beneath the table ("Conf-level adjustment: bonferroni method for 3 estimates"). Stata's `pwcompare` has no step-wise adjustment of p-values among its words except `snk` and `duncan`, which are not built (#99); this is from its manual as remembered, Stata not being installed.

| Option | For | Against |
| --- | --- | --- |
| A. `level: Float = 0.95` on `posthoc`, used by the five single-step methods; the step-wise methods keep a table without intervals, and with `level:` written out there a note says that the method has none | What R and Stata give; the same option as every other test; the quantiles needed are already computed for 95% | One more option on a frozen usage |
| B. As A, and the step-wise methods print Bonferroni's intervals with a line saying so, as `emmeans` | Every method prints intervals | An interval that can disagree with the p-value printed beside it: Holm may reject where Bonferroni's interval covers zero |
| C. Nothing: 95% only | No change | A gap under the rule of #98 and #99; an interval at 90% or 99% then needs the formulas by hand |

*Recommendation.* A, built with step 19b or as a small `[function]` step of 0.7.0 after it; B's intervals are better left out, since a reader takes an interval beside a p-value to agree with it. Deadline: 0.7.0.

Also found at step 19 and placed with the less common gaps of #99, under its question 3 (due at 0.8.0): R's `pairwise.t.test` takes `alternative = `, for one-sided comparisons, and `paired = `, for paired t-tests on repeated measures; `posthoc` gives two-sided comparisons of independent groups only. R's `TukeyHSD(ordered = )` only arranges the output and is not counted.

**The decision (13 November 2026).** The owner, shown the four decisions due at 0.7.0, answered "A, but B for #102". This one is A: `level: Float = 0.95` on `posthoc`, used by the five single-step methods; the step-wise methods keep a table without intervals, and with `level:` written out a note says that the method has none. Bonferroni's intervals beside step-wise p-values (B) are not given. Built as step 38f of the checklist of 0.7.0; the usage of catalogue row 195 gains the option, logged in `docs/spec_changes.md` at that step.

**Built (14 November 2026, step 38f of release 0.7.0).** As ruled: `level: Float = 0.95` on `posthoc`, in both forms; the heading of the table, the line under it and `ci()` on the result carry the level asked. The note for a step-wise method with `level:` written out has a code, W0401 ("an option written out that changes nothing in the call"), the first of the W04 family, so that a pragma may allow it and `--deny-warnings` refuse it, as with any coded note; its row is in section 11 of the Language Reference and its page in `zeff help W0401`. `ci(level:)` on the result at another level than the result's is still refused, since the intervals come from the method's own distribution; the message now sends the reader to `posthoc(level:)`.

### 106. A layer that overrides a mapping of its plot

Raised on 11 November 2026 at step 20 of release 0.7.0, when `points` was built. Section 7.6 of the Language Reference says, under "Mappings and constants", that "a layer may name its own frame (`.points(data: outliers)`) or override a mapping". The catalogue gives every mark `data:` and nothing for the second half of the sentence: no row from 461 to 475 has `x:`, `y:` or `color:` among its options (`tile` has `fill:`, `text` has `label:`, `ribbon` and `errorbars` have `low:` and `high:`, `hline` and `vline` have `at:`, each for the one thing the mark needs). So the two normative texts disagree, and the question is which is right.

What it is for. A plot of observed values with a fitted line needs two variables on one axis: `d.plot(x: educ, y: wage).points().lines(y: fitted)`. Without an override this is drawn only by giving the second layer a frame of its own in which the fitted values carry the name `wage`. In R with ggplot2 every layer takes its own `aes()`, which adds to or replaces the mappings of the plot. In Observable Plot each mark names its own channels and there is no mapping of the plot at all. Stata's `twoway` overlays plots that each name their own variables: `twoway (scatter wage educ) (line fitted educ)`.

| Option | For | Against |
| --- | --- | --- |
| A. Every mark takes `x:` and `y:`, and each of the six other aesthetics it can draw, for its own layer; an argument given replaces the mapping of the plot for that layer, one not given inherits it | What the sentence says and what ggplot2, Observable Plot and Stata all allow; observed and fitted on one plot without renaming; the axis keeps the title of the plot's mapping | Up to eight more options on each of fifteen catalogued usages, a change under the freeze; the title of an axis shared by two variables has to be ruled (proposed: that of the plot's mapping, changed by `labels`) |
| B. As A for the two positions only, `x:` and `y:`; colour and the rest are set per layer by constants where the mark's usage already has them | The common case, a second variable on the same axis, at a quarter of the cost | A layer cannot be coloured by a variable the plot does not map |
| C. The sentence is corrected: a layer may name its own frame, and the mappings are those of the plot | No change to the catalogue; the simplest rule to teach | The overlay of two variables needs a reshaped or renamed frame, where every reference tool does it directly |

*Recommendation.* A, with the mark's own arguments typed as on `plot`, decided before step 23 so that the seven marks of that step are built once; step 20 builds `points` with the catalogued usage, and `points(y: …)` is refused as an unknown argument (Z0404) until then. Deadline: 0.7.0, before step 23.

**The decision (12 November 2026).** Option A, taken on the recommendation under the owner's standing instruction of that day ("If at any point any decisions to be taken by me, I will go by your recommended decisions"), which he may overturn. Every mark takes `x:` and `y:`, and each other aesthetic it draws, for its own layer, typed as on `plot`: a bare name maps a variable of the layer's frame, an expression is computed on it, and for a colour, a size or a shape a string or a number sets a constant for that layer. An argument given replaces the mapping of the plot for that layer; one not given inherits it. The layers of a plot still share one scale of each aesthetic, so two variables shown by one aesthetic must be of one kind, categories or a number. The title of an axis or a legend shared by two variables is that of the plot's mapping, or of the first layer's where the plot maps none, until `labels` changes it (step 26). Built at step 23a on `points`, before the seven marks of step 23, with the usage lines of the catalogued marks amended as each is built and logged in `docs/spec_changes.md`.

### 107. What an expression mapped to an aesthetic may call

Raised on 11 November 2026 at step 20b of release 0.7.0, when expressions in `x:` and `y:` of `plot` were built. Section 7.6 of the Language Reference says that "`x` and `y` may be variables or expressions over the frame" and says no more about what such an expression may hold. Section 3.7 ("Columns and scalars") says that every built-in scalar function of `core`, `text` and `time` applies row by row when given a column. Step 20b builds the narrowest reading that draws the examples of the specification (`log(wage)`, `wage / hours`): variables, numbers, logical values, text, `NA`, the operators, script constants, and the functions of `core` that the engine computes row by row. Everything else is refused with Z0401 and the advice "compute anything else with `add` first and map the new variable". The question is how far the reading should be widened, and when.

What is refused today, by kind. (1) The reducers of `data`: `y: wage - mean(wage)`, `y: wage / sd(wage)`, `y: rank(wage)`. (2) The scalar functions of `time` and `text`: `x: year(hired)`, `y: contains(name, "Ltd")`. (3) Four functions of `core` that are written in Zeff and not in the engine: `logit`, `inv_logit`, `clamp`, `fill_missing`; a formula refuses the same four. (4) `case` and `if`, a function of the project, and a part taken from another value (`w["wage"]`).

What the reference tools do. In R with ggplot2, `aes()` takes any R expression and evaluates it in the layer's data: `aes(y = wage - mean(wage))` is computed with the mean of the rows that layer draws (checked with ggplot2 on R 4.6.0), and the axis is titled by the expression. Observable Plot takes a name or a function of the row for each channel, and offers reducers only through its transforms. Stata's `twoway` takes variables only: an expression needs `generate` first.

| Option | For | Against |
| --- | --- | --- |
| A. As built: the operators and the row-by-row functions of `core` in the engine; the rest through `add` | The commonest transformations (`log`, `sqrt`, a ratio, a power) are drawn directly; one rule, short to teach; nothing here has to be withdrawn later; wider than Stata | Narrower than ggplot2; `wage - mean(wage)` and `year(hired)` need a line of `add` |
| B. A, and everything section 3.7 lifts: the scalar functions of `time` and `text`, the four Zeff-written functions of `core`, and `case` | An expression in a mapping then holds what an argument of `add` holds, less the reducers; `year(hired)` on an axis is common | A result that is text or a date cannot be drawn until its scale exists (step 25 for dates; categories on a position with `bars` at step 24), so most of the gain arrives only then; `plot` comes to depend on `time` and `text` |
| C. B, and the reducers of `data`, computed on the frame the layer draws | What ggplot2 allows; centring and standardising without a new variable | A layer with its own frame gets its own mean, so two layers of one plot can be centred differently without the script saying so; the rows dropped for `NA` in the other aesthetic are still in the mean, which ggplot2 shares and few users expect |

*Recommendation.* A for the plots of 0.7.0, with B taken up when the date scale is built at step 25, since a date or text result has no scale to be drawn on before then; C left out of 1.0 unless the owner wants it, because of the mean that silently differs between layers. Every widening is an addition: a script accepted under A means the same under B and C. Deadline: 0.7.0, before step 25.

**The owner's decision (11 November 2026).** "Yes, as built", after samples of step 20b compared with ggplot2: option A. An expression mapped to an aesthetic holds variables, numbers, logical values, text, `NA`, the operators, script constants and the functions of `core` that the engine computes row by row; everything else is refused with the advice to compute the variable with `add` first, as step 20b builds it. The widening to B when the date scale is built at step 25, and C, were not ruled on: whether to widen is put to the owner again at step 25. Any widening is an addition, so a script accepted now keeps its meaning.

**The decision on the widening (12 November 2026).** Option B, as recommended, taken under the owner's standing instruction of that day, which he may overturn: at step 25c of release 0.7.0, where the date scale is built, an expression mapped to an aesthetic may also call the scalar functions of `time` and `text`, the four functions of `core` that are written in Zeff (`logit`, `inv_logit`, `clamp`, `fill_missing`) and `case`. Option C, the reducers of `data` inside a mapping, is not taken: a mean that silently differs between two layers of one plot is not offered in 1.0.

### 108. A numeric function of `core` given a text or category column

Raised on 11 November 2026 at step 20b of release 0.7.0, found while the edge cases of an expression in a mapping were read; the behaviour is that of `core` since the row-by-row functions were built, not of the step. A numeric function of `core` given a single text value is refused when the script is checked: `log("a")` gives Z0401, "`x` of `log` must be a Float, but this is a Str". Given a text or category column, the same function gives `NA` on every row and says nothing:

```
let d = frame(s = ["a", "b"], n = [1, 2])
print(d.add(a = log(s), b = abs(s), c = exp(s)))
```

prints three columns of `NA` with no warning and no note. The same holds for `sqrt` and `round`, and for a category (`log(industry)` on the `wages` data). In a mapping the consequence is a plot of no points, `w.plot(x: educ, y: log(industry)).points()`, with the note "18,412 rows with NA in educ or log(industry) dropped", which reads as a fact about the data and is a mistake of the script. The operators do refuse: `industry + 1` is Z0221, "`+` does not apply to a category".

Section 3.7 ("Columns and scalars") says that a scalar function "applies row by row when given a column and returns a column", and does not say what happens when the column's type is one the function does not take. Section 5.6 lists the counted warnings for a number outside a function's domain (W0204 for the logarithm of zero, W0205 for that of a negative number, and the others); none is for a value of the wrong type.

What the reference tools do. R refuses: `log(c("a", "b"))` is the error "non-numeric argument to mathematical function" and `log()` of a factor is the error "'log' not meaningful for factors" (R 4.6.0). Stata refuses: `generate a = log(industry)` on a string variable is "type mismatch", r(109), from its manual as remembered, Stata not being installed.

| Option | For | Against |
| --- | --- | --- |
| A. Refuse it when the statement runs, with Z0401 worded as for one value and naming the column and its type; the help points to `to(x, Float)` for text that holds numbers and to `to(x, Int)` for the positions of a category's levels | What R and Stata do; the same rule for one value and for a column; a mistake is found where it is made and not three statements later in a mean of `NA` | A script that today runs and gives `NA` stops with an error: a breaking change, to be listed in the release notes |
| B. Keep `NA` on every row and add a counted warning, in the manner of W0205 | Nothing stops; the mistake is at least announced | A warning is easily missed in a long run, and every later result is `NA`; a new warning code |
| C. As now | No change | A wrong type passes in silence, against the engine's own rule for one value |

*Recommendation.* A, at 0.8.0, in a step of its own with the snapshot of every numeric function of `core` on a text, a category, a date and a logical column, so that the rule is the same for all of them; the release notes of 0.8.0 list it as a breaking change with the rewrite (`to(x, Float)` or the removal of the call). Deadline: 0.8.0.

**Found at step 25c3a (12 November 2026): the functions of `time` and `text` fare worse.** Given a column of a type they do not take, they stop with an internal error and not with `NA`: `f.add(y = year(industry))` on the shipped firms prints Z9001, "internal error: this is a bug in Zeff, not in your script (a value that is not a date or a time)", and `f.add(y = upper(size))` the same with "an argument that is not text". Given one value the checker refuses both with Z0401, as it refuses `log("a")`. The behaviour is that of `add` since release 0.6.0 and not of the step; the step lets a mapping call these functions (open decision #107, option B), so `f.plot(x: year(industry), y: size)` now reaches the same internal error. It is the mistake of this entry seen from another side, and option A settles both: the type of the column is checked against the parameter when the statement runs, and the refusal is Z0401 in the checker's words. Nothing was changed at step 25c3a, since the rule is the owner's to choose and its step is planned for 0.8.0; until then the internal error stands in `add` and in a mapping alike, and the tests of the step do not pin it.

*Decided 15 November 2026, as recommended (option A).* Built at step 6 of `docs/releases/0.8.0.md`: a numeric function of `core`, a function of `time` or of `text` given a column of a type it does not take is refused when the statement runs, Z0401 worded as for one value, with `to(x, Float)` and `to(x, Int)` in the help; the release notes of 0.8.0 list it as a breaking change with the rewrite. The internal errors of `time` and `text` go with it.

### 109. The colours of a plot: more categories than colours, a level with no row, and the names of colours

Raised on 11 November 2026 at step 20c of release 0.7.0, where `color:` and `fill:` of `data.plot(…)` were built. Section 7.6 of the Language Reference fixes the defaults, "Okabe–Ito for categories, viridis for continuous scales, never a rainbow", and says that `scale(color: [...])` overrides them. Three things it does not say were met while coding. Each is built in the way recommended below, and each can be changed before step 25, where the legend is drawn and `scale` is built, without breaking a released script.

**Question 1. More categories than the palette has colours.** The palette of Okabe and Ito has eight colours and `industry` of the `wages` data has twelve categories.

| Option | For | Against |
| --- | --- | --- |
| A. Repeat the colours from the ninth category and say so in a note, as built: "`industry` has 12 categories and the palette 8 colours, so the colours repeat from the ninth category" | The plot is drawn, so a first look at the data never stops; the note says exactly what is wrong; D3 and Observable Plot repeat too, and R's `palette.colors(recycle = TRUE)` | Two categories share a colour, and a reader who misses the note may confuse them |
| B. Refuse the plot with an error that advises `scale(color: [...])` or fewer categories | No plot can mislead; ggplot2's manual scale does this ("Insufficient values in manual scale") | A scatter by industry stops for a reason that is about appearance, not about the data; `scale` does not exist until step 25 |
| C. Above eight categories, take the colours at equal steps along viridis | Every category has its own colour, however many | Neighbouring colours of a ramp are hard to tell apart beyond about ten, and the ramp suggests an order the categories may not have; the colour of a category changes when a ninth appears |

*Recommendation.* A. With twelve colours no palette lets a reader tell every pair apart, so the honest course is to draw and to say so; `scale(color: [...])` at step 25 gives the remedy.

**Question 2. A level with no row among the marks drawn.** `w.where(wage > 100)` keeps 8 of the 12 industries. Its plot coloured by `industry` could show 8 categories or 12.

| Option | For | Against |
| --- | --- | --- |
| A. Leave the level out, as built: the scale shows the categories of the marks drawn | The legend lists what the reader can see; ggplot2's default (`drop = TRUE`) | A subset is not coloured as the whole is: Construction is the fourth colour in the plot of all workers and the first in the plot of the best paid |
| B. Keep every level of a Category, with its colour and its legend entry | One industry has one colour in every plot of the same variable, which helps when two plots are compared | A legend with entries for which nothing is drawn; a Str variable has no levels, so the rule would hold for a Category only |

*Recommendation.* A for the default, with an option of `scale` at step 25 to keep every level, as ggplot2's `drop = FALSE`.

**Question 3. The names of colours.** A string for `color:` or `fill:` is read, as built, as one of the 148 names of the W3C's CSS Color Module Level 4 or as `#` and six or three hexadecimal digits.

| Option | For | Against |
| --- | --- | --- |
| A. The CSS names and the hexadecimal forms only, as built | One published list, the same that the SVG the engine writes understands; a name means one colour | A user of R who writes `"gray40"` or `"firebrick3"` is refused, with the help naming the forms accepted |
| B. Also the 657 names of R's `colors()` | R scripts carry over | Four names differ between the two lists: `gray` and `grey` (`#BEBEBE` in R, `#808080` in CSS), `green` (`#00FF00` and `#008000`), `maroon` (`#B03060` and `#800000`) and `purple` (`#A020F0` and `#800080`); one list must win and the other reader is silently given another colour |

*Recommendation.* A.

Deadline: 0.7.0, before step 25.

**The decision (12 November 2026).** Each question as recommended, taken at step 25d of release 0.7.0 under the owner's standing instruction of that day ("If at any point any decisions to be taken by me, I will go by your recommended decisions"), which he may overturn. Question 1: option A, as built; the eight colours repeat from the ninth category with a note, and `scale(color: [...])` or `scale(color: viridis)`, built at that step, gives the remedy. Question 2: option A for the default, as built; a scale shows the categories of the marks drawn. The option of `scale` to keep every level of a Category, which the recommendation named, is not built at step 25d: it needs a name and a place in the usage line of `scale`, which the catalogue does not have, and nothing drawn today changes when it is added. It is put to the owner here with a deadline of 0.8.0: `p.scale(color: [...], levels: all)`, with `levels: drawn` the default, is the form recommended. Question 3: option A, as built; the 148 names of CSS and the hexadecimal forms, in `scale` as in `color:`.

*Question 2, the argument of `scale` to keep every level (15 November 2026, at the opening of release 0.8.0).* Not built before 1.0, under the owner's instruction of 11 November 2026: `levels: all | drawn = drawn` is an addition that breaks no script and is placed with the grammar extensions of release 1.2 under #122.

### 110. The sizes and shapes of a plot: where a size starts, the shapes and their names, sizes of 0 and below, `group` and `label`, and shapes of equal area

Raised on 11 November 2026 at step 20d of release 0.7.0, where `size:`, `shape:`, `group:` and `label:` of `data.plot(…)` were built. Section 7.6 of the Language Reference names the four aesthetics and the function `scale(color:, fill:, size:, shape:)`; it fixes the default colours and says nothing of the default sizes and shapes. Five things it does not say were met while coding. Each is built in the way recommended below, but for the number of shapes in question 2, and each can be changed before step 25, where the legend is drawn and `scale` is built, without breaking a released script.

**Question 1. Where a scale of sizes starts.** `w.plot(x: educ, y: wage, size: hours)`: `hours` runs from 7 to 69.2.

| Option | For | Against |
| --- | --- | --- |
| A. From 0, by true area, as built: a mark's area is in proportion to its value, the largest value is 18 points across and a value of 0 has no size | The picture is honest: twice the hours is twice the ink; the rule of ggplot2's `scale_size_area()`, of Stata's weighted markers and of Observable Plot's radius scale | A variable far from 0 (years 1990 to 2020, a temperature in kelvin) gives marks of almost one size; a size of 0 is not seen |
| B. From the smallest value to the largest, as ggplot2's default `scale_size()`: the smallest value 1 mm across, the largest 6 mm | Differences show whatever the level of the variable | Area no longer means amount: a value twice another may be drawn twenty times as large, which is the distortion a bar chart avoids by starting at zero (section 7.6 fixes that for `bars`) |
| C. A, with B offered at step 25 as an option of `scale(size:)` | Honest by default, and the other reading is one argument away | One more option to document |

*Recommendation.* C: keep A as the default now and take up the option with `scale` at step 25. The largest size, 18 points, is four times the default point, so the largest mark has sixteen times its area; that figure is the owner's to change.

**Question 2. The shapes, their order and their names.** As built, categories are given circle, triangle, square, diamond, plus and cross in that order; `NA` is an asterisk; a constant is one of those seven names or `circle_open`, `triangle_open`, `square_open`, `diamond_open`. With more than six categories the shapes repeat from the seventh and a note says so. The owner's proposal on graphics capabilities asks at F9 for a fixed order of eight shapes that can be told apart in greyscale; F9 was adopted with open decision #118 on 11 November 2026, which leaves the shapes themselves to this question.

| Option | For | Against |
| --- | --- | --- |
| A. As built | The four shapes with an inside come first, the easiest to tell apart when small; six is what ggplot2 offers too; repeating with a note is the ruling asked for colours in #109, so the two aesthetics behave alike | Beyond six categories two share a shape |
| B. ggplot2's order (circle, triangle, square, plus, a crossed square, an asterisk) and its names with a space (`"circle open"`) | R users know them | The crossed square is hard to read at 4.5 points; a name of two words in a string is easy to mistype, and Stata writes `circle_hollow`, so no spelling is shared anyway |
| C. Refuse more than six categories, or drop the categories beyond the sixth with a warning as ggplot2 does | No two categories share a shape | A first look at the data stops, or silently loses rows, for a reason of appearance |
| D. Eight shapes, as F9 proposes: the six as built in their order, then two more, proposed a triangle on its point (`triangle_down`) and a five-pointed star (`star`) | One shape for each of the eight colours of the palette, so that the redundant encoding the proposal plans (each colour group a shape as well) pairs them one to one; the first six do not change, so nothing built so far changes but the count in the note | The seventh and eighth are the hardest to tell apart at 4.5 points: a triangle on its point from a triangle on its base, a star from an asterisk; the two shapes and their names are still to be chosen |

*Recommendation.* D, because F9 is adopted and asks for eight: the six shapes as built stay the first six, and the two further shapes and their names are the owner's to choose. It is a small `[engine]` step before step 21, which is the first to draw a shape. If the owner prefers six, A stands as built and nothing changes.

*Decided 12 November 2026: option D, with `triangle_down` and `star`.* The owner's words: "Yes, add both". Built at step 20f of release 0.7.0: the palette of shapes is circle, triangle, square, diamond, plus, cross, `triangle_down`, `star`; a constant may also be `"triangle_down_open"` or `"star_open"`; with more than eight categories the shapes repeat from the ninth and a note says so.

**Question 3. A size below 0 and a size of 0.** An area cannot be negative, and an area of 0 cannot be seen.

| Option | For | Against |
| --- | --- | --- |
| A. As built: a row with a size below 0 is dropped and counted in a note ("N rows with `v` below 0 dropped: `size` shows amounts of 0 or more"); a row with 0 is kept, drawn with no size, and counted in a note ("N rows with `v` of 0 are drawn with no size") | Nothing is hidden: the count of points and the notes account for every row; 0 is a true amount and stays in the data of the layer, where a later mark (a label) may still show it | A reader must read the notes to know why a point is not seen |
| B. Refuse a variable with a negative value for `size` | Says at once that the variable is not an amount | One wrong row stops the plot; a balance or a change is often negative for a few rows only |
| C. Size by the absolute value | Every row is seen | A loss of 5 looks like a gain of 5, with nothing in the picture to say so |

*Recommendation.* A.

**Question 4. `group` and `label` before a mark reads them.** No mark built so far reads either: `lines` (step 23) reads `group` and `text` (step 24) reads `label`.

| Option | For | Against |
| --- | --- | --- |
| A. As built: each is accepted, checked to be a variable of the frame, held in the plot and printed in the table ("each value a group", "the text of a mark"), with a line saying that no layer draws it; it drops no row and changes nothing in the scene | A plot written today works unchanged when lines and text arrive; the printed plot says plainly what has no effect yet; ggplot2 likewise accepts an aesthetic no layer uses | An argument is accepted that does nothing until step 23 or 24 |
| B. Refuse them until a mark reads them | Nothing is accepted without effect | The catalogue's row 460 could not be `implemented` as written before step 24, and scripts written between the steps would break |

*Recommendation.* A. The wording of the two lines can be settled when `lines` and `text` are built.

**Question 5. How large a triangle of size 6 is.** A size is the diameter of a circle. For the other shapes the writers of step 21 need a rule.

| Option | For | Against |
| --- | --- | --- |
| A. Equal area, as the comment of the shape module states for step 21: a triangle, a square and a diamond of size s have the area of the circle s points across, and a plus, a cross and an asterisk are s points across | The shapes of one size weigh the same to the eye, so a shape does not suggest an amount, and sizing by area (question 1) holds for every shape; D3 and Observable Plot do this | A triangle of size 6 is wider than 6 points |
| B. Equal width: every shape fits the square s points across, as R's base graphics nearly do | Simple to state | A square has 27% more ink than a circle of the same size and a triangle much less, so shape and size are confused |

*Recommendation.* A. Nothing is drawn before step 21, so this one must be settled by then or built as A.

Deadline: 0.7.0, before step 25.

**The decision on questions 1, 3, 4 and 5 (12 November 2026).** Each as recommended, taken under the owner's standing instruction of that day ("If at any point any decisions to be taken by me, I will go by your recommended decisions"), which he may overturn. Question 1: option C; a size starts at 0 and shows an amount by area, as built, and `scale(size:)` of step 25d offers the other reading, from the smallest value to the largest. Question 3: option A, as built; a size below 0 drops its row and a size of 0 is drawn with no size, each counted in a note. Question 4: option A, as built; since steps 23c and 24d `lines` reads `group` and `text` reads `label`, so the question is closed by the marks themselves. Question 5: option A, as built at step 21; shapes of one size have one area.

### 111. `p_adjust` inside `add`: adjusted p-values as a column, and families with `by:`

Raised on 11 November 2026 by the owner, after samples of step 18b of release 0.7.0 (four methods added to `p_adjust`). A frame of tests with their p-values is the usual starting point, and the natural statement is `t.add(p_holm = p_adjust(p))`. As built at commit 19250e1 it is refused: `p_adjust` returns a `List<Float>` and `add` defines columns only from a column or a single value (Z0401). The workaround is to build a new frame from lists, `frame(p = p, holm = p_adjust(p))`, which loses the other variables of the frame. `p_adjust(t["p"])` accepts a column but still returns a list.

The data functions of the same kind already work in `add`: `t.add(r = rank(p))` gives a column, and `t.add(r = rank(p), by: outcome)` ranks within each group. For multiple testing the grouped form matters in its own right: a study often adjusts within each family of hypotheses (each outcome, each contrast set) and not across all of them, as `group_by(family) |> mutate(p.adjust(p))` does in R. Stata's `mtest()` adjusts within one `test` command and has no command that adjusts a variable of p-values; `qqvalue`, a user-written command, does.

| Option | For | Against |
| --- | --- | --- |
| A. `p_adjust` behaves like `rank`: given a column inside `add` it returns a column of the same length; with `by:` each group is a family adjusted on its own, its count n taken within the group; an `NA` keeps its row and is not counted, as now; given a list it returns a list as now | The statement a user writes first works; families by `by:` with no new argument and no new function (#97); the same rule as the other column functions | `p_adjust` moves from a function on lists to one on columns as well, so its help page and usage gain a second form |
| B. `add` accepts any list whose length equals the number of rows | One rule for every function that returns a list | A list carries no rows, so after a `where` or a sort it is joined to the wrong rows without a word, against "nothing disappears silently"; no `by:`; reaches far beyond `p_adjust` |
| C. Leave it as built; the help page shows the frame built from lists | Nothing to build | The commonest use is refused; adjustment within families must be done by hand, one `where` per family |

*Recommendation.* A, as a small `[function]` step of 0.7.0, because `p_adjust` first ships in 0.7.0 and changing what it returns after the release would break scripts. `posthoc` (step 19) is unaffected: it calls the adjustment internally. Deadline: 0.7.0.

**The owner's decision (11 November 2026).** "Yes, option A." `p_adjust` given a column of a frame returns a column, as `rank` does; with `by:` each group is a family adjusted on its own, its count taken within the group; an `NA` keeps its row and is not counted; given a list it returns a list as before. Built as step 18c of release 0.7.0, the next step to be taken.

### 112. The names of the binary models and of the ROC curve before the 1.0 freeze: `glm` and `curve`

Raised on 11 November 2026 by the owner, after a study of a reference on regression and econometric models for empirical research, which placed some 700 models, tests, plots and options at three levels and planned their releases after 1.0 (the study is kept with the owner's working papers, outside this repository). Under the rule of #97, one function with an option rather than a family of names, the study names one function for every generalized linear model, `glm(y ~ x, family:, link:)`, after the convention of R's `glm()` and Stata's `glm`, and one function for every curve of a classifier, `fit.curve(type:)`. Release 0.9.0 is about to ship `logistic`, `probit` and `fit.roc()` (catalogue rows 262, 263 and 515), and 1.0.0 freezes the names that have shipped. Kept, they would become siblings of `glm` and of `curve` the day the first count model or the first precision–recall curve arrives, and renaming them after 1.0 would break scripts. Nothing else in the study needs to change before 1.0: every other function it plans arrives after 1.0, and every row it places before 1.0 matches the catalogue.

The owner asked on the same day that nothing be added before 1.0 unless it is necessary. These two changes add no work: the estimators, their tests and their reference outputs (R `glm(family = binomial)`, `pROC::roc()`) are those already planned for 0.9.0; only the names change.

Question 4 of #99 proposed the same merge under another name, `logistic(link: logit | probit | cloglog | cauchit | log = logit)`. This entry answers it. Question 3 of #99, which is open, asks whether the routine options R and Stata give enter the release that owns each function before 1.0; for 0.9.0 they are the cloglog, cauchit and log links, odds ratios (`summary(odds_ratios: true)`), successes out of a number of trials, elasticities and group margins in `margins`, and an interval and a comparison test for the area under the ROC curve. That question meets the owner's instruction above, and the owner settles it; whichever way it goes, each option is a word added to `glm`, `margins` or `curve` and can come later without breaking a script.

| Option | For | Against |
| --- | --- | --- |
| A. At 0.9.0, `glm(y ~ x, family: binomial, link: logit \| probit = logit)` with `family:` required, and `fit.curve(newdata:, type: roc)`; further families, links and curve types added as words in the releases that build them | One function for every GLM, as R and Stata have it (#97); the names frozen at 1.0 are the ones the plan after 1.0 builds on; no added work | `glm(family: binomial)` is longer than `logistic` for a newcomer; a diagnostic for `d.logistic(...)`, `d.logit(...)` and `d.probit(...)` that names `glm` covers it |
| B. `logistic(link: logit \| probit)` as question 4 of #99 proposed, `roc` kept | A name every reader knows | A Poisson, gamma or negative binomial model then needs another function, or `logistic(family: poisson)`, which misreads; `roc` beside a later `curve` is a family of names |
| C. Keep the catalogue as it is: `logistic`, `probit`, `roc` | No specification change | Two families of names frozen at 1.0, against #97 |

*Recommendation.* A, with `family:` required (a forgotten family is an error, not a silent linear model, which `reg` fits) and the logit link as the default. On adoption, logged in `docs/spec_changes.md`: catalogue rows 262 and 263 merged into one row `glm` and row 515 renamed `curve`; in the Engine Specification, the 0.9.0 row of section 14, the `glm` package list and its count (4 to 3 functions in 1.0), the total (272 to 271) and the sentence on `margins` "for `reg`, `logistic` and `probit`"; in the Language Reference, the naming example of section 3, the code examples and the separation diagnostic (Z0803) that name `logistic`; the help examples; the sample script; and the 0.3 row of the Zemp Specification. Deadline: 0.9.0, before its plan is written.

**The owner's decision (11 November 2026).** "Yes, option A." At 0.9.0, `glm(y ~ x, family: binomial, link: logit | probit = logit)`, with `family:` required, replaces `logistic` and `probit`, and `fit.curve(newdata:, type: roc)` replaces `fit.roc()`; `d.logistic(...)`, `d.logit(...)` and `d.probit(...)` give a diagnostic that names `glm`. Question 4 of #99 is answered by this entry; question 3 of #99 stays open. The specification edits listed in the recommendation are made as the first step of release 0.9.0, before its functions are built, and not now: the catalogue, the sample script and the help examples are read by the engine's tests and snapshots (`kernel/zeff-registry`, `crates/zeff-engine/tests`), so they change with the full routine of a step, and release 0.7.0 holds the catalogue open in the meantime. The "Start release 0.9.0" plan lists that step first.

### 113. The plan of releases after 1.0, every model placed by level, and the names of the model functions

Raised on 11 November 2026 by the owner, after a study of a reference on regression and econometric models for empirical research (third edition, 33 sections). Every model, test, plot and option it lists, 720 rows with the roadmap items it leaves out, was given a proposed Zeff function, the R or Mplus function its output is matched against, a level and a release. The table is kept with the owner's working papers, outside this repository; this entry records its rules, its releases and its names. Two of its consequences before 1.0 are already decided: `glm` and `curve` at 0.9.0 (#112), and no further option from the review of #99 before 1.0 (question 3 of #99).

**The owner's rules.** (1) Levels follow the reference's curriculum map: basic, the bachelor's level; advanced, the master's level; research grade, the doctoral level and above. (2) All basic models first, then all advanced, then all research-grade ones. (3) Within a level, econometrics, then psychometrics, then machine learning, then the rest (multivariate methods, Bayesian estimation, missing data, surveys, meta-analysis, design, optimisation). (4) Outputs are matched against R or Mplus only; Python is never used as a reference, and Stata is not available to the owner, so a method whose only reference implementation is in Stata is research grade and held in the last release until a Stata reference can be had. (5) Nothing is added before 1.0 unless necessary.

**Names.** One function per kind of model, the variants as option words (#97): `glm(family:, link:)` for every generalized linear model; `penalized(penalty: lasso | ridge | elastic_net | adaptive_lasso | group_lasso | scad | mcp | sqrt_lasso, family:)`; `discrete_choice(model: conditional | nested | mixed | latent_class | rank_ordered)`; `selection(type: sample | treatment | switching)`; `did(method:)`, `synth(method:)`, `quantile_reg(method:)`, `robust_reg(method:)`, `decompose(method:)`, `cluster(method:)`, `embed(method:)`, `cov_test(type:)`, `normality_test(method:)`, `factor_count(method:)`. Arguments mean the same on every estimator: `fe:`, `re:`, `instruments:` (two-stage least squares for linear models, a control function for nonlinear ones), `se:`, `cluster:`, `weights:`, `scale:` (a model for the variance), `prior:` (Bayesian estimation of the same model). Shapes of an outcome or a term are formula helpers (`surv()`, `interval()`, `s()`, `spline()`); diagnostics, plots and interpretation are methods of the result (`fit.het_test(type:)`, `fit.serial_test(type:)`, `fit.spec_test(type:)`, `fit.plot(type:)`, `fit.curve(type:)`, `fit.importance(method:)`, `fit.feature_effects(type: pdp | ice | ale)`). Roadmap catalogue rows merged: `logistic`, `probit`, `poisson`, `negbin`, `gamma_reg` and `fractional` into `glm`; `glmm` into `mixed(family:)`; `lasso` into `penalized` and `spec.lasso` into `spec.penalized`; `conditional_logit`, `nested_logit` and `mixed_logit` into `discrete_choice`; `blp_demand` renamed `demand`; `heckman` renamed `selection`; `lee_bounds` renamed `bounds`; `ipw` and `entropy_balance` into `weighting`; `aipw` renamed `doubly_robust`; `late` and `cuped` into `experiment`; `oster` into `sensitivity(method:)`; `loess` into `kernel_reg(method:)`; `kmeans`, `kmedoids`, `hierarchical`, `dbscan` and `gaussian_mixture` into `cluster`; `tsne` and `umap` into `embed`; `jarque_bera_test` and `mardia_test` into `normality_test` (`shapiro_test`, built, stays); `box_m_test` and `sphericity_test` into `cov_test`; `parallel_analysis` into `factor_count`; `reset_test` into `spec_test`; `durbin_watson` and `ljung_box_test` into `serial_test`; `partial_dependence` renamed `feature_effects`; and `panel_reg(effects:)` renamed `panel_reg(method:)`.

**The releases.**

| Release | Level | Domain | Contents |
| --- | --- | --- | --- |
| 1.1 | Basic | Engine | As section 14 has it: thread pool, bootstrap in the variance layer, readers, `external`, `fetch`, `unnest` |
| 1.2 | Basic | Econometrics | Diagnostic tests and plots, model selection, Box–Cox, LOESS, time-series basics (moving averages, exponential smoothing, AR, forecasts); the plot layers of the old 1.2 |
| 1.3 | Basic | Psychometrics | PCA, basic EFA, KMO and Bartlett, scale scores, item statistics |
| 1.4 | Basic | Machine learning | Train/test split, k-fold cross-validation, the regression tree, error metrics |
| 1.5 | Basic | Other | Clustering, linear discriminant analysis, correspondence analysis, contingency plots, association rules, linear programming |
| 1.6 | Advanced | Econometrics | Linear models and inference: GLS, constraints, splines, moderation, ridge and lasso, PCR and PLS, bootstrap intervals |
| 1.7 | Advanced | Econometrics | GLM families, counts, ordinal, multinomial, choice, Tobit, Heckman, fractional and beta models |
| 1.8 | Advanced | Econometrics | Endogeneity, systems and panels: IV diagnostics, control functions, SUR, 3SLS, FE/RE/FD, Hausman, GEE; `stream` |
| 1.9 | Advanced | Econometrics | Causal designs: experiments, matching, weighting, 2×2 DiD, sharp and fuzzy RD, synthetic control |
| 1.10 | Advanced | Econometrics | Time series, forecasting and survival |
| 1.11 | Advanced | Econometrics | Multilevel, quantile, robust and nonparametric models; cross-sectional frontier and DEA |
| 1.12 | Advanced | Psychometrics | Full EFA, CFA, SEM, reliability and validity, basic invariance, mediation and conditional process |
| 1.13 | Advanced | Psychometrics | PLS-SEM, LCA, LPA |
| 1.14 | Advanced | Machine learning | The tuning workflow, forests, boosting, SVM, kernel ridge, neural networks, stacking, conformal prediction, interpretation |
| 1.15 | Advanced | Other | User estimators and optimisation (`mle`, `gmm`, `minimize`, automatic differentiation, matrices), text, networks, spatial description; third-party packages and user-defined types of the old 1.11 |
| 1.16 | Advanced | Other | Multivariate methods: MANOVA, Hotelling, covariance tests, CCA, QDA, MDS, MCA, conjoint, log-linear models, anomaly detection |
| 1.17 | Advanced | Other | Bayesian estimation (on the automatic differentiation of 1.15), missing data, surveys, meta-analysis, design (the old 1.9) |
| 1.18 | Research | Econometrics | Inference and identification: few-cluster and multiway inference, weak IV, IV-free methods, post-double-selection |
| 1.19 | Research | Econometrics | Design-based inference: staggered DiD (the old 1.2), synthetic DiD, RD variants, bunching, MTE, sensitivity |
| 1.20 | Research | Econometrics | Causal machine learning and policy learning (from the old 1.5) |
| 1.21 | Research | Econometrics | Advanced panel econometrics |
| 1.22 | Research | Econometrics | Structural econometrics and efficiency |
| 1.23 | Research | Econometrics | Advanced time series; distributional and nonparametric econometrics |
| 1.24 | Research | Econometrics | Spatial and network models, advanced survival, measurement and synthesis |
| 1.25 | Research | Psychometrics | The SEM frontier: ordinal CFA (the old 1.6), bifactor, ESEM, Bayesian and penalized SEM, alignment, latent interactions, RI-CLPM, multilevel SEM, MASEM |
| 1.26 | Research | Psychometrics | Latent transitions, mixtures, IRT and DIF, intensive longitudinal models, network psychometrics |
| 1.27 | Research | Machine learning | Nonconvex and grouped penalties, tree and kernel variants, explainable boosting, fairness, bandits |
| 1.28 | Research | Other | Bayesian and multivariate frontier: model averaging, BART, Gaussian processes, robust and high-dimensional multivariate methods, copulas, graphical models |
| 1.29 | Research | Stata only | The 20 methods whose only reference is Stata, held until a Stata reference is available |
| — | — | Not planned | Deep learning on images and text, LLMs, model-free reinforcement learning, AutoML, process mining, discrete-event simulation |

The old releases move as follows: 1.1 stays; 1.2 to 1.19 (its plot layers to 1.2, PPML to 1.7); 1.3 to 1.12; 1.4 to 1.8 and 1.11; 1.5 to 1.4, 1.14, 1.18 and 1.20; 1.6 to 1.12 and 1.25; 1.7 to 1.13 (`stream` to 1.8); 1.8 to 1.15; 1.9 to 1.17; 1.10 to 1.17 and 1.28; 1.11 to 1.15. User estimators come before Bayesian estimation because Hamiltonian Monte Carlo needs the gradients of automatic differentiation. At the cadence of section 14, a minor release about every three months, releases 1.2 to 1.29 take about seven years.

| Option | For | Against |
| --- | --- | --- |
| A. Adopt the plan as described, names and releases; the catalogue's release column, its merged rows and the after-1.0 table of section 14 amended in one step, and each release's contents confirmed when it is planned | One complete map of the language to 1.29; every name consistent with #97 before any of them is built; a user meets basic methods first | Seven years of releases; a research-grade method a referee asks for today waits for its phase |
| B. Adopt the order of releases only; names settled family by family when each release is planned | Less to decide now | Names chosen piecemeal drift apart, the risk #97 was written to remove |
| C. Keep the after-1.0 table of section 14 as it is | No change | Does not follow the owner's rules; leaves the merges undone |

*Recommendation.* A. The amendment touches only planned material (the catalogue's roadmap rows and the after-1.0 table), which the engine's catalogue checks read, so it is made with the full routine of a step, as the first documentation step of release 0.8.0, beside the merges of #97 and #99, and logged in `docs/spec_changes.md`. Still the owner's: the six names least certain (`discrete_choice`, `selection`, `experiment`, `weighting`, `dimension_reg`, `penalized`); the merge of `glmm` into `mixed(family:)`; the levels of five borderline methods (PLS-SEM advanced, sharp and fuzzy RD advanced, Callaway–Sant'Anna DiD research, IRT research, meta-analysis advanced); and the ten methods left unplanned. Deadline: 0.8.0.

**The owner's decision (11 November 2026).** "Yes, option A." The plan stands as described: the 29 releases after 1.0 in the order above, the naming scheme and the merges of roadmap rows. The catalogue's release column, its merged rows and the after-1.0 table of section 14 are amended as the first documentation step of release 0.8.0, with the full routine of a step, and logged in `docs/spec_changes.md`; the "Start release 0.8.0" plan lists that step. Each release's contents are confirmed when it is planned. Left open, for the owner to settle when the release that builds each one is planned: the six names least certain, the merge of `glmm` into `mixed(family:)`, the levels of the five borderline methods and the ten methods left unplanned.

**Added by the owner (11 November 2026).** The same documentation step of 0.8.0 brings the release plan of the Zemp Specification (section 15) into line: its row for Zemp 0.3 names the `glm` and `curve` of #112 in place of "dialogs for logistic and probit regression", and its rows 1.2 to 1.11 are remapped to the releases of this entry, Zemp's minor number following Zeff's by the convention of that section, so that each Zemp Studio release adds the dialogs and guides for the methods of the Zeff release of the same number; Zemp Lite gains each release's methods through the engine, as now. The change is logged in `docs/spec_changes.md` with the others.

### 114. "Copy as Zeff" in Zemp Lite's viewers

Raised on 11 November 2026 by the owner, from mockups of Zemp Lite's data viewer. In Zemp Studio each viewer offers **Add to script**, which writes the statement equivalent to what the user did by pointing (a sort, a filter, a selection of rows) at the cursor and shows it first (Zemp Specification, section 5). Zemp Lite leaves out every control that writes a statement (section 2, the table of parts), so a Lite user who filters the data grid to the rows of interest must type the `where()` that keeps the filter. The specification does not say whether a control that only puts the statement on the clipboard counts as one that writes a statement. The same question holds for the plot viewer's selections and for the variable view, where Studio writes `label`, `recode` or `category`.

| Option | For | Against |
| --- | --- | --- |
| A. **Copy as Zeff** in Lite's data, plot and variable viewers: the equivalent statement on the clipboard, the script untouched until the user pastes it; a selection of more than 20 rows copied as the same `merge` on a file under `data/selections/` that Studio writes | The reproducible statement is one keystroke away without the user retyping a filter; the script still changes only by the user's own hand, so Lite stays code-first; the code that builds the statement already exists for Studio's **Add to script** | Blurs the line section 2 draws, so its table of parts needs a row; the selection file under `data/selections/` is a write to the project, which Lite otherwise never makes from a viewer |
| B. As now: nothing in Lite turns a view into a statement | The rule of section 2 stays simple: Lite never writes Zeff for the user | A user who wants the filter in the script retypes it, and may type it differently from what the viewer showed |
| C. A, without selections: sorts, filters and variable changes only | No file written from a viewer in Lite | Selections of rows, the case where retyping is hardest, are left out |

*Recommendation.* C: the sorts, filters and label changes are short statements and copying them keeps Lite code-first, while a selection of many rows needs a file, which a code-first edition should leave to the user's own `save()`. On adoption, the table of parts in section 2 of the Zemp Specification gains the row, logged in `docs/spec_changes.md`. Deadline: Zemp 0.2 alpha, with Zeff 0.8.0, when the Lite web preview first ships its viewers.

**The owner's decision (11 November 2026).** Option C. Zemp Lite's data, plot and variable viewers offer **Copy as Zeff**: the statement equivalent to a sort, a filter or a change of type, label or contrast goes to the clipboard, and the script changes only when the user pastes it. A selection of rows is not copied, since it needs a file under `data/selections/`; a Lite user who wants one writes the `save()` and `merge()` by hand. Zemp Studio keeps **Add to script**. The table of parts in section 2 of the Zemp Specification gains the row, and section 5's paragraph on Zemp Lite names it, both logged in `docs/spec_changes.md` the same day.

### 115. Zemp Lite's look: colour scheme, syntax colours, type and the screens of the mockups

Raised on 11 November 2026 by the owner, after mockups of Zemp Lite drawn from sections 2, 5, 7 and 8 of the Zemp Specification (kept with the owner's working papers, outside this repository). The specification fixes Lite's layout and controls but not its colours, type or the look of its screens; #33 holds the open questions of density, icons, desktop menus and small screens, and the dark theme was decided on 17 October 2026. The plots' own colours are not part of this entry: they are the engine's (Okabe and Ito for categories, viridis for amounts; section 7.6 of the Language Reference), and exported tables and figures always use the light print theme (Zemp Specification, section 5).

**Colour tokens.** One name per role, so that every panel takes its colours from the same few values and the dark theme is a second set of the same names.

| Token | Use | Light | Dark |
| --- | --- | --- | --- |
| page | Behind the windows | #f7f7f5 | #0f1013 |
| window | Panels, editor, viewers | #ffffff | #16171b |
| soft | Output boxes, hovered rows | #f9f9f8 | #1b1d22 |
| line, line strong | Dividers; borders of controls | #ebebe8, #dededa | #24262c, #2e3037 |
| ink | Main text | #1b1b1f | #e9e9ec |
| ink 2 | Secondary text | #55555c | #a7a8b0 |
| ink 3 | Captions, line numbers, timings, margins | #6b6b73 | #8f919c |
| ink 4 | Decoration only, never text: the faded stale plot, inactive dots | #b9b9bf | #4d4f57 |
| accent, accent soft | Focus, the selected row, links | #3d5afe, #eef1ff | #7d8dff, #1f2440 |
| warning, warning soft | Warnings and the "edited" mark; the warning card | #8a5d00, #fbf6e9 | #e2b552, #272215 |

The **Run** button is drawn in ink on window (inverted), the one filled control in the top bar; every other control is outlined.

**Syntax colours.** Seven categories, as the Zeff language server reports them (semantic tokens), the same in the editor, the console and every place code is shown:

| Category | Examples | Light | Dark |
| --- | --- | --- | --- |
| Keyword | `let`, `fn`, `for`, `in`, `if`, `model` | violet #7b4fd6 | #c4a7ff |
| Function or method | `reg`, `where`, `plot`, `derive` | blue #1f63c2 | #82b8ff |
| Option | `se:`, `fe:`, `color:` | rose #a3367c | #f29ac4 |
| Operator | `~`, `+`, `*`, `==`, `=`, `->` | teal #0f7b8a | #6fd3dc |
| Text in quotes | `"output/table2.docx"` | green #2b7a4b | #8fdcab |
| Number | `2010`, `0.5` | amber #a8520c | #f4b77e |
| Comment | `// Wage gap by sex` | grey italic #6b6b73 | #8f919c |

Variables of a frame and bindings take the main ink (#3a3a42 light, #d4d5da dark); the colours mark structure, never data.

**Contrast.** Every colour used for text reaches at least 4.5:1 against the ground it sits on (the WCAG AA level for normal text), in both themes; measured for the values above against the window and the soft grounds. Three values of the mockups are corrected here for that reason: comments and ink 3 (#9a9aa2 and #8e8e96 in the mockups, 2.8:1 and 3.3:1), the number colour (#b5590f, 4.5:1 on the soft ground) and the warning colour (#9a6a00, 4.4:1 on its card); ink 4 is kept for decoration only.

**Type.** IBM Plex Sans for the interface and IBM Plex Mono for code, the family the engine's plots embed (step 21 of release 0.7.0), shipped with the app so that nothing is fetched at run time; interface text 13 to 15 points, code 13 points with a line height of 1.75, numbers in tables with tabular figures. Ctrl + and Ctrl − change the size (section 5).

**Shapes and space.** Windows with 14-pixel corners and one hairline border, output boxes and cards with 10-pixel corners, controls with 7-pixel corners; no gradients; a faint shadow under windows in the light theme only. The Inline column at most 760 pixels wide, with the line number, the code and a 120-pixel margin for timings and states.

**The screens.** The mockups are the reference for the first build: the welcome screen; the Inline layout, with output in soft boxes under each statement, warnings as an amber card with the code and one sentence, and stale output dashed and faded with its reason ("Out of date · line 5 changed after it ran · ⌘↵ to run again"); the dark theme; the Panes layout with the hover help of a function; the Git control and panel; the data grid, with sorting and filtering as removable pills and "View only · the script is unchanged", and the variable view with its distribution sketches.

**Two points for #33.** The Git control of the mockups carries a small branch icon, where #33 recommends no icons and section 5 says the top bar's controls are all words: this entry draws it as words, the branch and the count of changed files ("main · 2"). The mockups use one density.

| Option | For | Against |
| --- | --- | --- |
| A. Adopt the tokens, syntax colours, type, shapes and screens above as the reference for Zemp Lite and Zemp Studio, the two editions sharing them; a short design section added to the Zemp Specification when the interface framework (#24) is chosen | The first build starts from a settled look; one set of tokens keeps both themes and both editions consistent; the contrast rule is written down before any screen is built | A look fixed before users have tried it; changes after the 0.2 alpha's reader test cost a little rework |
| B. Adopt the syntax colours and the contrast rule only; the rest settled at the 0.2 alpha from user feedback | The parts that touch code and accessibility are fixed; the rest stays open | The first build starts without a reference for its screens |
| C. Leave the look to the interface work | Nothing to decide now | Colours chosen piecemeal by component |

*Recommendation.* A, with the corrections for contrast and the Git control in words. Deadline: Zemp 0.2 alpha, with Zeff 0.8.0, when the Lite web preview is built.

**The owner's decision (11 November 2026).** "Yes, option A." The colour tokens, syntax colours, contrast rule, type, shapes and screens above, with the three colours corrected for contrast and the Git control in words, are the reference for Zemp Lite and Zemp Studio alike. A short design section carrying them is added to the Zemp Specification when the interface framework (#24) is chosen, and logged in `docs/spec_changes.md` then; until that day this entry is the reference. The mockups, updated the same day to the corrected values, stay with the owner's working papers.

### 116. Knowing how many use Zeff and Zemp, and who: counts, optional registration and citations, without a required login

Raised on 11 November 2026 by the owner, who wants to keep track of how many download Zeff and Zemp, who they are, and how many use the browser version, and asked whether a login system is needed. The specification says nothing on measuring use; what it says constrains the answer: Zemp is "free and private" and data stays on the user's machine (Zemp Specification, section 1, principle 7, and section 13); there is no telemetry by default and crash reports are opt-in (sections 9 and 13); the web editions run with no server, as #27 recommends; a replication package must run anywhere with `zeff verify`, offline, on a referee's machine (Engine and Project Specification, section 2.1); and Zeff is open under Apache 2.0, so a requirement in the code could be removed by anyone.

**A required login does not fit.** It would stop `zeff verify` on a referee's machine and Zemp on locked-down university computers; it would need a server and an account database, against section 13 and #27; holding accounts makes the project a data controller under the UK GDPR, the EU GDPR and India's Digital Personal Data Protection Act 2023, with a privacy policy, consent, deletion on request and breach duties; and the open code could be built without it.

**The proposal: three layers.**

1. *Anonymous counts (no personal data, no cookies).* Downloads by version and platform, from the release host and the package managers' own statistics (Homebrew, winget, Scoop); engine fetches by version and operating system, counted by the server the launcher downloads from, without the address of the machine being kept; and visits to the web editions, installs of their offline apps, countries, browsers and screen sizes from a cookie-free, aggregate page counter on the web host (Plausible or Cloudflare Web Analytics, or one run by the project), which needs no consent banner. Nothing is sent by the app itself, so the rule of no telemetry stands.
2. *Optional registration (who uses the tools).* **Register for wrong-result alerts**: an email list with optional fields (institution, role, field, country). A registered user is told by email when a known wrong-result bug (the known-issues list that `zeff audit` reads; Language Reference, section 11.1) affects the engine version they named. One quiet entry in ⌘K and on the welcome screen, never a prompt; a privacy notice saying what is kept, where and for how long; leaving the list in one step. This layer alone holds personal data, and only of those who chose to give it.
3. *Citations (use in research).* A `CITATION.cff` file and a DOI for each release (through Zenodo), and "how to cite" in `zeff version` and the help; papers citing Zeff are the measure of use that matters for an academic tool and for research-impact assessments.

**Accounts later.** If paid features are ever offered (cloud sync, version history, Pro; #27, #35, #36), an account is needed for those features alone. Zeff, the `zeff` command line, Zemp Lite and Zemp Studio's free tier stay usable without any account.

| Option | For | Against |
| --- | --- | --- |
| A. The three layers, no login | Counts of every download and visit; a picture of who uses the tools from those who register, with a real reason to register; the academic measure through citations; nothing broken in replication, offline use or privacy; the least legal burden | "Who" is known only for those who register, never for everyone |
| B. An optional login offered to everyone, with the alerts and settings carried across machines | More users identified than by an email list | A server and an account database before any paid feature needs one; most of the legal burden of C |
| C. A login required to download or use | Every user identified | Breaks `zeff verify` and offline use, contradicts sections 1 and 13 and #27, carries the full legal burden, and is removable from the open code |

*Recommendation.* A, introduced in step with the releases: layer 1 with the Zemp 0.2 alpha (Zeff 0.8.0), when the web preview first has visitors; layer 3 at 0.10.0, when the validation suite is published and there is something to cite; layer 2 with 1.0, when users rely on results that the alerts protect. On adoption, section 13 of the Zemp Specification gains a paragraph listing exactly what is counted and what is never collected, and section 9's telemetry row points to it, logged in `docs/spec_changes.md`. Deadline: Zemp 0.2 alpha, for layer 1; this is general information about data-protection law, not legal advice.

**The owner's decision (11 November 2026).** "Yes, option A." The three layers, with no required login: anonymous counts from the Zemp 0.2 alpha (Zeff 0.8.0), a citation file and a DOI per release from 0.10.0, and the optional registration for wrong-result alerts from 1.0; an account only for paid features, if they are ever offered. Section 13 of the Zemp Specification gains the paragraph listing what is counted and what is never collected, and section 9's telemetry row points to it, logged in `docs/spec_changes.md`. The release that brings each layer lists its work (the host's counter and the launcher's anonymous count at 0.8.0, the citation file and DOI at 0.10.0, the list and its privacy notice at 1.0).

### 117. Opening a shared zip project in Zemp, one file from a zip of several, and keeping browser projects safe

Raised on 11 November 2026 by the owner, who wants a whole project to be shareable as a zip, replication files included, and asked where browser projects are kept. What the specification already gives: no login for any edition (#116); browser projects in the browser's private file system on the user's own device, exportable or synced to a folder (Zemp Specification, section 10); `import` reading a compressed file, a `.gz` or a `.zip` holding one file, as its content (Language Reference, section 6, with the formats of 0.11.0); `zeff export replication` building the project's zip and `zeff verify <file.zip | folder>` unpacking it into a temporary folder and re-running it offline (Engine and Project Specification, section 2.1; 0.11.0). Three gaps remain.

1. *Zemp cannot open a zip project.* Its welcome screen offers New project, Open folder, Clone from Git and Scratch project (section 5); a user who receives a replication package must unzip it by hand, and in the browser there may be no folder to open (Safari and Firefox do not give a web page access to a local folder), so the package cannot be opened at all.
2. *A zip of several files cannot be imported.* `import` reads a zip holding exactly one file; data are often shared as one archive of several tables.
3. *Browser storage can vanish.* A browser deletes a site's private storage when the user clears its data, and may evict it under storage pressure unless the site holds persistent storage; nothing in the specification asks for it or warns the user.

**The proposal.**

| Piece | What it does | Reference |
| --- | --- | --- |
| **Open a replication package…** | On the welcome screen of both editions and in ⌘K: choose a `.zip` (or drop it on the window); Zemp unpacks it into a new project, in the private file system on the web or a folder the user picks on the desktop, refusing a path that would leave the project (`..`, absolute paths) and a package larger than the memory budget; shows the package's first page (engine version, how to run, external steps); opens the project with the engine it pins, loading or installing it as for any project (section 4) | `zeff verify`, which unpacks the same zip |
| **Verify** | A button on that first page: runs every script of the run record fresh and offline, as `zeff verify` does, and shows *identical* or each difference, on the web as on the desktop; the external steps of a desktop project are not run on the web, and the result says so | `zeff verify --fresh --offline --no-external` |
| `import(path, file: "wages.csv")` | Reads one named file of a zip that holds several; without `file:`, a zip of several files is an error that lists them; the hash recorded is the archive's and the member's | R `unz()`, `readr::read_csv(unz(...))` |
| Persistent browser storage | Zemp web asks the browser for persistent storage when the first project is created; when it is refused, or a project has not been exported for 30 days, a line in the status bar says so, with **Export project** in ⌘K; nothing is exported without the user | the browser's `navigator.storage.persist()` |

| Option | For | Against |
| --- | --- | --- |
| A. All four at 0.11.0, with the Zemp 0.5 desktop beta, when replication packages arrive | A shared zip opens and verifies in either edition, the web included, with no command line; data shared as one archive import directly; browser users are warned before they lose work | Four more pieces for 0.11.0, already a large release; Verify on the web must load the pinned engine version |
| B. Open and Verify only; `file:` and the storage warnings after 1.0 | The main need, a shared project, is met | A multi-file archive still needs unzipping; browser projects can still vanish without warning |
| C. Nothing in Zemp: users unzip by hand and verify from the command line | No work | No way at all on the web in browsers without folder access; the replication story stops at the command line |

*Recommendation.* A. The storage warning is small and protects users from losing work, and `file:` is a few lines on the reader of 0.11.0. On adoption, section 5 of the Zemp Specification (the welcome screen), section 10 (Zemp web's storage), section 6 of the Language Reference (`import`'s `file:`) and the catalogue row of `import` are amended, logged in `docs/spec_changes.md`; the 0.11.0 plan lists the steps. Deadline: 0.11.0.

**The owner's decision (11 November 2026).** "Yes, option A." All four pieces at 0.11.0, with the Zemp 0.5 desktop beta: **Open a replication package…** and **Verify** in both editions, web included; `import(path, file:)`; and persistent browser storage with the export reminder. Sections 5 and 10 of the Zemp Specification are amended the same day, logged in `docs/spec_changes.md`. The amendment of `import` in section 6 of the Language Reference and in its catalogue row is made at the step of 0.11.0 that builds `file:`, since the catalogue is read by the engine's tests; the 0.11.0 plan lists the steps.

**Built in part at Zemp Lite 0.2.2 (22 November 2026, #208).** By the owner's decision of that day, the parts of this entry that need no run record come with Zemp Lite 0.2.2, on Zeff 0.8.0: **Export project…** as a zip, the opening of an exported zip as a new project (without **Verify**, without the package's first page and without a pinned engine, none of which a plain copy of a project has), the request to the browser for persistent storage when a project is first made, and the reminder to export. **Open a replication package…** with **Verify** stays at 0.11.0, with the run record it reads; `import(path, file:)` stays at 0.11.0 with the reader it belongs to.

### 118. The plot renderer's fixed points before step 21 of 0.7.0

Raised on 11 November 2026 by the owner's proposal on graphics capabilities (`docs/proposals/zeff_graphics_capabilities.md`, section 4, decisions F1 to F12, and `docs/proposals/zeff_diagrams.md`, section 3.1, the amendment of F1). Each item is cheap to settle before the SVG writer (step 21) and the PDF and PNG writers (step 22b) are built and their bytes are fixed by snapshot tests, and costly after. Steps 20 to 20d have built the scene of marks in data units (section 7.6 of the Language Reference); the items below concern what it may carry and how the writers draw it. F6 is #119 and F12 is #120.

| # | Proposal | Where 0.7.0 stands | Recommendation |
| --- | --- | --- | --- |
| F1 and D3 | A closed set of drawing primitives the writers support: point with a shape, path of line, cubic Bézier and elliptical-arc segments, polygon with holes, rectangle, segment with arrow heads, text run with styled spans and an optional box (padding, corner radius, fill, border, wrap width), raster cell grid, clip region; every primitive may carry an element name (`node:engage`), so an exported SVG has named groups; no gradients, images from files or blur | Not built; the writers start at step 21 | Adopt at step 21: every mark and every later layer, diagram and map reduces to these, so no later release needs a new primitive; element names name shapes, never data rows |
| F2 | Each mark in the viewer's scene carries the binding and row positions it came from; exports never carry them | The scene keeps each layer's values in row order | Adopt; the writers of steps 21 and 22b drop them, checked by a test that no export contains a row position |
| F3 | Three optional reserved scene fields: a frame index (animation), a `z` coordinate with a projection (3D), a `crs` on the coordinate system (maps); a reader that does not know one refuses the scene with a message | Not present | Adopt in scene version 1, unused until their families arrive |
| F4 | A layer flag `raster: true` that draws one layer as a raster grid inside a vector export | Not present | Adopt the flag in scene version 1; the option on the marks after 1.0 (#122) |
| F5 | `scale_x(limits:)` and `scale_y(limits:)` set the visible range only: marks outside are clipped and counted in a note, statistical layers use every row, and removing rows is `where` | Step 25 builds `limits:` | Adopt for step 25; ggplot2's default, where limits drop rows and change box plots and smooths, is the trap avoided |
| F7 | The note on dropped or clipped rows shows in the viewer, the run record and `p.notes()`, and stays off an exported figure unless `export(notes: true)` | Notes print with the plot | Adopt for steps 21 and 22b |
| F8 | Value labels name legend keys and facet strips; `facet(strip: full)` writes `label: value` | Legend keys built so at 20c | Adopt; strips at step 27, `strip: full` after 1.0 |
| F9 | Fixed orders of eight shapes and six line types, distinguishable in greyscale | Six shapes built at 20d; their order and names are question 2 of #110 | Settle the shapes with #110; fix the line-type order at step 23, which builds `lines` |
| F10 | Ticks by the extended Wilkinson algorithm (Talbot, Lin and Hanrahan 2010, as R's `labeling` package and ggplot2 use) | Built at step 20 by the rule section 7.6 states, steps of 1, 2 or 5 times a power of ten (Observable Plot and D3), in every help page and snapshot since; the samples of step 20 showed the difference (8, 10, 12, … against ggplot2's 7.5, 10, 12.5, …) | **The owner's choice.** Keeping the built rule gives rounder steps and changes nothing; the extended algorithm matches ggplot2's axes but changes every figure built since step 20 and needs a specification change. Recommended: keep the built rule |
| F11 | Every layer that draws an interval records its kind and level in the scene (`ci 0.95`, `se 1`, `sd 1`, `range`), and the legend or caption states it | `errorbars` is built at steps 28 and 30 | Adopt for those steps |

*Recommendation.* Adopt F1 with D3, F2, F3, F4, F5, F7, F8, F9 with #110 and F11 as in the table; keep the tick rule of step 20 for F10. On adoption, section 7.6 of the Language Reference and the help pages of the steps concerned carry each rule when its step is built, logged in `docs/spec_changes.md`. Deadline: 0.7.0, before step 21.

**The owner's decision (11 November 2026).** "Option A, keep the ticks." F1 with D3, F2, F3, F4, F5, F7, F8, F9 and F11 are adopted as the table says, and the ticks stay on the rule of section 7.6 built at step 20 (steps of 1, 2 or 5 times a power of ten); no figure, help page or snapshot built so far changes. Each rule is written into the 0.7.0 step that builds it: F1, F2, F3, F4 and F7 at step 21 and step 22b; F9's line types at step 23; F5 at step 25; F8 at step 27; F11 at steps 28 and 30. The order of the shapes stays with question 2 of #110. Section 7.6 of the Language Reference gains each rule when its step is built, logged in `docs/spec_changes.md`.

### 119. Names reserved before the 1.0 freeze for diagrams and rich text

Raised on 11 November 2026 by the owner's proposals on graphics (`docs/proposals/zeff_graphics_capabilities.md`, F6) and diagrams (`docs/proposals/zeff_diagrams.md`, D1). Two later features need a name settled before 1.0 freezes `core` and the reserved words, or they can never arrive without breaking a script.

1. *The `diagram { }` block* (flowcharts and conceptual models, #121) is a block like `dag { }` and `model { }`. Since `if x { … }` makes a bare name followed by `{` legal elsewhere, the block cannot be recognised by context: `diagram` must join the words reserved for later use (section 3.1 of the Language Reference: 26 today, eight of them for later), and the type `Diagram` be declared in `core` beside `Dag` and `Model`. A script naming a column `diagram` is then warned at 1.0 rather than broken later.
2. *Rich text in labels* (italic, bold, superscripts, Greek letters, as in "*R*² within"): if label strings were later read as markup, every existing label containing `*` or `^` would change. A separate type keeps it additive: `RichText` reserved as a type name in `core` now, and `rich("*R*^2^ within")` after 1.0, which label parameters then accept as `Str | RichText`.

| Option | For | Against |
| --- | --- | --- |
| A. Reserve `diagram` and the type names `Diagram` and `RichText` at 0.11.0, with the other words of #84 | Both features arrive after 1.0 as additions; the cost is three names | A column named `diagram` needs backticks from 1.0 |
| B. Reserve nothing | No change | The diagram block can only arrive with a new edition; rich text only by a breaking change to labels |

*Recommendation.* A. The owner's rule that nothing be added before 1.0 unless necessary is met: a reserved name adds no function, and it is the only way the two features can come later. Deadline: 0.11.0, with #84.

**The owner's decision (11 November 2026).** "Option A." At 0.11.0, in the same step as #84: `diagram` joins the words reserved for later use (27 reserved words, nine of them for later, more if #84 adds `each` or `peek`), and `Diagram` and `RichText` are reserved as type names in `core`, with no constructor and no function in 1.0. A column named `diagram` is written in backticks from that release. Sections 3.1, 3.4 and 3.6 of the Language Reference are amended in that step and logged in `docs/spec_changes.md`; the 0.11.0 checklist carries the step when that release is planned.

### 120. Result plots: one `_plot` method per kind with a `kind:` option, and no further plots in 1.0

Raised on 11 November 2026 by the owner's proposal on graphics (`docs/proposals/zeff_graphics_capabilities.md`, rule G10, F12, sections 7 and 14). The proposal names derived plots as `_plot` methods on results (`fit.residual_plot()`, `fit.influence_plot()`, `fit.prediction_plot()`), defined once per result type in the package that owns it, because rule 6 of section 8.1 of the Engine and Project Specification allows one definition of `plot` for every `Result` and no second one for a subtype. The model table of #113 wrote some of the same plots as words of one `fit.plot(type: residuals | qq | scale_location | leverage | …)`; that cannot stand under rule 6, since the words differ by result type. The two are reconciled by #97 applied within each kind: one `_plot` method per kind of figure, its variants as a `kind:` option, as the proposal already writes `residual_plot(kind: all | fitted | qq | scale_location | histogram)`.

The proposal also names four plots as candidates for 1.0 (`residual_plot`, `prediction_plot` with `predict(se: true)`, `influence_plot`, `roc_plot`). Release 1.0 has `fit.plot()` and `fit.coef_plot()` (step 30). Under the owner's rule that nothing be added before 1.0 unless necessary, and since each of the four is an addition that breaks no script when it comes later, they wait; the ROC figure is `fit.curve(type: roc).plot()` under #112, not a separate `roc_plot`.

| Option | For | Against |
| --- | --- | --- |
| A. `_plot` methods, one per kind, each with a `kind:` option; `fit.plot()` stays each type's default view; the four candidates at 1.2, with basic econometrics (#113) | Consistent with rule 6 and #97; a reader finds `influence_plot` by its name; nothing added to 1.0 | The rows of #113's table that use `fit.plot(type:)` are renamed in the documentation step of 0.8.0 |
| B. As A, with the four candidates in 1.0 | A regression release with its diagnostic and prediction plots, as R's `plot.lm` and Stata's postestimation give | Four functions and `predict(se: true)` before 1.0, against the owner's rule; a step for 0.9.0 or 0.10.0 |
| C. One `fit.plot(type:)` for everything | One name to learn | Breaks rule 6, or makes `type:` mean different things on different results, against principle 8 (one meaning per argument) |

*Recommendation.* A. Deadline: 0.7.0, before step 30, which builds `fit.plot()` and `fit.coef_plot()` and so fixes the convention.

**The owner's decision (11 November 2026).** "Option A." A derived plot of a result is a method named `<kind>_plot`, one per kind of figure, defined once per result type in the package that owns the type, its variants given by a `kind:` option; `fit.plot()` stays each result type's default view, the one definition rule 6 of section 8.1 allows. Release 1.0 has `fit.plot()` and `fit.coef_plot()` only (step 30 of 0.7.0, which states the convention in their help pages). `residual_plot`, `prediction_plot` with `predict(se: true)` and `influence_plot` are planned for 1.2 with basic econometrics; the ROC figure is `fit.curve(type: roc).plot()` (#112), with no `roc_plot`. The rows of the models table of #113 written as `fit.plot(type: …)` are renamed to this convention in the documentation step of 0.8.0.

### 121. Diagrams: one layout engine, positions in the script for export, SEM path diagrams and the model builder

Raised on 11 November 2026 by the owner's proposal on diagrams (`docs/proposals/zeff_diagrams.md`, decisions D2 and D4 to D10; D1 is #119 and D3 is part of #118). Causal graphs (`g.plot()`, 0.10.0), flowcharts, SEM path diagrams and pipeline graphs are all nodes and edges, and the proposal draws them with one engine.

| # | Proposal | Release under #113 | Recommendation |
| --- | --- | --- | --- |
| D4 | Zeff's own deterministic layout engine in Rust, in `plot`: layered (Sugiyama) layout with network-simplex layers, median-heuristic crossing reduction with a fixed number of sweeps, Brandes–Köpf coordinates, spline or orthogonal routing; grid placement for templates; ties broken by declaration order, so the same diagram gives the same bytes everywhere. Graphviz (EPL-1.0) and ELK (EPL-2.0) are not linked; a crate such as `layout-rs` only if its licence is permissive | 0.10.0, for `g.plot()` | Adopt; the algorithms from their papers if no crate qualifies |
| D2 | Positions: automatic layout by default; dragging in the viewer is view state; **Add to script** writes `layout:` (a grid of names) or `at:` (coordinates) into the call; exports use only positions the script holds, and the viewer says so when dragged positions differ. This corrects the current text, under which `g.plot()`'s positions live in `.zemp/views.json`, which a replication package leaves out | 0.10.0 | Adopt, amending section 7.3 of the Language Reference and sections 6 and 7 of the Zemp Specification; `direction:`, `layout:`, `at:`, `same_rank:` and `spacing:` on `g.plot()` |
| D10 | One canvas component in Zemp for causal graphs, diagrams and SEM | Zemp 0.4, with the causal-graph builder | Adopt |
| D9 | `zeff graph` (the script's dependency graph from the typed IR, stale steps marked) and Zemp's Pipeline view | 1.1 | Adopt |
| D5 | `reason: Str` on `where`, `drop_missing`, `distinct` and `merge`, naming the box of a sample flow | 1.2, with `sample_flow` (the deadline of 0.2.0 has passed; the option is additive) | Adopt |
| — | The `diagram { }` block with ISO 5807 shapes, moderation arrows onto arrows, groups, Mermaid and DOT export; `frame.sample_flow()` and `fit.sample_flow()` from the counts the engine already keeps; the Diagram builder in Zemp Studio | 1.2 | Adopt; the reserved word is #119 |
| D6 | SEM diagrams as `m.plot()` on a `Model` and `fit.path_plot()` on SEM and CFA results, in RAM notation, with the options and templates of the proposal (sections 6.3 and 6.4), the viewer interactions of section 7 in both editions and the model builder of section 8 in Zemp Studio | 1.12, with CFA and SEM; ESEM display at 1.25; the growth template with `latent_growth` at 1.25 | Adopt |
| D7 | The fixed-parameter syntax of model blocks, an open decision of section 15 of the Engine and Project Specification, which the builder needs | 1.12 | Settle with the SEM release |
| D8 | Identification checks at check time: free parameters against moments (the t-rule), a scale for every latent variable, the three-indicator rule, degrees of freedom shown live, as new Z05xx errors and W05xx warnings | 1.12 | Adopt |
| — | CONSORT diagrams from `randomize`; PRISMA from a screening frame; mediation diagrams with `mediate` | 1.17 (design and meta-analysis); `mediate` at 1.12 | Adopt |

*Recommendation.* Adopt as the table says. Deadline: 0.10.0 for D2, D4 and D10, which `g.plot()` needs; the rest with their releases.

**The owner's decision (11 November 2026).** "Option A." Every row of the table is adopted at the release it names. D4: the layout engine is Zeff's own, in the `plot` package, deterministic, with no Graphviz or ELK linked; a layout crate is used only under a permissive licence, recorded in `docs/decisions.md`. D2 corrects the specification the same day: `g.plot()` gains `direction:`, `layout:`, `at:`, `same_rank:` and `spacing:` at 0.10.0 (section 7.3 of the Language Reference and the catalogue row), and dragged positions are view state that **Add to script** turns into `layout:` or `at:`, an export using only the positions the script holds (sections 5 and 6 of the Zemp Specification); logged in `docs/spec_changes.md`. D10: one canvas component in Zemp, built with the causal-graph builder at Zemp 0.4. The items from 1.1 onward enter the catalogue with the other rows of #122 in the documentation step of 0.8.0; D7, the syntax of fixed parameters, stays an open question of section 15 of the Engine and Project Specification until the SEM release, 1.12.

### 122. Placing the graphics and diagram proposals on the releases of #113

Raised on 11 November 2026 by the owner's proposals on graphics (`docs/proposals/zeff_graphics_capabilities.md`, sections 5 to 13 and 15) and diagrams (`docs/proposals/zeff_diagrams.md`, section 9). Their release tables follow the roadmap before #113, which the owner has since replaced: their "1.2" is the old difference-in-differences release with the plot layers, "1.3" the old latent-variable release, "After 1.11" the end of the old roadmap. Under #113, release 1.2 is basic econometrics and keeps the old 1.2's plot layers; SEM is 1.12; the design methods are 1.17.

**Placement by the rules of #113.**

| Proposal items | Release |
| --- | --- |
| The grammar extensions of sections 5.1 to 5.10 for everyday figures: `linetype:` and `alpha:`, paths, segments, rectangles, polygons, interval styles, label boxes and repelled labels; positions with `dodge:`; scale transforms, label formats and break counts; more palettes and greyscale; facet `wrap:`, free space, margins and context; annotation shapes, highlighting, callouts, panel tags, alt text; typed theme overrides; composition with nesting, collected guides, tags and insets; export at a chosen DPI, TIFF, rasterised layers and `with_data:` | 1.2, beside the plot layers already there; if 1.2 proves too large, the composition and export items move to 1.5 (P8, now within #113's basic phase) |
| Data shorthands of section 6 at the basic level: `violin`, `ecdf`, `hexbin`, `dotchart`, `dumbbell`, `likert`, `scatter_matrix`, ordered `heatmap` | 1.5 |
| Distribution and relationship layers beyond the basics: `ridges`, `beeswarm`, `halfeye`, `raincloud`, `density2d`, `ellipse`, `qq` bands, bounded and weighted densities, `.curve()` | 1.16 (advanced multivariate methods) |
| Result plots of section 7 | With their method's release in #113: the `regress` diagnostics and `binscatter` at 1.2 with the four candidates of #120; margins and moderation plots at 1.6; count, survival and time-series plots at 1.7 and 1.10; multilevel plots at 1.11; PCA, cluster and multivariate plots at 1.3, 1.5 and 1.16; SEM, IRT and latent-class plots at 1.12, 1.13 and 1.26; learning plots at 1.4 and 1.14; design-based and causal plots at 1.9, 1.17 to 1.20; meta-analysis plots at 1.17; Bayesian plots at 1.17 |
| Zemp viewer features of section 8 | With the Zemp releases the proposal names: brushing, zoom, legend toggling and copy from the 0.2 alpha; linked views, the colour-vision check and the plot as a table at 1.1 |
| The later families and the items marked "After 1.11": dot plots, hulls, nested facets, magnifiers, slopegraphs, waffles, treemaps, mosaics, alluvials, UpSet, calendars, ternary and perspective coordinates, EPS, TikZ, interactive HTML (P12), animation as GIF and animated SVG without MP4 (P13), 3D surfaces, maps and networks | The research phase's "other" release (1.28) or later, each planned with the family it belongs to; maps with the `spatial` package |
| Left out (section 13): pie and donut shorthands, 3D bars and scatter, dual axes on unrelated data, radar charts, rainbow palettes, broken axes, Venn beyond three sets, word clouds, images as points, chart-junk themes, basemap tiles, stars without effect sizes, bitmap-only output | Not planned |

**The proposal's own decisions not covered by #118 to #121.** P7: significance brackets `.brackets()`, always printing adjusted p-values and effect sizes, at 1.2. P9: the names with two homes (`violin`, `ecdf`, `ridges`, `beeswarm`, `raincloud`, `qq`) added to rule 6 of section 8.1 when each arrives. P10: project font files in themes, hashed as inputs, at 1.2. P11: the scene exported as a versioned `.zplot` file, at 1.2.

| Option | For | Against |
| --- | --- | --- |
| A. Adopt the placement above; the catalogue gains the rows in the documentation step of 0.8.0 (#113), each release's contents confirmed when it is planned | One plan for every figure, consistent with the owner's release order; nothing before 1.0 beyond #118 to #121 | About 200 roadmap rows to add, and 1.2 becomes a heavy release |
| B. Adopt the proposals' own release numbers | As written | They follow a roadmap the owner has replaced |
| C. Keep the proposals as reference only, placing items one by one as each release is planned | Less to decide now | The plan drifts; the names are not settled together |

*Recommendation.* A. Deadline: 0.8.0, with the documentation step of #113.

**The owner's decision (11 November 2026).** "Option A." The placement in the table is adopted, with P7, P9, P10 and P11 as stated. The documentation step of 0.8.0 that amends the catalogue, section 14 and the Zemp release table for #113 also adds the rows of the two proposals, `docs/proposals/zeff_graphics_capabilities.md` and `docs/proposals/zeff_diagrams.md`, at these releases, with the result plots named as #120 decides. Each release's contents are confirmed when that release is planned; if release 1.2 proves too large then, composition and the further export formats move to 1.5. The items of section 13 of the graphics proposal are not planned. Nothing in either proposal enters a release before 1.0 beyond what #118 to #121 decided.

### 123. No third-party packages before 2.0.0

Raised on 11 November 2026 by the owner, after a review of the package plan of section 8.4 against the principles of section 1.2. The owner does not want Zeff's methods spread over many overlapping packages of uneven design, as R's are. The plan before this entry placed third-party packages at 1.11, which #113 moved to 1.15, and #8 recommended Zeff-source packages earlier, at 1.6.

**The owner's decision (11 November 2026).** "No third party package before release 2.0.0." No package of either kind of section 8.4, Zeff-source or component, ships before 2.0.0, and neither do the registry, the validated badge or the commands `zeff add`, `zeff update`, `zeff remove`, `zeff packages`, `zeff vendor` and `zeff publish`. Release 2.0.0 is a major release under the semantic versioning of section 15. The plan of #113 runs to 1.29 and places no 2.0.0, so the release that ships packages is set when 2.0.0 is planned; if 2.0.0 follows 1.29, that is about seven years after 1.0. No options were put: the owner stated the decision.

**What stays as it is.**

- A project's own functions in `functions/`, brought in with `use functions::<name>` (tier 4 of section 8.1), are unaffected. They are how a researcher writes a method of their own before 2.0.0.
- The format of section 8.4, rule 9 of section 8.1 and point 6 of section 8.3 stay in the specification as the plan, so the built-in packages are still built to the same interfaces and a project's `zeff.toml` needs no change when packages arrive. Until then a `[packages]` table in `zeff.toml` is refused (Z0713), and `use` of a package that is not built in is refused (Z0304).
- User-defined types stay at 1.15, where #113 places them; they serve a project's own functions as well as packages.

**What changes.**

- #8 is settled: no Zeff-source packages before 2.0.0, so neither its option A (1.6) nor its option C (1.1) is taken.
- #9, the registry's host, and #10, `synthesize` as a registry package, wait until 2.0.0 is planned.
- Amended on the same day and logged in `docs/spec_changes.md`: in the Engine Specification, section 1.3, the project files and command table of section 2, section 8.1 (the tier diagram, rule 9 and tier 4), point 6 of section 8.3, section 8.4 and the list of section 15; the glossary of the Language Reference (section 16); sections 3, 4, 5 and 13 of the Zemp Specification; and risk R10 of the risk register.
- Left to the documentation step of 0.8.0 that amends section 14 and the Zemp release table for #113: both tables are written without third-party packages, so release 1.15 holds user-defined types but no packages. The same step corrects the engine's messages, with their snapshot tests: the label of Z0304 for `use` of a package says "arrives with Zeff 1.11", already out of date since #113, and the help of Z0304 and Z0713 says "after 1.0"; all three are to name 2.0.

**Left open, to settle when 2.0.0 is planned.** Raised in the same review, on how packages keep Zeff's principles once they arrive:

1. Whether a package's names must follow the rule of #97 and the naming conventions of section 3 of the Language Reference, refused at `zeff publish` rather than warned.
2. Whether the shared arguments of principle 8 (`by:`, `cluster:`, `weights:`, `se:`, `fe:`, `re:`) must mean in a package function exactly what they mean in a built-in.
3. Whether a package function must report dropped rows, skipped missing values and coercions with counts, as principle 6 requires of every built-in, and how the registry checks it.
4. Whether the registry refuses or flags a function that duplicates a built-in or another package's method; whether a reviewed set of packages carries a second badge beside *validated*; and whether a widely used validated package is taken into the built-in catalogue in a later release.

### 124. The report document: a Typst file with a settings file, simple chunks, dialogs, and a Quarto-like default

Raised on 11 and 12 November 2026 by the owner, after reviewing a mockup of a Typst report in Zemp Studio (kept outside the repository). Decision #73 recommends Typst as the engine of every PDF and the report document as a `.typ` file with executed Zeff chunks; its example of that file opened with `#import` and `#show` lines and wrote chunks and inline values with the word `zeff` each time. The owner found this hard for researchers who do not write code, and asked for: Typst writing as simple as Quarto's, without Quarto itself; the settings in a separate file beside the document, as `refs.bib` is; chunks as simple as Quarto's; a complete dialog, with tabs, for the settings, fonts included; a table builder for complex tables; and a default document that looks like Quarto's default PDF in font, size and spacing. The owner declined an Insert menu and dialog for values.

**Option A, the design reviewed.**

1. *Files.* A report is `report/<name>.typ`, holding only the writing: headings, text, chunks and values, with no `#import` or `#show` line, since Zeff applies the template. Its settings are in `report/<name>.yml` beside it; Typst reads YAML natively, so the pair stays standard. References are in `refs.bib`, holding the user's own entries only. The scripts named in the settings under `uses:` run first, and the document uses their results by name, never repeating the analysis.
2. *Chunks and values.* With `chunks: language: zeff` in the settings (the default), a plain ```` ``` ```` block is a Zeff chunk, its options written as `//|` lines (`label:`, `caption:`, `echo:`, `notes:`), as Quarto writes `#|`. An inline value is `` `= m2.coef(educ)` ``. Both are Typst's own syntax for code, so the file remains valid Typst; cross-references (`@tbl-main`) and citations (`@card1995`) are Typst's.
3. *The default template, Article,* looks like Quarto's default PDF (KOMA-Script article class, page layout DIV=11):
   - New Computer Modern, the font Typst ships, close to LaTeX's Latin Modern, at 11 pt with single spacing;
   - bold sans-serif title and unnumbered headings;
   - no first-line indent, with a small space between paragraphs;
   - table captions "Table 1: …" centred above tables, figure captions below figures;
   - Chicago author-date citations, page numbers at the bottom centre, US Letter paper.
   
   Further templates: APA 7, an economics journal, a thesis chapter, and a template file of the project.
4. *Document settings* (Zemp Studio), a dialog of nine tabs that writes the settings file, showing it as it will be written and keeping only what differs from the template:
   - *Document:* title, subtitle, running head, authors with affiliation, email, ORCID and the corresponding author, date, abstract, keywords and language.
   - *Template and page:* the template, paper, margins, columns, line spacing, indent, page numbers, title page, and line numbers and anonymity for review.
   - *Fonts:* body, headings, mathematics and code fonts, size, plots in the body font, and a project font file hashed like any input, each font with its licence and a live sample.
   - *Numbers:* decimals, p-value style, leading zero, thousands separator, decimal mark and the minus sign.
   - *Tables:* what sits under each estimate (standard error, interval, t or p), stars (off by default), notes from the fits, style, decimal alignment, column numbers, caption position and text size.
   - *Figures:* width, shape, theme, greyscale, raster resolution, interval statements and alt text.
   - *References:* bibliography files, citation style, method citations, software citation and DOI links.
   - *Outputs:* PDF, Word, HTML and slides, PDF/A, tagged PDF, slide shape and the output folder.
   - *Analysis:* the scripts run first, chunk defaults, parameters and the seed.
5. *The table builder* (Zemp Studio), five tabs:
   - *Columns:* results, headings, group headings, order.
   - *Rows:* the terms with their labels, each shown as an estimate, a Yes/No row or hidden, and a row for absorbed fixed effects.
   - *Statistics* at the foot.
   - *Layout:* under-estimate, stars and decimals, each able to follow the document; column numbers, dependent-variable line, Word, LaTeX and CSV exports.
   - *Notes and caption:* caption, label, the note written from the fits and the user's own words.
   
   It writes one `let tab = report(...)` statement into a script and one chunk naming the table into the document, and opens again on a double-click of the table in the preview or from the Tables tab of Document settings. The `report()` of 1.0 has `stats:`, `digits:`, `stars:` and `labels:`. The builder needs further options, all additions:
   - `columns:`, `groups:`, `keep:`, `controls:`, `effects:`, `under:` and `note:`;
   - the statistics `adj_r_squared`, `n_clusters`, `first_stage_f` and `mean_y`.
6. *The enhancements:*
   1. numbers in sentences come from the results;
   2. table and figure notes are written from the fits (standard errors, clusters, instruments, scaling, stars, dropped rows, the run record); on by default, switchable;
   3. the sources of the methods run are cited automatically, with Zeff and its run record; on by default, switchable;
   4. plots are set in the document's font;
   5. rich text in labels uses Typst markup (`rich("ln(_w_)")`, `$R^2$`), which settles the syntax of the `RichText` type #119 reserves;
   6. `fit.equation()` gives the estimated equation;
   7. the templates;
   8. slides from the same report (PDF, 16:9 or 4:3);
   9. clicking a number or a table in the preview opens the source at its line.
7. *Outputs.* PDF set by Typst, identical bytes with the fonts embedded; Word written by Zeff from the same document, which needs a writer for the subset of Typst a report uses (headings, paragraphs, emphasis, tables, figures, equations, references), the cost of this option; HTML after.

| Option | For | Against |
| --- | --- | --- |
| A. The design above | The writing reads like plain text; the settings and tables need no code; one engine for PDF; every capability of #73 kept and the nine enhancements added | Zeff writes Word from Typst content itself; the dialogs are Zemp Studio work; seven new `report()` options |
| B. A Quarto-style Markdown file (`.qmd`) rendered by Zeff, with a Jupyter kernel so Quarto can also run Zeff chunks | Familiar to R users; Word from Markdown is simpler | The owner declined Quarto; a second document format beside Typst |
| C. #73 as written: chunks and `#import`/`#show` lines inside the `.typ` file | Nothing to add | Hard for researchers who do not write code |

*Recommendation.* A, with the report document's release as #73 settles it (the 0.8.0 slice proves the chunk model; the document at Zemp 1.11 under section 15, or earlier by a logged change). `fit.equation()`, the new `report()` options and `RichText` follow at 1.2, with basic econometrics (#113). Two settings for the owner: US Letter as Quarto's default paper or A4; stars off by default. Deadline: 0.8.0, with #73.

**The owner's decision (12 November 2026).** "A except table builder." Option A is adopted for items 1 to 4, 6 and 7: the `.typ` document with its settings file beside it, plain ```` ``` ```` chunks and `` `= value` `` inline values, the Article template as the default with the other templates, the nine-tab Document settings dialog, the nine enhancements and the outputs. Item 5, the table builder, is not adopted. A table is a `report()` statement in a script, used in the document by name, and its look across the document is set in the Tables tab of Document settings; the Tables tab has no button for a builder, and a double-click on a table in the preview opens the `report()` statement it comes from, as for any other result. The seven `report()` options and four statistics proposed for the builder are not adopted with it; any of them may be proposed on its own later. The two settings stand as written in option A: US Letter, as Quarto's default, and stars off. With this entry the syntax of the `RichText` type that #119 reserves is Typst markup, and `fit.equation()` is planned for 1.2. The report-document part of #73 is read with this entry; #73 itself stays open for its deadline at 0.8.0, and the Zemp Specification, section 8, gains the design when the report document's release is planned, logged in `docs/spec_changes.md`.

*Amended by #141 (13 November 2026).* The report document is written in Markdown, in a `.zmd` file, not in Typst; Typst stays the PDF typesetter underneath. Its settings, templates, enhancements and dialog carry over, and HTML of Quarto's quality and HTML slides join its outputs.

### 125. Zemp Lite only, with Zemp Studio later

Raised on 12 November 2026 by the owner: "Can I go with Zemp Lite only and start working on Zemp Studio after 2.0?" The Zemp Specification builds two editions from one codebase (section 2). Zemp Lite is code-first and leaves out the modules that write Zeff for the user: the Methods panel, dialogs, guides, builders and question-first menus. Zemp Studio has them, and is the edition planned as paid. Section 15 ships Zemp Lite 1.0 with Zeff 1.0.0 and Zemp Studio 1.0 once its dialogs cover every 1.0 method, then adds dialogs and guides at each release. Lite's parts (workspace, editor with the language server, viewers, plot renderer) all exist by Zeff 0.7.0 and 0.8.0; the dialogs are the slowest part to build.

**For and against Lite only.** For:

- the work of every release goes to the engine and its methods, with no dialogs, guides or builders to add at each of the 28 releases from 1.1 to 1.29;
- the questions that only Studio raises wait: the menu plan, the dialogs, the causal-graph and SEM model builders, and Studio's licence (#25).

Against:

- researchers who will not write code, a stated audience (section 1), have nothing until Studio exists; #123 puts 2.0.0 about seven years after 1.0 if it follows 1.29;
- the edition planned as paid does not exist, so revenue comes from services or from paid features of Lite, which #39 leaves for after 1.0;
- features adopted as Studio's wait: the Document settings dialog of #124 and the diagram builders of #121.

**Question 1. When Zemp Studio starts.**

| Option | For | Against |
| --- | --- | --- |
| A. After Zeff 2.0.0 | The clearest focus; Studio arrives with packages (#123) and a stable 1.x catalogue | The longest wait for non-coders and for revenue; fixed now for about seven years |
| B. Lite only for now, with the start of Studio decided at a review at the close of 1.5, the end of #113's basic phase, which may confirm 2.0.0 | By then Zeff has its basic methods and Lite its first users, so the decision rests on evidence; nothing is lost if the review keeps A | One more decision later |

In both, the one-codebase design of section 2 is kept, with the shared modules never depending on Studio's, so Studio can start from the same code at any time.

**Question 2. The repository.** No Zemp repository exists yet; Zeff's is public under Apache 2.0.

- *Recommended:* Zemp is created as a private repository under the owner's account from its first commit, holding the whole codebase. Zemp Lite is free of charge but not open source, at least until 1.0. Starting private keeps every choice open: opening later takes one setting, while code once published cannot be withdrawn. Every Zemp analysis is Zeff code, re-run by the open engine, so replication is unaffected.
- Section 16 of the Zemp Specification, which calls Lite "the free, open part", and #25, which recommends Apache 2.0 for all of Zemp, are amended with this question.
- The browser edition's code is downloaded by every visitor, so its licence protects it more than secrecy does.
- GitHub's free plan gives a private repository 2,000 build minutes a month, macOS minutes counting ten times and Windows twice, so the desktop builds may need a paid plan or self-hosted builders when they begin.
- The alternative is a public repository for Lite, open to contributors from the start, with any Studio code kept private later.

**Question 3. What of Studio's comes into Lite.** The Document settings dialog of #124 writes the settings file of a report, not Zeff code, so it does not break Lite's rule that nothing writes a statement for the user. Recommended: it is part of Zemp Lite. Nothing else of Studio's moves.

**Kept on file for when Studio starts: the menu plan of 12 November 2026.** Not decided here.

- Menus list one entry per function, about 220 dialogs against 705 rows of #113's table, with a function's methods and variants as choices inside its dialog and research-grade choices under "More methods".
- The method menus are modules: Statistics always on; Econometrics, Causal inference, Psychometrics, Multivariate, Machine learning, Bayesian, Meta-analysis and Operations switched on in View › Modules or by a flavour chosen at first start ("Zemp for Econometrics"), all builds of the one Studio.
- One Results menu holds every after-estimation step and shows only what applies to the selected result.
- The research goals of section 11 remain a second way to browse the Methods panel.
- Separate programs per discipline, or per research question, were considered and not recommended: they share one engine, a mixed project would need two of them, and each adds an installer, a release train and documentation.

*Recommendation.* Question 1, B; question 2, the private repository; question 3, the settings dialog in Lite. On adoption, section 15 of the Zemp Specification becomes a release plan for Lite, Studio's rows move to "later (#125)", and sections 2, 6 and 16 say Studio is deferred, all logged in `docs/spec_changes.md`. Deadline: Zeff 0.8.0, with Zemp's 0.2 alpha; question 2 before Zemp's first commit.

**The owner's decision (12 November 2026).** "Yes, as recommended." Question 1, B: Zemp Lite only for now, the start of Zemp Studio decided at a review at the close of Zeff 1.5, which may confirm 2.0.0. Question 2: Zemp in a private repository under the owner's account from its first commit, holding the whole codebase; Zemp Lite free of charge and not open source at least until 1.0; Zemp Studio proprietary; this settles #25. Question 3: the Document settings dialog of a report (#124) is part of Zemp Lite, and nothing else of Studio's moves. The Zemp Specification was amended the same day: the opening paragraph, section 2 (the rule "Lite first" and the Free row), section 6 (Studio deferred, the menu plan to be weighed when it starts), section 15 (a release table for Lite alone, the Studio dialogs, builders and checks of the 0.2 alpha, 0.3, 0.4 and later rows moved to one row for Zemp Studio), section 16 (the repository and the licences) and section 17 (the licence item decided), logged in `docs/spec_changes.md`. The review at the close of 1.5 is added to the checklist of that release when it is planned.

### 126. How the text of an exported SVG figure is written: as outlines, or as text with the font embedded

Raised on 12 November 2026 at the opening of step 21 of release 0.7.0, the SVG writer. Section 7.6 of the Language Reference says the renderer is part of the engine "with its own fonts embedded (IBM Plex Sans and Mono, under the Open Font License) and never the system's, so a figure's SVG and PDF bytes are identical on every machine". A PDF carries its fonts inside the file and every reader of PDF honours them. An SVG can carry a font too, as data in its style sheet, but not every program honours it: browsers do; Word, PowerPoint and Inkscape ignore it and look for a font of that name on the computer, and when IBM Plex is not installed they set the labels in another font, whose letters have other widths, so a label measured to fit no longer fits. The bytes of the file are identical on every machine either way; what differs is how the figure looks where it is opened.

| Option | For | Against |
| --- | --- | --- |
| A. Outlines: every letter is drawn as the shape the font gives it; the words are also kept in the file as its title and description, and as a hidden text layer for search | The figure looks the same in every browser, in Word, PowerPoint, Inkscape and Illustrator, with nothing installed; this is the aim the sentence of section 7.6 states; the default of matplotlib (`svg.fonttype: path`); the smallest file, since each letter's shape is written once and reused; no font subsetting to build | A label cannot be retyped in a drawing program, only moved or replaced; the sentence "with its own fonts embedded" is then true of PDF and, for SVG, of the shapes of the letters |
| B. Text with the font embedded: labels are text elements, and the part of the font they use is carried in the file | Labels can be selected, searched and retyped in a browser or a drawing program; the literal reading of "fonts embedded" | Exact only where the embedded font is honoured (browsers); in Word, PowerPoint and Inkscape the labels change font and may overlap unless IBM Plex is installed on that computer; needs a subsetter of the font, a further engine step; a larger file |
| C. A, with B as an option of `export` (`text: outlines \| editable = outlines`) | One default that is exact everywhere, and the other for those who edit figures by hand, as open decision #97 prefers an option to a second function | The option and the subsetter are more work; under the rule that nothing is added before 1.0 unless necessary, the option waits |

*Recommendation.* C in two parts: A at step 21c of 0.7.0, as the one way an SVG is written in 1.0, and the option `text: editable` after 1.0, an addition that breaks no script. On adoption the sentence of section 7.6 is amended to say that an SVG draws its text as outlines of the embedded fonts and keeps the words for search, logged in `docs/spec_changes.md`. If the owner prefers B, the font subsetter becomes a step before 21c. Deadline: 0.7.0, before step 21c; steps 21a and 21b do not depend on it.

**The owner's decision (12 November 2026).** "Yes, outlines." Option A is the one way an SVG is written in 1.0: every letter is drawn as the outline the embedded font gives it, and the words of each text are kept in the file, as the label of its group and as text that is not painted, so that a search finds them and a reader of the screen says them. The option `text: editable` of `export`, with a subsetter of the font, is an addition for after 1.0 and is not scheduled. Section 7.6 of the Language Reference is amended at step 21c, logged in `docs/spec_changes.md`.

### 127. A name bound to a number, where an option takes a number or a variable

Raised on 12 November 2026 at step 23e1 of release 0.7.0, when `hline` and `vline` were built. Their `at:` is catalogued as `Float | Var`, and section 7.6 of the Language Reference says that they "take a value or a variable". `size:` of `plot` and of `points` is `Var | Float` in the same way. Section 3.7, "Resolution by parameter type", says what a bare name means for a parameter of type `Var` (a column) and for "any other type" (a loop variable or parameter, then a script binding), and says nothing of a parameter that takes either. The engine reads a bare name there as a variable, always. So

```
let centre = 0.5
d.plot(x: dose, y: change).points().hline(at: centre)
```

is refused: "no variable `centre` in `d`" (Z0101). The line is drawn by `at: centre + 0`, since an expression that is not a bare name is computed as a number. A reference line at a number computed earlier in the script, a mean, a threshold, an estimate, is the commonest use of the mark, so the question is what a bare name should mean there.

What the reference tools do. In R with ggplot2, `geom_hline(yintercept = centre)` reads `centre` from the workspace, and a variable of the data is named inside `aes()`: the two are written differently. In Stata, `yline()` takes numbers, and a stored value is written as a macro, `yline(`centre')`.

| Option | For | Against |
| --- | --- | --- |
| A. As built: a bare name is a variable; a bound number is written as an expression | One rule, already in three functions; nothing to build | The commonest use meets an error whose message does not point to the way out; `+ 0` is a trick, not a notation |
| B. The order of name lookup that section 3.7 gives for a frame method: a variable of the frame the layer draws; where it has none of that name, a script binding whose value the option takes; where both exist the variable, with W0102 | What the reader expects, and what section 3.7 already says of names in a frame method; no script that runs today changes its meaning, since a name that is not a variable is refused today | The checker does not always know the variables of the frame before the run, so the choice is made when the statement runs; a mistyped variable that happens to be the name of a binding draws the binding, and says nothing |
| C. A notation for "the value of this binding" in such a place | Nothing is guessed | New syntax, under the freeze, for one kind of option |

*Recommendation.* B, built as an engine step of its own right after step 23 (step 23f), for `at:` of `hline` and `vline` and for `size:` of `plot` and `points`, with the row of section 3.7's table for a parameter that takes a variable or a value written at that step and logged in `docs/spec_changes.md`. Until then `hline` and `vline` follow A and their help pages say how a bound number is written. Deadline: 0.7.0, before step 24.

**The decision (12 November 2026).** Option B, taken on the recommendation under the owner's standing instruction of that day ("If at any point any decisions to be taken by me, I will go by your recommended decisions"), which he may overturn. It is built at step 23f of release 0.7.0.

### 128. Which variables make an axis of categories

Raised on 12 November 2026 when step 24 of release 0.7.0 was opened. `bars` stands its bars side by side at the values of `x`, and a bar chart by industry needs an axis whose places are the industries. Until now `x:` and `y:` take a Bool, an Int or a Float, on a linear scale, and a Str or a Category is refused ("the scales of categories, text and dates are not in this release yet"). Section 7.6 of the Language Reference says nothing of which variables are categories on a position. For a colour, a shape and a type of line the engine already has a rule (step 20c): a Category, a Str, a Bool and a number with value labels are shown as categories, a number without value labels on a continuous scale. The question is whether a position follows that rule.

What speaks against the same rule. A layer is placed on the axis of its plot, and the layers share one scale. A variable coded 0 and 1, a Bool or an Int with the value labels "no" and "yes", is the outcome of every binary model, and the figure that shows such a model is the outcome as points with the fitted probability as a line over them, on one axis from 0 to 1: `w.plot(x: educ, y: union).points(jitter: 0.1).lines(y: fitted)`. On an axis of two categories the line has no place. Stata's users keep most of their categories as labelled numbers, so this is the usual case there and not a rare one.

What the reference tools do. In R with ggplot2 a number is continuous on an axis, labelled or not, and a factor or a character vector is discrete, at the places 1, 2, 3 with 0.6 of a place beyond each end; a number is made discrete by `factor()`. Stata's `graph bar, over()` takes a variable of either kind as categories and shows its value labels; `twoway` treats every variable as a number and shows value labels on the axis with `xlabel(, valuelabel)`.

| Option | For | Against |
| --- | --- | --- |
| A. A Str and a Category make an axis of categories; a Bool, an Int and a Float a linear axis, with value labels or without | ggplot2's rule; nothing drawn today changes; the outcome with its fitted curve keeps its axis | Bars by a labelled number stand at 0 and 1 on an axis that reads 0, 0.5, 1; a position and a colour follow different rules |
| B. The rule of colour: a Category, a Str, a Bool and a number with value labels make an axis of categories | One rule for every aesthetic; bars by a labelled number read "no" and "yes" | The outcome with its fitted curve cannot be drawn on one axis; every scatter drawn today against a Bool or a labelled number changes its axis |
| C. A, and a linear axis of a variable with value labels has its breaks at the labelled values, written as their labels | The outcome with its curve, and bars that read "no" and "yes", both; what Stata's `valuelabel` does | A linear axis that reads as words is a third thing to explain; the breaks are part of the scales, which are step 25 |

*Recommendation.* C in two parts: the axis of categories for a Str and a Category at step 24a, where `bars` needs it, with the rule A; and the breaks of a labelled variable on a linear axis at step 25, with the scales. A variable is made categories on an axis by `category()`, as it is for a shape. Deadline: 0.7.0, before step 24a.

**The decision (12 November 2026).** Option C, taken on the recommendation under the owner's standing instruction of that day ("If at any point any decisions to be taken by me, I will go by your recommended decisions"), which he may overturn: A is built at step 24a and the labelled breaks at step 25.

### 129. How an axis of dates chooses its breaks and writes them

Raised on 12 November 2026 when step 25c1 of release 0.7.0 built the axis of dates. Section 7.6 of the Language Reference names `date` among the kinds of scale and says no more: not where the breaks of an axis of dates fall, nor how they are written. Section 3.8 has dates printed in tables in the ISO 8601 form, `2024-03-05`, and section 7.6 has the numbers of an axis "format as in tables".

What the reference tools do. In R with ggplot2 4.0.3 a Date is placed by its count of days and a date-time by its seconds; the breaks are chosen by the range, and the labels by the step, in English: `Feb`, `Mar` for months within a year (the year appears nowhere on the axis), `2020`, `2021` for years, `Mar 05` for days. Stata's `tsline` writes `01jan2024` in the display format of the variable. Observable Plot writes the part that changes and adds the year under the first break of each year, on a second line.

| Option | For | Against |
| --- | --- | --- |
| A. The ISO form of tables, cut to the part the breaks differ in: `2024` where every break starts a year, `2024-03` where every break starts a month, `2024-03-05` for days, the time of the clock as `14:30`, with the day before it where the breaks fall on more than one day | One form in tables and on axes; no language in the figure; the year is always on the axis where the breaks are days or months; nothing to learn | Longer than `Mar`, so every second label is written where five would touch; `2024-03` is less familiar in print than `Mar 2024` |
| B. Month names in English, as ggplot2 writes them: `Feb`, `Mar 05`, `2024` | Shorter; what many journals print | English in every figure; the year is lost from an axis of months or days within one year; a second form of date to explain |
| C. Two lines, the part that changes above and the year under the first break of each year | Short and complete | A second line of labels under the axis is a change to the layout and its theme, frozen at step 22 |

*Recommendation.* A as the default, with `labels:` of `scale_x` and `scale_y` taking a pattern at step 25c2, so that `labels: "%b %Y"` writes `Mar 2024` where a journal asks for it. The places of the breaks are a matter of engineering and are in `docs/decisions.md`: the start of a year (or of every second, fifth, tenth year), of a half-year, a quarter or a month, a Monday, a day, or a round time of the clock, whichever is nearest to a fifth of the range. Deadline: 0.7.0, at step 25c1.

**The decision (12 November 2026).** Option A, taken on the recommendation under the owner's standing instruction of that day ("If at any point any decisions to be taken by me, I will go by your recommended decisions"), which he may overturn; the pattern for `labels:` follows at step 25c2.

### 130. The notes of a plot built in a chain: call by call, or for the plot the statement ends with

Raised on 12 November 2026 at step 25d of release 0.7.0. Each function of a plot raises the notes that are new to the plot it gives back: `points` counts the rows it does not draw, a layer coloured by twelve categories says that the eight colours repeat. A later call of the same chain can undo the cause. In `w.plot(x: educ, y: wage, size: gap).points().scale(size: [2, 12])` the layer notes "1,447 rows with gap below 0 dropped", and `scale` then reads the sizes from the smallest value to the largest and draws those rows; in `...points().scale(color: viridis)` the note of the colours that repeat is printed although every category has its colour in the plot that results. The note was true of the plot as it stood after `points` and is not true of the plot the statement ends with. Written in the other order, `...scale(size: [2, 12]).points()`, no such note is raised.

| Option | For | Against |
| --- | --- | --- |
| A. As built: the notes are raised call by call; the page of `scale` says to write it before the layers | Simple, and what every step since step 20 does; each note is true of some plot of the chain | A note under a figure can contradict the figure; the order of a chain changes what is printed, though not what is drawn |
| B. The notes of a statement are those of the plot it ends with: a later call withdraws a note of an earlier call of the same statement that no longer holds | What is printed describes what is drawn, whatever the order | The messages of a statement are a list the interpreter keeps positions in (for pragmas, for the codes it checks after a call), so a call cannot take a message out of it today; that plumbing has to change first, with care for `attempt` and for loops |

*Recommendation.* A for 0.7.0, where the remedy is one sentence of the help page, and B at 0.8.0, in a step of its own that first gives the interpreter a way to withdraw a message of the statement. It was tried at step 25d and taken back: removing the note shifted the positions the interpreter holds and stopped the run. Deadline: 0.8.0.

**The decision (12 November 2026).** As recommended, taken under the owner's standing instruction of that day, which he may overturn: A stands for 0.7.0, and B is planned for 0.8.0.

*The second half (15 November 2026).* Option B is built at step 8a of `docs/releases/0.8.0.md`: the interpreter gains a way to withdraw a message of the statement, and the notes of a plot chain are those of the plot the statement ends with.

### 131. The titles of the legends of size, shape and fill

Raised on 12 November 2026 at step 26a of release 0.7.0, where `labels` was built. The catalogue gives it six arguments, `labels(title:, subtitle:, x:, y:, color:, caption:)`: the two axes and the legend of colour can be titled in the words of the script, and no other legend. A plot that sizes its marks by one variable and colours them by another has two legends, and the second keeps the label of its variable; so does a legend of shapes or of fills by a variable of its own. The label of the variable is often a good title, and where it is not the only remedy today is to change the label in the frame.

| Option | For | Against |
| --- | --- | --- |
| A. As the catalogue has it: `color:` alone, which also titles the legend of fills where nothing is mapped to `color` | Nothing to add; most plots have one legend | A legend of sizes or of shapes cannot be titled from the plot |
| B. Add `fill:`, `size:` and `shape:` to `labels`, each the title of the legend of that aesthetic | Every legend can be titled where the figure is made; what ggplot2's `labs()` takes | Three more arguments in the usage line, which is a change to the catalogue |

*Recommendation.* B, at 0.8.0: an addition that breaks no script. Until then A, as built. Deadline: 0.8.0.

**The decision (12 November 2026).** As recommended, taken under the owner's standing instruction of that day ("If at any point any decisions to be taken by me, I will go by your recommended decisions"), which he may overturn: A stands for 0.7.0, and the three arguments are planned for 0.8.0.

*The second half (15 November 2026).* Option B is built at step 8a of `docs/releases/0.8.0.md`: `labels` gains `fill:`, `size:` and `shape:`, its catalogue row amended at the step and logged.

### 132. Polar coordinates: where the circle starts, which way it turns, and which variable is the angle

Raised on 13 November 2026 at step 27c1 of release 0.7.0, where polar coordinates were built. The catalogue gives `coord` two arguments, `coord(type: cartesian | flip | polar | fixed, ratio:)`, and the Language Reference says `coord(flip | polar | fixed)` and no more. So `coord(polar)` has one form: `x` is the angle, which starts at the top of the circle and turns clockwise, once round over the limits of `x`; `y` is the distance from the centre. ggplot2's `coord_polar()` takes three settings more: `theta = "y"`, which makes `y` the angle (a stacked bar becomes a pie); `start =`, the angle of the first value; and `direction =`, clockwise or not. None can be written in Zeff today.

| Option | For | Against |
| --- | --- | --- |
| A. As the catalogue has it: `x` is the angle, from the top, clockwise | Nothing to add; it draws the rose of bars, the series round a clock or a year, and the rings of tiles; the top and clockwise is how a clock and a compass are read | No pie: a share of a whole as an angle needs `y` as the angle |
| B. Add `theta: x \| y = x`, `start:` in degrees and `direction: clockwise \| anticlockwise` to `coord` | Everything `coord_polar()` draws; a pie is `bars(position: stack)` with `coord(polar, theta: y)` | Three more arguments in the usage line, a change to the catalogue under the freeze; a pie is the chart the Language Reference's defaults were written to discourage |

*Recommendation.* A for 0.7.0, as built. B at 0.8.0 if the owner wants pies and wind roses that start elsewhere: an addition that breaks no script. Deadline: 0.8.0.

**The decision (13 November 2026).** As recommended, taken under the owner's standing instruction of 12 November 2026 ("If at any point any decisions to be taken by me, I will go by your recommended decisions"), which he may overturn: A stands for 0.7.0, and the three arguments are a question for 0.8.0.

*The second half (15 November 2026, at the opening of release 0.8.0).* The three arguments are not built before 1.0, under the owner's instruction of 11 November 2026 that nothing is added before 1.0 unless necessary; they are an addition that breaks no script and are placed with the grammar extensions of release 1.2 under #122.

### 133. `by:` on the shorthand plots

Raised on 13 November 2026 at step 29 of release 0.7.0, where the shorthands are built. The specification's sample script writes `wages.histogram(lwage, bins: 40, by: female)`. The catalogue's usage line for the shorthand `histogram` (row 229) has no `by:`, and no other shorthand has one either; the Language Reference says only that `histogram(x)` is `plot(x:).histogram()`. So the sample and the catalogue disagree on `histogram`, and what `by:` means in a plot is nowhere said. Everywhere else in Zeff `by:` names the variable whose values part the rows into groups (`summary(by:)`, `aggregate(by:)`), and in the grammar the same word parts a plot into panels (`facet(by:)`).

| Option | For | Against |
| --- | --- | --- |
| A. Every shorthand that draws rows takes `by: Var \| Vars`, and it parts the plot into panels as `facet(by:)` does: `histogram`, `scatter`, `line`, `bar`, `boxplot`, `density`, `qq_plot`; not `heatmap`, which draws correlations and not rows | One rule for the family, the one `facet(by:)` already has; the sample script runs as written; Stata's `by()` works on every graph in the same way, and R's users write `facet_wrap` after any plot; the panels are built, so it costs one argument each | Seven usage lines of the catalogue gain an argument under the freeze |
| B. `by:` on `histogram` alone, as the sample has it | The smallest change that makes the sample run | The owner learns `by:` on one plot and finds it missing on the next; a second round of the same change later |
| C. `by:` fills by the variable in one panel, as `color:` does | Nothing | Two words for one thing, and stacked or overlaid histograms of groups are the hardest to read; `color:` is there for it on the shorthands that take it |

*Recommendation.* A. Deadline: 0.7.0, step 29a.

**The decision (13 November 2026).** As recommended, taken under the owner's standing instruction of 12 November 2026 ("If at any point any decisions to be taken by me, I will go by your recommended decisions"), which he may overturn: A. The usage lines of catalogue rows 229 to 235 gain `by: Var | Vars` as each shorthand is built at steps 29a to 29d, each logged in `docs/spec_changes.md`; the sentence on shorthands in section 7.6 of the Language Reference gains the rule at step 29a.

### 134. Making a manipulated analysis harder to hide in a replication package

Raised on 13 November 2026 by the owner: "what if the researcher manipulates code inside? Results can be the same." `zeff verify` (section 2.1 of the Engine and Project Specification) proves that a replication package reproduces its own run record: every statement's result, every output file and every warning. It does not show that the analysis is the one the researcher planned, that the data are the data collected, or how many specifications were tried before the one reported. A package built from an analysis with awkward observations dropped, fifty specifications tried and the best kept, or a data file edited by hand verifies as *identical*. Reproducibility and honesty are different properties, and no program can guarantee the second; what a program can do is make dishonesty harder to hide and honesty easy to show.

What Zeff records already: the scripts themselves, in which every filter and every dropped row is a readable statement, with a note printed for each row dropped; the hash of every file a run reads, with a warning when one changed since the last run; the seed of every random draw and every use of `today()` and `now()` (W0703); every silenced warning, with the reason its pragma must give; and every `external` step, listed on the package's first page as not verifiable. What it cannot show today: data fabricated or edited before Zeff read them; runs made on another machine, or run records deleted; and an analysis plan changed after the results were seen.

Six measures, each an addition:

1. *A stamp of the plan.* A command, proposed `zeff plan`, writes the content hash of the project's scripts and functions before any run has read the project's data, and refuses once a run record shows a read. The researcher registers the hash with an outside registry (OSF, AsPredicted, the AEA RCT Registry), which dates it; Zeff sends nothing itself (#116). `zeff verify --plan <hash>` reports that the package's scripts match the registered plan, or shows every change since, line by line. Limit: a researcher can look at the data outside Zeff first.
2. *The run history.* `zeff export replication --history` adds every run record of `.zeff/runs/` (result hashes, the statements that changed, warnings; never data), and the package's first page counts the runs and the distinct specifications of each model. Each run record carries the hash of the record before it, so a record deleted from the middle of the history leaves a visible gap. Off by default, since a working history is the researcher's own; a journal may require it. Limit: runs on another machine leave no trace.
3. *The raw data's provenance.* A data source in `zeff.toml` may declare where it was deposited (a DOI, ICPSR or openICPSR, Dataverse, Zenodo, a statistics office) and the checksum the archive publishes; `verify` compares each raw file with it, and the first page traces every analysis file back to a declared raw source and flags any data file that no script produced from one, such as a file edited by hand. Limit: data a researcher collected have no outside checksum until they are deposited.
4. *Signed and dated run records.* The author signs the package's run record with a key of their own (ed25519 in the minisign format, as the engine's archives are signed, section 2.1 of the Engine and Project Specification), with an optional trusted timestamp (RFC 3161); `verify` reports the signer, the date and whether anything changed after signing. Limit: a signature proves who and when, not that the work is honest.
5. *Specification curves.* Planned in the `meta` package at the research level (#113): a result shown across every reasonable specification rather than one. Nothing new is needed.
6. *The Git commit of each run.* Added the same day, after the owner asked whether keeping the project in Git would be enough. Git alone is not: the researcher controls the history, which can be amended, reordered or rewritten and whose dates are set by the researcher's own computer; work between commits leaves no trace; the `.gitignore` that `zeff new` writes leaves out `data/` and the run records of `.zeff/runs/` (section 2 of the Engine and Project Specification), so Git shows how the code changed but not what the data were or what each run gave; and a repository can be started after the analysis is finished. Git is strong evidence when it is joined to the record of the results. Each run record names the Git commit the project's scripts and functions were at, and whether any of them had uncommitted changes; the package's first page says that every result came from one named commit with no uncommitted changes, or lists the results that did not; the stamp of measure 1 may be that commit's hash, registered before the data are read; and a project pushed regularly to an outside host gains dates the researcher cannot set. Limit: a project not kept in Git records no commit, and the first page says so.

| Option | For | Against |
| --- | --- | --- |
| A. The fields at 0.11.0, the commands after 1.0: the versioned run-record schema of 0.11.0 gains the fields measures 1 to 4 and 6 need (the previous record's hash, a plan hash, declared source checksums, a signature block, the Git commit and whether it had uncommitted changes), left empty until used; the commands follow as additions in 1.1 or the first later release with room; measure 5 stays where #113 places it | Keeps the owner's rule that nothing is added before 1.0 unless necessary; the run-record format never has to break after 1.0; a package written by 1.0 already carries a chained history | The commands wait until after 1.0 |
| B. All four commands at 0.11.0, with the replication packages | Zeff 1.0 ships with them, a claim no other statistics program can make | Four commands and their tests in a release that already carries the formats, the caches and the packages; additions before 1.0 |
| C. After 1.0 only, with no fields now | Nothing to do now | A later change of the run-record format needs a new schema version, and packages written before it cannot show a chained history |
| D. None: integrity left to journals and registries | Zeff stays a tool for computing | Misses what a referee will ask first, and the one place where the record that makes honesty visible is made |

*Recommendation.* A. The owner's choices are the name of the stamp command (proposed `zeff plan`) and whether the run history is in a package by default (proposed: off, with a switch a journal may require). The fields, the hash chain and the signature format are engineering, settled when 0.11.0 is planned. Deadline: 0.11.0.

### 135. A function that gives citation details: `cite`

Raised on 13 November 2026 by the owner: "a citation function for Zeff that will provide citation details". No function of the catalogue gives one. The pieces exist or are planned: every help page lists the references of its method (section 9.1 of the Engine and Project Specification), every shipped dataset carries a citation on its page (section 8.5), and from 0.10.0 every release has a DOI and a citation file (#116, layer 3). What is missing is a way to ask for them from a script, in a form a paper can use, and a way to gather everything one analysis used into one bibliography. R answers the first with `citation()` and `citation("pkg")`; Stata with `cite` notes in its help files; neither cites the methods a particular fit used.

The proposal, one function with an option in the manner of #97:

| Call | Gives |
| --- | --- |
| `cite()` | Zeff itself at the engine version the project pins: title, version, year, DOI and address, with a sentence for a paper's software statement |
| `fit.cite()` | The references of exactly the methods and options the fit used, and Zeff's own: a `reg` with `se: robust` cites MacKinnon and White (1985), with `cluster:` Cameron and Miller (2015), with `fe:` Correia (2016) and Gaure (2013) |
| `cite(t_test)` | The references of a function's help page |
| `cite(wages)` | A dataset's citation, from its page |
| `format: text \| bibtex \| ris = text` | Author–date text; BibTeX for LaTeX and Typst; RIS for Zotero and EndNote |

The value is a `Table` of references, which prints and exports as any table does: `fit.cite().export("output/references.bib")`. A command, `zeff cite`, writes one bibliography of everything the last run used, read from its run record: the engine version, every method's references (each fit's options included) and every dataset; the report document of #124 cites its methods from it, which is that entry's third enhancement.

Engineering it needs, settled when the release is planned: the references of the help pages, today free text, kept as structured entries (authors, year, title, venue, pages, DOI) in one file per package, from which each page's References section is generated, so the two cannot disagree; a link from each option to the references it rests on, declared beside the function's help; and the package that holds `cite`, under the tier rules of section 8.1. The structured references fit the revision of the help pages under #103, at 0.8.0 to 0.10.0.

| Option | For | Against |
| --- | --- | --- |
| A. `cite` and its formats at 0.10.0, with the citation file and the DOI; `zeff cite` at 0.11.0, with the run records it reads | The references are made structured once, while the help pages are revised anyway; ships before `core` freezes at 0.11.0; a paper written with 1.0 cites its software and its methods in one line | One function, three formats and a command before 1.0, against the owner's rule that nothing is added before 1.0 unless necessary |
| B. After 1.0, as additions in 1.1 or the first release with room | Keeps 1.0 as planned | The help pages' references are revised twice; papers written with 1.0 cite by hand |
| C. The citation file and the DOI only, as #116 plans, with no function | Nothing new | The methods a fit used are never gathered, and a researcher copies references from help pages |

*Recommendation.* A. The name is the owner's choice, made with this entry. Deadline: 0.10.0.

**The owner's ruling on the name (13 November 2026).** "cite is better": the function is `cite`, and the command `zeff cite`.

**The owner's decision (13 November 2026).** "Option A". What follows:

- `cite()`, `fit.cite()`, `cite(function)` and `cite(dataset)`, with `format: text | bibtex | ris = text`, are built at 0.10.0, beside the citation file and the DOI of #116; the help pages' references become structured entries, one file per package, in the revision of the pages under #103, complete at 0.10.0, and each page's References section is generated from them.
- `zeff cite` is built at 0.11.0, when the versioned run records it reads arrive.
- The catalogue row of `cite`, the command's row in section 2.1 of the Engine and Project Specification, the rows of section 14 for 0.10.0 and 0.11.0, and the count of functions in 1.0 are amended when 0.10.0 is planned, and logged in `docs/spec_changes.md` then; the package that holds `cite` is chosen under the tier rules of section 8.1 at the same time.

*Settled by #141 (13 November 2026).* The package that holds `cite` is `publish`, created at 0.10.0.

### 136. `vif` in Zeff: what the language lacks to print it as it prints now

Raised on 13 November 2026 at step 30b of release 0.7.0. Open decision #71 (decided 6 November 2026) has `design()` built at step 30 and "`vif` returns to Zeff in the same step". `design()` is built (step 30a), and a script computes the factors from it: with the columns of `m.design()` read by name, each column's part along the others taken out by projection, and the centred sum of squares over what is left, the wages model `log(wage) ~ educ + exper + exper^2 + female` gives 1.118, 14.101, 14.008 and 1.001, the factors `vif` prints. So the arithmetic can be written in Zeff. Three things stand between that and the function as it is:

1. *The lines under the table.* `vif` names the terms omitted as collinear, and says when there is no regressor beside the intercept, in lines under its grid. `table(title:, columns:)` (#72) builds a grid and a title and nothing under them.
2. *The title.* `vif` is titled `vif · m`, by the name the fit is bound to, or by its formula. Nothing in the result interface gives a Zeff body the name or the formula of a fit; `name_of` is of a variable.
3. *The cost.* The Rust function reads the Gram matrix the fit keeps and makes no pass over the data. From the design a Zeff body makes of the order of k² passes for each of k columns by projection, or k² in all by a QR and the inverse of its triangle written out in Zeff: for 50 columns on a million rows, seconds at best and minutes at worst, where there were none.

| Option | For | Against |
| --- | --- | --- |
| A. `vif` stays in Rust at 0.7.0, its numbers and its print unchanged; a test computes the factors in Zeff from `design()` and holds them equal to it, so the purpose of #71, the design within reach of a script, is shown to hold; the rewrite waits for a ruling on what it needs | Nothing printed changes; no function is added to `core` before 1.0; no model becomes slower | The catalogue's `layer` for row 248 stays Rust, against the letter of #71 |
| B. Add what Zeff lacks, then rewrite: `table(..., notes: List[Str])` for lines under a grid, and a way to title a table by a result (its bound name, or its formula) | `vif` in Zeff printing as now; every tabulating function written in Zeff gains the lines under its grid | Two additions to `core` under the freeze and before 1.0; the cost of point 3 remains |
| C. Rewrite now with what there is: a table titled `vif`, and the omitted terms said in a `note` | No addition | The printed layout of a shipped function changes; the cost of point 3 |

*Recommendation.* A for 0.7.0. B is the question for 0.8.0, with the other diagnostics over the design (partial residual and added-variable plots) that #71 names: if Zeff-written tables are to carry lines under their grids, that is decided once for all of them, and `vif` follows. Deadline: 0.8.0.

**The decision (13 November 2026).** As recommended, taken under the owner's standing instruction of 12 November 2026 ("If at any point any decisions to be taken by me, I will go by your recommended decisions"), which he may overturn: A. It sets aside, for this release, the clause of #71 that returns `vif` to Zeff at step 30, and so it is named to him in the announcement of the step. The note of catalogue row 248 says why the layer is Rust still, logged in `docs/spec_changes.md`.

*The second half (15 November 2026, at the opening of release 0.8.0).* `vif` stays in Rust, and the question of Zeff-written tables carrying lines under their grids (option B) is not taken before 1.0, under the owner's instruction of 11 November 2026; it returns with the diagnostics over the design at release 1.2 of #113's plan. The `layer` of catalogue row 248 stays Rust.

### 137. The layout `zeff fmt` writes where section 3.5 is silent

Raised on 13 November 2026 at step 31 of release 0.7.0, where `zeff fmt` is built. Section 3.5 of the Language Reference gives the style in eight rows and one example: four spaces, lines of at most 100 characters, a chain that does not fit broken one `.method()` to a line, `name: value`, long calls broken after a comma, the spacing of the operators, model blocks with their operators aligned. A formatter must also decide what the table does not say, and it decides for every script anyone writes, since it has no settings and Zemp runs it on save. The specification's own sample script (`zeff_samples.zeff`) is written in one consistent hand, and shows an answer to each question; the recommendation in each row is that answer.

| # | Question | What the sample script does | Recommendation |
| --- | --- | --- | --- |
| 1 | A call too long for its line: how are its arguments laid out? | The first argument stays beside the opening parenthesis and each of the others has a line of its own, starting under the first (`import("data/wages.csv",` then `na_values: ...` under the quotation mark) | The same. Where even that does not fit in 100 characters, the arguments start on the next line, one to a line, four spaces in, with the closing parenthesis on a line of its own |
| 2 | A comment at the end of a line of code: where does it start? | Most start at column 55, whatever the code beside them, so that they read as a column down the page | Column 55, or two spaces after the code where the code reaches further. A fixed column does not move when a neighbouring line changes, so a change to one line is a change to one line |
| 3 | Blank lines between statements | One between related statements, two before each numbered section | Kept as written, two at most; none at the start or the end of a block, one at the end of the file |
| 4 | The arms of a `case` written one to a line | The arrows `=>` are aligned | Aligned |
| 5 | A chain: when is it broken? | Whenever it does not fit, every `.method()` on its own line, four spaces in; a chain of one method keeps to its line and its call is broken as in 1 | The same |
| 6 | A line that cannot be made to fit (a long string, a long name) | Left long | Left long: a string is never split |
| 7 | The specification's sample script itself, a normative file | It is a script of the repository | Formatted like every other; what changes is spacing alone, logged in `docs/spec_changes.md` as a change of form |

| Option | For | Against |
| --- | --- | --- |
| A. The seven recommendations, which make the sample script the standard | The style is the one the specification already shows; the sample changes least | Arguments aligned under a parenthesis move when the name before it changes length, which a block indent would not |
| B. As A, but a broken call always puts its arguments four spaces in, one to a line (question 1) | A change of a name never moves the lines under it; no deep indents | Not the hand of the sample: nearly every broken call in it would be rewritten, and so the text of sections 3 and 6 that quotes them |

*Recommendation.* A. Deadline: 0.7.0, step 31a.

**The decision (13 November 2026).** As recommended, taken under the owner's standing instruction of 12 November 2026 ("If at any point any decisions to be taken by me, I will go by your recommended decisions"), which he may overturn: A. The rules live in one place in the formatter, so a ruling for B, or another answer to any one question, is a change of that place and a second run of `zeff fmt` over the repository.

**Found while building (13 November 2026, step 31a).** Writing the formatter raised five points that the table above does not settle. Each is taken on the recommendation under the same standing instruction, and is the owner's to overturn.

| # | Question | What the formatter does | Why |
| --- | --- | --- | --- |
| 8 | Lines the author broke: are they ever joined? | No. What stands on one line stays on one line while it fits in 100 characters; what the author spread over several lines (the arguments of a call, the members of a list, a chain, the operands of a long sum) stays spread, laid out in the one style. The formatter breaks a line that is too long and never joins two | A list of twelve variables written three to a line, or a call written one argument to a line, was written so on purpose; a formatter that packed them again would undo the author at every save. It also follows that the example of section 3.5 had to be lengthened: its line fits in 100 characters once spaced, and so is left on one line (`docs/spec_changes.md`) |
| 9 | A chain too long for its line that has one method only | One method on a name stays on the line of the name and its arguments are broken (`wages.add(` and the arguments under the first), as in the sample script. One method on a call starts a new line (`frame(...)` and under it `.to_wide(...)`), and so does any method after something that took several lines to write | The arguments of the method would otherwise hang under a parenthesis far to the right of the page |
| 10 | A comment at the end of a line of code that reaches column 100 | The 100 characters are of the code. A comment is never moved to another line, cut or re-wrapped to make room, so a line of code with its comment may be longer than 100 | A comment is the author's text; where to break it is not for a formatter to say |
| 11 | Which line a comment at the end of a line belongs to, when the formatter lays the code out afresh | The line that holds the piece of code the comment was written after. A pragma (`// zeff: allow(...)`) at the end of a line is the exception: it stays at the end of the first line of its statement, however the statement is laid out | Section 11.1 ties such a pragma to the statement that begins on its line. Were it carried to a later line it would allow nothing, or be an error (Z0317) |
| 12 | What the formatter does when it cannot vouch for its own result | It checks every result before giving it back: the formatted script must parse to the same tree, with the same documentation lines, the same comments in the same order and each pragma on the same statement, and formatting it again must change nothing. A result that fails any of these is not written; the script is left as it is and an internal error is reported | A formatter runs on save, on every script; one that could change the meaning of a script, or lose a comment, would be worse than none |

**Found while formatting the repository (13 November 2026, step 31b).** Reading what the formatter made of the repository's own scripts showed two layouts to put right, taken as the others were and the owner's to overturn.

| # | Question | What the formatter does | Why |
| --- | --- | --- | --- |
| 13 | An `if` with an `else` that is too long for one line | Every branch is written over several lines: `if c {`, the statements, `} else {`, the statements, `}`. A branch is never left on the line of its brace while another is opened | The first writing left `if c { a long expression } else {` on one line and `g2` alone under it, which reads as if the two branches were of different kinds |
| 14 | A list, or the arguments of a call, that the author began on a new line after the opening bracket | It keeps that form: each member on a line of its own, four spaces in, a comma after the last, the closing bracket on its own line. Members begun beside the bracket stay under the first, as in point 1 | It is the form in which section 6 of the Language Reference writes the list of `robustness`, and point 8 already says that a line the author broke is not joined |

Under these rules the specification's sample script changes on 11 of its 292 lines, in spacing alone: nine comments at the ends of lines move to column 55 (or to two spaces after their code), one `by:` aligned by hand moves under the first argument of its call, and the one line whose code is longer than 100 characters (line 246, a call of 105) is broken after its commas. The scripts are rewritten at step 31b, not at this one.

### 138. What `zeff test` runs, and what a failed `assert` does under it

Raised on 13 November 2026 at step 32a of release 0.7.0, where the test run of a project is built. The specification says that `zeff test` "runs every `assert` and each project function's examples written in its `///` help, so a project can check its own numbers on every run" (Engine and Project Specification, sections 2.1 and 12), and that `assert(condition, message)` "stops when a check fails" (Language Reference, section 5.4). It does not say which scripts are run, nor whether a failed check stops the script under `zeff test` as it does under `zeff run`. Four questions follow; the first two decide what a statistician sees when he types the command.

| # | Question | Option A (recommended) | Option B |
| --- | --- | --- | --- |
| 1 | Which scripts does `zeff test` run? | The scripts that call `assert`, anywhere in their text. A script with no `assert` holds no check: it is named as such and not run | Every script of `scripts/`. A script that stops on any error is then a failed test, but an analysis of an hour with no check in it is run each time the tests are |
| 2 | A failed `assert` under `zeff test` | It is kept, with the error it would have stopped the script with, and the script carries on: every check of the script is reported in one run. Under `zeff run` nothing changes, the first failed check stops the script | The script stops at the first failed check, as under `zeff run`; the checks after it are not evaluated until the first is put right |
| 3 | The examples of a function private to its file (a name beginning with `_`) | Not run: no script can call such a function, so no example can | Run inside the module, which needs a second way of running an example |
| 4 | `--deny-warnings` and the examples | The option holds for the scripts. An example is compared with its page, warnings included: a warning the page shows is expected, and one it does not show already fails the example | The option stops an example at its first warning, so a page could not show a warning |

*For A on question 1.* `zeff test` is the command one runs often, before sharing a project or after changing a function. It should cost what the checks cost. A project keeps its checks in a script written for them (`scripts/checks.zeff`) or beside the analysis; either way the script says so by calling `assert`.

*For A on question 2.* An `assert` binds nothing and returns nothing, so the statements after it never depend on it: carrying on is always possible. Three checks that fail are then three lines of one report, not three runs.

*Recommendation.* A on all four. Deadline: 0.7.0, step 32a.

**The decision (13 November 2026).** As recommended, taken under the owner's standing instruction of 12 November 2026 ("If at any point any decisions to be taken by me, I will go by your recommended decisions"), which he may overturn: A on the four questions. Question 1 is one function of the engine (`testing::calls_assert`) and question 2 one option of a run (`collect_checks`), so another ruling on either is a small change.

### 139. How the "Warnings and notes" of a built-in written in Zeff is narrowed

Raised on 13 November 2026 at step 32c of release 0.7.0. Section 9.1 of the Engine and Project Specification has the "Warnings and notes" of a page "generated from the diagnostics the implementation declares, and checked both ways: a code raised but not declared, or declared but never raised, fails the build. For a function written in Zeff, generated from the functions its body calls and from the codes it gives `warn` and `note`". For the fourteen built-ins written in Zeff the generated list is the union of everything the called functions declare, and it is too long: the page of `kurtosis` names fifteen codes, among them the warning of a data root's file that changed (W0702) and the error of binding `Nothing` (Z0206), neither of which its body can raise. The known-issues list has carried this since 0.5.0, with the promise to narrow the lists at this release. The specification does not say by what rule.

| Option | What it is | For | Against |
| --- | --- | --- | --- |
| A. Reasoned exclusions, and a demonstration of what is left | Beside each function written in Zeff, in its package, a short list: each code of a called function that the body cannot raise, with one line saying why (the body gives `frame` one column, so the columns cannot differ in length). The page lists the union less these. A test holds both sides: every code left on a page is raised through that function by a script, and no excluded code is raised by any script of the tests | Sound in both directions, as the section asks of a Rust function; the reasons are read at review; fourteen functions make it a bounded piece of work | A list kept by hand, which a change to a body may make stale: the test catches a code that has become reachable only if some script reaches it |
| B. What the examples and the tests raise | A page lists the codes that some example or test raises through the function, and nothing else | Wholly mechanical | Unsound: a code that no test happens to raise leaves the page, and is then raised at a user's desk, undeclared |
| C. The union as it is | Nothing changes; the entry stays in the known issues | No work, and never an undeclared code | The pages go on naming errors that cannot happen, which a reader cannot tell from those that can |

*Recommendation.* A. A page that under-declares is worse than one that over-declares, which rules out B; and fourteen pages that list impossible errors are a fault of the help that the release promised to mend, which rules out C. Deadline: 0.7.0, step 32d.

**The decision (13 November 2026).** As recommended, taken under the owner's standing instruction of 12 November 2026 ("If at any point any decisions to be taken by me, I will go by your recommended decisions"), which he may overturn: A. The sentence of section 9.1 on a function written in Zeff is completed at step 32d to say so, and logged.

(Later on 13 November 2026: the narrowing is step 32e of the checklist; 32d became the correction of the wrong results of `mad`, `skewness` and `kurtosis`, found while preparing it.)

**Built (13 November 2026, steps 32e and 32f).** As option A, with one thing more than the option said: besides the codes a body cannot raise, a function carries the codes of its own operators and conditions, which the generated list missed (an `if` on `NA` is Z0203; a variable of the frame called as a parameter is Z0106). Where the wording of a called function did not fit the caller, its line is taken off and the function's own put on: the page of `winsorize` says "`at:` is below 0 or above 0.5", not "the script stopped itself with the message given". The fourteen pages now list 49 codes between them, each raised by a script of `crates/zeff-engine/tests/zeff_pages.rs`.

### 140. Inside a function's body, which comes first: the function's own name or the frame's variable?

Raised on 13 November 2026 at step 32d of release 0.7.0, from a wrong result. Section 6 of the Language Reference ("Names inside a frame's methods") has a bare name inside a frame's method be the frame's variable first; a binding of the same name gives way under the warning W0102, and a parameter or a loop variable of the same name is an error (Z0106). In a script this is right: the author sees the frame and names its variables. In the body of a function it is a trap, for built-ins and for a project's functions alike, because the function is written before the frame is known:

```
fn spread(data: Frame, var: Var) -> Float {
    let centre = data.median(var)
    data.add(deviation = abs(var - centre)).median(deviation)
}
```

On a frame that has a variable called `centre`, `var - centre` subtracts that variable and not the median, and the function returns another number under a warning. This is how `mad`, `skewness` and `kurtosis` were wrong from 0.5.0 to 0.6.0 (now corrected by rewriting their bodies, step 32d). And a frame that has a variable called `var` cannot be given to the function at all (Z0106). Section 5.3 says that a function "gives the same result wherever it is called"; with this rule its result depends on the names of the variables of the frame it is given.

| Option | What it is | For | Against |
| --- | --- | --- | --- |
| A. Inside a function's body, the function's own names come first | A name the function binds (a parameter, a `let`, a loop variable) is that binding inside a frame's methods too. A variable of the frame that happens to share the name is reached as `data["name"]`. A name the function does not bind is the frame's variable, as now, so `data.where(year >= 2015)` in a function still names the variable `year`. Scripts are unchanged | A function means the same on every frame; no frame is refused for the names of its variables; the author of a function controls every name in it. It is the ordinary rule of scope | Two rules, one for a script and one for a function body; a function that wanted the variable and bound the same name by accident now gets its binding, silently |
| B. As now, but a `let` that meets a variable is an error too | The warning W0102 becomes an error inside a function body, as Z0106 is for a parameter | Never a wrong number | A function then fails on some frames for a reason its user cannot see or mend; built-ins must be written around every name, as the three now are |
| C. As now | The rule of section 6 everywhere, with the warning | One rule | A function's result can depend on the names of the frame's variables, under a warning that names a binding the user did not write |

*Recommendation.* A. The functions written in Zeff are written by people who cannot see the frame, and the rule of section 5.3 promises that this does not matter. Deadline: 0.7.0, step 38c.

**The decision (13 November 2026).** As recommended, taken under the owner's standing instruction of 12 November 2026 ("If at any point any decisions to be taken by me, I will go by your recommended decisions"), which he may overturn: A. It changes a rule of the language, so it is built as a step of its own (38c), late in the release, where a ruling for B or C before then costs nothing; the wrong results themselves are already corrected without it.

**Built (14 November 2026, step 38c of release 0.7.0).** A binding carries whether it was made in the body of a function; inside a frame's methods such a name is the binding at once, with no Z0106 and no W0102, at the check and at the run. Sections 3.7, 5.3, 6 and 11 of the Language Reference say so (logged in `docs/spec_changes.md`). The pages of `iqr`, `mad`, `skewness` and `kurtosis` no longer list Z0106. One thing it does not reach is entered as #147: a name where a term of a fit is expected.

### 141. A `publish` package: reports written in Markdown, with HTML of Quarto's quality and HTML slides

Raised on 13 November 2026 by the owner, in five requests: that Zeff create HTML as it creates PDF; "It should have quality like quarto"; HTML presentations; "I want using markdown"; and "Can we have a report package?".

Where the plan stands. #73 adopts Typst for exports and the report document. #124 (decided 12 November 2026) makes the report a Typst file with a settings file beside it, plain chunks and inline values, its PDF set by Typst, its Word written by Zeff, HTML "after" and slides as PDF; it declined a Quarto-style Markdown file (its option B). Section 8 of the Zemp Specification has tables exported to HTML after 1.0 and the report document, with HTML, at Zemp 1.11. Typst's own HTML export is experimental (Typst 0.15.1, November 2026: behind a feature flag, no styling written, not for production), so it cannot carry HTML of this quality.

The proposal.

1. *The writing format: Markdown, in the dialect Quarto users know.* A settings block at the top (title, authors, formats, bibliography); Zeff chunks as ```` ```{zeff} ```` with options on `#|` lines (`#| echo: false`); inline values as `` `{zeff} m2.coef(educ)` ``; equations in LaTeX notation (`$\hat\beta = (X'X)^{-1}X'y$`); citations `[@card1995]`; cross-references `@tbl-main` and `@fig-wages`; callouts `::: {.callout-note}`; and a raw Typst block that passes through to the PDF alone, for fine control. Typst stays the PDF typesetter underneath; an author never needs to write it. This amends the writing format of #124; its settings, templates, enhancements and Document settings dialog carry over.
2. *Four outputs from one document.* HTML: an article of Quarto's quality, with a contents sidebar, numbered sections, captions and cross-references, citations in any journal style, callouts, foldable code, tabs, light and dark themes and clean printing, as one self-contained file with its fonts, figures and styling inside, nothing fetched from the internet, and identical bytes on every machine. Slides: HTML slides (a slide for each `##`, step-by-step lists, speaker notes, a presenter view, printing to PDF) and PDF slides through Typst. PDF: set by Typst. Word: written by Zeff, as #124 plans.
3. *The package.* A package `publish` in tier 1, after `plot`, since it needs figures and the result interface and no model package needs it: `render(path, format: html | slides | pdf | docx)` and the command `zeff render`; `report()` moved into it from `results`; and `cite()` of #135, whose package #135 leaves to be chosen, placed in it. `render` and `zeff render` are proposed names, the owner's to change.
4. *The technology,* each piece pinned and recorded in `docs/decisions.md` with its licence when it is built, and confirmed by a spike first: `pulldown-cmark` (MIT) to read the Markdown, with Zeff's chunk, citation, cross-reference and callout rules on top; a LaTeX-to-MathML converter in Rust (MIT), so that browsers draw equations natively with no JavaScript; Hayagriva (MIT or Apache-2.0), Typst's bibliography library, with BibTeX input and the CSL styles, so that citations match the PDF; reveal.js (MIT), the player Quarto uses, inside the file for HTML slides; Typst (Apache-2.0) for PDF. Pandoc, which Quarto is built on, is GPL and is not used (dependency rules).
5. *The file extension,* the owner's choice: `.zmd` (recommended: a Quarto installation would otherwise take the file as its own and fail on the Zeff chunks) or `.qmd`.

What it amends, when its release is planned and logged in `docs/spec_changes.md` then: the writing format of #124 and the report part of #73; the catalogue rows of `report` (its package), of `cite` (its package) and new rows for `render`; the command table of section 2.1 of the Engine and Project Specification; the package list of section 8.1; and section 8 of the Zemp Specification, where HTML of tables and reports is placed.

| Option | For | Against |
| --- | --- | --- |
| A. At 0.10.0, `publish` with `report()` and `cite()` in it, so that the package and its names are settled before the names freeze at 1.0; `render`, the Markdown reader and HTML in the first release after 1.0 with room, then HTML slides, then PDF and Word, the report document's dialogs with Zemp as #124 has them | Names settled before the freeze at no cost; nothing large added before 1.0; HTML, which needs no other program, first | Documents of Quarto's quality wait until after 1.0 |
| B. As A, and `render` to HTML and PDF at 0.11.0, with the formats release | A paper can be written in Zeff at 1.0 | A large new piece in the last release before 1.0, against the rule of adding nothing before 1.0 that is not needed |
| C. Everything with the report document at Zemp 1.11, `report()` staying in `results` | Nothing moves now | `report()` would move after the names freeze, a breaking change; HTML waits longest |

*Recommendation.* A, with `.zmd`. Deadline: 0.8.0, with #73 and #124, whose report part this amends.

**The owner's rulings (13 November 2026).** On the writing format: "I want using markdown": reports are written in Markdown, which amends #124. On the name: "publish, record it": the package is `publish`. The release, the file extension and the names `render` and `zeff render` are open.

**The owner's decision (13 November 2026).** "Yes, A with .zmd". What follows:

- At 0.10.0 the package `publish` is created in tier 1, after `plot`, and `report()` moves into it from `results`; `cite()` of #135, built at the same release, is placed in it. The package and these names are settled before the names freeze at 1.0.
- After 1.0, in the first release with room: the Markdown reader with Zeff's rules, `render` and `zeff render`, and HTML of the quality of item 2; then HTML slides; then PDF through Typst and Word. The dialogs of the report document stay with Zemp as #124 has them.
- A report document is a `.zmd` file.
- `render` and `zeff render` are taken as proposed, under the owner's standing instruction of 12 November 2026, his to overturn.
- The catalogue rows of `report` and `cite`, the package list of section 8.1 and the row of 0.10.0 in section 14 of the Engine and Project Specification are amended when 0.10.0 is planned; the rows of `render`, the command table of section 2.1 and section 8 of the Zemp Specification when the release after 1.0 that builds the documents is planned. Each is logged in `docs/spec_changes.md` then.

### 142. Figures of an HTML document in the light and the dark theme

Raised on 13 November 2026 by the owner, on seeing the sample report of #141 drawn as HTML: "figures should be in png not white background in dark theme". Every figure Zeff exports today is drawn on a white page: the PNG writer of step 22b2 of release 0.7.0 paints its page white, the SVG and PDF writers draw a white rectangle under the figure, and the themes of `theme(name: journal | minimal | slide = journal, base_size:)` (catalogue row of `theme`) set dark ink, light gridlines and nothing for a dark page. An HTML document that follows its reader's light or dark setting (#141, item 2) therefore shows each figure as a white box on a dark page.

The mockup of the sample report shows what is wanted. Its three figures are PNG, each in a light and a dark version, both on no background: in the dark version the ink of the text, the axes and the caption line is light (#E6E2EE) and the gridlines dark (#3A3546), and the colours of the data are unchanged, since the Okabe–Ito colours and viridis read on both grounds. The page shows the version its theme asks for. The mockup's versions were made by changing those three colours in Zeff's own SVG and drawing it in a browser; Zeff can make neither today.

The proposal, in the manner of #97 (one function with an option):

1. *A page with no background.* `export(path, size:, notes:, background: white | none = white)`: with `none` the PNG has a transparent page (an alpha channel, written by the engine's own writer with the same fixed arithmetic), and the SVG and PDF draw no rectangle under the figure. The default leaves every export of today unchanged, byte for byte.
2. *A dark form of each theme.* `theme(name: journal | minimal | slide = journal, base_size:, mode: light | dark = light)`: in `dark` the ink, the axes and the text are light and the gridlines dark, by constants named in one place for the owner's reading as #118 asks of the theme's constants; the scales of colour, the shapes and the 8-point rule are unchanged; `mode: dark` with `background: white` is refused with a message, since the light ink would not read on it.
3. *In an HTML document.* `render` (#141) writes each figure twice, as PNG, light and dark, both on no background, at twice the resolution of the page, and the page shows the one its theme asks for, by the reader's system setting or by the page's switch. PDF and Word documents keep the light figure on a white page. The figures of HTML slides follow the same rule.

| Option | For | Against |
| --- | --- | --- |
| A. The three parts, with the HTML documents of #141 after 1.0 | Figures that belong on the page in both themes; an export of today is unchanged; one option on each of two functions | An HTML document carries each figure twice, about twice the bytes; the PNG writer gains an alpha channel and the identity tests the new bytes |
| B. Part 1 alone: transparent figures in the light theme, and in the dark theme each figure set on a light panel | Half the work | A light panel on a dark page is the white box the owner asked to remove, made smaller |
| C. Nothing: figures on white in both themes | No work | What the owner asked to change |

*Recommendation.* A, built with the HTML documents of #141 (after 1.0, HTML first). `background:` and `mode:` are new options on frozen usages (the catalogue rows of `export` and `theme`), amended and logged in `docs/spec_changes.md` when that release is planned. Deadline: the first release after 1.0 that builds the HTML documents of #141.

**The owner's ruling (13 November 2026).** "figures should be in png": the figures of an HTML document are PNG. The rest is open.

**Decided (13 November 2026): option A.** Asked to settle #142 as recommended, the owner answered "Yes, option A". All three parts are built with the HTML documents of #141, in the first release after 1.0 that builds them: `export(background: white | none = white)`, `theme(mode: light | dark = light)` with the dark constants named in one place for the owner's reading, and `render` writing each figure of an HTML document as a light and a dark PNG on no background. The catalogue rows of `export` and `theme` are amended, and the change logged in `docs/spec_changes.md`, when that release is planned; until then every export is unchanged.

### 143. How a step's memory is estimated before the step starts

Raised on 13 November 2026 at step 33c of release 0.7.0, where the memory budget is built. Section 8.3 of the Engine and Project Specification, point 7, gives each window a memory budget (`memory = "3.8GB"` under `[project]` in `zeff.toml`; `--memory` on the command line) and says: "A step whose estimated memory would exceed the budget stops before it starts, with an error that gives the estimate and suggests the desktop app or a narrower `select`", and "the estimate is a step's peak allocation, not its steady state". It does not say how the estimate is made. The purpose is plain: in the browser a step that asks for more memory than there is does not fail, it kills the page; a refusal with a number is better. An estimate that is too low lets the page die; one that is too high refuses work that would have fitted.

| Option | What it is | For | Against |
| --- | --- | --- | --- |
| A. By rule, for each statement, from what it reads | Before a statement runs, the engine adds what the run holds (each buffer once, however many frames share it) to what the statement may add, by a short list of rules: an `import` by its file (rows from the file's size and its first rows, bytes from the columns as typed); a loop over a range by its list of items; a verb that builds a frame from frames, at most those frames once more; `merge`, `append` and `to_long` by the rows and columns they can make; a fit by its rows times its terms. A statement no rule covers adds what it reads, once | It is what the section says: an estimate, before the step, with a number in the error. The rules are few, written in one place and shown by tests | An estimate by rule is wrong at times, in both directions, and a new function that allocates much needs its rule; the error must say it is an estimate |
| B. Exactly, where memory is asked for | Nothing is estimated. Each time the engine is about to build a column it knows the bytes, and refuses if they would pass the budget | Exact; no rule to keep | The step has started by then, and may be far along; and every place that builds a column has to ask, in every package, for ever. It is also not what the section describes |
| C. After the fact | A statement runs; if the run then holds more than the budget, it stops there | Trivial to build and never wrong about what was used | It is too late: the memory was already taken, which in the browser is the crash the budget exists to prevent |

*Recommendation.* A, with the rules kept few and each leaning high rather than low, since the cost of a low estimate is the crash and the cost of a high one is an error that names the estimate and can be answered with `--memory`. Outside the browser the command line has no budget unless one is set, so nothing changes for a project that sets none. Deadline: 0.7.0, step 33c.

**The decision (13 November 2026).** As recommended, taken under the owner's standing instruction of 12 November 2026 ("If at any point any decisions to be taken by me, I will go by your recommended decisions"), which he may overturn: A. The rules are listed in `docs/decisions.md` when they are built, with what each was checked against.

**Rule 2 amended (14 November 2026, step 38g of release 0.7.0).** A loop over a range no longer lists its items, so the rule counts one item for the loop; a list built by such a loop, `[for i in a..b { ... }]`, is counted at three times its values, which is the most a list holds while it grows by doubling. Recorded in `docs/decisions.md`.

**Built (13 November 2026, step 33c).** Three rules, in `crates/zeff-engine/src/budget.rs`: a file imported counts five times its size on disk (an upper bound that the shipped `wages.csv` comes within a tenth of); a loop over a range written out counts the list of its items; a statement that may build a frame, a list or a result from the frames it reads counts those frames once more. What the run holds is counted with each shared column once. What no rule covers yet is listed in `docs/decisions.md` under the same date.

### 144. Reducing a file larger than memory before it is loaded

Raised on 13 November 2026 by the owner, on asking whether Zeff can handle a terabyte of data. What the specification gives: every frame is held in memory, on the desktop all of the machine's and in the browser the memory budget of section 8.3, point 7, of the Engine and Project Specification (at most 4 GB); from 1.1, lazy chains read only the columns a chain uses from Parquet and skip the row groups a `where` rules out (section 8.3, point 10), but what they read must still fit; from 1.7, `stream(path)` (catalogue row of `stream`) feeds OLS and fixed-effects estimation from a file larger than memory, through the streaming QR decomposition. Nothing reduces such a file. A researcher with a terabyte of records cannot keep the rows of a sample, the columns of a model or the totals of each firm and year without first leaving Zeff for a database tool, whose step then stands outside the run record and the replication package. And `import` parses every format but Parquet and Arrow into the import cache (section 8.7), so a terabyte of CSV would first be copied in full on disk.

The proposal, in the manner of #97 (the frame's own methods on a stream, not a second family of names):

1. *Steps of one pass.* A stream takes the chain steps that read each row once and need memory that does not grow with the rows: `where`, `select`, `add` with expressions of one row, and `aggregate`, `tabulate` and the means, variances and counts of `summary` by groups whose number fits in memory. Each runs row group by row group, in the file's order.
2. *Where a chain ends.* A chain on a stream whose result fits in memory gives an ordinary frame, checked before it runs against the memory budget by the estimate of #143; one whose result would not fit is refused with a message that names `save`. `save(path)` on a stream writes Parquet piece by piece, so a reduced file of any size can be written and read again.
3. *Steps that need every row at once* (`sort`, `merge`, `distinct` on many keys, exact `median` and `quantile`, `to_long` and `to_wide`) are refused on a stream with a message that says what to do instead: aggregate first, or save a reduced file and load it.
4. *The same numbers.* A reducer on a stream gives, bit for bit, what it gives on the same rows in memory: sums already use fixed blocks and compensated summation (section 8.3, point 3), and the stream's blocks are the row groups of 1,048,576 rows that `save` writes (section 8.7), on condition that the reducers in memory use the same block, which the building release checks with an identity test.
5. *CSV and the cache.* A stream reads a delimited file in one pass and writes no import cache; its help page advises saving such a file once as Parquet, which later passes read faster and skip by its row-group statistics.
6. *The record.* A file read by a stream is hashed in the run record as an imported file is, and the data-root rules apply (section 2 of the Engine and Project Specification). The stream behaves the same on the desktop, on the command line and in the browser, where the browser's storage limits what files it can hold.

| Option | For | Against |
| --- | --- | --- |
| A. All six parts with `stream` at 1.7, the release whose subject is large data | The commonest need with large data met inside the run record; the lazy chains of 1.1 already push columns and filters down to the file; no new function names | A second way for every reducer to run, tested against the first |
| B. The same at 1.1, with the lazy chains | Sooner | 1.1 already carries the thread pool, the bootstrap, the lazy chains and the rest of the language server |
| C. Nothing: reduction left to outside tools, and said so on the help page of `stream` | No work | The step that shapes the sample sits outside the run record and the replication package |

*Recommendation.* A. Merging two files larger than memory, and out-of-core estimation beyond the streaming QR of 1.7, stay where section 14.1 puts them. The catalogue row of `stream` and the table of section 6 of the Language Reference are amended, and the change logged in `docs/spec_changes.md`, when 1.7 is planned. Deadline: 1.1, because the lazy chains built then should run row group by row group, so that 1.7 adds steps to them rather than rewrites them.

**Decided (13 November 2026): option A.** Asked to settle #144 as recommended, the owner answered "Yes, option A". The six parts are built with `stream` at 1.7. The lazy chains of 1.1 are designed to run row group by row group, so that 1.7 adds the steps of a stream to them. The catalogue row of `stream` and the table of section 6 of the Language Reference are amended, and the change logged in `docs/spec_changes.md`, when 1.7 is planned.

### 145. Where the missing count and the label of a variable come from, in completion and hover

Raised on 13 November 2026 at the opening of step 35 of release 0.7.0. Open decision #77 (option A, decided 6 November 2026) has completion offer "the columns of that frame at that point, with type, missing count and label", and section 12 of the Engine and Project Specification has hover show "a variable's type and missing count". The type is known before a run: the schema pass of 0.6.0 follows the columns and their types through every step of a chain. The missing count is not: it is a fact of the data, the schema pass reads a file's header and first rows only, and a step such as `where` changes the count in a way no check can foresee. A label is known where it is written (a shipped dataset's page, a `label(...)` step with its text in the script) and otherwise is a fact of the file. The specification does not say where the two come from.

| Option | What it is | For | Against |
| --- | --- | --- | --- |
| A. What the engine knows without running, and what the last run recorded | A shipped dataset's counts and labels are known when the engine is built. A run records, for each `let` that binds a frame, each variable's missing count and label (the columns already keep their count of missing values, so nothing is read again); the next check shows them for that frame, as it already shows the columns of a step checked at run time. Both are carried through the steps of a chain that keep every row and the variable (`select`, `rename`, `sort`, an `add` of other variables), and a label through any step. After a step that changes the rows the count is not shown; the type always is. A label written by `label(...)` with its text in the script is known at once | Nothing is read to answer an editor; nothing shown is a guess; the mechanism is the one section 11 already has for steps checked at run time | Before the first run of a project a file's variables show their types alone; inside a long chain the counts stop at the first `where` |
| B. Count when the schema is read | The schema pass reads the whole file once, counts the missing values of each column and keeps them in the schema cache with the file's size and time | Counts for a file that was never run | The first check after a file changes reads all of it (about 0.7 s for each million rows by the benchmark of step 33e3), in an editor at every change of the file; the check of section 11 was designed to read no data; still nothing after a `where` |
| C. Run the chain to the cursor | The editor's engine runs the statements up to the cursor and counts | Exact everywhere | A model or a long import run at a keystroke; section 12 says the language services never run a model |

*Recommendation.* A. An editor answers at once and says only what is known; the counts appear after the first run, which a researcher makes within minutes of opening a project. Deadline: 0.7.0, step 35d.

**The decision (13 November 2026).** As recommended, taken under the owner's standing instruction of 12 November 2026 ("If at any point any decisions to be taken by me, I will go by your recommended decisions"), which he may overturn: A. What is built is recorded in `docs/decisions.md` at step 35d.

**Built (13 November 2026, step 35d).** A shipped dataset's variables carry their labels and counts of missing values. A run records, for each `let` of the script that binds a frame, each variable's count and label under the hash of the statement's text (a last section, `frames`, of `.zeff/schemas/schemas.json`), when the frame is let go or the run ends and only of the columns the run has gathered, so that recording reads nothing; the next check shows them, each count marked `from last run`. A label written with `label(var, "text")` is known at once. The counts are carried through `sort`, `select`, `drop`, `rename`, `add`, `update`, `label` and `panel` and through no other step; the labels through every step that keeps the variable. The note of 13 November 2026 in `docs/decisions.md` has the details and the tests.

### 146. The look of the website and of Zemp Lite web

Raised on 13 and 14 November 2026 by the owner, while the website of decision #103 and Zemp Lite web (Zemp 0.2 alpha, with Zeff 0.8.0) were drawn as private mockups. Each item below is the owner's ruling, in his words where he gave them; the full settings (every colour, size and behaviour) are kept with the mockups in the owner's design guide, outside the repository, as the website is (held until 1.0).

1. *Brand.* The logo is D4, the hexagon of contour lines, violet `#5B3A9E` with a `#2E1A5E` centre in light and `#9B7FDB` with `#E4DBFA` in dark. The tagline is "A statistics language for empirical and causal research." Buttons are the logo's violet ("purple buttons").
2. *Fonts* ("use best visibility fonts"). Atkinson Hyperlegible Next for text, menus and tables; JetBrains Mono for code; Newsreader for large headings and the tagline only.
3. *One type scale* ("Yes, apply it"): page title 50 px, section heading 26, sub-heading 18, body 16.5, introduction 18, tables 15, captions and notes 14, code blocks 14 and code in a sentence 0.9 of its text; app screens 13.5 and never below 12.5.
4. *On every page* ("every page must have dark/light button", "every page must have logo watermark"): a light/dark button, the choice shared by every page; the D4 watermark, fixed while the page scrolls, behind the content and never over text.
5. *Realistic pages* ("If I click open in browser it must show the page. pages must be realistic"). Every link resolves; "Open in your browser" opens Zemp Lite itself, filling the window, not an address that does not yet exist.
6. *Hosting*, as #27 records: the website at `zeff-lang.dev` on GitHub Pages at 1.0, Zemp Lite web at `lite.zeff-lang.dev` on Cloudflare Pages from its 0.8.0 preview.
7. *Zemp Lite: Panes* ("left pane -> tabbed (project, objects), no right pane. data will be tabbed with zeff script page"; "output will be below the zeff script"). One left panel with the tabs Project (the tree and past runs) and Objects (the objects and the engine's memory); no right panel; the data, the results and the plots open as editor tabs beside the script; the script's output sits in a panel under the script, each result under the line that made it, the panel resizable and folding away.
8. *Zemp Lite: the data and variables viewers* ("data viewer + variable viewer must be modern"). A Data | Variables switch with search. The grid gives each column a type badge, a small histogram and its missing count, keeps the header and the row numbers in place while scrolling, and shows a clicked column's summary (mean, standard deviation, minimum, median, maximum, missing). The variables view lists name, type, label, distribution, mean, standard deviation, range, missing and distinct values, and opens a variable in the grid. Both are read-only, as section 5 has it.
9. *Zemp Lite: code text size* ("a button is needed in the zemp lite coding page for changing syntax font size"). A− and A+ in the top bar, from 12 to 22 px with 14 as the default, changing every part of the code, its output, the console and the help on hover, remembered between visits; the same commands in ⌘K. Larger text on the start and welcome screens ("new project page font are too small").

*What this changes in the specification.* Items 7 and 9 depart from section 5 of the Zemp Specification. In Zemp Lite, that section has the Objects panel take the Methods column on the right, with the console under the editor and the viewers in a bottom panel. It also says that everything beyond the top bar's word controls is reached through ⌘K "so it adds no buttons". Zemp Studio is not affected: its Methods panel keeps its column. Under the specification freeze (#40) the text is not edited now. Section 5 (the Zemp Lite paragraph and the top-bar table) is amended, and logged in `docs/spec_changes.md`, as the first step of Zemp 0.2 alpha, when Zemp Lite web is built. The other items are look and wording, which no normative text fixes. The Zeff side is untouched.

**Decided (14 November 2026): the owner's rulings as above.** Deadline for the amendment: Zemp 0.2 alpha (Zeff 0.8.0).

**What the amendment must also cover (found by the Zemp Lite session on 15 November 2026; entered 20 November 2026).** This entry says that its items other than 7 and 9 are look and wording, which no normative text fixes. Section 5 of the Zemp Specification does fix them: its paragraph "The look" (#115), with its table of tokens, its table of syntax colours and its sentence naming the fonts. The amendment placed at the first step of the Zemp 0.2 alpha therefore covers that paragraph too. The quiet look adopted for the margin editor on 18 November 2026 (#174) keeps everything ruled here and changes how the window is drawn; the design guide's section 9 is brought into line with it by the owner's word.

### 147. Inside a function's body, a parameter where a term of a fit is expected

Raised on 14 November 2026 at step 38c of release 0.7.0, while testing #140. Where a parameter takes a term of a fit (`coef`, `std_errors`, `ci`, `p_values` and the tests on terms), a bare name is that term "whatever else bears the name" (Language Reference, section 3.7, "Resolution by parameter type"). In a script that is right. In the body of a function it means that the name of a parameter cannot stand for the term it holds:

```
fn slope(data: Frame, outcome: Var, on: Var) -> Float {
    data.reg(outcome ~ on).coef(on)
}
```

The formula regresses the variables the two parameters name, since #140 and before it; but `coef(on)` asks for a term called `on`, and the call stops with Z0414 ("`on` is not a term of this fit"), listing the terms. It is an error and never another number. The way round exists and is tested: `coef(name_of(on))`.

| Option | What it is | For | Against |
| --- | --- | --- | --- |
| A. Inside a function's body, a bound name that holds a `Var` or a `Str` is the term it names | `coef(on)` in a function reads the coefficient of the variable `on` holds; a name the function does not bind is the term of that name, as now; scripts are unchanged | One rule with #140: in a function, the function's names come first everywhere | A second place where a function body and a script differ |
| B. As now, and the pages of the term-taking methods show `name_of` | Nothing changes; `coef(name_of(on))` is the documented way | No new rule | A function author meets Z0414 first and must learn the way round |

*Recommendation.* A, with the help line of Z0414 naming `name_of` until it is built. Deadline: 0.8.0, when the help-page contract is amended (#103) and the pages of these methods are reopened; nothing of it is built in 0.7.0.

*Decided 15 November 2026, as recommended (option A).* Built at step 5 of `docs/releases/0.8.0.md`: inside a function's body a bound name that holds a `Var` or a `Str` is the term it names, one rule with #140; the help line of Z0414 no longer needs `name_of`.

### 148. Zemp Lite in a public repository under Apache 2.0; Zemp Studio in a private one

Raised and ruled on 15 November 2026 by the owner, when the folder and the repository for Zemp were about to be created. #125 (question 2, 12 November 2026) had put the whole of Zemp in one private repository from its first commit, with Zemp Lite free of charge but not open source at least until 1.0 and Zemp Studio proprietary; that settled #25, which had recommended Apache 2.0 for all of Zemp.

**The owner's decision (15 November 2026).** "Yes, create the zemp folder and repository. but I want repository public. Zemp lite will be public but zemp studio will be private. A separate repository will be created for zemp studio. current proposed zemp folder may be renamed as zemp-lite." Asked which licence Zemp Lite carries, he chose "Apache 2.0, as Zeff". The ruling was given in the session that created the repository, and is recorded here from that session's account of it. No options were put: the owner stated the decision.

**What it means.**

- Zemp Lite is developed in a public repository of its own, `zemp-lite`, under Apache 2.0, with the licence text and a notice file as Zeff has them. It was created the same day, with the specification copied from this repository at commit 9f2b74b; its copy is refreshed at the start of Zemp 0.1.
- Zemp Studio, when its work starts (the review at the close of Zeff 1.5 of #125, question 1, which stands), gets a separate private repository and a proprietary licence. It builds on Lite's code and adds the modules Lite leaves out: the Methods panel, the dialogs, the guides, the builders and the controls that write a statement.
- Lite never depends on Studio's modules. This is the build check of section 3 of the Zemp Specification, which is kept; what was one codebase built twice by a build flag is Lite's public codebase with Studio's modules on top of it in their own repository.
- Everything else of #125 stands: Lite only for now, the review at the close of 1.5, and the Document settings dialog of a report in Lite.

**What it replaces.** The repository and licence parts of #125, and with them the settlement of #25: Lite is open source, where #125 had it closed at least until 1.0. The sentence of #125 that code once published cannot be withdrawn is the reason the owner weighed; he chose to publish.

**Amended on the same day and logged in `docs/spec_changes.md`:** in the Zemp Specification, the opening paragraph, section 2 (the rows "One shared codebase" and "Free"), section 3 (the row "Editions"), section 15 (the sentence on version numbers), section 16 and the decided list of section 17.


### 149. A warning repeated word for word in the passes of a loop

Raised on 16 November 2026 at step 8o of release 0.8.0. That step makes a note that is raised word for word in several passes of one loop, or for several items of `map`, `filter` or `reduce`, print once with the count: a simulation of a thousand passes that draws a sample in each printed a thousand notes. A warning is left as it was: `for i in 1..1000 { let m = d.mean(x) }` on a variable with a missing value prints `warning[W0201]: 1 row with NA in x was skipped` a thousand times.

| Option | What it gives | What it costs |
| --- | --- | --- |
| A. A warning repeated word for word in the passes of one loop is said once, with the count, as a note is | A simulation's messages can be read; the rule is one for notes and warnings | A warning is seen once where a reader skimming for repetition might take the count for less; stored outputs of loops change |
| B. Every pass, as now | Nothing changes; each pass's warning stands beside its place in the output | A long loop buries its output under its warnings |

**Recommended: A**, with the count always printed, and never across statements: two statements that warn alike keep a warning each. Deadline 0.9.0, so that it is settled before the simulations of the first manual are written.

**The owner's decision (21 November 2026, at the opening of release 0.9.0).** The owner said "go" to the checklist of `docs/releases/0.9.0.md` without changing its answers, so its recommendation is taken under his standing instruction of 12 November 2026, his to overturn. Option A: a warning raised word for word in several passes of one loop, or for several items of `map`, `filter` or `reduce`, is said once with the count of passes, never across statements; built at step 30, listed in the notes as a change of printed output.

**Status (23 November 2026).** Built at step 30 of Zeff 0.9.0, as decided: `for i in 1..1000 { let m = d.mean(x) }` prints `warning[W0201]: 1 row with NA in x was skipped; the same for each of 1,000 passes`. A warning whose words differ between passes is not merged; W1006 keeps its own words; to a pragma and to `--deny-warnings` a counted warning is one. Deadline for the owner to overturn: 1.0.

### 150. The small-sample form of the two-stage standard errors, and Stata's large-sample default

Raised on 16 November 2026 at step 10 of release 0.8.0. The ruling of the release gives two-stage least squares the standard errors of the variance layer unchanged, with the degrees of freedom of `reg`: the classical variance divides the residual sum of squares by N − k, the robust one carries N/(N − k), the clustered one G/(G − 1) · (N − 1)/(N − k), and the table is a t table. R's `ivreg` and fixest do the same, and so does Stata's `ivregress 2sls` with its option `small`. Stata's default, without `small`, divides by N, carries no small-sample factor in the robust form, and prints z statistics: its standard errors are those of Zeff times the square root of (N − k)/N under classical and robust errors, a difference of about one part in a thousand on Card's data (N = 3,010, k = 7) and a visible one in a small sample. The owner's rule of decisions 98 and 99 is that every method R or Stata offers for a procedure is an option in Zeff; his instruction of 11 November 2026 is that nothing is added before 1.0 unless it is necessary, and his decision on question 3 of #99 placed the other alternatives for instrumental variables (LIML, GMM, the conditional likelihood ratio test) after 1.0.

| Option | What it gives | What it costs |
| --- | --- | --- |
| A. One form at 1.0, the small-sample one of `reg`; the page's "Coming from R or Stata" names `ivregress 2sls ..., small` as the Stata command that prints the same figures and says by what factor Stata's default differs; the large-sample form an option after 1.0, an addition that breaks no script | `ivreg` and `reg` agree on what a standard error is; R users and fixest users find their figures; the release stays its row | A Stata user who omits `small` sees standard errors that differ in the third digit until the page is read |
| B. An option on `ivreg` at 0.8.0, in the manner of decision 97 (one function with an option), choosing the small-sample or the large-sample form | Stata's default is reproduced to the digit | A new option on a catalogued signature under the freeze; the same question then arises for every later estimator Stata fits by default with N |

**Recommended: A.** Deadline 0.8.0, at step 13, where the page is written. Taken as recommended under the standing instruction of 12 November 2026 unless the owner says otherwise.


## Raised for the journal article

Raised on 16 November 2026, when the owner's draft of an article on Zeff for the Journal of Statistical Software was checked against the journal's rules for authors and its mission statement (both read on that day at jstatsoft.org). The rules that bear on Zeff: the journal considers only software under the GPL, version 2 or 3, or a licence compatible with it, and says it does not consider software under other licences; submitted software should, if at all possible, also be available from a standard repository such as CRAN, Bioconductor or PyPI, not only from a hosting site such as GitHub; a reader with the technical background should be able to install the software and run the replication material within about an hour on an ordinary computer; manuscripts longer than 30 pages take much longer to review; software in a less common language takes longer to find referees for; the journal seeks comprehensive implementations of broad classes of models or the infrastructure on which such implementations are built, illustrated by a non-trivial case study; full reproducibility is mandatory; and a rejected manuscript cannot be resubmitted. Since there is one attempt, these entries remove the reasons to refuse before they add to the contribution. The licence questions are general information, not legal advice.

### 151. The licence and the journal's rule of GPL compatibility

Zeff is under Apache 2.0 (#21, decided 16 October 2026; section 1.4 of the Engine Specification). The Free Software Foundation counts Apache 2.0 as compatible with version 3 of the GPL but not with version 2, so whether the journal accepts it turns on how the editors read "compatible". The owner holds the copyright in all of Zeff, so a change now costs one commit; each outside contributor makes it harder. Zemp Lite (#148) is separate and is not part of the article.

| Option | What it gives | What it costs |
| --- | --- | --- |
| A. Dual licence, MIT or Apache 2.0 at the user's choice, as the Rust project and most of its crates are licensed; the patent grant of Apache 2.0 kept for those who take it | MIT is compatible with both versions of the GPL, so the doubt is gone; nothing changes for any current user | A second licence text in the repository and in every package's manifest; the notice file and section 1.4 amended |
| B. Apache 2.0 kept, and the editors asked before submission whether it is acceptable | No change | The answer may be no, and the change would then come after outside contributions |
| C. The GPL | Beyond doubt for the journal | Against the permissive rule of section 1.4 and the open-core plan; discourages packages and institutional use |

**Recommended: A**, from 0.9.0, after a short message to the editors confirming that the dual licence meets their rule. The owner's word is asked: this reverses #21.

**Moved to 0.11.0 on 20 November 2026.** The owner's words, after the release of 0.8.0: "I want to skip #151 and #31 now and want to move it to 0.11.0". The decision is his and is due by 0.11.0; the licence stays Apache 2.0 until then. Two things follow from this entry's own text and are kept in view. The change costs one commit only while he holds all the copyright: the repository is public, so if an outside contribution is offered before 0.11.0, this decision is put to him before it is merged. And the article is not submitted to the journal before this is settled (#156, still open, puts the submission at 1.0.0 at the earliest).

### 152. A home in a standard repository: an R package on CRAN and a Python wheel on PyPI

The plan of section 2.1 installs Zeff by an installer script, Homebrew, winget, Scoop and a container image at 1.0.0, and builds a PyPI wheel and conda packages "after 1.0, on request". None of these is a statistical repository of the kind the journal names. An R package on CRAN would also lower the barrier the journal warns of for a less common language: a referee who works in R could run the article's scripts from R.

| Option | What it gives | What it costs |
| --- | --- | --- |
| A. At 1.0.0, an R package `zeff` on CRAN that builds the engine from its Rust source, runs a script or a project and returns frames as data frames and fits as tables of estimates; and a wheel on PyPI that does the same for Python | Both repositories the journal names; a referee in R or Python needs no terminal; the selftest of the engine runs as the package's own check, so the determinism contract is shown under CRAN's compilers | CRAN's rules for packages with Rust code (crates vendored, an offline build, size and build-time limits) to meet; a package to maintain for each engine release; its licence must be GPL-compatible (#151) |
| B. The PyPI wheel only, at 1.0.0 | One repository the journal names, cheaply | The barrier for referees who work in R stays |
| C. The plan as it is | No new work | The journal's "if at all possible" is not met, and a referee may say so |

**Recommended: A.** Deadline 0.10.0, so that the work can be placed in 0.11.0 or 1.0.0 when those releases are planned. Amends the table of section 2.1 when it is built.

### 153. The NIST Statistical Reference Datasets in the validation suite

The draft article promises the accuracy of Zeff on the Statistical Reference Datasets of the National Institute of Standards and Technology, after McCullough's method, as the log relative error. Neither the validation suite nor the testing handbook has these problems. Their certified values were computed independently of any statistical package, so they are the one check that owes nothing to a reference program; the data are works of the United States government and in the public domain.

| Option | What it gives | What it costs |
| --- | --- | --- |
| A. At 0.9.0, the eleven linear-regression problems (Longley and the Wampler problems among them), the nine univariate problems and the eleven analysis-of-variance problems, each as a conformance case giving the log relative error of every certified value, with R's and Stata's beside them in the release notes | The article's accuracy table exists two releases before the full suite is published; a fault on the hard problems is found early | One step of 0.9.0; the testing handbook gains the kind of test |
| B. At 0.10.0, with the validation suite run in full | No change to 0.9.0 | A fault on these problems found later |
| C. Not at all, and the claim taken out of the article | No work | The article loses its most independent evidence |

**Recommended: A.**

**The owner's decision (21 November 2026, at the opening of release 0.9.0).** The owner said "go" to the checklist of `docs/releases/0.9.0.md` without changing its answers, so its recommendation is taken under his standing instruction of 12 November 2026, his to overturn. Option A: the eleven linear-regression, nine univariate and eleven analysis-of-variance problems as conformance cases giving the log relative error of every certified value; the testing handbook gains the kind of test, logged; built at step 36.

**Status (23 November 2026, step 36a of Zeff 0.9.0).** The thirty-one files are in `tests/reference/nist` with their source note, and the eleven linear-regression problems are conformance cases: each problem's data are imported by a script and every certified value is computed through the whole engine and compared, the record stored in `crates/zeff-conformance/tests/snapshots/nist__nist_record.snap`. The least log relative error of the coefficients and of the standard errors, with R 4.6.0's `lm` on the same machine beside it: Norris 12.5 and 14.0 (R 12.7 and 13.7); Pontius 12.4 and 13.6 (12.3 and 13.7); NoInt1 14.7 and 15.0 (14.7 and 14.5); NoInt2 15.0 and 15.0 (the same); Longley 11.7 and 12.7 (13.0 and 14.2); Wampler1 9.1 and 9.5 (9.4 and 9.9); Wampler2 12.8 and 14.3 (13.0 and 14.9); Wampler3 9.3 and 13.5 (9.4 and 13.3); Wampler4 7.8 and 13.6 (7.8 and 13.3); Wampler5 5.8 and 13.6 (5.8 and 13.3); Filip 0, the tenth power dropped as collinear by the rule of section 7.2 (R at its default the same; 7.8 and 7.3 with its tolerance lowered). Two known issues are entered (Filip; Wampler5 at the edge of its class) and one question, #214. The univariate and analysis-of-variance problems are step 36b, the handbook's kind of test step 36c.

**Status (23 November 2026, step 36b of Zeff 0.9.0).** The nine univariate and the eleven analysis-of-variance problems are conformance cases beside the regressions, thirty-one in all, each certified value with its log relative error in the stored record. Univariate: the mean agrees to all 15 digits in the nine; the standard deviation and the first autocorrelation to 13 and more, but the standard deviation of NumAcc3 (9.5) and NumAcc4 (8.3); R gives the same two figures. The autocorrelation has no function at this release and is written out by its definition in the script. Analysis of variance, the least over the seven certified values: SiRstv 13.1; SmLs01 to SmLs03 14.8 to 14.9; AtmWtAg 8.8; SmLs04 to SmLs06 9.2 to 9.3; SmLs07 to SmLs09 3.1 to 3.3, below the class and a known issue: the stored numbers allow 4.2 to 4.4 digits whatever the arithmetic, and R's `oneway.test` gives 3.3 (its `aov` 4.7, 2.7 and 0.2). F and R² are read from the result of `anova`; the sums of squares, which the result prints to three decimals, are taken from the function's own computation, held to the printed F bit for bit. A way to reach the digits the stored numbers allow (the group means taken about a constant) is noted in `docs/decisions.md` and not built. The handbook's kind of test and R's figures for the notes are step 36c.

### 154. Independent checking of the numerical core

A referee of the journal checks correctness partly by reading the code. The reference tests compare Zeff with R's own output, computed by R on every build, so the numbers are not Zeff's; but the cases, the tolerances and the scripts that run them were written with the engine, not apart from it, and no second person has read the Rust of the kernels or the variance layer.

| Option | What it gives | What it costs |
| --- | --- | --- |
| A. By 0.10.0 the owner reads the reference scripts of the estimators (`reg`, `ivreg`, `first_stage`, the variance layer) and runs the Stata references himself; before submission an outside statistician or numerical analyst who reads Rust audits the kernels and the variance layer, and the audit is published with the release notes | The correctness of the engine rests on a person's reading as well as on its tests; the article can say who checked what | The owner's time; the cost of the audit, or a co-author who does it |
| B. The owner's reading and Stata runs only | No outside cost | No one but the owner vouches for the code |
| C. The tests alone | No work | A referee may judge the code unreviewed |

**Recommended: A.** Deadline 1.0.0 for the audit; the owner's part by 0.10.0.

### 155. The article's case study and its replication project

The journal asks for a non-trivial case study and for replication material that runs within about an hour. The draft's examples use the synthetic `wages` and `staff` and Card's data, whose licence is not yet confirmed.

| Option | What it gives | What it costs |
| --- | --- | --- |
| A. One published empirical study whose data are openly licensed (CC0, CC-BY 4.0, ODC-BY, ODbL or public-domain government data, as section 8.5 requires), replicated end to end in Zeff: the raw files, the cleaning, a panel with fixed effects, instrumental variables, the check against a causal graph, the tables and `zeff verify`; and the article itself a project that `zeff verify` re-runs within an hour, with the ten-million-row benchmark given a referee's size and its full-size timings stated apart | The case study the journal expects; the published figures as an outside check; replication within the journal's hour | Finding a study whose data may be shipped; replication packages deposited with a CC-BY licence (some at openICPSR) are the likely source |
| B. The synthetic datasets only | No licence question | Not the non-trivial case the journal asks for |

**Recommended: A.** The choice of study is the owner's. Deadline 0.11.0, the release of `zeff verify`.

### 156. When the article is submitted

The draft proposes submission at a tagged release, preferably 1.0.0. The range of methods at 1.0 is narrow; under the plan of #113 bootstrap standard errors and the `external` step come at 1.1, Poisson regression with fixed effects at 1.7, two-period difference-in-differences at 1.9 and staggered adoption at 1.19, at about one minor release every three months.

| Option | What it gives | What it costs |
| --- | --- | --- |
| A. A preprint at 1.0.0; the submission at 1.1, presenting Zeff as infrastructure on which methods are built | The preprint fixes priority and draws comment; 1.1 adds the bootstrap to every model and a recorded route to R and Python for what Zeff lacks | About three months later than B |
| B. Submission at 1.0.0 | Earliest | Bootstrap absent; a referee may find the methods thin |
| C. Wait for a wider range, for instance 1.9 | Broad coverage | About two years later; the article's argument does not rest on breadth |

**Recommended: A.** The owner's word is asked.

### 157. Packages written in Zeff during 1.x, beside the ruling of #123

The article presents Zeff as infrastructure, which a referee may question while no one but the project can add a function to it before 2.0.0 (#123, 11 November 2026). #8 had recommended packages written in Zeff at 1.6, with WebAssembly components later.

| Option | What it gives | What it costs |
| --- | --- | --- |
| A. #123 stands: no third-party package before 2.0.0; the article points to the project's own functions, the published package manifest of 1.0.0 and `external` of 1.1 | No change; every function a script calls is the project's own | The infrastructure claim rests on a promise |
| B. Packages written in Zeff, with the manifest and `zeff.lock`, at 1.6; WebAssembly components at 2.0.0 | Others can add methods several years earlier; no sandbox is needed, since the language is the sandbox | The manifest and lock-file work moves forward from 2.0.0 |

**Recommended: B.** The owner's word is asked: this reopens #123. Deadline 1.1, so that the article can say which.

### 158. Testing a causal graph's implied independencies against the data

`g.implied_independencies()` (catalogue row 431, 0.10.0) lists the independencies a graph implies as testable predictions, but nothing tests them. The test is what makes a graph something the data can contradict, and it is the feature a referee from the causal-inference side will look for. dagitty's `localTests` is the reference; it is not in the R library and needs the owner to install it.

| Option | What it gives | What it costs |
| --- | --- | --- |
| A. An option `data:` on `g.implied_independencies()` at 0.10.0: with a frame, each implied independence is tested (a partial correlation for numeric variables, as dagitty's linear test) and printed with its estimate, interval and p value, under `p_adjust`'s methods | One function with an option (decision 97); the graph meets the data | One option on a catalogued signature; a reference test that needs dagitty |
| B. A function of its own | A shorter call | Against decision 97 |
| C. After 1.0 | No work now | The article's centrepiece lacks its test |

**Recommended: A.**

**Two findings from drawing the diagrams (the Zemp Lite session, 18 November 2026; entered 20 November 2026).** Made with the shipped `wages` data and R, for the print of this function and its reference tests. First, the example graph met the data and was contradicted twice: age and union (r 0.085, with interval 0.035 to 0.135) and union and experience given sex (r 0.093, 0.042 to 0.142); the print states it and no arrow is drawn (#194, rule 1). Second, three of the graph's implications cannot be tested on `wages`, because `exper` is exactly `max(0, age − educ − 6)` in all 1,516 rows of 2019 where it is known, so a test of `exper ⊥ union | age, educ` is degenerate: the print should say so when a conditioning set determines a variable, rather than print a number.

### 159. Candidate instruments from a causal graph

`iv.exclusion_check(g, instrument:)` (row 436, 0.10.0) checks an instrument the analyst names; nothing lists the instruments a graph allows. dagitty's `instrumentalVariables` lists them, with the conditioning set of each conditional instrument.

| Option | What it gives | What it costs |
| --- | --- | --- |
| A. `g.instruments(exposure:, outcome:)` at 0.10.0, a result method of `identify` listing the instruments and conditional instruments with their conditioning sets, held to dagitty | Identification by instruments read from the graph, beside adjustment sets; a natural example for the article | One function added to the catalogue before the names freeze |
| B. `exclusion_check` only | No new function | The graph checks a choice but does not suggest one |

**Recommended: A.**

### 160. `sample_flow` before 1.0

`data.sample_flow()` and `fit.sample_flow()` (rows 684 and 685) print and draw every step that changed the number of rows, with its count and reason; they are planned for 1.2. The verbs and the fits already count every dropped row, so the table is the sixth design principle ("nothing disappears silently") shown in one call, which the article can print.

| Option | What it gives | What it costs |
| --- | --- | --- |
| A. Both, the table and the drawing, at 0.11.0 | The whole feature in 1.0 | The drawing needs the diagram layer, work of 1.2 |
| B. The printed table at 0.11.0, the drawing at 1.2 as planned | The part the article needs, on machinery that exists | The two rows split across releases |
| C. 1.2, as planned | No change | The article describes the principle without showing it |

**Recommended: B.** Deadline 0.10.0, so that 0.11.0 is planned with it.

### 161. Aids for the article's referees

The journal warns that software in a less common language is slow to review. The website is held until 1.0.0 by the owner's ruling.

| Option | What it gives | What it costs |
| --- | --- | --- |
| A. From 0.9.0, a citation file in the repository and an archived copy of each release with its own persistent identifier (the owner links the archive to the repository once); at 1.0.0, on the website, a playground in the browser build with the article's examples loaded, and a page translating Zeff to R and Stata, generated from the "Coming from R or Stata" part of every help page | A referee can run the examples with nothing installed and read Zeff through R or Stata; the article can cite an exact archived version | One page of the site generated from the help; the owner's one-time link of the archive |
| B. At 1.0.0, all of it | One piece of work | The archived versions start later |

**Recommended: A.**

**The owner's decision (21 November 2026, at the opening of release 0.9.0).** The owner said "go" to the checklist of `docs/releases/0.9.0.md` without changing its answers, so its recommendation is taken under his standing instruction of 12 November 2026, his to overturn. Option A: a citation file in the repository from 0.9.0 (step 37); the archived copy of each release with its own identifier needs the owner to link the archive to the repository once; the playground and the translation page at 1.0.0 (or with #162 if it is taken).

**Status (24 November 2026, step 37 of Zeff 0.9.0).** `CITATION.cff` is in the repository, in the Citation File Format 1.2.0: the title, an abstract in the words of the README, the author as the copyright notice names him, the licence, the repository, and the version and date of the last release (0.8.0, 20 November 2026). A test holds it: while the workspace is at a development version the file names a version before it, and once the workspace is set to a release the file must name that version, so a release cannot be made without bringing the citation to it. The owner's to give or change, none blocking: his name as he wishes it cited (taken as family name Patra, given name Sabyasachi), an affiliation and an ORCID if he wants them in the file, the wording of the title and abstract, and the one-time link of the repository to an archive that gives each release its identifier, which then goes into the file as its `doi`.

### 162. A release 0.12.0 for the bridges to R and Python, before the names freeze

Raised on 16 November 2026, when the owner asked which changes to the release plan follow from #151 to #161. #152 places an R package on CRAN and a Python wheel on PyPI at 1.0.0, and #161 places the browser playground and the R and Stata translation page there too. Release 1.0.0 already carries the freeze of names, the edition of 2027, the installers, signed archives, the container image, reproducible official builds and the public benchmark suite (section 14 of the Engine Specification). The bridges are also the first use of Zeff's functions and results from outside the language: a name or a result layout that reads badly from R or Python can still be changed before the freeze, and not after it.

| Option | What it gives | What it costs |
| --- | --- | --- |
| A. A release 0.12.0 "Bridges" between 0.11.0 and 1.0.0: the R package and the Python wheel of #152, the playground with the article's examples and the translation page of #161, and any change of name or layout that the bridges show to be needed | Each release stays the size of one theme; the names are tried from R and Python before they freeze; 1.0.0 stays the release of the freeze and the installers | A row added to section 14; 1.0.0 later by the length of one release, at the pace so far one or two weeks |
| B. All of it in 1.0.0, as #152 and #161 have it | No new release | The heaviest release grows; a name the bridges show to be awkward is found as the names freeze |

**Recommended: A.** The owner's word is asked: it adds a release to the build order. Deadline 0.10.0, so that 0.11.0 is planned knowing what follows it. If taken, section 14 of the Engine Specification gains the row, logged in `docs/spec_changes.md`.

### 163. Poisson regression with fixed effects, the wild cluster bootstrap and difference-in-differences before the plan of #113 places them

Raised on 16 November 2026 with #162. Under the plan of #113 (11 November 2026), which orders the releases after 1.0 by level (basic, then advanced, then research grade), Poisson regression with fixed effects is built at 1.7 (`glm` with `family: poisson` and PPML), the two-period difference-in-differences design with `pre_trends` at 1.9, the wild cluster bootstrap at 1.18, and difference-in-differences under staggered adoption (`did` with `method: sa | dcdh | imputation`, Callaway and Sant'Anna's estimator, `honest_did`) at 1.19, about five years after 1.0 at one minor release every three months. These are among the methods applied economists use most, and the article's referees may ask for them during a review that can take a year.

| Option | What it gives | What it costs |
| --- | --- | --- |
| A. Poisson regression with fixed effects (with the separation checks of ppmlhdfe) and the wild cluster bootstrap at 1.2; the two-period design with `pre_trends` and the staggered-adoption methods at 1.3; the releases of #113 from 1.2 on each moved two places later | The methods most asked for by applied work within about six months of 1.0; an answer in a released version if referees ask for them; references in R (fixest, fwildclusterboot, did, didimputation) | The level order of #113 broken for one family; staggered adoption is numerically delicate and needs its validation time; every later release about six months later |
| B. Poisson with fixed effects and the two-period design only, at 1.2; the bootstrap at 1.18 and staggered adoption at 1.19 as planned | The two methods closest to the basic level; one release moved, not two | Staggered adoption still about five years away |
| C. #113 stands | The plan the owner set | As now |

**Recommended: A.** The owner's word is asked: it reopens #113. Deadline 1.1, since release 1.2 is planned at the close of 1.1.

## Raised from the owner's questions

### 164. Orthogonal polynomials of a continuous variable in a formula

Raised on 17 November 2026, when the owner asked how a polynomial regression is fitted in Zeff. Powers are written in the formula and worked out before fitting (section 7.1 of the Language Reference): `y ~ x + x^2 + x^3` fits the cubic. On the shipped `wages` for 2019, a quadratic and a cubic in experience with robust errors agree with R's `lm` with `I(exper^2)` and `sandwich`'s HC1 to every printed digit, with the joint test of the higher powers (F(2, 1485) = 6.07), the turning point by `derive` (28.20 years, standard error 2.96) and the predictions. What Zeff lacks is the orthogonal form: R's `poly(x, 3)` and Stata's `orthpoly` replace the powers by polynomials of degree 1 to 3 that are uncorrelated in the sample. The fitted values, the residuals, R² and the joint test of the higher terms are those of the raw powers; the coefficients differ, each term's test is then the test of that degree's own contribution, and the columns stay well conditioned at a high degree, where the raw powers of a variable with large values are nearly collinear. Zeff has orthogonal trends only for an ordered category, by `category(x, contrast: poly)` (catalogue row 40); neither the catalogue nor the plan of #113 has the continuous form. The owner's rule of decisions 98 and 99 is that every method R or Stata offers for a procedure is an option in Zeff; his instruction of 11 November 2026 is that nothing is added before 1.0 unless it is necessary.

Two points of design go with any option that adds the term. Predictions and `design()` on new rows evaluate the basis of the fit's own sample (its centring constants and norms, kept with the fit), never a basis recomputed from the new rows, as R's `predict` does for `poly`; otherwise a prediction at a single row is wrong. And the term names follow section 7.1's rule that a transformation keeps its written form, with the degree appended as R does: `poly(exper, 3)1`, `poly(exper, 3)2`, `poly(exper, 3)3`, given as text in `coef`, `test` and `derive`.

| Option | What it gives | What it costs |
| --- | --- | --- |
| A. A formula term `poly(x, degree: Int, raw: Bool = false)` in `regress` at 1.2, with the basic econometrics of #113: orthogonal by default as in R, `raw: true` the plain powers under one term; held to R's `poly` and `predict`, and to Stata's `orthpoly` up to its scaling; meanwhile the help page of `reg` shows the raw powers and names `poly` as coming | One function with an option (decision 97); R's and Stata's method present; nothing added to 1.0; an addition at 1.2 breaks no script | A user coming from R waits for 1.2 and writes the powers until then |
| B. The same term at 0.9.0, with `glm` and the models of that release | Present at 1.0 | A new catalogued function before 1.0 that is not necessary, since the raw powers fit the same model |
| C. No `poly`; the help page of `reg` shows the raw powers and how to read them | Nothing to build | The rule of decisions 98 and 99 unmet for a method both R and Stata offer |

**Recommended: A.** Deadline 1.1, since release 1.2 is planned at the close of 1.1. Taken as recommended under the standing instruction of 12 November 2026 unless the owner says otherwise.

**Addendum (20 November 2026; from the owner's questions of that day, written up in a Zemp Lite session and entered here).** Two points the entry does not settle, each the owner's; with them go #195 (slopes and turning points through `margins`, due at 0.9.0) and #196 (the two messages an R user meets today).

*1. The names of a `poly` term's columns.* The second paragraph above names them as R does, `poly(exper, 3)1` and on. But the owner ruled on 23 October 2026 that the columns of the `poly` contrast, the same orthogonal polynomials over a category's levels, are named `var=linear`, `var=quadratic`, `var=cubic`, `var=degree4` and on; one idea would then be named two ways. And on the same rows the column of each degree is the same whatever the highest degree asked for, so a name that holds the degree puts one column on two rows of a `report` of a quadratic and a cubic fit.

| Option | For | Against |
| --- | --- | --- |
| A. `poly(exper, 3)=linear`, `=quadratic`, `=cubic`: the written form, then the ruling's words | Keeps section 7.1's rule (a transformation keeps its written form) and the ruling's words | A quadratic and a cubic fit name the same column differently, so `report` gives it two rows |
| B. `poly(exper, 3)1`, `2`, `3`, as this entry and R have it | R's names | Departs from the ruling of 23 October; the same two rows in `report` |
| C. **`poly(exper)=linear`, `=quadratic`, `=cubic`**: the degree left out of the name | The ruling's words; a column keeps one name across fits of different degrees, so `report` aligns them; still plainly a `poly` term | A small exception to section 7.1's rule, needing a sentence in it when the term is built |
| D. `exper=linear`, the contrast's form exactly | Short; aligns across fits | Reads as a level of a category called `exper`; the name no longer says the column is orthogonal rather than a power |

*Recommended: C.* Under `raw: true` the columns take the names the powers would have if written by hand (`exper`, `exper^2`, `exper^3`), so that a raw term is interchangeable with the powers written out. Whatever is chosen, the pages of `category` and of `poly` show R's names beside Zeff's. Deadline: this entry's own, 1.1.

*2. The rows and the weights the basis is orthogonal on.* A statistical definition, and the owner's to settle. *Rows:* recommended, the rows the fit uses (after `where`, after the listwise deletion of every model variable, after singletons are removed), so that each column is uncorrelated with the others in the estimation sample itself. R evaluates `poly(x)` on the rows of the model frame before the rows with `NA` in other variables are dropped, and refuses `NA` in `x` itself; its columns are therefore orthonormal on the estimation sample only when no row is dropped. The fitted values, the residuals, R² and the joint test are the same either way; the coefficients of the separate degrees differ from R's whenever rows are dropped, so the reference tests compare coefficients on data from which no row is dropped and fitted values otherwise, and the page says so. *Weights:* recommended, the basis is unweighted, as R's `poly` is, so that it depends on `x` and the rows alone; the fitted values are the same under either choice. Stata's `orthpoly` documents a weight option, and what it does with it is to be checked when the reference is built. *Fixed effects:* under `fe:` the basis is computed on the estimation rows before the effects are absorbed; after absorption the columns are no longer orthogonal, which changes nothing in the fit, and the page says so.

## Raised by the review against R

**The owner's answer (21 November 2026, given in the Zemp Lite session at step 1 of its release 0.2 and passed on in the shared notes; entered from a Zeff session, which alone edits this register).** His words to the seventeen questions of that checklist, each put with a recommendation: "Go, as recommended." Point 1 of the addendum: option C, the columns named `poly(exper)=linear`, `=quadratic`, `=cubic`. Point 2: the basis is orthogonal on the rows the fit uses, unweighted. The rest of the entry stays open, deadline 1.1.

### 165. The length of each text value in a column

Raised on 19 November 2026 by the review against R. Inside `add`, `len(job)` gives the number of rows (41,188 on every row of the bank-marketing file), as its page says a column's length is its row count; there is then no way to get the length of each value, which R gives by `nchar` and Stata by `strlen`. The result looks plausible and nothing warns.

| Option | What it gives | What it costs |
| --- | --- | --- |
| A. A text function `n_chars(s)` in `text` at 0.9.0, the length of each value in code points as `len` counts a string, `NA` for `NA`; and a note when `len` of a column is spread over the rows of `add` | The per-row length; `len` keeps one meaning; the trap is signalled | A new catalogued function before 1.0 |
| B. `len` of a Str column inside `add` gives each value's length | No new name | `len` of a column means two things by context |
| C. Nothing; the page of `len` warns | Nothing to build | R's and Stata's method missing (decisions 98 and 99) |

**Recommended: A.** Deadline 0.9.0. The name is the owner's to choose.

**The owner's decision (21 November 2026, at the opening of release 0.9.0).** The owner said "go" to the checklist of `docs/releases/0.9.0.md` without changing its answers, so its recommendation is taken under his standing instruction of 12 November 2026, his to overturn. Option A with the name `n_chars(s)` in `text`, the length of each value in code points, `NA` for `NA`, and a note when `len` of a column is spread over the rows of `add`; built at step 31 with its catalogue row.

**Status (23 November 2026).** Built at step 31a of Zeff 0.9.0, as decided: `n_chars(s)` in `text`, held to R's `nchar(type = "chars")`; `add` notes `len` of a text variable written as a whole definition, and names `n_chars`. Catalogue row 698. Deadline for the owner to overturn the name: 1.0.


### 166. A row-wise count of missing values

Raised on 19 November 2026 by the review against R. `n_missing([x1, x2, x3])` inside `add` counts the three columns as the items of a list and gives 0 on every row. The count of missing values across variables in each row, R's `rowSums(is.na(...))` and Stata's `egen rowmiss()`, cannot be written; step 20j makes a list of columns an error.

| Option | What it gives | What it costs |
| --- | --- | --- |
| A. `row(vars, stat: n_missing)`, a statistic of `row` beside `mean` and `sum`, at 0.9.0 | One function with an option (decision 97); Stata's form | One more value of `stat:` |
| B. `n_missing` of a list of columns counts across each row | No new option | `n_missing` of a list means two things |

**Recommended: A.** Deadline 0.9.0.

**The owner's decision (21 November 2026, at the opening of release 0.9.0).** The owner said "go" to the checklist of `docs/releases/0.9.0.md` without changing its answers, so its recommendation is taken under his standing instruction of 12 November 2026, his to overturn. Option A: `row(vars, stat: n_missing)`; built at step 31.

**Status (23 November 2026).** Built at step 31c of Zeff 0.9.0, as decided: `row(vars, stat: n_missing)`, an Int in every row, for variables of any type, held exactly to R's `rowSums(is.na())`. Catalogue row 136 has the wider usage. Open beside it, for the owner: whether the number of values present (Stata's `egen rownonmiss()`) should be a word of `stat:` too. Deadline for the owner to overturn: 1.0.


### 167. A row number

Raised on 19 November 2026 by the review against R. `add(id = seq(1, n()))` is refused, since a list is not a column; the review wrote `add(one = 1).add(id = running(one, stat: sum))`. R's `row_number()` and Stata's `_n`, within groups as `by: g` gives them, are used to number observations, to make an identifier before `to_long`, and to take the first row of each group.

| Option | What it gives | What it costs |
| --- | --- | --- |
| A. `row_number()` in `data` at 0.9.0, the position of the row, within each group under `by:` | Stata's and dplyr's method | A new catalogued function |
| B. `seq(1, n())` accepted inside `add` as a column | No new name | A list turned into a column by context |

**Recommended: A.** Deadline 0.9.0. The name is the owner's to choose.

**The owner's decision (21 November 2026, at the opening of release 0.9.0).** The owner said "go" to the checklist of `docs/releases/0.9.0.md` without changing its answers, so its recommendation is taken under his standing instruction of 12 November 2026, his to overturn. Option A with the name `row_number()` in `data`, within each group under `by:`; built at step 31 with its catalogue row.

**Status (23 November 2026).** Built at step 31b of Zeff 0.9.0, as decided: `row_number()` in `data`, within each group under `by:`, in the order the rows stand; it stands in `add`, `update` and `where`. Held exactly to base R. Catalogue row 699. Deadline for the owner to overturn the name: 1.0.


### 168. One value out of a column

Raised on 19 November 2026 by the review against R. `first(col)`, `last(col)` and `col[1]` are refused for a column, and `to(col, List)` converts each value and gives a Column<List>. One prediction had to be read as `frame(p = m.predict(newdata: b.head(1))).mean(p)`.

| Option | What it gives | What it costs |
| --- | --- | --- |
| A. `first`, `last` and `x[i]` take a column, and `to(column, List)` gives the List of its values, at 0.9.0 | The reading R gives by `x[1]` | Three functions widened |
| B. Only `to(column, List)` changed | One change | Two steps for one value |

**Recommended: A.** Deadline 0.9.0.

**The owner's decision (21 November 2026, at the opening of release 0.9.0).** The owner said "go" to the checklist of `docs/releases/0.9.0.md` without changing its answers, so its recommendation is taken under his standing instruction of 12 November 2026, his to overturn. Option A: `first`, `last` and `x[i]` take a column, and `to(column, List)` gives the List of its values; built at step 32.

**Status (23 November 2026).** Built at step 32a of Zeff 0.9.0, as decided: `x[i]`, `first(x)` and `last(x)` take a column, and `to(x, List)` gives the list of its values. A range of rows is not taken from a column, which the message says with the ways to get it. Deadline for the owner to overturn: 1.0.


### 169. Widening a guessed Int to Float on import

Raised on 19 November 2026 by the review against R. `import` guesses each type from the first 1,000 rows and stops when a later value does not fit (Z0606): the bank-marketing file stops at row 7,765, where `nr.employed` holds 5228.1. The message names the fix, but widening Int to Float loses nothing, and R's `read.csv` reads the file as it is.

| Option | What it gives | What it costs |
| --- | --- | --- |
| A. A later value with a decimal widens an Int column to Float, with a note naming the row; only text in a numeric column, or a value in a Date column that is not a date, stops the import | The file read whole, the change reported | Section 6.1's rule amended and logged |
| B. As now | Nothing to build | A whole-number column with a late decimal always needs `types:` |

**Recommended: A.** Deadline 0.9.0.

**The owner's decision (21 November 2026, at the opening of release 0.9.0).** The owner said "go" to the checklist of `docs/releases/0.9.0.md` without changing its answers, so its recommendation is taken under his standing instruction of 12 November 2026, his to overturn. Option A: a later value with a decimal widens a guessed Int column to Float with a note naming the row; section 6.1's rule is amended at step 32 and logged.

**Status (23 November 2026).** Built at step 32b of Zeff 0.9.0, as decided: a later decimal widens a guessed Int column to Float with a note naming the row; text in a numeric column, a value that is not a date in a Date column, and a decimal in a column declared `Int` stop the import. Sections 4.1, 5.6 and 6.1 of the Language Reference are amended and logged. Deadline for the owner to overturn: 1.0.


### 170. Six smaller refusals with workarounds

Raised on 19 November 2026 by the review against R. Each has a workaround; together they are friction a user meets on a first real script.

| Item | As now | Recommended |
| --- | --- | --- |
| `equal(a, b)` on two columns | refused | allowed, true when the values and their `NA` agree |
| `category(["low", "high"], levels: ...)` on a list | refused | allowed, giving a Category column |
| `duplicates(vars)` | a Table that cannot be counted | the Table kept; `n_rows()` and the frame of duplicates available from it |
| `attempt(error("…"))` | refused, `error` having no value | allowed, giving a failed value |
| `map` over a list holding a failed value | refused even to apply `is(Failed)` | allowed when the function is `is`; otherwise as now |
| `chi2_test(x, p:)` when a category is never observed | refused for the count of values | `levels:` names the categories, the unobserved counting 0 |

Deadline 0.9.0.

**The owner's decision (21 November 2026, at the opening of release 0.9.0).** The owner said "go" to the checklist of `docs/releases/0.9.0.md` without changing its answers, so its recommendation is taken under his standing instruction of 12 November 2026, his to overturn. Each item as the table recommends; built at step 33.

**Status (23 November 2026).** Built at step 33 of Zeff 0.9.0, each item as the table recommends. Two readings were taken on the way, the owner's to overturn: the frame of `duplicates` is given by `to(t, Frame)`, the conversion, and not by a new function; and `filter` goes over a list with a failed value under the same test by `is` as `map`, so that `filter(fits, fn(f) !f.is(Failed))` keeps the fits that succeeded. Deadline for the owner to overturn: 1.0.


### 171. Six printed layouts the review found

Raised on 19 November 2026 by the review against R. Each is a printed layout and so the owner's to decide.

| Item | As now | Recommended |
| --- | --- | --- |
| Small coefficients in `report` | `-0.0005` prints as `0.000`, `credit_score` as `0.000*` | significant digits below 0.0005, sign kept: `-0.00052` |
| `labels:` of `report` | renames `lsize` but not `lsize:city=Mumbai` | applied to each part of an interaction |
| The df column of Welch comparisons | `1762.3` | `1,762.3`, as elsewhere |
| Facet titles | `0`, `1` | `kids = 0`, `kids = 1` |
| Hypotheses of the proportion test with a Bool group | lines of 230 characters | the symbols' line wrapped under the words |
| The estimates of a t-test | `std_error` is `NA` | the standard error of the difference |

Deadline 0.9.0.

**The owner's decision (21 November 2026, at the opening of release 0.9.0).** The owner said "go" to the checklist of `docs/releases/0.9.0.md` without changing its answers, so its recommendation is taken under his standing instruction of 12 November 2026, his to overturn. Each layout as the table recommends; built at step 34.

**Status (23 November 2026).** Built at step 34 of Zeff 0.9.0, each layout as the table recommends. Readings taken on the way, the owner's to overturn: (1) a coefficient that would print as zeros is written to two significant digits, at any `digits:`, and its standard error to the same number of decimals (to two digits of its own where that would be zeros again); a coefficient that shows at `digits:` decimals prints as it did, its standard error with it, `(0.000)` included; below one in a million both are written with an exponent; the others of the column make room so that the decimal points stay one under another. (2) A label of a variable is printed in every term it is a part of, a level included (`Sex=female`); a label of a whole term comes first. (3) A facet is titled with its variable where its values are numbers or true and false with no value labels; a level, a text and a value label stand alone. (4) The symbols go under the words in any test whose line would pass the page's 118 characters, not in the proportion test alone. (5) A t-test's estimates carry the standard error of the mean or of the difference in every form (one sample, paired, pooled, Welch), held to R's `stderr`, and that of the effect size. Deadline for the owner to overturn: 1.0.

### 172. Typst inside Zemp, for the PDF of a script with its output

Raised on 19 November 2026 by the measurement of #73 (release 0.8.0, steps 21b and 21c). Decision #73 gives Zemp Lite an **Export script with output** as a PDF set by Typst, and the report document after it. The measurement found that Typst sets the same bytes on every target, that it cannot be inside the engine or the bundle a page loads at its start (26 MB; 7.2 MB compressed against a limit of 5 MB for the whole bundle), and that as published it compiles in 14 sets of hyphenation patterns under licences this project does not allow in what it links (the LPPL for 13 languages; the MPL, GPL or LGPL for Hungarian), with no way to leave a language out.

| Option | What it gives | What it costs |
| --- | --- | --- |
| A. Typst as a module of Zemp's own, fetched the first time a PDF is asked for, built with a copy of the hyphenation crate that holds the 33 languages under permissive licences (34 once the Slovak header is corrected at its source) | The PDF of #73, the same bytes everywhere, on the desktop and in the browser; nothing in the engine; the licence rule kept as it is | A fetch of about 7 MB on first use; a changed copy of one small crate to carry at each upgrade of Typst; no hyphenation in Dutch, Russian, Swedish and eleven more until their patterns can be chosen |
| B. The same module with every language, the rule for dependencies widened to take the LPPL and the MPL for data that is compiled in | Nothing to carry; every language hyphenated | A licence rule with two more entries, each with conditions on changed copies; a lawyer's reading before 1.0 |
| C. No Typst inside Zemp: the `.typ` export, set by the Typst the user has installed, and the browser's own printing to PDF for a script with its output | Nothing to build, fetch or carry | Not the same bytes everywhere; nothing for a user without Typst but the browser's print; the live preview of #73 is given up |
| D. Wait for a Typst that lets the languages be chosen, and decide then | Option A without the changed copy | No date: the nearest thing is an open request of the hyphenation crate, to load a language's patterns when the program runs |

**Recommended: A**, built when Zemp reaches **Export script with output**, with C as what a user has until then (it is what 0.8.0 ships). Deadline: the Zemp release that builds that export. Whether the LPPL and the MPL may ever be accepted (option B) is the owner's alone to say.

### 173. Probabilities below the smallest double: a log scale for the tails

Raised on 20 November 2026 by the reading of the coverage report (release 0.8.0, step 24a2b): the lines of `dist.truncated` that no test reached are reached only by a truncation whose whole probability is near the smallest double, and there its results lose their digits. A double holds a number below 2.2e-308 with fewer digits and nothing below 4.9e-324; every distribution function has that limit (`dist.normal().cdf(-38.4)` is 6.4e-323 to two digits, in Zeff as in R). A truncation divides two such probabilities, so the loss shows in a probability of ordinary size: for a standard normal cut at −37, whose whole probability is 5.7e-300, the distribution function agrees with R's on the log scale to nine digits down to −37.8, is 5 per cent off at −38.4 (1.208e-23 for 1.153e-23) and is 0 from −38.5, and the quantile at 1e-300 is −38.49 where −52.44 is right. Nearer than 37 standard deviations, or 700 mean lifetimes of an exponential, nothing is lost.

| Option | What it gives | What it costs |
| --- | --- | --- |
| A. State the limit: on the page of `dist.truncated`, in the known issues, and hold its edge by tests against R | The page no longer promises what a double cannot hold; nothing changes for any use nearer than 37 standard deviations | Beyond the edge a number is printed that has lost its digits, with no word at the place |
| B. Give `NA` with a warning wherever a probability below 2.2e-308 enters a truncation | No number without digits is printed | A rule with an edge of its own (a tail of exactly zero is also the right answer far outside the interval), and `NA` where an answer to several digits exists |
| C. Tails on the log scale: `log: true` on `pdf` and `cdf`, as R's `log.p`, and the truncation computed on that scale | The far tail to full precision, as R gives it on the log scale; useful beyond truncations (likelihoods, extreme tails) | New arithmetic in every family's tails and a new option on two methods: a design, not a correction, under the freeze |

**Recommended: A now, C placed in the plan of releases after 1.0.** A is done at step 24a2b of release 0.8.0. B is not recommended: it trades a visible limit of the numbers for a rule that is hard to state. Deadline for C: when the plan of releases after 1.0 is next amended. Decided for A on the recommendation, under the owner's standing instruction; his to overturn.

## Raised from Zemp Lite

Entered on 20 November 2026 from the notes the Zemp Lite sessions wrote between 15 and 20 November 2026 in the form of this register, which is edited from a Zeff session only. One entry records a ruling of the owner (#174); every other is a proposal that is his to decide, with the deadline its note proposed. The design document, the mockups and the screenshots they name are kept outside the repositories, in `~/dev/handbook/zemp-lite-margin-editor/`.

### 174. The editor of Zemp Lite: the margin editor

Raised on 17 November 2026 by the owner's plan for a margin editor, and revised in the Zemp Lite session into the design `~/dev/handbook/zemp-lite-margin-editor/zemp_lite_margin_editor_specification.md` (version 0.9, with its mockup and screenshots in the same folder, outside the repositories). Entered here on 20 November 2026 from the notes of that session, which may not edit this register.

The Inline layout of the Zemp Specification, section 5, puts each result under its statement. That shows everything, but the script changes shape at every run, a long script is buried in its output, and the page can move under the reader (two mends in the 0.1 prototype). The owner's plan answered with a column of one-line notes beside the code; its weakness is the reverse, since one line is not a result.

| Option | For | Against |
| --- | --- | --- |
| A. **One editor: a note beside every statement, the full result opened under its statement when the reader asks.** The note is narrow and is the engine's own text; opening it shows what the statement prints, as `zeff run` prints it, with its warnings and its figure; several may be open. Running never moves the code; only the reader's opening does. An edit marks what depends on it stale in place, the last value kept. No lens, no shelf, no thread rail, no second layout to keep | The plan's central gain (the code never moves; every `let` has a result; staleness in place) with the Inline layout's (the whole result, in the engine's print, several at once). One mechanism instead of two layouts. Works in a narrow window | An opened result pushes the lines under it down, at the reader's asking. The top bar of section 5 gains the places |
| B. The plan as written: a margin with a lens, a shelf, a thread rail, spreads and six places, Inline dropped or kept beside it for a release | The fullest form of the idea; the owner's own design | The lens cannot hold the engine's print. Scope several times the plan of section 15 before 1.0. Collides with decisions taken (#146, #124, #141). Keys a browser does not give a page |
| C. The Inline layout as section 5 has it, with what is planned for it | Built in the 0.1 prototype and tested against the engine's print byte for byte | The script's shape still changes at every run; a `let` shows nothing; long scripts stay long |

*Decided 18 November 2026 (option A, on the recommendation).* The owner, asked whether the margin editor could be built before Zemp release 0.1 closed, was advised not, and was offered instead to adopt it as the editor of the 0.2 alpha; he answered: "Yes, adopt it." The ruling is recorded in the Zemp Lite repository's `docs/decisions.md` (18 November 2026). The margin editor replaces the Inline layout as the editor of Zemp Lite. Running never moves the code; a narrow note beside each statement gives what it made and how large it is (a name, a type and dimensions, a warning's code); the whole output unfolds under its statement at the reader's asking, in the engine's own print; stale results are marked in place and keep their last value; a script the check refuses runs nothing and loses nothing.

*Ruled with it, the same day.* A script opens with every output folded, under three controls, **Notes · Fold all · Unfold all**, where **Notes** shows or hides the notes column, which stays at the right of the code (R3 of the document). No strip of rows is added to an unfolded output: an output holds what the engine said and printed. The places are **Script, Data, Plots and Paper**; a place for models side by side was withdrawn (R5), and its two ideas worth keeping are #188 and #189. **Paper** edits `.zmd` papers by default and also opens and edits `.typ` files, the page always set by Typst; this touches #73, #124 and #141 and changes none of them. The quiet look is adopted for the editor (R11): it keeps everything #146 rules and changes how the window is drawn; the design guide's section 9 is brought into line by the owner's word. A one-line headline for each result and a term to head each fit, asked for in earlier versions, are withdrawn (R6 and R7).

*When.* The margin, the notes of type and dimensions, outputs unfolded at the width of their place, the three controls, a refused script, the run record, the list of what the run said, and Plots with its list are the editor of the Zemp 0.2 alpha, on Zeff 0.8.0; then ⌘↵ and **Run to here**, with the loop inspector of #76. Every later piece comes at the release it rests on. Zemp 0.1 closed as it stood.

*What it amends, when the 0.2 alpha is planned,* each change logged in `docs/spec_changes.md` then: the Zemp Specification's sections 2 (the Lite row of what writes to the script), 5 (Inline replaced by the margin editor; the results bar, the notes, the three controls), 7 (the plot viewer's list at the left), 8 (where reports and papers are written, with #141) and 15 (the 0.2 alpha's row names the margin editor). No normative text is changed by this entry.

*Two earlier proposals it meets.* Copying a result and saving a figure from its place under the statement is the header of an opened result (Copy, Open in Plots, Fold). Going from an object to the line that made it is the Data place, which lists each frame by its statement. Neither needs an entry of its own.

*What it asks of the engine* is #175; what it leaves open is #176.

**Revised (21 November 2026, the owner's ruling; #203).** From Zemp Lite 0.2.1 the margin editor and Panes are one layout of three panes on every place: the margin of notes is the **Notes** pane at the right of the Script place, open at first, and **Run** unfolds every output under its statement, where the ruling of 18 November 2026 left them folded; a script opened is folded as before. The 0.2 alpha is as this entry describes it.

### 175. What the margin editor asks of the engine: the contract, C1 to C27

Raised on 18 November 2026 with #174; entered on 20 November 2026. The interface is only as exact as what the engine tells it. The editor's design lists what it asks as twenty-seven items (its section 12), each a question the interface asks and the form of the answer, none a rule the interface keeps for itself. They are to be agreed with the Zeff side item by item, each placed at a release, and each tested in the Zeff repository. The section numbers in the third column are the design document's.

| # | The engine says | For | Where it stands |
| --- | --- | --- | --- |
| C1 | Where it begins and ends, and where each step of a chain and each block begins and ends | Placing notes and opened results | Statement ranges come with Zeff 0.8.0. Steps and blocks are new |
| C2 | Its **identity**: the hash its result is kept under | Staleness; "the same as in run *n*"; knowing a result again after a reload | Exists inside the engine (section 8.3, point 4). Handing it out is new |
| C3 | Whether its result would be **reused** for the script as it stands, and if not, which of the things of section 7.9 changed | Staleness and its reason | New as a question; the rule exists |
| C4 | What it reads and what it makes | The thread; "where it is used"; running what a statement needs | Asked for already by "Stale output"; the names come with the language server's navigation (1.1) |
| C5 | Its result's **type and dimensions** | The note | The engine knows both for every result. Handing them out per statement is new, and is far less than the one-line headline of version 0.2 |
| C6 | Its **print at a width** | The opened result | Exists (`--width`). Asking per statement is new |
| C7 | Its print's tables **as rows and cells** | A screen reader; copying a cell | New (decision R10) |
| C8 | What it said | Tags, cards, the list of what the run said | Exists: a diagnostic is one record for every reader (section 8.3, point 1) |
| C9 | For a fit, **the rows**: read, left out by each step, dropped for NA by variable, singletons, used; the clusters | `report()`'s note of different rows (9.1); the sample flow | In part: the engine prints the rows dropped and the singletons. By variable and by step is new |
| C10 | For a step of a chain, the rows in and out | Step notes | Planned as an inlay hint at 1.1 (Engine Specification, section 12) |
| C11 | For a fit, what it is | `report()`'s note and the plot of several fits (9.1, 9.2), inside the engine | In the result already; as fields, new |
| C12 | The run record, and for a partial run which statements it ran | The run record; section 7.8, rule 6 | Run records exist; partial runs are new |
| C13 | The difference of two runs | Comparing with an earlier run | `zeff runs diff` exists |
| C14 | Results kept between two runs in a browser | Running less than the script; comparison | The cache; at 0.7.0 nothing is kept in a browser |
| C15 | For rows in view, the rows a step left out and the condition that failed | Data, 8.1 | New |
| C16 | A plot as a scene | Redrawing without a run; reading a mark's row | Known since the 0.1 prototype, where a redraw is a run of the script |
| C17 | A paper's parts, with their lines: the settings, each chunk with its options, each inline value, citation, cross-reference and heading | The outline; the checks; finding the source | New, with the Markdown reader of #141 |
| C18 | For each place on the page, the line of the source it came from, and back | Clicking the page; lighting the cursor's place | Typst keeps the source of what it sets; carrying it through from the Markdown is new |
| C19 | Each inline value, table and figure with the identity (C2) of the result it shows | Stale values; Mark sources | New |
| C20 | The paper's checks, with code, sentence and line, as diagnostics (C8), the typed-number check among them | Checks | New |
| C21 | The page set again as the source is typed, fast enough to follow typing, with the values of the last run | The live page | Typst sets again only what changed; whether it is fast enough in a browser is part of the measurement of Zeff 0.8.0, step 21b |
| C22 | For an error raised inside a function, each link of the chain from the script's line to the line that raised it, with what each call was given; where a link is a pass, its number and the loop variable's value | The trail (7.16.1) | The chain and the arguments are in the engine's message from 0.7.0 (`= in: …, called at …`). As fields, and the pass, new |
| C23 | For a block, each pass: the loop variable, the rows after each step of its body, the result's type and dimensions, what it said and its time; for `iterate`, each state, each field's change from the pass before, and what `done` returned | The passes, *Find* (7.16.2) | Decided in #76: the run record keeps the pass. The fields, new |
| C24 | The replay of one call or one pass, keeping its bindings, each with its type and dimensions (C5) and its print at a width (C6) | Opening a pass or a call (7.16.3) | New. The values must be the run's, to the last bit |
| C25 | A statement evaluated in the names of a replayed pass or call | The console line inside it (7.16.4) | The console's evaluation exists; in a scope, new |
| C26 | Each statement's time, and each binding's memory | Where the time went (7.16.5) | Exists: the worker's step messages carry the time, which the 0.1 prototype shows, and `zeff run -v` writes the memory |
| C27 | A field of `iterate`'s states drawn against the pass, at the size of its place | Drawing a field (7.16.2) | The engine draws `line` from 0.7.0; from the run record's states, new |

Three things are worth weighing before any is built. Staleness is defined as the engine's own answer (C3): a result is stale exactly when the engine would not reuse it (Engine Specification, section 8.3, point 4), so a statement that draws random numbers, added or removed, makes every later drawing statement stale. The opened result is what `zeff run --width n` prints (C6), the interface choosing *n* and changing no character. And the check that a number typed by hand in a paper matches a result of the run (C20) is a candidate warning code for the document reader of #141.

| Option | For | Against |
| --- | --- | --- |
| A. Each item is placed at an engine release when the checklist of that release is written, as a step with its tests: for the Zemp 0.2 alpha C1 (built at 0.8.0, step 8b), C5, C6, C8 (built) and C12 (built); the items for finding a fault (C22 to C27) with #183 to #187; the items of a paper (C17 to C21) with the documents of #141; the plot as a scene (C16) with #197 | The interface never keeps a rule of its own; each item has a test in the engine's repository | Engine steps that no row of section 14 names, at several releases |
| B. The interface works each out from the engine's print and its own reading of the script | No engine work | Two implementations of the engine's rules, which drift; against the editor's own principle |

*Recommendation.* A. Whether the public crates of Zeff 0.8.0 already let the Zemp worker read C5 and C6 for each statement is to be found when the 0.2 alpha is planned; if not, a note falls back to the first line the engine printed for the statement, so that every character stays the engine's, and C5 and C6 are put to the checklist of Zeff 0.9.0. Deadline: the opening of Zeff 0.9.0 for C5, C6 and the items of #183 to #187; each other item at the release it rests on.

**The owner's decision (21 November 2026, at the opening of release 0.9.0).** The owner said "go" to the checklist of `docs/releases/0.9.0.md` without changing its answers, so its recommendation is taken under his standing instruction of 12 November 2026, his to overturn. Option A, with each item placed: C1, C8 and C12 are built; C5 and C6 (a result's type and dimensions, its print at a width) are built at 0.9.0, step 26, for the Zemp 0.2 alpha; C22 (an error's chain as fields, with the pass) at step 27, for #183; C16 (a plot as a scene) with #197 at step 25; C23 to C27 (the passes, replay, the console inside a pass or call, a field drawn across passes; #184 to #186) are placed at 0.11.0, beside the versioned run-record schema of that release; C7 with R10 of #176, C17 to C21 with the documents of #141, C2 to C4 and C9 to C15 at the releases they rest on, each placed when that release is planned.

### 176. Six questions the margin editor leaves open

Raised on 18 November 2026 with #174; entered on 20 November 2026. Each is the owner's to decide; the letters are those of the design document's section 20.2. (R4, the comment that begins a section, is with #178; R14 to R19 are #183 to #187, with a line under #76.)

| Document | Question | Recommendation | Deadline |
| --- | --- | --- | --- |
| R2 | The Panes layout beside the margin editor, or dropped from Lite | Kept for one release, then reviewed | Zemp 0.2 alpha |
| R8 | The keys of the document's section 15. A browser keeps ⌘1 to ⌘6, ⌘T, ⌘R, ⌘L and ⌘P for itself | As written there, with its four rules: ⌘↵ for the statement, ⌘⇧↵ for the script and ⌘K; places and the rest through ⌘K | Zemp 0.2 alpha |
| R9 | What ⌘↵ runs: the statement with every stale statement it reads, or the one statement alone in the live session | The first; the Engine Specification's paragraph on running code interactively (section 12) is then amended to match, since under the second the session can be out of sync | Zemp 0.2 alpha |
| R10 | The print's tables handed over as rows and cells, for a screen reader (C7 of #175) | Yes, when the engine can hand over the cells | A Zeff release to be named when C7 is placed |
| R12 | What Paper shows for a `.zmd` paper before the PDF of #141 is built, since #141 builds HTML first | The HTML article in the page's place until the PDF comes | The release after 1.0 that builds the documents of #141 |
| R13 | Where a paper's chunks run: in the paper's own session, or reading the results of the project's scripts | In the paper's own session, as #141 and Quarto have it, the cache making a repeated chunk cost nothing | The same |

**The owner's answer (21 November 2026, given in the Zemp Lite session at step 1 of its release 0.2 and passed on in the shared notes; entered from a Zeff session, which alone edits this register).** His words to the seventeen questions of that checklist, each put with a recommendation: "Go, as recommended." R2: Panes kept beside the margin editor for the 0.2 alpha, reviewed at Zemp 0.3. R8: the keys of the editor's document, section 15 (⌘↵ the statement, ⌘⇧↵ the script, ⌘K everything, ⌥↵ open or fold, F1 help, ⌃` the console; every action in ⌘K). R9: ⌘↵ runs the statement with every statement it reads that is out of date or not yet run; the paragraph "Running code interactively" of section 12 of the Engine Specification is amended to match, logged. R10, R12 and R13 stay open at their deadlines.

**R2 revised (21 November 2026, the owner's ruling; #203).** Panes was kept beside the margin editor for one release and reviewed at Zemp 0.3; the review is settled sooner: at Zemp Lite 0.2.1 the **Panes** button and the **Margin** link are removed, and both layouts become the one layout of three panes.

### 177. Zemp Lite is not a notebook: the statement is the cell

Raised on 16 November 2026 from the owner's review of notebook features. A notebook keeps code in cells inside a file of its own format, runs them in any order against a state that changes, and numbers each run so that the reader can reconstruct the order. The Inline layout already gives a notebook's main convenience, the result under the code that made it.

| Option | For | Against |
| --- | --- | --- |
| A. No cells. The script stays one plain text file that `zeff run` executes from top to bottom; the statement is the unit that is run, shown, folded and marked stale. Not adopted, with the reason: cells as a file format (hard to compare in Git; not runnable by `zeff run`), Markdown cells inside a script (prose belongs to the report document), a changeable shared state (a Zeff binding never changes, Language Reference, section 3.1), execution counters (they exist to diagnose out-of-order runs, which stale marks handle), and a semicolon to hide output (Zeff has no semicolons; `let` is already quiet) | One file, the same result from the interface and the command line; nothing a reader must reconstruct | A user arriving from a notebook has to learn that structure comes from sections (#178) and prose from the report |
| B. A cell mode beside Inline, saved in a notebook file | Familiar to notebook users | A second project format, against "Same projects" (Zemp Specification, section 2); results that depend on the order of running |

*Recommendation.* A. Deadline: the Zemp 0.2 alpha, since the public preview will be asked the question. The margin editor adopted on 18 November 2026 (#174) is built on this principle; the owner has not yet ruled on it in these words.

**The owner's answer (21 November 2026, given in the Zemp Lite session at step 1 of its release 0.2 and passed on in the shared notes; entered from a Zeff session, which alone edits this register).** His words to the seventeen questions of that checklist, each put with a recommendation: "Go, as recommended." Option A: no cells; the statement is the cell.

### 178. Sections in a script

Raised on 16 November 2026. A long script has no visible structure: nothing to outline, fold or run as a part. Notebooks get this from Markdown cells; R from `# Title ----`.

| Option | For | Against |
| --- | --- | --- |
| A. A comment written as a heading starts a section: `// # Data`, `// ## Cleaning` for a level below. The language server reports sections as the document's outline and folding ranges; Zemp shows the outline in the left panel, folds a section, and offers **Run section**. The engine ignores it, as any comment | No change to the language; the heading reads as the Markdown heading users know; the script stays plain text | A convention the formatter and the language server must both keep |
| B. A marker line in R's manner, `// Data ----` | Familiar to R users | No levels; easy to write by accident in a comment that ends with dashes |
| C. No sections; a long analysis is split into several scripts | Nothing to build | No outline or folding within a script, and no **Run section** |

*Recommendation.* A, built before Zemp Lite 1.0. Deadline: Zeff 0.9.0.

*The form of the comment (R4 of the margin editor's document).* The editor's design asks for one canonical comment that begins a section: `// ── Title ──`, `// # Title`, or none. It is the owner's choice, and the rules of the document's section 7.6 hold for either; it is due at the Zemp 0.2 alpha, where the outline of a script is first shown.

**The owner's decision (21 November 2026, at the opening of release 0.9.0).** The owner said "go" to the checklist of `docs/releases/0.9.0.md` without changing its answers, so its recommendation is taken under his standing instruction of 12 November 2026, his to overturn. Option A, with the comment `// # Title` (and `// ## Title` a level below) as recommended; the language server reports sections as outline and folding at step 28, and the formatter keeps them. The form is due at the Zemp 0.2 alpha and is decided here because the outline is built now; the owner may still change it before the alpha.

**The owner's answer (21 November 2026, given in the Zemp Lite session at step 1 of its release 0.2 and passed on in the shared notes; entered from a Zeff session, which alone edits this register).** His words to the seventeen questions of that checklist, each put with a recommendation: "Go, as recommended." The form of the comment (R4): `// # Title`, with `// ## Title` a level below, as the opening of Zeff 0.9.0 had taken it on the recommendation.

**In Zemp Lite 0.2.1 (21 November 2026, the owner's ruling; #203).** The outline and **Run section** come at Zemp Lite 0.2.1, on Zeff 0.8.0, in the interface's own reading of the comments `// # Title` and `// ## Title`; the interface takes the engine's sections (step 28 of Zeff 0.9.0) once Zemp moves to 0.9.0.

**Status (22 November 2026).** Built at step 28 of Zeff 0.9.0. A heading is a `//` comment alone on its line, of one or two `#`, a space and a title, standing between two statements of the script; a section runs to the next heading of its level or above. The language server gives the sections as the document's outline (`textDocument/documentSymbol`, a subsection inside its section) and folds each; the formatter keeps every heading as written, and a formatted script has the sections it had. The sample script's headings are in the chosen form, and section 3 of the Language Reference and section 12 of the Engine and Project Specification say what a section is (`docs/spec_changes.md`). The engine's reading counts a heading by where it stands, between two statements, where Zemp Lite 0.2.1's own reading asks for the first column; they agree on every formatted script, and Zemp takes the engine's when it moves to 0.9.0.

### 179. The last run's output shown when a project is opened

Raised on 16 November 2026. In the 0.1 prototype a reloaded project shows the script with no output until Run is pressed; a notebook keeps its outputs in the file.

| Option | For | Against |
| --- | --- | --- |
| A. On opening, show the last run's output from the run record and the engine's cache, each block labelled with when it ran, the stale rules of section 5 applying at once | The reader sees results immediately; nothing stored in the script; a stale result is still marked | Needs the run record and the cache kept in the browser's storage and on the desktop alike |
| B. Run the script automatically on opening | Always current | A long script runs unasked |
| C. As now: nothing until Run | Nothing to build | A project looks empty each time it is opened |

*Recommendation.* A, before Zemp Lite 1.0. Deadline: Zeff 0.11.0 (the desktop beta).

### 180. Run from a fresh session

Raised on 16 November 2026, from a notebook's "restart and run all". After console statements, a user wants proof that the script alone produces its results.

| Option | For | Against |
| --- | --- | --- |
| A. A ⌘K command **Run from a fresh session**: the console's statements are dropped, the "out of sync" flag is cleared, and the script runs from its first line; the cache may serve unchanged statements, since results are identical by the determinism contract | One clear action; fast | A user may expect every statement to be computed again |
| B. As A, with every statement computed again and the cache not read | The strongest proof | Slow on a long script, for a result the contract already guarantees |
| C. No command; Run already runs the whole script | Nothing to build | The state of the console after it is not obvious |

*Recommendation.* A, with B as a second command only if users ask. Deadline: Zeff 0.9.0.

**The owner's decision (21 November 2026, at the opening of release 0.9.0).** The owner said "go" to the checklist of `docs/releases/0.9.0.md` without changing its answers, so its recommendation is taken under his standing instruction of 12 November 2026, his to overturn. Option A: **Run from a fresh session** is a command of Zemp; the engine needs no step of its own for it.

### 181. Run from here to the end

Raised on 16 November 2026, as the partner of **Run to here** (#76). After editing a statement in the middle of a script, the user wants it and everything below it run, without the lines above.

| Option | For | Against |
| --- | --- | --- |
| A. **Run from here** as a command and a gutter action beside **Run to here**: the statements from the cursor to the end, those above served from the session or the cache | Completes #76; the stale label already names these lines ("⌘↵ to re-run lines 5–6") | One more gutter action |
| B. Nothing; re-running the stale lines covers the common case | Nothing to build | Lines below that are not stale cannot be run again in one action |

*Recommendation.* A, as an amendment of #76, with **Run to here**. Deadline: the planning of the Zemp 0.2 alpha if it is to come with it, otherwise Zeff 0.9.0.

**The owner's decision (21 November 2026, at the opening of release 0.9.0).** The owner said "go" to the checklist of `docs/releases/0.9.0.md` without changing its answers, so its recommendation is taken under his standing instruction of 12 November 2026, his to overturn. Option A, as an amendment of #76: **Run from here** beside **Run to here**, a command of Zemp with no engine step.

**The owner's answer (21 November 2026, given in the Zemp Lite session at step 1 of its release 0.2 and passed on in the shared notes; entered from a Zeff session, which alone edits this register).** His words to the seventeen questions of that checklist, each put with a recommendation: "Go, as recommended." Option A, as an amendment of #76: **Run from here** in ⌘K beside **Run to here**, at the Zemp 0.2 alpha.

### 182. Small conveniences, none before 1.0

Raised on 16 November 2026, the "may" group of the same review; recorded so that they are not lost.

1. Move the statement at the cursor up or down with the keyboard, as a whole statement.
2. Hide the output of one statement, kept as view state in `.zemp/views.json`, beside the **Outputs** switch.
3. A section heading (#178) shown as formatted text, with equations, in the Inline layout.
4. Reading a point's values by hovering over it in the plot viewer (to be checked against section 7 first).
5. The time a statement last ran, shown on hovering over its timing in the margin.

*Recommendation.* Keep as roadmap, each to be weighed after Zemp Lite 1.0. Deadline: 1.1.

### 183. The error's trail in the margin

Raised on 18 November 2026. Since Zeff 0.7.0 an error raised inside a function names its chain: ``= in: `returns(data: a frame of 0 rows × 13 variables)`, called at scripts/by_year.zeff:8:28``. In Zemp it is text under the failing line. A failure three calls deep is then followed by hand, file by file.

| Option | For | Against |
| --- | --- | --- |
| A. **The trail.** The failing statement's output lists each link of the chain from the script's line to the line that raised the error, each a link. Each file of the chain opens with a note on the lines the chain names and on no other: what the call was given, in the engine's words; and the error, unfolded. Nothing is run again. Where a link is a pass of a loop, the trail names the pass and the loop variable's value | Shows what the engine already prints; costs no run; the reader lands on the line that failed with what it was given | The pass is not in the engine's message at 0.7.0; it needs the engine to hand the chain over as fields with the pass (C22) |
| B. The engine's message under the failing line, as 0.1 shows it | Nothing to build | A chain through several files is followed by hand |

*Recommendation.* A, at Zemp's 0.2 alpha, beside the list of what the run said. Until the engine gives the pass, the trail shows the chain without it. Deadline: Zemp 0.2 alpha. (R14 of the margin editor's document.)

**The owner's answer (21 November 2026, given in the Zemp Lite session at step 1 of its release 0.2 and passed on in the shared notes; entered from a Zeff session, which alone edits this register).** His words to the seventeen questions of that checklist, each put with a recommendation: "Go, as recommended." Option A (R14), at the Zemp 0.2 alpha; the pass is added to the trail when the engine gives it (C22, placed at Zeff 0.9.0, step 27).

**Status (22 November 2026).** What the trail needs of the engine is built at step 27 of Zeff 0.9.0 (C22 of #175): the record of the error that stopped a run lists each link of the chain as fields, a call with what it was given and where it was called, a pass of a loop with its number, the number of passes and what the loop variable held; the statement that stopped the run carries the record in `zeff run --format json` and in the outcome of the browser build. The printed message is unchanged. It reaches Zemp Lite when Zemp moves to an engine of 0.9.0.

### 184. Searching the passes of a block, and drawing a field across them

Raised on 18 November 2026. #76's inspector gives one row per pass: for `for`, the loop variable, the rows after a `where`, the result and the time; for `iterate`, the state and what `done` compared. A debugger's conditional breakpoint answers "which pass went wrong?" without reading every row. A sampler's trace plot answers "is it still moving?" for `iterate`.

| Option | For | Against |
| --- | --- | --- |
| A. **Find the first pass that** failed · warned · kept no rows (view state; the pass found is the lowest-numbered, as the engine reports a failing loop). For a long block, the first three passes, the last three and any found, with *Show all n*. For `iterate`, each field's change from the pass before; one field of the state drawn by the engine against the pass, as view state, in no export | Replaces conditional breakpoints with facts the run record already holds; the drawing shows a run that crawls. The mockup's EM moves its log-likelihood by 4.1e-05 a pass at pass 500, while the share of the first component keeps rising | Each field's change, and the drawing from the run record's states, are new in the engine (C23, C27) |
| B. The inspector as #76 has it | Nothing more | A loop of 10,000 passes is read row by row |

*Recommendation.* A, before Zemp 1.0. The inspector itself stays at the 0.2 alpha, as #76 decided. Deadline: Zeff 0.9.0, so that C23 and C27 can be placed in an engine release before 1.0.0. (R16 of the margin editor's document.)

**The owner's decision (21 November 2026, at the opening of release 0.9.0).** The owner said "go" to the checklist of `docs/releases/0.9.0.md` without changing its answers, so its recommendation is taken under his standing instruction of 12 November 2026, his to overturn. Option A, before Zemp 1.0; the engine items it needs, C23 and C27 of #175, are placed at 0.11.0.

**The owner's answer (21 November 2026, given in the Zemp Lite session at step 1 of its release 0.2 and passed on in the shared notes; entered from a Zeff session, which alone edits this register).** His words to the seventeen questions of that checklist, each put with a recommendation: "Go, as recommended." Option A (R16), before 1.0, at the Zeff release that gives C23 and C27: placed at 0.11.0 by the opening of Zeff 0.9.0.

### 185. Opening a pass or a call, by replay

Raised on 18 November 2026. #76 rejects a breakpoint-and-step debugger. What is still missing is the inside of one call of a project function: each `let` of its body, for that call. A Zeff function cannot change anything outside itself (Language Reference, section 5.3), cannot call itself, and draws random numbers by its place and pass (Engine and Project Specification, section 8.3, point 3). So the engine can run one call, or one pass of a block, again from the arguments the run gave it, and get the same values to the last bit.

| Option | For | Against |
| --- | --- | --- |
| A. **Replay.** Opening a call shows the function's file with a note on every line for that call (each parameter, each `let` with its type and dimensions, the result), each unfolding to the engine's print. Opening a pass shows the block's body with its notes for that pass. A band names the call or pass. A replay changes no result, staleness, run number or export, and its values are those of the run. Where the arguments are no longer kept, the engine rebuilds them first and says so; a long replay shows progress and can be stopped | Everything step into shows, all at once and without stepping; no second way of running the interpreter, since the body runs as it ran with its bindings kept; testable bit for bit against the run | New engine work (C24); a replay costs what the call cost in the run |
| B. Breakpoints and stepping | What programmers expect | Rejected in #76, for the reasons there |
| C. Neither: the trail and the passes only | Nothing to build | The inside of a call stays unseen |

*Recommendation.* A, before Zemp 1.0, at the Zeff release that gives C24. Deadline: Zeff 0.9.0, for placing C24. (R17 of the margin editor's document.)

**The owner's decision (21 November 2026, at the opening of release 0.9.0).** The owner said "go" to the checklist of `docs/releases/0.9.0.md` without changing its answers, so its recommendation is taken under his standing instruction of 12 November 2026, his to overturn. Option A, before Zemp 1.0; C24 of #175 is placed at 0.11.0.

**The owner's answer (21 November 2026, given in the Zemp Lite session at step 1 of its release 0.2 and passed on in the shared notes; entered from a Zeff session, which alone edits this register).** His words to the seventeen questions of that checklist, each put with a recommendation: "Go, as recommended." Option A (R17), before 1.0, at the Zeff release that gives C24: placed at 0.11.0 by the opening of Zeff 0.9.0.

### 186. The console line inside a pass or a call

Raised on 18 November 2026. R's `recover()` and `browser()`, Jupyter's `%debug` and VS Code's debug console let an expression be typed where a failure happened. In a Zeff project the console line runs after the script only.

| Option | For | Against |
| --- | --- | --- |
| A. **While a pass or a call is open, the console line runs in its names.** For a pass: those of the script before the block, and the loop variable. For a call: the function's parameters and the bindings of its body, and nothing else, by the scope rule of section 5.3. A label says where (`in pass 13 · y = 2022`) | What `recover()` gives, without its danger: no binding can be changed, so the session cannot drift from its script. In the mockup, `waged.max(year)` typed in pass 13 answers 2021 | Evaluation in a replayed scope is new (C25) |
| B. The console at the top level only | Nothing to build | An expression at the point of failure needs a scratch script |

*Recommendation.* A, with #185. Deadline: as #185. (R18 of the margin editor's document.)

**The owner's decision (21 November 2026, at the opening of release 0.9.0).** The owner said "go" to the checklist of `docs/releases/0.9.0.md` without changing its answers, so its recommendation is taken under his standing instruction of 12 November 2026, his to overturn. Option A, with #185; C25 of #175 is placed at 0.11.0.

**The owner's answer (21 November 2026, given in the Zemp Lite session at step 1 of its release 0.2 and passed on in the shared notes; entered from a Zeff session, which alone edits this register).** His words to the seventeen questions of that checklist, each put with a recommendation: "Go, as recommended." Option A (R18), before 1.0, at the Zeff release that gives C25: placed at 0.11.0 by the opening of Zeff 0.9.0.

### 187. Where the time and memory of a run went

Raised on 18 November 2026. The run record shows the time of the whole run and the memory `zeff run -v` writes. RStudio has profvis, and Jupyter and VS Code have profilers.

| Option | For | Against |
| --- | --- | --- |
| A. **Two lists in the run record:** the statements by the time they took, each a link to its lines, with its share of the run; and the bindings by the memory they hold. The interface computes no time: it reads the engine's | Rests on what exists. The worker's step messages carry each statement's time, which the 0.1 prototype shows; `zeff run -v` writes the memory (C26). In the mockup, the EM takes 272 ms of a 289 ms run | Small; adds to the run record's screen |
| B. The time of the whole run, as now | Nothing to build | The slow statement is found by hand |

*Recommendation.* A. Whether it comes with the run record at the 0.2 alpha or at 1.0 is the owner's choice. Deadline: Zemp 0.2 alpha. (R19 of the margin editor's document.)

**The owner's answer (21 November 2026, given in the Zemp Lite session at step 1 of its release 0.2 and passed on in the shared notes; entered from a Zeff session, which alone edits this register).** His words to the seventeen questions of that checklist, each put with a recommendation: "Go, as recommended." Option A (R19), at the Zemp 0.2 alpha, with the run's facts: until the engine reports each statement's time (C26), the interface's own measure between the engine's two messages, said to be so.

### 188. `report()` says when its columns cannot be compared

Raised on 18 November 2026. A table of several fits invites the reader to read a coefficient across its columns. That reading is sound only when the columns explain the same outcome, with the same estimator, on the same rows. Today `report()` prints an N row, which shows a different count but not different rows of the same count, and says nothing when the outcome or the estimator differs.

| Option | For | Against |
| --- | --- | --- |
| A. **A note in the table's footer**, for each column that differs from the column before it: `(4) note: not directly comparable with (3): (3) explains log(wage), (4) explains wage`; `(5) note: not directly comparable with (4): (4) is least squares, (5) is logistic, so their coefficients are on different scales`; `(2) note: 14,902 rows, against 15,366 in (1): 464 rows have NA in tenure, which only (2) holds`. Nothing is added when the columns agree. Fits on different frames are said to be so, by count | It reaches every export of the table (text, LaTeX, Word, a paper), as every note does; it states facts of the calls and never a cause; a table of agreeing columns is unchanged | The rows check needs each fit to keep which rows it used (a bitmap over its frame, or a hash), and the NA reason needs the variables that dropped them; each estimator family words its own outcome |
| B. A warning at run time instead (a W code) | Seen while working | A column on other rows is often on purpose (a robustness check); a warning would nag, and a warning does not reach the table |
| C. Nothing: the N row is enough | No work | Equal counts are not the same rows; a different outcome or estimator goes unsaid |

*Recommendation.* A, each column against the one before it, since a table is read left to right as a run of specifications. Deadline: 0.10.0, when #141 moves `report()` into `publish` and its print is touched anyway.

### 189. A coefficient plot of several fits

Raised on 18 November 2026. `fit.coef_plot()` draws one fit. Showing a coefficient moving across specifications needs the fits on one axis, which separate plots cannot give. It is among the commonest figures of applied papers (Stata's user-written `coefplot`; R's `modelsummary::modelplot`).

| Option | For | Against |
| --- | --- | --- |
| A. **`coef_plot([fits])`, a function in `plot`**, beside the method: one row per term, each fit's estimate and interval side by side within the row with a shape for each fit, nought dashed; the intervals are each fit's own, the layer recording their level as #118 (F11) does for one fit; `terms:` to choose and order the terms, `labels:` as in `report`; a term a fit does not hold left out of its row. Fits with different outcomes or estimators are refused with an error naming the difference, since one axis would itself compare them; fits on different rows are drawn, with the note of #188 in the caption | The comparison of fits as a figure, written in the script, exported and set in a paper; the same rules as #188, so the table and the figure never disagree | A new name, which must come before the names freeze at 1.0 |
| B. A method on the table, `report([...]).plot()` | The figure tied to its table | A table is an odd owner for a figure; harder to find; refusing columns that cannot share an axis is less natural there |
| C. Nothing: one `coef_plot()` per fit, side by side | No work | Separate axes cannot show a coefficient move |

*Recommendation.* A. Deadline: 0.10.0 with #188, or the first release before 1.0 with room, the owner's choice; the name before 1.0 in any case. A mockup of it, drawn in the engine's style from the released engine's estimates and intervals for three fits on the shipped `wages` data, is screen 17 of `~/dev/handbook/zemp-lite-margin-editor/screenshots/`.

### 190. Building the layout engine in stages

Raised on 18 November 2026. #121 (D4) places Zeff's own layered layout engine at 0.10.0, for `g.plot()`, and describes it whole: network-simplex layers, median crossing reduction, Brandes–Köpf coordinates, spline or orthogonal routing, grid placement for templates. It is the largest piece of engine work in the diagram plan, and only part of it is needed at 0.10.0: causal graphs are small (rarely over 30 nodes), have no groups, and are drawn with straight or curved edges.

| Option | For | Against |
| --- | --- | --- |
| A. **In three stages, each with the release that first needs it.** At 0.10.0, for `g.plot()`: cycle-free layered layout (network simplex, median crossing reduction with fixed sweeps, Brandes–Köpf), straight and spline edges, `layout:`, `at:`, `direction:`, `same_rank:`, `spacing:`, label boxes. At 1.2, for `diagram { }` and `sample_flow`: cycles (flowcharts loop), orthogonal routing, groups (clusters), ports on box sides, side boxes for exclusions. At 1.12, for SEM: the grid templates of the proposal's section 6.4, indicator fans, covariance arcs outside the structural paths, label placement on crowded arrows | Each stage ships with a user of it and is tested on real figures; 0.10.0 carries the least risk; nothing is pulled forward | The whole engine is not proven until 1.12; a choice made at 0.10.0 (the graph form, #191) must allow the later stages |
| B. The whole engine at 0.10.0, as #121 reads | Built once | The largest risk on the release that also brings the causal-graph functions; orthogonal routing and templates built with no user to test them |

*Recommendation.* A. Nothing in #121's releases moves; the stages are its D4 spread over the releases its other rows already name. Deadline: 0.10.0.

### 191. One graph form under every diagram

Raised on 18 November 2026. #121 has one layout engine, but four values reach it: a `Dag`, a `Diagram`, a `Model` or fitted `SemFit`, and a sample flow. If each is laid out from its own structure, the viewer's interactions (hover, trace a path, the table view for screen readers), the text forms and the Mermaid and DOT exports are written four times.

| Option | For | Against |
| --- | --- | --- |
| A. **Every diagram is first lowered to one graph form**: nodes (name, shape, label lines, kind: observed, latent, constant, error, process, decision …), edges (from, to, heads, kind: path, loading, covariance, moderation onto an edge), groups, and ports; each node and edge carrying the stable element name the primitives already allow (`node:engage`, `edge:engage->perform`). Layout reads only this form; the scene of the plot keeps it beside the laid-out primitives (a section of the scene format, versioned with it), so the viewer finds what was pointed at, the table view lists it, and Mermaid and DOT are written from it | One mechanism for four kinds of figure; the viewer and the exports are written once; the margin editor's note of a diagram (`Dag · 6 nodes, 7 edges`) and the Plots list read it; a screen reader gets the same structure a sighted reader sees | A section added to the scene format, which is versioned (Language Reference, section 7.6), so it should be fixed at 0.10.0 with the first diagram |
| B. Each value laid out from its own structure | No new form | Four of everything; exports and the table view drift apart |

*Recommendation.* A, fixed at 0.10.0 with `g.plot()`, so that the later kinds add node and edge kinds and nothing else. Deadline: 0.10.0.

### 192. Testing layouts against known figures

Raised on 18 November 2026. The testing handbook's snapshot tests hold a diagram's bytes still, but a snapshot cannot say whether a layout is good, only that it has not changed. A layout engine written from papers needs a measure of quality from its first release.

| Option | For | Against |
| --- | --- | --- |
| A. **A suite of reference figures, measured.** Causal graphs: dagitty's example graphs. SEM: the Holzinger–Swineford CFA and Bollen's Political Democracy model (the two that lavaan and semPlot document). Flowcharts: a CONSORT trial and a sample flow of the shipped `wages` data. Each is measured on every change: edge crossings, overlapping nodes or labels (none allowed), the smallest text at the export size (8 pt or more, as the plot rules set), and the figure's size. Graphviz's `dot` is run in the tests alone as a yardstick for crossings: the layout must cross no more often than `dot` on the same graph, or the case says why. Graphviz is an external program in the test job, never linked or shipped | A failure says what got worse; the yardstick is the tool users know; the licence rule, which concerns what is linked into the engine or the apps, is kept | A program in the test job that is not under a permissive licence (Graphviz is EPL-1.0), which the owner may prefer to keep out of the repository's tooling too |
| B. The same suite and measures, with fixed targets written down once (the crossings `dot` gives today, recorded as numbers) and no Graphviz in the tests | Nothing EPL anywhere | Targets that do not move when a case is added or changed; recomputed by hand |
| C. Snapshots only | Nothing to build | A worse layout passes |

*Recommendation.* A, or B if the owner wants nothing EPL even in tooling; the licence question is his. Deadline: 0.10.0.

### 193. Dragged positions in Zemp Lite: Copy as Zeff

Raised on 18 November 2026. #121 (D2) makes dragged positions view state and lets **Add to script** write them as `layout:` or `at:`, an export using only positions the script holds. Zemp Lite has no **Add to script** (Zemp Specification, section 2). So in Lite a layout arranged by dragging can never reach an export unless the user types the grid by hand. #114 already gives Lite **Copy as Zeff** in the data, plot and variable viewers: the equivalent statement goes to the clipboard, and the script changes only when the user pastes it.

| Option | For | Against |
| --- | --- | --- |
| A. **Copy as Zeff extends to dragged positions** in every diagram viewer (`g.plot()`, `d.plot()`, `path_plot`): the `layout:` grid of the positions shown, as text, on the clipboard; `at:` when the positions do not fall on a grid | The rule of #114 unchanged: nothing written into the script; the user pastes; a Lite user can publish a diagram laid out by hand | One more kind of statement Copy as Zeff produces |
| B. Nothing: a Lite user types `layout:` by hand, reading the positions from the viewer | No change | Dragging in Lite is play; the figure exported is always the automatic one |

*Recommendation.* A. On adoption, the Copy as Zeff row of the Zemp Specification's section 2 and the text of #121's D2 name it. Deadline: Zemp 0.4, with Zeff 0.10.0, when the shared canvas (D10) is built.

### 194. Statistical guard rails, written into the diagram rules

Raised on 18 November 2026. Several are already decided in one place or another; one is new. Collecting them where the diagrams are specified keeps the viewer from drifting.

| Rule | Where it stands | Proposed |
| --- | --- | --- |
| 1. **No edge suggested by the data.** No viewer, option or function draws an arrow a test points to, on a causal graph or an SEM diagram. A graph states assumptions; drawing the arrows a test suggests is choosing the graph from the data | New. The owner's Margin plan of 17 November proposed "Show what the data suggest" on a causal graph | Adopt, as a rule of section 7.3 of the Language Reference and of the viewer |
| 2. Implied independencies tested with their estimate and interval, not a p value alone | #158 recommends the estimate, interval and p, with `p_adjust` | Keep with #158 |
| 3. The modification-index overlay (`mi:`) never on by default, and labelled as a diagnosis | #121, D6 (the proposal's section 6.3) | Keep |
| 4. Non-significant paths shown by default (`nonsignificant: show`) | #121, D6 | Keep |
| 5. Stars only beside an estimate: `show:` in `path_plot` refuses `stars` without `estimate` | #122 leaves out "stars without effect sizes" for plots; `path_plot` does not yet say so | Adopt for `path_plot` at 1.12 |

*Recommendation.* Adopt rules 1 and 5; keep 2 to 4 where they are. Deadline: 0.10.0 for rule 1, 1.12 for rule 5.

### 195. Slopes and turning points: `margins` through a transformed term

Raised on 20 November 2026 from the owner's questions on polynomial regression, with the addendum to #164. `margins` arrives at 0.9.0 (#88), before `poly` at 1.2, and its questions left for that release (the effect of a 0/1 regressor, regressors fixed or sampled, what `at:` holds the others at) do not yet include this one.

Under the orthogonal form the coefficients are not the polynomial's coefficients on `x`. So the turning point cannot be had from `derive` as it can today for the raw powers (`` m.derive(peak = -exper / (2 * `exper^2`)) ``): the basis constants live with the fit, not among its terms. With orthogonal terms, the slope at a value and the turning point must come from somewhere else. The natural place is `margins`, which arrives at 0.9.0, before `poly` at 1.2. Its questions left for that release (the effect of a 0/1 regressor, regressors fixed or sampled, what `at:` holds the others at) do not yet include this one.

| Option | For | Against |
| --- | --- | --- |
| A. **`margins` differentiates through every term that is a function of the variable asked for**: `exper` and `exper^2` today, giving the slope β₁ + 2β₂·exper at the values of `at:`; the interactions it enters; `poly(exper, k)` and `spline(exper, df:)` when they arrive | The slope of a curve, which is what an applied reader wants from a polynomial; the same as Stata's `margins` after `c.exper##c.exper` and R's `marginaleffects` after `I(exper^2)` or `poly()`; possible because Zeff's formula keeps `exper^2` as a function of `exper` | The derivative is taken through the formula's terms, which the model matrix must record for each column; a power computed beforehand into its own column (`exper2`) stays a separate regressor, as in Stata, and the page must say so |
| B. `margins` treats each column of the model matrix as a separate regressor | Simpler | A wrong slope for a quadratic, silently; this is the trap of a generated `exper2` in Stata |
| C. `margins` refuses a variable that appears inside a transformed term | Never wrong | Refuses the commonest use of marginal effects after a linear model |

*Recommendation.* A, settled when 0.9.0 opens with the other questions on `margins`. Deadline: 0.9.0. When #164 is built, `poly`'s page gives the slope at a value from `margins`. It gives the turning point from the raw form (`raw: true`, the same fit) through `derive`, or from `margins` by the value where the slope is nought.

**The owner's decision (21 November 2026, at the opening of release 0.9.0).** The owner said "go" to the checklist of `docs/releases/0.9.0.md` without changing its answers, so its recommendation is taken under his standing instruction of 12 November 2026, his to overturn. Option A: `margins` differentiates through every term that is a function of the variable asked for (`exper` and `exper^2` today; `poly` and `spline` when they arrive); a power computed beforehand into its own column stays a separate regressor, and the page says so. Built at 0.9.0, steps 10 to 12, with the three definitions left by #88 for this opening (see #88).

**The owner's answer (21 November 2026, given in the Zemp Lite session at step 1 of its release 0.2 and passed on in the shared notes; entered from a Zeff session, which alone edits this register).** His words to the seventeen questions of that checklist, each put with a recommendation: "Go, as recommended." Option A, confirmed in the same answer.

### 196. Two messages R users meet: `I()` and `poly` in a formula, and a function of a later release

Raised on 20 November 2026 from the owner's questions on polynomial regression. No new function is proposed.

Seen with the engine as built on the morning of 20 November 2026, and the same on the release of 0.8.0, inside a project, on `dataset(wages).where(year == 2019)`:

```
w.reg(log(wage) ~ educ + exper + I(exper^2))
error[Z0103]: no function is called `I`
  = help: did you mean `PI`?

w.reg(log(wage) ~ educ + poly(exper, 2))
error[Z0103]: no function is called `poly`
  = help: `zeff help` lists the functions of this release
```

The first is the commonest line an R user types for a quadratic, and the suggestion misleads. The second gives no route to what Zeff offers.

| Option | For | Against |
| --- | --- | --- |
| A. **Inside a formula, Z0103's help for `I(...)` says that Zeff works out arithmetic in a formula, so no `I()` is needed, and shows the term without it (`exper^2`). For `poly(x, k)` it writes the powers out (`exper + exper^2`), names `category(x, contrast: poly)` for an ordered category, and names `poly`'s release once #164 is decided** | An R user is shown the Zeff form at once; two help texts with their snapshots and explained pages; no name added, so within the owner's instruction of 11 November 2026 that nothing is added before 1.0 unless it is necessary | Two special cases in the message of Z0103 |
| B. Only the "did you mean" corrected (no suggestion for a one-letter name) | Smaller | The R user still learns nothing about `I()` or `poly` |
| C. As now | No work | The first message an R user meets misleads |

*Recommendation.* A, at 0.9.0. Deadline: 0.9.0. The suggestion of `PI` for `I` also suggests that "did you mean" should not offer a name for a single letter; that is for the Zeff session to judge.

*Seen on the way.* `w.logistic(union ~ educ)` gives the same generic help ("`zeff help` lists the functions of this release"). Yet `zeff help logistic` already answers "`logistic` arrives with release 0.9.0, in the `glm` package". Z0103 inside a script could give the same answer for any catalogued name of a later release. It is listed here because the alpha's users, who come from R and Stata, will meet it first.

**The owner's decision (21 November 2026, at the opening of release 0.9.0).** The owner said "go" to the checklist of `docs/releases/0.9.0.md` without changing its answers, so its recommendation is taken under his standing instruction of 12 November 2026, his to overturn. Option A: inside a formula, Z0103's help for `I(...)` and `poly(...)`, a catalogued name of a later release named with its release, and no "did you mean" offered for a name of one letter; built at step 35.

**The owner's answer (21 November 2026, given in the Zemp Lite session at step 1 of its release 0.2 and passed on in the shared notes; entered from a Zeff session, which alone edits this register).** His words to the seventeen questions of that checklist, each put with a recommendation: "Go, as recommended." Option A, confirmed in the same answer.

**Status (23 November 2026).** Built at step 35 of Zeff 0.9.0, as decided. The three messages quoted above now read: for `I(exper^2)`, "arithmetic in a formula is worked out before fitting, so `I()` is not needed: write `exper^2`"; for `poly(exper, 2)`, "write the powers out: `exper + exper^2`; in a model with an intercept the fitted values are those of R's `poly`, and the coefficients those of `raw = TRUE`", with a second line naming `category(x, contrast: poly)`; and a catalogued function of a later release, "`did` arrives with a release after 1.0, in the `quasi` package; `zeff help did` says what it will do". Readings taken on the way, the owner's to overturn: (1) the advice is made from what the script wrote, and a term whose operators would be the formula's own outside brackets is shown in brackets, `I(exper * hours)` as `(exper * hours)`; (2) the degree of `poly` is read from its second argument or from `degree:`, two where it cannot be read; the sentence on the fitted values is held to R's `lm` with `poly` in the regression reference; the release of `poly` is not named, #164 being open; (3) a name the catalogue has only as an option of a function (`poisson`, a family of `glm`) is said to arrive as an option; a name a release took out keeps its rewrite, and a function of the project not brought in keeps its advice, both before this; (4) a name of one letter is offered no near name anywhere the engine offers one (functions, variables, bindings, the words of options, `zeff help`), and the same letter in the other case is still offered. Two questions met on the way are #212 and #213. Deadline for the owner to overturn: 1.0.

### 197. The plots of a run handed to an interface

Raised on 15 November 2026 by the Zemp Lite session at the start of its release 0.1, and entered on 20 November 2026. The browser build of the engine (`zeff-web`) returns what a run printed and said and the frames it kept; it keeps no plot, and nothing draws one at a size. This was so at 0.7.0 and is still so at 0.8.0. Zemp 0.1 works round it: it adds an `export` of each plot to the text it sends and reads the file back from its own file system, so that a redraw at another size is a new run of the script. No step of the checklist of 0.8.0 named the proper hand-over, and no row of section 14 names it.

What an interface needs: the browser build keeping the plots of a run as it keeps frames; a call that lays one out at a size (in points) and a theme and returns SVG, without a new run; the scene as data, for inspecting a mark, focusing and comparing on shared axes (Zemp Specification, section 7, with its target of 16 ms in section 14). This is item C16 of #175. Also useful to an interface and absent: the language services through `zeff-web`, a file of the project read back, and which bindings hold a plot.

| Option | For | Against |
| --- | --- | --- |
| A. An engine step at 0.9.0, beside the launcher: the plots of a run kept, a plot laid out at a size and theme without a run, the scene handed over; the smaller items with it where they are small | The plot viewer of the Zemp alpha rests on it; the pieces (the layout and the SVG writer) exist since 0.7.0 and need only a door | A step that the row of 0.9.0 does not name, in a release that already holds the launcher |
| B. With the release of Zeff that Zemp's plot viewer is planned on, named when the Zemp 0.2 alpha is planned | Placed by the release that uses it | The alpha's Plots place redraws by running the script again until then |
| C. The work-round stays | No engine work | Every redraw and every resize is a run; no inspecting of a mark |

*Recommendation.* A, since the alpha is the first thing outside readers see and its Plots place is part of the adopted editor (#174). The placing is a priority and the owner's. Deadline: the opening of Zeff 0.9.0.

**The owner's decision (21 November 2026, at the opening of release 0.9.0).** The owner said "go" to the checklist of `docs/releases/0.9.0.md` without changing its answers, so its recommendation is taken under his standing instruction of 12 November 2026, his to overturn. Option A: an engine step at 0.9.0 (step 25) by which the browser build keeps the plots of a run, lays one out at a size and theme without a new run, and hands over the scene (C16 of #175); the owner's priority, as recommended.

**Status (22 November 2026, release 0.9.0, step 25).** Built as decided. The browser build keeps each plot a top-level statement of a run makes and lists them in the run's outcome (`plots`: number, name, line, kind, theme); the message `plot` lays one out on a page of a width and a height in points, under its own theme or `journal`, `minimal` or `slide`, and answers with the SVG, with no new run; the message `scene` gives the engine's JSON of its scene. At the points of a page `export` writes, the SVG is the same byte for byte, so the work-round of an added `export` is no longer needed. A file of the project is read back (`file`). Not built with it: the language services through the worker. A redraw takes under 16 ms up to about 20,000 points and 65 ms at 100,000 (`docs/decisions.md` of this day).

### 198. The small-sample factor of robust and clustered errors on a likelihood fit

Raised on 21 November 2026 at release 0.9.0, step 7b, when `glm` took `se:` and `cluster:`. The sandwich itself is not in question: the scores summed, or summed within clusters, between the inverse of the information. What multiplies it differs between the packages on a model fitted by maximum likelihood. With N observations, k estimates and G clusters:

| | Robust | Clustered |
| --- | --- | --- |
| The variance layer, as for `reg` (HC1, CR1) | N/(N − k) | G/(G − 1) · (N − 1)/(N − k) |
| R `sandwich`, `type = "HC1"`; `fixest::feglm`; Python `statsmodels` | N/(N − k) | G/(G − 1) · (N − 1)/(N − k) |
| R `sandwich::vcovCL` left to itself on a `glm` fit (`type = "HC0"`) | none (`sandwich()`) | G/(G − 1) |
| Stata `logit` and `probit`, `vce(robust)` and `vce(cluster id)` | N/(N − 1) | G/(G − 1) |

On the wages (N = 18,390, k = 4) the standard errors differ in their fifth digit; in a sample of 100 with 10 estimates, by five per cent.

| Option | For | Against |
| --- | --- | --- |
| A. The variance layer's factors for every model: `se: robust` is HC1 and `cluster:` is CR1 on a `glm` fit as on a `reg` fit | What section 8.3, point 2, of the Engine Specification says: "`se:`, `cluster:` and `weights:` work identically for every model"; the labels the summary prints, HC1 and CR1, mean these factors; the larger, more cautious errors; `fixest` and `statsmodels` do the same | A user who copies `logit y x, vce(cluster id)` from Stata, or `vcovCL(m, ~id)` from R, sees standard errors larger by √((N − 1)/(N − k)) |
| B. Stata's factors on a likelihood fit, N/(N − 1) and G/(G − 1) | Equal to Stata's `logit`, and to R's `vcovCL` left to itself when clustered | Two rules in the variance layer, by the kind of model; "HC1" and "CR1" would not mean on a `glm` fit what they mean on a `reg` fit; no degrees-of-freedom correction where the sample is small, which is where it matters |
| C. A, with Stata's factors offered as a word of an option after 1.0 | Every package's number can be had (the owner's rule of #98 and #99) | A new option, not in the catalogue; after the freeze |

*Recommendation.* A now, as the specification says, with the difference and its size written in the help page of `glm` under "Coming from R or Stata" and held by a test (the clustered variance times (N − k)/(N − 1) equals R's `vcovCL` left to itself, within 1e-8). C can be raised when the catalogue next opens. Deadline: 1.0, since changing a default after 1.0 would change published numbers.

**Decided (21 November 2026, release 0.9.0, step 7b).** A, taken as recommended under the owner's standing instruction of 12 November 2026 ("decisions can be taken as per your recommendations"), and announced to him with the step; his to overturn before 1.0.

### 199. Analytic weights on a binary model

Raised on 21 November 2026 at release 0.9.0, step 7c, when `glm` took `weights:` and `weight_type:`. Section 6 of the Language Reference gives three kinds of weight to regression-type models. Frequency weights (a row stands for several identical observations) and sampling weights (probability weights of a survey) mean on a binary model what they mean on a linear one. Analytic weights say how precisely each outcome was measured: in a linear model they scale the error variance, σ²/w, and the estimate of σ² makes their scale immaterial. A 0 or 1 has no σ²: its variance is fixed by its mean, so there is nothing for an analytic weight to scale, and a classical variance computed with such weights would change with their scale (doubling every weight would halve every variance).

| Option | For | Against |
| --- | --- | --- |
| A. Refuse `weight_type: analytic` on `glm` with `family: binomial` (Z0410), the message naming `frequency` and `sampling` | No number that depends on an arbitrary scale; Stata does the same (`logit` takes `fweight`, `pweight` and `iweight`, not `aweight`) | A user with weights that are neither counts nor survey weights must say which they are nearest to |
| B. Take them as R's `weights =`: the likelihood weighted, the classical variance the inverse of the weighted information, the weights as given | Equal to R's `glm(weights = w)` for any w | R itself warns of weights that are not whole numbers on a binomial; the standard errors depend on the scale of the weights; three kinds of weight would be two computations |
| C. Take them as sampling weights with a note | Something is fitted | Silently another kind than was written |

*Recommendation.* A. A family with a dispersion to estimate (the gamma, the quasi-families of release 1.7) takes analytic weights when it arrives; the refusal names the family. Deadline: 1.0.

**Decided (21 November 2026, release 0.9.0, step 7c).** A, taken as recommended under the owner's standing instruction of 12 November 2026, and announced to him with the step; his to overturn before 1.0.

### 200. What `predict` and `residuals` give a `glm` fit unless told

Raised on 21 November 2026 at release 0.9.0, step 8a. A model fitted by maximum likelihood has two scales of prediction, the linear predictor and the mean of the outcome, and four kinds of residual. Both methods take `type:` (one function with an option, #97; every kind R gives, #98 and #99): `predict(newdata:, type: response | link)` and `residuals(type: response | pearson | deviance | working)`. The question is the default.

| | `predict` unless told | `residuals` unless told |
| --- | --- | --- |
| R (`predict.glm`, `residuals.glm`) | the linear predictor | the deviance residuals |
| Stata (`predict` after `logit`) | the probability | none: the kind is always named |
| Python `statsmodels` | the mean | none: one attribute for each kind |
| R `marginaleffects::predictions` | the response | not applicable |

| Option | For | Against |
| --- | --- | --- |
| A. The scale of the outcome for both: `response` | `fitted()`, `predict()` and `residuals()` agree on every model: a prediction is what `fitted` would give that row, and the outcome is the fitted value plus the residual, for `reg` and `glm` alike; a probability is what a reader of `predict` expects, and R's default is a known trap; `margins` works on this scale too | An R user's `residuals(m)` gives deviance residuals, and here gives the response's; `predict(m)` in R gives log-odds |
| B. R's defaults: `link` and `deviance` | What an R script prints | `predict` and `fitted` on different scales; the identity outcome = fitted + residual lost for `glm` |
| C. No default for `residuals` on a `glm` fit: the kind must be named | Nothing silent | A required option on one kind of fit and not another |

*Recommendation.* A, with the page of each method saying what R gives and which word gives it here. Deadline: 1.0.

**Decided (21 November 2026, release 0.9.0, step 8a).** A, taken as recommended under the owner's standing instruction of 12 November 2026, and announced to him with the step; his to overturn before 1.0. On a least-squares fit `predict` takes either word (it has one scale) and `residuals` takes `response` alone, another word being refused (Z0401): the weighted kinds of a regression's residuals are not built.

### 201. A `glm` fit in `report` and under `summary(standardized:)`

Raised on 21 November 2026 at release 0.9.0, step 8c, when the tables of the result interface took a `glm` fit. Two questions of what is printed.

**(a) The `R²` row of `report`.** The row is among the default statistics, and a binary model has no R² of least squares.

| Option | For | Against |
| --- | --- | --- |
| A. McFadden's R², marked: `0.007 (McFadden)` | The row is not blank in a table of binary models, and says which R² it is, as `(within)` does for a model with fixed effects; Stata's tables print it as "Pseudo R2" | A reader may still set it beside a regression's R² in a mixed table |
| B. Blank for a `glm` fit, as for a fit by instrumental variables | No number that might be misread | A table of two logits has an empty row by default |
| C. A row of its own, with a new word in `stats:` | The cleanest table | A new option word, after the freeze |

*Recommendation.* A. **Decided (21 November 2026, step 8c):** A, under the owner's standing instruction; his to overturn before 1.0.

**(b) `summary(standardized: true)` on a `glm` fit.** For a regression the column `beta` is the estimate times the standard deviation of its regressor over that of the outcome. For a binary model that quotient is not a standard quantity: the outcome's standard deviation is that of a 0 or 1, not of what the coefficients are on the scale of.

| Option | For | Against |
| --- | --- | --- |
| A. Standardized by the regressor alone: the estimate times the standard deviation of its regressor, the change in the log-odds (or the normal score) for one standard deviation | Simple, and what Stata's `listcoef` calls bStdX (Long and Freese); needs no assumption about a latent outcome | Not what `beta` means for a regression; the column needs a heading of its own |
| B. Fully standardized through the latent outcome, whose variance is that of the linear predictor plus π²/3 (logit) or 1 (probit) | The nearest thing to a regression's beta; `listcoef`'s bStdXY | Rests on the latent-variable reading of the model; changes with the link |
| C. Refused on a `glm` fit, the help saying to standardize the regressors before fitting | No number whose meaning is unsettled | An option that one kind of fit does not take |

*Recommendation.* A, with the heading `beta (x)`, when the catalogue next opens; until the owner chooses, C is what the engine does (a refusal settles nothing). Deadline: 1.0.

**Status (21 November 2026).** (b) is open: `standardized: true` is refused on a `glm` fit (Z0401).

### 202. `margins`: what the definitions of #88 leave unsaid

Raised on 21 November 2026 at release 0.9.0, step 11a, when `margins` was built for `reg`. The definitions decided at the opening of the release (#88, (a) to (d)) say what an effect is, that the regressors are held fixed, and what `type:` and `at:` mean. Five further points had to be settled to print a table.

**(a) The average under weights.** A weighted fit stands for a population, or for more rows than the frame holds.

| Option | For | Against |
| --- | --- | --- |
| A. The average over the rows is weighted by the fit's weights, of whatever kind | The average effect in the population the weights describe; with frequency weights, the average over the observations the rows stand for; what Stata's `margins` does | `marginaleffects` weights only when `wts = TRUE` is given, so its default differs |
| B. Unweighted always | `marginaleffects`' default | After frequency weights the average is over the rows of the file, not over the observations; after sampling weights it describes the sample, not the population the fit describes |

*Recommendation.* A. **Decided (21 November 2026, step 11a):** A, under the standing instruction.

**(b) `margins` with no variable named.** The catalogue's row 260 says "marginal effects and adjusted predictions", and its usage has only `vars`, `at:` and `type:`.

| Option | For | Against |
| --- | --- | --- |
| A. The adjusted prediction: the average of the predictions (`type: ame`) or the prediction at the means (`type: mem`), the variables of `at:` set to its values | Gives the catalogue's adjusted predictions a home without a new argument; Stata's reading (`margins` alone, `margins, at(...)`), where `dydx()` asks for effects | The effects of every variable need the variables listed |
| B. The effects of every variable of the model | `marginaleffects::avg_slopes(m)`; one call for the whole table | Adjusted predictions then need an argument the catalogue does not have |
| C. Refused: `vars` is required | Nothing to explain | The row's description promises adjusted predictions, and nothing gives them |

*Recommendation.* A, built at step 11b with `at:`. Until then `vars` is required. **Decided (21 November 2026, step 11a):** A, under the standing instruction; built at step 11b.

**(c) The reference distribution after `reg`.**

| Option | For | Against |
| --- | --- | --- |
| A. Student's t on the fit's degrees of freedom (N − k, or G − 1 when clustered), as the fit's own table and `derive` | An effect that is one coefficient has the coefficient's own p-value and interval; Stata's `margins` after `regress` | `marginaleffects` uses the normal distribution unless `df =` is given |
| B. The normal distribution | `marginaleffects`' default | The effect of `educ` in `y ~ educ` would have another interval than the coefficient of `educ` |

*Recommendation.* A. After `glm` the reference is the normal distribution, as the fit's own table has it (step 12). **Decided (21 November 2026, step 11a):** A, under the standing instruction.

**(d) The printed table.** One row for each effect: the variable; what is compared (`dy/dx` for a slope, `female − male` for a 0/1 variable with value labels, `true − false` for a Bool, `1 − 0` otherwise, `Mining − Agriculture` for a level against the first); then the columns of a fit's own table. Above it, the model, what the average is over, and the kind of standard error; under it, a line for each slope taken through more than the variable's own term (`the slope of exper is taken through exper and exper^2`). The result is a `TestResult`, read by `estimates`, `ci` and `p_values`; a row is named as the fit names the term (`exper`, `industry=Mining`), so that the names of a level's coefficient and of its effect are one. Alternatives: the term's name in the first column (wider by the length of the longest level, which wraps the table of a category of twelve levels); `effect` for the heading `contrast`. *Recommendation.* As built. **Shown to the owner at step 11a**, his to change.

**(e) A category under `type: mem`.** Definition (c) of #88 says "at the means of the regressors"; a category has no mean.

| Option | For | Against |
| --- | --- | --- |
| A. Each level's indicator at its mean, the share of the rows at that level; a 0/1 variable and a Bool at their means | "At the means" taken as written; Stata's `atmeans`; no row of the data need look like the point | The point is no observable case: 46 per cent female |
| B. A category at its commonest level, numbers at their means | `marginaleffects`' `newdata = "mean"`; an observable case | Not the means; changes with a tie between levels; a 0/1 number is treated otherwise than a Bool |

*Recommendation.* A, built at step 11b. **Decided (21 November 2026, step 11a):** A, under the standing instruction; built at step 11b.

**Status (21 November 2026).** (a), (c) and (d) are built; (b) and (e) are built at step 11b. Deadline for the owner to overturn any: 1.0.

**Status (22 November 2026, step 11b).** (b) and (e) are built as decided. Of (b): after `fe:` the adjusted prediction is refused, since it would need the level of each fixed effect; the effects of variables are given as before. Of (e): a Bool and a 0/1 number are held at their means, the share of ones, and under weights the means and the shares are weighted. Two further points met and settled the same way, each the owner's to overturn: a 0/1 variable or a category both asked for and held by `at:` is refused, its effect being the difference between its values, while the slope of a number may be asked where the number itself is held; and several values of one variable in `at:` are refused for `reg` until step 12 builds them.

**Status (22 November 2026, step 12a).** After `glm` the table is a z table, with the p-values and the intervals of the normal distribution, as (c) said and as the fit's own table has them; every effect and prediction is on the scale of the probability, and the line above the table says so. At the means the probability is taken at the one point of the means (the link applied to the linear predictor of the means), as Stata's `atmeans` has it, not averaged over the levels of the categories.

**Status (22 November 2026, step 12b).** `at:` takes a list of values for a variable and gives the result at each, and at each combination of several variables' values, the first changing slowest, as Stata's `at(x = (5 10 20))` does. Taken as recommended, the owner's to overturn: the values stand in a column of their own before `variable` (or `quantity`); a row's name carries them, `exper @ exper=5`; a value given twice is refused; the rows are separate statements, with no test between them (`contrasts`, 1.6).

### 203. One layout of three panes on every page (Zemp Lite)

Raised and decided by the owner on 21 November 2026, in the Zemp Lite session, while trying the hosted alpha of Zemp Lite 0.2; written up by that session in the shared notes and entered here on 22 November 2026 from a Zeff session, which alone edits this register, on the owner's word in that session ("Yes, enter it"). His words of the 21st are given as the note gives them. The mockup he approved (its second draft) is kept outside the repositories; Zemp Lite records the same decision in its own `docs/decisions.md` of that day.

*The ruling.* The owner proposed one layout for every page in the place of the margin editor's two layouts (the margin and Panes). A mockup was drawn twice to his corrections; he answered "looks fine", asked that it be "0.2 (patch 1)", and then: "Yes, record it as 0.2.1 and reader test, your checks by hand etc will move to 0.2.1".

**Decided (21 November 2026).** Every place (Script, Data, Plots, Help) has a left pane, the place itself in the middle, and a right pane. A bar under the top bar carries one link for each pane, above that pane, which opens and closes it; a pane also closes by its own ×. The **Panes** button and the **Margin** link are removed: the margin editor and the Panes layout become this one layout. Each place remembers which of its panes are open.

- **Script.** Left: **Project** (the project's files) and **Outline** (the sections `// # Title` and subsections `// ## Title`, each going to its line, with **Run section**), each with its own link, both closed at first, open together one above the other. Right: **Notes**, open at first: the note beside each statement, which says what it made, so the **Objects** panel is removed. **Run** unfolds every output under its statement (as **Unfold all**); a script opened is folded as before.
- **Data.** Left: **Frames of the run**, open at first. Middle: the grid as in 0.2. Right: **Variables**, closed at first, showing either the list of variables (each with its box to show or hide it in the grid) or one variable: clicking a column's header or a name in the list opens the pane at that variable's properties in the place of the list (label, type, missing count and share, histogram; for a number N, mean, sd, min, first quartile, median, third quartile, max and the number of distinct values; for a category each level's count and share), with "‹ All variables" back to the list. The pane is read-only: the type, contrast and label changes and their **Copy as Zeff** buttons are removed from it.
- **Plots.** Left: **Plots of the run**; right: **The plot** (its statement, line, kind and export size); both open at first, both closable.
- **Help.** Left: the outline of the help page (its sections, each going to its place); right: **Related functions**, the page's See also with each function's one-line title, opening its page; both open at first.

*When.* Zemp Lite 0.2.1, on Zeff 0.8.0, after 0.2 is released. It needs no engine release (the Zemp Specification, section 15: a Zemp fix never needs an engine release): the quartiles and the median are the engine's `quantile` and `median` of 0.8.0; the related functions are the help pages' See also; until the language server reports sections (#178, built at Zeff 0.9.0, step 28), the interface reads the comments `// # Title` and `// ## Title` itself, and takes the engine's sections once Zemp moves to 0.9.0.

*What it revises.* #174 (the margin editor: its margin becomes the Notes pane, and a run now unfolds every output, where the ruling of 18 November 2026 left them folded) and #176, R2 (Panes kept beside the margin editor for one release: settled by removing Panes as a layout). #178's outline and **Run section** come at Zemp Lite 0.2.1 in the interface's reading of the comment, ahead of the engine's.

*Also moved to 0.2.1 by the same ruling.* The reader test (one test for Zemp and Zeff 0.9.0, run on the hosted alpha with five to eight readers the owner chooses; step 45 of Zeff 0.9.0) is run on the hosted 0.2.1, not 0.2; the test still serves Zeff 0.9.0 and its scripts are unchanged. So are the acceptance tests left for the hand at Zemp Lite 0.2 (the keyboard alone, VoiceOver, the print at every size and in the dark theme) and the check by hand of the timing of ⌘↵ in Safari.

**Status (22 November 2026).** Decided; nothing in Zeff waits on it. The Zemp Specification is amended with this entry (section 15, the rows of 0.2 and 0.2.1; sections 5, 7 and 8), logged in `docs/spec_changes.md`.

**Added on 22 November 2026.** The Help place of 0.2.1 shows its pages set in type, not as printed (the dated paragraph of that day under #74). Zemp Lite 0.2.2, the release after this one, is #208.

### 204. `curve`: what #112 leaves unsaid

Raised on 22 November 2026 at release 0.9.0, step 14, when `fit.curve(newdata:, type: roc)` was built. #112 fixes the name and the usage, and #120 that the figure is `fit.curve(type: roc).plot()`.

**(a) Is `type:` required?** The usage writes `type: roc` with no default, as it writes `family: binomial` for `glm`.

| Option | For | Against |
| --- | --- | --- |
| A. Required: `g.curve()` is refused (Z0406), the help naming `type: roc` | A function of several curves says which it draws; when the precision–recall curve arrives no script changes its meaning and none has a default chosen by which curve came first; as `family:` of `glm` | Four more characters while there is one curve |
| B. `roc` unless given | Shorter today | The first curve built becomes the default for good |

*Recommendation.* A. **Decided (22 November 2026, step 14):** A, under the standing instruction.

**(b) How the result is read.** The result prints the rows, the counts of ones and noughts, and the area. A script needs the area as a number and the points as a frame, and the catalogue gives no new method for either.

| Option | For | Against |
| --- | --- | --- |
| A. `statistic()` gives the area; `estimates()` gives the points, a frame of `threshold`, `sensitivity` and `specificity` from the highest threshold to the lowest; `plot()` draws the curve; no p-value and no interval | The area is a statistic, the Mann–Whitney statistic of the fitted probabilities over the number of pairs; the points are what the curve estimates; nothing new to learn | `estimates()` of other results has a row for each estimate with its standard error |
| B. The area by `estimates()`, a frame of one row; the points only through the plot | `estimates()` as elsewhere | No way to the points, which a custom figure or a choice of threshold needs |

*Recommendation.* A. **Decided (22 November 2026, step 14):** A, under the standing instruction.

**(c) Weights.**

| Option | For | Against |
| --- | --- | --- |
| A. Each row counts as its weight, frequency or sampling; the rows of `newdata` once each | After frequency weights it is the curve of the rows written out, exactly; after sampling weights it describes the population the fit describes | pROC does not weight; Stata's `lroc` takes frequency weights alone |
| B. Frequency weights counted, sampling weights refused | Stata's reading | A fit with survey weights has no curve |

*Recommendation.* A. **Decided (22 November 2026, step 14):** A, under the standing instruction.

**Status (22 November 2026).** Built as decided. An interval for the area and a test between two curves wait until after 1.0 (question 3 of #99). Deadline for the owner to overturn any: 1.0.

### 205. `confusion`: the printed table and how its shares are read

Raised on 22 November 2026 at release 0.9.0, step 15, when `fit.confusion(newdata:, cutoff: 0.5)` was built. The catalogue gives the usage and "classification table at a cutoff with sensitivity, specificity and accuracy"; the checklist adds the two predictive values.

| Point | Option taken (A) | The other reading |
| --- | --- | --- |
| The table | The observed outcome in the rows (`union 1`, `union 0`), the predicted in the columns (`predicted 1`, `predicted 0`), with the totals of both | Stata's `estat classification` has the observed outcome in the columns |
| The shares | Five, as proportions to four decimals under the table: sensitivity, specificity, accuracy, positive and negative predictive value | Stata prints percentages, and the false rates too, each one less a share printed here |
| Reading them | `estimates()` gives the five as a frame; `statistic("sensitivity")` gives one by its name, and `statistic()` the accuracy | A method for each share, which the catalogue does not have |
| A row at the cutoff | Predicted 1, as Stata's "classified + if predicted Pr(D) >= .5" | Predicted 0 |
| Weights | A row counts as its weight, as in `curve` (#204 (c)); under sampling weights the cells are sums of the weights scaled to the rows used, printed to one decimal | Sampling weights refused |
| A share of nothing | `NA` | 0 |

*Recommendation.* A throughout. **Decided (22 November 2026, step 15):** A, under the standing instruction.

**Status (22 November 2026).** Built as decided. Deadline for the owner to overturn: 1.0.

### 206. What `zeff audit` reads before run records have their schema

Raised on 22 November 2026 at release 0.9.0, step 22. Section 11.1 of the Language Reference gives `zeff audit` to 0.9.0 and has it check "the project's run records" against the list of known wrong results ("your run of 12 October used `reg` with `weights:`, affected by issue 123, fixed in 0.9.2"). The build order gives the versioned schema of run records, with `zeff runs`, to 0.11.0, and at 0.9.0 the engine keeps no record of a run at all. Something has to be read.

| Option | What it is | What it costs and buys |
| --- | --- | --- |
| A | From 0.9.0 `zeff run` keeps a small record of each run in `.zeff/runs/`: the engine's version and build, the day, the script, and every built-in function the script and the project's functions it reaches call, each with the options written at the call. The form is provisional and says so; the schema of 0.11.0 replaces it, and `zeff audit` reads both from then on. Nothing is printed for it: the closing line `run recorded: ...` of section 2.1 comes with the schema | `zeff audit` does what the specification says from the day patch releases exist, on the runs that were made, each with its day and its engine. The calls are those the script contains, read from the checked script, not traced as it runs: a call in a branch not taken is listed too, which errs on the side of saying so |
| B | No record before 0.11.0; `zeff audit` reads the scripts as they stand and says which of their calls the pinned engine gets wrong | Nothing is written to the project; and the audit cannot say which run, or that a result made last month on another engine was affected |
| C | `zeff audit` waits for 0.11.0; 0.9.0 ships the list alone | Contradicts section 11.1, and leaves the first patch releases with no way to tell a user that a result is affected |

Two smaller points go with A. (a) *Which engine answers.* The list of an engine knows the wrong results found until that engine was built, so the newest engine installed knows the most: the launcher hands `zeff audit` to the newest release installed, whatever the project pins, and that engine does not check the pin for this command, as when it is tried by `zeff engine use`. (b) *The numbers of the issues.* The two wrong results found so far (corrected at 0.7.0, before there was a public tracker) are numbered 1 and 2 in the list itself; an issue of the public tracker keeps its tracker number from 1.0.

*Recommendation.* A, with (a) and (b). **Decided (22 November 2026, step 22):** A, under the standing instruction.

**Status (22 November 2026).** Built as decided, in steps 22a (the list), 22b (the record) and 22c (the audit). Deadline for the owner to overturn: 0.11.0, when the schema is fixed.

### 207. Which machines the archives of 0.9.0 are built for

Raised on 22 November 2026 at release 0.9.0, step 23, when the release workflow was written. Section 2.1 of the Engine and Project Specification lists six targets, Linux, macOS and Windows on x86-64 and on ARM64; the checklist's step speaks of "the four targets", which are the four machines of the identity check (two x86-64 machines of different SIMD levels, ARM64 Linux and Apple silicon). No test of the project has ever been run on Windows or on an Intel Mac.

| Option | What it is | What it costs and buys |
| --- | --- | --- |
| A | Archives for the three machines that continuous integration builds and proves on every push: `linux-x86-64` (one archive for every x86-64 processor, as section 2.1 has it) and `linux-arm64`, both static, and `macos-arm64`. Windows and Intel Macs get theirs when the whole suite runs there, at the latest with the installers of 1.0.0 | Every archive is of a build whose results are proved identical on its machine. Users on Windows and on Intel Macs wait, or build from source |
| B | All six at 0.9.0 | The Windows paths of the launcher (the store under the programs folder, `zeff.exe`, the zip) are written and tested as values on other systems, and never once run on Windows; an archive would ship unproved |

The launcher already reads and writes the Windows forms, and the workflow takes a further target as one more line, so B is a matter of runners and of time on them when the owner wants it.

*Recommendation.* A. **Decided (22 November 2026, step 23):** A, under the standing instruction.

**Status (22 November 2026).** Built as decided: the trial run of the workflow made the three archives. Deadline for the owner to overturn: any release.

### 208. Zemp Lite 0.2.2: a project saved to the computer, and the actions on files, folders and projects

Raised by the Zemp Lite session on 22 November 2026 and decided by the owner the same day in the coordinating session, which passed his words on; written up by the Zemp Lite session in the shared notes and entered here from a Zeff session, which alone edits this register, on the owner's word in that session ("Yes, enter it", 22 November 2026). His words are given as the note gives them. Nothing is built yet: 0.2.2 starts after Zemp Lite 0.2.1 is released.

*What was found.* (1) **Open folder…** is on Zemp Lite's welcome screen in section 5 of the Zemp Specification with no release named, and is in no row of the release plan (section 15); Zemp Lite's notes of 0.1 promised it for the alpha, the checklist of 0.2 neither built it nor listed it as left out, and the hosted alpha shows it greyed out. (2) **Export project** (section 10; #117) is placed with Zeff 0.11.0, with the request for persistent storage, the reminder to export and **Open a replication package…**; until then a project lives only in the browser's private storage, which the browser may clear, and a visitor cannot take a copy of it. (3) Only the replication package needs Zeff 0.11.0, for its run record; a plain copy of a project's files does not. (4) Zemp Lite 0.2.1 has no control to make a script, or to rename, move or delete a file, or to delete a project: the Project pane offers only **+**, to add a data file, though the project's own code can already do each.

**Decided (22 November 2026).** Zemp Lite 0.2.2, on Zeff 0.8.0, after 0.2.1. The owner's words: first "Yes, into 0.2.2"; then, widening it the same day, "Add file actions, folder actions, project actions". He named the three groups; the list under each is the Zemp Lite session's reading, recorded in its own `docs/decisions.md`.

- **Projects.** **Export project…** as a zip, downloaded, in every browser the alpha supports; the opening of such a zip as a new project; a project renamed, duplicated and deleted; the request to the browser for persistent storage when a project is first made, and the reminder to export (the parts of #117 that wait on no engine).
- **Files.** **New script**; files added from the computer into any folder; a file renamed, moved, duplicated and deleted; a copy of a file saved to the computer.
- **Folders.** A folder made, renamed, moved and deleted with what it holds; a folder saved to the computer as a zip.

No action writes a statement into a script, and renaming or moving a file rewrites none: the app says which scripts name the old path, and changes nothing. `zeff.toml` and the folders the engine knows by name are not renamed, moved or deleted. `.zemp/` is left out of an exported zip.

**Open folder…** comes with the 0.5 desktop beta (Zeff 0.11.0): on the desktop and, on the web, in the browsers that allow a page a folder of the computer (Chrome and Edge; not Safari or Firefox).

*When, and what it needs of the engine.* Nothing: it is on Zeff 0.8.0, and a Zemp fix or addition of this kind needs no engine release (the Zemp Specification, section 15).

*Still to be put to the owner at the plan of 0.2.2,* each with the Zemp Lite session's recommendation; none changes the row. How a deletion is confirmed and whether it can be undone (recommended: asked once, naming what goes and how many files, and saying it cannot be brought back, since the browser's storage has no bin; the deletion of a project offers **Export project…** first). Whether `scripts/main.zeff`, which the engine and the app open by name, may be renamed or deleted (recommended: not). Moving a file by dragging its row (recommended: left out of 0.2.2; **Move to…** in the row's menu does it, and is reached by the keyboard).

**Status (22 November 2026).** Decided; nothing in Zeff waits on it. The Zemp Specification is amended with this entry (section 15: a row for 0.2.2 after that of 0.2.1, and **Open folder…** in the row of the 0.5 desktop beta), and #117 says which of its parts are built at 0.2.2; logged in `docs/spec_changes.md`.

### 209. What the Variables pane shows for each kind of variable (Zemp)

Raised by the owner on 23 November 2026 in a Zeff session, from the hosted Zemp Lite: when a variable is chosen in the Data place, the right chart should be shown for it, a histogram or a bar chart; what then belongs in the pane for the other kinds, such as a column of customer reviews, a paragraph to a row? Recorded on his word in that session ("Yes, record both"). Nothing is decided and nothing is built.

*What the pane shows now* (Zemp Lite 0.2.1, #203, by the owner's answer of 22 November 2026; Zemp Lite's `docs/decisions.md` of that day). Every variable: its label, type, the number of distinct values, and the rows without a value with their share. A number: a histogram, and N, mean, sd, min, the quartiles and the median, max. A variable the engine counts value by value in its column's header (a category, a truth value, a text or a whole number of few values): a bar chart and each value's count and share. A date or a time of many values: a histogram and its earliest and latest. Every figure is a cell of what the engine prints; the interface computes and formats none. A text of many values, such as a review, gets the two lines every variable has and nothing else: no chart and no figure of its own.

*Recommendation (A), kind by kind.* The rule of 0.2.1 holds throughout: every figure is the engine's print, and a figure the engine does not print is left out until it does, never computed by the interface.

| Kind | Chart | Figures | Asked of the engine |
| --- | --- | --- | --- |
| Long text (many values: reviews, comments, open answers) | A histogram of the lengths, in characters | The rows with text, the rows of empty text (`""`) and the rows missing, each counted, since an empty answer and a missing one are not the same; the shortest, median and longest length; five values in full, the first five rows, wrapped; any value that repeats, with its count (a default answer, or text pasted again) | `n_chars` (0.9.0, step 31a), with `summary` and `quantile` of it; the empty rows counted as the missing ones are; `head(5)`; `duplicates` |
| Text of few values (a country code kept as text) | A bar chart, as now | As now, and one line: "Text of few values: `category()` would make it a category" (text never becomes a category silently, Language Reference, section 4) | As now |
| A variable whose every value differs | For text, no chart: "Every value is different: this looks like an identifier", with its first and last values in the order of the rows | For a whole number, the histogram and statistics stay (incomes in whole rupees may all differ), with the same line under them | `n_distinct` against the rows with a value, as now |
| An ordered category (a Likert item, a level of education) | Bars in the order of the levels, never by count | Each level's count and share, then the cumulative share and the median level | `tabulate`; the cumulative share and the median of an ordered category are not printed by the engine of 0.9.0 (Stata's `tabulate` prints the cumulative column), so until an engine release adds them the pane shows the levels in order only |
| A number on an interval scale (a temperature in °C, a test score, a sum of Likert items) | A histogram, as any number | As any number: every figure shown is right on an interval scale; what is not (a ratio of two values, the coefficient of variation) the pane does not show | As now; the column's `unit` and `measure` (section 4.3, rule 3) shown under its label where the data carries them |
| An opaque column (a list, a record, bytes, a file, a geometry, a duration, a period) | None | The rows with a value and without; the first five values as the engine prints them (`<list 3>`, `<file scans/0142.jpg>`) | `head(5)` |

| Option | What it is | What it costs and buys |
| --- | --- | --- |
| A | The table above, at Zemp 0.3, on Zeff 0.9.0 | Each kind says something true of itself; one engine function new to Zemp (`n_chars`) and two already there (`head`, `duplicates`); the ordered category waits in part for the engine |
| B | Long text only (the first row of the table) at Zemp 0.3; the rest when asked | The gap the owner met closed first, with less to test |
| C | Nothing until the desktop beta | The hosted alpha keeps a pane that says almost nothing of a text |

*Recommendation.* A. Word counts and word frequencies are text analysis, which belongs to a later package, and are left out.

*Deadline.* The plan of Zemp 0.3, which opens on the owner's word after Zeff 0.9.0 is released; the release plan's row of 0.3 (the Zemp Specification, section 15) is amended only when the owner decides.

### 210. Symbolic data: interval-, set-, histogram- and distribution-valued variables

Raised by the owner on 23 November 2026 in a Zeff session, with the taxonomy of Billard and Diday (*Symbolic Data Analysis*, 2006), after asking what the Variables pane should show for interval data (#209). Recorded on his word in that session ("Yes, record both"). Symbolic data is named nowhere in either volume, the catalogue or the decisions; nothing is decided.

*What symbolic data is.* In a classical frame a cell holds one value. In a symbolic one a cell holds a description of many: an interval (a day's temperature, [18, 31]), a finite set of numbers ({0, 1, 2, 3} children per household in a region), a histogram ({[0, 10): 0.2, [10, 20): 0.5, [20, 30]: 0.3}), a vector of quantiles or a boxplot's five numbers, a whole distribution or density; and among categorical variables a set of categories ({Hindi, Bengali, English}), categories with weights or probabilities (modal, bar-chart-valued: {Hindi: 0.6, Bengali: 0.3, English: 0.1}), or with capacities or belief functions in place of probabilities. The classical single value is the special case. Such cells arise mostly by aggregation: many classical rows (a station's hourly readings, a region's households) summarised to one row each, keeping their spread rather than their mean alone.

*Where Zeff stands.* Zeff 1.0 computes on seven column types, one value to a cell (Language Reference, section 4.3). Some symbolic values can be stored now in the opaque types: an interval as a `Column<Record>` of its two ends, a set or a modal value as a `Column<List<T>>` or a list of records. Zeff keeps such a column through the row verbs and prints it, but no function computes on it. R offers symbolic data analysis through packages such as RSDA and symbolicDA; Stata has none.

| Option | What it is | What it costs and buys |
| --- | --- | --- |
| A | A package after 1.0 whose column types carry symbolic values through the one interface of section 4.3, rule 1 (each with its Arrow storage and extension name, so Parquet and Arrow files carry them), with no change to the language: intervals, sets of numbers, histograms, quantiles and distributions, sets of categories and modal categories. It starts with the aggregation that makes them from classical rows, then the descriptive statistics (the symbolic mean and variance of Bertrand and Goupil), regression on intervals (the centre method of Billard and Diday; the centre-and-range method of Lima Neto and De Carvalho), principal components (the centres and vertices methods) and clustering. Capacity- and belief-valued modal variables last, or not at all | The methods R offers, as decisions 98 and 99 ask; section 4.3 was written so that types arrive this way. The cost is a package of its own and a reference held to R packages not yet in the reference library |
| B | Store only: functions that build and take apart symbolic values in the opaque types (an interval as a record of two ends), no methods | Data of this kind can be carried and exported; nothing can be analysed |
| C | Not in Zeff; a user aggregates to classical summaries (lower and upper ends as two Float columns) and analyses those | Nothing to build; the methods that treat an interval as one value are out of reach |

*How Zemp would show such a variable* (the Variables pane of #209, once the types exist). Each kind has rules of its own that a classical variable has not (an interval's lower end not above its upper, weights adding to one, quantiles in order), and the pane counts the rows that break them.

| Kind | Chart | Figures | Rows that break its rules |
| --- | --- | --- | --- |
| Set of numbers | Bars: how often each value appears across the cells | The size of the sets (least, median, greatest); the distinct values over all cells | Empty sets |
| Interval | The intervals as horizontal segments ordered by their midpoints (the first 50); for more rows, the histogram that spreads each interval evenly over its range | Symbolic mean and variance (Bertrand and Goupil); the mean width; the lowest lower end and the highest upper end | A lower end above the upper; intervals of no width counted |
| Histogram | The average of the cells' histograms | Symbolic mean and variance; the number of bins in a cell | Weights not adding to one; overlapping bins |
| Distribution | A sample of the densities drawn over one another | The mean and the spread of the cells' means; the mean of their standard deviations | A parameter out of its range (a standard deviation not above 0) |
| Quantiles or boxplot | A row of small boxplots (the first 20) | The median of the medians; the least minimum and the greatest maximum | Quantiles out of order |
| Set of categories | Bars: the share of cells holding each category (the shares may add to more than 100 per cent, and the pane says so) | The size of the sets; the commonest combination | Empty sets |
| Modal categories | Bars: each category's average weight; the first few cells as stacked bars | The number of categories in a cell | Weights not adding to one |
| Capacity or belief | Paired bars for each category: the average belief and the average plausibility | As for modal categories | A belief above its plausibility |

*Recommendation.* A, after 1.0; nothing before. Until then a user can carry an interval as two Float columns, which every method of 1.0 takes.

*Deadline.* 1.0.0, when the roadmap after 1.0 is drawn: only the release of the package is decided then, and its column types, functions and options are put to the owner when that release opens.

### 211. The Help place redesigned: a home page of methods, a page for each method, moving about, and search as one types (Zemp)

Raised by the owner on 23 November 2026 in a Zeff session: the Help place of Zemp Lite should have a **Home** button and **Next** and **Previous** buttons, or something better; Home is the landing page of Help and lists the statistical methods, as a statistics package's features page lists its features by area; each method opens on a click and shows a minimal example and its output; and search shows the matching items from the second letter typed. Recorded on his word in that session ("Write this plan properly"). Nothing is decided and nothing is built.

*What the Help place is now* (Zemp Lite 0.2.1; #74, #203, and the owner's choice of 22 November 2026 to set pages in type). The middle shows one page: a function's page as `zeff help` prints it, or a code's as `zeff explain` does, set in type, every word the engine's. The left pane is the page's outline; the right pane its related functions, each with its one line, and a field that opens a page by its exact name. A page is reached from the script (F1 on a name, **Explain** on a warning, ⌘K) or from another page's links. There is no page to start from, no way back to the page before, and no search: a reader must know a function's name to find it.

**The plan, in five parts.**

*1. Home, the landing page.* Opened by a **Home** button at the head of the Help place, and shown whenever Help is opened with no page asked for. It lists what Zeff does by area, each area a heading with its methods under it as links, in two or three columns as space allows. The areas are of the analyst's work, not of the engine's packages; for the engine of 0.9.0 they would be about these (names and order the owner's, on a mockup): Linear regression; Panel data and fixed effects; Instrumental variables; Binary outcomes; Effects and predictions after a fit; Tests of means, proportions and variances; Nonparametric tests; Analysis of variance and comparisons; Association and tables; Distributions and random numbers; Graphics; Tables of results and export; Data: import, cleaning, reshaping and joins; Dates and text; Scripts, functions and reproducibility. A method is what an analyst asks for, which is often an option rather than a function: "clustered standard errors" is `reg(..., cluster:)`, "Welch's test" is `t_test(..., var_equal: false)`. At the head of Home: the search field (part 4), and the engine's version, so a reader knows what the list is of. Home lists only what the engine in use can run; nothing of a later release.

*2. A page for each method.* Short, and in the same type as the function pages: the method's name; two or three sentences in Zeff's own words on what it is and when to use it; the minimal example, a few statements on a shipped dataset; the output exactly as the engine printed it; the function and options it uses, each a link to the full function page; its R and Stata counterparts, so a reader coming from either recognises it; and its neighbours in the area. Two buttons under the example: **Copy** and **Open in a new script**, which makes a script in the Scratch project holding the example and runs it, so the reader can change it. The output shown is the one kept with the example, not a run made on opening, so the page shows at once and is the same for every reader; the engine's tests hold every kept output to what the engine prints, at every build, as they hold the examples of the function pages.

*3. Moving about: Home, Back and Forward, and a trail, in place of Next and Previous.* Next and Previous buttons suppose that a reader goes through the pages in one order, as through a book; a help system is read by jumping (from a method to its function, to a related function, and back). So:
- **Back** and **Forward** buttons beside **Home**, which go through the pages this reader opened, as a web browser's do; the mouse's own back and forward buttons and ⌘[ and ⌘] do the same.
- A trail at the head of each page saying where it stands, each step a link: *Help › Panel data and fixed effects › Clustered standard errors*, and for a function page *Help › Linear regression › `reg`*.
- The left pane, which now gives the open page's outline, lists the area's methods with the open one marked, and the outline under it; so the previous and the next method are always in view and one click away, and a reader sees where the page sits.
- At the foot of a method page, the previous and the next method of its area, by name, for a reader who does want to read an area through.

*4. Search as one types.* One field, on Home and at the head of every page; `/` puts the cursor in it. From the second character typed, a list opens under the field with up to eight matches, refreshed at each key; arrows move through it, Enter opens one, Esc closes it. Each match shows its kind (method, function, code, dataset), its name and its one line. It matches, best first: a name typed exactly; a name beginning with what is typed; a word of a method's name or of a page's one line beginning with it; an R or a Stata name (`lm`, `xtreg`, `t.test`, `logit`, so a reader finds `reg`, `glm` or `t_test` by the name they know); a code (`Z0402`); and from the fourth character a name one letter away (a slip of the keyboard). Nothing found gives a sentence and the nearest names, never an empty box. Search runs in the browser on the list the engine gives, with no request to any server.

*5. Where the words come from.* The Help place keeps the rule it has: every word and number on its pages is the engine's. The function pages and the codes are the engine's already. The methods index of parts 1 and 2 would be new data held in the engine beside the help pages: for each method its area, its name, its sentences, the function and options it uses, its R and Stata counterparts, its example and the example's kept output. Held there, the same index serves `zeff help` on the command line (with no name, it would list the areas and their methods), the books, and the website at 1.0; its examples are run and checked like every other example. Its sentences follow the own-words rule (never copied from another package's documentation) and the help contract of 0.8.0 (#103).

**Options.**

| Option | What it is | What it costs and buys |
| --- | --- | --- |
| A | Two stages. Stage 1, in Zemp alone on the engine it already has: **Home**, **Back**, **Forward**, the trail, the left pane of part 3, search over the function pages, the codes and the R and Stata names that the pages' *Coming from R or Stata* sections give; Home lists the functions by the catalogue's groups until stage 2. Stage 2: the methods index in the engine, its examples checked, at a Zeff release; then Home and the method pages in Zemp at the Zemp release on that engine | Moving about and search, the larger part of the daily use, come without waiting for the engine; the methods, which are new writing (about 120 to 150 methods at 0.9.0, each with sentences, an example and its output), come once and serve the command line, the books and the website as well |
| B | All at once in Zemp, the methods index kept in Zemp Lite's own files | Sooner; but the examples' outputs are not held by the engine's tests, the index is written again for the command line and the website, and the rule that every word is the engine's is broken |
| C | Stage 1 only; no methods index | Little to build; the reader who does not know which function does what is not helped |

*Recommendation.* A: stage 1 in the plan of Zemp 0.3 (it needs no engine release, the Zemp Specification, section 15); stage 2 put to the owner when Zeff 0.10.0 is planned, with Zemp 0.4 showing it on that engine. The release rows are amended only when the owner decides.

**Questions only the owner can answer** (each with the recommendation).
1. *What Home lists:* methods by area (recommended, as the features page of a statistics package and as part 1 says), or functions by package.
2. *The areas:* their names and their order, on a mockup of Home drawn from the engine of 0.9.0.
3. *Methods of later releases:* left out (recommended: a reader of Zemp Lite runs one engine, and a link to what it cannot run disappoints), or shown greyed with the release that brings them.
4. *Levels:* a filter of Basic, Advanced and Research on Home, from the models plan's levels; recommended later, once there are enough methods for it to help.
5. *Next and Previous:* replaced by Back, Forward, the trail and the left pane, with the previous and next method at a page's foot (recommended), or kept as buttons at the head of a page.
6. *An example's output:* kept with the example and checked at every build (recommended), or run when the page opens.

*Deadline.* Stage 1: the plan of Zemp 0.3. Stage 2: the opening of Zeff 0.10.0. A mockup of Home and of a method page goes to the owner first, in the look of the design guide.

**Decided (23 November 2026).** A mockup of Home, of search and of a method page was drawn in the look of the design guide (`~/dev/handbook/zemp-lite-help-redesign-mockup.html`, outside the repositories; its method page shows the engine's own print of its example). The owner answered: "Yes as drawn, except change "What Zeff does" with something else which is direct lick Index of methods or something else. after no method will be shown which is planned but not ready. search bar was already there, use it. left pane can be collapsed, button name Topics instead Areas. Add topics sections like packages, Errors/warning, etc". So, with the second draft drawn to it:

1. Home is an index of methods by topic, titled **Index of methods**; the topics, their names and their order as drawn (15 at the engine of 0.9.0: Linear regression; Panel data and fixed effects; Instrumental variables; Binary outcomes; Effects after a fit; Means, proportions and variances; Rank-based tests; Analysis of variance; Tables and association; Describing data; Distributions; Graphics; Tables of results; Data management; Dates and text). "Area" is called "topic" throughout.
2. A method planned for a later release is not shown until the release that brings it (question 3).
3. Search is the field already in the Help place's bar, which gains the suggestions of part 4; Home has no second search field.
4. The left pane is called **Topics**, is opened and closed by its link and its ×, as every pane is (#203), and holds sections that fold: **Methods** (the topics, each with its count; a click scrolls the index to it), **Packages** (each with its functions), **Errors and warnings** (the codes by family, as `zeff explain` lists them), **Datasets**, and **Every function, A to Z**. On a method's page it holds the topic's methods with the open one marked, the page's sections, and **‹ All topics**.
5. As drawn otherwise: Back, Forward and Home with the trail, and the previous and next method at a page's foot, in place of Next and Previous buttons (question 5); the small green *new* beside what the release added; no filter by level for now (question 4); an example's output kept with it and checked at every build (question 6); the method page as drawn (sentences, example with **Copy** and **Open in a new script**, output, what it uses, R and Stata counterparts, related functions, methods and codes in the right pane).

**Changed the same day (23 November 2026), on the second draft.** The owner: "every function A to Z and datasets both can be accessed from packages. clicking package will show list of functions. I would also like to add zeff specifications (or something else) like types, frame, if-else, for, fn, etc". So, drawn in a third draft:

6. **Topics** holds four sections that fold: **Methods**, **The language**, **Packages** and **Errors and warnings**. *Every function, A to Z* and *Datasets* are no longer sections of their own.
7. A package opens a page of its own: its one line and the functions this engine has, each with its one line and a link to its page (no function of a later release is listed), and the methods that use it. The shipped datasets are listed on the page of `datasets`, after its two functions.
8. **The language** (named so, rather than "specification", which is the document for those who build Zeff): a short page for each part of the language a script is written in, as a method has, with a few sentences, a minimal example and its kept output. At the engine of 0.9.0, eighteen: values and types; missing values; names and `let`; operators; text and dates; lists, records and mappings; frames and variables; categories and labels; `if`, `else` and `case`; `for` loops; functions with `fn`; `map`, `filter` and `reduce`; errors and `attempt`; repeating to convergence; formulas; the grammar of plots; comments and sections; projects, scripts and seeds. The engine has no such pages now (`zeff help for` finds nothing); they are written in the engine's own words, from the Language Reference, with the methods of stage 2, and `zeff help` on the command line reads them too.

**Added the same day (23 November 2026), on the third draft.** The owner asked for a page of the language to be drawn, and: "also add a section on zeff-lite use". So, in a fourth draft:

9. **Topics** gains a fifth section, **Using Zemp Lite**: a short page for each thing a reader does in the app (at Zemp Lite 0.2.2, thirteen: getting started; projects, scripts and files; running a script; the notes beside each statement; sections and the outline; the Data place; the Plots place; finding help; commands with ⌘K; keyboard shortcuts; exporting and opening a project; where the work is kept; reporting a problem). These pages describe the app, not the language, so their words are Zemp's own and kept in Zemp, not in the engine: the one exception to the rule that every word of the Help place is the engine's. A picture of the screen stands where a method has its example. They are written with stage 1, and each Zemp release that changes what a page describes changes the page.
10. A page of the language has the form of a method's page, as drawn for `for` loops: a few sentences, a minimal example with its kept output (the drawn one is the engine's print of a loop making one regression for each of two outcomes, set side by side by `report`), the rules worth knowing, and the R and Stata ways of writing it; the Topics pane lists the language's pages with the open one marked.

**Placed (23 November 2026).** Asked whether the design could be released as Zemp Lite 0.2.3, the owner was told what Zemp Lite, which runs the released Zeff 0.8.0, can build without a new engine and what needs one, and answered "Yes, enter 0.2.3". So stage 1 is Zemp Lite 0.2.3, on Zeff 0.8.0, after 0.2.2: **Home**, **Back**, **Forward** and the trail; the **Topics** pane with three of its sections, **Packages** (each opening the list of its functions from the engine's print, the datasets under `datasets`), **Errors and warnings** (the codes by family, as the engine lists them) and **Using Zemp Lite** (the thirteen pages, in Zemp's words); and search in the bar's field from the second letter, over names, one-line descriptions, packages and codes, finding an R or a Stata name where a page names it. Until the methods arrive, Home is titled **Index of functions** and lists the functions by package; it becomes the **Index of methods** when they do. The lists are those of Zeff 0.8.0 until Zemp moves to a later engine. The sections **Methods** and **The language**, the method and language pages with their kept outputs, the green *new* and the exact matching of R and Stata names need data the engine does not yet carry, and are stage 2: recommended at Zeff 0.10.0 and shown by Zemp 0.4, put to the owner at the opening of Zeff 0.10.0. The Zemp Specification gains the row of 0.2.3 in section 15 and a clause in the Help row of section 7; logged in `docs/spec_changes.md`. Zemp Lite 0.2.3 is planned and built in the Zemp Lite session on the owner's "Start release 0.2.3".

**The size of the download (23 November 2026, asked by the owner).** Measured on the engine of 0.9.0 under development: the 257 function pages and 148 explained codes the engine already carries are 1.30 MB of text, about 260 kB with brotli, inside the worker that Zemp Lite downloads at 2.92 MB with brotli (Zemp Lite 0.2.2; the limit of risk R4 is 5 MB). The two pages drawn in the mockup are 2.1 and 2.3 kB of text each, about 1 kB each compressed alone; the 111 new pages (80 methods, 18 of the language, 13 of the app) would add between 50 and 110 kB, under 4 per cent of the download, which the browser keeps after the first visit and fetches again only when the engine changes. Two rules follow, taken as engineering: (a) the pictures of the **Using Zemp Lite** pages are not part of the first download: each is drawn small (SVG where the screen can be drawn so) and fetched the first time its page is opened, then kept by the browser; (b) each release of Zemp Lite and of the engine reports the help's share of the download beside the worker's size, and the help text moves into a file of its own, fetched when Help is first opened, if the download comes within 1 MB of the limit.

**Added the same day (23 November 2026): the examples run in the page.** The owner proposed: "in help pages codes will be given and a run button. clicking it execute the code, print output, error, warning, plots using the app kernel in the help page. So open in zeff not required. This will create the plot locally, need not to download". Asked how much it saves, the owner was told the measure: of the 257 function pages (0.93 MB of text, 198 kB with brotli), the printed output of the examples is 187 kB of text and 32 kB with brotli, and no page holds a stored picture; so it saves about 0.03 MB now, and its gain in size is in the pages still to be written, where a drawn plot on each of the 80 method pages could have added 1 to 2 MB. The owner answered "record it for 3.0". So:

11. From Zemp 0.3 (Zemp Lite 0.3, on Zeff 0.9.0), every example on a page of the Help place (a function, a method, the language, an explained code) has a **Run** button, and runs in the engine the app already carries: what the engine said (its output, its warnings and errors with their codes) and the plots it drew stand under the example, drawn in the browser, so no picture of a plot is downloaded and what is shown is always what the reader's engine prints. The reader may change the example and run it again; a **Copy to script** button takes it into the open script. Taken as engineering: an example runs in a session of its own, so it never reads or changes the reader's frames and names; an example that draws random numbers sets its seed, so every reader sees the same numbers; an example is kept fast enough to answer at once. As recommended to the owner (option A), an example runs by itself when its page opens, so the output is there before **Run** is pressed; the owner may choose instead that a page shows only the code until **Run** (option B). The kept output stays in the engine's pages, where the tests check it and `zeff help` on the command line and the website at 1.0, which carry no engine, print it. The pictures of the **Using Zemp Lite** pages show the app, not a result, so rule (a) still holds for them.

**A standing rule (24 November 2026, the owner).** "one thing to record for future releases, you must update the help pages (including codes and figures) as per the zeff engine." So:

12. At every release of Zemp, the Help place agrees with the engine that release carries. The engine's own pages (functions, codes, and from stage 2 the methods and the language) come from the engine, so they change with it. The pages Zemp keeps itself (**Using Zemp Lite**, and any word of Help that Zemp writes) are brought to that engine before the release: every piece of code in them runs on that engine at the build and prints what the page shows; every function, option or code they name exists in that engine under that name (a name the engine has changed, as `logistic` became `glm` at 0.9.0, is rewritten); and every picture is made again from the running app of that release. The build fails where one of these does not hold, so no release ships a page that the engine it carries would contradict. In force from Zemp Lite 0.2.3, whose pre-release routine checks it.

**Changed on 24 November 2026, on the running 0.2.3 (the owner).** Shown the Help place as step 5b left it, the owner asked: "remove "related functions" move search bar to right end, Move "on this pane" to the right pane". So:

13. The right pane of the Help place is **On this page**: the open page's sections, the one in view marked, a click going to it (on a function's, a method's or a language page; on a list, such as a package's page or Home, the pane says the page has no sections). **Related functions** is removed: the page's own **See also** keeps the links it gave. **Topics**, the left pane, keeps the page's neighbours and **‹ All topics**, and no longer holds the page's sections. The search field stands at the right end of the Help place's bar, after **Back**, **Forward**, **Home** and the trail, with nothing after it but the right pane's link. Each pane is still opened and closed by its own link (#203). In Zemp Lite 0.2.3, before the pictures of its pages are made final, so that the pictures show this layout.

The same day the owner added: "Also I want to remove "Help" from right to "Home"". So the trail beside **Home** no longer begins with the word **Help**: it begins with the first step under Home (_Packages › plot › bars_, _Using Zemp Lite › Finding help_, _Errors and warnings › Z01xx › Z0103_), and on Home itself there is no trail, **Home** being marked as the page open.

**Home, a landing page (24 November 2026, the owner).** The owner asked: "Also I want a landing page for the home page with a basic description about Zeff and Zemp Lite. their versions. All Main topics and their basic descriptions. Show me mockup". On a first draft (the two side by side, the Index of functions a tile) the owner answered: "not side by side. one after another. Index of functions not required (as it is accecced from packages). Provide 5 designs. minimalistic, simple but modern"; shown five (plain page, soft panels, title and rows, labels in the margin, numbered; `~/dev/handbook/zemp-lite-help-home-designs.html`), the owner chose: "Soft panel is final and record it". So:

14. **Home** is a landing page, in the design of soft panels: the title **Help**; a panel for **Zeff** with its version, a few words on what it is and its counts (functions, packages, explained codes), then a panel for **Zemp Lite** with its version, its stage (alpha) and a few words, one after the other; then, under **Topics**, a panel for each section of the Topics pane, one under another, each with its count, a sentence on what it holds and an arrow, a click opening it: **Packages** (with the packages' names as small links inside the panel), **Errors and warnings** (opening the codes by family) and **Using Zemp Lite**. Zeff's version and every count are read from the engine, Zemp Lite's version from the app; the words are Zemp's own and held to the engine by item 12. **On this page** lists the panels. The **Index of functions** is no longer a page of Help: every function is reached from its package, from search and from its links. When the engine brings the methods and the pages of the language (stage 2), each joins as a panel under Topics, the methods first. In Zemp Lite 0.2.3, before the pictures of its pages are made final.

### 212. A small estimate in a fit's own table

Raised on 23 November 2026 at step 34 of release 0.9.0. Decision #171 settled how `report` prints a coefficient too small for its decimals. The table a fit prints of itself has the same fault and was not in that decision. Seen on the shipped wages of 2019:

```
w.reg(log(wage) ~ educ + exper + exper^2 + exper^3)
  term          estimate   std.error   t value   p value   95% CI
  exper^2        -0.0011      0.0016     -0.74     0.462   [-0.0042, 0.0019]
  exper^3         0.0000      0.0000      0.27     0.788   [0.0000, 0.0001]
```

The row of `exper^3` gives neither the size nor the sign of the coefficient; section 3.8 of the Language Reference rounds the estimates, standard errors and limits of such a table to four decimals. The other tables of estimates round to a fixed number of decimals too.

| Option | For | Against |
| --- | --- | --- |
| A. **The rule of `report` in every table of estimates: a number that is not zero and would print as zeros is written to two significant digits with its sign; its standard error and the limits of its interval follow its decimals; the column keeps its decimal points one under another** | One rule for every table; a reader never meets `0.0000` for a coefficient that is not zero; R's `summary` does as much by its exponents | Printed output changes in every table that has such a row; the row is wider than the others |
| B. As now, with the advice already on the pages to rescale the variable (`exper / 10`) | No change | The sign and the size stay hidden in the commonest table of the engine |
| C. A note under the table naming the terms below its precision | The table keeps its width | The number is still not shown |

*Recommendation.* A, at 0.10.0, when the help pages of `regress` and `results` are revised and their printed examples are renewed in any case. Deadline: 0.10.0.

### 213. R's names for Zeff's functions, `factor(x)` in a formula first

Raised on 23 November 2026 at step 35 of release 0.9.0, where decision #196 gave `I(...)` and `poly(...)` their own advice. The third call an R user writes inside a formula was not in that decision:

```
w.reg(log(wage) ~ educ + factor(industry))
error[Z0103]: no function is called `factor`
  = help: `zeff help` lists the functions of this release
```

Zeff's form is `category(industry)`. The catalogue records, for most functions, the name R or Stata gives the same thing (its column of equivalents), so the engine already holds what is needed to answer.

| Option | For | Against |
| --- | --- | --- |
| A. **Where a call names no function and the name is the R or Stata equivalent the catalogue records for exactly one Zeff function, Z0103 names that function: "R's `factor` is `category` in Zeff"; a test holds every such line to a function that exists** | The R and Stata user is answered from the catalogue, with nothing to keep by hand; no name is added | The column of equivalents was written for readers and must be read through first: some entries are phrases, some name several functions |
| B. `factor(x)` alone, inside a formula, as #196 did for two names | Smallest | The next name needs the next decision |
| C. As now | No work | The commonest R line for a categorical regressor is answered with a list of functions |

*Recommendation.* A, at 0.10.0. Deadline: 0.10.0.

### 214. NIST's Filip problem and the tolerance of the collinearity rule

Raised on 23 November 2026 at step 36a of release 0.9.0. Of NIST's eleven linear-regression problems Zeff solves ten, nine of them to within 0.3 of a digit of what R's `lm` reaches and Longley to 11.7 digits against R's 13.0. The eleventh, Filip, fits a polynomial of degree ten in one variable; its design is so nearly singular that the collinearity rule of section 7.2 of the Language Reference (a pivot below 1e-7 of its column's norm) drops the tenth power. The engine says so, in a note and in the warning of near-singular regressors, and the certified values are not reproduced. R's `lm` has the same default, 1e-7, and drops the same term; with `tol = 1e-10` it keeps the term and agrees with NIST to 7.8 digits in the coefficients. The owner's draft article promises Zeff's accuracy on these datasets, so what is said of Filip is his to choose.

| Option | For | Against |
| --- | --- | --- |
| A. **As now at 1.0: the tolerance stays the fixed 1e-7 of section 7.2; Filip is a known issue, reported in the release notes and the article as a problem the engine declines with a warning, as R does by default** | No option added before 1.0; the rule that protects every ordinary regression from a nearly duplicated regressor is untouched; a package that says it cannot solve a problem is counted as correct in McCullough's method | The article's table has one problem unsolved |
| B. An option of `reg`, after 1.0, to lower the tolerance for one fit (R's `tol`) | Filip is solved to about seven digits by a user who asks; nothing changes for anyone else | One more option; a user can keep a column that should have been dropped |
| C. A lower fixed tolerance for everyone (1e-10) | Filip is solved with no option | Nearly collinear regressors are kept in ordinary work, with standard errors that mean little; every stored regression output must be read again; R's default is left behind |

*Recommendation.* A now, and B weighed after 1.0. Deadline: 0.10.0, when the validation suite is run in full and the article's table is drawn.

### 215. A name with two functions in the help

Raised on 24 November 2026 by the owner, on the running Zemp Lite 0.2.3: "many functions have multiple usage like histogram. The help page is showing both but one after another, this may confuse user and also hover over script will show which one?"

Nine names of Zeff 0.8.0 each name two functions (section 8.1 of the Engine Specification, a name with two homes, and the shorthands of `plot`): `histogram`, `boxplot` and `density` (a layer of a plot, `p.histogram()`, and a plot of a frame's variable in one call, `w.histogram(wage)`); `plot` (of a frame, and of a fit); `min`, `max` and `append` (of numbers or lists in `core`, and of a frame's column in `data`); `quantile` (of a column in `data`, and of a distribution in `dist`); `summary` (of a frame in `data`, and of a result in `results`). Each has its own page in the engine, but `zeff help histogram` prints both one after the other, each headed `histogram —`, and the Help place of Zemp Lite shows both under one name, each titled *histogram · plot*, with **On this page** listing the same sections twice. Nothing tells the reader which is which.

Hover and F1 are not affected: the language server takes the function the checker would choose, by the value the call is made on (`w` a frame, `p` a plot), and `crates/zeff-engine/tests/hover.rs` holds this for `histogram`. Only a name written without a call, `map(xs, min)`, gives the first of the two.

| Option | For | Against |
| --- | --- | --- |
| A. **One page for each function, told apart by what it is called on: the title says so ("histogram · of a frame", "histogram · of a plot"); the bare name, from search or a link that does not say which, opens a short chooser (*histogram is two functions*), each with its usage line and its one line, a click opening it; search, Topics and a package's page list the two apart; hover and F1 go to the one in use, as now; `zeff help histogram` prints the chooser, and a way to ask for one form prints its page** | Each page reads alone; the reader sees there are two and picks; search can point at one; the same in the app and on the command line | A chooser page and a way to name one form on the command line; nine names to word |
| B. One page with a tab for each form | Compact; the two side by side | The second tab is easily missed; search and links cannot point at one form; the command line has no tabs |
| C. As now, with titles that differ | Least work | A long page of two functions, still easily confused |

*Decided (24 November 2026).* The owner, shown the three: "Yes, option A". The Help place of Zemp Lite does it in 0.2.3, on Zeff 0.8.0, from what the engine already gives (the two functions apart in the index, each with its page): the words that tell the forms apart are taken from what each is called on (the type of its first value), so nothing is written by hand for the nine. The engine's part (`zeff help` printing the chooser, and the way to ask for one form, whose spelling is settled when the step is written) is a step of Zeff 0.10.0, since 0.9.0 is closing; until then Zemp Lite splits the engine's print by function. Deadline: Zemp Lite 0.2.3; Zeff 0.10.0.

