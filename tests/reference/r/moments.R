# The four forms of sample skewness and kurtosis of Joanes and Gill (1998) written out in base R
# (release 0.5.0, step 20): g1 = m3 / m2^(3/2), G1 = g1 * sqrt(n(n - 1)) / (n - 2), g2 = m4 / m2^2
# - 3 and G2 = ((n + 1) g2 + 6)(n - 1) / ((n - 2)(n - 3)), with the moments about the mean over
# the values present, on skewed samples drawn under a seed, a sample with values to 1e12, and the
# small samples of the edge cases; written with 17 significant digits and compared within a
# relative 1e-12. g1 and g2 + 3 are what moments::skewness() and moments::kurtosis() and Stata's
# `summarize, detail` report; G1 and G2 are SPSS's and Excel's (SKEW, KURT). Run with
# `Rscript tests/reference/r/moments.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

dec <- function(x) ifelse(is.na(x), "NA", ifelse(is.infinite(x), ifelse(x > 0, "Inf", "-Inf"), sprintf("%.17g", x)))
forms <- function(x) {
  x <- x[!is.na(x)]
  n <- length(x)
  m <- mean(x)
  m2 <- mean((x - m)^2); m3 <- mean((x - m)^3); m4 <- mean((x - m)^4)
  g1 <- m3 / m2^1.5
  g2 <- m4 / m2^2 - 3
  G1 <- if (n >= 3) g1 * sqrt(n * (n - 1)) / (n - 2) else NA
  G2 <- if (n >= 4) ((n + 1) * g2 + 6) * (n - 1) / ((n - 2) * (n - 3)) else NA
  list(n = n, g1 = dec(g1), G1 = dec(G1), g2 = dec(g2), G2 = dec(G2),
       g2_plus_3 = dec(g2 + 3), G2_plus_3 = dec(G2 + 3))
}
set.seed(20261031)
samples <- list(
  c(2, 4, 4, 4, 5, 5, 7, 9, NA),
  round(rexp(50, rate = 0.5), 6),
  round(rlnorm(200, 1, 0.8), 6),
  round(rnorm(120, 10, 2), 6),
  c(1e12, 2e12, 2.5e12, 1e7, 3e12, 7e12, 1e-8, 5e11),
  c(1, 2, 3, 10),
  c(1, 2, 10),
  c(-5, 0, 5, 0, -5, 5)
)
cases <- lapply(samples, function(x) c(list(items = as.list(dec(x))), forms(x)))
write_json(list(reference = reference_block(), cases = cases), file.path(here(), "moments.json"))
