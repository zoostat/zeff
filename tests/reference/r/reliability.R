# R's answers for reliability (release 0.10.0, step 10): Cronbach's alpha of a set of items, the
# alpha of the standardized items, the average correlation between items, each item's
# correlation with the sum of the others and the alpha of the scale without it, Feldt's interval
# and Duhachek and Iacobucci's standard error with its interval. Every value is written twice:
# from the formulas, in base R, and from psych::alpha(), the reference the specification names,
# with its own reading of each (raw_alpha, std.alpha, average_r, ase, r.drop, alpha.drop,
# feldt). The two agree to the last digits, and the engine is held to both.
#
# On the shipped survey `engagement` (six items from 1 to 5, the fourth worded in reverse, rows
# with a missing answer left out), as it is written and with the fourth item reversed; and on
# small scales typed in here: five items, two items, yes-or-no items (where alpha is KR-20),
# and items that run against one another, whose alpha is negative. A reversed item is the
# negative of itself: alpha and every figure here depend on the items' covariances alone, so
# no end of the scale is needed. Run with `Rscript tests/reference/r/reliability.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
.libPaths(c(file.path(here(), "library"), .libPaths()))
options(warn = -1)
suppressPackageStartupMessages(library(psych))

root <- normalizePath(file.path(here(), "..", "..", ".."))

alpha_of <- function(C) {
  k <- ncol(C)
  k / (k - 1) * (1 - sum(diag(C)) / sum(C))
}

by_formula <- function(x, level) {
  n <- nrow(x); k <- ncol(x)
  C <- cov(x); R <- cor(x)
  figures <- function(M) {
    rest <- vapply(seq_len(k), function(j) {
      others <- sum(M[j, -j])
      others / sqrt(M[j, j] * (sum(M) - 2 * sum(M[j, ]) + M[j, j]))
    }, 0)
    dropped <- vapply(seq_len(k), function(j) if (k > 2) alpha_of(M[-j, -j, drop = FALSE]) else NA_real_, 0)
    a <- alpha_of(M)
    g <- 1 - level
    feldt <- 1 - (1 - a) * qf(c(1 - g / 2, g / 2), n - 1, (n - 1) * (k - 1))
    Q <- (2 * k^2 / ((k - 1)^2 * sum(M)^3)) *
      (sum(M) * (sum(diag(M %*% M)) + sum(diag(M))^2) - 2 * sum(diag(M)) * sum(M %*% M))
    ase <- sqrt(Q / n)
    z <- qnorm(1 - g / 2)
    list(alpha = num(a), item_rest = as.list(num(rest)), alpha_if_deleted = as.list(num(dropped)),
         feldt = as.list(num(feldt)), std_error = num(ase), duhachek = as.list(num(c(a - z * ase, a + z * ase))))
  }
  list(raw = figures(C), standardized = figures(R),
       average_r = num(mean(R[lower.tri(R)])),
       mean = as.list(num(unname(colMeans(x)))), sd = as.list(num(unname(apply(x, 2, sd)))))
}

by_psych <- function(x, level) {
  a <- suppressMessages(psych::alpha(as.data.frame(x), check.keys = FALSE, warnings = FALSE))
  list(raw_alpha = num(a$total$raw_alpha), std_alpha = num(a$total$std.alpha),
       average_r = num(a$total$average_r), ase = num(a$total$ase),
       r_drop = as.list(num(a$item.stats$r.drop)),
       alpha_drop = as.list(num(if (ncol(x) > 2) a$alpha.drop$raw_alpha else rep(NA_real_, ncol(x)))),
       # psych's interval is at 0.95; at another level the formula alone stands.
       feldt = as.list(num(if (level == 0.95) c(a$feldt$lower.ci[[1]], a$feldt$upper.ci[[1]]) else c(NA_real_, NA_real_))))
}

scale_case <- function(name, x, reversed = character(0), level = 0.95) {
  x <- as.matrix(x)
  storage.mode(x) <- "double"
  rows <- nrow(x)
  x <- x[stats::complete.cases(x), , drop = FALSE]
  scored <- x
  for (item in reversed) scored[, item] <- -scored[, item]
  list(name = name, items = as.list(colnames(x)), reversed = as.list(reversed),
       level = num(level), rows = as.integer(rows), n = as.integer(nrow(x)),
       formula = by_formula(scored, level), psych = by_psych(scored, level))
}

survey <- read_dataset(file.path(root, "packages", "tier1", "datasets", "data", "engagement.csv"))
items <- survey[, paste0("eng", 1:6)]

five <- cbind(a = c(4, 2, 5, 3, 4, 1, 5, 2, 3, 4, 2, 5, 3, 1, 4),
              b = c(5, 2, 4, 3, 4, 2, 5, 1, 3, 5, 2, 4, 2, 1, 3),
              c = c(4, 3, 5, 2, 5, 1, 4, 2, 4, 4, 1, 5, 3, 2, 4),
              d = c(3, 2, 4, 3, 3, 2, 5, 3, 2, 4, 3, 4, 3, 1, 5),
              e = c(5, 1, 4, 4, 4, 1, 3, 2, 3, 5, 2, 5, 4, 2, 3))
yes_no <- cbind(q1 = c(1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 0, 0, 1, 1),
                q2 = c(1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1, 1, 0, 1, 1, 0),
                q3 = c(1, 1, 1, 1, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0, 1, 1),
                q4 = c(0, 0, 1, 1, 0, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1, 1))
against <- cbind(u = c(1, 2, 3, 4, 5, 2, 4, 3, 5, 1),
                 v = c(5, 4, 3, 2, 1, 4, 3, 3, 1, 4),
                 w = c(2, 3, 3, 5, 4, 1, 4, 2, 5, 2))
gaps <- five
gaps[c(2, 9), "b"] <- NA
gaps[13, "e"] <- NA

cases <- list(
  scale_case("engagement_reversed", items, reversed = "eng4"),
  scale_case("engagement_as_written", items),
  scale_case("engagement_level_90", items, reversed = "eng4", level = 0.90),
  scale_case("five_items", five),
  scale_case("two_items", five[, c("a", "b")]),
  scale_case("yes_or_no", yes_no),
  scale_case("against", against),
  scale_case("against_reversed", against, reversed = "v"),
  scale_case("with_gaps", gaps))

write_json(list(reference = reference_block(), cases = cases), file.path(here(), "reliability.json"))
