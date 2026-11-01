# R's date parts on a fixed set of days (release 0.6.0, step 3): year, month, day, the ISO day of
# the week (`%u`, Monday 1 to Sunday 7), the quarter and the ISO week (`%V`), on a working grid
# of dates, on the ends of the calendar, and at every year boundary from 1 to 9999 (1 January
# and 31 December), where the ISO week belongs to a neighbouring year; and the same parts of
# date-and-time values. The engine is held to these values exactly. Run with
# `Rscript tests/reference/r/time.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
Sys.setlocale("LC_TIME", "C")

grid_texts <- c("2026-10-16", "2024-02-29", "1970-01-01", "1969-12-31", "2000-03-01", "1900-02-28",
                "2021-01-01", "2021-01-03", "2021-01-04", "2024-12-29", "2024-12-30", "2015-12-31",
                "2026-01-01", "2026-03-31", "2026-04-01", "2026-06-30", "2026-07-01", "2026-09-30",
                "2026-12-31", "0001-01-01", "9999-12-31", "0400-02-29", "1600-12-31", "2000-01-02",
                "2010-01-03", "2008-12-29", "2004-01-04", "2005-01-01", "2016-01-03", "2020-12-31")
grid <- as.Date(grid_texts, format = "%Y-%m-%d")
parts <- function(d) list(
  year = as.list(as.integer(format(d, "%Y"))),
  month = as.list(as.integer(format(d, "%m"))),
  day = as.list(as.integer(format(d, "%d"))),
  weekday = as.list(as.integer(format(d, "%u"))),
  quarter = as.list(as.integer(substr(quarters(d), 2, 2))),
  week = as.list(as.integer(format(d, "%V")))
)

years <- 1:9999
jan1 <- as.Date(sprintf("%04d-01-01", years), format = "%Y-%m-%d")
dec31 <- as.Date(sprintf("%04d-12-31", years), format = "%Y-%m-%d")
packed <- function(x) paste(as.integer(x), collapse = " ")

dt_texts <- c("2026-10-16T09:30:00", "1999-12-31T23:59:59", "2024-02-29T12:00:00", "2021-01-03T00:00:00")
dt <- as.POSIXct(dt_texts, format = "%Y-%m-%dT%H:%M:%S", tz = "UTC")
dt_parts <- list(
  year = as.list(as.integer(format(dt, "%Y", tz = "UTC"))),
  month = as.list(as.integer(format(dt, "%m", tz = "UTC"))),
  day = as.list(as.integer(format(dt, "%d", tz = "UTC"))),
  weekday = as.list(as.integer(format(dt, "%u", tz = "UTC"))),
  week = as.list(as.integer(format(dt, "%V", tz = "UTC")))
)

out <- list(
  reference = reference_block(),
  grid_texts = as.list(grid_texts),
  grid = parts(grid),
  boundaries = list(
    jan1_weekday = packed(format(jan1, "%u")),
    jan1_week = packed(format(jan1, "%V")),
    dec31_weekday = packed(format(dec31, "%u")),
    dec31_week = packed(format(dec31, "%V"))
  ),
  dt_texts = as.list(dt_texts),
  dt = dt_parts
)
write_json(out, file.path(here(), "time.json"))
