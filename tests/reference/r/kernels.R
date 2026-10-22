# R's values for the linear-algebra kernel on the cases of tests/reference/kernels.py (release
# 0.4.0, step 2): the limited-pivoting QR of base R's `qr` (LINPACK's dqrdc2, the factorisation
# behind `lm`, with its default tolerance 1e-7), the coefficients, residuals and leverages it
# gives, the exact condition number of the kept R factor from its singular values, the singular
# values of each matrix, `backsolve` on the triangular system and `chol` and `solve` on the
# positive-definite one. R is the main reference (testing handbook, section 1.1); the exact
# rational reference of kernels.py stays the strict one. Run with
# `Rscript tests/reference/r/kernels.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

cases <- read.csv(file.path(here(), "..", "data", "kernels_cases.csv"), colClasses = "character")
cases$row <- as.integer(cases$row)
cases$col <- as.integer(cases$col)
cases$value <- as.numeric(cases$value)  # hexadecimal floats, read exactly

matrix_of <- function(name, kind) {
  sub <- cases[cases$case == name & cases$kind == kind, ]
  m <- matrix(0, max(sub$row) + 1L, max(sub$col) + 1L)
  m[cbind(sub$row + 1L, sub$col + 1L)] <- sub$value
  m
}
vector_of <- function(name, kind) as.vector(matrix_of(name, kind))
rows_of <- function(m) lapply(seq_len(nrow(m)), function(i) as.list(num(m[i, ])))

least_squares <- list()
for (name in unique(cases$case[cases$kind == "x"])) {
  x <- matrix_of(name, "x")
  y <- vector_of(name, "y")
  q <- qr(x, tol = 1e-7)
  rank <- q$rank
  r <- qr.R(q)
  coefficients <- qr.coef(q, y)
  residuals <- qr.resid(q, y)
  kept <- r[seq_len(rank), seq_len(rank), drop = FALSE]
  d <- svd(kept)$d
  # The same with the kept columns scaled to unit norm, the condition number the collinearity
  # rule of section 7.2 reports (the kernel's accuracy is governed by it, not by the raw one).
  scaled <- if (rank > 0) kept %*% diag(1 / sqrt(colSums(kept^2)), nrow = rank) else kept
  ds <- svd(scaled)$d
  leverage <- if (rank > 0) rowSums(qr.Q(q)[, seq_len(rank), drop = FALSE]^2) else rep(0, nrow(x))
  least_squares[[name]] <- list(
    rank = as.integer(rank),
    pivot = as.integer(q$pivot - 1L),
    r = rows_of(r),
    coefficients = as.list(num(coefficients)),
    residual_sum_of_squares = num(sum(residuals^2)),
    fitted = as.list(num(y - residuals)),
    leverage = as.list(num(leverage)),
    condition_number = num(if (rank > 0) max(d) / min(d) else 1),
    condition_number_scaled = num(if (rank > 0) max(ds) / min(ds) else 1),
    singular_values = as.list(num(svd(x)$d))
  )
}

tri_r <- matrix_of("triangular", "r")
tri_b <- vector_of("triangular", "b")
pd_a <- matrix_of("positive_definite", "a")
pd_b <- vector_of("positive_definite", "b")

out <- list(
  reference = reference_block(),
  least_squares = least_squares,
  triangular = list(
    x = as.list(num(backsolve(tri_r, tri_b))),
    inverse = rows_of(backsolve(tri_r, diag(nrow(tri_r))))
  ),
  # The system is solved through the Cholesky factor, as the kernel solves it: `solve(A, b)`,
  # LU with partial pivoting, loses four digits on this matrix (a diagonal entry of 1e12 beside
  # entries of order one), where the factor-based solve keeps ten.
  positive_definite = list(
    l = rows_of(t(chol(pd_a))),
    x = as.list(num(backsolve(chol(pd_a), forwardsolve(t(chol(pd_a)), pd_b))))
  )
)
write_json(out, file.path(here(), "kernels.json"))
