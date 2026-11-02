# R reads the sample frame `save` wrote (release 0.6.0, step 15): read.csv on
# tests/reference/exports/sample.csv gives every value under the writing rules of section 3.8,
# which this script asserts; it writes no reference file. Run with
# `Rscript tests/reference/r/exports.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

path <- file.path(here(), "..", "exports", "sample.csv")
d <- read.csv(path, stringsAsFactors = FALSE, na.strings = "", strip.white = FALSE, encoding = "UTF-8")
stopifnot(identical(names(d), c("id", "x", "flag", "note", "day", "at", "level")))
stopifnot(identical(d$id, 1:8))
# The seventeen-digit value as the file writes it, so that R parses the same text here and there.
stopifnot(identical(d$x, c(1.5, 0, 0.1, Inf, 123456789.12345679, NA, 1e300, 120)))
# R keeps `true` and `false` as text; `as.logical` reads them.
stopifnot(identical(as.logical(d$flag), c(TRUE, FALSE, NA, TRUE, FALSE, TRUE, NA, FALSE)))
stopifnot(identical(d$note, c("plain", "with, comma", "with \"quotes\"", "two\nlines", NA, NA, " padded ", "naïve café")))
stopifnot(identical(as.Date(d$day), as.Date(c("1970-01-01", "2024-01-01", NA, "1900-01-01", "9999-12-31", "1970-01-02", "1969-12-31", "2019-04-14"))))
stopifnot(identical(d$at[1], "1970-01-01T00:00:00"), identical(d$at[4], "2024-01-01T00:00:00.123456"))
stopifnot(identical(d$level, c("low", "high", "mid", NA, "high", "low", "mid", "high")))
cat("R read the sample CSV back value for value\n")
