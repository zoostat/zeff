# The Zeff and Zemp specifications

This folder holds the specifications of the Zeff language and engine and of the Zemp application. Where the code and these documents disagree, the documents win unless they are wrong; then the document is corrected and the change is logged in `docs/spec_changes.md`.

## Normative documents

These are the documents an implementation must follow. Anything in them marked "from 1.1", "after 1.0", "proposed" or "roadmap" is a plan, not yet a requirement.

| File | What it is |
| --- | --- |
| `zeff_language_reference.md` | The Zeff Language Reference: the language itself (volume one). |
| `zeff_engine_specification.md` | The Zeff Engine and Project Specification: purpose, projects, packages and tiers, the engine, the build order and release plan (volume two). |
| `zeff_functions.csv` | The function catalogue: one row per function, with its package, tier, release, accuracy class and status. |
| `zeff_testing.md` | The testing handbook: the kinds of test, the routine on each change, and the definition of done for a function. |
| `zemp_specification.md` | The Zemp Specification, including the Zemp release plan. |

## Supporting documents

These are not normative in themselves.

| File | What it is |
| --- | --- |
| `zeff_help_examples.md` | Worked examples of the help-page contract. Their numbers are invented. |
| `zeff_samples.zeff` | Sample Zeff code, valid for Zeff 1.0 unless marked otherwise. |
| `zeff_open_decisions.md` | The register of open decisions, with options, recommendations and deadline releases. |
| `zeff_risk_register.md` | The register of project risks, reviewed at every minor release. |

## Section numbers

The Language Reference and the Engine and Project Specification are two volumes of one specification. They share section numbers, so "section 8.3" means the same place in whichever volume carries it, and the text both volumes carry (the status paragraph and the headers) is kept identical.

## Records kept outside the repository

The decisions list and the specifications refer to some historical records: the audits (`zeff_audit_2026-10-16.md` and the three of 17 October), `zeff_structure_review.md`, `zeff_language_spec_gaps.md`, `zeff_cli_install.md` and `zemp_business_model.md`. They are kept elsewhere and are deliberately not in this repository.
