# R's correlation and covariance matrices and the p-values of cor.test (release 0.3.0, step 11):
# on the large wages file (six numeric columns, hours with gaps) under pairwise and listwise
# deletion, and on small series without ties, where cor.test gives exact p-values for Spearman and
# Kendall. Run with `Rscript tests/reference/r/association.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

wages <- read_dataset(file.path(here(), "..", "..", "datasets", "wages_large.csv"))
numeric <- wages[, c("id", "wage", "hours", "female", "educ")]
numeric$female <- as.numeric(tolower(wages$female) == "true")

matrix_json <- function(m) lapply(seq_len(nrow(m)), function(i) as.list(num(unname(m[i, ]))))

matrices <- list()
for (method in c("pearson", "spearman", "kendall")) {
  for (use in c("pairwise.complete.obs", "complete.obs")) {
    matrices[[length(matrices) + 1]] <- list(
      method = method, missing = if (use == "complete.obs") "listwise" else "pairwise",
      variables = as.list(names(numeric)),
      correlation = matrix_json(cor(numeric, use = use, method = method))
    )
  }
}
covariances <- list(
  list(missing = "pairwise", variables = as.list(names(numeric)), covariance = matrix_json(cov(numeric, use = "pairwise.complete.obs"))),
  list(missing = "listwise", variables = as.list(names(numeric)), covariance = matrix_json(cov(numeric, use = "complete.obs")))
)

# Pairwise N on the large file.
n_pairs <- matrix_json(crossprod(!is.na(as.matrix(numeric))))

# p-values: pairs of the large file (approximate branches: ties and n >= 1290 for Spearman, ties
# for Kendall) and small untied series (exact branches).
set.seed(20261019)
pairs <- list()
add_pair <- function(label, x, y) {
  out <- list(label = label, x = as.list(sprintf("%.17g", x)), y = as.list(sprintf("%.17g", y)))
  for (method in c("pearson", "spearman", "kendall")) {
    test <- suppressWarnings(cor.test(x, y, method = method))
    out[[method]] <- list(estimate = num(unname(test$estimate)), p = num(test$p.value))
  }
  pairs[[length(pairs) + 1]] <<- out
}
add_pair("wage and educ, large", wages$wage, wages$educ)
ok <- !is.na(wages$hours)
add_pair("wage and hours, large", wages$wage[ok], wages$hours[ok])
for (n in c(5, 7, 8, 9, 10, 12, 20, 49, 50, 100, 1289, 1290, 1291)) {
  x <- rnorm(n); y <- x * 0.5 + rnorm(n)
  add_pair(sprintf("untied n = %d", n), x, y)
}
# Perfect and reversed agreement, and ties in both series.
add_pair("perfect", 1:12, (1:12) * 2)
add_pair("reversed", 1:12, -(1:12))
add_pair("tied n = 30", rep(1:6, 5), rep(c(2, 1, 3, 3, 5), 6))
add_pair("tied n = 8", c(1, 1, 2, 3, 3, 4, 5, 5), c(2, 1, 1, 4, 4, 3, 6, 6))

write_json(list(reference = reference_block(), matrices = matrices, covariances = covariances,
                n_pairs = n_pairs, variables = as.list(names(numeric)), pairs = pairs),
           file.path(here(), "association.json"))
