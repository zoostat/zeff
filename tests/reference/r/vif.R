# Variance inflation factors from lm in base R (release 0.5.0, step 21): for each column j of the
# model matrix but the intercept, 1 / (1 - R2_j) with R2_j from lm(x_j ~ the other columns), which
# is car::vif for single-column terms; for a model without an intercept the uncentred form,
# 1 - RSS / sum(x_j^2) from lm(x_j ~ the others - 1). On the shipped wages (a textbook model and
# a categorical regressor) and Wooldridge's wage1, written with 17 significant digits and compared
# within a relative 1e-10. Run with `Rscript tests/reference/r/vif.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

root <- file.path(here(), "..", "..", "..")
wages <- read_dataset(file.path(root, "packages", "tier1", "datasets", "data", "wages.csv"))
wages$lwage <- log(wages$wage)
wage1 <- read.csv(file.path(here(), "..", "data", "wage1.csv"))
dec <- function(x) sprintf("%.17g", x)

factors <- function(formula, data, intercept = TRUE) {
  X <- model.matrix(formula, data)
  cols <- setdiff(colnames(X), "(Intercept)")
  lapply(cols, function(j) {
    y <- X[, j]
    others <- X[, setdiff(colnames(X), c(j, if (intercept) "(Intercept)")), drop = FALSE]
    if (intercept) {
      r2 <- summary(lm(y ~ others))$r.squared
    } else {
      fit <- lm(y ~ others - 1)
      r2 <- 1 - sum(residuals(fit)^2) / sum(y^2)
    }
    list(term = j, vif = dec(1 / (1 - r2)))
  })
}
cases <- list(
  list(data = "wages", formula = "log(wage) ~ educ + exper + exper^2 + female", intercept = TRUE,
       factors = factors(lwage ~ educ + exper + I(exper^2) + female, wages)),
  list(data = "wages", formula = "log(wage) ~ educ + industry", intercept = TRUE,
       factors = factors(lwage ~ educ + industry, wages)),
  list(data = "wages", formula = "log(wage) ~ educ + exper - 1", intercept = FALSE,
       factors = factors(lwage ~ educ + exper - 1, wages, intercept = FALSE)),
  list(data = "wage1", formula = "lwage ~ educ + exper + tenure", intercept = TRUE,
       factors = factors(lwage ~ educ + exper + tenure, wage1)),
  list(data = "wage1", formula = "lwage ~ educ + exper + exper^2 + tenure + female", intercept = TRUE,
       factors = factors(lwage ~ educ + exper + I(exper^2) + tenure + female, wage1))
)
write_json(list(reference = reference_block(), cases = cases), file.path(here(), "vif.json"))
