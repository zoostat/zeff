# R's accuracy on the Statistical Reference Datasets of NIST (release 0.9.0, step 36c; open
# decision #153), in R 4.6.0 and base R alone: for each of the thirty-one problems of
# tests/reference/nist, the log relative error of R's own answers against the certified values,
# -log10(|q - c| / |c|), from 0 to 15, to one decimal. They stand beside Zeff's in the release
# notes; Zeff's are in crates/zeff-conformance/tests/snapshots/nist__nist_record.snap.
#
# These figures are a comparison and not a reference: no test of the engine reads them, and the
# continuous-integration job that rebuilds the references does not rebuild this file, since the
# last digits of an ill-conditioned fit may differ between R's builds. They are remade on the
# machine a release is made on.
#
# Regression: lm() with its default tolerance, the least figure over the coefficients and over
# their standard errors; for Filip also with tol = 1e-10, since at the default R drops the tenth
# power as Zeff does. Univariate: mean(), sd() and the first autocorrelation by its definition.
# Analysis of variance: F by oneway.test(var.equal = TRUE) and by anova(lm()).
# Run with `Rscript tests/reference/r/nist.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

root <- normalizePath(file.path(here(), "..", "nist"))
lre <- function(q, c) {
  if (is.na(q)) return(0)
  if (q == c) return(15)
  e <- if (c == 0) abs(q) else abs((q - c) / c)
  max(0, min(15, -log10(e)))
}
one <- function(x) sprintf("%.1f", x)
numbers <- function(s) as.numeric(regmatches(s, gregexpr("-?[0-9]+\\.?[0-9]*(E[-+][0-9]+)?", s))[[1]])
part <- function(lines, name) {
  header <- lines[1:12]
  line <- header[grepl(name, header, fixed = TRUE) & grepl("lines", header, fixed = TRUE)][1]
  range <- as.integer(numbers(sub(".*lines", "", line)))[1:2]
  lines[range[1]:range[2]]
}

degrees <- list(Norris = 1, Pontius = 2, NoInt1 = 0, NoInt2 = 0, Filip = 10, Longley = NA,
                Wampler1 = 5, Wampler2 = 5, Wampler3 = 5, Wampler4 = 5, Wampler5 = 5)
regression <- list()
for (name in names(degrees)) {
  lines <- readLines(file.path(root, "lls", paste0(name, ".dat")))
  d <- read.table(text = part(lines, "Data"))
  certified <- part(lines, "Certified Values")
  b <- do.call(rbind, lapply(strsplit(trimws(certified[grepl("^ +B[0-9]+ ", certified)]), " +"),
                             function(t) as.numeric(t[2:3])))
  k <- degrees[[name]]
  if (is.na(k)) {
    names(d) <- c("y", paste0("x", 1:6))
    f <- y ~ x1 + x2 + x3 + x4 + x5 + x6
  } else {
    names(d) <- c("y", "x")
    f <- if (k == 0) y ~ x - 1 else
      as.formula(paste("y ~", paste(c("x", if (k > 1) paste0("I(x^", 2:k, ")")), collapse = " + ")))
  }
  fitted <- function(tol) {
    m <- lm(f, data = d, tol = tol)
    est <- coef(m)
    se <- rep(NA_real_, length(est))
    se[!is.na(est)] <- suppressWarnings(summary(m)$coefficients[, 2])
    list(coefficients = one(min(mapply(lre, est, b[, 1]))),
         standard_errors = one(min(mapply(lre, se, b[, 2]))),
         terms_dropped = as.integer(sum(is.na(est))))
  }
  regression[[name]] <- fitted(1e-7)
  if (name == "Filip") regression[[name]]$with_tolerance_1e_10 <- fitted(1e-10)
}

univariate <- list()
for (name in c("PiDigits", "Lottery", "Lew", "Mavro", "Michelso", "NumAcc1", "NumAcc2", "NumAcc3", "NumAcc4")) {
  lines <- readLines(file.path(root, "univ", paste0(name, ".dat")))
  y <- as.numeric(part(lines, "Data"))
  certified <- part(lines, "Certified Values")
  value <- function(symbol) numbers(sub(".*:", "", certified[grepl(symbol, certified, fixed = TRUE)][1]))[1]
  r1 <- sum((y[-1] - mean(y)) * (y[-length(y)] - mean(y))) / sum((y - mean(y))^2)
  univariate[[name]] <- list(mean = one(lre(mean(y), value(" ybar:"))),
                             standard_deviation = one(lre(sd(y), value(" s:"))),
                             autocorrelation = one(lre(r1, value(" r(1):"))))
}

anova_problems <- list()
for (name in c("SiRstv", "SmLs01", "SmLs02", "SmLs03", "AtmWtAg", "SmLs04", "SmLs05", "SmLs06",
               "SmLs07", "SmLs08", "SmLs09")) {
  lines <- readLines(file.path(root, "anova", paste0(name, ".dat")))
  d <- read.table(text = part(lines, "Data"))
  names(d) <- c("g", "y")
  d$g <- factor(d$g)
  certified <- part(lines, "Certified Values")
  f <- numbers(sub("^Between +\\S+", "", certified[grepl("^Between", certified)][1]))[4]
  anova_problems[[name]] <- list(
    f_oneway_test = one(lre(unname(oneway.test(y ~ g, data = d, var.equal = TRUE)$statistic), f)),
    f_anova_lm = one(lre(suppressWarnings(anova(lm(y ~ g, data = d)))$`F value`[1], f)))
}

write_json(list(reference = reference_block(),
                note = "log relative errors of R against NIST's certified values, to one decimal; a comparison for the release notes, not a reference",
                regression = regression, univariate = univariate, anova = anova_problems),
           file.path(here(), "nist.json"))
