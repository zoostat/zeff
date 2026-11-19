// a Zeff export: place it in a document with `#include`
// t_test · one-sample · wage
#table(
  columns: 1,
  align: left,
  stroke: none,
  table.hline(stroke: 0.8pt),
  "  H0: the population mean of wage equals 20            μ = 20",
  "  H1: the population mean of wage differs from 20      μ ≠ 20",
  "",
  "N = 18,201   mean = 27.370   sd = 14.027",
  "difference (mean − 20) = 7.370   95% CI [7.166, 7.573]",
  "t = 70.88   df = 18,200   p < 0.001",
  "decision at α = 0.05: reject H0 (p < 0.001)",
  table.hline(stroke: 0.8pt),
)
