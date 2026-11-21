# Separation of a binary outcome for release 0.9.0, step 6a: Konis's linear program (2007, the
# method of the R packages `safeBinaryRegression` and `detectseparation`), written here in base R
# with a tableau simplex under Bland's rule, since neither package nor an LP solver is installed.
# For the design X of a model and zᵢ = sᵢ xᵢ (sᵢ = +1 for an outcome of 1, −1 for 0), the program
# is max Σ zᵢᵀβ subject to zᵢᵀβ ≥ 0 in every row and −1 ≤ β ≤ 1: a positive maximum is
# separation, and the coefficients not zero at the maximum diverge. The names follow the rule of
# `glm`: the regressors that separate the outcome alone (with the intercept), or else those
# not zero in the program's solution for the whole model. (R's own `glm` says that it converged
# on every quasi-complete case here, with estimates near 20; that is not written to the file,
# since where its iterations stop on separated data is not the same on every machine.)
# Input: tests/datasets/separation.csv (made by whole-number arithmetic) and the shipped wages
# with `union` set to 1 wherever `female` is 1. Run with `Rscript tests/reference/r/separation.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
root <- normalizePath(file.path(here(), "..", "..", ".."))

# max cᵀw subject to A w ≤ b, w ≥ 0, with b ≥ 0, by the tableau simplex from the slack basis;
# Bland's rule (the lowest index enters and, on a tie of the ratio, the lowest basic leaves).
simplex <- function(A, b, c) {
  m <- nrow(A); p <- ncol(A)
  tab <- cbind(A, diag(m), b)
  basis <- p + seq_len(m)
  obj <- c(-c, rep(0, m), 0)
  for (iteration in 1:100000) {
    entering <- which(obj[1:(p + m)] < -1e-12)
    if (length(entering) == 0) break
    q <- entering[1]
    col <- tab[, q]
    rows <- which(col > 1e-12)
    if (length(rows) == 0) stop("unbounded")
    ratios <- tab[rows, p + m + 1] / col[rows]
    best <- min(ratios)
    tied <- rows[abs(ratios - best) <= 1e-15 * max(1, best)]
    r <- tied[which.min(basis[tied])]
    tab[r, ] <- tab[r, ] / tab[r, q]
    for (s in seq_len(m)) if (s != r && tab[s, q] != 0) tab[s, ] <- tab[s, ] - tab[s, q] * tab[r, ]
    obj <- obj - obj[q] * tab[r, ]
    basis[r] <- q
  }
  w <- numeric(p + m)
  w[basis] <- tab[, p + m + 1]
  list(value = obj[p + m + 1], w = w[1:p])
}

# Konis's program on the design X (columns at unit norm) for the outcome y.
konis <- function(X, y) {
  X <- sweep(X, 2, sqrt(colSums(X^2)), "/")
  Z <- X * ifelse(y == 1, 1, -1)
  k <- ncol(Z)
  cz <- colSums(Z)
  # A row written twice is one constraint: the program is the same on the distinct rows, with
  # the objective summed over every row.
  Z <- unique(Z)
  # β = p − q with p, q ≥ 0.
  A <- rbind(cbind(-Z, Z), cbind(diag(k), -diag(k)), cbind(-diag(k), diag(k)))
  b <- c(rep(0, nrow(Z)), rep(1, 2 * k))
  s <- simplex(A, b, c(cz, -cz))
  beta <- s$w[1:k] - s$w[(k + 1):(2 * k)]
  list(separated = s$value > 1e-9 * max(1, abs(cz)), beta = beta)
}

case <- function(name, formula, data) {
  X <- model.matrix(formula, data)
  y <- model.response(model.frame(formula, data))
  whole <- konis(X, y)
  names_out <- character(0)
  if (whole$separated) {
    has_intercept <- colnames(X)[1] == "(Intercept)"
    others <- which(colnames(X) != "(Intercept)")
    alone <- others[vapply(others, function(j) {
      cols <- if (has_intercept) c(1, j) else j
      konis(X[, cols, drop = FALSE], y)$separated
    }, logical(1))]
    chosen <- if (length(alone) > 0) alone else
      others[abs(whole$beta[others]) > 1e-6 * max(abs(whole$beta))]
    names_out <- colnames(X)[chosen]
  }
  list(
    name = name,
    formula = deparse(formula),
    separated = whole$separated,
    diverging = as.list(names_out)
  )
}

d <- read_dataset(file.path(root, "tests", "datasets", "separation.csv"))
d$group <- factor(d$group, levels = c("g1", "g2", "g3"))
wages <- read_dataset(file.path(root, "packages", "tier1", "datasets", "data", "wages.csv"))
wages$union <- ifelse(!is.na(wages$female) & wages$female == 1, 1, wages$union)
wages <- wages[complete.cases(wages[, c("union", "educ", "exper", "female")]), ]

cases <- list(
  case("one_variable", y_one ~ a + x2, d),
  case("combination", y_combo ~ x1 + x2, d),
  case("quasi", y_quasi ~ b + x2, d),
  case("level", y_level ~ group + x1, d),
  case("rare", y_rare ~ x2, d),
  case("overlap", y_quasi ~ x1 + x2, d),
  case("wages_female", union ~ educ + exper + female, wages)
)
# The names as `glm` writes a level of a category: `group=g3` where R writes `groupg3`.
for (i in seq_along(cases)) {
  cases[[i]]$diverging <- lapply(cases[[i]]$diverging, function(v) sub("^group", "group=", v))
}

out <- list(software = paste("R", getRversion()), cases = cases)
writeLines(to_json(out), file.path(here(), "separation.json"))
