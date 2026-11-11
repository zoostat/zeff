# Zeff Diagrams: flowcharts and interactive SEM path diagrams

Version of this document: 11 November 2026 · Status: proposal; nothing here is normative until applied to the two volumes, zeff_functions.csv and the Zemp Specification. Companion to zeff_graphics_capabilities.md, whose design rules G1 to G10 and pre-renderer decisions F1 to F12 it follows. It covers three things: a general diagram system for flowcharts and conceptual models (sections 3 and 4), flowcharts Zeff draws by itself from what it already knows (section 5), and structural equation path diagrams, static and interactive (sections 6 to 8).

> **Status in this repository (11 November 2026).** A proposal of the owner, kept here for reference and not normative. Its decisions are recorded in `docs/spec/zeff_open_decisions.md`, entries #118 to #122, which place its items on the releases of decision #113; the release numbers in the text below are those of the roadmap before #113.

---

## 1. Summary

1. **One layout engine, four kinds of graph.** Causal graphs (`g.plot()`, 0.10.0), flowcharts, SEM path diagrams and pipeline graphs are all nodes and edges. One deterministic layout engine, written in Rust and built for `g.plot()` at 0.10.0, serves all of them. Graphviz and ELK, which R's DiagrammeR and many GUIs use, are EPL-licensed and cannot be linked under Zeff's permissive-only rule (section 1.4 of the Engine and Project Specification).
2. **A `diagram { }` block**, a value like `dag { }` and `model { }`, draws flowcharts and conceptual models in code: boxes, decisions, groups and arrows, including moderation arrows that point at another arrow. The word `diagram` must be reserved before the 1.0 freeze, or the block can never be added without breaking scripts.
3. **Flowcharts Zeff draws by itself are its distinctive feature.** Zeff already counts every dropped row (principle 6), so `m.sample_flow()` draws the path from the raw file to the estimation sample, with an exclusion box at every step and the counts filled in. CONSORT diagrams come from `randomize` (1.9), PRISMA diagrams from a screening frame (`meta`), and `zeff graph` draws the script's own dependency graph. No mainstream package draws a sample flow from the analysis code itself.
4. **SEM path diagrams follow the RAM conventions.** Latent variables are ellipses, observed ones rectangles, means triangles; single-headed arrows are paths and loadings, curved double-headed arrows covariances. The diagram is drawn from the `model { }` block before fitting (`m.plot()`) and with estimates after (`fit.path_plot()`), with layout templates for CFA, second-order, bifactor, MIMIC, mediation, cross-lagged and growth models.
5. **Interactive in two ways.** In both editions the viewer explores a fitted diagram: hover for estimates, click a path to trace every effect through it, switch standardised and raw values, overlay modification indices and step through groups. In Zemp only, the model builder draws the model, writes the `model { }` block, checks identification live and keeps drawing and text in sync, as Stata's SEM Builder and AMOS do, but with code first.
6. **Positions belong to the script when a figure is exported.** Dragging nodes is view state. **Add to script** writes the positions as a `layout:` grid, and exports use only positions the script holds, so a replication package redraws every published diagram exactly. This also settles a gap in the current text for `g.plot()`.

---

## 2. What R and other tools offer

| Need | Tools | What they do well | What Zeff takes from them |
| --- | --- | --- | --- |
| General flowcharts | DiagrammeR (Graphviz DOT and mermaid), diagram, ggflowchart, Graphviz, mermaid | Text-defined graphs with automatic layout | A text block, automatic layered layout, shapes and groups |
| Study flowcharts | consort, flowchart (`fc_filter`), PRISMA2020, ggconsort | CONSORT and PRISMA diagrams from data frames | Flow from counts; Zeff gets the counts from its own lineage |
| Pipeline graphs | targets `tar_visnetwork`, drake | The dependency graph of an analysis, outdated steps marked | `zeff graph` from the typed IR, with stale steps marked |
| Static SEM diagrams | semPlot `semPaths`, tidySEM `graph_sem`, lavaanPlot, semptools | Diagrams from fitted lavaan models; layout matrices (tidySEM); curvature and position fixes after the fact (semptools) | A layout grid in code, RAM shapes, estimate labels, post-hoc adjustment by arguments |
| Interactive SEM | Stata SEM Builder, AMOS, Ωnyx, lavaangui, JASP and jamovi (SEMLj), Mplus Diagrammer, LISREL | Draw the model, estimate, see estimates on the drawing | The builder, round trip with text, estimates on the same canvas |
| Conceptual and statistical diagrams | Hayes's PROCESS model templates, hand drawing | Conceptual model and statistical model as separate figures | Both views from one model, by `kind: conceptual \| statistical` |

---

## 3. The diagram system

### 3.1 Architecture

| Part | Rule |
| --- | --- |
| Value types | `Diagram` joins `Dag` and `Model` as a specification type declared in `core` (rule 4 of section 8.1 of the Engine and Project Specification). A `Dag`, a `Model`, a fitted `SemFit` and the results of `sample_flow` are all drawn through the same layout engine |
| Drawing | Every diagram becomes a `Plot` (G1): `d.plot()`, `g.plot()`, `m.plot()`, `fit.path_plot()`. It is exported, styled with `.theme()`, combined with `arrange` and placed in the viewer like any plot |
| Scene | Diagrams use the primitives of F1, which must therefore include **cubic Bézier and arc segments** in paths (curved covariances, self-loops, routed edges) and **text boxes with wrapping and padding** (node labels). Both are part of the amendment to F1 below |
| Layout engine | Zeff's own, in Rust, in the `plot` package, first used by `g.plot()` at 0.10.0. Layered (Sugiyama) layout: cycle removal, layer assignment by network simplex, crossing reduction by the median heuristic with a fixed number of sweeps, coordinates by Brandes–Köpf, and edges routed as splines or orthogonal lines. Grid placement for templates and `layout:` grids. A force layout with a fixed iteration count for networks later |
| Determinism | Ties are broken by declaration order, sweeps are fixed in number, and no step draws a random number unless `seed:` is given. The same diagram gives the same bytes on every machine (G3), checked by snapshot tests |
| Licences | Graphviz (EPL-1.0) and ELK (EPL-2.0) are not linked. dagre (MIT) is JavaScript and not used by the engine. A Rust crate such as layout-rs may serve as a starting point if its licence checks out; otherwise the algorithms are implemented from their papers |
| Tiers | The `diagram { }` block and `Diagram` type are in `core`; layout and drawing in `plot` (tier 1); `sample_flow` in `plot`, which may read `data` and `results`; `m.plot()` and `path_plot` in `latent`; mediation diagrams in `mediate`; CONSORT in `experiment`; PRISMA in `meta`; `g.plot()` stays in `identify`. Each higher package calls `plot`'s engine downward |

**Amendment to F1** (zeff_graphics_capabilities.md, section 4): the path primitive carries line, cubic Bézier and elliptical-arc segments; the text-run primitive carries a box (padding, corner radius, fill, border) and a wrap width; and every primitive may carry a stable element name (`node:engage`, `edge:engage->perform`), so an exported SVG has named groups that Inkscape or Illustrator users can adjust. Element names identify shapes, never data rows (F2 is unchanged).

### 3.2 Positions: script or view state

The current text makes node positions of `g.plot()` view state, kept in `.zemp/views.json`. A replication package leaves that file out (Zemp Specification, section 5), so an exported causal graph could look different when a referee re-runs it. The proposed rule, for every diagram:

| Rule | Detail |
| --- | --- |
| Automatic layout is the default | Deterministic, so a diagram without positions redraws identically everywhere |
| Dragging is view state | In the viewer, nodes can be dragged, snapped to a grid and aligned with guides; nothing in the script changes |
| **Add to script** writes positions | The viewer writes a `layout:` grid (section 3.3) or `at:` positions into the call or the block, shown first as every Add to script is |
| Exports use script positions only | `export` uses automatic layout or positions written in the script, never view state; when dragged positions differ from what an export would draw, the viewer says so beside the Export button |

### 3.3 Placing nodes in code

Three ways, from least to most control. All are optional.

```
d.plot(direction: right)                       // automatic layered layout, left to right

d.plot(layout: [                               // a grid: each name in its row and column, NA for empty
    [training, NA,     perform],
    [NA,       engage, NA],
])

d.plot(at: {training: [0, 0], engage: [1, 1], perform: [2, 0]})   // exact positions, in grid units
```

| Option | Meaning | R source |
| --- | --- | --- |
| `direction: down \| right \| up \| left` | Direction of the layered layout | DOT `rankdir`, mermaid `TD`/`LR` |
| `layout: List<List<Var?>>` | A grid of names with `NA` for empty cells; unlisted nodes are placed by the engine around those listed | tidySEM `get_layout`, semPlot `layout` matrices |
| `at: Record` | Grid coordinates per node, fractional allowed | semptools `set_sem_layout` |
| `same_rank: List<Vars>` | Nodes kept on one layer | DOT `rank = same` |
| `spacing: [Float, Float]` | Distance between layers and between nodes, in millimetres | DOT `ranksep`, `nodesep` |

---

## 4. The `diagram { }` block

### 4.1 Example: a research model with hypotheses

Management and organisational behaviour papers almost always open with a figure like this one, with the hypotheses on the arrows and a moderator pointing at a path.

```
let framework = diagram {
    direction: right
    training {label: ["Training", "intensity"]}
    engage   {label: "Work engagement"}
    perform  {label: "Job performance"}
    support  {label: "Supervisor support"}
    group hypothesised {label: "Mediation"} [training, engage, perform]

    training -> engage   {label: "H1 (+)"}
    engage   -> perform  {label: "H2 (+)"}
    training -> perform  {label: "H3", style: dashed}
    support  -> (training -> engage) {label: "H4"}      // moderation: an arrow onto an arrow
}
framework.plot().export("output/fig1.pdf", size: one_column)
```

### 4.2 Example: a process flowchart with data-driven labels

```
let n_raw  = raw.n_rows()
let n_kept = clean.n_rows()
let process = diagram {
    direction: down
    start    {shape: terminal, label: "Survey closes"}
    clean    {shape: process,  label: ["Clean responses", n_raw, "rows"]}
    check    {shape: decision, label: "Attention check passed?"}
    analyse  {shape: process,  label: ["Analyse", n_kept, "rows"]}
    drop     {shape: process,  label: "Exclude"}
    start -> clean -> check
    check -> analyse {label: "yes"}
    check -> drop    {label: "no"}
}
```

A `label:` is a `Str` or a list of lines; numbers in it are formatted as tables format them (`18,184`), so counts need no `format` call. Labels are expressions evaluated in the script's scope, which is the one difference from `dag { }`, whose lines hold names only.

### 4.3 Grammar and meaning

```
diagram_block = "diagram" "{" { NEWLINE } { diagram_line NEWLINE { NEWLINE } } "}"
diagram_line  = setting | node_line | edge_line | group_line
setting       = NAME ":" expr                              // direction, spacing, …
node_line     = NAME record                                // a node and its attributes
edge_line     = target { ( "->" | "--" | "<->" ) target } [ record ]
target        = NAME | "[" NAME { "," NAME } "]" | "(" NAME ( "->" | "--" | "<->" ) NAME ")"
group_line    = "group" NAME [ record ] "[" NAME { "," NAME } "]"
```

| Element | Meaning |
| --- | --- |
| `a -> b`, `a -> b -> c` | Arrow, chains allowed |
| `a -- b`, `a <-> b` | Line without heads; arrow with two heads |
| `[a, b] -> c` | Several sources at once, as in `dag { }` |
| `m -> (a -> b)` | An arrow that ends on the middle of another arrow: moderation in a conceptual model |
| `name {attributes}` | Declares a node; a node used in an edge without a declaration is a box labelled with its name |
| `group name {label:} [a, b]` | A labelled frame around nodes, laid out together (Graphviz clusters, mermaid subgraphs) |
| Setting `direction: right` | Layout direction for the whole block; a setting name is never a node name |

**Node attributes:** `label:` (Str or a list of lines), `shape:`, `style: solid | dashed | bold`, `fill:` (a palette role: `none | muted | accent | warning`, never a raw colour, so themes and print stay consistent), `width:` (wrap width in millimetres), `note:` (a smaller second text).

**Shapes** follow ISO 5807 flowchart symbols: `process` (box, the default), `terminal` (rounded), `decision` (diamond), `data` (parallelogram), `document`, `database` (cylinder), `subprocess` (box with double sides), `connector` (small circle), `note` (box with a folded corner). SEM shapes (section 6.1) are available too, so a hand-drawn measurement model can use them: `latent` (ellipse), `observed` (rectangle), `constant` (triangle), `error` (small circle).

**Edge attributes:** `label:`, `style: solid | dashed | dotted | bold`, `heads: end | both | none` (overrides the operator), `curve: Float` (bend), `route: spline | orthogonal | straight`.

**Checks.** A name used as both a node and a setting, an edge to an undeclared group, or a moderation arrow onto a pair that has no edge is an error at check time; cycles are allowed (flowcharts loop), unlike in `dag { }`.

**Printing and export.** A `Diagram` prints as its block text, one line per node and edge, as a `Dag` does. `d.export("fig.mmd")` and `d.export("fig.dot")` write Mermaid and DOT text for documents and GitHub, with layout left to those tools. Reading DOT or Mermaid back is not proposed.

**Reserved word.** `diagram` is not a reserved word today. Because `if x { … }` makes a bare name followed by `{` legal elsewhere, the block cannot be recognised contextually. `diagram` must join the words reserved for later use (section 3.1 of the Language Reference) before 1.0 freezes them, so a script naming a column `diagram` is warned now rather than broken later. `group` inside the block is contextual, as `latent` is inside `dag { }`.

---

## 5. Flowcharts Zeff draws by itself

### 5.1 Sample flow: from raw file to estimation sample

```
let wages = import("data/wages.csv")
    .where(year >= 2010, reason: "survey years before 2010")
    .drop_missing([wage, educ], reason: "wage or schooling missing")
    .merge(firms, on: firm_id, join: inner, relation: many_to_one, reason: "firm not in register")
let m = wages.reg(lwage ~ educ + exper + female, fe: firm + year, cluster: firm)
m.sample_flow()
```

```
sample_flow · m
  25,310   rows in data/wages.csv
   −3,102  survey years before 2010                      where(year >= 2010)
   −1,880  wage or schooling missing                     drop_missing([wage, educ])
   −1,916  firm not in register                          merge(firms, join: inner)
  18,412   rows in wages
     −216  NA in a model variable (exper 194, female 22) reg
      −12  singletons in fe: firm                        reg
  18,184   rows in the estimation sample of m
```

The plot draws the same flow vertically, with each exclusion as a side box (the CONSORT layout).

| Rule | Detail |
| --- | --- |
| Source of the counts | The lineage the typed IR already keeps (section 7.5 of the Language Reference uses it for leakage) and the counts every verb already reports (principle 6). Nothing is recomputed |
| Steps shown | Every step that changes the row count: `where`, `drop_missing`, `distinct`, `sample`, inner and left `merge` (unmatched rows), `append` (added rows), `aggregate` (rows become groups, shown as a change of unit), the model's `NA` and singleton drops, and `holdout` (training and testing branches) |
| Steps not shown | Steps that keep every row (`add`, `select`, `sort`); a frame merged in from elsewhere is shown as a side input with its own count |
| Reasons | `reason: Str`, a proposed option on the row-dropping verbs, names the box; without it the box shows the condition as written (`not year >= 2010`) |
| Branches | Several models from one frame, or a `holdout` split, draw as branches from the shared step |
| On what | `frame.sample_flow()` up to that frame; `fit.sample_flow()` up to the estimation sample; `report()` can add the final count of each model to its table note |
| Text form | Printed as above, so a log or a methods section can quote it |

### 5.2 CONSORT, PRISMA and pipeline diagrams

| Diagram | Zeff | Draws | Owner | R source | When |
| --- | --- | --- | --- | --- | --- |
| Sample flow | `frame.sample_flow()`, `fit.sample_flow()` | Raw file to analysis sample, exclusions with reasons | `plot` | consort, flowchart | 1.2 |
| CONSORT | `trial.consort_plot(assessed:, followed:)` | Enrolment, allocation by arm, follow-up and analysis counts, from `randomize` and the attrition variables | `experiment` | consort `consort_plot`, ggconsort | 1.9 |
| PRISMA | `screening.prisma_plot(stage:, reason:)` | Records identified, screened, excluded with reasons and included, from a frame with one row per record | `meta` | PRISMA2020 | With `meta` |
| Analysis pipeline | `zeff graph [script]`; Zemp's Pipeline view | Imports, frames, models, tables and figures as a graph from the typed IR; stale steps marked; click to jump to the line | Engine and Zemp | targets `tar_visnetwork` | 1.1 |
| Recipe and validation | `rec.plot()`, `folds.plot()` | Preprocessing steps in order; folds and splits as blocks of rows | `learn`, `resample` | none built in | 1.5 |
| Mediation and moderation | `med.path_plot(kind: conceptual \| statistical)` | Hayes-style conceptual figure and the statistical figure with paths a, b, c′ and their estimates | `mediate` | PROCESS templates | With `mediate` |

`zeff graph` is a command, not a function, because a function sees only its arguments (section 5.3 of the Language Reference) and a script's own graph is not a value inside it.

---

## 6. SEM path diagrams

### 6.1 Notation

The reticular action model (RAM) conventions, as McArdle and McDonald set them out and as semPlot, AMOS and Ωnyx draw them:

| Element | Drawn as | Zeff source |
| --- | --- | --- |
| Latent variable | Ellipse | Left side of `=~` |
| Observed variable | Rectangle, labelled with the variable's label when it has one | A column |
| Mean or intercept | Triangle (the constant 1), with arrows to the variables whose means are modelled | Mean structure, multi-group and growth models |
| Regression or loading | Single-headed straight arrow | `~`, `=~` |
| Covariance | Double-headed curved arrow, drawn outside the main flow | `~~` between two variables |
| Variance | Double-headed self-loop, or the value printed in a small error circle | `~~` of a variable with itself |
| Residual or disturbance | Small circle with an arrow into the variable, or a short arrow alone (`residuals: circle \| arrow \| none`) | Every endogenous variable |
| Fixed parameter | Its value on the arrow (`1`), arrow drawn thinner | Marker loadings, fixed values |
| Labelled parameter | The label beside the estimate (`a = 0.42`) | `as a` |
| Equality constraint | The shared label on each constrained arrow | The same label used twice |
| Defined parameter | Listed in a box below the diagram; `highlight:` colours its paths | `:=` |
| Latent interaction | A small filled circle joining the two parents, with one arrow to the outcome | Latent interaction terms, when they arrive |

### 6.2 Two diagrams from one model

```
let m = model {
    ind60 =~ x1 + x2 + x3
    dem60 =~ y1 + y2 + y3 + y4
    dem65 =~ y5 + y6 + y7 + y8
    dem60 ~ ind60 as a
    dem65 ~ ind60 as c + dem60 as b
    y1 ~~ y5
    indirect := a * b
}
m.plot()                                     // specification: free parameters as *, labels shown

let fit = pd.sem(m, se: bootstrap, reps: 2000, seed: 42)
fit.path_plot(estimates: std, show: [estimate, stars], fit_box: true, highlight: indirect)
fit.plot()                                   // the same diagram with its defaults
```

`m.plot()` is a method on `Model` and `fit.path_plot()` a method on `SemFit` and `CfaFit`, both in `latent`. `fit.plot()`, the generic default for every result, calls `path_plot` with its defaults, which is why the options live on `path_plot`: rule 6 of section 8.1 of the Engine and Project Specification allows one definition of `plot` for every `Result` and no second one for a subtype. (This corrects the row `sem.plot(estimates:)` in zeff_graphics_capabilities.md, section 7.4.)

### 6.3 What `path_plot` can show

| Option | Values | Effect | R source |
| --- | --- | --- | --- |
| `estimates:` | `std \| std_lv \| raw \| none` | Fully standardised, latent-standardised or raw estimates on the arrows | semPlot `whatLabels`, lavaan `standardizedSolution` |
| `show:` | List of `estimate`, `se`, `ci`, `p`, `stars`, `label` | What each arrow's label carries, in that order (`0.42 [0.31, 0.53]`) | semPlot, tidySEM `edge_label` |
| `digits:` | Int, default 2 | Decimals on arrows | semPlot `nDigits` |
| `width:` | `fixed \| estimate` | Arrow width proportional to the absolute standardised estimate | semPlot `fade`, `esize` |
| `sign:` | `none \| color \| style` | Negative estimates in a second colour or dashed | semPlot `posCol`, `negCol` |
| `nonsignificant:` | `show \| dashed \| grey \| hide` at `level:` | How arrows whose interval includes zero are drawn | tidySEM `sig` |
| `threshold:` | Float | Hide loadings below a value; ESEM cross-loadings shown faintly above it | semPlot `cut` |
| `residuals:` | `circle \| arrow \| none` | Residual notation | semPlot `residuals`, `style` |
| `means:` | Bool | Draw the mean structure with triangles | semPlot `intercepts` |
| `r_squared:` | Bool | R² inside each endogenous variable | lavaanPlot |
| `fit_box:` | Bool, or a list of measures | χ²(df), CFI, TLI, RMSEA with its interval, SRMR in a caption box | tidySEM, by hand |
| `collapse:` | `none \| indicators` | Hide indicators to show the structural model alone, as a conceptual figure | semPlot `structural` |
| `highlight:` | A defined parameter, a list of variables, or a path `x -> m -> y` | Colours the paths that make up an indirect effect or a chosen route | semptools |
| `group:` | A group level, `all` or `compare: [g1, g2]` | One panel per group, or both groups' estimates on each arrow (`0.42 / 0.37`) with differences marked | semPlot multi-group panels |
| `mi:` | Float | Draws the paths and covariances whose modification index exceeds the value, dashed grey with the index; for diagnosis, never the default | LISREL and AMOS modification-index displays |
| `layout:`, `at:`, `direction:` | As section 3.3 | Placement | tidySEM, semptools |
| `template:` | Section 6.4 | A layout family | semPlot `layout = "tree2"` and others |

### 6.4 Layout templates

The automatic layout places exogenous variables before endogenous ones in the layout direction, each factor's indicators on its outer side as a fan, and covariances on the side away from the structural paths. Templates encode the conventional figure of each model family:

| Template | Layout | Used for |
| --- | --- | --- |
| `auto` | Layered: exogenous to endogenous, indicators fanned outward, crossings reduced | Any model; the default |
| `cfa` | Factors in a row, indicators below each, factor covariances as arcs above | Measurement models |
| `second_order` | Higher-order factor above, first-order factors in the middle, indicators below | Second-order models |
| `bifactor` | General factor on one side, specific factors on the other, indicators between | Bifactor models |
| `mimic` | Causes on the left, the factor in the middle, indicators on the right | MIMIC models |
| `mediation` | The triangle: X bottom left, M on top, Y bottom right, c′ along the base | Simple and parallel mediation; serial mediation as a chain |
| `cross_lagged` | Columns for waves, rows for constructs, autoregressive arrows across, cross-lagged arrows diagonal, within-wave covariances vertical | CLPM and random-intercept CLPM (the random intercepts above and below) |
| `growth` | Intercept and slope factors above the repeated measures, with loadings 1, 1, 1 and 0, 1, 2 shown | Latent growth models (with `latent_growth`) |
| `multigroup` | One panel per group with identical positions | Multi-group and invariance models |

Templates accept `layout:` and `at:` overrides for single nodes, so a user moves one variable without losing the rest.

### 6.5 Label placement and edge routing

| Problem | Rule |
| --- | --- |
| Labels on crowded arrows | Each label sits at a fixed fraction along its arrow on an opaque box, then moves by deterministic repulsion only if it collides; it never leaves its arrow's side |
| Many indicators | Indicators wrap into two rows when a factor has more than eight, keeping the fan readable |
| Covariance arcs | Drawn outside the convex hull of the structural paths; nested arcs are stacked by span, shortest innermost |
| Crossing arrows | Crossing reduction runs on the structural layer first, then indicators are ordered to follow it |
| Size | The diagram is laid out in millimetres at the export size, so labels stay at 8 pt or more (G6); when they would not fit, the viewer suggests `collapse: indicators` or a larger size |

---

## 7. Interaction in the viewer (Zemp and Zemp Lite)

Everything here is view state (G7) and works in both editions, because exploring a result writes no statement.

| Action | Shows | Source of the numbers |
| --- | --- | --- |
| Hover a variable | Label, type, N, mean and SD for observed variables; R², residual variance; for a factor, its reliability (omega, CR, AVE when those functions have shipped) | Data and result interfaces; `measure` |
| Hover an arrow | Raw and standardised estimate, SE, z, p, interval, parameter label, constraints that bind it, and its row in `fit.summary()` | `SemFit` estimates |
| Click an arrow | Every direct, indirect and total effect that passes through it, with the paths drawn in colour | `sem.effects()` |
| Click a factor | Its measurement model alone, with loadings, residuals and reliability | `sem.loadings()`, `measure` |
| Toggles | Standardised and raw; residuals; means; R²; non-significant arrows; modification-index overlay at an adjustable cutoff; loadings threshold slider | `path_plot` options, applied as view state |
| Group switcher | Steps through groups with the same positions, or shows two groups side by side with differences marked | Multi-group fits |
| Compare models | Two fits of nested models side by side, added and removed parameters marked, χ² difference in the caption | `fit.compare(other)` |
| Drag and align | Nodes snap to a grid with alignment guides; arrows reroute | Layout engine |
| **Add to script** | Writes the current options and positions into the `path_plot` call (section 3.2) | Script |
| Search | Find a variable in a large model | Viewer |
| Table view | The diagram as an edge list with estimates, for screen readers and copying | `p.computed()` |

---

## 8. The model builder (Zemp)

Zemp's specification already promises a model-block builder from 1.3: draw factors and paths, or type the block, with both kept in sync (Zemp Specification, section 6). This section specifies it. Zemp Lite leaves the builder out, as it leaves out every builder, but keeps section 7.

### 8.1 Drawing

| Action | Writes in the `model { }` block |
| --- | --- |
| Drag a variable from the frame's variable list onto the canvas | An observed variable (nothing yet; a variable appears in the block once it has an arrow) |
| Add a latent variable and name it | Nothing until it has indicators |
| Draw an arrow from a latent variable to observed ones | `f =~ x1 + x2 + x3` |
| Draw an arrow between two variables | `y ~ x` |
| Draw a two-headed arrow | `a ~~ b` |
| Double-click an arrow and type a label | `y ~ x as a` |
| Give two arrows the same label | An equality constraint |
| Fix an arrow's value | The fixed-parameter syntax, once open decision "fixing a parameter in a model block" (section 15 of the Engine and Project Specification) is settled; the builder needs it by 1.3 |
| Select a chain of arrows and choose **Define indirect effect** | `indirect := a * b`, labelling the arrows if they have no labels |
| Choose a template (CFA, second-order, bifactor, MIMIC, mediation, Hayes model 1, 4, 7, 14, CLPM, RI-CLPM, growth, multi-group) | The template's block with placeholder names, laid out by its template |
| Drag several items onto a latent variable at once | One `=~` line with all of them |

### 8.2 Rules

| Rule | Detail |
| --- | --- |
| Code first, always visible | The block sits beside the canvas and updates with every edit; typing in the block redraws the canvas; the block is what runs (Zemp principle 2) |
| Round trip without loss | Comments, line order and labels in a typed block survive edits made on the canvas; positions are kept as view state until **Add to script** writes them into `path_plot(layout:)` |
| Checked as you draw | The checker runs on every edit: names against the frame, operators, and the model's identification: free parameters against the moments available (the t-rule), a scale for every latent variable, the three-indicator rule for single factors, and the degrees of freedom shown live (`df = 41 · identified`). These become new checks in the Z05xx and W05xx families |
| Estimates on the same canvas | **Fit** runs `sem` or `cfa` through the dialog's statement; the canvas switches to the fitted diagram of section 7 with the same positions; editing returns it to the specification view, with stale estimates dimmed as stale outputs are (Zemp Specification, section 5) |
| Undo and history | Every canvas edit is undoable, and the block's history is the editor's history |
| Large models | Collapsing a factor hides its indicators on the canvas without changing the block |
| Keyboard | Every drawing action has a keyboard path: select nodes with arrow keys, `L` to draw a loading, `R` a regression, `C` a covariance |

### 8.3 Beyond SEM

The same canvas, with the flowchart shapes of section 4.3, builds `diagram { }` blocks: a **Diagram builder** in Zemp that writes the block as the model builder writes `model { }`. It replaces the causal-graph builder's private canvas, which becomes the same component with `dag { }` rules (no cycles, latent and selected nodes).

---

## 9. Releases

| Release | What arrives |
| --- | --- |
| Before 0.7.0 | The F1 amendment (curves, text boxes, element names); the positions rule of section 3.2; `diagram` reserved and `Diagram` declared in `core` |
| 0.10.0 | The layout engine, first for `g.plot()`, with `direction:`, `layout:` and `at:`; positions written by **Add to script**; the causal-graph builder on the shared canvas (Zemp 0.4) |
| 1.1 | `zeff graph` and Zemp's Pipeline view |
| 1.2 | `diagram { }` with the flowchart shapes; Mermaid and DOT export; `sample_flow` with the `reason:` option; the Diagram builder in Zemp |
| 1.3 | `m.plot()`, `path_plot` with every option of section 6.3 except ESEM thresholds; templates `auto`, `cfa`, `second_order`, `bifactor`, `mimic`, `mediation`, `cross_lagged`, `multigroup`; the viewer interactions of section 7; the model builder of section 8 with identification checks |
| 1.5 | `rec.plot()`, `folds.plot()` |
| 1.6 | ESEM display: thresholds and faint cross-loadings |
| 1.9 | `consort_plot` |
| With their methods | `growth` template with `latent_growth`; reliability on hover with `composite_reliability` and `ave`; mediation diagrams with `mediate`; PRISMA with `meta`; latent interactions when they arrive |

---

## 10. Decisions to add to the register

| # | Decision | Recommendation | Deadline |
| --- | --- | --- | --- |
| D1 | Reserve `diagram` among the words reserved for later use, and declare `Diagram` in `core` | Adopt; it is the only way the block can arrive after 1.0 without breaking a script | 1.0 freeze |
| D2 | Positions of diagrams, `g.plot()` included: view state for dragging, script for export | Adopt section 3.2, amending the current text of the Language Reference (section 7.3) and the Zemp Specification (sections 6 and 7) | 0.10.0 |
| D3 | F1 amended with Bézier and arc segments, text boxes and element names | Adopt | 0.7.0 |
| D4 | Zeff's own layout engine, with no Graphviz or ELK | Adopt; check layout-rs's licence first | 0.10.0 |
| D5 | `reason:` on `where`, `drop_missing`, `distinct` and `merge` | Adopt; additive, but its name should be settled before `where` ships so that no project function takes it first | 0.2.0 |
| D6 | SEM diagrams as `m.plot()` on `Model` and `fit.path_plot()` on SEM results | Adopt; corrects `sem.plot()` in the graphics proposal | 1.3 |
| D7 | Fixed-parameter syntax in model blocks | Settle the open decision; the builder cannot fix a value without it | 1.3 |
| D8 | Identification checks at check time | Adopt, as new Z05xx errors (no scale for a latent variable) and W05xx warnings (negative degrees of freedom, a factor with fewer than three indicators and no other identifying restriction) | 1.3 |
| D9 | `zeff graph` and the Pipeline view | Adopt | 1.1 |
| D10 | One canvas component for causal graphs, diagrams and SEM in Zemp | Adopt | Zemp 0.4 |
