# R's answers for the test of an implied independence on data (release 0.10.0, step 18; open
# decision #158): the partial correlation of two variables given a set of others, its t test
# and Fisher's interval. In base R alone, two ways that share nothing with the engine's:
#
#   r      the correlation of the residuals of lm(x ~ given) and lm(y ~ given);
#   t, p   the row of x in the coefficient table of lm(y ~ x + given), whose t test is the
#          test of the partial correlation, on n - 2 - k degrees of freedom;
#   lower, upper   tanh(atanh(r) -+ qnorm(1 - (1 - level) / 2) / sqrt(n - 3 - k)), by formula;
#   and with nothing given, cor.test()'s own estimate, p value and interval beside them.
#
# k is the number of columns given: one for a number, one less than its levels for a category.
# On the shipped `wages` (the rows of 2019) and `staff`, for the statements of the graphs
# tested on them and others, and on a small frame typed in here. A statement whose given
# variables determine one of the two is held as `determined`, a yes or no and not a number,
# since what is left of such a variable is rounding error and differs by machine.
# Run with `Rscript tests/reference/r/partial.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
options(warn = -1)

root <- normalizePath(file.path(here(), "..", "..", ".."))
shipped <- function(name) read_dataset(file.path(root, "packages", "tier1", "datasets", "data", paste0(name, ".csv")))

one_case <- function(frame, data, x, y, given = character(0), level = 0.95) {
  d <- data[stats::complete.cases(data[, c(x, y, given), drop = FALSE]), c(x, y, given), drop = FALSE]
  for (v in c(x, y)) d[[v]] <- as.numeric(d[[v]])
  n <- nrow(d)
  rest <- function(v) if (length(given)) resid(lm(reformulate(given, v), d)) else d[[v]] - mean(d[[v]])
  rx <- rest(x); ry <- rest(y)
  share <- function(v, r) sum(r^2) / sum((d[[v]] - mean(d[[v]]))^2)
  determined <- share(x, rx) < 1e-10 || share(y, ry) < 1e-10
  k <- if (length(given)) ncol(model.matrix(reformulate(given), d)) - 1 else 0
  out <- list(frame = frame, x = x, y = y, given = as.list(given), level = num(level),
              n = as.integer(n), k = as.integer(k), determined = determined)
  if (determined) return(out)
  r <- cor(rx, ry)
  fit <- summary(lm(reformulate(c(x, given), y), d))$coefficients
  z <- atanh(r); half <- qnorm(1 - (1 - level) / 2) / sqrt(n - 3 - k)
  out$r <- num(r)
  out$t <- num(fit[x, "t value"])
  out$p <- num(fit[x, "Pr(>|t|)"])
  out$df <- as.integer(n - 2 - k)
  out$lower <- num(tanh(z - half))
  out$upper <- num(tanh(z + half))
  if (!length(given)) {
    own <- cor.test(d[[x]], d[[y]], conf.level = level)
    out$cor_test <- list(r = num(unname(own$estimate)), p = num(own$p.value),
                         lower = num(own$conf.int[1]), upper = num(own$conf.int[2]))
  }
  out
}

wages <- shipped("wages")
w19 <- wages[wages$year == 2019, ]
w19$lwage <- log(w19$wage)
staff <- shipped("staff")
small <- data.frame(a = c(2.1, 3.4, 1.9, 5.2, 4.4, 3.8, 2.7, 6.1, 5.5, 4.0, 3.1, 2.2),
                    b = c(1.0, 2.2, 1.4, 3.9, 3.1, 2.4, 2.0, 4.8, 4.1, 2.9, 2.5, 1.1),
                    c = c(5.0, 4.1, 5.5, 2.2, 3.0, 3.6, 4.4, 1.5, 2.1, 3.3, 3.9, 5.1),
                    d = c(0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0),
                    g = c("u", "v", "w", "u", "v", "w", "u", "v", "w", "u", "v", "w"),
                    stringsAsFactors = FALSE)
small$sum <- small$a + 2 * small$b
gaps <- small
gaps$a[c(3, 8)] <- NA
gaps$c[5] <- NA

cases <- list(
  # The graph of open decision #158 on the rows of 2019, and what it could not test.
  one_case("wages2019", w19, "age", "female"),
  one_case("wages2019", w19, "age", "educ"),
  one_case("wages2019", w19, "age", "union"),
  one_case("wages2019", w19, "educ", "union", "female"),
  one_case("wages2019", w19, "exper", "union", "female"),
  one_case("wages2019", w19, "exper", "female", c("age", "educ")),
  one_case("wages2019", w19, "exper", "union", c("age", "educ")),
  one_case("wages2019", w19, "age", "lwage", c("educ", "exper", "female", "union")),
  one_case("wages2019", w19, "age", "union", level = 0.90),
  one_case("wages2019", w19, "educ", "union", c("female", "industry")),
  # The staff of the training study.
  one_case("staff", staff, "age", "tenure", "training"),
  one_case("staff", staff, "offer", "productivity", "training"),
  one_case("staff", staff, "offer", "tenure", c("training", "region")),
  one_case("staff", staff, "distance", "productivity", c("training", "age", "region")),
  one_case("staff", staff, "age", "distance"),
  # A small frame: a number, a yes or no, a category given, rows with a gap, a variable that
  # is the sum of two given, and levels of confidence.
  one_case("small", small, "a", "b"),
  one_case("small", small, "a", "c", "b"),
  one_case("small", small, "a", "d", c("b", "c")),
  one_case("small", small, "a", "c", "g"),
  one_case("small", small, "b", "d", c("g", "c")),
  one_case("small", small, "sum", "c", c("a", "b")),
  one_case("small", small, "c", "sum", c("a", "b")),
  one_case("small", small, "a", "b", level = 0.99),
  one_case("gaps", gaps, "a", "b"),
  one_case("gaps", gaps, "a", "c", "b"),
  one_case("gaps", gaps, "b", "d", c("a", "c")))

write_json(list(reference = reference_block(), cases = cases), file.path(here(), "partial.json"))
