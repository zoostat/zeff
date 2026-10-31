# R's values for the column functions of the data package written in Zeff (release 0.5.0, step
# 16): scale(), scale(scale = FALSE) and dplyr::between()'s definition in base R,
# low <= x & x <= high, on fixed vectors with NA among the values. scale() uses the sample
# standard deviation (divisor n - 1) and skips NA, as Zeff's mean and sd do; the values are
# written with 17 significant digits and compared within a relative 1e-12. Run with
# `Rscript tests/reference/r/scale.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

dec <- function(x) ifelse(is.na(x), "NA", ifelse(is.infinite(x), ifelse(x > 0, "Inf", "-Inf"), sprintf("%.17g", x)))
vectors <- list(
  c(2, 4, 9),
  c(1, 2, 3, NA),
  c(1.5, -2.25, 1e12, 0.5, 1e7),
  c(1e-8, 1e7, -1e12, 0, 2.5, NA, 1e7),
  c(2, 2, 2),
  c(2.5),
  c(0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0)
)
scale_case <- function(x) list(
  items = as.list(dec(x)),
  centered = as.list(dec(as.vector(scale(x, scale = FALSE)))),
  standardized = as.list(dec(suppressWarnings(as.vector(scale(x)))))
)
ranges <- list(c(2, 10), c(0, 0), c(-1e12, 1e12), c(5, 1))
between_case <- function(x, r) list(
  items = as.list(dec(x)), low = dec(r[1]), high = dec(r[2]),
  inside = as.list(ifelse(is.na(x), "NA", ifelse(r[1] <= x & x <= r[2], "true", "false")))
)
between_cases <- list()
for (x in list(c(1, 5, NA, 10), c(0, -0, 1e-300, 2))) for (r in ranges) {
  between_cases[[length(between_cases) + 1]] <- between_case(x, r)
}

write_json(list(
  reference = reference_block(),
  scale = lapply(vectors, scale_case),
  between = between_cases
), file.path(here(), "scale.json"))
