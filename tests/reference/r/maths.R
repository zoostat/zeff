# R's values of the maths kernel's functions on the grid of tests/reference/maths.py (release
# 0.2.0, step 10). R is the main reference of the testing handbook (section 1.1); the exact
# decimal reference of maths.py stays the strict one. Run with `Rscript tests/reference/r/maths.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

# The inputs come from maths.py as exact hexadecimal floats (see there: R's reading of a decimal
# such as 1e300 is a few bits off, and R reads a hexadecimal float exactly).
inputs <- read.csv(file.path(here(), "..", "data", "maths_inputs.csv"), colClasses = "character")
exact <- function(set) as.numeric(inputs$hex[inputs$set == set])
decimal <- function(set) inputs$decimal[inputs$set == set]
grid <- exact("grid")

# Zeff's answers at the edges (Language Reference, section 4.1): a logarithm of zero is -Inf and
# an argument outside the domain gives NA, which is what R gives too (NaN with a warning).
functions <- list(
  exp = exp, log = log, log2 = log2, log10 = log10, log1p = log1p, expm1 = expm1,
  sin = sin, cos = cos, tan = tan, asin = asin, acos = acos, atan = atan, sqrt = sqrt
)
values <- lapply(functions, function(f) num(suppressWarnings(f(grid))))

atan2_pairs <- list()
for (y in c(0.0, 1.0, -1.0, 2.5)) for (x in c(0.0, 1.0, -1.0, -0.5)) {
  atan2_pairs[[length(atan2_pairs) + 1]] <- list(num(y), num(x), num(atan2(y, x)))
}

pow_pairs <- list()
for (x in c(2.0, 0.5, 10.0, -2.0, 1.5, 0.0)) for (y in c(0.0, 1.0, 2.0, 3.0, 0.5, -1.0, 10.0, -0.5)) {
  pow_pairs[[length(pow_pairs) + 1]] <- list(num(x), num(y), num(suppressWarnings(x^y)))
}

# R's round() (R >= 4.0.0 chooses, between the two candidates, the one whose decimal reading is
# nearer to x) and signif(); the comparison test lists where this departs from the exact rule.
to_round <- exact("round")
rounding <- list()
for (i in seq_along(to_round)) for (digits in c(0L, 1L, 2L, 3L, -1L, -2L)) {
  rounding[[length(rounding) + 1]] <- list(decimal("round")[i], digits, num(round(to_round[i], digits)))
}
significant <- list()
to_signif <- exact("significant")
for (i in seq_along(to_signif)) for (sig in c(1L, 2L, 3L, 6L)) {
  significant[[length(significant) + 1]] <- list(decimal("significant")[i], sig, num(signif(to_signif[i], sig)))
}

# Floored division and remainder: R's %/% and %% have the sign of the divisor, as Zeff's div and
# mod do. R has no 64-bit integers, so the two extreme values of maths.py are left out.
divisions <- list()
for (a in c(7, -7, 6, -6, 0, 1, -1, 100, -100)) for (b in c(2, -2, 3, -3, 7, -7, 1, -1, 10)) {
  divisions[[length(divisions) + 1]] <- list(as.integer(a), as.integer(b), as.integer(a %/% b), as.integer(a %% b))
}
float_divisions <- list()
for (a in c(7.5, -7.5, 6.0, -6.0, 0.0, 1e10, -1e10, 0.3)) for (b in c(2.0, -2.0, 3.0, 0.1, -0.1, 7.0)) {
  float_divisions[[length(float_divisions) + 1]] <- list(num(a), num(b), num(a %/% b), num(a %% b))
}

out <- list(
  reference = reference_block(),
  grid = as.list(decimal("grid")),
  functions = lapply(values, as.list),
  atan2 = atan2_pairs,
  pow = pow_pairs,
  round = rounding,
  round_significant = significant,
  div_mod_int = divisions,
  div_mod_float = float_divisions
)
write_json(out, file.path(here(), "maths.json"))
