# R's answers for t_test (release 0.7.0, step 9): t.test() in its three forms on the shipped
# wages (lwage = log(wage), the two groups of female and of union) and on R's own sleep data
# (paired, as the shipped sleep.csv in wide form), with one-sided alternatives and other levels,
# and one frequency-weighted case by replicating rows. The effect sizes are computed here in base
# R: Cohen's d on the pooled standard deviation, Hedges' g = d (1 - 3 / (4 (n1 + n2) - 9)), and
# the large-sample interval of Hedges and Olkin (1985). Numbers are written with 15 significant
# digits and the engine is held to a relative 1e-6 (the closed-form class). Run with
# `Rscript tests/reference/r/t_test.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

root <- normalizePath(file.path(here(), "..", "..", ".."))

case <- function(name, t, extra = list()) {
  ci <- as.vector(t$conf.int)
  c(list(name = name, statistic = num(unname(t$statistic)), df = num(unname(t$parameter)),
         p_value = num(t$p.value), estimate = num(unname(sum(t$estimate * c(1, -1)[seq_along(t$estimate)]))),
         ci_lower = num(ci[1]), ci_upper = num(ci[2])), extra)
}

effect <- function(x1, x2, level, alternative) {
  n1 <- length(x1); n2 <- length(x2)
  sp <- sqrt(((n1 - 1) * var(x1) + (n2 - 1) * var(x2)) / (n1 + n2 - 2))
  d <- (mean(x1) - mean(x2)) / sp
  se <- sqrt((n1 + n2) / (n1 * n2) + d^2 / (2 * (n1 + n2)))
  j <- 1 - 3 / (4 * (n1 + n2) - 9)
  bounds <- function(est, se) {
    if (alternative == "two.sided") { q <- qnorm(1 - (1 - level) / 2); c(est - q * se, est + q * se) }
    else if (alternative == "less") c(-Inf, est + qnorm(level) * se)
    else c(est - qnorm(level) * se, Inf)
  }
  list(n1 = n1, n2 = n2, mean1 = num(mean(x1)), mean2 = num(mean(x2)), sd1 = num(sd(x1)), sd2 = num(sd(x2)),
       d = num(d), d_ci = as.list(num(bounds(d, se))), g = num(d * j), g_ci = as.list(num(bounds(d * j, se * j))))
}

wages <- read_dataset(file.path(root, "packages", "tier1", "datasets", "data", "wages.csv"))
wages$lwage <- log(wages$wage)
w <- wages[!is.na(wages$lwage), ]
by_female <- split(w$lwage, w$female)   # 0 = male first, as Zeff orders the groups by code
by_union <- split(w$lwage, w$union)

two <- function(name, groups, var.equal, level = 0.95, alternative = "two.sided") {
  t <- t.test(groups[[1]], groups[[2]], var.equal = var.equal, conf.level = level, alternative = alternative)
  case(name, t, effect(groups[[1]], groups[[2]], level, alternative))
}
one <- function(name, x, mu, level = 0.95, alternative = "two.sided") {
  t <- t.test(x, mu = mu, conf.level = level, alternative = alternative)
  case(name, t, list(n = length(x), mean = num(mean(x)), sd = num(sd(x))))
}

wages_cases <- list(
  two("welch_female", by_female, FALSE),
  two("welch_female_less", by_female, FALSE, alternative = "less"),
  two("pooled_union_99", by_union, TRUE, level = 0.99),
  two("welch_union_greater_90", by_union, FALSE, level = 0.90, alternative = "greater"),
  one("mu3_greater", w$lwage, 3, alternative = "greater"),
  one("mu3_90", w$lwage, 3, level = 0.90),
  one("mu0", w$lwage, 0))

a <- sleep$extra[sleep$group == 1]
b <- sleep$extra[sleep$group == 2]
paired <- function(name, level = 0.95, alternative = "two.sided") {
  t <- t.test(a, b, paired = TRUE, conf.level = level, alternative = alternative)
  case(name, t, list(n = length(a), mean_a = num(mean(a)), mean_b = num(mean(b)), sd_d = num(sd(a - b))))
}
sleep_cases <- list(paired("paired"), paired("paired_less", alternative = "less"), paired("paired_greater_99", 0.99, "greater"))

# Frequency weights replicate rows: x = (1, 2, 4) with weights (2, 1, 3).
x <- rep(c(1, 2, 4), c(2, 1, 3))
weighted_case <- one("frequency_mu1", x, 1)

out <- list(reference = reference_block(),
            sleep_a = as.list(num(a)), sleep_b = as.list(num(b)),
            wages = wages_cases, sleep = sleep_cases, weighted = weighted_case)
write_json(out, file.path(here(), "t_test.json"))
