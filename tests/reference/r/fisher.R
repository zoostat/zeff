# R's answers for fisher_test (release 0.7.0, step 13): fisher.test() on 2 x 2 tables (the exact
# p-value for the three alternatives, the conditional maximum-likelihood estimate of the odds
# ratio and its interval) and on larger tables (the p-value alone). R solves for the estimate and
# the interval with uniroot() at its default tolerance, about 1e-4 on the root, so each 2 x 2 case
# also carries the same quantities re-solved here to 1e-12 with R's own formulas ("tight"); the
# engine is held to the tight values within a relative 1e-9 and to R's within 1e-3, and to R's
# p-values within 1e-12. On the shipped wages (union, female) and on small tables including
# empty cells, a row of zeros, and a symmetric table whose outcomes tie in probability. Run with
# `Rscript tests/reference/r/fisher.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
options(warn = -1)

root <- normalizePath(file.path(here(), "..", "..", ".."))

# R's conditional distribution of the first cell given the margins, as fisher.test() defines it,
# re-solved to a tight tolerance.
tight <- function(tab, alternative, level = 0.95) {
  m <- sum(tab[1, ]); n <- sum(tab[2, ]); k <- sum(tab[, 1]); x <- tab[1, 1]
  lo <- max(0, k - n); hi <- min(k, m); support <- lo:hi
  logdc <- dhyper(support, m, n, k, log = TRUE)
  dnhyper <- function(ncp) { d <- logdc + log(ncp) * support; d <- exp(d - max(d)); d / sum(d) }
  mnhyper <- function(ncp) { if (ncp == 0) return(lo); if (ncp == Inf) return(hi); sum(support * dnhyper(ncp)) }
  pnhyper <- function(q, ncp = 1, upper.tail = FALSE) {
    if (ncp == 1) return(if (upper.tail) phyper(q - 1, m, n, k, lower.tail = FALSE) else phyper(q, m, n, k))
    if (ncp == 0) return(as.numeric(if (upper.tail) q <= lo else q >= lo))
    if (ncp == Inf) return(as.numeric(if (upper.tail) q <= hi else q >= hi))
    sum(dnhyper(ncp)[if (upper.tail) support >= q else support <= q])
  }
  tol <- 1e-12
  mle <- function(x) {
    if (x == lo) return(0); if (x == hi) return(Inf)
    mu <- mnhyper(1)
    if (mu > x) uniroot(function(t) mnhyper(t) - x, c(0, 1), tol = tol)$root
    else if (mu < x) 1 / uniroot(function(t) mnhyper(1 / t) - x, c(.Machine$double.eps, 1), tol = tol)$root
    else 1
  }
  ncp.U <- function(x, alpha) {
    if (x == hi) return(Inf)
    p <- pnhyper(x, 1)
    if (p < alpha) uniroot(function(t) pnhyper(x, t) - alpha, c(0, 1), tol = tol)$root
    else if (p > alpha) 1 / uniroot(function(t) pnhyper(x, 1 / t) - alpha, c(.Machine$double.eps, 1), tol = tol)$root
    else 1
  }
  ncp.L <- function(x, alpha) {
    if (x == lo) return(0)
    p <- pnhyper(x, 1, upper.tail = TRUE)
    if (p > alpha) uniroot(function(t) pnhyper(x, t, upper.tail = TRUE) - alpha, c(0, 1), tol = tol)$root
    else if (p < alpha) 1 / uniroot(function(t) pnhyper(x, 1 / t, upper.tail = TRUE) - alpha, c(.Machine$double.eps, 1), tol = tol)$root
    else 1
  }
  ci <- switch(alternative,
    less = c(0, ncp.U(x, 1 - level)),
    greater = c(ncp.L(x, 1 - level), Inf),
    two.sided = { alpha <- (1 - level) / 2; c(ncp.L(x, alpha), ncp.U(x, alpha)) })
  list(estimate = num(mle(x)), ci = as.list(num(ci)))
}

two <- function(name, cells, alternative = "two.sided") {
  tab <- matrix(cells, nrow = 2, byrow = TRUE)
  t <- fisher.test(tab, alternative = alternative)
  list(name = name, table = as.list(as.integer(cells)), alternative = alternative,
       p_value = num(t$p.value), estimate = num(unname(t$estimate)), ci = as.list(num(as.vector(t$conf.int))),
       tight = tight(tab, alternative))
}

larger <- function(name, rows) {
  tab <- do.call(rbind, rows)
  t <- fisher.test(tab)
  list(name = name, rows = as.integer(nrow(tab)), columns = as.integer(ncol(tab)),
       table = as.list(as.integer(as.vector(t(tab)))), p_value = num(t$p.value))
}

wages <- read_dataset(file.path(root, "packages", "tier1", "datasets", "data", "wages.csv"))
uf <- table(wages$union, wages$female)   # rows union 0, 1; columns female 0 (male), 1 (female)
uf_cells <- as.vector(t(uf))

two_by_two <- list(
  two("basic", c(3, 1, 1, 3)),
  two("basic_less", c(3, 1, 1, 3), "less"),
  two("basic_greater", c(3, 1, 1, 3), "greater"),
  two("zero_cell_infinite", c(5, 0, 3, 4)),
  two("zero_cell_zero", c(0, 5, 5, 0)),
  two("zero_cell_greater", c(5, 0, 3, 4), "greater"),
  two("medium", c(12, 5, 7, 20)),
  two("medium_less", c(12, 5, 7, 20), "less"),
  two("lopsided", c(1, 30, 2, 2)),
  two("large", c(300, 120, 80, 500)),
  two("balanced", c(4, 4, 4, 4)),
  two("wages_union_female", uf_cells),
  two("wages_union_female_greater", uf_cells, "greater"))

bigger <- list(
  larger("three_by_two", list(c(2, 4), c(3, 1), c(1, 5))),
  larger("three_by_three", list(c(2, 4, 0), c(3, 1, 2), c(1, 5, 3))),
  larger("zero_row", list(c(0, 0), c(3, 1), c(1, 5))),
  larger("two_by_four", list(c(3, 1, 4, 1), c(5, 9, 2, 6))),
  larger("four_by_three", list(c(3, 2, 1), c(1, 4, 2), c(2, 2, 5), c(4, 1, 1))),
  larger("five_by_two_sixty", list(c(8, 4), c(6, 6), c(3, 9), c(7, 5), c(5, 7))),
  larger("three_by_four_forty", list(c(5, 3, 2, 1), c(2, 4, 3, 5), c(1, 2, 6, 6))),
  larger("symmetric_ties", list(c(1, 2, 3), c(2, 3, 1), c(3, 1, 2))),
  larger("four_by_four", list(c(4, 1, 2, 0), c(1, 3, 1, 2), c(0, 2, 5, 1), c(2, 0, 1, 4))))

out <- list(reference = reference_block(), two_by_two = two_by_two, larger = bigger)
write_json(out, file.path(here(), "fisher.json"))
