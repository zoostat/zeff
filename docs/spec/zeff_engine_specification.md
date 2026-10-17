# Zeff Engine and Project Specification

Version of this document: 17 October 2026 (audited; revisions so far: storage formats and the Parquet profile, a 1.0 limited to linear and binary models, `fe:`, `re:` and `instruments:` arguments, failure handling, file schemas, shared memory, data roots, accuracy classes, names and case rules, engine architecture, language at a glance, definitions with `=`, causal graphs, model-matrix rules, contrasts, random-number generator, interoperability, package bundle and release plan, lexical rules and limits, the command line and installation, column types and metadata, reserved types, figure rendering and source files per function) · Engine today: none; the code starts afresh at 0.1.0 (section 14)

Zeff is a statistics language for empirical and causal research. The Zeff Specification is published in two volumes with section numbers shared between them; each volume is edited directly, and the text both carry (the status paragraph and the headers) is kept identical by hand. This volume, the **Engine and Project Specification**, holds the purpose, principles and constraints (1.1 to 1.4), projects and the command line (2), packages, tiers and the engine (8), the function catalogue and the help contract (9), editor support (12), accuracy and validation (13), the build order and release plan (14) and the open decisions (15). The companion volume, the **Language Reference**, holds the language itself: the language at a glance (1.5), syntax (3), types and values (4), control flow, functions and errors (5), frames (6), the declarative forms (7), a complete analysis with its output (10), diagnostics (11) and the glossary (16). A reference to a section of the other volume says so.

**Status of this document.** The rules for the language, types, frames, formulas, results, diagnostics and projects that apply to 1.0 are normative: an implementation of Zeff 1.0 must follow them. Anything marked with a later release ("from 1.1", "after 1.0"), "proposed" or "roadmap" is a plan, not yet a requirement, and the open decisions of section 15 are unsettled. Code examples and printed outputs are illustrative: their numbers are invented, but their form is normative.

**Conformance.** Zeff has no implementation-defined, unspecified or undefined behaviour: every operation's result is fixed by this document or by the function's entry in zeff_functions.csv, and every failure is a named diagnostic (section 11 of the Language Reference). The grammar of section 3.4 of the Language Reference is normative for syntax and the prose for meaning; where two readings of this document conflict, the conformance suite (section 13) decides and the document is corrected in the next release.

---

## 1. What Zeff is

### 1.1 Users

| User | What they need from Zeff |
| --- | --- |
| Applied researchers in management, economics, organisational behaviour and operations | Causal and latent-variable methods in one language, without stitching packages together |
| Analysts who know Stata or SPSS | Short commands with options, and less boilerplate |
| Zemp users who never type code | A readable script of everything they did, which they can save and re-run |

### 1.2 Principles

Every design decision must pass these eight rules; when two conflict, the earlier one wins.

1. **Correct before fast.** A method ships only when it matches R or Stata within the tolerance of its accuracy class (section 13).
2. **Code is the canonical form.** Zemp writes Zeff and runs it; it never calls the engine directly.
3. **One statement, one analysis.** A model is one call: a formula (or, for designs such as `did`, an outcome) plus named arguments.
4. **Statistics are built in.** Frames, formulas, model blocks and results are language types, not add-ons.
5. **Reproducible by default.** A project re-runs to identical numbers on any machine; random seeds are recorded even when not given.
6. **Nothing disappears silently.** Dropped rows, skipped missing values, singletons, coercions and failed resampling draws are always reported with counts.
7. **Readable by a referee.** Someone who has never used Zeff can tell from the code what model was run.
8. **One meaning per argument.** `by:`, `cluster:`, `weights:`, `se:`, `fe:` and `re:` mean the same in every function.

### 1.3 Not goals

Zeff is not a general-purpose language (no web servers or arbitrary programs), does not copy R or Stata source syntax, has no third-party packages before 1.0 (every built-in method is validated by the project), and is not a data-engineering tool.

### 1.4 Licence and technical constraints

| Area | Commitment |
| --- | --- |
| Licence | Apache 2.0 for the language and engine, decided 16 October 2026 (general information, not legal advice) |
| Engine | Rust, compiled to WebAssembly for the browser and to native code for the desktop app and command line; a project gives identical results on both |
| Data in memory | Columnar Apache Arrow buffers, shared between frames rather than copied (section 6 of the Language Reference) |
| Linear algebra | A kernel crate of Zeff's own with fixed SIMD width and block sizes, so results are bit-identical in the browser and natively (section 8.3) |
| Parallelism | From 1.1, loops, bootstraps and forest trees run in parallel on a thread pool with reproducible streams; Web Workers and WebAssembly threads in the browser. 1.0 runs each statement on one thread |
| Large data | After 1.0, estimators that need only running totals (OLS and fixed effects, through a streaming QR decomposition that keeps the accuracy of in-memory fits) stream files larger than memory |
| Dependencies | Permissive licences only (MIT, Apache 2.0, BSD); no GPL code linked into the engine |
| Cost | Every tool and library used is free. The expected costs are code signing for the desktop apps' setup files (the Apple Developer Program for macOS, and a Windows signing certificate unless a free open-source signing service is used), a domain name for the installer, documentation and registry hosts, and trademark filing for the names Zeff and Zemp (section 15; zeff_open_decisions.md, #23, #31 and #37) |
| Privacy | Data stays on the user's machine unless the user moves it; a script reaches the network only through `fetch`, in a project that sets `network = true` (section 8.6) |
| Browser limit | 32-bit WebAssembly caps memory at 4 GB; in practice about 2–3 GB of input data fits, after the engine, results and browser overhead; larger jobs go to the desktop app |

Section 1.5, the language at a glance, opens the Language Reference.

---

## 2. Projects and running code

Zeff code runs only inside a project: a folder whose `zeff.toml` pins the engine version, so a copied project re-runs unchanged on any machine.

```
schooling/
    zeff.toml          project name, engine version and data roots
    zeff.lock          exact versions and hashes of third-party packages (after 1.0)
    scripts/main.zeff  the script `zeff run` starts from
    functions/         the project's own functions, brought in with `use`
    data/              input files
    output/            tables, plots and saved frames
    .zeff/runs/        recorded seeds, warnings, file hashes and outputs of each run
    .zeff/schemas/     cached column names and types of each data file
    .zeff/cache/       results of earlier runs, and Arrow copies of imported files, reused while code and inputs are unchanged (section 8.7)
    .zeff/packages/    vendored copies of third-party packages, so a replication package needs no network (after 1.0)
```

```toml
[project]
name = "schooling"
zeff = "0.1.0"
edition = "preview"   # "2027" from 1.0.0 (section 2, Editions)
memory = "3.5GB"      # optional; overrides the default browser memory budget
network = false       # true lets `fetch` reach the network (section 8.6)

[data]
lfs = { path = "D:/surveys/lfs" }        # always read-only

[packages]            # after 1.0: third-party packages by version requirement; zeff.lock records what was resolved
survival = "^1.2"
```

| Rule | What it means |
| --- | --- |
| Project only | Every script, dataset and output belongs to a project; there are no loose scripts |
| Paths from the root | `import("data/wages.csv")` resolves from the project folder; other paths are refused unless they sit under a data root. `zeff check` compares each path's case with the directory listing and reports a mismatch (Z0702), so a script that works on Windows or macOS does not fail on Linux |
| Data roots | A folder declared under `[data]` is read with its name as a prefix, `import("lfs:wages.parquet")`, so large shared datasets are not copied into every project. Roots are read-only; each run records the hash of every file it read and warns when a file has changed since the last run. In the browser, the user grants access to the folder once |
| Editions | An edition is a named set of lexical and grammatical rules, reserved words, diagnostic codes, term names and printed layouts (section 3.7 of the Language Reference), as Rust's editions are. It never changes a number: results depend on the pinned engine version alone, so two editions of one script on one engine print the same estimates. All files of a project share its edition. `edition` is required; before 1.0.0 the only value is `"preview"`, which may change between minor releases, and 1.0.0 declares `"2027"`. An engine runs every edition from 2027 on; an edition it does not know is an error (Z0706) naming the engine version needed. `zeff fix --edition` rewrites a project to a newer edition and shows each changed line (section 8.3) |
| Writes | `save` and `export` write only under the project folder (`output/` by default); a path in a data root or outside the project is an error (Z0707) |
| Scripts | Each file in `scripts/` is a complete program with its own bindings; scripts do not include one another, and code shared between them lives in `functions/` (section 5.3 of the Language Reference) |
| Locked packages | After 1.0, `zeff.lock` records the exact version and content hash of every third-party package the project uses; a run uses the lock, never the requirement in `zeff.toml`, and refuses to start when an installed package differs from it (Z0703). The lock is part of the project and of every replication package (section 8.1, rule 9) |
| Recorded runs | Each run stores its seeds, warnings, file hashes, engine version, package versions and outputs in `.zeff/runs/` |
| One project per window | A window holds one project, fully isolated; nothing is shared between windows |
| No duplicate opening | A project open in one window cannot be opened in a second; a lock file in `.zeff/` enforces this. Data-root files are opened read-only with shared read access, so several projects can read the same files at once on every system |
| Scratch project | Zemp opens a Scratch project for quick exploration; its work can be promoted to a named project |
| Explicit sharing | Code moves between projects only through Import from another project; data through Import or a data root |

**Command line.**

| Command | What it does |
| --- | --- |
| `zeff new <name>` | Create a project folder with `zeff.toml`, `scripts/main.zeff`, `data/` and a `.gitignore` that leaves out the caches (`.zeff/cache/`, `.zeff/fetched/`), the run records (`.zeff/runs/`), the project lock file and `.zeff/session.zeff`, `.zemp/views.json` and `data/`, so a project can go into Git without its caches, its lock or, unless the user adds them, its data (Zemp Specification, section 5) |
| `zeff run [script]` | Run `scripts/main.zeff` or the named script. `--fresh` ignores the result cache, `--replay` reuses the recorded values of `today()` and `now()` (section 3.8 of the Language Reference), `--deny-warnings` turns warnings into errors (section 11.1 of the Language Reference), `--no-external` refuses external steps (section 8.3), and `--refresh` and `--offline` repeat or forbid network requests (section 8.6); `--frozen` downloads nothing at all, and `--install` installs a missing pinned engine without asking (section 2.1) |
| `zeff check [script]` | Parse, then check names, arguments, types and variables against the cached file schemas, without running any model (section 11 of the Language Reference) |
| `zeff fmt` | Format every script in the project (no options) |
| `zeff test` | Run the project's `assert` checks and function tests |
| `zeff lsp` | Start the language server used by editors (section 12) |
| `zeff help <name>` | Print a function's help page (section 9.1); `zeff help regress` lists a package |
| `zeff version` | Print the engine version |
| `zeff fix --edition` | Rewrite the project's scripts to a newer edition, showing each changed line (section 2) |
| `zeff explain <code>` | Explain an error or warning code: meaning, common causes, an example and the fix (section 11.1 of the Language Reference) |
| `zeff report-bug` | Build a local bug report with a shrunk reproducer, for the user to review before sharing (section 11.1 of the Language Reference) |
| `zeff audit` | Check the project's run records against the list of known wrong-result bugs (section 11.1 of the Language Reference) |
| `zeff repl [script]` | Open the project's console. Each statement is appended to the session script `.zeff/session.zeff` and run from there, so bindings stay immutable and the session replays like any script; `:undo`, `:save`, `:load`, `:help`, `:vars`, `:quit`. Outside a project the session runs in the scratch project (section 2.1) |
| `zeff eval "<statement>"` | Run one statement in the scratch session and print its value |
| `zeff selftest` | Run the conformance cases shipped with the engine against their published checksums and say whether this installation is conformant (section 8.3, point 3) |
| `zeff engine list \| install \| remove \| default \| use` | Manage the installed engines; `use <version>` runs the project on the pinned engine and on another, compares every result, lists the bare names the other engine's built-ins would capture (Z0109) and the warning codes it newly raises, and with `--write` rewrites those calls to their full form and moves the pin (section 2.1; section 3.6 of the Language Reference) |
| `zeff runs list \| show \| diff` | Read the run records in `.zeff/runs/`; `diff` compares two runs in sentences |
| `zeff export replication` | Build the project's replication package (section 11.1 of the Language Reference) |
| `zeff verify <package>` | Re-run a replication package in a temporary folder and compare every result and output with its run record (section 11.1 of the Language Reference) |
| `zeff doctor` | Report the versions and builds installed, the paths, the settings in effect and their source, PATH and completions, connectivity and disk space, each problem with the command that fixes it |
| `zeff app install \| list \| update \| remove` | Install and manage the desktop apps, Zemp Lite (`lite`) and Zemp (`zemp`), from the command line; `--from <file>` installs from the offline bundle (Zemp Specification, section 9) |
| `zeff self update \| uninstall \| install \| path` | Update or remove the launcher; register an unpacked archive; print the PATH line (section 2.1) |
| `zeff completions <shell>` | Completions for bash, zsh, fish and PowerShell: commands, flags, function names, diagnostic codes and engine versions |
| `zeff dataset fetch <name> \| --all` | Pre-fetch datasets larger than a megabyte for a machine without network (section 8.5) |
| `zeff add <package>` | After 1.0: add a third-party package to `zeff.toml` and record the version and hash resolved in `zeff.lock` (section 8.1, rule 9) |
| `zeff update [package]` | After 1.0: move third-party packages to the newest versions their requirements allow, rewrite `zeff.lock` and show what changed |
| `zeff remove <package>` | After 1.0: remove a package from `zeff.toml` and the lock; `zeff check` then reports every call that used it |
| `zeff packages` | After 1.0: list the project's packages with version, licence, validated badge and functions (section 8.4) |
| `zeff vendor` | After 1.0: copy the packages a project uses into `.zeff/packages/` for a self-contained replication package |
| `zeff publish` | After 1.0: check, build, sign and submit a package to the registry (section 8.4) |

### 2.1 Installing and running from the command line

**The launcher and the engines.** The `zeff` command is two things: a small *launcher*, which rarely changes, and *engines*, one per version, installed on demand. The launcher reads the project's `zeff.toml`, selects the engine it pins, verifies it and hands the command over; outside a project it uses the default engine. This is what exact pins, patch releases installed side by side (section 11.1 of the Language Reference) and one engine store shared with the Zemp desktop app (Zemp Specification, section 9) need, and it is how Rust's `rustup`, `uv` and `nvm` work. Before 0.9.0 launcher and engine are one binary; the split arrives with patch releases.

```
~/.zeff/                   ZEFF_HOME; one folder, no administrator rights
    bin/zeff               the launcher, on PATH
    engines/0.11.0/        one self-contained folder per engine version: the language,
    engines/0.11.1/        every first-party package, the help pages, the small datasets
    packages/              third-party packages (section 8.4, after 1.0)
    datasets/              datasets fetched on first use (section 8.5)
    config.toml            user settings: external interpreters, colour, mirror, proxy
    secrets.toml           keys for fetch (section 8.6); never in a project
    cache/                 downloads awaiting verification; safe to delete
    scratch/               the scratch project of zeff repl and zeff eval
```

A system-wide install (`/opt/zeff`, `C:\Program Files\Zeff`) holds `bin/`, `engines/`, `packages/` and `datasets/` read-only for every user; `config.toml`, `secrets.toml`, `cache/` and `scratch/` always live in the user's own `~/.zeff`, which the launcher searches first.

**Installing.** Host names are placeholders until the domain is decided (section 15).

| Channel | Command | Notes |
| --- | --- | --- |
| Installer script | `curl -fsSL https://zeff.dev/install.sh \| sh`; `irm https://zeff.dev/install.ps1 \| iex` | Prints every step before taking it (`--dry-run` takes none); installs the launcher and the newest engine into `~/.zeff`, verifies checksums and signatures, adds one line to the shell's start-up file or the user's PATH on Windows, shown first and removed by `zeff self uninstall`; never asks for administrator rights; `--system` with `ZEFF_HOME` for a shared install |
| Homebrew | `brew install zeff-lang/tap/zeff` | macOS and Linux; launcher only, engines on demand; completions and man page installed |
| winget, Scoop | `winget install Zeff.Zeff`; `scoop install zeff` | Windows, per user |
| Archive | `zeff-<version>-<target>.tar.zst` or `.zip`, with `SHA256SUMS` and its signature | Launcher and engine together; unpack anywhere and run `zeff self install` |
| Offline bundle | `zeff-offline-<version>-<target>` | Launcher, engine, help and datasets for machines without network; `zeff engine install --from <file>` |
| Container image | `ghcr.io/zeff-lang/zeff:<version>` | A static binary on a minimal image, for continuous integration and clusters; `--frozen` by default |
| GitHub Action | `uses: zeff-lang/setup-zeff@v1` | Reads the pin from `zeff.toml` and caches engines |
| From source | `cargo install zeff --locked` | An unofficial build: `zeff version` and every run record say so, and `zeff selftest` says whether it is conformant |
| Desktop apps | `zeff app install lite \| zemp`, `zeff app list \| update \| remove` | The launcher installs Zemp Lite or Zemp beside its engines, verified against the same built-in key; setup files for Zeff and the apps are deferred (Zemp Specification, section 17) |
| Zemp desktop | Settings → Install command-line tool | Puts `~/.zeff/bin` on PATH; the app and the command share one engine store |
| After 1.0, on request | PyPI wheel, conda-forge, Nix, Debian | Community packaging is safe because the launcher verifies every engine it installs |

| Target | Minimum | Build |
| --- | --- | --- |
| Linux x86-64 and ARM64 | Any distribution of the last ten years (kernel 3.2) | Static (musl), no system libraries |
| macOS x86-64 and Apple silicon | macOS 12 | Separate binaries |
| Windows x86-64 and ARM64 | Windows 10 | The console is switched to UTF-8 for the run; `--ascii` for fonts without box drawing (section 3.8 of the Language Reference) |
| WebAssembly | — | Zemp web only, not a command-line target |

Results are bit-identical across these targets by the determinism contract (section 8.3, point 3). The x86-64 build targets the baseline instruction set (SSE2): the kernels use 128-bit SIMD with no run-time dispatch, so a wider build would gain little and run on fewer machines.

**Trust.** Every archive is signed (ed25519, minisign format) with a key built into the launcher, and the launcher refuses an engine or package whose checksum or signature does not match (Z0711) before unpacking it; key rotation ships as a signed key list. Official binaries are reproducible builds: a tagged commit, a pinned Rust toolchain, `--locked`, fixed flags and a fixed build date, so anyone can rebuild and compare. `zeff version --verbose` prints version, launcher version, target, commit, toolchain and *official build* or *source build*, and the run record stores the same line. None of this is operating-system code signing: a terminal binary installed by the script or a package manager runs without Apple notarisation or a Windows certificate, and a browser-downloaded archive, which is quarantined on macOS and warned about on Windows, is the last choice on the download page. Code signing (section 15) concerns the Zemp desktop app.

**Running.** `zeff` walks up from the current folder to the nearest `zeff.toml`, as Cargo and Git do, or `--project <dir>` (`-p`) names one. Outside a project, `run`, `check`, `test` and `fmt` stop with Z0709 ("not inside a Zeff project; `zeff new <name>` creates one", exit code 1); `repl`, `eval` and `help` work anywhere. What the script prints goes to stdout; diagnostics, progress and the closing line `run recorded: .zeff/runs/<id>` go to stderr, so `zeff run > results.txt` captures exactly the printed output. Off a terminal, colour is off and the width is `print`'s default. A pinned engine that is not installed is installed after a question on a terminal, without one under `--install` or `-y`, and is otherwise exit code 4 naming `zeff engine install <version>` (Z0710). The exit codes of section 3.8 of the Language Reference are unchanged.

| Flag on every command | Meaning |
| --- | --- |
| `-p, --project <dir>` | The project to act on; default the nearest ancestor with `zeff.toml` |
| `--format text \| json` | Machine-readable output for `check`, `test`, `audit`, `runs`, `verify`, `doctor`, `version` and `engine list`; a diagnostic in JSON carries code, span, message and fixes, the record the editor shows (section 8.3, point 1) |
| `--color auto \| always \| never`; `NO_COLOR` | Colour; `auto` means a terminal |
| `--ascii`, `--width <n>` | Rules and symbols in ASCII; printing width (section 3.8 of the Language Reference) |
| `-q, --quiet`; `-v, --verbose` | Hide notes; show progress, cache hits and timings |
| `--frozen` | Nothing is downloaded: no engine, package or dataset; implies `--offline` (section 8.6) |
| `--install`, `-y, --yes` | Install a missing pinned engine without asking |
| `--memory <size>` | Overrides the project's memory budget for this run (section 8.3, point 7) |
| `--threads <n>` | From 1.1; results do not depend on it (section 8.3, point 10) |

**The console.** `zeff repl` is the command-line counterpart of Zemp's console. Inside a project it opens a session with the project's functions available; each statement typed is appended to `.zeff/session.zeff`, the session script, and run from there, so bindings stay immutable (a second `let m` is refused with the usual suggestion of a new name) and the session replays like any script. `:undo` removes the last statement and re-runs the session from the result cache; `:save scripts/explore.zeff` promotes the session to a script; `:load scripts/main.zeff` runs a script's statements into the session; `:help reg`, `:vars` and `:quit` do what they say. Outside a project the session runs in the scratch project, `~/.zeff/scratch/`, as Zemp's Scratch project does, and `:save` can create a project with `zeff new`. `zeff eval "dist.normal().quantile(0.975)"` runs one statement in the scratch session and prints its value.

**Engines.** `zeff engine list` shows the installed engines with size, build and the default; `install <version>` downloads (or reads with `--from <file>`), verifies and unpacks atomically, and `install 0.11` means the newest 0.11.x; `remove <version>` warns that projects pinning it will install it again; `default <version>` sets the engine used outside projects and by `zeff new`, which pins it exactly, never as a range. `zeff engine use <version>` is Zemp's "Update engine" from the command line: it runs the project on the pinned engine and on `<version>`, compares every result, prints the differences in sentences and rewrites the pin in `zeff.toml` only with `--write`.

**Verifying and replicating.** `zeff export replication [--out <file.zip>] [--data copy \| checksums]` builds the replication package of section 11.1 of the Language Reference: `zeff.toml`, `zeff.lock`, `scripts/`, `functions/`, `data/` (or checksums for data roots), the latest run record, vendored packages, and a first page naming the engine version, how to run, and every external step. `zeff verify <file.zip \| folder> [--engine <version>] [--data <root>=<path>]` unpacks into a temporary folder, installs the pinned engine if allowed, runs every script named in the run record with `--fresh --offline --no-external`, and compares each statement's result hash, each output file's hash and the warnings with the record; it prints *identical*, or a table of what differs with external steps listed as not verifiable, and exits 0 when identical and 1 when not. `--engine` replays on another engine, to show that a result survives an engine update. `zeff runs list`, `show <id>` and `diff <a> <b>` read `.zeff/runs/`; `diff` is the comparison Zemp shows. A referee's whole procedure is to install, run `zeff verify replication.zip` and read one word.

```yaml
# continuous integration, GitHub Actions
- uses: zeff-lang/setup-zeff@v1          # the launcher and the pinned engine; engines cached
- run: zeff check --format json           # names, arguments, types against the cached schemas
- run: zeff test                          # assert checks and function tests
- run: zeff run --deny-warnings --frozen  # the analysis; a warning fails the build
- run: zeff verify output/replication.zip # the package re-runs to identical numbers
```

On a cluster: the offline bundle or an institutional mirror (`ZEFF_MIRROR`), `ZEFF_HOME` on the shared file system read-only, `--frozen` on compute nodes, `--threads` and `--memory` to match the job's allocation, and `zeff selftest` on the node to show its results are the laptop's.

**Diagnosis and settings.** `zeff selftest` runs the conformance cases shipped with the engine against their published checksums, in seconds, and prints `conformant: 1,240 of 1,240 cases match the published results of Zeff 0.11.0 on this machine (x86-64)` or names the cases that differ: the user-facing side of the determinism contract and the test of a source build. `zeff doctor` reports version and build of launcher and engines, `ZEFF_HOME` and the user folder, PATH and completion status, the settings in effect with their source, whether the release host is reachable (unless offline), free disk space and the last `selftest`, each problem with the command that fixes it. Settings come, in order of precedence, from the flag, the environment variable, the project's `zeff.toml`, the user's `config.toml` and the default.

```toml
# ~/.zeff/config.toml                      user settings; never read from a project
[external]
r = "/usr/local/bin/Rscript"               # section 8.3, point 8
python = "/opt/miniconda3/bin/python"

[network]
mirror = "https://mirror.univ.edu/zeff"    # release and registry hosts, for institutions
proxy = "http://proxy.univ.edu:3128"       # otherwise HTTPS_PROXY is honoured

[console]
color = "auto"                             # auto | always | never
ascii = false
```

`ZEFF_HOME` names the engine store (default `~/.zeff`), `ZEFF_OFFLINE=1` and `ZEFF_FROZEN=1` act as the flags, `ZEFF_MIRROR` as `[network] mirror`, and `NO_COLOR` and `HTTPS_PROXY` are honoured as the wider ecosystem defines them.

| Code | Meaning |
| --- | --- |
| Z0709 | A command that needs a project was run outside one; the message names `zeff new`. Exit code 1 |
| Z0710 | The pinned engine is not installed and cannot be installed: frozen, offline, or no such version. Exit code 4 |
| Z0711 | A downloaded engine or package failed its checksum or signature check; nothing was unpacked |

---

---

## 8. Packages and tiers

Zeff's functions live in 31 packages on four tiers around a centre, `core`. Packages are grouped by method family, not by research question, because most methods serve several questions; every function also carries `purpose` tags (one or more of the 13 research goals of the Zemp Specification, section 11, or `general` for infrastructure), which Zemp uses to build question-first menus.

```
Tier 4  extensions   project functions/ · third-party packages after 1.0
Tier 3  designs      identify → experiment → quasi → adjust → mediate →
                     hetero → structural → decide → meta
Tier 2  models       regress → glm → multilevel → explore → series → latent →
                     measure → learn → groups
Tier 1  toolkit      time → text → data → dist → results → stats → optim →
                     resample → survey → plot → datasets → web
Tier 0  centre       core
        engine kernel (Rust only; its modules are fixed by rule 8)
```

### 8.1 Rules

1. **`core` calls nothing above it.** It is the language and changes least; it freezes at 1.0. A continuous-integration check enforces this: the `core` crate may depend only on the engine kernel's crates, and the build fails otherwise. Three `core` functions that touch other packages' values do so without calling them: `print` uses the display interface `core` declares (rule 5); `help` reads the engine's function registry, built from every package's help files and `///` text when the engine is compiled (section 9.1); and `x.to(Date)` uses the kernel's date parsing, not the `time` package.
2. **Calls go down freely.** A package may call any package on a lower tier.
3. **Calls go sideways only in order.** Within a tier, a package may call only the packages listed before it (`→`), so calls can never go round in a circle. Each package is its own Rust crate, and Rust refuses to build circular dependencies; the loader holds Zeff-written functions to the same rule. The order within a tier is also a build order: a package's prerequisites are the packages before it, so a roadmap package is scheduled only after they exist.
4. **Shared types are declared in `core`.** `Frame`, `Column`, `Date`, `DateTime`, `Table`, `Plot`, `Result`, `Failed`, `Model`, `Dag`, `Formula`, `Terms`, `Term`, `Distribution` and `SurveyDesign` are declared in `core` and implemented by packages, so a package that only accepts a result never depends on the package that makes it. Machinery several tiers need, such as date parsing, resampling and design-based variance, lives in the engine kernel, so `data` and `stats` honour a declared survey design without calling `survey`.
5. **Lower packages define interfaces; higher ones implement them.** `core` declares the display interface, how any value prints, which every type implements in its own package. `results` builds on it and defines the two interfaces every fitted model implements: the result interface (`summary`, `estimates`, `std_errors`, `p_values`, `ci`, `n_obs`, `export`, and `report` for tables) and the estimator interface (`coef`, `vcov`, `fit_measures`, `predict`, `residuals`, `fitted`, `compare`, `test`, `derive`; `margins` and `contrasts` after 1.0), which is the Zeff face of the estimator trait of section 8.3. A model package implements both for its own result types and adds only what is specific to its models (`r_squared`, `influence`, `vif` and `first_stage` in `regress`), so no model package depends on another model package, a new estimator has every generic method on its first day, and `plot` adds `.plot()` to every result.
6. **One home per name per type.** A name may be defined for several types of first argument, once per type, and the type of the first argument chooses the definition when the script is checked: `summary` (frames in `data`, results in `results`), `quantile` (columns in `data`, distributions in `dist`), `append` (lists in `core`, frames in `data`), `min` and `max` (several values row by row in `core`; one column in `data`), `plot` (frames and results in `plot`, causal graphs in `identify`), `histogram`, `density` and `boxplot` (frames as shorthands, plots as layers, both in `plot`), `aggregate` (frames in `data`, plots in `plot` from 1.2), and the recipe steps `impute`, `pca`, `standardize`, `center`, `bin`, `winsorize` and `add` (recipes in `learn`; frames or columns in `data` and `explore`). Two definitions whose types overlap, such as one for every `Result` and one for `IvFit`, are not allowed, which is why the instrument check is `exclusion_check` rather than a second `check`. A project function that reuses a built-in name is written in full, as `functions::name`. A package is never named after a frame verb or one of the shared arguments of section 6 of the Language Reference (`by:`, `group:`, `cluster:`, `fe:`, `re:`, `se:`, `weights:` and the rest), so `cluster:` and `panel()` keep their one meaning each; an ordinary argument such as `time:`, `data:` or `text:` may share a package's name, since an argument name means something only before a colon.
7. **Packages ship with the engine and load on use.** All packages are available without imports; pinning the engine version in `zeff.toml` pins every function. The browser loads `core`, `text`, `time` and `data` at start and fetches the rest on first use, and bundles only packages that have shipped a function. `zeff help` and Zemp's menus list a package, and a research goal, only once at least one of its functions has shipped; the rest appear in a roadmap view with their planned release, so an empty package is never a menu entry.
8. **The engine kernel is not a package.** It is Rust code every package uses and scripts cannot call. Its modules are fixed here, and adding one is a change to this specification, so the kernel cannot grow silently as the escape from rule 3: column store and selection vectors · NA rules · date parsing · compensated sums · linear algebra (`zeff-kernels`) · special functions · random streams (Philox) · resampling · design-based variance · the likelihood optimiser · thread pool (from 1.1) · display registry · function registry.
9. **Third-party packages (after 1.0) are qualified, declared and locked.** A third-party package is a WebAssembly component (section 8.3, point 6) with a manifest: name, version, tier (at or above every package it calls), purpose tags, and for every function its accuracy class and reference test. Its functions are always written qualified, `pkg::fn`, unless `use pkg` brings them in, and `use` refuses a name that collides with a built-in or with another imported package (Z0107), so no third-party function ever shadows a built-in and rule 6 holds across the whole namespace. `zeff.toml` names each package with a version requirement; `zeff.lock` records the exact version and content hash resolved, and a project runs from the lock, never from the requirement (Z0703 when an installed package differs from it). A package earns a *validated* badge only when its reference tests pass in Zeff's own continuous integration; Zemp shows the badge beside the package and marks results from unvalidated packages in the run record.

### 8.2 The packages

Counts are functions in 1.0 / proposed for 1.0 (none since 16 October 2026) / on the roadmap after 1.0 (zeff_functions.csv has every one).

**Tier 0: centre**

| Package | Objective | Focus | 1.0 / proposed / roadmap |
| --- | --- | --- | --- |
| `core` | The language and the types every package shares | Values, types, collections, missing values, maths, program control; matrices from 1.8 | 68 / 0 / 3 |

**Tier 1: toolkit**, general machinery every method uses

| # | Package | Objective | Focus | 1.0 / proposed / roadmap |
| --- | --- | --- | --- | --- |
| 1 | `time` | Work with dates and times | Parsing, date parts, date arithmetic | 14 / 0 / 2 |
| 2 | `text` | Work with text | Matching, replacing, splitting, formatting numbers and dates | 13 / 0 / 0 |
| 3 | `data` | Make any dataset ready for analysis | Frames, verbs, panels, reducers, files | 59 / 0 / 6 |
| 4 | `dist` | Probability distributions | Twelve families, the studentized range, truncation and mixtures; `pdf`, `cdf`, `quantile`, `draw` | 19 / 0 / 0 |
| 5 | `results` | Every result looks and behaves the same | Result interface, estimator interface, publication tables, export | 17 / 0 / 2 |
| 6 | `stats` | Classical inference | Correlation, tests, post-hoc comparisons, power, effect sizes, smoothing | 13 / 0 / 10 |
| 7 | `optim` | Fit anything defined by an objective | Minimisers, MLE and GMM frameworks, mathematical programming | 0 / 0 / 11 |
| 8 | `resample` | Uncertainty and validation by resampling | Bootstrap, jackknife, permutation, holdout splits and cross-validation folds | 0 / 0 / 7 |
| 9 | `survey` | Design-correct estimates from survey data | Design declaration, replicate weights, calibration | 0 / 0 / 8 |
| 10 | `plot` | Draw anything the same way | Layered plot grammar and renderer (section 7.6 of the Language Reference), shorthands, result plots | 35 / 0 / 6 |
| 11 | `datasets` | Data for examples, tests and users | Shipped public and synthetic datasets with help pages; the `zeff:` data root (section 8.5) | 2 / 0 / 0 |
| 12 | `web` | Data from APIs, reproducibly | `fetch` with caching, pagination and recorded provenance; secrets kept out of projects (section 8.6) | 0 / 0 / 4 |

**Tier 2: models**, which fit a model and return a result

| # | Package | Objective | Focus | 1.0 / proposed / roadmap |
| --- | --- | --- | --- | --- |
| 1 | `regress` | Linear and IV regression | OLS, fixed effects, IV, event-study terms, linear-model diagnostics, wild bootstrap, quantile regression | 8 / 0 / 7 |
| 2 | `glm` | Non-linear outcomes | Binary, count, ordinal, multinomial, censored and selection models | 2 / 0 / 12 |
| 3 | `multilevel` | Repeated and nested data | Random effects (`re:`), mixed and growth models, dynamic panels | 0 / 0 / 6 |
| 4 | `explore` | Find structure before testing | Dimension reduction, missingness, imputation and outlier checks | 0 / 0 / 9 |
| 5 | `series` | Time series and forecasting | ARIMA, ETS, VAR, state space, dynamic factors | 0 / 0 / 7 |
| 6 | `latent` | Latent-variable models | CFA and SEM (1.3); EFA, ESEM, latent classes and profiles | 0 / 0 / 18 |
| 7 | `measure` | Reliability and validity of measures | Reliability, validity, invariance, IRT, agreement | 1 / 0 / 13 |
| 8 | `learn` | Leakage-safe prediction | Recipes, model specifications, supervised learners, tuning, metrics, explanation | 0 / 0 / 32 |
| 9 | `groups` | Hidden groupings | Clustering, mixtures, topic models, communities | 0 / 0 / 7 |

**Tier 3: designs**, which use fitted models to answer a research question

| # | Package | Objective | Focus | 1.0 / proposed / roadmap |
| --- | --- | --- | --- | --- |
| 1 | `identify` | Say what a causal question needs before anything is fitted | Causal graphs, adjustment sets, implied independencies, checks of fitted models and instruments against a graph, graph drawing | 6 / 0 / 0 |
| 2 | `experiment` | Design and analyse randomised trials | Designed experiments, randomisation, balance, variance reduction, randomisation inference, non-compliance | 0 / 0 / 9 |
| 3 | `quasi` | Natural-experiment designs | DiD and event studies, RDD, synthetic control, shift-share | 0 / 0 / 10 |
| 4 | `adjust` | Selection on observables | Matching, weighting, double machine learning, sensitivity | 0 / 0 / 12 |
| 5 | `mediate` | How and why an effect happens | Indirect effects, moderated mediation, front-door | 0 / 0 / 5 |
| 6 | `hetero` | For whom and when | Moderation, conditional average treatment effects | 0 / 0 / 10 |
| 7 | `structural` | Counterfactual policy simulation | SMM, discrete choice and demand, dynamic models, simulation, welfare | 0 / 0 / 11 |
| 8 | `decide` | What should be done | Policy learning, treatment regimes, bandits, operational decisions | 0 / 0 / 8 |
| 9 | `meta` | What the evidence says | Meta-analysis, publication bias, specification curves | 0 / 0 / 13 |

**Tier 4: extensions.** The project's own functions in `functions/` (they may call every package) and, after 1.0, third-party packages under rule 9.

Totals: 257 functions in 1.0 (68 in tier 0, 172 in tier 1, 11 in tier 2, 6 in tier 3), of which 227 are written in Rust and 30 in Zeff; none remain proposed, and 248 are on the roadmap, 505 in all. Eighteen packages have no function in 1.0 (four in tier 1, six in tier 2 and every tier-3 package but `identify`): they give the roadmap its structure and stay out of menus until they ship (rule 7). Most 1.0 functions are small; what limits the schedule is the seven hard engine pieces listed in section 14.

### 8.3 Engine architecture

The packages sit on an engine built so that the principles of section 1.2 hold by construction rather than by care. Ten decisions define it.

```
script ──► parser ──► typed IR ──┬──► zeff check · language server · Zemp dialogs
                        ▲        └──► planner ──► executor ──► results (versioned schema)
          file schemas ─┘                           │   ▲
                                                    ▼   │
                                   estimators ◄── variance layer      result cache
                                        │                              (content-addressed)
                                   kernel: columns · linear algebra · special functions · RNG
```

**1. One typed intermediate representation.** A script is parsed once and lowered to a typed IR in which every name is resolved (column, binding or function), every argument is checked against its parameter type, and every frame carries its schema. `zeff check`, the language server, the interpreter, the planner and Zemp's dialogs all read this IR and nothing else, so what the editor accepts is exactly what the engine runs. A diagnostic is data on the IR (code, span, message, fixes) and is printed by the command line, underlined by the editor and shown by Zemp from the same record.

**2. Estimators and a separate variance layer.** Every model implements one Rust trait:

| Step | What an estimator provides |
| --- | --- |
| `design` | The model matrix and the rows used, with dropped rows counted |
| `fit` | Estimates, and convergence information for iterative fits |
| `scores` | Each observation's contribution to the estimating equations |
| `bread` | The Hessian or its equivalent |

Standard errors are computed by one variance layer from `scores` and `bread`: classical, HC1 to HC3, clustered (CR1) and, from 1.1, bootstrap. So `se:`, `cluster:` and `weights:` work identically for every model without each one coding them, and a new estimator gets every variance option on its first day. Clustering is one *dependence structure*: a rule giving, for each pair of observations, the weight of their score product in the middle of the sandwich (1 within a cluster, 0 across). Two-way clustering, Conley spatial errors (a kernel of distance with a cutoff) and network-robust errors (adjacency) are further structures, each a few lines in the variance layer and available to every model at once (section 14.1). Likelihood models (logistic, probit, then Poisson, SEM and mixed models) also implement a likelihood trait and share one optimiser, one convergence report and one sandwich.

**3. Floating-point determinism.** "Identical results on every machine" is a contract with six parts:

| Part | Rule |
| --- | --- |
| Arithmetic | IEEE 754 binary64 in round-to-nearest-even, never changed, with subnormals never flushed to zero; no fast-math and no fused multiply-add unless written explicitly; the same operations in the same order on every target |
| Elementary functions | `exp`, `log`, `sin` and the rest come from one pure-Rust implementation compiled into the engine, never the platform's C maths library, whose results differ between systems |
| Reductions | Sums, dot products and matrix products use a fixed blocking and reduction order, whatever the SIMD width or thread count |
| Linear algebra | A kernel crate of Zeff's own (`zeff-kernels`): tall-skinny Householder QR, Cholesky, triangular solves, symmetric eigendecomposition, matrix products and compensated reductions, written with explicit 128-bit SIMD (SSE2, NEON and simd128, present on every supported target) and fixed block sizes; no run-time feature dispatch, no system BLAS or LAPACK. Third-party linear-algebra crates (such as `faer`) are used only in tests and build tooling, never on a path that produces a result, because they choose SIMD width at run time |
| Random numbers | A counter-based generator, Philox4x32-10: each draw is a function of (seed, the ordinal of the drawing statement within the run, loop pass, draw number), never of thread scheduling, so parallel and serial runs give the same draws. As in R and Stata, inserting or removing a statement that draws random numbers changes the draws of the statements after it; two identical statements draw different numbers. A uniform Float is built from two 32-bit outputs (53 bits), and every distribution draws by inverse CDF from the uniform stream (`dist.mixture` uses two uniforms), so draws are a pure function of the stream and the algorithm changes only with a note in the known-issues list. The seed chosen when none is given comes from the operating system's entropy and is recorded; `seed` may appear once, before any draw (Z0111) |
| Proof | Continuous integration runs the validation suite on WebAssembly, on two x86-64 machines of different SIMD levels (one without AVX-512) and on ARM64, and requires bit-identical output. `zeff selftest` runs the same cases' published checksums on the user's machine (section 2.1), so any installation, including a source build, can show that it is conformant. Official x86-64 builds target the baseline instruction set (SSE2), since the kernels use nothing wider |

**4. Content-addressed incremental runs.** Because bindings never change and every input file is hashed, each statement's result is stored under a hash of its IR, the IR of every project function it reaches, the hashes of its inputs, the engine version and edition, and the seed. Re-running a script recomputes only statements whose code or inputs changed; a statement that depends on `today()` or `now()` is never cached (section 3.8 of the Language Reference); the cache lives in `.zeff/cache/` as Arrow IPC files (section 8.7), can be cleared at any time, and is never needed for correctness. `zeff run --fresh` ignores it, and the run record says which results came from the cache. Files are hashed with BLAKE3, on all cores; a file whose size and modification time are unchanged since it was last hashed keeps its hash, as the schema cache keeps its schema, and `--fresh` and `zeff verify` hash everything again.

**5. Stable formats.** Everything that leaves the engine has a versioned schema: frames as Arrow IPC, whose fields carry each column type's extension name and version and the column's metadata (section 4.3 of the Language Reference), results as Arrow tables plus a JSON description of their type, plot scenes as a JSON description of marks, scales and layout (section 7.6 of the Language Reference), and run records in `.zeff/runs/`; what `save` writes as Parquet follows the Zeff Parquet profile (section 8.7), a public page that names every key and type the file carries. Zemp, exports and other tools read these formats, never engine internals, so the engine can change freely behind them. Syntax changes after 1.0 come only with a new edition (section 2), and anything removed is deprecated with a warning for at least one edition first.

**6. Extension boundary.** After 1.0, third-party packages will be WebAssembly components that run sandboxed: no network or file access, only the frames and arguments passed to them, and the same estimator, variance and result interfaces as built-in packages. First-party packages are built against these interfaces from the start, so a built-in package is simply a package that ships with the engine, and the privacy promise of section 1.4 covers extensions too. Each component carries the manifest of section 8.1, rule 9 (name, version, tier, purpose tags, and the accuracy class and reference test of every function); `zeff.toml` names the packages a project uses and `zeff.lock` the exact versions resolved, so a replication package pins its extensions as exactly as its engine.

**7. Resource governance.** The engine runs in its own worker (a Web Worker in the browser, a thread natively), so the interface never freezes. Long operations report progress and can be cancelled between steps. Each window has a memory budget. In the browser the default is the smaller of 3.5 GB and half the memory the device reports; a project can set its own with `memory = "3.8GB"` under `[project]` in `zeff.toml`, up to what WebAssembly allows, so a collaborator on a smaller machine gets the same limit and a clear error rather than a crash. The desktop app uses available memory minus a safety margin. The setting is a budget, not the WebAssembly limit, so it carries over unchanged when 64-bit WebAssembly lifts the 4 GB ceiling. A step whose estimated memory would exceed the budget stops before it starts, with an error that gives the estimate and suggests the desktop app or a narrower `select`. The result cache lives on disk (the project folder, or the browser's private file system) and never in the memory budget; when space runs short, the least recently used results are removed first. A memory-mapped file (section 8.7) counts by the pages a step has touched, never by its size, so a 10 GB file is not refused by a script that reads three of its columns. The estimate is a step's peak allocation, not its steady state, because WebAssembly memory grows in pages and never shrinks; the engine worker is restarted when a project closes, which returns its memory to the browser.

**8. Interoperability at the data boundary.** Zeff exchanges data with other tools, never memory. Frames, with their labels and categories, and results, through `fit.estimates()`, save to and load from Arrow IPC and Parquet with nothing lost, so R (`arrow`), Python (`pyarrow`, `polars`), DuckDB and Stata (through Parquet) read Zeff's output directly, and read its labels and categories as their own through the Zeff Parquet profile (section 8.7). From 1.1, the desktop app adds one opt-in step for methods Zeff lacks: `external("r", script: "scripts/rdd.R", input: frame, output: "output/rdd.parquet")` runs a local R or Python script on a frame passed as Parquet and imports the frame it writes. It is enabled per project (`external = true` under `[project]` in `zeff.toml`), while the interpreter's path is a user setting outside the project (`~/.zeff/config.toml`, or Zemp's settings) and is never read from the project, so opening a downloaded project cannot run a program of its author's choosing; the user confirms the first external step of each project. It is never available in the browser, and the run record marks every result that depends on it as outside Zeff's validation and bit-identical guarantees, recording the interpreter's version, its package versions (`sessionInfo()` or `pip freeze`) and the script's hash. `zeff run --no-external` refuses such steps, for checking a replication package, and the package export lists every external step on its first page. There is no shared memory, foreign function interface or embedded interpreter: each would break the browser build, determinism and the privacy promise.

**9. Automatic differentiation (after 1.0).** Built-in estimators use analytic derivatives written in Rust. User-written objectives in `mle`, `gmm` and `minimize` get forward-mode automatic differentiation on the typed IR: because Zeff functions are pure and typed, each such function is differentiated and compiled into one kernel instead of being interpreted at every evaluation, and `gradient` and `hessian` become exact rather than numerical.

**10. Performance design.** Speed comes first from avoiding work, then from doing it efficiently; none of these measures changes a result.

| Measure | What it does | Release |
| --- | --- | --- |
| Tall-skinny QR | OLS and IV process the design in row blocks of at most one million rows: each block's R factor is combined with the next, no n × k matrix is ever allocated, residuals and fitted values are `y − Xβ`, and leverage comes from R. The same path serves files larger than memory later | 0.4.0 |
| Text columns | 64-bit offsets (`LargeUtf8`) or chunked arrays, so no single text column is limited to 2 GB | 0.2.0 |
| Import cache | The first import of any file that is not Parquet or Arrow stores an Arrow IPC copy in `.zeff/cache/`, keyed by the file's hash and the import options; later runs memory-map the copy on the desktop (LZ4-compressed in the browser), typically 10–50 times faster than parsing, and a changed file or option is parsed again (section 8.7) | 0.11.0 |
| Deterministic Parquet | `save` writes Parquet with fixed settings, statistics and a page index, and a panel sorted by unit and time, so a file is byte-identical on every target and a `where` on the sort key skips whole row groups (section 8.7) | 0.11.0 |
| Fast hashing | BLAKE3 on all cores for every input, cached copy, output and fetched response; a file whose size and modification time are unchanged keeps its last hash (section 8.7) | 0.2.0 |
| Reused fixed-effect work | Group indexes and demeaned columns are cached under (fixed effects, rows used), so a robustness table of models sharing `fe:` and sample demeans once | 0.6.0 |
| Reused model matrices | The same formula on the same rows reuses its model matrix and QR factorisation, so changing only `se:` or `cluster:` costs almost nothing | 0.4.0 |
| Fused column expressions | A column expression such as `log(wage) - mean(lwage)` is compiled from the typed IR into one pass over the data, with no temporary columns | 0.6.0 |
| Integer-coded categories | Categories, and text used as keys, are stored as integer codes, so grouping, fixed effects and merges compare integers | 0.4.0 |
| Panel indexes | `panel()` sorts once and keeps its index for every `lag`, `diff`, `by:` and `fe:` on that panel | 0.6.0 |
| Sufficient statistics and warm starts | X′X and X′y are computed once for estimates, standard errors and tests; iterative fits start from a nearby solution (logistic from the linear-probability fit, bootstrap draws from the full-sample estimate); adding a term updates a QR factorisation instead of recomputing it | 0.4.0 to 0.9.0 |
| Sparse structures | Fixed-effect indicators, random-effect design matrices and influence functions stay sparse | 0.6.0, then as each method arrives |
| Rows ordered by the largest fixed effect | Group sums are scatter-adds by group index, and with a million firms the sums array misses the cache on every add; sorting rows once by the largest fixed effect (free when it is the panel unit) makes that effect's accumulation sequential | 0.6.0 |
| Read only what is needed | Lazy chains read only the columns a chain uses from Parquet and skip row groups a `where` rules out | 1.1 |
| Deterministic parallelism | Rows split into fixed blocks whose partial results are always combined in the same order, so 1 or 64 threads give identical results; first used for separate models, bootstrap draws, DiD cells and CSV parsing | 1.1 |
| Browser transfers | Results move from the engine worker to the interface as Arrow buffers without copying; packages are optimised and compressed and load on first use | 0.7.0 |
| Benchmarks in CI | A benchmark suite runs on every build and fails it when a case slows by more than 5%; cases include two-way fixed effects on 10 million rows, double lasso on a 2 GB file and Callaway–Sant'Anna DiD on 4 million panel rows, each compared with the fastest reference tool (fixest, data.table or Polars, hdm, did) | 0.4.0 onwards |

Determinism has a price: fixed blocking and no system BLAS cost about 20–40% on the largest matrix products. The measures above recover far more, because they remove work rather than speed it up.

### 8.4 Packages from outside: format, registry, installing and publishing (after 1.0)

Rule 9 of section 8.1 fixes how a third-party function is named and locked. This section fixes what a package is, where it comes from and how it is trusted. Packages ship in 1.11 (section 14); the format is fixed now so that the first-party packages are built to it and a project's `zeff.toml` needs no change later.

**Two kinds of package, one contract.**

| Kind | Contents | Who writes it |
| --- | --- | --- |
| Zeff-source package | `package.toml`, `functions/*.zeff` with `///` help, one function per file (section 9.1, Files), `tests/`, optional `data/` and `articles/` (runnable vignettes, checked by `zeff test`) | Statisticians: the equivalent of an R package written in R, interpreted by the engine and sandboxed by the language itself |
| Component package | The same files plus a WebAssembly component built from Rust or any language that compiles to the component model, using the kernel's linear algebra and the estimator, variance and result interfaces | Engineers writing compiled methods |

Both carry the help contract, the manifest and the conformance tests, so a package function gets a dialog, a help page and an accuracy line exactly like a built-in.

**The manifest** (`package.toml`): `name` (lowercase, one word, never a built-in package's name or a reserved word), `version` (major.minor.patch), `tier` (at or above every dependency), `purpose` tags, `engine = ">=1.0, <2.0"`, `edition`, `license` (an SPDX identifier), `authors`, `[dependencies]` on other packages with version requirements, `effects = "none"` (the only value), and for every function its accuracy class and reference test. A package may ship data: its `data/` folder is a data root named after it, `import("survival:lung.csv")`, as the engine's own examples use `zeff:` (section 8.5).

**Importing.**

```toml
[packages]
survival = "^1.2"                                             # resolved to 1.2.4 and hashed in zeff.lock
pls_sem  = { git = "https://github.com/.../pls-sem", tag = "v0.9.0" }
mypkg    = { path = "../mypkg" }                              # local development
```

```
use survival                      // every public name; refused if one collides (Z0107)
use pls_sem::pls                  // one name
let fit  = lung.coxph(time ~ age + sex, event: status)                 // after `use survival`
let fit2 = lung.survival::coxph(time ~ age + sex, event: status)       // always available, qualified
```

`use` lines come before the first statement and apply file-wide; a qualified name works in method position (`"." qname "("` in section 3.4 of the Language Reference). One version of a package per project, so the flat namespace holds; a package may not override a built-in or another package's name; a package's functions may be methods on built-in types (any function whose first parameter is a `Frame` is a frame method) and must implement the result and estimator interfaces for their own result types, which is what makes `report`, `plot` and Zemp's dialogs work for them.

**Installing and locking.**

| Command | What it does |
| --- | --- |
| `zeff add <package>[@req]`, `--git <url> --tag <t>`, `--path <dir>` | Resolves the newest version that meets the requirement and the engine range, downloads it, checks the hash and the index signature, writes `zeff.toml` and `zeff.lock` |
| `zeff update [package]` | Moves packages to the newest versions their requirements allow and rewrites the lock, showing what changed |
| `zeff remove <package>` | Removes a package from `zeff.toml` and the lock; `zeff check` then reports every call that used it |
| `zeff packages` | Lists the project's packages with version, licence, validated badge and the functions each provides |
| `zeff vendor` | Copies the packages a project uses into `.zeff/packages/`, so a replication package needs no network; `zeff run` prefers the vendored copies |
| `zeff publish` | Runs the help contract and the conformance tests locally, builds the component if there is one, signs the archive and submits it to the registry |

A run uses the lock, never the requirement (Z0703). The desktop stores packages in `~/.zeff/packages/<name>/<version>/`; the browser caches them in its private file system, fetched on first use, and nothing but the package name leaves the machine.

**The registry.** A signed, static index in the style of Cargo's sparse registry, served from a static host, with package archives on a release host: it costs nothing to run. Each index entry carries the version, the hash, the engine range, the licence and the badge. The registry's own continuous integration re-runs a submitted package's conformance tests on all four targets (section 13) before it grants the *validated* badge; a package whose tests do not run, or fail, is listed without the badge, and results from it are marked in the run record. A yanked version stays downloadable for existing lock files, is never chosen anew, and `zeff audit` flags the runs that used it.

**Trust and determinism.** Components run sandboxed: no network, file or clock access, only the frames and arguments passed to them (section 8.3, point 6). Zeff-source packages inherit the language's own restrictions and may not call `external`. Components run as WebAssembly natively as well as in the browser, which makes their floating point bit-identical by construction, so the determinism contract covers them without a special case.

### 8.5 The `datasets` package

`datasets`, last in tier 1, ships every dataset the help pages, tests and samples use, and lets a user load one by name. It is also the package behind the `zeff:` data root, so the two spellings are one thing:

```
let wages = dataset(wages)                 // by name, with labels, categories and panel declaration
let wages = import("zeff:wages.csv")       // the data root the help pages use; Parquet is also there
datasets()                                 // name · rows × columns · kind · licence · source · size
```

**Two kinds of dataset, one rule each.**

| Kind | Rule |
| --- | --- |
| Public, open access | Redistributable under a stated licence: CC0, CC-BY 4.0, ODC-BY or ODbL, or public-domain government data. Each ships with its source URL, retrieval date, checksum, citation and the cleaning script that turned the raw file into the shipped one, so the derivation is reproducible. No "licence unknown", no scraped data, nothing beyond what the source itself publishes |
| Synthetic | Generated by a Zeff script shipped in the package, with its seed, so the data re-generate bit for bit; the true parameters are documented (the synthetic `wages` has a return to education of 0.074), which gives help examples stable numbers and truth-recovery tests a known answer |

**Help.** Every dataset has a help page under the contract of section 9.1, in the shape R gives `?mtcars`: Summary, Description, Format (each variable's name, type, label, units and missing codes, generated from the file's schema), Source or Generation, Licence and citation, References, Examples. `zeff help wages` prints it; Zemp shows it in the variable view. A dataset without a complete page does not ship.

| Rule | Detail |
| --- | --- |
| Immutability | A dataset never changes within an engine version; a corrected one gets a new name (`wages_v2`), because a replication package depends on the old |
| Size | Datasets under a megabyte, the ones examples and tests use, ship inside the engine; larger ones are fetched on first use from the registry, hash-checked and cached in `~/.zeff/datasets/` or the browser's private storage; `zeff vendor` copies the ones a project used into its replication package, and `zeff dataset fetch` pre-fetches them for a machine without network (section 2.1) |
| Formats | Each dataset is stored as Parquet with labels and categories, and as CSV; `dataset(name)` reads the Parquet |
| The 1.0 set | Synthetic: `wages`, `firms`, `staff`, `counties`, `loans`, `engagement`, `pupils`, `sims`; public: `sleep` (Student 1908) and others as their licences are verified; candidates from applied work enter with their cleaning scripts once their licences are confirmed |

### 8.6 The `web` package: data from APIs (from 1.1)

Zeff scripts reach the network only through `fetch`, and only in a project that allows it (`network = true` under `[project]` in `zeff.toml`). The design is the discipline of R's `pins` and `targets`: a request is made once, recorded and cached, and the script then runs from the cache like any other file, so a fetched dataset is as reproducible as an imported one.

**Sources.** `fetch` is the first of a family. Anything a project reads from outside its folder and data roots, an API, a database server, a survey platform or a cloud folder, is a *source*, and every source follows the same four rules: it is enabled per project in `zeff.toml`; credentials come only through `secret()`; every response is hashed, cached and recorded in the run record; and it is replayed by default, fetched again with `--refresh` and forbidden with `--offline`. Database queries and survey-platform downloads arrive as further source functions after 1.0 (section 14.1), each with its own key under `[project]` beside `network`.

```
let gdp = fetch("https://api.worldbank.org/v2/country/all/indicator/NY.GDP.MKTP.CD",
                params: {format: "json", per_page: 100, date: "2000:2024"},
                as: json, records: "1")                  // the array of records in the response
    .select([date, value])

let prices = fetch("https://api.example.org/resource/...",
                   params: {`api-key`: secret("DATA_API_KEY"), format: "csv", limit: 1000},
                   as: csv, pages: {param: offset, step: 1000, until: empty})
```

| Function | Rule |
| --- | --- |
| `fetch(url, params: Record, headers: Record, body: Record, as: json \| csv \| text \| parquet, records: Str, pages: Record, cache: Bool = true)` | A GET request, or a POST when `body:` is given. The response is stored in `.zeff/fetched/` under its hash and parsed by the `import` reader for its format (`records:` as for JSON files). `pages:` follows offset, page-number or cursor pagination into one frame. The run record stores the URL with secrets removed, the parameters, the time, the status and the response's hash |
| `secret(name)` | Reads a key from the system keychain or `~/.zeff/secrets.toml`, never from the project; its value never appears in a script, a printed value, a run record or a replication package |
| `flatten(frame)` | Turns record columns from JSON into dotted columns, with the rules `import` uses |
| `rate_limit(per_second: Float)` | Limits requests for the rest of the run, so a loop over pages respects the server |

| Rule | Detail |
| --- | --- |
| Replay | Later runs read the cache; `zeff run --refresh` fetches again, and `--offline` refuses any request not cached. A changed response is a note with both hashes |
| Replication | `zeff vendor` copies fetched responses into `data/fetched/`, readable as `import("fetched:gdp.parquet")`, so a replication package carries the data rather than depending on a server that may change |
| Marking | Results that depend on a fetch are marked in the run record, as results that depend on `external` are |
| Browser | `fetch` works where the server allows cross-origin requests; otherwise Zemp's "Add from URL" fetches through the desktop app or asks for the file |
| Scope | GET and POST with JSON, CSV, text and Parquet responses; no other methods, sockets or streaming. API wrappers (World Bank, IMF, OECD) are ordinary packages built on `fetch` and come through the registry (section 8.4), not the engine |
| Errors | A network failure, a status other than 2xx, or a response that does not parse is an error naming the URL and status (Z0708); `attempt` catches it like any other |

---

### 8.7 Storage: working files, kept files and the Parquet profile

Zeff has no file format of its own. Data on disk is in one of two open formats, each used where it is fastest, and nothing an analysis produces can be read only by Zeff. A format of Zeff's own could not load faster than Arrow IPC, which is the engine's memory layout, nor be smaller than Parquet without years of work, and it would tie every replication package to one program, which is what `.dta`, `.sav` and `.RData` do and what principle 5 forbids.

| Files | Format | Rule |
| --- | --- | --- |
| Working files: the import cache and the result cache in `.zeff/cache/`, read on every run and never shared | Arrow IPC | The in-memory layout written to disk. On the desktop and the command line the file is memory-mapped, so a frame of several gigabytes opens at once and only the columns a script reads are paged in, with nothing to decode; in the browser, where mapping is not available, the buffers are LZ4-compressed, which decodes at memory speed and keeps the private file system small |
| Kept files: `save`, exports, replication packages, the shipped datasets | Parquet, zstd-compressed | Several times smaller than Arrow IPC, which matters for disk, downloads and journal uploads; read directly by R, Python, DuckDB and Stata; decoded once per import, into the cache |

**The import cache.** The first import of any file that is not Parquet or Arrow (delimited text, fixed width, Excel, OpenDocument, Stata, JSON, and from 1.1 SPSS, SAS, R and HTML) stores an Arrow IPC copy in `.zeff/cache/`, keyed by the file's hash and the import options; later runs read the copy, and a changed file or a changed option is parsed again. Parquet files are read in place, with column projection and the row-group skipping below. Stata, SPSS and SAS files are therefore parsed once: `save` still writes them, for colleagues who work in those programs, but Zeff never works from them.

**Parquet writing.** Every file `save` writes is deterministic: fixed writer settings (zstd at level 3, row groups of 1,048,576 rows, dictionary encoding for text and categories, statistics and a page index for every column), so the same frame gives byte-identical files on every target and `zeff verify` compares output files by hash (section 2.1). A panel is written sorted by unit and time, and any other frame in the order it has, so the row-group statistics let a `where` on the sort key skip whole row groups: from 1.1, lazy chains read only the columns a chain uses and the row groups a `where` cannot rule out. The file carries the column metadata store and the extension types (section 4.3 of the Language Reference) under the profile below.

**The Zeff Parquet profile.** A public, versioned page (the file's key–value metadata holds `zeff.profile = "1"`) that fixes what a Zeff-written Parquet or Arrow file carries beyond plain Parquet: the frame keys (`zeff.panel`, the unit and time variables; `zeff.source`; `zeff.engine`); the column keys of section 4.3 of the Language Reference, each stored as JSON in the field metadata under `zeff.<key>` (`label`, `value_labels`, `missing_codes`, `measure`, `unit`, `question`, `source`, `format`, `crs`); the category encoding (an Arrow dictionary with `zeff.ordered` and `zeff.contrast`); and the extension type names and versions of the opaque types, using community names where they exist (`geoarrow.wkb` for `Geometry`) and `zeff.` names otherwise (`zeff.fileref`, `zeff.period`). A reader that does not know a key ignores it, and a file without the profile key is ordinary Parquet and imports like one from any other tool. Readers that turn the profile into `haven` labels and factors in R, or categoricals in pandas, are small packages outside the engine (section 14.1, Ecosystem). The page is published with 0.11.0, when the Parquet reader and writer arrive, and its version changes only with an edition.

**Hashing.** Every input file, cached copy, output file and fetched response is hashed with BLAKE3, which runs on all cores and is several times faster than SHA-256 on large files; the run record stores the hash with the algorithm's name, so a record stays readable if the algorithm ever changes, which it does not within edition 2027. A file whose size and modification time are unchanged since it was last hashed keeps its hash, as the schema cache keeps its schema (section 11 of the Language Reference); `zeff run --fresh` and `zeff verify` hash everything again. Engines, packages and archives keep their SHA-256 checksums and minisign signatures (section 2.1), which are a different matter: trust, not change detection.

**Memory accounting.** A memory-mapped file counts against the memory budget (section 8.3, point 7) by the pages a step has touched, not by its size, and the estimate made before a step counts the columns the step will read, so a 10 GB file on the desktop is not refused by a script that reads three columns of it. In the browser nothing is mapped, and a cached copy counts in full once decoded.

**Results, and several objects in one file.** `save` writes one frame. Fitted results need not be saved in 1.0: the result cache and `zeff run` reproduce them exactly, and `fit.estimates()` saves what a table or a plot needs. Whether a file holding several frames and results together arrives with the heavy models of 1.5 (a zip of Arrow files and JSON result descriptions in the versioned form of section 8.3, point 5, never a binary format of Zeff's own) is an open decision (section 15).

---

## 9. Function catalogue

Each package's functions by group. Usage, behaviour, R and Stata equivalents, purpose tags, release, status and implementation language for each are in zeff_functions.csv; the conventions there are: usage is written `name(required, optional: Type = default)`; `a | b` lists the allowed values; a leading `data.` marks a frame method, `fit.` a method on any result, and `iv.`, `did.`, `lasso.`, `forest.`, `sem.`, `lca.`, `fits.`, `med.`, `matching.` and `meta.` a method on that result type only; `g.` marks a method on a causal graph and `distribution.` one on a distribution.

### 9.1 The help contract

Every function in this catalogue, built in or written in a project module, ships with a help page, and a function without a complete one does not exist: the engine does not build with a built-in function whose page is incomplete, and `zeff check` fails a module in `functions/` whose function lacks one (Z0319). The page follows R's manual pages, the best in the field, with two differences. Usage and Arguments are generated from the signature, so the help can never disagree with the code (R's "codoc mismatch" cannot happen), and every example is run by the test suite, so the help can never go stale. `zeff help reg` prints the page; the editor shows the summary in completion and the page on hover (section 12); Zemp builds each dialog's labels and tooltips from Arguments and offers the examples as "Try it"; the reference manual on the website is generated from the same files. One source, four views.

| Part | Content | Source | Required |
| --- | --- | --- | --- |
| Summary | One line under 80 characters that completes "`reg` — …" | Written | Always |
| Usage | The signature with defaults, in method form (`data.reg(...)`); several forms when a function has them (`t_test`) | Generated | — |
| Description | What the function does and how, in one to three paragraphs, and what differs from R and Stata | Written | Always |
| Arguments | One entry per parameter in signature order: type, default and meaning. A shared argument (section 6 of the Language Reference) takes its text from the shared table, so `se:` reads the same everywhere, and the function may add one sentence | Written per parameter; shared text inserted | Every parameter |
| Value | The type returned and what it holds; for a result type, the methods that apply to it | Written; the method list generated from the interfaces (section 8.1, rule 5) | Always |
| Details | Algorithm, formulas, edge cases, canonical order | Written | Statistical methods (closed form, distribution, iterative and stochastic classes) |
| Warnings and notes | Every diagnostic the function can raise and when | Generated from the diagnostics the implementation declares, and checked both ways: a code raised but not declared, or declared but never raised, fails the build | — |
| Accuracy | Accuracy class, reference software with versions, datasets, and the latest conformance result | Generated from the catalogue and the conformance suite (section 13) | — |
| References | The definition implemented, with chapter, page or equation | Written | Statistical methods |
| See also | Related functions, checked against the registry | Written | Always |
| Examples | Statements with their printed output, on the example files that ship with the engine (`import("zeff:wages.csv")`, a built-in data root); `zeff test` runs a project's, the conformance suite runs the built-ins', and the output is compared | Written | At least one |
| Availability | Package, tier, release, status, purpose tags | Generated from the catalogue | — |

**Sources.** A built-in function written in Rust has one Markdown file per function in its package crate (`regress/help/reg.md`), with the parts above as headings; paragraphs, `code` and bullet lists are the only markup. The build compiles every file into the function registry (section 8.1, rule 1) and fails on a missing file or heading, an argument without an entry or an entry naming no parameter, a missing reference for a statistical method, an unknown name under See also, an example that fails or prints something else, or a diagnostic mismatch. A function written in Zeff, built in or in a project, carries the same content in `///` lines (section 5.3 of the Language Reference): the summary line, description paragraphs, one `parameter: text` line per parameter, `returns: text`, optional `details:`, `reference:` and `see also:` lines, and `> statement` examples followed by their output. For a module in `functions/`, `zeff check` requires the summary, every parameter, `returns:` and one example (Z0319); a `fn` inside a script needs only the summary line (W0303 without it), since Zemp uses it for the function's dialog. A help line that names a parameter the function does not have is an error (Z0320).

**Files.** Help is always one file per function; code is one file per function only when it is written in Zeff.

| Written in | Code | Help |
| --- | --- | --- |
| Rust (built-in) | One module per family of related functions in the package crate's `src/` (`regress/src/ols.rs` for `reg`, `regress/src/iv.rs` for `ivreg` and `first_stage`, `glm/src/binary.rs` for `logistic` and `probit`), so functions that share private helpers, such as one fitting loop or one variance routine, share a module instead of copying code; `src/lib.rs` registers the package's functions | One Markdown file per function in the crate's `help/`, named as the function's entry in zeff_functions.csv without its receiver prefix where the package has one home for the name (`help/reg.md`, `help/first_stage.md`) and with it where the name needs one (`identify/help/g.plot.md`, `dist/help/dist.normal.md`); a name with two homes (section 8.1, rule 6) has one page in each package that defines it |
| Zeff (built-in, Zeff-source package, or a project's `functions/`) | One `.zeff` file per function in the package's or project's `functions/`, named after it (`functions/share_missing.zeff`); a function and private helpers that only it uses may share its file, and a small family always read together (a shorthand and the function it wraps) may share a file named after the family's main function | In the same file, as `///` lines above each `fn` |

```
packages/tier2/regress/          a first-party package crate
    Cargo.toml
    src/lib.rs                   registers reg, ivreg, first_stage, r_squared, …
    src/ols.rs                   reg and its tall-skinny QR fit
    src/iv.rs                    ivreg, first_stage
    src/diagnostics.rs           r_squared, influence, vif
    src/terms.rs                 interact, event
    help/reg.md  help/ivreg.md  help/first_stage.md  help/r_squared.md
    help/influence.md  help/vif.md  help/interact.md  help/event.md
    tests/                       reference, edge-case and property tests
    functions/                   functions of this package written in Zeff, one per file
```

The registry build (section 8.1, rule 1) matches help to functions by name, whatever module defines them: it fails on a registered function with no help file and on a help file that names no registered function, so moving a function between modules never touches its help, and the page count always equals the function count.

**Datasets.** A dataset of the `datasets` package (section 8.5) has a page of the same kind: Summary, Description, Format (every variable's name, type, label, units and missing codes, generated from the schema), Source or Generation (the cleaning or generating script and its seed), Licence and citation, References, Examples; `zeff help wages` prints it, and a dataset without a complete page does not ship.

```
/// Share of rows where a variable is missing.
///
/// Counts the rows where `var` is `NA` and divides by the row count. An empty frame gives `NA`
/// with a warning, never a division by zero.
///
/// data: The frame to check.
/// var: The variable whose missing values are counted.
/// returns: The share, a Float between 0 and 1.
/// see also: n_missing, is_missing, drop_missing
///
/// > import("zeff:wages.csv").share_missing(educ)
/// 0.0125
fn share_missing(data: Frame, var: Var) -> Float {
    data.where(is_missing(var)).n_rows() / data.n_rows()
}
```

### 9.2 The functions by package

#### `core` (tier 0) · 68 in 1.0, 0 proposed, 3 on the roadmap

- **Maths:** `abs`, `sign`, `sqrt`, `exp`, `log`, `log2`, `log10`, `log1p`, `expm1`, `round`, `floor`, `ceil`, `truncate`, `min`, `max`, `clamp`, `div`, `mod`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `logit`, `inv_logit`, `factorial`, `choose`, `gamma`, `log_gamma`, `beta`, `is_finite`, `PI`, `E`
- **Types and categories:** `to`, `is`, `type_of`, `vars`, `category`, `levels`, `base`
- **Collections:** `len`, `first`, `last`, `unique`, `sorted`, `reversed`, `any`, `all`, `append`, `keys`, `is_in`, `seq`
- **Missing values:** `is_missing`, `fill_missing`, `coalesce`, `n_missing`
- **Program:** `print`, `warn`, `error`, `assert`, `attempt`, `iterate`, `seed`, `timer`, `version`, `help`
- **Roadmap (1.8; names reserved in `core` before the 1.0 freeze):** `matrix`, `vector`, `transpose`

#### `time` (tier 1) · 14 in 1.0, 0 proposed, 2 on the roadmap

- **Dates:** `date`, `datetime`, `today`, `now`, `year`, `month`, `day`, `weekday`, `quarter`, `week`, `add_days`, `add_months`, `days_between`, `months_between`
- **Roadmap:** `duration`, `to_timezone`

#### `text` (tier 1) · 13 in 1.0, 0 proposed, 0 on the roadmap

- **Text:** `upper`, `lower`, `trim`, `contains`, `starts_with`, `ends_with`, `replace`, `extract`, `split`, `join`, `slice`, `pad`, `format`

#### `data` (tier 1) · 59 in 1.0, 0 proposed, 6 on the roadmap

- **Frames:** `frame`, `import`, `n_rows`, `n_cols`, `variables`, `describe`, `head`, `tail`, `where`, `select`, `drop`, `rename`, `add`, `update`, `sort`, `distinct`, `duplicates`, `sample`, `merge`, `append`, `to_long`, `to_wide`, `aggregate`, `panel`, `drop_missing`, `label`, `recode`, `save`
- **Column functions:** `lag`, `lead`, `diff`, `running_sum`, `running_product`, `rank`, `quantile_group`, `bin`, `standardize`, `center`, `between`, `row_mean`, `row_sum`, `winsorize`, `fill_forward`
- **Reducers and summaries:** `mean`, `sd`, `variance`, `median`, `iqr`, `mad`, `quantile`, `min`, `max`, `sum`, `n`, `n_distinct`, `skewness`, `kurtosis`, `summary`, `tabulate`
- **Roadmap:** `stream`, `external` (desktop), `unnest`, `moving_average`, `growth_rate`, `index_number`

#### `dist` (tier 1) · 19 in 1.0, 0 proposed, 0 on the roadmap

- **Distributions:** `dist.normal`, `dist.lognormal`, `dist.t`, `dist.chi2`, `dist.f`, `dist.binomial`, `dist.poisson`, `dist.negbinomial`, `dist.uniform`, `dist.exponential`, `dist.gamma`, `dist.beta`, `dist.tukey`, `dist.truncated`, `dist.mixture`
- **Methods on a distribution:** `pdf`, `cdf`, `quantile`, `draw`

#### `results` (tier 1) · 17 in 1.0, 0 proposed, 2 on the roadmap

- **Result interface:** `summary`, `estimates`, `std_errors`, `p_values`, `ci`, `n_obs`, `export`
- **Estimator interface:** `coef`, `vcov`, `fit_measures`, `predict`, `residuals`, `fitted`, `compare`, `test`, `derive`
- **Tables and export:** `report`
- **Roadmap:** `margins`, `contrasts`

#### `stats` (tier 1) · 13 in 1.0, 0 proposed, 10 on the roadmap

- **Association:** `correlation`, `covariance`
- **Tests:** `t_test`, `wilcoxon_test`, `anova` (one-way and factorial, with `type:`), `posthoc`, `kruskal_test`, `chi2_test`, `fisher_test`, `proportion_test`, `shapiro_test`, `ks_test`, `p_adjust`
- **Roadmap:** `power.t_test`, `power.proportion_test`, `power.anova`, `mde`, `repeated_anova`, `tost` (1.9); `ci_median`, `effect_size`, `loess`, `spline`

#### `optim` (tier 1) · 0 in 1.0, 0 proposed, 11 on the roadmap

- **Roadmap:** `minimize`, `root`, `gradient`, `hessian` (1.8, with automatic differentiation), `mle`, `gmm`, `lp`, `mip`, `nlp`, `stochastic_program`, `robust_program`

#### `resample` (tier 1) · 0 in 1.0, 0 proposed, 7 on the roadmap

- **Roadmap:** `bootstrap`, `jackknife`, `permutation_test`; `holdout`, `training`, `testing`, `folds` (1.5, section 7.5 of the Language Reference)

#### `survey` (tier 1) · 0 in 1.0, 0 proposed, 8 on the roadmap

- **Roadmap (1.9):** `survey`, `total`, `ratio`, `replicate_weights`, `design_effect`, `poststratify`, `rake`, `calibrate`

#### `plot` (tier 1) · 35 in 1.0, 0 proposed, 6 on the roadmap

- **Grammar (section 7.6 of the Language Reference):** `plot`; marks `points`, `lines`, `step`, `bars`, `area`, `ribbon`, `text`, `tile`, `rug`, `hline`, `vline`; statistical layers `histogram`, `density`, `boxplot`, `errorbars`; `scale_x`, `scale_y`, `scale`, `coord`, `facet`, `labels`, `legend`, `theme`, `annotate`
- **Shorthands:** `histogram`, `scatter`, `line`, `bar`, `boxplot`, `density`, `qq_plot`, `heatmap`
- **Result plots:** `plot`, `coef_plot`
- **Roadmap:** `smooth`, `violin`, `aggregate`, `arrange`, `computed` (1.2); `scatter_matrix`

#### `datasets` (tier 1) · 2 in 1.0, 0 proposed, 0 on the roadmap

- **Datasets:** `dataset`, `datasets` (section 8.5)

#### `web` (tier 1) · 0 in 1.0, 0 proposed, 4 on the roadmap

- **Roadmap (1.1, section 8.6):** `fetch`, `secret`, `flatten`, `rate_limit`

#### `regress` (tier 2) · 8 in 1.0, 0 proposed, 7 on the roadmap

- **Models:** `reg`, `ivreg`
- **Linear-model methods:** `r_squared`, `influence`, `vif`, `first_stage` (the generic result methods are in `results`)
- **Formula terms:** `interact`, `event`
- **Roadmap:** `power.reg` (1.9), `wild_bootstrap`, `ivreg` method `liml`, `ar_test`, `quantile_reg`, `het_test`, `reset_test`

#### `glm` (tier 2) · 2 in 1.0, 0 proposed, 12 on the roadmap

- **Binary models:** `logistic`, `probit`
- **Roadmap:** `poisson` (1.2), `negbin`, `gamma_reg`, `beta_reg`, `fractional`, `ordinal`, `multinomial`, `tobit`, `heckman`, `zero_inflated`, `hurdle`, `surv` (the censored-outcome term of `surv(time, event) ~ x`)

#### `multilevel` (tier 2) · 0 in 1.0, 0 proposed, 6 on the roadmap

- **Roadmap:** `mixed`, `panel_reg`, `hausman` (1.4), `glmm`, `growth`, `dynamic_panel`

#### `explore` (tier 2) · 0 in 1.0, 0 proposed, 9 on the roadmap

- **Roadmap:** `impute`, `pca`, `correspondence`, `mds`, `tsne`, `umap`, `missing_pattern`, `mcar_test`, `outliers`

#### `series` (tier 2) · 0 in 1.0, 0 proposed, 7 on the roadmap

- **Roadmap:** `arima`, `ets`, `vector_ar`, `state_space`, `dynamic_factor`, `forecast`, `trend_forecast`

#### `latent` (tier 2) · 0 in 1.0, 0 proposed, 18 on the roadmap

- **Roadmap:** `model`, `cfa`, `sem`, `effects`, `loadings`, `mod_indices` (1.3), `lca`, `pick`, `profiles`, `efa`, `kmo`, `sphericity_test`, `parallel_analysis`, `factor_scores`, `latent_growth`, `lpa`, `lta`, `growth_mixture`; also WLSMV and `efa()` blocks (ESEM) in `cfa` and `sem`

#### `measure` (tier 2) · 1 in 1.0, 0 proposed, 13 on the roadmap

- **Reliability:** `reliability`
- **Roadmap:** `item_stats`, `polychoric`, `composite_reliability`, `ave`, `fornell_larcker`, `htmt`, `invariance`, `alignment`, `irt`, `dif`, `icc`, `kappa`, `g_study`

#### `learn` (tier 2) · 0 in 1.0, 0 proposed, 32 on the roadmap

- **Roadmap, 1.5 (section 7.5 of the Language Reference):** `recipe` and its steps `impute`, `standardize`, `center`, `one_hot`, `bin`, `pca`, `winsorize`, `add`; `prepare`, `apply`; `spec.lasso`, `spec.random_forest`, `spec.boosting`, `spec.logistic`, `spec.reg`; `learn`, `tune`, `score`, `grid`, `best`; `lasso`
- **Roadmap, later:** `tree`, `random_forest`, `boosting`, `svm`, `knn`, `naive_bayes`, `neural_net`, `calibration`, `shap`, `partial_dependence`

#### `groups` (tier 2) · 0 in 1.0, 0 proposed, 7 on the roadmap

- **Roadmap:** `kmeans`, `kmedoids`, `hierarchical`, `dbscan`, `gaussian_mixture`, `topic_model`, `communities`

#### `identify` (tier 3) · 6 in 1.0, 0 proposed, 0 on the roadmap

- **Causal graphs:** `dag`, `adjustment_sets`, `implied_independencies`, `check`, `exclusion_check` (the exclusion-restriction check on an IV fit), `plot` (`g.plot()`, drawing the graph)

#### `experiment` (tier 3) · 0 in 1.0, 0 proposed, 9 on the roadmap

- **Roadmap:** `factorial_design`, `fractional_design`, `response_surface`, `randomize`, `balance` (1.9), `cuped`, `ri_test`, `lee_bounds`, `late`

#### `quasi` (tier 3) · 0 in 1.0, 0 proposed, 10 on the roadmap

- **Roadmap:** `did`, `combine`, `pre_trends` (1.2), `did` methods `sa`, `dcdh` and `imputation`, `honest_did`, `rdd`, `rd_bandwidth`, `density_test`, `synth`, `shift_share`

#### `adjust` (tier 3) · 0 in 1.0, 0 proposed, 12 on the roadmap

- **Roadmap:** `double_lasso`, `selected`, `matching`, `ipw`, `aipw`, `entropy_balance`, `overlap`, `dml`, `oster`, `e_value`, `sensitivity`, `rosenbaum`

#### `mediate` (tier 3) · 0 in 1.0, 0 proposed, 5 on the roadmap

- **Roadmap:** `mediate`, `mediation_sensitivity`, `moderated_mediation`, `front_door`, `interventional`

#### `hetero` (tier 3) · 0 in 1.0, 0 proposed, 10 on the roadmap

- **Roadmap:** `causal_forest`, `ate`, `importance`, `simple_slopes`, `johnson_neyman`, `subgroups`, `meta_learner`, `blp`, `gates`, `qte`

#### `structural` (tier 3) · 0 in 1.0, 0 proposed, 11 on the roadmap

- **Roadmap:** `smm`, `indirect_inference`, `conditional_logit`, `nested_logit`, `mixed_logit`, `blp_demand`, `nfxp`, `ccp`, `entry_model`, `simulate`, `welfare`

#### `decide` (tier 3) · 0 in 1.0, 0 proposed, 8 on the roadmap

- **Roadmap:** `policy_tree`, `ewm`, `q_learning`, `a_learning`, `bandit`, `newsvendor`, `pricing`, `robust_decision`

#### `meta` (tier 3) · 0 in 1.0, 0 proposed, 13 on the roadmap

- **Roadmap:** `effect_convert`, `meta`, `meta_reg`, `meta_3level`, `network_meta`, `forest_plot`, `funnel_plot`, `egger_test`, `trim_fill`, `pet_peese`, `p_curve`, `replication_test`, `spec_curve`

---

---

## 12. Editing Zeff: IDE support

Zeff ships a language server (`zeff lsp`, the Language Server Protocol), a formatter and a syntax grammar, so the same editing help works in Zemp's built-in editor and in any editor that speaks LSP, such as VS Code (free extension). Zemp's editor is described in the Zemp Specification; this section fixes what the language provides to every editor. 1.0 ships completion, signature help, hover, live diagnostics, formatting, highlighting, running and folding; rows marked 1.1 follow.

| Feature | Behaviour |
| --- | --- |
| Completion | After `wages.`: every method valid for a Frame; inside a call: argument names, then for a fixed-list argument its allowed values, and for a `Var` or `Terms` argument the frame's variables with their types, taken from the cached schemas |
| Signature help | The function's usage line and the current argument's help while typing |
| Hover | A function's help page (section 9.1), a binding's type and size, a variable's type and missing count |
| Live diagnostics | The checks of section 11 of the Language Reference run as you type, with the same codes and suggestions, before anything runs; steps checked only at run time are marked |
| Quick fixes (1.1) | One-click fixes for suggestions: misspelt names, `if` → `case`, a missing `relation:`, an instrumented variable missing from the formula, pasted lme4 terms to `re:` |
| Formatting | `zeff fmt` on save; format selection |
| Navigation (1.1) | Go to definition and find references for bindings and project functions; document outline of bindings, functions and model blocks |
| Rename (1.1) | Rename a binding or project function across the project |
| Syntax highlighting | A tree-sitter grammar: keywords, formulas, model-block operators and bare option words each styled distinctly |
| Run | Run the current statement or chain, the selection, or the whole script; results appear in the console and object panel |
| Folding | Chains, model blocks, `case` blocks and functions fold |
| Snippets | Templates for common analyses (a regression table, an IV model with diagnostics, a two-period DiD, a logistic regression) |
| Inlay hints (1.1) | Optional: the type of each `let` binding and the row count of each frame after a chain step |

**Running code interactively.** A project keeps one live session per window. Running a statement updates the session's bindings, which the object panel shows with their memory use. Because bindings never change, re-running a chain simply shadows the old result; `zeff run` from a clean session always reproduces the script's output exactly, and Zemp marks the session "out of sync" when statements were run out of order.

**Testing.** `zeff test` runs every `assert` and each project function's examples written in its `///` help, so a project can check its own numbers on every run.

---

---

## 13. Accuracy and validation

**Numerical methods.** Sums use compensated summation and variances the two-pass algorithm; OLS is solved by QR decomposition, never by inverting X′X; distribution tails are computed directly rather than as one minus the CDF. The random-number generator gives identical draws on every platform and in the browser; from 1.1, parallel work gets its own reproducible stream per task, so results do not depend on the number of threads.

**Validation.** Every function is compared automatically, on every build, with reference software on shared test data before its release closes. One tolerance cannot fit every method, so each function belongs to an accuracy class, recorded in the `accuracy` column of zeff_functions.csv. The one exception is `external` (1.1), whose results are outside Zeff's guarantees.

| Class | Methods | Tolerance against the reference |
| --- | --- | --- |
| Exact | Data verbs, text, dates, types, collections, tables and plots | Identical output: the same rows, values, labels and layout |
| Closed form | Descriptive statistics, OLS, fixed effects, IV, classical tests | Estimates and standard errors within 1e-6 relative |
| Distribution functions | `pdf`, `cdf`, `quantile` and tail probabilities | Within 1e-12 relative |
| Iterative | Logistic and probit regression in 1.0; Poisson, Callaway–Sant'Anna DiD, CFA, SEM and mixed models after | Started from the reference's start values: estimates and standard errors within 1e-4 relative, log-likelihood or objective within 1e-6 relative |
| Stochastic | `sample` and `draw` in 1.0; bootstrap standard errors, forests, double lasso with cross-validation and latent classes after | Zeff reproduces itself exactly with the same seed, whatever the thread count; against the reference, agreement within Monte Carlo error over 50 seeds; for latent classes, the best log-likelihood across starts within 1e-4 after aligning class labels |

| Also required | Standard |
| --- | --- |
| Random draws | Identical across Windows, macOS, Linux and the browser |
| Resampling with failed draws | The same draws fail on every platform, so skipped draws do not change between machines |
| Cross-platform identity | Every build runs the suite on WebAssembly, x86-64 and ARM64 and requires bit-identical results (section 8.3) |
| Cached and fresh runs | A run from the cache and `zeff run --fresh` give identical output |
| Supported platforms | Bit-identical results are promised on WebAssembly with SIMD128, x86-64 from the baseline instruction set, SSE2 (results do not depend on SSE4, AVX2 or AVX-512 being present) and ARM64; other platforms run the same code but are best effort |

**Conformance protocol.** The validation suite is public and runs on every build. The Zeff Testing Handbook (zeff_testing.md) sets out the full testing routine: kinds of test, the routine on each change and release, how bugs are handled, and the definition of done for each function.

| Element | Rule |
| --- | --- |
| Reference versions | Each Zeff release names the exact versions of R, every R package and Stata it was checked against; a later reference version is adopted only in a new Zeff release |
| What is compared | Estimates, standard errors, test statistics, p-values, confidence limits, log-likelihoods and observation counts; predictions where a method has them |
| When references disagree | Zeff follows the definition its documentation states and records which reference matches (for example CR1 clustered errors with reghdfe's degrees of freedom); the disagreement is listed in the function's help |
| Edge-case datasets | Every method is also run on fixed datasets with missing values, zero variance, perfect separation, singleton clusters and fixed-effect groups, exact collinearity, extreme weights and empty groups; the expected result for each (a value, a warning or an error) is part of the suite |
| Reporting | Each release publishes the suite's results by function and platform |

| Package | Checked against |
| --- | --- |
| core, text, time | R base, stringr, lubridate; Stata functions |
| data | dplyr, tidyr; Stata |
| results | R `coef`, `vcov`, `confint` and `predict` on lm and glm fits; `derive` against Stata nlcom and car::deltaMethod; modelsummary and esttab output for tables |
| dist | R stats distribution functions and Boost.Math tables; truncdist for `dist.truncated` |
| stats | R stats tests and `cor`; Stata ttest, prtest, tabulate |
| plot | Scene snapshots: the same scene for the same data; statistical layers against the functions that compute them; visual checks at pane and journal sizes; text never below 8 pt in exports |
| datasets | Checksums and licence records for every file; synthetic datasets regenerated from their scripts bit for bit |
| regress | R lm and sandwich; fixest, reghdfe, ivreg2; Stata regress, ivregress |
| identify | dagitty: adjustment sets and implied independencies on its test graphs |
| glm | R glm; Stata logit, probit |
| measure | psych; Stata alpha |
| After 1.0 | quasi: did, csdid · Poisson: fixest fepois, ppmlhdfe · latent: lavaan, Stata sem · multilevel: lme4, lmerTest, Stata mixed · adjust, hetero: hdm, Stata dsregress, grf · latent classes: poLCA, Stata gsem |

**1.0 is done when** all six hold:

- Accuracy: every function matches its reference within its class's tolerance on a public benchmark suite
- Brevity: each target analysis is no longer in Zeff than in R or Stata
- Parity: every procedure gives identical results from Zemp's dialogs and from typed Zeff
- Reproducibility: re-running a project on the same data gives identical output
- Speed: a regression with two-way fixed effects on 10 million rows runs within 2× of fixest's time on one thread
- Usability: a researcher new to Zeff runs a fixed-effects panel regression or an IV model within 30 minutes

---

---

## 14. Build order and release plan

**Version numbers.** Releases are numbered major.minor.patch under semantic versioning, as Rust and Cargo number theirs. Each part promises something different:

| Part | Example | What it may change | Scripts written for an earlier version |
| --- | --- | --- | --- |
| Major (x.0.0) | 1.0.0, 2.0.0 | Anything: names, defaults, grammar, printed results; a major release ships with a migration guide, and `zeff fix --edition` rewrites what its new edition can rewrite | May break; 1.0.0 is the freeze, and only 2.0.0 may break a 1.x script |
| Minor (0.x.0, 1.x) | 0.7.0, 1.2 | Before 1.0: anything, since 0.x releases are where the language is shaped, each break listed in the release notes with the rewrite it needs. From 1.0: additions only, new functions, packages, options with defaults that keep earlier behaviour, formats and diagnostics. An addition never changes what an existing script calls or whether it runs: a bare name that a newer engine's built-in would capture is an error (Z0109), and `zeff engine use` rewrites such calls to their full form, and lists the new warning codes the newer engine raises on the project (so `--deny-warnings` never fails on an update unseen), before the pin moves (section 3.6 of the Language Reference) | Before 1.0, may break; from 1.0, run unchanged with the same results |
| Patch (third number) | 0.9.1, 1.2.1 | Fixes only: no new function, name or default; a result changes only where it was wrong, and the release notes and `zeff audit` name every result a patch changes. Patch releases begin at 0.9.0, when the launcher and the engine store arrive; before 0.9.0 a wrong-result fix ships in the next minor release | Run unchanged; a corrected result is the only difference |

Before 1.0 every release is a minor release and is written in full (0.7.0). After 1.0 a minor release is written as 1.2 and a patch as 1.2.1 in prose; pins, tags and `zeff engine` always carry three numbers (`zeff = "1.2.0"`). A version number never goes back to 0.0.x: the first release is 0.1.0. No version has been released: an earlier prototype engine (built as 0.0.1) was set aside on 17 October 2026, and the code starts afresh at 0.1.0, built on the architecture of section 8 from its first commit.

The 257 functions of 1.0 arrive over eleven minor releases, 0.1.0 to 0.11.0, then 1.0.0 freezes the names. Each release is named for what it is for. A function waits for everything it uses; functions written in Zeff wait for `fn` in 0.5.0 (frame `summary` runs in Rust until then). By 0.7.0, 243 of the 257 functions have shipped and every release up to it lets someone run one more real analysis; the four releases after it are engine milestones, each with a hard engine piece of its own.

**Why this scope.** The schedule is set by hard engine pieces, not by the number of functions. 1.0 has seven: the formula engine and model matrix with QR and robust and clustered SEs; fixed-effects demeaning; two-stage least squares with weak-instrument statistics; logistic and probit fitting; accurate special functions and distributions; the plot renderer; and the language server with the schema pass. Callaway–Sant'Anna DiD, PPML, the thread pool and bootstrap, SEM, mixed models, lasso and forests, WLSMV and ESEM, latent classes, streaming and lazy chains are each a project of their own, and come after 1.0, one per release. Causal graphs join 1.0 because their algorithms (cycle detection, d-separation, back-door adjustment sets) are small, exact and well documented, and checking a fitted model against a causal graph is something mainstream statistics packages do not do.

| Release | Name | Functions | Engine and language work | Unlocks |
| --- | --- | --- | --- | --- |
| 0.1.0 | First engine | 10: `frame`, `import`, `n_rows`, `n_cols`, `variables`, `describe`, `mean`, `sd`, `summary`, `print` | A Cargo workspace laid out by package and tier, with `core` and the kernel crates separate and the CI rule of section 8.1, rule 1; lexer and parser for the grammar of section 3.4 of the Language Reference; the typed IR between parser and interpreter, with diagnostics as data; Arrow columns; projects with `zeff.toml`; CSV import; the function registry compiled from each package's help files, with the build check of section 9.1, so the ten functions ship with complete help pages; the command line's `zeff new`, `zeff run`, `zeff check` (names, arguments and types; the schema pass follows in 0.6.0), `zeff version` and `zeff help`; continuous integration on the four targets with the bit-identity check of the testing handbook from the first commit, and the compressed bundle size and cold-start time tracked on every build (zeff_risk_register.md, R2 and R4); reference tests against Python's exact `statistics` module, with the R scripts of the testing handbook from 0.2.0 | Descriptive statistics on a CSV |
| 0.2.0 | Data cleaning | 50: frame verbs `head` to `sort`; 27 maths functions and constants; `is_missing`, `coalesce`; reducers; `warn`, `error`; `dataset`, `datasets` and the `zeff:` data root | Expressions over columns; NA, NaN and Inf rules; list, record and mapping literals; the column-type interface with Arrow extension metadata, the opaque rule and the reserved type names (section 4.3 of the Language Reference); reducers with `by:` and `weights:`; `zeff repl` and `zeff eval` with the scratch project; `--project`, `--color`, `--ascii`, `--width`, `--quiet` and `--format json` (section 2.1) | Everyday data cleaning |
| 0.3.0 | Distributions | 26: special functions; `correlation`, `covariance`; 12 distributions with `pdf`, `cdf`, `quantile`, `draw`, plus `dist.truncated` and `dist.mixture`; `seed` | Incomplete beta and gamma, inverse CDFs, accurate tails; Philox counter-based random streams; the determinism contract in full, proved by the four-target CI that has run since 0.1.0; `zeff selftest` | Simulation and p-values |
| 0.4.0 | Regression | 29: `reg` without fixed effects; the result and estimator interfaces of `results` (17 functions, `derive` and `report` among them); `r_squared` and `influence`; `interact`; types and categories with contrasts; `label`, `recode` | The `zeff-kernels` crate (tall-skinny QR, Cholesky, fixed SIMD width); formula parser and the model-matrix rules of section 7.2 of the Language Reference, with collinearity reporting; the column metadata store and its keys (section 4.3 of the Language Reference); the estimator trait and the variance layer (classical, HC, clustered); Word and LaTeX tables | Cross-sectional regression |
| 0.5.0 | Language complete | 32: 13 collection functions; `attempt`, `iterate`; 17 functions written in Zeff (and `summary` rewritten in Zeff) | `if`, `case`, `for`, `fn`, `use`, calling any function as a method; the `Failed` type | User-written functions |
| 0.6.0 | Panels | 36: joins, reshaping, `aggregate`, `panel`, `save`; panel column functions; `event`; dates and `format` | Grouped computation; `fe:` demeaning in `reg`; date types; shared Arrow columns and selection vectors; the schema pass behind `zeff check`; data roots in `zeff.toml` | Panel research |
| 0.7.0 | Tests, plots and the language server | 60: `tabulate`; nine tests, `anova` with factorial tables, `posthoc` and `dist.tukey`; `p_adjust`; the plot grammar and its shorthands (35 functions, section 7.6 of the Language Reference); 12 text functions | Language server (completion, signature help, hover, live diagnostics), tree-sitter grammar, `zeff fmt` and `zeff test`, ready for Zemp's prototype (Zemp 0.1; its alpha follows at 0.8.0); layered-grammar plot renderer with the Okabe–Ito and viridis scales, redraw to fit and journal sizes; exact Wilcoxon distribution; engine worker with progress, cancellation and memory budget; `zeff completions` | Exploratory analysis; Zemp's interface starts |
| 0.8.0 | Instrumental variables | 2: `ivreg`, `first_stage` | Two-stage least squares with `instruments:`; Kleibergen–Paap, Cragg–Donald and effective F; `zeff report-bug` with shrinking; `zeff explain` | IV studies |
| 0.9.0 | Binary outcomes | 3: `logistic`, `probit`, `timer` | The likelihood trait and shared optimiser; IRLS and Newton fitting; separation checks; robust and clustered SEs from the variance layer; patch releases begin, with the launcher and engine split, `zeff engine`, `zeff doctor` and signed engine downloads (section 2.1); the known-issues list and `zeff audit`, which patch releases need from their first day | Binary outcomes |
| 0.10.0 | Causal graphs and validation | 7: `reliability`; `dag`, `adjustment_sets`, `implied_independencies`, `check`, `exclusion_check`, `g.plot` | `dag { }` parser; cycle detection, d-separation and back-door adjustment sets; the validation suite run in full for every function and published | Scale reliability; causal graphs; public validation results |
| 0.11.0 | Help and formats | 2: `version`, `help` | Help built from `///`; the reader interface (section 6.1 of the Language Reference) and, through it, Excel, OpenDocument, Parquet, Arrow, Stata, JSON, fixed-width and compressed files in `import` and `save`, with opaque columns carried (section 4.3 of the Language Reference); versioned result and run-record schemas; the content-addressed result cache and the import cache, both Arrow IPC, deterministic Parquet writing and the Zeff Parquet profile (section 8.7); `zeff runs`, `zeff export replication`, `zeff verify` and `zeff dataset fetch`; `zeff app` for the desktop apps and the offline bundle (section 2.1), which Zemp's desktop beta installs through | Replication packages; a stable engine for Zemp's desktop beta |
| 1.0.0 | Names frozen | None | Names frozen; edition 2027 declared; extension interfaces and the package manifest published; public benchmark suite; documentation; the installer script, signed archives, Homebrew, winget, Scoop, the container image, the GitHub Action, `zeff self update` and reproducible official builds (section 2.1) | Public release of the language, installable from a package manager |

The ten functions once proposed for 1.0 were decided on 16 October 2026 (section 15): `iterate`, `event`, `posthoc` with factorial `anova` and `iv.check` (renamed `exclusion_check` on 17 October 2026) are in, and the five power functions and `survey()` form the Design release, 1.9.

**Zeff and Zemp.** Zeff 1.0.0 is the engine, the command line, the language server and the public validation suite. It does not wait for the desktop app, installers or code signing, which belong to Zemp's own release line (Zemp Specification, section 15); Zemp Lite 1.0, which needs no dialogs, ships with Zeff 1.0.0, and Zemp 1.0 follows once its dialogs cover every 1.0 method. Each Zemp release records the engine versions it supports, and a Zemp fix never requires an engine release.

**After 1.0.** Each minor release is built around one family of methods and the engine work it needs. The first is engine work alone, so that every method release after it is smaller and the 1.0 methods get faster at once.

| Release | Name | Methods | Engine work |
| --- | --- | --- | --- |
| 1.1 | Engine | `external` (desktop); `se: bootstrap` with `on_fail:` and `max_fail:` on every model; SPSS, SAS, R and HTML readers in `import` and SPSS in `save` (section 6.1 of the Language Reference); `fetch`, `secret`, `flatten`, `rate_limit` (section 8.6); `unnest` (section 4.3 of the Language Reference) | Thread pool with reproducible streams and deterministic parallelism (`--threads`); bootstrap in the variance layer; lazy chains with projection and filter pushdown; the remaining language-server features (quick fixes, navigation, rename, inlay hints) |
| 1.2 | Difference-in-differences | `did`, `combine`, `pre_trends`; `poisson` with `fe:`; the plot layers `smooth`, `violin` and `aggregate`, `arrange` and `computed` (section 7.6 of the Language Reference) | Callaway–Sant'Anna with doubly robust estimation and multiplier bootstrap, on the thread pool and bootstrap of 1.1; PPML |
| 1.3 | Latent variables | `model`, `cfa`, `sem`, `effects`, `loadings`, `mod_indices`; omega in `reliability` | `model { }` parser; ML and MLR with FIML |
| 1.4 | Multilevel | `mixed` with `re:`, `panel_reg` (with `within_between:` for the Mundlak form), `hausman` | REML and ML for linear mixed models; Satterthwaite and Kenward–Roger degrees of freedom |
| 1.5 | Machine learning: causal methods and leakage-safe prediction | `double_lasso`, `selected`, `causal_forest`, `ate`, `importance`; `holdout`, `training`, `testing`, `folds`; `recipe` with `impute`, `standardize`, `center`, `one_hot`, `bin`, `pca`, `winsorize`, `add`, `prepare`, `apply`; `spec.lasso`, `spec.random_forest`, `spec.boosting`, `spec.logistic`, `spec.reg`; `learn`, `tune`, `score`, `grid`, `best`, `lasso` | Lasso with plug-in and cross-validated penalties; honest forests on the thread pool; recipes re-estimated inside every fold; the leakage checker on column lineage (section 7.5 of the Language Reference) |
| 1.6 | Ordinal measurement | WLSMV for ordered items; `efa()` blocks (ESEM) in `cfa` and `sem`; `polychoric` | Polychoric correlations, robust weighted least squares, geomin and target rotation |
| 1.7 | Latent classes and large data | `lca`, `pick`, `profiles`; `stream` | EM with many random starts and class-label alignment; streaming estimation by QR |
| 1.8 | User estimators | `mle`, `gmm`, `minimize`, `root`, `gradient`, `hessian` | Forward-mode automatic differentiation on the typed IR, compiled into kernels (section 8.3) |
| 1.9 | Design | `power.t_test`, `power.proportion_test`, `power.anova`, `power.reg`, `mde`; `factorial_design`, `fractional_design`, `response_surface`; `repeated_anova`, `tost`; `randomize`, `balance`; `survey` and the survey family (`total`, `ratio`, `replicate_weights`, `design_effect`, `poststratify`, `rake`, `calibrate`) | Non-central t and F; design generators; sphericity corrections; design-based variance exposed through `survey` |
| 1.10 | Bayesian estimation | Bayesian fitting of the built-in models and of `mle` objectives, with priors and posterior summaries | Hamiltonian Monte Carlo (NUTS) on the automatic differentiation of 1.8 |
| 1.11 | Thirteen goals | Every research goal of the Zemp Specification covered in its core methods; third-party packages (section 8.4); user-defined types (section 15) | Package manifest and `zeff.lock`; component sandbox; a second edition only if the syntax needs one |

The remaining roadmap methods follow, package by package, in the order of the Zemp roadmap; the order within a tier (section 8.1, rule 3) says which packages must exist first.

**Cadence and support.**

| Rule | Policy |
| --- | --- |
| Cadence before 1.0 | One minor release every six to eight weeks, which puts 1.0.0 about fifteen to twenty months after 0.1.0 |
| Cadence after 1.0 | A minor release about every three months; a patch within two weeks of any confirmed wrong-result bug (section 11.1 of the Language Reference) |
| Patches | From 0.9.0; change no name, default or result except the one fixed; a project moves to a patch only through "Update engine" |
| Breaking changes | Allowed in a minor release before 1.0 and only in a major release after it (section 14, Version numbers); a feature planned after 1.0 that would break a 1.x script waits for a major release or ships behind a new edition |
| Support | Patches are issued for the current and the previous minor release; every released engine stays downloadable indefinitely, because a replication package needs the engine it pins |
| Editions | Edition 2027 ships with 1.0.0; no new edition before 1.11 |
| Proposed functions | None remain: the ten of 1.0 were decided on 16 October 2026 (section 15) |
| Specification | This document carries a date; from 1.0.0 it also carries the engine version it describes and changes only with a minor release; the normative document for an engine version is the one dated at or before its release. It is published in two volumes, a Language Reference (sections 1.5, 3 to 7, 10, 11 and the glossary) and an Engine and Project Specification (the rest), with the section numbers shared; there is no generated source, so each volume is edited directly and the text both carry is kept identical by hand |

**Risks to this plan.** The risks to the schedule and the design (scope, bit-identity across targets, matching references on edge cases, the browser build, language tooling, dependency churn, adoption and momentum) are kept in the risk register (zeff_risk_register.md), with an early warning, a mitigation and a fallback for each; it is reviewed at every minor release, first at 0.1.0.

### 14.1 Future plan

The releases above are dated by dependency and end at 1.11. This section sets out what comes after them, so that nothing built before 1.11 closes a door the later work needs. Nothing here is normative (section 1, status); each item becomes a release row when its prerequisites exist, and its design is written into this document before work starts.

**Methods, by research goal.** Planned after the dated releases, in roughly this order, each with the engine work it needs:

| Goal | Methods | Engine work or prerequisite |
| --- | --- | --- |
| Causal | Over-identification and endogeneity tests beyond Hansen's J; control-function, copula and Lewbel instruments; Conley spatial standard errors; financial event studies (market model, cumulative abnormal returns) | The variance layer's spatial kernel |
| Associational | Seemingly unrelated regression and 3SLS; multivariate outcomes and MANOVA (section 15); robust regression (M- and MM-estimators); Oaxaca–Blinder decomposition | Multivariate outcomes in the formula grammar |
| Design | Blocking and questionnaire pre-tests (factorial and fractional designs, response surfaces, repeated-measures ANOVA and equivalence tests are 1.9 rows) | The 1.9 Design release |
| Descriptive and series | Survival analysis (Kaplan–Meier, Cox, parametric and competing risks); unit-root, cointegration and structural-break tests | Censored-outcome likelihoods |
| Operations and productivity | Stochastic frontier analysis; data envelopment analysis; production-function estimation (Olley–Pakes, Levinsohn–Petrin, Ackerberg–Caves–Frazer) | `optim` with automatic differentiation (1.8); linear programming |
| Measurement | Common-method-variance checks (marker variable, unmeasured latent method factor); PLS-SEM; fuzzy-set QCA | The model-block parser (1.3) |
| Bayesian | Priors in `model { }` blocks; hierarchical models; posterior predictive checks | NUTS (1.10) |

**Data.** Built on the opaque column types, metadata keys, reader interface and source rules fixed in sections 4.3, 6.1 and 8.6: a `spatial` package (tier 1) with `Geometry` columns from GeoParquet, GeoJSON, shapefile and GeoPackage, spatial joins, distances, buffers and reprojection on pure-Rust geometry crates, weights matrices, Moran's I and spatial lag and error models, Conley errors as a dependence structure in the variance layer, and maps through `coord(map, crs:)`; readers for survey platforms (Qualtrics, SurveyCTO, ODK and KoBoToolbox) that keep question text and labels; database and survey-platform sources with `secret()` credentials; a text-analysis package over corpora read from text, Word and PDF files (tokenisation, dictionaries, topic models); XML and SDMX readers; gridded data reduced to frames; a `Network` type with centrality and network regressions; a `Period` column type for quarterly, monthly, weekly and fiscal-year panels and series; survival analysis with `surv(time, event)` on the left of `~`; multiple imputation as `List<Frame>` with pooling by Rubin's rules; a salted hashing function for pseudonymising identifiers; and media through `FileRef` columns and `external()`, never inside the engine.

**Language.** Closures, written `|x| expr` (section 4.3 of the Language Reference); `Matrix` and `Vector` with the user estimators of 1.8; variadic parameters, if the open decision of section 15 on them is taken; user-defined types belong to 1.11 (section 15); grapheme-aware text functions, locale-aware collation and time zones (section 3.8 of the Language Reference); a second edition only when these need syntax that edition 2027 cannot carry.

**Engine.** 64-bit WebAssembly, which lifts the browser's 4 GB ceiling without changing the memory budget's meaning (section 8.3, point 7); out-of-core execution for more estimators than the streaming QR of 1.7; optional GPU kernels for forests and large matrix products, admitted only if they meet the determinism contract on every supported target, otherwise kept outside the result path.

**Ecosystem.** Zeff-source packages before 1.11 if the open decision of section 15 is taken; the registry's validated badge grown into a public conformance record per package; wrapper packages for public data APIs built on `fetch`; datasets added to `datasets` as their licences are verified.

**Reporting and assistance.** Report documents of text and Zeff chunks that re-run from the project and export to Word, PDF and HTML (Zemp Specification, section 8) and AI assistance that writes Zeff and never results (Zemp Specification, section 12) arrive with Zemp 1.11 (Zemp Specification, section 15); after them, a methods paragraph drafted from the run record.

**Governance.** A public change process for this document once outside contributors arrive (a proposal, a review period, a decision recorded in section 15); and the support policy above kept for every engine ever released.

---

---

## 15. Open decisions

Everything above is the current design; these items still need a decision.

**Language**

- [ ] User-defined types after 1.0: a `type` declaration (a named record plus the result interface it implements, such as `summary` and `estimates`), so a package author's `NewsvendorFit` works with `report()` and `.plot()`; extensibility without classes or inheritance, as Rust's structs and traits
- [ ] Multivariate outcomes (`[y1, y2] ~ x`) with seemingly unrelated regression and MANOVA, after 1.0
- [ ] Fixing a parameter in a model block: `x1 @ 1`, as Mplus writes it, so `*` keeps its one meaning
- [ ] Variadic parameters (`...`) in `fn`, so functions such as `coalesce` could be written in Zeff
- [ ] A structural equality function for lists, records and frames (`==` on them is an error, section 3.7 of the Language Reference)
- [ ] Locale-aware text collation and time zones after 1.0 (section 3.8 of the Language Reference fixes code-point order and zone-free DateTimes for 1.0)
- [ ] Grapheme-aware text functions (`len` and `slice` count code points in 1.0, so a combining accent counts separately)
- [ ] Zeff-source packages before 1.11: they need only the loader and the manifest, not the component sandbox, so they could ship in 1.6 or 1.7 and let others write methods while the first-party releases continue; component packages stay at 1.11
- [ ] Where the package registry's static index and archives are hosted (a public repository host costs nothing and is the default)
- [ ] Synthetic copies of a user's own data for sharing (a `synthesize` function), after 1.0
- [ ] A file holding several frames and fitted results together, for the heavy models of 1.5: a zip of Arrow files and JSON result descriptions in the versioned form of section 8.3, point 5, never a binary format of Zeff's own; in 1.0 `save` writes one frame and the result cache reproduces every fit (section 8.7); decide by 1.5 (zeff_open_decisions.md, #32)


**Build and release**

- [ ] Libraries not yet named, to confirm when the 0.1.0 workspace is set up: `arrow-rs` and `parquet` (the Apache Arrow Rust crates) for columns and files, needed from 0.1.0; `wasm-bindgen` with `wasm-pack` for the WebAssembly build; `tower-lsp` for the language server (0.7.0); each pinned exactly and upgraded one at a time (zeff_risk_register.md, R6)
- [ ] Panels and fixed effects (0.6.0) before tests and plots (0.7.0)
- [ ] Swap 1.2 (difference-in-differences) and 1.3 (latent variables) if the first users are mostly in organisational behaviour
- [ ] Bayesian estimation (1.10): which models come first, and whether priors are written in `model { }` blocks or as arguments
- [ ] Code signing for the Zemp desktop app: pay for Apple and Windows signing, or ship unsigned builds with published checksums until a free open-source signing service accepts the project. Installation is from the command line only for now (17 October 2026): the command line and the apps installed by `zeff app install` or a package manager need neither for that route, and setup files, which do, are deferred until this is decided (section 2.1; Zemp Specification, section 17; zeff_open_decisions.md, #23 and #38)
- [ ] A specification freeze while 0.1.0 to 0.3.0 are built, as the risk register's R12 recommends: the normative text changes only for bugs found while coding, and further design work is recorded as an open decision with a deadline release (zeff_open_decisions.md, #40); decide at 0.1.0
- [ ] Names: the domain for the installer, documentation and registry hosts (`zeff.dev` or `zeff-lang.org`) and the GitHub organisation, needed before 0.9.0, when the launcher ships with the URL of its signed key list, and so well before the installer URL of 1.0.0 (zeff_open_decisions.md, #31)

**Decided on 15 and 16 October 2026**

- [x] 1.0 limited to the language, toolkit, `reg` with `fe:`, `ivreg`, `logistic`, `probit`, `reliability` and causal graphs (224 functions at the time, 256 once the plot grammar, the `datasets` package and the adopted proposals were counted; seven hard engine pieces); DiD and PPML in 1.2, CFA and SEM in 1.3, mixed models in 1.4, double lasso and forests in 1.5, WLSMV and ESEM in 1.6, latent classes and streaming in 1.7 (release numbers as revised later the same day; section 14)
- [x] Fixed effects, random effects and instruments as named arguments, `fe:`, `re:` and `instruments:`, replacing fixest- and lme4-style `|` parts; `|` never appears in a formula
- [x] Failed draws handled by `on_fail:` and `max_fail:` in resampling and by `attempt` in user loops; still no `try`/`catch`
- [x] `zeff check` works from cached file schemas; data-dependent steps are checked at run time
- [x] Apache Arrow columns shared between frames, with selection vectors and lazy chains
- [x] Read-only data roots in `zeff.toml`, with file hashes recorded on every run
- [x] Validation by accuracy class instead of one tolerance for every method
- [x] Names: case-sensitive with no case-insensitive matching, ASCII identifiers, backticks for file names that are not valid names, `names: keep | snake` at import, `core` names not rebindable while other built-in names may be shadowed with a warning, and the naming conventions of section 3.6 of the Language Reference
- [x] Engine architecture (section 8.3): one typed IR for checker, editor and runtime; estimator trait with a separate variance layer; bit-identical floating point on every platform with Zeff's own kernel crate; content-addressed incremental runs; versioned output formats and editions; sandboxed WebAssembly components for extensions; worker with cancellation and a memory budget
- [x] Memory budget: browser default the smaller of 3.5 GB and half the device's memory, adjustable per project with `memory =` in `zeff.toml`; desktop uses available memory
- [x] Adversarial audit (zeff_audit_2026-10-16.md), 16 October 2026: sampling weights default to robust errors and refuse classical ones; a kernel crate of Zeff's own replaces `faer` on result-bearing paths, with a second x86-64 CI machine; loop variables and parameters win over columns and a clash is an error; canonical row orders for every verb; a statement may not begin with an operator; `:`, `as`, `~`, `=~` and `~~` are expression operators so the grammar is context-free; `fit.check` uses the generalized adjustment criterion and graphs gain `selected` nodes; Zemp separates analysis state from view state; ambiguous dates are never guessed; `merge` refuses shared non-key columns without `suffixes:`; filtered frames release parent buffers; tall-skinny QR and 64-bit text offsets; the cache key includes called functions; IV identification rules and Hansen's J; `test` takes Bool expressions in coefficient space; column equilibration before pivoted QR; and the medium items M1 to M19
- [x] Testing handbook (zeff_testing.md): reference, truth-recovery, property, edge-case, snapshot, cross-platform, fuzz, mutation and benchmark tests; routines per change, night, week and release; bug severities S1 to S5; a definition of done for each function
- [x] Error handling and bug reports (section 11.1 of the Language Reference): explained codes, statistical warnings (W1xxx), offending rows shown, deliberate exceptions and `--deny-warnings`, internal errors (Z9xxx) with self-checks after every fit, property tests and fuzzing, `zeff report-bug` with automatic shrinking, `zeff audit` against a known-issues list, patch releases
- [x] `core` independence made checkable: a CI rule that `core` depends only on kernel crates; a display interface declared in `core` for `print`; `help` reading the engine's function registry; `to(Date)` using kernel date parsing
- [x] Performance design (section 8.3, point 10): import cache, reuse of fixed-effect work, model matrices and factorisations, fused column expressions, integer-coded categories, panel indexes, warm starts, sparse structures, pushdown in lazy chains, deterministic parallelism, benchmarks in CI
- [x] Core semantics written down (section 3.7 of the Language Reference): kinds of word, type rules, the full missing-value truth table, evaluation order and block scope, positional variable ranges, rules for `Failed` values, stability within an edition; term names for model matrices; `zeff check` reports every error; weights with `NA`, negative and zero values; supported platforms, practical browser limits and a conformance protocol; a status rule saying what is normative
- [x] After an external review (16 October 2026): record fields read as `r.name`; functions see only their parameters and other functions; a name in call position is always a function; filtered frames gather kept rows when a column is next read; fixed effects removed by accelerated alternating projections before pivoted QR, with absorbed regressors reported; `to_wide` columns cached from the last run for checking; a project lock file with shared read access to data roots; a 1.0 speed target against fixest
- [x] Audit of all documents (16 October 2026): `minimize`, `root` and `gradient` moved to 1.8 with automatic differentiation; `n_distinct` given its own entry; `reliability` gives alpha in 1.0 and omega from 1.3; `bartlett_test` renamed `sphericity_test`; streaming by QR, not normal equations; pure-Rust elementary functions and fixed SIMD width in the determinism contract; formula addition (`f + terms`) defined; mapping keys in `re:` may be terms; error locations corrected
- [x] Readability: `=` for definitions and `:` for options; `[a, b]` and `[x1..x9]` as variable lists inside frame methods, `vars()` only outside them; results read by term name (`m.coef(educ)`), never by position
- [x] Causal graphs in 1.0: `dag { }` blocks with `adjustment_sets`, `implied_independencies` and `fit.check` for bad controls
- [x] Model-matrix rules written down (section 7.2 of the Language Reference), including collinearity notes and a warning for `(a + b)^2`; `interact()` for all interactions of an order
- [x] Contrast coding in `category()`; `fit.derive()` for nonlinear combinations by the delta method; `dist.truncated` and `dist.mixture`
- [x] Philox counter-based random numbers; interoperability through Arrow and Parquet files, plus a desktop-only, opt-in `external` step from 1.1; forward-mode automatic differentiation for user objectives in 1.8
- [x] Any `List<T>` may hold `Failed` values; CSV type guesses that fail later stop the run with the row named (Z0606); ordered items before WLSMV are an error that suggests MLR
- [x] Structure review (zeff_structure_review.md, 16 October 2026), package bundle: 29 packages; `results` (tier 1) holds the result and estimator interfaces, replacing `report` and taking the generic result methods out of `regress`; `dist` split from `stats`; `identify` first in tier 3 for the causal-graph functions; `cluster` renamed `groups` and `panel` renamed `multilevel`; the order within a tier is also the build order; empty packages and goals hidden from menus until they ship; the kernel's modules listed and frozen behind a specification change; third-party packages qualified unless `use`d, with a manifest, `zeff.lock`, `zeff add` and `zeff update`, and a validated badge
- [x] Structure review, release plan: version numbers major.minor.patch from now, 0.0.1 to 0.6 relabelled 0.1.0 to 0.11.0 (superseded 17 October 2026: see the fresh start of the code below) and 1.0.0 the name freeze; releases named for what they are for; Zeff 1.0.0 decoupled from Zemp and the desktop, which have their own release line; 1.1 is engine work alone, DiD and PPML move to 1.2 and each later method release one number on, with Design in 1.9 and Bayesian estimation in 1.10; proposed functions adopted or dropped by 0.7.0; cadence and support policy in section 14
- [x] Language-specification gap review (zeff_language_spec_gaps.md, 16 October 2026): a conformance clause; literal rules and escapes; continuation tokens; associativity; a token table and notation key; eight words reserved for later use; `Nothing`; structural records, typed mappings and function types; category and date comparisons with text; indexing and conversion rules; argument and default evaluation order; modules as files; no recursion and `fn` at top level only; section 3.8 of the Language Reference on text, time, files, Arrow types, limits, positions, printing and exit codes; `today()` and `now()` recorded and replayable; a glossary. From the second, independent reading the same day: NEWLINE meaning decided by the innermost bracket and the value of a block; raw strings and the regex dialect; `1e+6`, the smallest Int and correctly rounded Float literals; a type-join rule (`List<Result>`); comparison and identity as two relations (`NA` identical to `NA` for `sort`, `distinct`, `is_in` and grouping); a conversion table and implicit Int-to-Float; columns as values and no lifting of project functions; `Word` and `Type` values; integer edge cases and Float types for `factorial` and `round`; scalar special values and the rounding mode; typed bare `NA` and `[]`; no `+` on text; namespaces `dist` and `power` as registry lookups; misuse diagnostics for other languages' tokens; `case` as a condition chain; `break` and `continue` in collecting loops; narrowing after `is(Failed)`; order-independent definitions, exact argument matching and effects allowed only in scripts; the random-number derivation; parallel-loop observables; a formal edition with `"preview"` before 1.0.0; the interpreter path for `external` kept outside the project; writes confined to the project; pragma and `///` grammar; the CSV writing rules; a category-level warning
- [x] Help contract (section 9.1, 16 October 2026): every function ships with an R-style help page (summary, generated usage, description, every argument, value, details and references for statistical methods, generated warnings and accuracy, see also, checked examples, availability); a built-in without a complete page does not build, a module function without one fails `zeff check` (Z0319), a script function needs its summary line (W0303); example data through the built-in data root `zeff:`; W1001 to W1006 numbered
- [x] Leakage handling (section 7.5 of the Language Reference, 16 October 2026), after tidymodels: splits as roles (`holdout`, `training`, `testing`, `folds` with `strata:`, `group:`, `time:`); recipes as fitted transformations in chain syntax, estimated on the analysis rows of every fold; `spec.*` model specifications and `grid`; `learn`, `tune`, `score`, `best`; the testing view scored once; a leakage checker on column lineage (W1101 to W1106); cross-fitting by default in the causal methods; all in 1.5, with `holdout` moved from `data` to `resample`
- [x] Importing packages (section 8.4, 16 October 2026): two kinds of package, Zeff-source and WebAssembly component, under one manifest and the help contract; `use pkg`, `use pkg::name` and qualified names, also in method position; one version per project; `zeff add`, `update`, `remove`, `packages`, `vendor` and `publish`; a signed static registry whose CI grants the validated badge; yanked versions kept for existing locks; components bit-identical by running as WebAssembly natively too
- [x] The `datasets` package (section 8.5): public open-access and synthetic datasets with licence, provenance, cleaning or generating scripts and help pages; the `zeff:` data root; immutable within an engine version; small ones in the engine, large ones fetched and cached; `dataset` and `datasets` in 0.2.0
- [x] Plots as a layered grammar (section 7.6 of the Language Reference): `plot` with aesthetic mappings, marks, statistical layers computed by the engine's own functions, scales, coordinates, facets, guides and one default theme; the ten plotting functions become shorthands; Okabe–Ito and viridis by default; 25 grammar functions in 0.7.0 and `smooth`, `violin`, `aggregate`, `arrange`, `computed` in 1.2
- [x] Importing data (section 6.1 of the Language Reference, 16 October 2026): one `import` for delimited, fixed-width, Excel, OpenDocument, Parquet, Arrow, Stata, JSON, SPSS, SAS, R, HTML and local database files, with shared cleaning options and a note of what each format kept; `save` mirrors it with Parquet as the lossless interchange; readers in pure Rust or permissively licensed so the browser reads the same formats
- [x] APIs (section 8.6): a `web` package with `fetch`, `secret`, `flatten` and `rate_limit`; opt-in per project; every response cached, hashed and recorded, replayed by default, `--refresh` and `--offline`; secrets never in a project; fetched data vendored into replication packages; API wrappers as ordinary packages
- [x] Decisions of 16 October 2026 (zeff_open_decisions.md, the "now" group): the specification is published as two volumes (section 14; their common source was removed on 17 October, so each volume is edited directly); `iterate` (0.5.0), `event` (0.6.0), `posthoc` with factorial `anova(type:)` and `dist.tukey` (0.7.0) and `iv.check` (0.10.0) are adopted, so 1.0 has 256 functions and nothing remains proposed; the five power functions and `survey()` form the Design release 1.9 together with `factorial_design`, `fractional_design`, `response_surface`, `repeated_anova` and `tost`, now in the catalogue; the licence is Apache 2.0
- [x] Column types for the data to come (section 4.3 of the Language Reference, 16 October 2026): one column-type interface with Arrow extension metadata, types added only by the engine; computable and opaque types, with list, record, `Bytes`, `FileRef`, `Geometry` and `Duration` columns carried from 1.0 and computed on as their packages arrive; a metadata store with fixed keys replacing labels as a special case; `Network` reserved as a value type; a reader interface behind `import`; `fetch` generalised to sources; clustering generalised to dependence structures in the variance layer; `unnest` added to the roadmap (500 functions in all)
- [x] Future-proofing sweep (17 October 2026; sections 3.3, 3.8, 4.3, 6, 7.6, 8.3): `Period` reserved with a frequency in its metadata and the meaning of a panel gap fixed by the time variable's type; `Date − Date` stays Int days when `Duration` arrives; `Matrix` and `Vector` reserved with `*` as the matrix product, `.transpose()`, `m[i, j]` and `to(Matrix)`; embeddings as fixed-size lists; closures as `|x| expr`; censored outcomes through the formula's expression left side; composite panel units additive; plots as a versioned scene exported as SVG, PDF or PNG from a renderer with embedded fonts, byte-identical on every machine
- [x] The command line and installation (zeff_cli_install.md, 16 October 2026; section 2.1): a small launcher with engines installed on demand, verified against a built-in key and kept side by side in one store shared with Zemp desktop; installer script, Homebrew, winget, Scoop, signed archives, offline bundle, container image and GitHub Action at 1.0.0, PyPI and conda after 1.0 on request; `zeff repl` as an append-only session script with `zeff eval`; `zeff selftest`, `zeff engine`, `zeff doctor`, `zeff runs`, `zeff export replication` and `zeff verify`; baseline x86-64 builds; Z0709 to Z0711
- [x] Structure review, research goals: a thirteenth goal, Design, with Mechanism and Heterogeneity shown under Causal in Zemp's menus; a tagging pass giving `reg`, `logistic`, `probit` and `predict` the `predictive` tag, `logistic` and `probit` the `causal` tag, `describe`, `head` and `tail` `descriptive`, and `sample`, `seed`, `draw`, the power functions, `survey` weighting and `randomize` the `design` tag; generic result methods tagged `general`
- [x] Storage formats (17 October 2026; section 8.7): no file format of Zeff's own; Arrow IPC for the import and result caches, memory-mapped on the desktop and LZ4-compressed in the browser, and every format but Parquet and Arrow parsed once into the cache; Parquet with fixed writer settings for everything kept, byte-identical on every target, with statistics and a page index and panels sorted by unit and time for row-group skipping; the Zeff Parquet profile for labels, categories, panel declarations and extension types; BLAKE3 for file hashes, with unchanged files keeping their last hash; memory-mapped files counted by the pages touched; a file of several frames and results left open for 1.5
- [x] Audit of all documents (zeff_audit_2026-10-17.md, 17 October 2026): all 26 reserved words allowed as argument and field names, so `type:` parses, and `match` renamed `matching`; 243 functions by 0.7.0; the x86-64 baseline is SSE2 everywhere; `smooth(method: lm)` fitted by the kernel's least squares and `smooth(fit:)` drawing any fitted model, so `plot` calls nothing above it; `g.plot()` in 1.0 at 0.10.0 (257 functions); `iv.check` renamed `exclusion_check` and rule 6 rewritten for every name with two homes; package names may match ordinary arguments but not shared ones; `records:` replaces the JSON `path:` option of `import` and `fetch`; `matrix`, `vector` and `transpose` reserved in `core` for 1.8 and `surv` placed in `glm` (505 functions in all); a `Term` type for term names; one-sided formulas, `y ~ .`, the `spec.` namespace and `Fn(…, ..)` types in the grammar; Floats print to six significant digits and `report` tables to three decimals; `cluster:` takes one variable in 1.0; `zeff explain` (0.8.0) and the names deadline (0.9.0) aligned
- [x] Semantic versioning (17 October 2026): major releases may break scripts and come with a migration guide; minor releases may break scripts only before 1.0 and add without breaking from 1.0; patches fix without adding, and name every result they change (section 14, Version numbers); the release once planned as 2.0 (third-party packages, user-defined types) adds without breaking and is therefore 1.11, and no 2.0 is planned
- [x] Second audit of 17 October 2026 (zeff_audit_2026-10-17b.md), applied the same day: the two volumes are the edited files, with no generated source; the 0.1.0 row gains the function registry with the help build-check and the first five commands; a bare name that a newer engine's built-in would capture is an error (Z0109) that `zeff engine use` rewrites before the pin moves, so additions after 1.0 never change what a script calls; the version table's patch row relabelled, `zeff fix` promised only for editions, patch releases from 0.9.0 with the known-issues list and `zeff audit`; Z0412 and Z0413 written into the Language Reference; user-defined types and AI assistance each given one release home; the business model's conflicts with the Zemp Specification recorded as open decisions (#25, #27, #28, #34 to #37 of zeff_open_decisions.md) rather than settled
- [x] Fresh start of the code (17 October 2026): the prototype engine built as 0.0.1 is set aside and no version counts as released; 0.1.0 is rebuilt from scratch with the ten functions under their final names, on the tiered Cargo workspace, the typed IR, diagnostics as data and Arrow columns from the first commit, so later releases add functions rather than rework foundations; the specifications are unchanged
- [x] Source files per function (17 October 2026; section 9.1, Files): help is always one file per function; Rust code is grouped by family into modules of its package crate, so functions that share helpers share a module; Zeff code is one `.zeff` file per function in `functions/`, with private helpers and a small family read together allowed to share it; the registry build matches help to functions by name and fails on a function without a help file or a help file without a function
- [x] Names kept (17 October 2026): Zeff for the language and Zemp for the application, rather than alternatives considered the same day; trademark and domain checks remain open decisions (#31, #37 of zeff_open_decisions.md)
- [x] Risk register (17 October 2026): zeff_risk_register.md, twelve risks with early warnings, mitigations and fallbacks, reviewed at every minor release (section 14)
- [x] `.gitignore` from `zeff new` (17 October 2026; section 2): caches, run records, the project lock file and session script, Zemp's view state and `data/` left out by default, for the Git panel of Zemp and Zemp Lite (Zemp Specification, section 5)
- [x] Installation from the command line only, for now (17 October 2026; section 2.1): the launcher from the installer script, a package manager, an archive or the offline bundle, and the desktop apps, Zemp Lite and Zemp, from `zeff app install lite | zemp` or a package manager, each verified against the launcher's built-in key; setup files for Zeff and the apps deferred to the code-signing decision (zeff_open_decisions.md, #23 and #38)
- [x] Third audit of 17 October 2026 (zeff_audit_2026-10-17c.md), applied the same day: the four-target identity check and bundle tracking in the 0.1.0 row, as the risk register's R2 and R4 assume; `zeff app` and the offline bundle placed at 0.11.0 with Zemp's desktop beta; eighteen packages, not fifteen, empty in 1.0; the project lock file and session script added to `zeff new`'s `.gitignore`; a printed form for column values (section 3.8 of the Language Reference) and the help examples aligned with the printing rules; Zemp Lite web placed on Zemp web's origin; cross-references settled on each document's current file name; the open-decisions register brought up to date (#23, #33, #38 to #40)

---
