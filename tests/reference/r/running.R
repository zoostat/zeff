# R's cumulative sums and products, ranks under the four tie rules with NA kept, and cut into
# intervals with its default labels (release 0.6.0, step 9): running_sum, running_product, rank
# and bin. The values summed and multiplied are integers to 1e12 and dyadic fractions, whose
# sums and products are exact in a double, so that R's long-double accumulator in cumsum (80 bits
# on x86-64, 64 on arm64) and the engine's plain double agree bit for bit on every platform. The
# engine is held to these values exactly, and to cut's labels character by character. Run with
# `Rscript tests/reference/r/running.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

set.seed(20261102)
text_or_na <- function(x) ifelse(is.na(x), "NA", as.character(x))

# Sums: 60 values, two of them NA in a second copy, so the rest after an NA is NA.
x <- c(sample(-1e6:1e6, 40), round(runif(16, -1e12, 1e12)), 0.5, -0.25, 0.125, 1e12)
x <- x[sample(length(x))]
x_na <- x
x_na[c(30, 45)] <- NA
# Products: small dyadic values, exact to the last row.
p <- sample(c(1, 2, 0.5, -1, 3, 0.25, -2, 1.5), 30, replace = TRUE)
p_na <- p
p_na[20] <- NA

sums <- list(x = as.list(num(x)), sum = as.list(num(cumsum(x))),
             x_na = as.list(num(x_na)), sum_na = as.list(num(cumsum(x_na))),
             p = as.list(num(p)), product = as.list(num(cumprod(p))),
             p_na = as.list(num(p_na)), product_na = as.list(num(cumprod(p_na))))

# Ranks: ties throughout, NA kept, on Ints and on Floats.
r_int <- sample(c(1:8, NA), 40, replace = TRUE)
r_float <- round(sample(c(-1.5, 0, 0.25, 2, 2, 7.75, NA), 40, replace = TRUE), 2)
rank_block <- function(v) {
  out <- list(x = as.list(num(v)))
  for (m in c("average", "min", "max", "first")) {
    out[[m]] <- as.list(num(rank(v, ties.method = m, na.last = "keep")))
  }
  out
}
ranks <- list(ints = rank_block(r_int), floats = rank_block(r_float))

# Cut: open ends, both closures, values on the breaks, a value outside every interval, labels
# given, and breaks whose default labels need more than three digits.
v <- c(round(rnorm(50) * 100, 1), NA, 0, 100, -100, 50, -50)
cut_block <- function(v, breaks, right = TRUE, labels = NULL) {
  f <- if (is.null(labels)) cut(v, breaks, right = right) else cut(v, breaks, right = right, labels = labels)
  list(x = as.list(num(v)), breaks = as.list(num(breaks)), levels = as.list(levels(f)),
       bin = as.list(text_or_na(as.character(f))), outside = sum(is.na(f)) - sum(is.na(v)))
}
small <- c(0.05, 0.1234, 0.12345, 0.5, 0.9, 1, NA)
bins <- list(
  open_right = cut_block(v, c(-Inf, -50, 0, 50, Inf)),
  open_left = cut_block(v, c(-Inf, -50, 0, 50, Inf), right = FALSE),
  closed_right = cut_block(v, c(-100, 0, 100)),
  closed_left = cut_block(v, c(-100, 0, 100), right = FALSE),
  outside = cut_block(v, c(0, 10, 100)),
  labelled = cut_block(v, c(-Inf, -50, 50, Inf), labels = c("low", "mid", "high")),
  digits = cut_block(v, c(-Inf, 0, 2.5, 1000, 123456, Inf)),
  more_digits = cut_block(small, c(0, 0.1234, 0.1235, 1)),
  unsorted = cut_block(small, c(1, 0.5, 0))
)

out <- list(reference = reference_block(), sums = sums, ranks = ranks, bins = bins)
write_json(out, file.path(here(), "running.json"))
