# R's formatting of numbers and dates on a fixed set of values (release 0.6.0, step 5): sprintf
# to a number of decimals (the correctly rounded decimal expansion of the binary value, which R
# and the engine share), formatC with a thousands separator on whole numbers, and format of dates
# and times by a pattern in the C locale. The engine is held to these texts exactly. Run with
# `Rscript tests/reference/r/format.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
Sys.setlocale("LC_TIME", "C")

numbers <- c(1234567.891, 2.675, 0.125, 0.375, -0.0001, 1e16, 1e-7, 123456789.123456789, -1234.5, 0.5, 1.5, 2.5,
             1e15, 999999.5, 1 / 3, 2 / 3, 1e12 + 0.5, 12345678901234.567, 42, -42.42)
decimals <- 0:6
fixed <- list()
for (x in numbers) for (d in decimals) {
  fixed[[length(fixed) + 1]] <- list(x = sprintf("%.17g", x), digits = d, out = sprintf(paste0("%.", d, "f"), x))
}
whole <- c(0L, 7L, 999L, 1000L, 1234567L, -1234567L, 100000L, -1L, 2147483647L)
marks <- lapply(whole, function(n) list(n = n, comma = formatC(n, format = "d", big.mark = ","), space = formatC(n, format = "d", big.mark = " ")))
patterns <- c("%d/%m/%Y", "%b %Y", "%B %d, %Y", "%A", "%a %d %b", "%j", "%y", "%Y-%m-%d", "%d.%m.%y", "%H:%M:%S")
date_texts <- c("2026-10-16", "2024-02-29", "2024-12-31", "1999-12-31", "2000-01-01", "2021-01-03", "1970-01-01")
dates <- as.Date(date_texts, format = "%Y-%m-%d")
date_out <- list()
for (i in seq_along(dates)) for (p in patterns) {
  date_out[[length(date_out) + 1]] <- list(d = date_texts[i], pattern = p, out = format(dates[i], p))
}
dt_texts <- c("2026-10-16T09:05:07", "1999-12-31T23:59:59", "2024-02-29T00:00:00")
dts <- as.POSIXct(dt_texts, format = "%Y-%m-%dT%H:%M:%S", tz = "UTC")
dt_patterns <- c("%Y-%m-%d %H:%M", "%H:%M:%S", "%d %b %Y at %H:%M", "%j %A")
dt_out <- list()
for (i in seq_along(dts)) for (p in dt_patterns) {
  dt_out[[length(dt_out) + 1]] <- list(t = dt_texts[i], pattern = p, out = format(dts[i], p, tz = "UTC"))
}
out <- list(reference = reference_block(), fixed = fixed, marks = marks, dates = date_out, times = dt_out)
write_json(out, file.path(here(), "format.json"))
