# Zemp Specification

Version of this document: 17 October 2026 · Engine today: none; Zeff's code starts afresh at 0.1.0 (Zeff Specification, section 14)

Zemp is a statistical application for empirical and causal research, built on the Zeff language. It aims at the analytical power of R and Stata with point-and-click and drag-and-drop on top. It comes in two editions, the second built on the code of the first, each on the web and the desktop: **Zemp Studio**, the full application, and **Zemp Lite**, a code-first edition without point-and-click for users who prefer to write Zeff, as RStudio is to R; the `zeff` command line completes the line (section 2). Zemp Lite is built and released first and alone; Zemp Studio's work starts when a review at the close of Zeff 1.5 decides (zeff_open_decisions.md, #125), and what this document says of Studio describes it for that time. Every action in Zemp writes a Zeff statement, shows it, and runs it, so clicking and typing always give the same result and every session leaves a script that re-runs. This document specifies the product: principles, product line, architecture, projects, the workspace, dialogs, plots, reporting, the desktop and web versions, updates, security, scope by research question, and the release plan. The companion files are the two volumes of the **Zeff Specification** (zeff_language_reference.md and zeff_engine_specification.md, with their section numbers shared), **zeff_functions.csv** (every function), the testing handbook (zeff_testing.md), the open decisions (zeff_open_decisions.md), the business model (zemp_business_model.md) and the risk register (zeff_risk_register.md).

---

## 1. Principles

1. **Purely statistical.** Zemp is statistical software for empirical analysis, not a general programming environment or a data-engineering tool.
2. **Code is the canonical form.** The interface writes Zeff and runs it; it never calls the engine directly. Point-and-click and syntax cannot disagree.
3. **Every procedure both ways.** In Zemp, everything is runnable by point-and-click and by syntax. Zemp Lite is syntax only by design (section 2): it leaves out the controls that write Zeff, never a procedure, so everything Lite runs, Zemp Studio can also run by point-and-click.
4. **Reproducible by default.** Work lives in projects that pin the engine version and record seeds and file hashes, so a project re-runs to identical numbers anywhere.
5. **Nothing hidden.** Every dropped row, skipped missing value, singleton, omitted collinear term and conversion is shown with a count.
6. **Readable output.** Plots and tables are drawn to fit the space they are shown in; nothing needs zooming to be read.
7. **Free and private.** Every tool and library used is free (code signing for the desktop apps' setup files, a domain name and trademark filing are the expected costs, open decisions #23, #31 and #37 of zeff_open_decisions.md); data stays on the user's machine unless the user moves it.
8. **Fast.** One Rust engine, near-native speed in the browser and full speed on the desktop.

Users are applied researchers in management, economics, organisational behaviour and operations; analysts who know Stata or SPSS; people who never type code but need a re-runnable record of what they did; and, for Zemp Lite, researchers who write code and want an editor, console and viewers without menus and dialogs.

---

## 2. Product line

Two editions, each in two forms, plus the command line. Every product runs the same projects with identical results.

| Product | For | Engine | Interface | Data limit |
| --- | --- | --- | --- | --- |
| Zemp Studio web | Quick analysis by point-and-click or code with nothing to install; links that re-run an analysis | Zeff compiled to WebAssembly, in Web Workers | Browser tab, installable as an offline app (PWA) | The project's memory budget: by default the smaller of 3.5 GB and half the device's memory; 32-bit WebAssembly caps it at 4 GB, so in practice about 2–3 GB of input data fits |
| Zemp Studio desktop | Research projects, large data, long runs, by point-and-click or code | Native Zeff: all memory, all cores | Tauri app window on the system web view (Electron if Linux rendering fails) | Available memory; files larger than memory streamed from 1.8 |
| Zemp Lite web | Writing Zeff in the browser with nothing to install | As Zemp Studio web | As Zemp Studio web, without the point-and-click modules | As Zemp Studio web |
| Zemp Lite desktop | Writing Zeff on one's own machine; installed beside the `zeff` command line with `zeff app install lite` (section 9), so that "Zeff + Zemp Lite" is one product line, as R ships with its own console | As Zemp Studio desktop | As Zemp Studio desktop, without the point-and-click modules | As Zemp Studio desktop |
| `zeff` command line | Servers, batch jobs, replication packages | Native Zeff | None (terminal) | As desktop |

**The two editions.** Zemp Lite is the Zemp interface built without the modules that write Zeff for the user; everything that shows, runs or exports is the same. The names: **Zemp** alone is the application and the product line, which both editions share; **Zemp Studio** and **Zemp Lite** are its two editions, named where the two are told apart (decided 1 November 2026).

**Lite first (decided 12 November 2026, zeff_open_decisions.md, #125).** Zemp Lite is built and released alone. Zemp Studio's work starts when a review at the close of Zeff 1.5 decides, which may set it after Zeff 2.0.0; until then everything below marked for Zemp Studio is planned for that time. The one-codebase rule is kept, the shared modules never depending on Studio's, so that Studio can start from the same code. Zemp Lite also has the Document settings dialog of a report (#124), since it writes the report's settings file and no Zeff.

| Part | Zemp Studio | Zemp Lite |
| --- | --- | --- |
| Project panel, editor with the language server, console, log, objects | Yes | Yes |
| Data, results and plot viewers, including sorting and filtering as view state, redraw to fit, compare, pop out and journal-size export | Yes | Yes |
| Exports of tables, plots and frames, replication packages, Update engine, run records, packages panel | Yes | Yes |
| Methods panel, dialogs and guides (section 6) | Yes | No; `zeff help`, F1 and the Help panel give every function's help page, and hover in a script gives its usage block (the one the call would run, where a name has two functions; from 0.2.4, #220) |
| Formula, graph, model, plot-layer and recipe builders | Yes | No; written as Zeff |
| Controls that write a statement: drag a data file into the editor, drag a frame onto a method, **Add to script** in the viewers, edits in the variable view, the import dialog | Yes | No; the variable view and the import preview are read-only |
| **Copy as Zeff** in the data, plot and variable viewers: the statement equivalent to a sort, a filter or a change of type, label or contrast, put on the clipboard; never a selection of rows (zeff_open_decisions.md, #114) | No; **Add to script** writes the statement instead | Yes; the script changes only when the user pastes it |
| AI assistance (section 12) | Yes | Not through 1.0; decided with zeff_open_decisions.md #27 and #28, the same features in both editions if any (#39) |

| Rule | What it means |
| --- | --- |
| One shared codebase | Zemp Lite is the shared interface, developed in a public repository; Zemp Studio is that interface with the point-and-click modules added, which are kept in a private repository of their own (section 16). Lite's download is therefore smaller, and every fix to a shared part reaches both |
| Same projects | A project made in either edition opens and runs unchanged in the other and from the command line; nothing in a project records which edition made it, and view state (`.zemp/views.json`) is shared |
| Layouts | Zemp Lite's editor is the **margin editor** (zeff_open_decisions.md, #174): the script in the centre, a narrow column of notes beside it, one per statement, and the whole output of a statement unfolded under it when the reader asks (section 5). The **Panes** layout of section 5 is kept beside it in the 0.2 alpha and reviewed at 0.3 (#176, R2). Zemp Studio opens in Panes. The choice is view state saved per project in `.zemp/views.json`. The script is a plain `.zeff` file in every layout: output is never written into it |
| Moving up | Zemp Lite offers **Open in Zemp Studio** for the current project; Zemp Studio has no "Lite mode", since a user who wants no dialogs simply ignores them or uses Lite |
| Same engine store | On a desktop, Zemp Lite, Zemp Studio and the `zeff` command line share one engine store (`~/.zeff/engines/`), so an engine downloaded by any of them serves all three |
| Free | Zemp Lite is in the free tier with no limits (zemp_business_model.md); it is free of charge and open source, under Apache 2.0 as Zeff is (section 16) |

The web and desktop interfaces are the same code in both editions. Desktop adds real project folders, native speed, background jobs, the updater and, from 1.1, the opt-in `external` step for R and Python scripts.

---

## 3. Architecture

```
┌──────────────────────── Zemp interface (TypeScript, web standards) ────────────────────────┐
│ Project panel · Editor · Console · Objects · Methods · Data viewer · Results · Plot viewer │
│ Dialogs generated from function signatures → write Zeff statements   (left out of Lite)    │
└───────────────────────────────┬────────────────────────────────────────────────────────────┘
                                │ Zeff source text only (the one execution path)
┌───────────────────────────────▼────────────────────────────────────────────────────────────┐
│ Zeff engine (Rust): parser · typed IR · checker · interpreter · 31 packages on 4 tiers ·   │
│ kernel · result cache                                                                       │
│   web: WebAssembly in a Web Worker (threads with cross-origin isolation from 1.1)           │
│   desktop and CLI: native binary, one per pinned engine version                             │
└───────────────────────────────┬────────────────────────────────────────────────────────────┘
                                │
                 Storage: project folder (desktop) · Origin Private File System (web)
```

| Part | Choice |
| --- | --- |
| Engine | Zeff in Rust; WebAssembly build for the browser, native build for desktop and CLI; bit-identical results on both (Zeff Specification, section 8.3) |
| Interface | TypeScript with standard web features only, so it renders the same in WebView2 (Windows), WebKit (macOS) and WebKitGTK (Linux); Svelte 5, bundled by Vite (decided 11 November 2026; zeff_open_decisions.md, #24) |
| Editions | Zemp Lite's public codebase is the shared interface; Zemp Studio is the same codebase with the Methods panel, dialogs, guides, builders and statement-writing controls added from modules kept in a private repository of their own (sections 2 and 16); Lite never depends on Studio's modules, which a build check enforces |
| Code editor | CodeMirror 6 with Zeff's tree-sitter grammar and the Zeff language server |
| Desktop shell | Tauri: a small app bundle, the system web view, signed updates |
| Engine ↔ interface | The interface sends Zeff source and receives results in the engine's versioned formats: tables as Arrow, plots as vector scenes, warnings and notes with codes |
| Packages | Shipped inside each engine version; `core`, `text`, `time` and `data` load at start, the rest on first use; from Zeff 2.0 at the earliest, third-party packages are fetched from the registry on first use, hash-checked and cached (Zeff Specification, section 8.4) |
| Plots | The engine returns a resolution-independent scene; the interface renders it to SVG or canvas at the exact pane size |
| Engine worker | The engine runs in its own worker, so the interface never freezes; long steps report progress and can be cancelled; a step that would exceed the memory budget stops before it starts |
| Parallel work | From Zeff 1.1, loops, bootstraps and forests run on a thread pool with reproducible random streams |

---

## 4. Projects and windows

Zemp works only in projects. A project is a folder with `zeff.toml` (name, pinned engine version, edition, optional memory budget and data roots), `zeff.lock` (not before Zeff 2.0: the exact versions of third-party packages), `scripts/`, `functions/`, `data/`, `output/`, `.zeff/runs/` (recorded seeds, warnings, file hashes and outputs of every run), `.zeff/schemas/` (cached column names and types) and `.zeff/cache/` (results reused when code and inputs are unchanged). The edition in `zeff.toml` is the Zeff language edition, not the Zemp edition; both Zemp and Zemp Lite follow every rule of this section.

| Rule | Behaviour in Zemp |
| --- | --- |
| Project only | Opening Zemp shows a project chooser; there are no loose scripts |
| Scratch project | A Scratch project is always available for quick exploration; its work can be promoted to a named project in one step |
| One project per window | Each window holds one project, fully isolated; frames, bindings and results are never shared between windows |
| No duplicate opening | Opening a project already open elsewhere brings that window forward with "already open in another window"; a lock file in the project's `.zeff/` folder enforces this across processes, and data-root files are read with shared access so several projects can use them at once. The lock holds across editions: a project open in Zemp Lite cannot also be opened in Zemp Studio |
| Data roots | Large shared datasets are declared once as read-only data roots and read in place (`import("lfs:wages.parquet")`); Zemp shows them in the project tree and warns when a file has changed since the last run. In the browser the user grants access to the folder once |
| Explicit sharing | Code moves between projects only through Import from another project; data through Import or a data root |
| Engine pin | A project opens with the engine its `zeff.toml` names; Zemp installs that engine if it is missing (desktop) or loads it (web) |
| Replication package | Export a project as a zip with `zeff.toml`, scripts, functions, data or data-root checksums and the run record, ready for a journal; `zeff verify` re-runs it and confirms identical numbers |

---

## 5. The workspace

One window, one project. Panels can be resized, collapsed and rearranged; layouts are saved per project.

```
┌───────────────┬──────────────────────────────────────────┬──────────────────────┐
│ Project       │ Editor (tabs: main.zeff, functions/…)     │ Objects              │
│  scripts/     │                                            │  frames  results     │
│  functions/   │                                            │  plots   graphs      │
│  data/        ├──────────────────────────────────────────┤──────────────────────┤
│  data roots   │ Console · Log (warnings, notes, runs)      │ Methods              │
│  output/      │                                            │  by research question│
│  runs         │                                            │                      │
├───────────────┴──────────────────────────────────────────┴──────────────────────┤
│ Viewer: Data · Results · Plots (tabs; can pop out into its own window)            │
└──────────────────────────────────────────────────────────────────────────────────┘
```

| Panel | What it shows | Key behaviour |
| --- | --- | --- |
| Project | The project folder tree, data roots, scripts, functions, data and outputs, and the list of past runs | Drag a data file into the editor to write its `import`; open a past run to see its seeds, warnings, file hashes and outputs |
| Editor | Zeff scripts and project functions | Completion, signature help, hover, live diagnostics and formatting from the Zeff language server in 1.0; quick fixes, navigation, rename and inlay hints from 1.1; run statement, selection or script |
| Console | Interactive Zeff and printed output | History, re-run, copy as script; out-of-order runs flag the session "out of sync" |
| Log | Every warning, error and note with its code and line, and which results came from the cache | Click to jump to the line; filter by kind; each entry links to its help; statistical warnings (W1xxx) are marked apart from coding errors; an internal error offers "Report a problem", which builds a local bug report with a shrunk reproducer for the user to review |
| Objects | Every binding: frames (rows × columns), results, formulas, causal graphs, recipes, tables, plots, lists; memory used against the project's budget; a frame split by `holdout` shows its training and testing views, the testing view with a lock badge until it is scored | Type and size at a glance; drag a frame onto a method to open its dialog; preview on hover; bindings downstream of a re-run statement are marked stale, with one click to re-run them in dependency order (stale results stay valid values, since bindings never change) |
| Methods | Every function, browsable by research goal (eleven entries from 13 purpose tags, with Mechanism and Heterogeneity under Causal), by workflow stage (Prepare, Describe, Model, Report), by package, or by search | Each entry opens the function's dialog or inserts its usage, and shows release and status; a goal or a package appears only once at least one of its methods has shipped, and the rest are listed under "Coming in 1.x" with their release; third-party packages show their validated badge |
| Data viewer | A frame as a grid | Sorting and filtering here are view state and never change the script; **Add to script** writes the equivalent `sort` or `where` statement at the cursor and shows it first; variable view shows types, labels, missing codes, categories and contrasts, and every metadata key (unit, measure, question, source; Zeff Specification, section 4.3); an opaque cell (a list, record, file or geometry) shows its summary, and a `FileRef` opens the file; frozen first column; a statistics header on every column (type, missing count, a small distribution, asked of the engine); search with matches marked; a live row count under a filter; copy of a range; each variable shown or hidden as view state; in Zemp Lite, the frames of the run listed each by the statement that made it (#75), from 0.2.1 in the left pane of the Data place, **Frames of the run**, with a read-only **Variables** pane at its right that shows the list of variables or one variable's properties (#203) |
| Results viewer | Model and test results as cards | Expand to the full summary; add to a report table; copy as text, Word or LaTeX; results computed on a training view are labelled exploratory, and a `LearnFit` card offers "Score on test" once (from 1.14) |
| Plot viewer | All plots of the session (section 7), including causal graphs | Redraw to fit, compare, pop out, export at journal sizes |
| Help | The help page of the function under the cursor, and a code's explained page | Follows the cursor unless switched off; opened by F1, ⌘K and **Explain**; in both editions (#74); from Zemp Lite 0.2.3 a home page, **Back** and **Forward**, a **Topics** pane and search as one types, and from Zemp Lite 0.3 a **Run** button under each example (#211); at every release Help agrees with the engine the release carries: the code in Zemp's own pages runs on that engine at the build and prints what the page shows, every name they use exists in that engine, and every picture is made again from the running app, the build failing where one does not hold (#211, item 12) |
| Packages (not before Zeff 2.0) | The project's third-party packages and the registry | Search, add, update and remove packages; each shows its version, licence, validated badge and functions; results from an unvalidated package are marked (Zeff Specification, section 8.4) |

**In Zemp Lite.** The editor is the margin editor (below). Its places are **Script**, **Data** and **Plots**, and **Paper** from Zemp 0.3 (for `.typ` files; section 15); the project tree is a drawer under **Projects ▾**. The behaviours that write a statement are absent (dragging a data file into the editor, dragging a frame onto a method, **Add to script**, adding a card to a report table, edits in the variable view), and the variable view is read-only. Function help is reached by hover, completion, F1, the Help panel (#74) and `:help` in the console. The data, plot and variable viewers offer **Copy as Zeff** instead, which puts the equivalent statement on the clipboard for the user to paste; a selection of rows is not copied (#114). In Panes, kept beside the margin editor at the 0.2 alpha (#176, R2), one left panel has the tabs Project and Objects, there is no right panel, the data, results and plots open as editor tabs beside the script, and the script's output sits in a panel under it, each result under the line that made it (#146, item 7).

**From Zemp Lite 0.2.1: three panes on every place (#203).** The margin editor and Panes become one layout: every place (Script, Data, Plots, Help) has a left pane, the place itself in the middle and a right pane, each opened and closed by its own link in a bar under the top bar, and each place remembers which of its panes are open. In Script the left panes are **Project** and **Outline** and the right pane **Notes**, the margin described below; the **Objects** panel, the **Panes** button and the **Margin** link are removed, and **Run** unfolds every output under its statement. What follows describes the margin editor as the 0.2 alpha has it, and holds from 0.2.1 except where #203 says otherwise.

**The margin editor (Zemp Lite's editor, #174).** Running never moves the code. The script fills the centre as plain text; a margin of about 300 pixels at its right holds one **note** per statement on the statement's first row: the name it binds or its verb, the type and the dimensions the engine gives (`wages Frame · 15,571 × 13`, `m3 RegFit · N 15,366`), and a warning's or an error's code. A note holds no estimate, measure of fit or drawing. Clicking a note, or ⌥↵, **unfolds** the statement's output under its last line: what the engine said, then what it printed at the width of the place, or its figure; every character is the engine's, and **Copy → Text** is what `zeff run` prints. A script opens with every output folded, under three controls, **Notes · Fold all · Unfold all**, all view state. A block (`for`, `iterate`) has one note, and its unfolded output carries the passes of #76. A refused script runs nothing and loses nothing: the error stands under its line, always unfolded. The foot of the window is one line: the state of the page against the script, the console line, and the engine. The console line runs a statement after the script without adding it to the script; ↑ and ↓ recall history and Tab completes; `:help`, `:vars`, `:save` and `:clear` work as in `zeff repl`; ⌘↵ on a console line copies it into the script at the cursor. On the desktop only, a **Terminal** button at the right of the foot opens the computer's own terminal in the project folder (as Projects ▾ → Open terminal here and ⌃⇧` do); the web editions show none. The top bar is identical on the web and the desktop:

| Control | What it does |
| --- | --- |
| **Projects ▾** | As before: open, create or switch projects; the project tree as a drawer; the folder in Explorer or Finder; **Open terminal here** (desktop only) |
| Project and script | The open project and the script shown, as a menu of the project's scripts, with its saved state |
| The places | **Script · Data · Plots** (and **Paper** from Zemp 0.3), the current one underlined |
| ⌘K | A search field: every command, with its key and its standing |
| **A− 14 A+** | The code size, 12 to 22 px, changing the code, the notes and the prints (#146, item 9) |
| **Git** status | As before, desktop only from 1.1 |
| **Dark** switch | Switches between the light and dark themes |
| **Check** and **Run** | `zeff check` on the project; **Run** runs the script (⌘⇧↵) and ⌘↵ the statement at the cursor with every statement it reads that is out of date or not yet run (#176, R9); **Run to here** and **Run from here** in ⌘K, and *Run to here* on resting over a statement's line number (#76, #181) |

Everything else (the run's facts, what the run said, export, update the engine, help, settings) is reached through ⌘K or by clicking the foot.

**Stale output.** An output is fresh only while it is the result of the code above it. When a statement is edited, or anything it depends on changes (a statement it reads from, a project function it calls, an input file, the seed or its place among the statements that draw), its note and any unfolded output are marked in place, keeping their last value: `edited · not run yet` on the edited statement, `stale` on every result downstream, hatched and quieter, with the reason on resting over it ("reads m2, edited at line 15"); the foot and **Run** count them. Outputs that do not depend on the change stay as they are. The dependency rule is the one the engine already uses for its cache and for stale bindings in the Objects panel (Zeff Specification, section 8.3, point 4), so Inline adds no logic of its own, and a stale result can never be mistaken for a current one. Until the engine says, for each statement, whether it would reuse the result (#175, C3, with the cache of 0.11.0), the interface marks by the engine's own dependency graph, a statement that reads a file or draws random numbers counted as stale whenever anything above it is edited: too many marked is safe, too few is not.

**The welcome screen.** With no project open, Zemp Lite shows one quiet screen: **New project…**, **Open folder…**, **Open a replication package…** (from 0.11.0), **Clone from Git…** (on the desktop from 1.1, with the Git panel) and **Scratch project**, each with a one-line description and its shortcut, a list of recent projects with their folders and when each was last opened, and the Dark switch; the line at the bottom says no project is open. Choosing a project opens it in this window. **Open a replication package…**, in both editions and in ⌘K, takes a `.zip` (chosen or dropped on the window) and unpacks it into a new project, in the browser's private file system on the web or a folder the user picks on the desktop; it refuses a path that would leave the project and a package larger than the memory budget, shows the package's first page (engine version, how to run, external steps) and opens the project with the engine it pins. That page offers **Verify**, which re-runs every script of the run record fresh and offline and shows *identical* or each difference, as `zeff verify` does; on the web the external steps of a desktop project are not run, and the result says so (zeff_open_decisions.md, #117).

**Planned for the margin editor (before Zemp Lite 1.0).** Sections begun by a comment `// # Title` (`// ## Title` a level below), with an outline and **Run section** (#178); **Run from a fresh session** (#180); the error's trail (#183, from the 0.2 alpha); the passes of a block searched and a field drawn across them, a pass or a call opened by replay, and the console line inside it (#184 to #186); where the time and memory went (#187, from the 0.2 alpha); **Compare with run n** and the last run shown on opening (#179), with the run records of 0.11.0. A ⌘K command **Export script with output** writes the script with every output unfolded, as an HTML or PDF file with the engine version and run record line at its foot (#172 for the PDF). In Zemp Studio the same editor sits in the middle column of the Panes workspace, with the Project, Objects and Methods panels kept.

**Git.** A Git panel shared by both editions: the branch, the changed files with line counts, a commit message, and Commit, Pull and Push. It works with any Git remote (GitHub, GitLab, Bitbucket or an institution's own server), signing in to GitHub being a shortcut, not a requirement. `zeff new` writes a `.gitignore` that leaves out the caches (`.zeff/cache/`, `.zeff/fetched/`), the run records (`.zeff/runs/`), the project lock file and session script in `.zeff/`, `.zemp/views.json` and `data/` (Zeff Specification, section 2); the lock file must be left out because the panel commits while the project is open. A data file is committed only when the user adds it, and the panel says what is left out. Nothing leaves the machine until the user presses Push. On the desktop the panel uses the Git installed on the computer and arrives with 1.1; in the browser it waits on the hosting decision of section 17, since pushing from a web page needs a sign-in flow and a relay server.

**Analysis state and view state.** Sorting, filtering, zooming, brushing and selecting in the data, results and plot viewers, and positions dragged in a causal graph, are view state: they change what is shown, never the script or any result, and are saved in `.zemp/views.json`, which a replication package omits. Each viewer offers **Add to script**, which writes the equivalent statement at the cursor and shows it before inserting it. A selection of more than 20 rows is written to `data/selections/<name>.csv` and joined by one statement, `let picked = wages.merge(import("data/selections/outliers.csv"), on: worker, join: inner)`, so the script stays readable and the selection is reproducible. For a causal graph, **Add to script** writes the dragged positions into the `g.plot()` call as `layout:` or `at:`; an exported figure uses only the positions the script holds, and the viewer says when dragged positions differ from them (decision #121).

**Themes and access.** A light theme and a dark theme in both editions, switched from the top bar and remembered per user, not per project. The dark theme changes the interface and the on-screen plots' ground and text; exported tables and plots always use the light print theme, so a figure looks the same in every paper whichever theme drew it on screen. Full keyboard operation, screen-reader labels on every control, adjustable interface text size, and colour-blind-safe plot palettes by default.

**The look (decided 14 November 2026, #146, and 18 November 2026, #174: the quiet look).** The logo D4 in the logo's violet; Atkinson Hyperlegible Next for the interface, JetBrains Mono for code, Newsreader for the names of things (a project, a frame, a plot) and large headings only, shipped with the app; app screens at 13.5 px and no text under 12.5 px; code at 14 px, 12 to 22 by the code size control. One surface with hairlines between regions, words for controls, one filled button (**Run**), the current place underlined in violet, a dashed underline on any control that changes only the view. Both themes take their colours from one set of tokens, the dark theme a second set of the same names; every colour used for text reaches at least 4.5:1 against its ground in both themes.

| Token | Use | Light | Dark |
| --- | --- | --- | --- |
| window | The one surface | #ffffff | #111216 |
| soft | A control pointed at; the search and console fields | #f6f6f4 | #191a1f |
| softer | An unfolded output | #fafaf9 | #15161a |
| tint | An edited line | #f8f6fd | #141120 |
| line, line strong | Hairlines | #ececea, #dededa | #22242a, #2e3037 |
| ink | Text | #17171a | #ececef |
| ink 2, ink 3 | Secondary text; small print, line numbers | #55555c, #6b6b73 | #a7a8b0, #8f919c |
| accent, accent soft | The logo's violet: active, links, the filled button; selection | #5b3a9e, #f3effb | #b9a4f0, #201a33 |
| warning, warning soft | Warnings, missing values, the "edited" mark | #8a5d00, #fbf6e9 | #e2b552, #272215 |
| error, error soft | Errors | #b3261e, #fcf0ef | #f2a39b, #2a1715 |
| ok | In sync; the same | #2b7a4b | #8fdcab |

The code colours keep the categories of the earlier table (keyword, function or method, option, operator, text in quotes, number, comment, variable or binding), with the values of the owner's design guide, which hold the same contrast; until the language server gives semantic tokens (1.1) they come from the interface's own reading of Zeff's words. Figures are on white in both themes: they are the engine's print figures, and exports use the light print theme.

**Density, icons, menus and screen size (decided 11 November 2026, zeff_open_decisions.md, #33).** One density, with a **Larger type** setting that scales the whole interface by 1.15, remembered per user, beside Ctrl + and Ctrl −. No icons in 1.0: every control is a word. On the desktop, native menus mirror the ⌘K command palette one for one. No tablet or phone layout in 1.0; below 1200 pixels of width the browser editions say so.

---

## 6. Point-and-click: dialogs that write Zeff

Dialogs, guides and builders are part of Zemp Studio only; Zemp Lite leaves this section out (section 2). Zemp Studio is deferred (zeff_open_decisions.md, #125), so this section describes it for when its work starts; the menu plan recorded with #125 (one entry per function with its variants inside the dialog, method menus as modules, one Results menu) is weighed then.

Every dialog is generated from the function's signature and help, so each of the functions in zeff_functions.csv has a dialog the day it ships, and a project's own `fn` functions get one too.

| Parameter type | Control |
| --- | --- |
| `Frame` | Frame selector (or the frame dragged onto the method) |
| `Formula` | Formula builder: drag variables into outcome and regressor slots; the formula text updates live |
| `Terms` (`fe:`) | Drop zone for fixed-effect variables, with a switch to interact two of them (`industry:year`) |
| `instruments:` | Two drop zones: instrumented regressors and excluded instruments; an instrumented variable is added to the regressors automatically |
| `re:` (from 1.11) | One row per grouping variable, each with its random terms (intercept, slopes); nested and crossed groups by adding rows |
| `Var`, `Vars` | Drop zones that accept variables from the frame's variable list; ranges like `x1..x9` by shift-select |
| Definitions (`add`, `update`, `aggregate`, `rename`, `derive`, `frame`) | A table of new names and expressions, written as `name = expr` |
| Fixed list (`se: classical \| robust …`) | Segmented choice or menu, default marked |
| `Bool` | Switch |
| `Int`, `Float` | Number field showing the default and allowed range |
| `Dag` | Graph builder: draw variables and arrows, mark latent and selected variables, exposure and outcome; or type the `dag { }` block; both stay in sync. Dragged node positions are view state until **Add to script** writes them into `g.plot()` (section 6). The canvas is the one every later builder of boxes and arrows shares |
| `Model` (from 1.12) | Model-block builder: draw factors and paths, or type the block; both stay in sync |
| `Record`, `List` | Small table editor |
| `Fn` (`iterate`'s `step:`) | Function picker listing the project's functions whose signature matches |
| `Plot` | A layer list: the frame and its mappings at the top, then one row per layer, scale, facet and theme setting, each editable, written as the plot chain (Zeff Specification, section 7.6); the quick plots write the shorthands and open in the same list |
| `Recipe` (from 1.14) | An editable list of steps, each with its variables and options, written as the recipe chain; the leakage checker's warnings (W1101 to W1106) appear beside the step or the split that caused them |

Rules:

- **Code first, always visible.** Every dialog shows the Zeff statement it will write, updated as options change, and the statement is what runs.
- **One step per dialog.** A dialog appends one `.method()` step to a chain. Zemp shows a chain as an editable list of steps, so a filter can be changed or removed after the fact.
- **Checked before running.** The language server's checks run on the statement as it is built, so a dialog cannot submit an invalid call.
- **Question-first menus.** The Methods panel and the menu bar open on the research goals (section 11), using each function's purpose tags, then list the methods that answer that goal; a second axis by workflow stage (Prepare, Describe, Model, Report) serves users whose question is "make table 1".
- **Guides.** Common analyses are short guided sequences of dialogs that write ordinary Zeff. In 1.0: descriptive table 1, a regression table with robustness columns, IV with diagnostics, a two-period DiD, a logistic regression, scale reliability, and a causal graph with its adjustment sets and bad-control check. Event studies of staggered designs follow in 1.19 and CFA with fit and reliability in 1.12.

---

## 7. The plot viewer

Plots are never scaled bitmaps. The engine returns a resolution-independent scene, and the viewer redraws it to the exact size of its pane, so text, ticks and legends stay readable at every size and nothing needs zooming. The plot viewer is the same in Zemp Studio and Zemp Lite.

| Feature | Behaviour |
| --- | --- |
| Redraw to fit | Resizing the pane re-lays out the plot (text sizes fixed in points, labels wrapped or thinned, legend moved), never stretches it |
| List of plots | Every plot of the run, listed at the viewer's left by script, each with a small picture, its name, its line (a link to it) and its kind; the console line's plots apart, as not in the script; the list hides with **‹ Hide**, and its order and the plot shown are view state (#174); from Zemp Lite 0.2.1 the list is the left pane of the Plots place, **Plots of the run**, and a right pane, **The plot**, gives the plot's statement, line, kind and export size (#203) |
| Compare | Pin two to four plots side by side on shared axes |
| Focus | Click a panel of a faceted plot (`by:`) to show it alone at full size; Esc returns |
| Pop out | Any plot opens in its own window, which can sit on a second screen and still redraws to fit |
| Inspect | Hover shows the data values under the pointer; click a point to see its row in the data viewer |
| Causal graphs | `g.plot()` lays out a graph with latent variables marked; clicking a variable highlights its back-door paths |
| Journal sizes | Preview and export at one-column, two-column and slide sizes; exported text is never below 8 pt |
| Export | PDF, SVG, PNG (any DPI), and the Zeff statement that made the plot |
| History | Plots from earlier runs stay browsable from the run record |

---

## 8. Data, results and reporting

**Importing data.** An import dialog previews the file, detects types and missing codes, proposes a date format for date-like columns and shows three parsed examples beside the raw text (writing the `types:` entry only when the user accepts it, since Zeff never guesses an ambiguous date format), strips a byte-order mark, offers `names: snake` for awkward column names, and writes one `import()` call with its options. Format panes add what each format needs: a sheet picker with a preview and range selection for Excel and OpenDocument, a tree for choosing the records inside a JSON file, a table chooser for HTML, an object chooser for R files, and a summary of the labels and missing codes a Stata, SPSS or SAS file carries. CSV in 0.1.0; Excel, OpenDocument, Parquet, Arrow, Stata and JSON from 0.11.0; SPSS, SAS, R and HTML from 1.1 (Zeff Specification, section 6.1). Data files stay in the project's `data/` folder or a declared data root; the shipped datasets (Zeff Specification, section 8.5) appear under their own tab with licence and source, and **Add from URL** (from 1.1) writes a `fetch` call for a project that allows network access. In Zemp Lite the preview is read-only and the user writes the `import()` call; the formats are the same.

**Sources.** A Sources tab lists every file and fetched response the project has read, with its hash, size, date and the statements that use it, so a replication package can state its provenance; a file that changed since the last run is flagged.

**Variable view.** For each variable: type, label, value labels, missing codes, measure, unit, question text and source where the file carried them (Zeff Specification, section 4.3), category levels and contrast, and a small distribution sketch. Changing a type, label or contrast writes the matching Zeff step (`label`, `recode`, `category`); in Zemp Lite the view is read-only. From Zemp Lite 0.2.1 it is the **Variables** pane of the Data place, and its **Copy as Zeff** buttons for a change of type, contrast or label are removed (#203).

**Results.** Each result is a card with the headline numbers, warnings and a button for the full summary. Cards can be added to a report table, which writes `report([...])`.

**Exports.** Tables to Word (.docx) and LaTeX (HTML after 1.0); plots to PDF, SVG and PNG; frames to CSV, Stata, Parquet and Arrow, with labels and categories kept; every export also records the statement that produced it.

**Reports and papers (section 15).** A report document of text and Zeff chunks that re-runs from the project and exports to Word, PDF and HTML, in the forms #73, #124 and #141 decide. In Zemp Lite and Zemp Studio it is written in **Paper**, the margin editor's place for writing: the source beside its page as Typst sets it, every number on the page from the engine's run (#174). It arrives in three stages: `.typ` files from Zemp 0.3, with Typst a module of Zemp's own fetched on first use (#172); the settings file, the references and the Article template at Zemp 0.4 (#124); and, after 1.0 as #141 places them, `.zmd` papers with Zeff chunks and inline values, which are then the default, HTML and Word, and the Document settings dialog.

---

## 9. Zemp desktop

Everything in this section applies to Zemp Studio desktop and Zemp Lite desktop alike, except where the Installer row says otherwise.

| Area | Specification |
| --- | --- |
| Platforms | Windows 10 and later (WebView2), macOS 12 and later (WebKit), mainstream Linux (WebKitGTK); x86-64 and ARM |
| Installation | From the command line only, for now; setup files come later (section 17; zeff_open_decisions.md, #38). The `zeff` launcher installs the apps beside its engines: `zeff app install lite` (or `studio`) downloads the desktop app, verifies its signature against the launcher's built-in key and puts it in the user's applications, per user and without administrator rights; `zeff app list`, `zeff app update` and `zeff app remove` do the rest. So "Zeff + Zemp Lite" is two commands: the installer script (or Homebrew, winget, Scoop; Zeff Specification, section 2.1), then `zeff app install lite`. Package managers carry the apps too: `winget install Zeff.ZempLite`, `scoop install zemp-lite`, `brew install --cask zemp-lite`, and the Linux package tools or Flathub, with `zemp-studio` in place of `zemp-lite` for Zemp Studio; a package-manager install updates through the package manager, not the app's own updater. Both editions can be installed side by side |
| Offline install | The offline bundle of the Zeff Specification (section 2.1) also carries each app, installed with `zeff app install lite --from <file>` on locked-down university and company computers |
| File associations | Opening a `zeff.toml` or `.zeff` file opens its project; double-click to open in Zemp Studio, or in Zemp Lite when only Lite is installed; with both installed, the user's choice decides |
| Windows | One project per window; any number of windows; plots and data viewers can pop out to more windows |
| Performance | Native engine and full memory in 1.0; all cores from 1.1; files larger than memory streamed for OLS and fixed effects from 1.8 |
| Background jobs | Long runs continue in the background with progress and a notification; the window stays usable |
| External scripts (from 1.1) | Opt-in per project: `external` runs a local R or Python script on a frame passed as Parquet; the interpreter's path is a Zemp setting on this computer, never read from the project, and the first external step of a project asks for confirmation; results that depend on it are marked in the run record as outside Zeff's guarantees; never available in Zemp web |
| Engines | Several engine versions installed side by side in the engine store `~/.zeff/engines/`, shared with the `zeff` command and with the other edition, so an engine downloaded by any of them serves all; each project uses the one it pins; Zemp offers to remove engines no project uses |
| Command-line tool | Settings → Install command-line tool puts the `zeff` launcher on PATH (Zeff Specification, section 2.1); Export replication package and Update engine are `zeff export replication` and `zeff engine use` |
| Security | Every part signed and checked against a key built into the app; operating-system code signing and macOS notarisation as settled by the 0.5 desktop beta (open decision, Zeff Specification, section 15) |
| Telemetry | None by default; an opt-in crash report shows exactly what it would send; what the project counts about downloads and visits, outside the app, is in section 13 |

### 9.1 Updates

The desktop app updates part by part and downloads only the parts that changed, much as Rust's installer rustup updates each component separately; it never re-downloads the whole app. Zemp Lite updates the same way and never downloads the point-and-click modules.

| Part | How it updates |
| --- | --- |
| App shell (Tauri window) | Rarely; signed with the app's key, and signed for the operating system once code signing is settled (by 0.5) |
| Interface files | Only changed files |
| Engines | Each new engine installs beside the old ones, as a binary patch where possible; projects keep the engine they pin |
| Help and examples | Only changed pages |

Updates download in the background, apply at the next restart, and keep the previous version for one-click rollback. A project can be moved to a newer engine with "Update engine", which runs the project on both engines and shows any differences before changing `zeff.toml`. Moving to a new edition runs `zeff fix --edition` and shows the rewritten lines first.

---

## 10. Zemp web and the command line

**Zemp Studio web.** Runs entirely in the browser: the engine is WebAssembly in a Web Worker, projects live in the browser's private file system (Origin Private File System) and can be exported or synced to a folder. That storage is on the user's own device and is lost if the user clears the site's data, or if the browser evicts it, so Zemp web asks the browser for persistent storage when the first project is created; when that is refused, or a project has not been exported for 30 days, a line in the status bar says so, with **Export project** in ⌘K, and nothing is exported without the user (#117). It installs as an offline app. From Zeff 1.1, hosting sends the cross-origin isolation headers threads need. Data never leaves the machine; a "re-run" link carries the scripts and engine version, not the data, and asks the viewer for the data files.

**Zemp Lite web.** The Lite build of the same web app, at its own path on Zemp Studio web's origin (`…/lite`; the domain waits on hosting, zeff_open_decisions.md #27, and the trademarks, #37) and installable as its own offline app with its own manifest scope, with everything above. Both web editions on one origin share the browser's private file system, which is kept per origin, so a project made in one opens in the other; **Open in Zemp** moves the current project across without exporting it.

**Command line.** `zeff new`, `zeff run`, `zeff check`, `zeff fmt`, `zeff test`, `zeff lsp`, `zeff help`, `zeff version`, `zeff fix --edition`, `zeff explain`, `zeff report-bug`, `zeff audit`, `zeff repl`, `zeff eval`, `zeff selftest`, `zeff engine`, `zeff runs`, `zeff export replication`, `zeff verify`, `zeff doctor`, `zeff app`, `zeff dataset fetch`, `zeff self` and `zeff completions`, as in the Zeff Specification (section 2.1), installed by the installer script, Homebrew, winget, Scoop, an archive, the offline bundle, the container image, or Zemp desktop's Install command-line tool. A project made in either edition runs unchanged from the command line; `zeff verify` is how a replication package is checked, and `zeff repl` is the console without the window.

---

## 11. Scope by research goal

Zemp is organised around 13 research goals, shown as eleven menu entries: Mechanism and Heterogeneity sit under Causal as the questions "how?" and "for whom?". Every function carries one or more goals as `purpose` tags (infrastructure such as maths or file handling is tagged `general`), which drive the question-first menus; a second menu axis orders the same methods by workflow stage (Prepare, Describe, Model, Report). A goal appears in the menus only once at least one of its methods has shipped; until then it is listed in the roadmap view with its release. Zemp Lite has no menus by goal but runs every method in the table, since the methods belong to Zeff. The table gives the Zeff packages that serve each goal and the full scope planned across releases; zeff_functions.csv lists every method with its release and status (implemented, planned for 1.0 or roadmap).

| # | Research goal | Packages | Methods in scope | In 1.0 |
| --- | --- | --- | --- | --- |
| 1 | Design: how should the data be collected? | experiment, survey, stats, data | Power and sample size, minimum detectable effects, blocked, stratified and cluster randomisation, balance checks, sampling, survey designs with replicate weights, post-stratification, raking and calibration; in 1.16 and 1.17: designed experiments (factorial and fractional designs, response surfaces), repeated-measures ANOVA, equivalence tests; later: blocking, questionnaire pre-tests | `sample`, `seed` and `draw` for simulation-based design; power, designed experiments and `survey()` arrive in 1.17 |
| 2 | Descriptive: what is the state of the world? | data, stats, survey, series | Summaries, frequency and cross tables, intervals for means, proportions and medians, survey designs with weighted estimates and design-based SEs (Taylor, BRR, jackknife, bootstrap), design effects, moving averages, growth rates, index numbers | Summaries, tables, weighted statistics, distributions |
| 3 | Exploratory: what patterns exist? | plot, explore, stats, data | A layered plot grammar (Zeff Specification, section 7.6) with histograms, densities, box plots, scatter and scatter matrices, correlation heat maps, QQ plots, PCA, correspondence analysis, MDS, t-SNE, UMAP, LOESS, splines, missingness patterns, Little's MCAR test, outliers, explore/confirm holdout with exploratory results labelled | The plot grammar with its eight shorthands, result and coefficient plots; correlation |
| 4 | Measurement: can the construct be measured reliably and validly? | measure, latent | Item statistics, alpha, omega, polychoric correlations, EFA with retention tools, CFA, ESEM, bifactor and higher-order models, CR, AVE, Fornell–Larcker, HTMT, invariance (configural to strict, partial, alignment), IRT and DIF, G-theory, ICC, kappa, factor scores | Cronbach's alpha (`reliability`); omega arrives with CFA in 1.12 |
| 5 | Associational: do theorized relationships hold? | regress, glm, multilevel, latent, stats | OLS, high-dimensional fixed effects, robust and clustered SEs, wild bootstrap, GLMs, ordinal and multinomial models, tobit, Heckman, zero-inflated and hurdle models, quantile regression, multilevel and growth models (`re:`), random effects, dynamic panels, SEM, path analysis, latent growth, multi-group SEM, diagnostics, delta-method combinations, marginal effects, contrasts | OLS with `fe:`, logistic, probit, marginal effects (`margins`), linear-model diagnostics, `derive`, classical tests |
| 6 | Causal: what is the effect of X on Y? | identify, experiment, quasi, adjust, regress, glm | Causal graphs with adjustment sets and bad-control checks; randomisation inference, CUPED, Lee bounds, LATE; DiD (2×2, TWFE, Callaway–Sant'Anna, Sun–Abraham, de Chaisemartin–D'Haultfœuille, imputation, event studies, HonestDiD); RDD; IV (2SLS, LIML, Anderson–Rubin, shift-share); synthetic control and synthetic DiD; matching, IPW, AIPW, entropy balancing; double lasso, DML; sensitivity analysis | Causal graphs, IV with weak-instrument diagnostics and the Anderson–Rubin test, 2×2 DiD by regression, logistic and probit for binary outcomes |
| 6a | Mechanism: how or why does the effect occur? (under Causal) | identify, mediate, latent | Causal graphs separating mediators from confounders; product-of-coefficients and bootstrap mediation, causal mediation with sensitivity, serial, parallel and moderated mediation, SEM indirect effects, front-door, interventional effects | Causal graphs |
| 6b | Heterogeneity: for whom or when? (under Causal) | hetero, regress, stats | Interactions, simple slopes, Johnson–Neyman, subgroups with multiplicity correction, causal forests, meta-learners, BLP, GATES and CLAN, quantile treatment effects | Interactions, `p_adjust` |
| 7 | Predictive: what will happen for new cases? | learn, series, resample, results | Leakage-safe splits and recipes with a leakage checker (Zeff Specification, section 7.5), cross-validation and tuning, lasso, ridge, elastic net, trees, random forests, boosting, SVM, k-NN, naïve Bayes, neural networks, AUC to Brier score, calibration, forecasting (ARIMA, ETS, VAR, state space), SHAP, partial dependence | `predict` from `reg`, `ivreg`, `logistic` and `probit`; `roc` and `confusion` for binary models |
| 8 | Prescriptive: what should we do? | decide, optim | Policy trees, empirical welfare maximisation, dynamic treatment regimes, bandits, LP, MIP, nonlinear, stochastic and robust optimisation, newsvendor and pricing decisions | None |
| 9 | Hidden groupings: what hidden groupings exist? | groups, latent, series | k-means, k-medoids, hierarchical, DBSCAN, Gaussian mixtures, latent class, profile and transition analysis, growth mixtures, dynamic factor models, topic models, network communities | None |
| 10 | Counterfactual: what would happen under an unobserved policy? | structural, optim | User-defined MLE and GMM with automatic differentiation, simulated method of moments, indirect inference, conditional, nested and mixed logit, BLP demand, NFXP, CCP, entry models, counterfactual simulation, welfare | None |
| 11 | Evidence synthesis: what does the body of evidence say? | meta | Effect-size conversion, fixed- and random-effects, three-level and network meta-analysis, meta-regression, I², τ², prediction intervals, forest and funnel plots, Egger, trim-and-fill, PET-PEESE, p-curve, replication tests, specification curves | None |

Scale: 279 functions in Zeff 1.0 (186 of them infrastructure tagged only `general`), none still proposed, and 394 roadmap methods after it (673 in all). In 1.0 the menus show Design, Descriptive, Exploratory, Measurement, Associational, Causal with its two children, and Predictive; Prescriptive, Hidden groupings, Counterfactual and Evidence synthesis stay in the roadmap view until their first method ships. 1.0 is therefore presented as a language with regression, IV, binary outcomes and causal graphs, not as software for every research goal.

---

## 12. AI assistance (later version)

AI assistance has no release yet (section 15 names the open decisions that set one); when it comes it is under three rules that keep Zemp's principles intact. Whether Zemp also sells metered access to a model of its own, as the business model's Pro tier proposes, or only connects the user's own key or a local model, is open (section 17; zeff_open_decisions.md, #28). Zemp Lite does not offer it through 1.0; whether it does later is decided with #28 (#39).

1. **It writes Zeff, never results.** The assistant proposes Zeff statements; they appear in the editor or a dialog and run only when the user runs them, through the same checked path as any code.
2. **It explains, with sources in the output.** It can explain an error, a warning or a result by pointing to the printed numbers and the function's help.
3. **Data stays local by default.** Using a remote model sends only what the user approves (code, variable names, summaries), never raw data unless the user explicitly allows it.

Planned uses: "describe what you want" to a draft script, choosing a method from the research question, drafting a causal graph from the user's description for them to correct, explaining diagnostics, and drafting the methods paragraph of a paper from the run record.

---

## 13. Security and privacy

The rules of this section apply to both editions.

- Data stays on the user's machine; Zemp web stores projects in the browser's private storage and sends nothing to a server. The business model (zemp_business_model.md) proposes paid cloud sync, server compute and a self-hosted server; whether Zemp offers them is open (section 17; zeff_open_decisions.md, #27). If it does, they are opt-in, local-only stays the default, and this section will say what is sent, how it is encrypted and where it is held.
- Releases and updates are signed; the app verifies every part against its built-in key, the same signed key list the `zeff` launcher uses for engines; the offline bundle and an installable offline web app protect against a compromised server.
- Zeff scripts read only their own project folder and the read-only data roots it declares. They reach the network only through `fetch`, in a project that sets `network = true`; every response is cached, hashed and recorded, and API keys come from the user's keychain, never the project (Zeff Specification, section 8.6). The other exception is the opt-in `external` step below.
- The desktop-only `external` step (from 1.1) is off unless a project enables it, runs only the interpreter named in the user's own settings (never one named by the project, so a downloaded project cannot choose the program that runs), asks for confirmation on a project's first external step, and is flagged in every run record that uses it.
- Scripts write only under their own project folder; `save` and `export` to a data root or elsewhere are refused (Zeff Specification, section 2).
- From Zeff 2.0 at the earliest, third-party packages run as sandboxed WebAssembly components with no network or file access; they are declared in `zeff.toml`, locked in `zeff.lock`, and marked validated only when their reference tests pass in Zeff's own continuous integration (Zeff Specification, section 8.1, rule 9).
- No telemetry by default; crash reports are opt-in and shown before sending.
- **What is counted, and what is never collected** (zeff_open_decisions.md, #116). Counted, without any personal data or cookie: downloads by version and platform, from the release host and the package managers' own statistics; engine fetches by version and operating system, at the server the launcher downloads from, without keeping the machine's address (from Zeff 0.8.0); visits to the web editions, installs of their offline apps, countries, browsers and screen sizes, by a cookie-free aggregate counter on the web host (from the Zemp 0.2 alpha). Collected only from those who choose it: the optional **Register for wrong-result alerts** list (from 1.0), an email address with optional institution, role, field and country, used only to say when a known wrong-result bug affects the engine version given, left in one step, under a privacy notice saying where it is kept and for how long. Never collected: the user's data, scripts, results, file names, keystrokes or use of functions, and nothing is sent by the app itself. No login is needed to download or use Zeff, the `zeff` command line, Zemp Lite or Zemp Studio's free tier; an account exists only for paid features, if they are ever offered (#27). Each release from 0.10.0 carries a citation file and a DOI.
- The Git panel (section 5) sends nothing until the user presses Push or Pull, and then only to the remote the project names; `data/` is ignored by default and the panel warns before a data file is first committed.

---

## 14. Performance targets

| Target | Goal |
| --- | --- |
| Zemp Studio web first start | Interface and `core`/`data` packages ready within 3 seconds on a typical laptop and broadband connection; Zemp Lite web, with its smaller interface, within 2 seconds |
| Later starts | Under 1 second (cached, offline) |
| Import | A 1 GB CSV on the desktop in under 15 seconds |
| OLS with two-way fixed effects | 10 million rows on the desktop within 2× of fixest's time on one thread in 1.0, and within fixest's time with all cores from 1.1 |
| Memory | A chain of steps on a frame needs about the frame's size plus new columns, not a copy per step |
| Re-runs | An unchanged statement is reused from the cache, never recomputed |
| Interface responsiveness | No interface freeze during any run; long jobs report progress and can be cancelled |
| Plot redraw | A resize redraws within one frame (16 ms) for plots up to 100,000 points |

---

## 15. Release plan

Zemp has version numbers of its own (major.minor.patch, as Zeff's; Zeff Specification, section 14). Zemp Lite carries the same version numbers as Zemp, since Studio is built on Lite's code; a Lite release is the same release without the point-and-click modules. Each Zemp release names the Zeff engine versions it supports, and a Zemp fix never needs an engine release. After 1.0, by convention rather than by rule, Zemp's minor number follows the Zeff release it supports, so Zemp 1.3 runs the methods of Zeff 1.3. Zeff 1.0.0 does not wait for Zemp: the language's name freeze depends on the engine, the command line, the language server and the validation suite, not on the desktop app, installers or code signing.

Zemp Lite needs no dialogs, so it is usable earlier than Zemp Studio: its parts (the workspace, the editor with the language server, the viewers and the plot renderer) all exist by Zeff 0.7.0 and 0.8.0, while the dialogs that Zemp Studio needs are the slowest part to build. Zemp Lite is therefore the only edition in this plan (decided 12 November 2026, zeff_open_decisions.md, #125); Zemp Studio's work starts when a review at the close of Zeff 1.5 decides, possibly after Zeff 2.0.0, and its row at the foot of the table holds what it adds. The releases after 1.0 are those of the Zeff plan of 11 November 2026 (zeff_open_decisions.md, #113; Zeff Specification, section 14), without third-party packages before 2.0.0 (#123).

| Zemp | Zeff engine | What it adds |
| --- | --- | --- |
| — | 0.1.0 to 0.6.0 | Engine only, used from the `zeff` command line |
| 0.1 prototype | 0.7.0 | Framework chosen; workspace and plot viewer prototyped on the new plot renderer |
| 0.2 alpha | 0.8.0 | **Zemp Lite web preview**, the first public way to write Zeff in an IDE, at `lite.zeff-lang.dev`: the margin editor (#174) with the language server, the Data place with the data viewer (#75), the Plots place with the plot viewer, the Help panel (#74), Panes kept beside them (#176); "Report a problem"; a cookie-free count of visits (#116) |
| 0.2.1 | 0.8.0 | One layout of three panes on every place, each pane opened and closed by its own link (#203): Script with Project, Outline (#178, read from the comments) and Notes, the Objects panel removed and Run unfolding every output; Data with Frames of the run and a read-only Variables pane that shows the list or one variable's properties; Plots with its list and the plot's facts; Help with the page's outline and its related functions; Panes and the Margin link removed; the reader test of Zeff syntax runs on this alpha |
| 0.2.2 | 0.8.0 | A project saved to the computer, and the actions on what it holds (#208): **Export project…** as a zip, in every browser, and the opening of an exported zip as a new project; the request for persistent storage and the reminder to export (#117, the parts that need no run record); a project renamed, duplicated and deleted; **New script**, files added into any folder, and a file renamed, moved, duplicated, deleted and saved to the computer; a folder made, renamed, moved, deleted and saved as a zip |
| 0.2.3 | 0.8.0 | The Help place redesigned, its first stage (#211): **Home**, **Back** and **Forward**, with a trail saying where a page stands; Home a landing page of panels (#211, item 14): **Zeff** and **Zemp Lite** one after the other, each with its version and a few words, then a panel for each topic with its count and a sentence on what it holds, the methods and the language joining as panels when the engine brings them; the **Topics** pane, opened and closed by its link, with sections that fold for the packages (each opening the list of its functions, the shipped datasets under `datasets`), the errors and warnings by family, and **Using Zemp Lite** (thirteen pages on the app, in Zemp's own words); the search field of the Help place's bar suggesting from the second letter typed: names, one-line descriptions, packages, codes, and R and Stata names where the pages give them, the field at the right end of the bar; the right pane **On this page**, the open page's sections with the one in view marked, in place of **Related functions** (#211, item 13); a name with two functions, such as `histogram`, a page for each, titled by what it is called on, and the bare name a short chooser of the two (#215) |
| 0.2.4 | 0.8.0 | Zemp Lite's front door redrawn (#220): a landing page at the root of `lite.zeff-lang.dev` (the name large, **Open Zemp Lite** with the version beside it, bands for why, an example run on the carried engine, uses, the release's counts, and help and the source), its numbers and output taken from the carried engine at the build, and the app at `/app/` on the same origin; the loading screen with the three steps in one card under one bar, the size that travels, and the four promises in a band; the project screen without a heading, four cards in two rows (**New project…**, **Open project…**, which was **Open a project's zip…**; **Open folder…**, **Scratch project**), the five projects opened last in view with the lower edge fading, and the storage note in two columns; the name drawn as *Zemp* with a raised *Lite* badge wherever it is shown; and in Help (#220, items 6 to 9): hover in a script shows the usage block alone (F1 the whole page), **On this page** always shown as part of the page with no button or pane, each start of the app opening the Script place and the first click on Help showing its landing page with Topics closed, and Help's landing page in the look of these screens |
| 0.2.5 | 0.8.0 | The app back at the root of `lite.zeff-lang.dev` (#225): the landing page of 0.2.4 removed with its tests, an address with `/app/` in it going on to the same place at the root, and a page of Help at `/#/help/…`; then, as its second step, the engine sent packed tight at Brotli's highest setting (#224), about 2.9 MB in place of 4.1 MB, tried at the address first and left out if the host cannot send it so |
| 0.3 | 0.9.0 | The preview gains the release's methods through the engine (the binary models as `glm` and the ROC curve as `curve`, #112). Its own additions are two. The examples of the Help place run in the page (#211, item 11): a **Run** button under every example, the output, warnings, errors and plots drawn by the app's engine in a session of their own, each example running by itself when its page opens, and **Copy to script**. And **Paper** for `.typ` files, its first stage (#172, option A): the source beside its live page; a click on the page finds the source; an outline; tables and figures from scripts through `export` (`.typ` and SVG), refreshed by **Run**; the PDF set by Typst in the browser, Typst being a module of Zemp's own that is fetched the first time a paper is opened and is never inside the engine or the bundle a page loads at its start; and **Export script with output** as PDF |
| 0.4 | 0.10.0 | The preview gains the release's methods through the engine. **Paper**, its second stage: the settings file beside the paper (#124), `refs.bib` with citations and cross-references checked as one types, and the Article template |
| 0.5 desktop beta | 0.11.0 | **Zemp Lite desktop beta**, installed with `zeff app install lite`: native engine packaged with Tauri, part-by-part updater, `zeff app install` and the offline bundle; the read-only import preview for Excel, Stata and Parquet; Install command-line tool; **Open folder…**, on the desktop and in the browsers that allow it (#208); code signing decided by this release (Zeff Specification, section 15) |
| 1.0 | 1.0.0 | Zemp Lite 1.0, web and desktop, ships with Zeff 1.0.0 |
| 1.1 | 1.1 | Faster runs on all cores; `external` on the desktop; quick fixes, navigation, rename and inlay hints in the editor; the Git panel on the desktop; the Pipeline view, a script's steps as a graph with the stale ones marked, drawn from `zeff graph` (zeff_open_decisions.md, #121) |
| 1.2 to 1.29 | 1.2 to 1.29 | Each Zeff release's methods through the engine, Zemp's minor number following Zeff's; the report document with its Document settings dialog when #73 and #124 schedule it; AI assistance only as #27, #28 and #39 decide |
| Zemp Studio | At the review at the close of 1.5 (#125), possibly after 2.0.0 | The Methods panel, dialogs and guides for every method then shipped, each checked for identical results from dialog and code; the formula, graph, model, plot-layer and recipe builders; the import dialog and the controls that write a statement (section 2); the menu plan recorded with #125 |

Beyond the plan's releases, Zemp follows the future plan of the Zeff Specification (section 14.1): the browser's memory ceiling lifted by 64-bit WebAssembly, and, once Zemp Studio has started, dialogs and guides for each new method family.

---

## 16. Licensing

Zeff (language and engine) is Apache 2.0. Zemp Lite is developed in a public repository of its own and is Apache 2.0 as well (the owner's ruling of 15 November 2026, zeff_open_decisions.md, #148, which replaces the repository and licence parts of #125 and with them #25). Zemp Studio, the paid edition once its work starts, is developed in a separate private repository under a proprietary licence and builds on Lite's code; Lite never depends on Studio's modules (section 3). Every Zemp analysis is Zeff code run by the open engine, so replication never depends on Zemp's licence. All dependencies use permissive licences (MIT, Apache 2.0, BSD), whose notices ship with the apps; no GPL code is linked into the engine or the apps. This is general information, not legal advice.

---

## 17. Open decisions

- [ ] Start Zemp's interface at Zeff 0.7.0 rather than alongside the engine from the start
- [x] Where Zemp web is hosted (Cloudflare Pages at `lite.zeff-lang.dev`, 13 November 2026), sync through the user's own folder sync, and no Zemp server before 1.0 (21 November 2026); a server returns as a question with the paid tiers (zeff_open_decisions.md, #27)
- [ ] Which remote or local model powers AI assistance, and its data-sharing defaults; whether Zemp sells metered access to a model of its own (zeff_open_decisions.md, #28)
- [ ] Whether Zemp's minor versions keep following Zeff's after 1.0, or go their own way once the interface changes faster than the engine
- [ ] Performance targets above, once benchmarks exist; candidate stress cases are double lasso on a 2 GB file (1.18) and Callaway–Sant'Anna DiD on 4 million panel rows (1.19)
- [ ] Long-term-support releases for institutions, which the business model offers and the support policy of the Zeff Specification (section 14) does not cover (zeff_open_decisions.md, #34)
- [ ] Pricing levels and AI usage quotas per tier (zeff_open_decisions.md, #35)
- [ ] How students are verified for the free Pro tier (zeff_open_decisions.md, #36)
- [ ] When to file the Zeff and Zemp trademarks (zeff_open_decisions.md, #37)
- [ ] Setup files for the desktop apps and for Zeff (Windows `.exe`/`.msi`, macOS `.dmg`/`.pkg`, Linux `.deb`, `.rpm`, AppImage): deferred on 17 October 2026, when installation was kept to the command line; to be planned with the code-signing decision (Zeff Specification, section 15; zeff_open_decisions.md, #38), since unsigned setup files meet Windows SmartScreen and macOS Gatekeeper warnings

**Decided**

- [x] Zemp Lite only, Zemp Studio later, and Zemp's repository and licence (12 November 2026; sections 2, 6, 15 and 16): Zemp Lite built and released alone; Zemp Studio's start decided at a review at the close of Zeff 1.5, possibly after 2.0.0; Zemp in a private repository from its first commit, Lite free of charge and not open source at least until 1.0, Studio proprietary; the Document settings dialog of a report in Lite (zeff_open_decisions.md, #125, settling #25; its repository and licence parts replaced on 15 November 2026, the next entry)
- [x] Zemp's repositories and licences (15 November 2026; sections 2, 3, 15 and 16): Zemp Lite in a public repository under Apache 2.0, as Zeff; Zemp Studio, when its work starts, in a separate private repository under a proprietary licence, built on Lite, which never depends on Studio's modules (zeff_open_decisions.md, #148)
- [x] Zemp Lite's paid features, web address and desktop preview (11 November 2026; sections 2 and 10): free-tier only through 1.0, the rest with #27 and #28; Zemp Lite web at a path on Zemp Studio web's origin; no Lite desktop preview before the 0.5 desktop beta (zeff_open_decisions.md, #39)
- [x] Density, icons, desktop menus, tablet and phone (11 November 2026; section 5): one density with a Larger type setting, no icons in 1.0, native desktop menus mirroring the command palette, no tablet or phone layout in 1.0 (zeff_open_decisions.md, #33)
- [x] Interface framework (11 November 2026; section 3): Svelte 5 with TypeScript, bundled by Vite, without the spike against SolidJS once planned (zeff_open_decisions.md, #24); the look in section 5 (#115)
- [x] Zemp Studio (1 November 2026; section 2): the full edition's name, beside Zemp Lite; Zemp alone names the application and the product line; the installer identifiers follow (`zeff app install studio`, `zemp-studio`)
- [x] Zemp Lite (17 October 2026): a code-first edition for users who do not want point-and-click, on both the web and the desktop, built from the same codebase without the Methods panel, dialogs, guides, builders and statement-writing controls; same projects, engine store and version numbers as Zemp Studio; the desktop app installs from the command line with `zeff app install lite`, setup files later; free; a web preview with the 0.2 alpha, a desktop beta with the 0.5 desktop beta, and 1.0 with Zeff 1.0.0 (sections 1, 2, 5, 9, 10 and 15)
- [x] Zemp Lite's look (17 October 2026; sections 2 and 5): the Inline layout as Lite's default and Panes as Zemp Studio's, both available in both editions; a top bar of word controls (Projects ▾, project and file name, Git status, Dark switch, Check and Run) with everything else in ⌘K; a dark theme beside the light one in both editions, with exports always in the light print theme; a Git panel for any Git remote, on the desktop from 1.1 and in the browser once hosting is decided; a Terminal button and the Zeff console on the bottom line; stale outputs dimmed and labelled until re-run; a welcome screen; output folding and Export script with output planned before Lite 1.0. Mockups: the "Zemp Lite layouts" design canvas and zemp_lite_inline.pdf
