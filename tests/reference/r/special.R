# R's values of the special functions on the cases of tests/reference/special.py (release 0.3.0,
# step 2): gamma, lgamma, beta and lbeta, factorial and choose, the normal tails and quantiles,
# and the incomplete gamma and beta functions with their inverses through pgamma, qgamma, pbeta
# and qbeta with unit rate and both tails. R is the main reference (testing handbook, section
# 1.1); the exact decimal reference of special.py stays the strict one. Run with
# `Rscript tests/reference/r/special.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

cases <- read.csv(file.path(here(), "..", "data", "special_cases.csv"), colClasses = "character")
hex <- function(column, kind) as.numeric(cases[[column]][cases$kind == kind])
dec <- function(x) sprintf("%.17g", x)

gamma_grid <- hex("a", "gamma")
beta_a <- hex("a", "beta"); beta_b <- hex("b", "beta")
erf_grid <- hex("a", "erf")
normal_grid <- hex("a", "normal")
quantile_p <- hex("p", "normal_quantile")
ig_a <- hex("a", "incomplete_gamma"); ig_x <- hex("x", "incomplete_gamma")
igi_a <- hex("a", "incomplete_gamma_inverse"); igi_p <- hex("p", "incomplete_gamma_inverse")
ib_a <- hex("a", "incomplete_beta"); ib_b <- hex("b", "incomplete_beta"); ib_x <- hex("x", "incomplete_beta")
ibi_a <- hex("a", "incomplete_beta_inverse"); ibi_b <- hex("b", "incomplete_beta_inverse"); ibi_p <- hex("p", "incomplete_beta_inverse")

# R has no erf: erf(x) = 2 pnorm(x sqrt(2)) - 1 and erfc(x) = 2 pnorm(x sqrt(2), lower.tail = FALSE).
erf <- function(x) 2 * pnorm(x * sqrt(2)) - 1
erfc <- function(x) 2 * pnorm(x * sqrt(2), lower.tail = FALSE)

triples <- function(...) {
  columns <- list(...)
  lapply(seq_along(columns[[1]]), function(i) lapply(columns, function(column) column[i]))
}

out <- list(
  reference = reference_block(),
  gamma_grid = as.list(dec(gamma_grid)),
  log_gamma = as.list(num(suppressWarnings(lgamma(gamma_grid)))),
  gamma = as.list(num(suppressWarnings(gamma(gamma_grid)))),
  factorial = lapply(c(0L, 1L, 2L, 5L, 10L, 18L, 19L, 20L, 22L, 25L, 50L, 100L, 150L, 170L),
                     function(n) list(n, num(factorial(n)))),
  choose = lapply(list(c(0, 0), c(5, 2), c(10, 3), c(30, 15), c(50, 25), c(60, 30), c(100, 50),
                       c(120, 60), c(200, 100), c(1000, 3), c(1000, 500), c(1029, 514), c(1030, 515),
                       c(1e12, 2), c(1e9, 7)),
                  function(nk) list(dec(nk[1]), dec(nk[2]), num(choose(nk[1], nk[2])))),
  beta = triples(dec(beta_a), dec(beta_b), num(lbeta(beta_a, beta_b)), num(beta(beta_a, beta_b))),
  erf_grid = as.list(dec(erf_grid)),
  erf = as.list(num(erf(erf_grid))),
  erfc = as.list(num(erfc(erf_grid))),
  normal_grid = as.list(dec(normal_grid)),
  normal_lower = as.list(num(pnorm(normal_grid))),
  normal_upper = as.list(num(pnorm(normal_grid, lower.tail = FALSE))),
  normal_quantile = triples(dec(quantile_p), num(qnorm(quantile_p)), num(qnorm(quantile_p, lower.tail = FALSE))),
  incomplete_gamma = triples(dec(ig_a), dec(ig_x), num(pgamma(ig_x, ig_a)), num(pgamma(ig_x, ig_a, lower.tail = FALSE))),
  incomplete_gamma_inverse = triples(dec(igi_a), dec(igi_p), num(qgamma(igi_p, igi_a)), num(qgamma(igi_p, igi_a, lower.tail = FALSE))),
  incomplete_beta = triples(dec(ib_a), dec(ib_b), dec(ib_x), num(pbeta(ib_x, ib_a, ib_b)), num(pbeta(ib_x, ib_a, ib_b, lower.tail = FALSE))),
  incomplete_beta_inverse = triples(dec(ibi_a), dec(ibi_b), dec(ibi_p), num(qbeta(ibi_p, ibi_a, ibi_b)), num(qbeta(ibi_p, ibi_a, ibi_b, lower.tail = FALSE)))
)
write_json(out, file.path(here(), "special.json"))
