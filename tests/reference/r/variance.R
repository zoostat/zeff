# R's answers for variance_test (release 0.8.0, step 8j; open decision #97): the tests of equal
# variances across groups in R 4.6.0 with car 3.1-5. Levene's test, car::leveneTest(), centred on
# the median (Brown and Forsythe) and on the mean; Bartlett's test, bartlett.test(), where every
# group has two values or more; and for two groups the ratio of the variances with its 95%
# interval, var.test(), the first group over the second. The cases: a fixed series split into 2
# to 6 groups of unequal sizes, the groups' spreads made to differ; ties; a group of one value;
# NA in the variable and in the grouping variable; text groups; a group with no spread; and the
# two shipped datasets, wages by union, by female and by industry in 2019, and the size of the
# firms by industry. Groups are taken in the order of the sorted values, as factor() gives them
# in the C locale. Statistics and p-values are held to a relative 1e-10. Run with
# `Rscript tests/reference/r/variance.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
suppressMessages(library(car))
options(warn = -1)

series <- function(n, digits = 6) round(3 * sin(seq_len(n) * 1.3) + 0.4 + seq_len(n) / 17, digits)

vt <- function(x, g) {
  ok <- complete.cases(x, g)
  xs <- x[ok]
  gs <- factor(g[ok])
  k <- nlevels(gs)
  sizes <- as.integer(table(gs))
  out <- list(x = as.list(num(x)),
              g = as.list(ifelse(is.na(g), "NA", as.character(g))),
              text_groups = is.character(g),
              n = as.integer(sum(ok)), groups = as.integer(k),
              sizes = as.list(sizes),
              variances = as.list(num(as.numeric(tapply(xs, gs, var)))))
  lev <- function(center) {
    t <- leveneTest(xs, gs, center = center)
    list(statistic = num(t[["F value"]][1]), df1 = as.integer(t$Df[1]), df2 = as.integer(t$Df[2]),
         p_value = num(t[["Pr(>F)"]][1]))
  }
  out$levene_median <- lev(median)
  out$levene_mean <- lev(mean)
  if (all(sizes >= 2)) {
    b <- bartlett.test(xs, gs)
    out$bartlett <- list(statistic = num(unname(b$statistic)), df = as.integer(unname(b$parameter)),
                         p_value = num(b$p.value))
    if (k == 2) {
      f <- var.test(xs[gs == levels(gs)[1]], xs[gs == levels(gs)[2]])
      out$f <- list(statistic = num(unname(f$statistic)), df1 = as.integer(unname(f$parameter[1])),
                    df2 = as.integer(unname(f$parameter[2])), p_value = num(f$p.value),
                    ratio = num(unname(f$estimate)), ci_low = num(f$conf.int[1]), ci_high = num(f$conf.int[2]))
    }
  }
  out
}

cases <- list()
# Distinct values, k groups of unequal sizes, the spread growing with the group.
for (k in 2:6) {
  n <- 7 * k + 3
  g <- rep(seq_len(k), length.out = n)
  cases[[length(cases) + 1]] <- vt(round(series(n) * (1 + g / 3), 6), g)
}
# Two groups of equal spread, and two of very different spread.
cases[[length(cases) + 1]] <- vt(series(40), rep(1:2, each = 20))
cases[[length(cases) + 1]] <- vt(round(series(31) * c(rep(1, 16), rep(9, 15)), 6), c(rep(1, 16), rep(2, 15)))
# Heavy ties: values rounded to one digit, then to integers.
cases[[length(cases) + 1]] <- vt(series(24, digits = 1), rep(1:3, each = 8))
cases[[length(cases) + 1]] <- vt(round(series(30, digits = 0)), rep(1:4, length.out = 30))
# A group of one value: Levene's test stands, Bartlett's and the ratio do not.
cases[[length(cases) + 1]] <- vt(series(12), c(rep(1, 6), rep(2, 5), 3))
# NA in the variable and in the groups.
x <- series(20); x[c(3, 11)] <- NA
g <- rep(1:3, length.out = 20); g[c(7, 11)] <- NA
cases[[length(cases) + 1]] <- vt(x, g)
# Text groups.
cases[[length(cases) + 1]] <- vt(series(18, digits = 1), rep(c("north", "south", "east"), each = 6))
cases[[length(cases) + 1]] <- vt(round(series(22) * rep(c(1, 4), each = 11), 6), rep(c("treated", "control"), each = 11))
# Larger groups.
cases[[length(cases) + 1]] <- vt(series(300, digits = 2), rep(1:5, length.out = 300))

data_dir <- file.path(here(), "..", "..", "..", "packages", "tier1", "datasets", "data")
wages <- read_dataset(file.path(data_dir, "wages.csv"))
wages <- wages[wages$year == 2019, ]
firms <- read_dataset(file.path(data_dir, "firms.csv"))
datasets <- list(
  wage_by_union = vt(wages$wage, wages$union),
  wage_by_female = vt(wages$wage, wages$female),
  wage_by_industry = vt(wages$wage, wages$industry),
  size_by_industry = vt(as.numeric(firms$size), firms$industry)
)
# The datasets are read by the engine itself, so their values are not repeated here.
for (name in names(datasets)) { datasets[[name]]$x <- NULL; datasets[[name]]$g <- NULL }

write_json(list(reference = reference_block(), cases = cases, datasets = datasets), file.path(here(), "variance.json"))
