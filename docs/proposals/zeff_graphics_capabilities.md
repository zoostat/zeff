# Zeff Graphics Capabilities: what R's plotting ecosystem teaches, and what Zeff should draw

Version of this document: 11 November 2026 · Status: proposal. Nothing here is normative until it is applied to the two volumes (zeff_language_reference.md, zeff_engine_specification.md) and to zeff_functions.csv. It builds on section 7.6 of the Language Reference (the layered grammar), section 8.2 of the Engine and Project Specification (the `plot` package: 35 functions in 1.0, 6 on the roadmap) and section 7 of the Zemp Specification (the plot viewer).

> **Status in this repository (11 November 2026).** A proposal of the owner, kept here for reference and not normative. Its decisions are recorded in `docs/spec/zeff_open_decisions.md`, entries #118 to #122, which place its items on the releases of decision #113; the release numbers in the text below are those of the roadmap before #113.

R has about 150 general-purpose plotting packages: base graphics, grid, lattice, ggplot2 and the 167 extensions registered in its gallery, the htmlwidgets libraries, and the 3D, map and network systems. This document breaks what they do into separate capabilities. It sorts out which of them Zeff already specifies, which it should add and in which release, and which it should leave out on purpose. It also lists the decisions that must be made before the plot renderer is built at 0.7.0, because changing them later would change figures that existing scripts draw.

---

## 1. Summary

1. **R spreads graphics over nine layers and two drawing systems; Zeff needs one system with nine layers.** R's split between base graphics and grid is why `plot.lm()` cannot be restyled and why ggfortify exists. In Zeff every plotting function, the shorthands and the result plots included, returns the same editable `Plot` value built from the grammar, so `.labels()`, `.theme()` and further layers work on everything.
2. **The 1.0 grammar is a sound core.** It has 11 marks, 4 statistical layers, scales, coordinates, facets, guides, themes, a versioned scene and a renderer with embedded fonts. That covers ggplot2's own geoms for everyday work. The gap is in three places: the specialised statistical layers that ggdist, ggridges, ggbeeswarm, ggforce and qqplotr add; composition and annotation (patchwork, ggrepel, ggtext); and, above all, result plots.
3. **Result plots are the largest gap and Zeff's biggest advantage.** Half of R's plotting packages exist to plot one kind of fitted object: GGally, ggfortify, sjPlot, see, factoextra, bayesplot, survminer, cobalt, metafor, specr and more. Zeff's result interface lets each model package add `_plot` methods for its own result types, drawn by the one grammar. Section 7 lists about 95 of them, each placed with its method's release.
4. **Twelve decisions belong before 0.7.0** (section 4): the scene's primitives, row references for the viewer, reserved dimensions for animation, 3D and maps, scale limits as zoom rather than filter, a reserved `RichText` type, where notes go on exported figures, tick and number formatting, and interval labelling. Each is cheap now and breaking later.
5. **Interactivity is view state, not a second plot system.** plotly, ggiraph and crosstalk turn a figure into a JavaScript widget. In Zeff, hover, brushing, linked selection and zoom live in Zemp's viewer, never change the script, and offer **Add to script**. A self-contained interactive HTML export can come later from the same scene.
6. **Determinism reaches the layout algorithms.** Label repulsion, beeswarms, jitter, network layouts and large-data sampling run a fixed number of iterations on the seeded stream, so a figure's bytes are identical on every machine, as its numbers are.
7. **The tier rules decide who owns each plot.** `plot` (tier 1) may call `data`, `dist`, `results` and `stats` but nothing in tier 2. So binned scatter plots live in `regress`, dendrograms in `groups`, path diagrams in `latent`, and quantile-regression lines are drawn through `.smooth(fit: m)`.
8. **Diagrams have a companion document.** Flowcharts, the `diagram { }` block, sample-flow and CONSORT diagrams drawn from Zeff's own row counts, and interactive SEM path diagrams with a model builder are specified in zeff_diagrams.md, on one deterministic layout engine shared with `g.plot()`.
9. **Some things stay out**: pie charts and 3D bars as shorthands, dual y-axes on unrelated data, rainbow palettes, word clouds, Venn diagrams beyond three sets and basemap tiles. Section 13 gives the reason for each.

---

## 2. How R's packages divide the work, and where Zeff stands

| # | Layer | What R's packages do | Representative R packages | Zeff as specified | Gap |
| --- | --- | --- | --- | --- | --- |
| 1 | Engine and devices | Draw primitives on a device; fonts; file formats | graphics, grid, grDevices, ragg, svglite, Cairo, devEMF, tikzDevice | A versioned scene and a renderer in Rust with embedded IBM Plex fonts; SVG, PDF and PNG at 300 dpi; byte-identical output | The primitive set, rasterised layers, TIFF and EPS, project fonts (sections 4, 9) |
| 2 | Grammar | Data → aesthetics → marks and statistics → scales → coordinates → facets → theme | ggplot2, lattice, tinyplot | `plot()` with 11 marks, 4 statistical layers (3 more in 1.2), 4 scale functions, `coord`, `facet`, `labels`, `legend`, `theme`, `annotate` at 0.7.0 | Positions, more scale types, `linetype:` and `alpha:` aesthetics (section 5) |
| 3 | New layers | Distributions, uncertainty, 2D density, beeswarms, hulls, mosaics, flows | ggdist, ggridges, ggbeeswarm, gghalves, ggrain, ggforce, ggdensity, ggpointdensity, qqplotr, ggmosaic, ggalluvial, waffle, treemapify, ggupset | Histogram, density, box plot, error bars; smooth, violin and aggregate from 1.2 | About 25 statistical layers (section 5.2) |
| 4 | Text, annotation, colour, themes | Non-overlapping labels, rich text, highlighting, palettes, themes | ggrepel, ggtext, geomtextpath, directlabels, gghighlight, latex2exp, viridis, RColorBrewer, scico, colorspace, ggthemes, hrbrthemes | `labels`, `annotate`, `legend(direct)`, three themes, Okabe–Ito and viridis by default, text thinned rather than overlapped | Repulsion, rich text, highlighting, panel tags, more palettes (sections 5.5, 5.8) |
| 5 | Composition | Several plots in one figure; insets; aligned panels | patchwork, cowplot, gridExtra, egg, lemon, ggmagnify, ggExtra, ggside | `arrange([p1, p2], cols:)` from 1.2 | Nesting, tags, shared legends and axes, insets, margins (section 5.10) |
| 6 | Wrappers and result plots | One call per analysis or per fitted object | GGally, ggfortify, ggpubr, ggstatsplot, sjPlot, see, performance, factoextra, bayesplot, survminer, cobalt, metafor, ggeffects, marginaleffects | 8 shorthands, `fit.plot()`, `fit.coef_plot()`, `g.plot()`; `forest_plot`, `funnel_plot` and `scatter_matrix` on the roadmap | Result plots per package (section 7); more data shorthands (section 6) |
| 7 | Interaction and animation | JavaScript widgets, linked brushing, frame-by-frame animation | plotly, ggiraph, highcharter, echarts4r, crosstalk, gganimate | Zemp's viewer: redraw to fit, hover values, click to row, compare, focus, pop out | Brushing, linked views, legend toggling, HTML export, animation (sections 8, 11) |
| 8 | 3D, maps, networks | Specialised engines | rgl, plot3D, rayshader, sf, tmap, ggspatial, ggraph, igraph | `coord(map, crs:)` with the future `spatial` package; `g.plot()` for causal graphs | Contours and surfaces, choropleths, network layouts (section 11) |
| 9 | GUI builders | Build a ggplot by dragging | esquisse, ggplotgui | Zemp's plot-layer dialog writes the chain (Zemp Specification, section 6) | Covered |

---

## 3. Design rules carried into every capability

These rules come from the principles of section 1.2 of the Engine and Project Specification and from section 7.6 of the Language Reference. Every proposal below obeys them.

| # | Rule | What it means for graphics |
| --- | --- | --- |
| G1 | **One system** | Every plotting function returns a `Plot` built from the grammar: shorthands, result plots, `g.plot()`, path diagrams and maps alike. There is no second drawing system, so every figure can be restyled and extended, and `p.computed()` returns the data behind any layer |
| G2 | **The engine computes; the plot draws** | A statistical layer calls the engine's own function and inherits its accuracy class: `density` comes from `stats`, `qq` bands from `dist`, `smooth(fit: m)` from the result interface. No layer re-implements a statistic, and a layer that needs a tier-2 model is drawn by that model's package or through `fit:` |
| G3 | **Deterministic layout** | Jitter, beeswarm, label repulsion, network layout and sampling of large data run a fixed number of iterations on the seeded stream (section 8.3, point 3 of the Engine and Project Specification), so a figure's SVG and PDF bytes are identical on every machine |
| G4 | **Nothing disappears silently** | Rows dropped for `NA`, marks clipped by a zoom, points binned or sampled for speed, and observations outside a facet are counted in the plot's notes (principle 6) |
| G5 | **Honest defaults** | Bars start at zero; no rainbow scales; one y-axis per panel, unless a second axis is an exact transform of the first; zooming never filters data; every interval states what it is (95% CI, ± 1 SE, ± 1 SD) |
| G6 | **Readable at every size** | The scene is redrawn to the pane; text is set in points and never below 8 pt in exports; labels are wrapped, thinned or repelled, never overlapped |
| G7 | **Interaction is view state** | Hovering, brushing, zooming, toggling a legend entry and dragging a graph node never change the script or a result; **Add to script** writes the equivalent statement (Zemp Specification, section 5) |
| G8 | **Accessible by default** | Colour-blind-safe palettes, optional redundant encoding by shape or line type, alt text generated from the scene, minimum contrast in every theme |
| G9 | **Print first** | Exports use the light print theme, journal sizes and vector formats by default, and can carry the figure's source data |
| G10 | **Names follow section 3.6 of the Language Reference** | Data plots are plain chart nouns and frame methods (`ecdf`, `ridges`); derived plots end in `_plot` and are methods on results (`influence_plot`); a layer and a shorthand that draw the same thing share a name (as `histogram` does), and each such name joins the list of names with two homes in rule 6 of section 8.1 of the Engine and Project Specification |

---

## 4. Decide before the renderer is built (0.7.0)

Each item is cheap to fix now and would later break the bytes of existing figures or the scene format. These items continue the future-proofing sweep of 17 October 2026.

| # | Decision | Recommendation | Why now |
| --- | --- | --- | --- |
| F1 | The scene's primitives | Fix the set at version 1: point (with a shape), path (open or closed), polygon with holes, rectangle, segment with optional arrow heads, text run (styled spans), raster cell grid, and clip region. Everything in sections 5 to 11 reduces to these, so no later layer needs a new primitive. Gradients, images from files and blur effects stay out | Zemp, the exporters and the snapshot tests all read the scene; a new primitive is a new scene version that every reader must handle |
| F2 | Row references | Each mark in the viewer's scene carries the binding and row positions it came from; exported SVG, PDF and PNG never carry them | Hover, click to row, brushing and linked views (section 8) all need them, and exports must not leak hidden data |
| F3 | Reserved dimensions | Reserve three optional scene fields: a frame index (animation), a `z` coordinate with a projection (3D), and a `crs` on the coordinate system (maps). Readers that do not know them refuse the scene with a message rather than drawing it wrongly | Animation, surfaces and maps (section 11) then arrive without a new scene format |
| F4 | Rasterised layers | A layer flag `raster: true` renders that layer as a raster grid inside an otherwise vector export (as ggrastr does) | Keeps a 500,000-point PDF small and fast while axes and text stay vector; the flag must exist in scene version 1 |
| F5 | Scale limits are a zoom, not a filter | `scale_x(limits:)` and `scale_y(limits:)` set the visible range only. Marks outside it are clipped and counted in a note, and statistical layers are computed on all rows. Removing rows is written as `.where()` on the data. ggplot2's default, where limits drop rows and silently change box plots and smooths, is a known trap | `scale_x` ships at 0.7.0; reversing its meaning after 1.0 would change existing figures |
| F6 | Rich text in labels | Labels take `Str` as plain text in 1.0, so `*` and `^` are always literal. Reserve the type name `RichText` in `core` now. Later, `rich("*R*^2^ within")`, with italic, bold, superscript and subscript and Greek letters typed directly as UTF-8, becomes an addition that label parameters accept as `Str \| RichText` | Interpreting markup in plain strings later would change every existing label containing `*`; a separate type keeps it additive and checked |
| F7 | Notes on exported figures | The note counting dropped or clipped rows shows in the viewer, goes into the run record and into `p.notes()`, but stays off an exported figure unless `export(notes: true)` | Journal figures should not carry engine notes by default; the current text says "counted in a note on the plot" without separating screen from export |
| F8 | Labelling defaults | A variable label becomes the axis title (already specified). Value labels name legend keys and facet strips, and strips read `label: value` when `facet(strip: full)` asks | Snapshot tests fix these bytes; choose once |
| F9 | Shape and line-type sequences | Fix the default order of eight shapes and six line types, distinguishable in greyscale, beside the fixed Okabe–Ito order | Needed for redundant encoding and greyscale print, and part of a figure's bytes |
| F10 | Ticks and number formats | Ticks from the extended Wilkinson algorithm (Talbot, Lin and Hanrahan 2010, as R's labeling package implements it), with a fixed candidate set; numbers formatted as in tables (section 3.8 of the Language Reference) | Every figure's axes depend on it |
| F11 | Interval statements | Any layer that draws an interval records its kind and level in the scene (`ci 0.95`, `se 1`, `sd 1`, `range`), and the legend or caption states it (`error bars: 95% CI`) | A referee must be able to read what a bar means (principle 7) |
| F12 | Result-plot convention and prediction errors | Derived plots are `_plot` methods (section 7), and the estimator interface gains standard errors for predictions (`predict(newdata, se: true)`), which prediction plots need for their bands | The interface is declared in `results` and frozen at 1.0 |

---

## 5. The grammar, extended

The usage examples below follow the conventions of section 7.6 of the Language Reference. "Spec" in the When column marks something already specified, with its release.

```
plot(wages, x: educ, y: lwage, color: female)
    .bins2d(shape: hex)                                  // 18,000 points, readable
    .smooth(fit: wages.reg(lwage ~ educ * female, cluster: worker))
    .facet(by: industry, wrap: 4)
    .scale_y(limits: [1, 5])                             // zoom: no row is dropped (F5)
    .labels(title: "Returns to schooling", y: rich("log *wage*"), tag: a)
    .theme(journal)
    .export("output/fig2.pdf", size: two_column, with_data: true)
```

### 5.1 Data, aesthetics and marks

| Capability | Zeff | Draws | R source | When |
| --- | --- | --- | --- | --- |
| Aesthetics | `plot(frame, x:, y:, color:, fill:, size:, shape:, group:, label:)` | Mappings; a bare name maps, a string sets | ggplot2 `aes()` | Spec 0.7.0 |
| More aesthetics | `linetype:`, `alpha:` | Line type and transparency by variable | ggplot2 `aes()` | 1.2 |
| Basic marks | `points`, `lines`, `step`, `bars`, `area`, `ribbon`, `text`, `tile`, `rug`, `hline`, `vline` | The marks of section 7.6 | ggplot2 geoms | Spec 0.7.0 |
| Paths | `.paths()` | Points joined in data order: trajectories, cycles, outlines | `geom_path` | 1.2 |
| Segments and arrows | `.segments(x_end:, y_end:, arrow: none \| end \| both, curvature: Float = 0)` | Straight or curved segments, with arrow heads | `geom_segment`, `geom_curve` | 1.2 (spec: additive) |
| Rectangles | `.rects(x_low:, x_high:, y_low:, y_high:)` | Shaded periods, bands, boxes | `geom_rect` | 1.2 (spec: additive) |
| Polygons | `.polygons(group:)` | Closed shapes with holes | `geom_polygon` | 1.2 (spec: additive) |
| Interval styles | `.errorbars(style: bar \| range \| point_range \| crossbar)` | Error bars, line ranges, point ranges, crossbars | `geom_errorbar`, `geom_pointrange`, `geom_crossbar` | 1.2 |
| Label boxes | `.text(box: true)` | Text on a filled box | `geom_label` | 1.2 |
| Non-overlapping labels | `.text(repel: true, max_overlaps: Int)` | Labels pushed apart with leader lines; deterministic iterations (G3) | ggrepel | 1.2 |
| Text along a path | `.text(along: path)` | Labels that follow a curve | geomtextpath | After 1.11 |
| Raster | `.raster(fill:)` | A regular grid as an image, for very large heat maps | `geom_raster` | 1.2 |
| Geometry | `.geometry(fill:)` | A `Geometry` column (section 4.3 of the Language Reference) | `geom_sf` | With `spatial` |

### 5.2 Statistical layers

Each layer is computed by the package named in its row (G2).

| Purpose | Zeff | Computes and draws | Computed by | R source | When |
| --- | --- | --- | --- | --- | --- |
| Distribution | `.histogram(bins:, binwidth:)` | Freedman–Diaconis bins | `data` | `geom_histogram` | Spec 0.7.0 |
| Distribution | `.histogram(weights:, scale: count \| density \| percent)` | Weighted histograms; densities to overlay a curve | `data` | ggplot2 `weight`, `after_stat(density)` | 1.2 |
| Distribution | `.density(bandwidth:, adjust:)` | Silverman's bandwidth | `stats` | `geom_density` | Spec 0.7.0 |
| Distribution | `.density(bounds: [0, 1], weights:)` | Bounded densities with boundary correction, and weighted densities, for shares and survey data | `stats` | ggdist `density_bounded` | 1.2 |
| Distribution | `.boxplot(outliers:)` | Tukey box plot | `data` | `geom_boxplot` | Spec 0.7.0 |
| Distribution | `.violin(bandwidth:)` | Mirrored densities | `stats` | `geom_violin` | Spec 1.2 |
| Distribution | `.ecdf()` | Empirical cumulative distribution as a step line | `data` | `stat_ecdf` | 1.2 |
| Distribution | `.qq(distribution, band: none \| pointwise \| simultaneous)` | Q–Q points with a reference line and confidence band | `dist` | qqplotr `stat_qq_band` | 1.2 |
| Distribution | `.ridges(scale: Float)` | Overlapping densities, one row per group | `stats` | ggridges | 1.2 |
| Distribution | `.beeswarm(method: swarm \| quasirandom \| sina)` | Every observation, packed without overlap | `plot` | ggbeeswarm, ggforce `geom_sina` | 1.2 |
| Distribution | `.halfeye(level: List = [0.66, 0.95])` | Density slab with median and nested intervals | `stats` | ggdist `stat_halfeye` | 1.2 |
| Distribution | `.raincloud()` | Half violin, box plot and points together | `stats`, `data` | ggrain, gghalves | 1.2 |
| Distribution | `.dotplot(quantiles: Int)` | Quantile dot plots: n dots, each a 1/n probability | `data` | ggdist `stat_dots` | After 1.11 |
| Relationship | `.smooth(method: lm \| loess, fit:)` | Fitted line with band; `fit: m` draws any model exactly as `m` prints | kernel, `stats`, result interface | `geom_smooth` | Spec 1.2 |
| Relationship | `.smooth(method: spline, df:)` | Natural-spline fit | `stats` (`spline`) | `geom_smooth(formula = y ~ ns(x))` | With `spline` |
| Relationship | Quantile lines | `.smooth(fit: data.quantile_reg(y ~ x, q: [0.1, 0.5, 0.9]))` | `regress` through the result interface | `geom_quantile` | With `quantile_reg` |
| Relationship | `.bins2d(shape: hex \| square, bins:)` | Counts on a hexagonal or square grid | `data` | `geom_hex`, `geom_bin2d` | 1.2 |
| Relationship | `.density2d(levels: Int \| hdr, fill: Bool)` | Contours of a 2D kernel density, or highest-density regions at 50/80/95% | `stats` | `geom_density_2d`, ggdensity | 1.2 |
| Relationship | `.pointdensity()` | Points coloured by local density | `stats` | ggpointdensity | After 1.11 |
| Relationship | `.ellipse(level: Float = 0.95, type: normal \| t)` | Confidence or data ellipses per group | `stats`, `dist` | `stat_ellipse` | 1.2 |
| Relationship | `.hull()` | Convex hull per group | `plot` | ggforce `geom_mark_hull` | After 1.11 |
| Surface | `.contour(z:, filled: Bool)` | Contours of a gridded surface, such as a response surface | `plot` | `geom_contour`, rsm `contour` | 1.9 |
| Summary | `.aggregate(fun: mean \| median \| sum \| proportion, errorbars: none \| se \| ci \| sd)` | One summary per group with stated intervals; Wilson intervals for proportions | `data`, `stats` | `stat_summary` | Spec 1.2; `proportion` 1.2 |
| Function | `.curve(dist: dist.normal(mean: 3, sd: 0.5), type: pdf \| cdf)`, `.curve(fn: f)` | A distribution or a named function over the x range; with `fill_between:` for a tail area | `dist`; the named function | `stat_function`, `geom_function` | 1.2 |
| Comparison | `.brackets(test: t \| wilcoxon, comparisons: List, adjust: holm)` | Brackets with adjusted p-values and effect sizes between groups | `stats` | ggpubr `stat_compare_means`, ggsignif | Decision (section 15) |
| Annotation from a result | `.annotate(result: t)` | The test's one-line summary as a subtitle | result interface | ggstatsplot | After 1.11 |
| Categorical | `.mosaic()` | Mosaic and spine plots: areas proportional to counts | `data` | ggmosaic, vcd | After 1.11 |

### 5.3 Positions

| Zeff | Use | R source | When |
| --- | --- | --- | --- |
| `bars(position: stack \| dodge \| fill)` | Stacked, side-by-side and 100% bars | `position_*` | Spec 0.7.0 |
| `points(jitter:)` | Seeded jitter | `position_jitter` | Spec 0.7.0 |
| `text(nudge:)` | Offset labels | `position_nudge` | Spec 0.7.0 |
| `dodge: Float` on `points`, `errorbars`, `boxplot`, `aggregate` | Side by side within an x position: several models' coefficients, grouped box plots | `position_dodge` | 1.2 |
| `area(position: stack \| fill)` | Stacked and 100% areas | `position_stack`, `position_fill` | 1.2 |
| `points(jitter:, dodge:)` together | Jittered points beside dodged box plots | `position_jitterdodge` | 1.2 |

### 5.4 Scales and number formats

| Capability | Zeff | R source | When |
| --- | --- | --- | --- |
| Position scales | `scale_x(type: linear \| log \| sqrt \| date, limits:, breaks:, labels:)` | `scale_x_*` | Spec 0.7.0 |
| More transforms | `type: reverse \| log1p \| pseudo_log \| logit \| datetime` (`log1p` for counts with zeros, `pseudo_log` for signed values, `logit` for probabilities) | scales `log1p_trans`, `pseudo_log_trans`, `logit_trans`, `reverse_trans` | 1.2 |
| Label formats | `labels: percent \| comma \| scientific \| p_value \| "pattern"` | scales `label_percent`, `label_comma`, `label_scientific`, `label_pvalue` | 1.2 |
| Break count | `breaks: Int` asks for about n ticks; `minor: Bool` | scales `breaks_extended` | 1.2 |
| Second axis | `scale_y(secondary: log_odds \| percent \| Fn)`: only an exact one-to-one transform of the first axis | `sec_axis` | After 1.11 |
| Colour, fill, size, shape | `scale(color:, fill:, size:, shape:)` | `scale_colour_*` | Spec 0.7.0 |
| Line type and alpha | `scale(linetype:, alpha:)` | `scale_linetype_*`, `scale_alpha_*` | 1.2 |
| Binned and classed colour | `scale(fill: binned, breaks:)`; `classes: quantile \| equal \| jenks` for maps | `scale_fill_binned`, classInt | 1.2; classes with `spatial` |
| Diverging scales | `scale(fill: diverging, mid: 0)` | `scale_fill_gradient2` | 1.2 |
| Redundant encoding | `scale(redundant: true)` maps each colour group to a shape or line type as well | ggplot2 by hand | 1.2 |

### 5.5 Colour and palettes

| Capability | Zeff | R source | When |
| --- | --- | --- | --- |
| Categories | Okabe–Ito, eight colours in fixed order | ggthemes `colorblind_pal`, palette.colors | Spec 0.7.0 |
| Continuous | viridis | viridisLite | Spec 0.7.0 |
| More continuous | `magma`, `inferno`, `cividis`, `mako`, `rocket` | viridisLite | 1.2 |
| ColorBrewer | `brewer` with a palette name: qualitative `Set2`, `Dark2`; sequential `Blues`, `Greens`; diverging `RdBu`, `PuOr`, `BrBG` | RColorBrewer | 1.2 |
| Scientific colour maps | Perceptually uniform maps such as `vik` and `batlow`; licence to be confirmed before shipping | scico | After 1.11 |
| Greys | `grey` palette and a greyscale print check | scales `grey_pal` | 1.2 |
| Rules | More than eight categories gives a note suggesting facets or direct labels; a diverging scale needs a midpoint; rainbow and jet palettes are not shipped (`scale(color: [...])` still accepts any list) | colorspace guidance | Spec in part; rest 1.2 |
| Colour-vision simulation | The viewer shows the figure as seen with deuteranopia, protanopia or tritanopia (view state) | colorblindr `cvd_grid` | Zemp, 1.2 |

### 5.6 Coordinates

| Capability | Zeff | R source | When |
| --- | --- | --- | --- |
| Cartesian, flipped, polar, fixed ratio | `coord(cartesian \| flip \| polar \| fixed, ratio:)` | `coord_*` | Spec 0.7.0 |
| Zoom | `coord(x_range: [a, b], y_range: [c, d])`, the same as `scale_*(limits:)` under F5 | `coord_cartesian(xlim =)` | Fix now (F5) |
| Map projections | `coord(map, crs:)` | `coord_sf` | Spec: with `spatial` |
| Ternary | `coord(ternary)` for three-part compositions | ggtern | After 1.11 |
| Perspective | `coord(perspective, theta:, phi:)` for surfaces | `persp`, plot3D | After 1.11 (F3) |

### 5.7 Facets

| Capability | Zeff | R source | When |
| --- | --- | --- | --- |
| Wrap and grid | `facet(by:, rows:, cols:, scales: fixed \| free \| free_x \| free_y)` | `facet_wrap`, `facet_grid` | Spec 0.7.0 |
| Number of columns | `facet(by:, wrap: Int)` | `facet_wrap(ncol =)` | 1.2 |
| Panel sizes | `facet(rows:, cols:, space: fixed \| free)`: panel size proportional to range | `facet_grid(space = "free")` | 1.2 |
| All-groups panel | `facet(by:, margins: true)` adds an "All" panel | `facet_grid(margins = TRUE)` | 1.2 |
| Strip labels | Value labels by default; `strip: full` writes `label: value` (F8) | `labeller` | Fix now (F8) |
| Nested facets | `facet(rows: [region, country])` with nested strips | ggh4x `facet_nested` | After 1.11 |
| Context in every panel | `facet(by:, context: true)` draws the other groups in grey behind each panel | gghighlight | 1.2 |

### 5.8 Guides, text and annotation

| Capability | Zeff | R source | When |
| --- | --- | --- | --- |
| Titles | `labels(title:, subtitle:, x:, y:, color:, caption:)` | `labs` | Spec 0.7.0 |
| Legends | `legend(right \| top \| bottom \| none \| direct)` | `theme(legend.position)`, directlabels | Spec 0.7.0 |
| Legend inside the panel | `legend(inside, at: [0.9, 0.1])` | `legend.position.inside` | 1.2 |
| Merged legends | One legend when several aesthetics map the same variable (fixed rule) | ggplot2 guide merging | Fix now |
| Crowded axis labels | Labels wrapped, thinned or set on two rows; `angle:` only when asked | `guide_axis(n.dodge =)` | Spec in part; 1.2 |
| Text annotation | `annotate(text:, x:, y:)` | `annotate("text")` | Spec 0.7.0 |
| Shape annotation | `annotate(kind: rect \| segment \| arrow \| band, ...)` | `annotate("rect")` | 1.2 |
| Highlighting | `.highlight(condition, label: Bool)`: matching groups in colour, the rest grey | gghighlight | 1.2 |
| Callouts | `.callout(where: condition, text: Var)`: repelled labels for chosen rows | ggrepel, ggforce `geom_mark_*` | 1.2 |
| Panel tags | `labels(tag: a \| A \| 1 \| i)` on a plot or an arrangement | patchwork `tag_levels` | 1.2 |
| Rich text and symbols | `rich("*β*~educ~")`, `rich("*R*^2^")` (F6) | ggtext, latex2exp, plotmath | After 1.0 (type reserved now) |
| Interval statement | Automatic legend or caption line for every interval layer (F11) | none built in | Fix now (F11) |
| Alt text | `labels(alt:)`, or generated from the scene: chart type, axes, ranges, groups, trend | BrailleR | 1.2 |

### 5.9 Themes and fonts

| Capability | Zeff | R source | When |
| --- | --- | --- | --- |
| Themes | `theme(journal \| minimal \| slide, base_size:)` | `theme_minimal` and friends | Spec 0.7.0 |
| Typed overrides | `theme(journal, grid: none \| major \| x \| y, axis_lines: Bool, legend_title: Bool, aspect: Float, font_size: Float)`: a fixed record, never free styling | ggplot2 `theme()` elements | 1.2 |
| Greyscale print | `theme(print_grey)` with redundant encoding switched on | ggthemes `theme_few` with greys | 1.2 |
| Project fonts | `theme(font: "fonts/Body.otf")`: a font file in the project, hashed like any input, so a figure can match a journal's typeface and stay byte-identical | ragg, showtext, systemfonts | 1.2 |
| Dark mode | Screen only: Zemp's dark theme redraws plots on screen; exports stay light (Zemp Specification, section 5) | none built in | Spec (Zemp) |

### 5.10 Composition

| Capability | Zeff | R source | When |
| --- | --- | --- | --- |
| Side by side | `arrange([p1, p2], cols:, legend: shared \| each)` | patchwork | Spec 1.2 |
| Nesting and sizes | `arrange([p1, arrange([p2, p3])], widths: [2, 1], heights:)` | patchwork | 1.2 |
| Collected guides | `arrange(..., collect: legends \| axes \| both)` | patchwork `guides = "collect"`, `axes = "collect"` | 1.2 |
| Tags and shared titles | `arrange(..., tag: a, title:, caption:)` | patchwork `plot_annotation` | 1.2 |
| Aligned panels | Panel areas aligned across an arrangement by default | egg, cowplot `align_plots` | 1.2 |
| Insets | `p.inset(q, at: [x, y, width, height])` | patchwork `inset_element`, cowplot `draw_plot` | 1.2 |
| Magnifier | `p.magnify(x_range:, y_range:, at:)`: an enlarged inset of a region with connecting lines | ggmagnify, ggforce `facet_zoom` | After 1.11 |
| Marginal plots | `p.margins(x: density \| histogram \| boxplot, y: ...)` | ggExtra `ggMarginal`, ggside | 1.2 |

---

## 6. Data plots: shorthands by question

Shorthands are frame methods returning the same `Plot` as their layer chain (G1). They are what Zemp's quick plots write.

| Question | Shorthand | Draws | R source | Built from | When |
| --- | --- | --- | --- | --- | --- |
| Distribution | `histogram`, `density`, `boxplot`, `qq_plot` | As specified | base R, ggplot2 | Their layers | Spec 0.7.0 |
| Distribution | `violin(var, group:)` | Violins per group | `geom_violin` | `.violin()` | 1.2 |
| Distribution | `ecdf(var, color:)` | ECDF per group | `stat_ecdf` | `.ecdf()` | 1.2 |
| Distribution | `ridges(var, group:)` | Ridgeline of densities | ggridges | `.ridges()` | 1.2 |
| Distribution | `beeswarm(var, group:)` | All points per group | ggbeeswarm | `.beeswarm()` | 1.2 |
| Distribution | `raincloud(var, group:)` | Raincloud | ggrain | `.raincloud()` | 1.2 |
| Relationship | `scatter`, `line` | As specified | base R | `.points()`, `.lines()` | Spec 0.7.0 |
| Relationship | `hexbin(x, y)` | Hexagonal counts | `geom_hex` | `.bins2d(shape: hex)` | 1.2 |
| Relationship | `data.binscatter(y, x, controls:, fe:, bins:)` | Binned scatter with controls and fixed effects, after Cattaneo, Crump, Farrell and Feng; in `regress` | binsreg, Stata binscatter | `regress`, `.points()`, `.errorbars()` | 1.2 |
| Multivariate | `scatter_matrix(vars, color:)` | Typed panels: scatter below, correlations above, densities on the diagonal; box plots or bars for category pairs | GGally `ggpairs` | Layers per panel | Roadmap; propose 1.2 |
| Multivariate | `parallel_coordinates(vars, color:, scale: std \| range)` | One line per row across standardised variables | GGally `ggparcoord` | `.lines()` | After 1.11 |
| Correlation | `heatmap(vars)` | Correlation tiles | corrplot, ggcorrplot | `.tile()` | Spec 0.7.0 |
| Correlation | `heatmap(vars, order: given \| angle, triangle: full \| lower \| upper, p_values: Bool, values: Bool)` | Ordering by angular order of eigenvectors (kernel eigendecomposition), triangle, significance marks, printed values; ordering by hierarchical clustering comes from `groups` | corrplot `order = "AOE"`, ggcorrplot | `.tile()`, `.text()` | 1.2 |
| Comparison | `bar` | As specified | base R | `.bars()` | Spec 0.7.0 |
| Comparison | `dotchart(var, value:)` | Cleveland dot plot, sorted | base R `dotchart` | `.points()` | 1.2 |
| Comparison | `dumbbell(var, from:, to:)` | Two values joined per row (before and after, two groups) | ggalt `geom_dumbbell` | `.segments()`, `.points()` | 1.2 |
| Comparison | `slopegraph(group, time:, value:)` | Lines between two or a few time points, labelled at both ends | CGPfunctions `newggslopegraph` | `.lines()`, `.text(repel:)` | After 1.11 |
| Comparison | `lollipop(var, value:)` | Stick and dot | ggalt | `.segments()`, `.points()` | After 1.11 |
| Comparison | `bump(group, time:, rank:)` | Ranks over time | ggbump | `.lines()`, `.points()` | After 1.11 |
| Composition | Stacked and 100% bars | `bar(var, color:)` with `position:` | ggplot2 | `.bars()` | Spec 0.7.0 |
| Composition | `likert(items, levels:)` | Diverging stacked bars for Likert items, centred on the neutral level, items ordered by agreement | ggstats `gglikert`, likert | `.bars()` | 1.2 |
| Composition | `waffle(var)` | Unit squares | waffle | `.tile()` | After 1.11 |
| Composition | `treemap(var, size:, parent:)` | Nested rectangles (squarified) | treemapify | `.rects()` | After 1.11 |
| Composition | `mosaic(a, b)` | Mosaic of a cross-table, coloured by residuals | ggmosaic, vcd | `.mosaic()` | After 1.11 |
| Flow | `alluvial(vars, weight:)` | Flows between categorical stages | ggalluvial, ggforce `geom_parallel_sets` | `.polygons()` | After 1.11 |
| Sets | `upset(sets)` | Intersection sizes with a membership matrix | ggupset, UpSetR | `arrange`, `.bars()`, `.points()` | After 1.11 |
| Time | `line(time, y, color:)` | As specified | base R | `.lines()` | Spec 0.7.0 |
| Time | `calendar(date, value:)` | Calendar heat map | ggTimeSeries, openair | `.tile()`, `facet` | After 1.11 |
| Panels | Spaghetti plots | `line(time, y, group: unit, alpha: 0.2)` with `.aggregate()` on top | ggplot2 | `.lines()` | Spec (pattern) |
| Panels | `data.treatment_plot(treat:, unit:, time:)` | Treatment status by unit and period: rollout, missing cells, reversals; in `quasi` | panelView | `.tile()` | 1.2 |
| Missing data | `missing_pattern(vars)` | Pattern matrix with counts (table and plot); in `explore` | mice `md.pattern`, naniar, visdat `vis_miss` | `.tile()`, `.bars()` | Roadmap; propose with `explore` |
| Missing data | `data.missing_plot(vars, by:)` | Share missing per variable, and per group or period | naniar `gg_miss_var`, `gg_miss_fct` | `.bars()`, `.tile()` | With `explore` |

---

## 7. Result plots by package

This is ggfortify's `autoplot` idea, built into the result interface. `fit.plot()` stays the default for each result type (already specified: a coefficient plot for `reg`, an event study for `did`). Each named `_plot` method is defined once per result type, in the package that owns the type, and draws with the `plot` grammar (calls go downward, rule 2). Generic plots that need only the result and estimator interfaces live in `plot` and apply to every `Result`; rule 6 forbids redefining them for a subtype, so type-specific plots get their own names.

```
let m = wages.reg(lwage ~ educ * female + exper, fe: industry + year, cluster: worker)
m.coef_plot()                                       // estimates with 95% CIs
m.residual_plot()                                   // four panels, editable like any Plot
m.prediction_plot(x: educ, by: female)              // predictions with delta-method bands
coef_plot([m1, m2, m3], terms: [educ, female])      // several models, dodged
m.influence_plot().labels(title: "Influential workers")
```

### 7.1 Every result (`plot` package, through the result and estimator interfaces)

| Plot | Zeff | Draws | R source | When |
| --- | --- | --- | --- | --- |
| Default | `fit.plot()` | The type's default view | `plot()`, `autoplot()` | Spec 0.7.0 |
| Coefficients | `fit.coef_plot(terms:, standardize: Bool, level:)` | Estimates with intervals, term labels from variable labels | Stata coefplot, dotwhisker, modelsummary `modelplot` | Spec 0.7.0; options 1.2 |
| Several models | `coef_plot([m1, m2], terms:)` | Models dodged and labelled | dotwhisker `dwplot` | 1.2 |
| Residuals | `fit.residual_plot(kind: all \| fitted \| qq \| scale_location \| histogram)` | Residuals against fitted values, Q–Q with band, scale–location, distribution | `plot.lm`, performance `check_model`, ggResidpanel | Candidate for 1.0 (section 14) |
| Predictions | `fit.prediction_plot(x:, by:, at: Record)` | Adjusted predictions with bands over a regressor, by group (needs F12) | ggeffects, marginaleffects `plot_predictions`, Stata marginsplot | Candidate for 1.0 |
| Slopes and comparisons | `fit.slope_plot(x:, by:)`, `fit.comparison_plot()` | Marginal effects across a moderator | marginaleffects `plot_slopes`, `plot_comparisons` | With `margins` |
| Binary outcomes | `fit.roc_plot()`, `fit.pr_plot()` | ROC and precision–recall curves with AUC | pROC, plotROC, yardstick | Candidate for 1.0 (ROC) |
| Calibration | `fit.calibration_plot(bins:)` | Predicted against observed with intervals | probably `cal_plot_breaks`, rms `calibrate` | 1.2 |
| Separation diagnostics | `fit.binned_residual_plot(bins:)` | Binned residuals with ± 2 SE bands for binary models | arm `binnedplot` | 1.2 |

### 7.2 `regress`, `glm` and `multilevel` (tier 2)

| Plot | Zeff | Draws | R source | When |
| --- | --- | --- | --- | --- |
| Influence | `fit.influence_plot()` | Studentized residuals against leverage, bubble size Cook's distance, top rows labelled | car `influencePlot`, `plot.lm` (5) | Candidate for 1.0 |
| Added-variable | `fit.partial_plot(term)` | Partial regression plot for one term, the slope equal to its coefficient | car `avPlots`, Stata avplot | 1.2 |
| Component plus residual | `fit.component_plot(term)` | Nonlinearity check for one term | car `crPlots` | 1.2 |
| Collinearity | `fit.vif_plot()` | VIFs with reference lines at 5 and 10 | performance `check_collinearity` | 1.2 |
| Event study | `fit.event_plot()` | Lead and lag coefficients from `event()` terms, reference period marked | fixest `iplot`, Stata coefplot | 1.2 (1.0 draws it through `coef_plot`) |
| First stage | `iv.first_stage_plot()` | Binned first-stage relationship with fitted line and F | ivreg diagnostics | 1.2 |
| Weak-IV robust set | `iv.ar_plot()` | Anderson–Rubin confidence set over the coefficient | ivmodel | With `ar_test` |
| Quantile process | `qfit.plot()` | Coefficients across quantiles with bands, the OLS estimate as a line | quantreg `plot.summary.rqs` | With `quantile_reg` |
| Random effects | `mixed.ranef_plot()` | Caterpillar plot of conditional modes with intervals | lme4 `dotplot.ranef.mer`, sjPlot | 1.4 |
| Growth trajectories | `mixed.trajectory_plot(sample: Int)` | Fitted unit trajectories with the mean curve | lme4, ggplot2 | 1.4 |
| Count models | `fit.rootogram()` | Hanging rootogram of observed against expected counts | countreg, topmodels | With `poisson` 1.2 |
| Survival | `fit.survival_plot(risk_table: Bool, censor: Bool)` | Kaplan–Meier or adjusted curves with numbers at risk and censoring marks; `cumulative_incidence` for competing risks | survminer `ggsurvplot`, ggsurvfit | With survival analysis |
| Proportional hazards | `cox.zph_plot()` | Scaled Schoenfeld residuals over time | survival `plot.cox.zph` | With survival analysis |

### 7.3 `stats`, `dist`, `resample` and `results` (tier 1, drawn in `plot`)

| Plot | Zeff | Draws | R source | When |
| --- | --- | --- | --- | --- |
| A distribution | `distribution.plot(type: pdf \| cdf, shade: [a, b])` | Density or CDF with a shaded region, for tail probabilities and critical values | base R `curve`, visualize | 1.2 |
| A test | `test.plot()` | The statistic on its null distribution with the rejection region and p-value area | ggstatsplot, infer `visualize` | 1.2 |
| Post-hoc comparisons | `posthoc.plot()` | Pairwise differences with adjusted intervals, zero line | emmeans `pwpp`, `plot.TukeyHSD` | 1.2 |
| Bootstrap | `boot.plot()` | Bootstrap distribution with percentile and BCa intervals | boot `plot.boot` | With `bootstrap` |
| Permutation | `perm.plot()` | Permutation null with the observed statistic | infer, coin | With `permutation_test` |
| Power | `power.plot(over: n \| effect)` | Power curves across sample size or effect | pwr `plot.power.htest` | 1.9 |

### 7.4 `explore`, `series`, `latent`, `measure`, `learn` and `groups` (tier 2)

| Package | Plot | Zeff | Draws | R source | When |
| --- | --- | --- | --- | --- | --- |
| explore | Scree | `pca.scree_plot()` | Eigenvalues or variance explained, cumulative line | factoextra `fviz_eig`, psych `scree` | With `pca` |
| explore | Biplot | `pca.biplot(components: [1, 2])` | Scores and loading arrows | factoextra `fviz_pca_biplot`, ggbiplot | With `pca` |
| explore | Loadings | `pca.loadings_plot()` | Loadings heat map or bars | factoextra `fviz_contrib` | With `pca` |
| explore | Correspondence map | `ca.plot()` | Row and column points on two dimensions | factoextra `fviz_ca_biplot` | With `correspondence` |
| explore | Embeddings | `emb.plot(color:)` | t-SNE and UMAP coordinates | ggplot2 | With `tsne`, `umap` |
| series | Time plot with forecast | `fit.forecast(h: 12).plot()` | Fan chart of forecast intervals | forecast `autoplot`, fable | With `forecast` |
| series | ACF and PACF | `data.acf_plot(y, lags:)`, `pacf_plot` | Autocorrelations with bands | feasts `ACF`, forecast `ggAcf` | With `series` |
| series | Seasonal | `data.seasonal_plot(y, period:)`, `subseries_plot` | Seasons overlaid, and one panel per season | feasts `gg_season`, `gg_subseries` | With `series` |
| series | Lags | `data.lag_plot(y, lags:)` | y against its lags | feasts `gg_lag` | With `series` |
| series | Decomposition | `dec.plot()` | Trend, seasonal and remainder panels | stats `plot.stl`, feasts | With a decomposition method |
| series | Impulse responses | `var.irf_plot()` | Responses with bands | vars `plot.varirf` | With `vector_ar` |
| latent | Path diagram | `m.plot()` on a `Model`; `fit.path_plot(estimates: std \| raw, show:, template:, mi:)` on a fit; interactive in the viewer, with a model builder in Zemp (zeff_diagrams.md, sections 6 to 8) | RAM notation, estimates, fit box, layout templates | semPlot `semPaths`, lavaanPlot, tidySEM, Stata SEM Builder | 1.3 (spec: additive) |
| latent | Factor diagram | `efa.plot(cut:)` | Factors with loadings above a cut | psych `fa.diagram` | With `efa` |
| latent | Parallel analysis | `pa.plot()` | Observed against simulated eigenvalues | psych `fa.parallel` | With `parallel_analysis` (spec: "with scree") |
| latent | Class profiles | `lca.profile_plot()` | Item probabilities or means by class | poLCA `plot`, tidyLPA `plot_profiles` | 1.7 |
| latent | Class enumeration | `fits.plot()` | AIC, BIC and entropy across class counts | tidyLPA, MplusAutomation | 1.7 |
| measure | Item curves | `irt.item_plot(items:)` | Item characteristic curves | mirt `itemplot` | With `irt` |
| measure | Information | `irt.information_plot()` | Test information and standard error | mirt `plot(type = "info")` | With `irt` |
| measure | Wright map | `irt.wright_map()` | Persons and item difficulties on one scale | WrightMap | With `irt` |
| measure | Agreement | `data.agreement_plot(a, b)` | Bland–Altman: differences against means with limits of agreement | BlandAltmanLeh, blandr | With `icc` |
| measure | Invariance | `inv.plot()` | Change in fit across invariance levels | semTools | With `invariance` |
| learn | Importance | `fit.importance_plot(n:)` | Permutation or impurity importance | vip | With learners |
| learn | SHAP | `fit.shap_plot(kind: summary \| dependence \| waterfall, var:)` | SHAP beeswarm, dependence and one-row waterfall | shapviz, SHAPforxgboost | With `shap` |
| learn | Partial dependence | `fit.partial_dependence(var).plot(ice: Bool)` | PD curve with ICE lines | pdp, iml, DALEX | With `partial_dependence` |
| learn | Tuning | `tuned.plot()` | Metric against each tuned parameter with fold intervals | tune `autoplot` | 1.5 |
| learn | Confusion matrix | `fit.confusion_plot(newdata:)` | Tile of predicted against observed | yardstick `autoplot.conf_mat` | 1.5 |
| learn | Lift and gain | `fit.gain_plot(newdata:)` | Cumulative gain and lift | yardstick `gain_curve` | 1.5 |
| learn | Lasso path | `lasso.path_plot()` | Coefficients across the penalty, selected λ marked | glmnet `plot.glmnet` | 1.5 |
| learn | Tree | `tree.plot()` | Fitted decision tree | rpart.plot, ggparty | With `tree` |
| groups | Clusters | `km.plot()` | Clusters on the first two principal components, with hulls | factoextra `fviz_cluster` | With `kmeans` |
| groups | Silhouette | `km.silhouette_plot()` | Silhouette widths by cluster | factoextra `fviz_silhouette` | With `kmeans` |
| groups | Number of clusters | `fits.plot(metric: wss \| silhouette \| gap)` | Elbow, silhouette and gap curves | factoextra `fviz_nbclust` | With `kmeans` |
| groups | Dendrogram | `hc.plot(k:)` | Dendrogram with k clusters coloured (spec: "with a dendrogram") | ggdendro, dendextend | With `hierarchical` |
| groups | Clustered heat map | `hc.heatmap(vars)` | Heat map ordered by the clustering, with dendrograms | pheatmap, ComplexHeatmap | With `hierarchical` |
| groups | Communities | `comm.plot(layout:)` | Network coloured by community | igraph, ggraph | With `communities` and `Network` |

### 7.5 Designs (tier 3)

| Package | Plot | Zeff | Draws | R source | When |
| --- | --- | --- | --- | --- | --- |
| identify | Causal graph | `g.plot()` | Latent nodes dashed, exposure and outcome marked | dagitty, ggdag | Spec 0.10.0 |
| identify | Adjustment and paths | `g.plot(highlight: adjustment_set \| open_paths, exposure:, outcome:)` | The chosen set and the open back-door paths in colour | ggdag `ggdag_adjustment_set`, `ggdag_paths` | 1.2 |
| experiment | Balance | `bal.plot()` (also for `matching` and `ipw`) | Love plot of standardised differences before and after, with ±0.1 lines | cobalt `love.plot` | 1.9 |
| experiment | Randomisation inference | `ri.plot()` | Permutation distribution and observed effect | ri2 | With `ri_test` |
| experiment | Designed experiments | `doe.effects_plot()`, `doe.interaction_plot()` | Main-effect and interaction plots | FrF2 `MEPlot`, `IAPlot` | 1.9 |
| experiment | Effect screening | `doe.half_normal_plot()`, `doe.pareto_plot()` | Half-normal (Daniel) and Pareto plots of effects | FrF2 `DanielPlot`, qcc | 1.9 |
| experiment | Response surface | `rs.plot(kind: contour \| perspective)` | Fitted surface with the stationary point | rsm `contour`, `persp` | 1.9 (contour); perspective after 1.11 |
| quasi | Event study | `did.plot()` | ATT by event time with simultaneous bands | did `ggdid`, csdid | Spec 1.2 |
| quasi | Group-time effects | `did.group_time_plot()` | ATT(g, t) by cohort | did `ggdid(type = "group")` | 1.2 |
| quasi | Sensitivity | `honest.plot()` | Robust confidence sets across M | HonestDiD `createSensitivityPlot` | With `honest_did` |
| quasi | Regression discontinuity | `rdd.plot(bins:)` | Binned means with polynomial fits on each side | rdrobust `rdplot` | With `rdd` |
| quasi | Manipulation | `dens.plot()` | Density on each side of the cutoff with bands | rddensity `rdplotdensity` | With `density_test` |
| quasi | Bandwidth sensitivity | `rdd.bandwidth_plot()` | Estimate across bandwidths | rdrobust | With `rdd` |
| quasi | Synthetic control | `synth.plot(kind: paths \| gaps \| placebo \| weights)` | Treated against synthetic, gaps, placebo distribution, donor weights | Synth `path.plot`, augsynth, synthdid | With `synth` |
| adjust | Overlap | `ov.plot(kind: density \| mirrored)` | Propensity distributions by group, mirrored histogram | cobalt `bal.plot`, WeightIt | With `overlap` |
| adjust | Sensitivity contours | `sens.plot()` | Contours of the adjusted estimate over confounder strength, with benchmarks | sensemakr `plot` | With `sensitivity` |
| adjust | E-value | `ev.plot()` | Bias-factor curve | EValue `bias_plot` | With `e_value` |
| mediate | Effects | `med.plot()` | ACME, ADE and total effect with intervals | mediation `plot.mediate` | With `mediate` |
| mediate | Sensitivity | `medsens.plot()` | ACME across ρ | mediation `plot.medsens` | With `mediation_sensitivity` |
| hetero | Moderation | `fit.simple_slopes(...).plot()`, `fit.johnson_neyman(...).plot()` | Simple slopes; Johnson–Neyman band with significance regions | interactions `interact_plot`, `johnson_neyman` | With those methods |
| hetero | CATE | `forest.cate_plot(by:)` | Distribution of conditional effects and by covariate | grf | 1.5 |
| hetero | GATES | `forest.gates(...).plot()` | Sorted group effects with intervals | GenericML | With `gates` |
| hetero | Subgroups | `fit.subgroups(...).plot()` | Subgroup forest plot with adjusted intervals | forestploter | With `subgroups` |
| hetero | Quantile effects | `qte.plot()` | QTE across quantiles | qte | With `qte` |
| decide | Policy tree | `pt.plot()` | Assignment tree with leaf actions | policytree `plot` | With `policy_tree` |
| structural | Counterfactuals | `sim.plot()` | Outcome distributions under baseline and policy | none built in | With `simulate` |
| meta | Forest | `meta.forest_plot(sort:, cumulative: Bool)` | Study estimates, pooled diamond, prediction interval | metafor `forest`, meta | Roadmap (spec) |
| meta | Funnel | `meta.funnel_plot(contours: Bool)` | Contour-enhanced funnel | metafor `funnel` | Roadmap (spec) |
| meta | Influence | `meta.influence_plot()` | Leave-one-out, Baujat and GOSH | metafor `baujat`, `gosh` | With `meta` |
| meta | Drapery | `meta.drapery_plot()` | p-value functions per study | meta `drapery` | With `meta` |
| meta | Meta-regression | `metareg.plot(moderator)` | Bubble plot with fitted line | metafor `regplot` | With `meta_reg` |
| meta | Network | `nma.plot()` | Treatment network, edge width by evidence | netmeta `netgraph` | With `network_meta` |
| meta | p-curve | `pc.plot()` | Observed against expected p-curves | dmetar `pcurve` | With `p_curve` |
| meta | Specification curve | `sc.plot()` | Sorted estimates over the indicator panel of choices | specr `plot_specs` | With `spec_curve` |
| Bayesian (1.10) | Diagnostics | `fit.trace_plot()`, `fit.rank_plot()` | Chains and rank histograms | bayesplot `mcmc_trace`, `mcmc_rank_hist` | 1.10 |
| Bayesian (1.10) | Posteriors | `fit.posterior_plot(kind: areas \| intervals \| halfeye)` | Posterior densities and intervals; prior against posterior | bayesplot `mcmc_areas`, ggdist | 1.10 |
| Bayesian (1.10) | Predictive checks | `fit.ppc_plot(kind: density \| stat)` | Replicated against observed data | bayesplot `ppc_dens_overlay` | 1.10 |

---

## 8. The viewer and interaction (Zemp)

Zemp's viewer is where R's htmlwidgets layer belongs (G7). These are Zemp features built on the scene's row references (F2). They need no Zeff function.

| Capability | Behaviour | R source | Zemp |
| --- | --- | --- | --- |
| Redraw to fit, gallery, compare, focus, pop out, journal sizes | As specified | none built in | Spec |
| Hover | Values and the row under the pointer, with the variable labels | plotly, ggiraph | Spec |
| Click to row | Opens the row in the data viewer | ggiraph | Spec |
| Brushing and lasso | Select marks; the selection is view state; **Add to script** writes the `where` statement, or saves a selection file over 20 rows (Zemp Specification, section 5) | plotly, ggiraph, crosstalk | 0.2 alpha onwards |
| Linked views | A selection in one plot or in the data viewer highlights the same rows in every view of that frame | crosstalk, linked brushing | 1.1 |
| Zoom and pan | View state; **Add to script** writes `coord(x_range:, y_range:)` | plotly | 0.2 alpha onwards |
| Legend toggling | Hide or show a group | plotly | 0.2 alpha onwards |
| Graph layout | Drag nodes of `g.plot()` and path diagrams; positions saved as view state (spec) | ggdag, semPlot | Spec |
| Colour-vision check | Simulate three kinds of colour-vision deficiency | colorblindr | 1.1 |
| Plot as table | A screen-reader table of each layer's computed data (`p.computed()`) | BrailleR | 1.1 |
| Show the code | The statement that made the plot, and its layer list in the plot dialog | esquisse | Spec |
| Copy | Copy as SVG or PNG to the clipboard | none built in | 0.2 alpha onwards |

---

## 9. Export and output

| Capability | Zeff | R source | When |
| --- | --- | --- | --- |
| Vector and raster | `export("fig.pdf" \| ".svg" \| ".png", size: one_column \| two_column \| slide \| [w, h])`; PNG at 300 dpi | grDevices, svglite, ragg | Spec 0.7.0 |
| Resolution | `export(..., dpi: 600)` | ragg | 1.2 |
| TIFF | `.tiff` with LZW compression at 300 or 600 dpi, which many journals ask for | ragg `agg_tiff` | 1.2 |
| EPS | `.eps` with text converted to outlines | `postscript`, `cairo_ps` | After 1.11 |
| Rasterised layers | `points(raster: true)` keeps a dense layer small inside a vector file (F4) | ggrastr, scattermore | 1.2 |
| Source data | `export(..., with_data: true)` writes each layer's computed data beside the figure as CSV, for journals that ask for the source data of figures | none built in | 1.2 |
| Accessible files | Alt text as SVG `title`/`desc` and as PDF alternative text | BrailleR, tagged PDF | 1.2 |
| Scene | `export("fig.zplot")`, the versioned JSON scene, for reloading in Zemp or other tools | none built in | Decision |
| LaTeX-native | `.tex` as PGF/TikZ, so the document's fonts set the labels | tikzDevice | After 1.11 |
| Interactive HTML | One self-contained HTML file with the scene and a small viewer (hover and zoom), no external scripts | plotly `saveWidget`, ggiraph | Decision; after 1.11 |
| Animated | GIF and animated SVG from a scene with frames (section 11) | gganimate, gifski | After 1.11 |
| Word and PowerPoint | Through SVG, which current Office versions place as editable vector shapes; EMF is not written | devEMF, rvg, officer | Spec (SVG) |
| Colour | sRGB only; CMYK conversion is left to publishers | grDevices `colormodel` | Spec |

---

## 10. Large data and performance

| Rule | Detail | R source |
| --- | --- | --- |
| Aggregate first | A scatter plot of more than 50,000 points draws normally but gives a note suggesting `.bins2d()` or `.density2d()`, which the engine computes in one pass over the column | ggplot2 `geom_hex`, datashader |
| Sampling is seeded | When the viewer samples points for speed, it draws from the seeded stream and counts what it left out (G3, G4) | scattermore |
| Rasterise in export | `raster: true` per layer (F4) | ggrastr |
| Canvas on screen | The viewer draws dense layers on a canvas and the rest as SVG; the scene is the same | plotly WebGL |
| Plots are cached | A plot is a value, so its scene is stored in the content-addressed result cache like any result (section 8.3, point 4 of the Engine and Project Specification) | none built in |
| Target | Redraw within 16 ms up to 100,000 points (Zemp Specification, section 14) | none built in |

---

## 11. Later families: animation, 3D, maps, networks

| Family | Design | Prerequisite | R source | When |
| --- | --- | --- | --- | --- |
| Animation | `p.animate(over: year, transition: states \| time, fps: Int)` returns a `Plot` whose scene has frames (F3); the viewer plays it; exports to GIF and animated SVG. MP4 is not written: the free encoders are GPL-licensed or patent-encumbered | Frame field in the scene | gganimate, gifski | After 1.11 |
| Contours | `.contour(z:)` on gridded data; filled contours | None | ggplot2, rsm | 1.9, with `response_surface` |
| 3D surfaces | `.surface(z:)` under `coord(perspective, theta:, phi:)`; static projection in exports, rotation in the viewer as view state | z and projection in the scene (F3) | plot3D `persp3D`, rgl, plotly 3D | After 1.11 |
| 3D scatter | Not offered: hard to read in print; use facets, colour or a scatter matrix | None | scatterplot3d | Left out |
| Maps | `.geometry(fill:)` with `coord(map, crs:)`; classed colour scales; `annotate(kind: scale_bar \| north_arrow)`; inset locator maps through `p.inset()` | `spatial` package; `Geometry` columns | sf `geom_sf`, tmap, ggspatial | With `spatial` |
| Spatial diagnostics | Moran scatter plot; local cluster (LISA) maps | `spatial` | spdep, rgeoda | With `spatial` |
| Basemap tiles | Not fetched: tiles need the network and carry usage terms; a user may place a raster file of their own | None | ggmap, maptiles | Left out |
| Networks | `net.plot(layout: force \| stress \| circle \| layered, seed:)`, with deterministic layouts and edge bundling left out | `Network` type | ggraph, igraph | With `Network` |
| Causal graphs, flowcharts and SEM | One layered (Sugiyama) layout engine of Zeff's own, deterministic (zeff_diagrams.md) | None | ggdag, DiagrammeR, semPlot | 0.10.0 (`g.plot`); 1.2 (`diagram { }`); 1.3 (`path_plot`) |
| Text | Term frequency bars and topic prevalence through ordinary layers | text package | tidytext | With text analysis |

---

## 12. Plot extensions

R's ggplot2 gallery grew to 167 packages because ggproto let anyone add geoms and stats. Zeff gets the same reach without opening the renderer.

| Level | How | When |
| --- | --- | --- |
| Project functions | `fn sales_plot(data: Frame, by: Var) -> Plot { ... }` in `functions/`: a plot is a value, so a project can define plot recipes today, with help pages and a Zemp dialog generated from the signature | From 0.7.0 |
| Zeff-source packages | Plot functions, themes and palettes shipped as a package of `.zeff` files | Open decision on Zeff-source packages before 1.11 (section 15 of the Engine and Project Specification) |
| Component packages | A new statistical layer is a pure function from a frame to a frame, the layer's computed data, plus a choice of how existing primitives draw it. A package can never add a scene primitive, so the renderer stays closed, deterministic and small | 1.11 |

---

## 13. Left out or discouraged, and why

| Item | Decision | Reason and alternative |
| --- | --- | --- |
| Pie and donut charts | No shorthand; `coord(polar)` on bars still draws them | Angles are read worse than lengths; `dotchart`, `bar` or `waffle` |
| 3D bars, 3D pies, 3D scatter | Not offered | Perspective distorts values; facets or colour |
| Dual y-axes on unrelated data | Refused; only an exact transform of the first axis (`secondary:`) | Two scales invite spurious correlation; two aligned panels with `arrange` |
| Radar and spider charts | No shorthand | Area depends on variable order; `parallel_coordinates` or `dotchart` |
| Rainbow and jet palettes | Not shipped | Not perceptually uniform, unreadable with colour-vision deficiency; viridis family |
| Broken axes on bars | Not offered | Bars must start at zero; a log scale or an inset |
| Venn diagrams beyond three sets | Not offered | Areas become unreadable; `upset` |
| Word clouds | Not offered | Size and position carry no reliable quantity; term frequency bars |
| Images or emoji as points | Not offered | Not reproducible across fonts and platforms; shapes and labels |
| Chart-junk themes (newspaper or spreadsheet look-alikes) | Not shipped | Principle of one default theme; typed overrides cover journal styles |
| Basemap tiles | Not fetched | Network access and tile terms of use; a user's own raster file |
| p-value stars without effect sizes | Not offered | `.brackets()`, if adopted, prints adjusted p-values beside effect sizes |
| Bitmap-only output | Never | Every figure is a vector scene first |

---

## 14. Where the proposals land

The 1.0 candidates are the only proposals that would add functions to Zeff 1.0. Four plots are recommended, because a regression release without diagnostic and prediction plots falls short of what R and Stata offer by default: `residual_plot` and `prediction_plot` in `plot`, `influence_plot` in `regress`, and `roc_plot` in `plot`. Adding them would take 1.0 from 257 to 261 functions and needs the specification-freeze decision (#40 of zeff_open_decisions.md).

| Placement | What | Count |
| --- | --- | --- |
| Already specified | The 0.7.0 grammar and shorthands, the 1.2 layers, `g.plot`, `forest_plot`, `funnel_plot`, `scatter_matrix`, `did.plot`, `sem.plot` and the Zemp viewer features marked Spec | — |
| Decide before 0.7.0 | F1 to F12 (section 4) | 12 |
| Candidates for 1.0 | `residual_plot`, `prediction_plot`, `influence_plot`, `roc_plot` | 4 |
| 1.2, beside the layers already planned there | Grammar extensions in sections 5.1 to 5.10, data shorthands in section 6, generic result plots, `regress` diagnostics, `identify` highlighting, `quasi` treatment and group-time plots, export additions | about 85 |
| With their method's release | Result plots in section 7 for 1.3 to 1.10 and the unscheduled roadmap methods, and layers that wait for `spline`, `quantile_reg` or `spatial` | about 85 |
| After 1.11 | Dot plots, hulls, nested facets, magnifiers, slopegraphs, waffles, treemaps, mosaics, alluvials, UpSet, calendars, ternary and perspective coordinates, EPS, TikZ, interactive HTML, animation | about 27 |
| Left out | Section 13 | 13 |

1.2 is already the plot-bearing release (smooth, violin, aggregate, arrange, computed) and carries the difference-in-differences methods, whose event-study and group-time plots need several of these layers. If the grammar extensions make 1.2 too heavy, split them out as a release of their own between 1.1 and 1.2. That renumbers the later releases, so it is listed as a decision.

---

## 15. Decisions to add to the register

| # | Decision | Recommendation | Deadline |
| --- | --- | --- | --- |
| P1 | The scene's version 1 contents (F1 to F4) | As section 4 | 0.7.0 |
| P2 | Scale limits as zoom (F5) | Zoom; filtering is `where` | 0.7.0 |
| P3 | `RichText` reserved in `core` (F6) | Reserve now; `rich()` after 1.0 | 1.0 freeze |
| P4 | Notes on exported figures (F7) | Off by default, `notes: true` | 0.7.0 |
| P5 | Result-plot convention and prediction standard errors in the estimator interface (F12) | `_plot` methods; `predict(se: true)` | 0.4.0 for the interface, 0.7.0 for plots |
| P6 | Four diagnostic and prediction plots in 1.0 | Adopt | With #40 (specification freeze) |
| P7 | Significance brackets (`.brackets()`) | Offer, with adjusted p-values and effect sizes always printed | 1.2 |
| P8 | A separate plot release between 1.1 and 1.2 | Only if 1.2 proves too large | 1.1 |
| P9 | Names with two homes (`violin`, `ecdf`, `ridges`, `beeswarm`, `raincloud`, `qq`) added to rule 6 | Adopt | 1.2 |
| P10 | Project font files in themes | Adopt, hashed as inputs | 1.2 |
| P11 | Exported scene files (`.zplot`) | Adopt as a versioned format | 1.2 |
| P12 | Self-contained interactive HTML export | Defer; decide with Zemp web hosting | After 1.11 |
| P13 | Animation and its export formats | GIF and animated SVG, no MP4 | After 1.11 |
| P14 | Diagrams, flowcharts and SEM path diagrams | The ten decisions D1 to D10 of zeff_diagrams.md, among them reserving `diagram` before the 1.0 freeze and amending F1 with curves, text boxes and element names | 0.7.0 to 1.3 |
