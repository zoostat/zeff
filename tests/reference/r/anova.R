# R's answers for anova (release 0.7.0, steps 16 and 17). The one-way form: oneway.test() for Welch's
# F and aov() for the classical F in R 4.6.0 (oneway.test(var.equal = TRUE) agrees where it runs). A fixed series split into 2 to 6 groups of unequal
# sizes, each group scaled and shifted so that the variances differ; ties (rounded values); a
# group of one observation, where Welch's test is undefined and the classical one is not; NA in
# the variable and in the grouping variable; text groups; a large sample; groups far apart, so
# that the p-value is tiny; and two groups, where Welch's F is the square of Welch's t. The
# statistics, degrees of freedom, p-values, sums of squares and eta squared are held to a relative
# 1e-9. Run with `Rscript tests/reference/r/anova.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
options(warn = -1)

series <- function(n, digits = 6) round(3 * sin(seq_len(n) * 1.3) + 0.4 + seq_len(n) / 17, digits)

one_way <- function(x, g) {
  ok <- complete.cases(x, g)
  xf <- x[ok]; gf <- factor(g[ok])
  welch <- tryCatch(oneway.test(xf ~ gf, var.equal = FALSE), error = function(e) NULL)
  # The classical F from aov(), which admits a group of one where oneway.test() does not; where
  # both run they agree to the last digit.
  a <- anova(lm(xf ~ gf))
  classical <- list(statistic = a[1, "F value"], parameter = c(a[1, "Df"], a[2, "Df"]), p.value = a[1, "Pr(>F)"])
  if (all(table(gf) >= 2)) {
    o <- oneway.test(xf ~ gf, var.equal = TRUE)
    stopifnot(abs(unname(o$statistic) - classical$statistic) <= 1e-12 * classical$statistic)
  }
  list(x = as.list(num(x)),
       g = if (is.character(g)) as.list(ifelse(is.na(g), "NA", g)) else as.list(ifelse(is.na(g), "NA", as.character(g))),
       text_groups = is.character(g),
       welch_f = if (is.null(welch)) "NA" else num(unname(welch$statistic)),
       welch_df1 = if (is.null(welch)) "NA" else num(unname(welch$parameter[1])),
       welch_df2 = if (is.null(welch)) "NA" else num(unname(welch$parameter[2])),
       welch_p = if (is.null(welch)) "NA" else num(welch$p.value),
       f = num(unname(classical$statistic)),
       df1 = num(unname(classical$parameter[1])), df2 = num(unname(classical$parameter[2])),
       p_value = num(classical$p.value),
       ss_between = num(a[1, "Sum Sq"]), ss_within = num(a[2, "Sum Sq"]),
       eta_squared = num(a[1, "Sum Sq"] / sum(a[, "Sum Sq"])),
       n = as.integer(sum(ok)), groups = as.integer(nlevels(gf)))
}

anova_cases <- list()
# Distinct values, k groups of unequal sizes and unequal variances.
for (k in 2:6) {
  n <- 7 * k + 3
  g <- rep(seq_len(k), length.out = n)
  x <- series(n) * (0.5 + 0.4 * g) + 0.3 * g
  anova_cases[[length(anova_cases) + 1]] <- one_way(x, g)
}
# Ties: values rounded to one digit, then to integers.
anova_cases[[length(anova_cases) + 1]] <- one_way(series(24, digits = 1), rep(1:3, each = 8))
anova_cases[[length(anova_cases) + 1]] <- one_way(round(series(30, digits = 0)), rep(1:4, length.out = 30))
# A group of one observation: Welch's test is undefined.
anova_cases[[length(anova_cases) + 1]] <- one_way(series(12), c(rep(1, 6), rep(2, 5), 3))
# NA in the variable and in the groups.
x <- series(20); x[c(3, 11)] <- NA
g <- rep(1:3, length.out = 20); g[c(7, 11)] <- NA
anova_cases[[length(anova_cases) + 1]] <- one_way(x, g)
# Text groups, with ties.
anova_cases[[length(anova_cases) + 1]] <- one_way(series(18, digits = 1), rep(c("north", "south", "east"), each = 6))
# A large sample.
anova_cases[[length(anova_cases) + 1]] <- one_way(series(300, digits = 2), rep(1:5, length.out = 300))
# Groups far apart: a tiny p-value in both forms.
g <- rep(1:4, each = 10)
anova_cases[[length(anova_cases) + 1]] <- one_way(series(40) + 6 * g, g)
# Two groups of equal size: Welch's F is the square of Welch's t.
anova_cases[[length(anova_cases) + 1]] <- one_way(series(16) * rep(c(1, 2), each = 8), rep(c(1, 2), each = 8))
# Two groups, the classical F equal to the pooled t squared, printed by t_test with equal_var.
anova_cases[[length(anova_cases) + 1]] <- one_way(series(15), rep(c(1, 2), length.out = 15))

# The factorial form (release 0.7.0, step 17): the sequential table of anova(lm()), type 1, and
# car's Anova(type = 2) and Anova(type = 3) in car 3.1-5, the factors coded by contr.sum, which
# type 3 needs and the other two do not depend on. The factors are laid out with periods that
# share no divisor, so the cells are unequal; one case is balanced, one has an empty cell (type 3
# is refused, as car refuses it), one has three factors, one a covariate alone and one a covariate
# that interacts with a factor, one no factor at all, one NA in the outcome, a factor and the
# covariate, and two are weighted: frequency weights against the rows written out, analytic
# weights against lm(weights =) with the weights scaled to sum to the rows. The term order of
# each table is R's, by degree. Sums of squares, degrees of freedom, F and p are held to a
# relative 1e-9.
.libPaths(c(file.path(here(), "library"), .libPaths()))
suppressMessages(library(car))

layout <- function(n) {
  i <- seq_len(n)
  data.frame(
    y = round(3 * sin(i * 1.3) + 0.4 + i / 17, 4),
    a = c("p", "p", "q", "p", "r", "q", "p")[1 + (i - 1) %% 7],
    b = c("u", "v", "w", "z", "u", "v", "u", "w", "z", "u", "v")[1 + (i - 1) %% 11],
    c = c("m", "n", "n", "m", "n")[1 + (i - 1) %% 5],
    x = round(2 * cos(i * 0.7) + i / 20, 4),
    z = round(sin(i * 0.31) * 3 + (i %% 4), 4),
    stringsAsFactors = FALSE)
}

table_rows <- function(tab, residual_row) {
  keep <- setdiff(rownames(tab), c("(Intercept)", residual_row))
  lapply(keep, function(term) {
    list(term = gsub(" ", "", term),
         df = as.integer(tab[term, "Df"]),
         ss = num(tab[term, "Sum Sq"]),
         f = num(tab[term, "F value"]),
         p = num(tab[term, "Pr(>F)"]))
  })
}

factorial <- function(d, text, factors, weights = NULL, weight_type = NULL, expand = FALSE) {
  f <- as.formula(text)
  vars <- all.vars(f)
  used <- d
  w <- NULL
  if (!is.null(weights)) {
    ok <- complete.cases(d[, c(vars, weights)]) & d[[weights]] > 0
    used <- d[ok, ]
    if (expand) {
      # Frequency weights: each row written out as many times as its weight.
      used <- used[rep(seq_len(nrow(used)), used[[weights]]), ]
    } else {
      w <- used[[weights]] * nrow(used) / sum(used[[weights]])
    }
  }
  for (name in factors) used[[name]] <- factor(used[[name]])
  contrasts <- setNames(rep(list(contr.sum), length(factors)), factors)
  used$.w <- w
  m <- if (length(factors) == 0) {
    if (is.null(w)) lm(f, used) else lm(f, used, weights = .w)
  } else {
    if (is.null(w)) lm(f, used, contrasts = contrasts) else lm(f, used, weights = .w, contrasts = contrasts)
  }
  one <- anova(m)
  two <- Anova(m, type = 2)
  three <- tryCatch(Anova(m, type = 3), error = function(e) NULL)
  fstat <- summary(m)$fstatistic
  rss <- one["Residuals", "Sum Sq"]
  column <- function(name) {
    v <- d[[name]]
    if (is.character(v)) as.list(ifelse(is.na(v), "NA", v)) else as.list(num(v))
  }
  list(formula = text,
       factors = as.list(factors),
       columns = setNames(lapply(c(vars, weights), column), c(vars, weights)),
       weights = if (is.null(weights)) "NA" else weights,
       weight_type = if (is.null(weight_type)) "NA" else weight_type,
       type1 = table_rows(one, "Residuals"),
       type2 = table_rows(two, "Residuals"),
       type3 = if (is.null(three)) "NA" else table_rows(three, "Residuals"),
       residual_df = num(df.residual(m)),
       residual_ss = num(rss),
       total_ss = num(sum(one[, "Sum Sq"])),
       model_f = num(unname(fstat[1])),
       model_df = as.integer(fstat[2]),
       model_p = num(pf(fstat[1], fstat[2], fstat[3], lower.tail = FALSE)),
       r_squared = num(summary(m)$r.squared),
       n = as.integer(nobs(m)))
}

factorial_cases <- list()
add <- function(case) factorial_cases[[length(factorial_cases) + 1]] <<- case
# Balanced: two factors, three observations in each of the six cells.
balanced <- layout(18)
balanced$a <- rep(c("p", "q"), each = 9)
balanced$b <- rep(rep(c("u", "v", "w"), each = 3), 2)
add(factorial(balanced, "y ~ a * b", c("a", "b")))
# Unbalanced: two factors with their interaction, in both orders, and the main effects alone.
d <- layout(60)
add(factorial(d, "y ~ a * b", c("a", "b")))
add(factorial(d, "y ~ b * a", c("a", "b")))
add(factorial(d, "y ~ a + b", c("a", "b")))
# A covariate beside the factors, and a covariate that interacts with a factor.
add(factorial(d, "y ~ a * b + x", c("a", "b")))
add(factorial(d, "y ~ x + a * b", c("a", "b")))
add(factorial(d, "y ~ a * x", "a"))
add(factorial(d, "y ~ a * x + b", c("a", "b")))
# Three factors with every interaction.
add(factorial(layout(90), "y ~ a * b * c", c("a", "b", "c")))
# An empty cell: no row with a = r and b = z, so type 3 is refused.
empty <- d[!(d$a == "r" & d$b == "z"), ]
add(factorial(empty, "y ~ a * b", c("a", "b")))
# No factor: every row of the table tests a slope.
add(factorial(d, "y ~ x + z", character(0)))
add(factorial(d, "y ~ x * z", character(0)))
# NA in the outcome, in a factor and in the covariate.
holes <- d
holes$y[c(4, 17)] <- NA; holes$a[c(9, 17)] <- NA; holes$x[31] <- NA
add(factorial(holes, "y ~ a * b + x", c("a", "b")))
# Frequency weights, against the rows written out; a zero weight leaves its row out.
weighted <- layout(40)
weighted$w <- c(1, 2, 1, 3, 1, 1, 2, 0, 1, 4)[1 + (seq_len(40) - 1) %% 10]
add(factorial(weighted, "y ~ a * c + x", c("a", "c"), weights = "w", weight_type = "frequency", expand = TRUE))
# Analytic weights, against weighted least squares with the weights scaled to sum to the rows.
weighted$v <- round(0.5 + (seq_len(40) %% 7) / 3, 4)
add(factorial(weighted, "y ~ a * c + x", c("a", "c"), weights = "v", weight_type = "analytic"))

write_json(list(reference = c(reference_block(), list(car = as.character(packageVersion("car")))),
                anova = anova_cases, factorial = factorial_cases),
           file.path(here(), "anova.json"))
