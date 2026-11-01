# R's quantile groups and winsorised values (release 0.6.0, step 10): quantile_group(x, n:, type:)
# is the group 1 to n whose cut points are the sample quantiles at i/n under the nine definitions,
# a value equal to a cut point falling in the group below it (right-closed, as R's cut with
# include.lowest = TRUE), which base R's findInterval(x, cuts, left.open = TRUE) + 1 computes for
# any cut points and as.integer(cut(x, c(min, cuts, max), include.lowest = TRUE)) computes when
# the cut points are distinct; the script checks the two agree wherever cut applies. When two cut
# points coincide (fewer distinct values than groups) the groups between them are empty and the
# numbering is kept, as Stata's xtile keeps it. winsorize(x, at:) is pmin(pmax(x, q(at)), q(1 - at))
# with the quantile of type 7. The values are written with 17 significant digits and the engine
# is held to them exactly. Run with `Rscript tests/reference/r/quantile_group.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

dec <- function(x) ifelse(is.na(x), "NA", ifelse(is.infinite(x), ifelse(x > 0, "Inf", "-Inf"), sprintf("%.17g", x)))
int_or_na <- function(x) ifelse(is.na(x), "NA", as.character(x))

set.seed(20261103)
vectors <- list(
  c(5, 1, 9, 3, 7, NA, 2, 8),
  round(rnorm(60) * 100, 2),
  c(2.5, 3.5, 1e12, 7.25, -3, 0.5, 1e7, 12, NA, -1e12, 0.25),
  c(1, 1, 1, 1, 2),
  c(4, 4, 4, 4),
  c(5),
  sample(1:6, 40, replace = TRUE),
  c(NA, 3, 1, NA, 2)
)

groups_of <- function(x, n, type) {
  present <- x[!is.na(x)]
  if (length(present) == 0) return(rep(NA, length(x)))
  cuts <- if (n > 1) quantile(present, probs = (1:(n - 1)) / n, type = type, names = FALSE) else numeric(0)
  g <- findInterval(x, cuts, left.open = TRUE) + 1
  if (n > 1 && !anyDuplicated(c(min(present), cuts, max(present)))) {
    by_cut <- as.integer(cut(x, c(min(present), cuts, max(present)), include.lowest = TRUE))
    stopifnot(identical(as.integer(g[!is.na(x)]), by_cut[!is.na(x)]))
  }
  g
}

group_case <- function(x) {
  out <- list()
  for (n in c(1L, 2L, 3L, 4L, 5L, 10L)) {
    for (type in 1:9) {
      out[[length(out) + 1]] <- list(n = n, type = type, groups = as.list(int_or_na(groups_of(x, n, type))))
    }
  }
  out
}

winsor_case <- function(x) {
  present <- x[!is.na(x)]
  lapply(c(0.01, 0.05, 0.1, 0.25, 0, 0.5), function(at) {
    w <- if (length(present) == 0) rep(NA, length(x)) else {
      low <- quantile(present, at, type = 7, names = FALSE)
      high <- quantile(present, 1 - at, type = 7, names = FALSE)
      pmin(pmax(x, low), high)
    }
    list(at = dec(at), values = as.list(dec(w)))
  })
}

cases <- lapply(vectors, function(x) list(items = as.list(dec(x)), groups = group_case(x), winsorized = winsor_case(x)))
write_json(list(reference = reference_block(), cases = cases), file.path(here(), "quantile_group.json"))
