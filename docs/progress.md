# Progress

This file is the running record of the work: at the end of every session a dated entry says what was done, what is green, what comes next and anything the owner must decide. It is kept under 200 lines by folding entries older than the current release into one paragraph each.

## 18 October 2026

**Done.** The repository is on GitHub, the continuous-integration workflow is in place and passing (it refuses forbidden words in names, contents and history, and shows the pinned Rust version), and the specification is in `docs/spec/`. The working files in `docs/` have been created.

**State.** Release 0.1.0 has not started. There is no code yet.

**Next.** When the owner says "Start release 0.1.0", read the release's row in section 14 of the Engine and Project Specification and write its checklist in `docs/releases/0.1.0.md`.

**For the owner to decide.** Nothing at present.

## 18 October 2026, end of session

**Done.** The working files were checked at the start of the session, and an audit of the folder (working files, names, the whole history, branch and tag names) found no forbidden word. No code was written.

**Green.** The continuous-integration run on the last commit passed (both the forbidden-words check and the build placeholder). The working tree is clean and `main` matches GitHub.

**Next.** Start release 0.1.0 when the owner says so: write the checklist in `docs/releases/0.1.0.md`, show the plan, and wait for "go". The checklist's steps will carry the `[engine]` or `[function]` tags.

**For the owner to decide.** Nothing at present.

## 18 October 2026, release 0.1.0 started

**Done.** The previous state was checked (clean tree; no earlier release exists). The sections named for 0.1.0 were read and the checklist was written in `docs/releases/0.1.0.md`: 25 steps, each tagged `[engine]` or `[function]`, ending with the pre-release routine. No code was written.

**State.** Waiting for the owner's "go" and for three printing questions (listed at the top of the checklist; the first is needed at step 19).

**Next.** Step 1, the workspace skeleton and the tier rule `[engine]`.

**For the owner to decide.** The three printing questions in `docs/releases/0.1.0.md`.
