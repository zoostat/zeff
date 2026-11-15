# R's date parts on a fixed set of days (release 0.6.0, step 3): year, month, day, the ISO day of
# the week (`%u`, Monday 1 to Sunday 7), the quarter and the ISO week (`%V`), on a working grid
# of dates, on the ends of the calendar, and at every year boundary from 1 to 9999 (1 January
# and 31 December), where the ISO week belongs to a neighbouring year; and the same parts of
# date-and-time values. The engine is held to these values exactly. Run with
# `Rscript tests/reference/r/time.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
Sys.setlocale("LC_TIME", "C")

days <- function(d) ifelse(is.na(d), "NA", as.character(as.integer(d)))
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

# Date arithmetic (step 4): days added, months added with the clamping of section 3.8 (`%m+%` of
# lubridate, written out in base R as the same day of the target month or its last day),
# differences in days, and whole months between with the same clamping; from release 0.8.0,
# step 8d, whole weeks and whole years between as well, for `time_between`.
arith_texts <- c("2026-01-31", "2024-01-31", "2026-10-16", "2026-02-28", "2024-02-29", "1999-12-31",
                 "2026-03-31", "2026-12-31", "0001-01-01", "9999-12-31")
arith <- as.Date(arith_texts, format = "%Y-%m-%d")
day_steps <- c(1L, -1L, 30L, 365L, -366L, 10000L)
month_steps <- c(1L, -1L, 3L, 12L, -13L, 24L)
clamp_month <- function(d, n) {
  lt <- as.POSIXlt(d)
  total <- lt$year * 12L + lt$mon + n
  y <- total %/% 12L
  m <- total %% 12L
  year <- y + 1900L
  leap <- (year %% 4L == 0L & year %% 100L != 0L) | year %% 400L == 0L
  last <- c(31L, if (leap) 29L else 28L, 31L, 30L, 31L, 30L, 31L, 31L, 30L, 31L, 30L, 31L)[m + 1L]
  as.Date(sprintf("%04d-%02d-%02d", year, m + 1L, min(lt$mday, last)), format = "%Y-%m-%d")
}
add_days_out <- list()
add_months_out <- list()
for (i in seq_along(arith)) {
  for (n in day_steps) {
    moved <- arith[i] + n
    add_days_out[[length(add_days_out) + 1]] <- list(d = arith_texts[i], n = n, out = days(moved))
  }
  for (n in month_steps) {
    moved <- tryCatch(clamp_month(arith[i], n), error = function(e) as.Date(NA))
    add_months_out[[length(add_months_out) + 1]] <- list(d = arith_texts[i], n = n, out = days(moved))
  }
}
between_out <- list()
for (i in seq_along(arith)) for (j in seq_along(arith)) {
  m <- 0L
  safe_clamp <- function(d, n) tryCatch(clamp_month(d, n), error = function(e) as.Date(NA))
  if (arith[j] >= arith[i]) {
    while (isTRUE(safe_clamp(arith[i], m + 1L) <= arith[j])) m <- m + 1L
  } else {
    while (isTRUE(safe_clamp(arith[i], -(m + 1L)) >= arith[j])) m <- m + 1L
    m <- -m
  }
  # Whole years, counted as months are, twelve at a step (release 0.8.0, step 8d), and whole
  # weeks, the days in sevens, truncated towards zero.
  y <- 0L
  if (arith[j] >= arith[i]) {
    while (isTRUE(safe_clamp(arith[i], 12L * (y + 1L)) <= arith[j])) y <- y + 1L
  } else {
    while (isTRUE(safe_clamp(arith[i], -12L * (y + 1L)) >= arith[j])) y <- y + 1L
    y <- -y
  }
  gap <- as.integer(arith[j] - arith[i])
  between_out[[length(between_out) + 1]] <- list(a = arith_texts[i], b = arith_texts[j],
    days = gap, months = m, weeks = as.integer(sign(gap) * (abs(gap) %/% 7L)), years = y)
}
# Whole days between two times, truncated towards zero (difftime in days, truncated).
time_pairs <- list(c("2026-01-31T23:00:00", "2026-02-02T22:59:59"), c("2026-01-31T23:00:00", "2026-02-02T23:00:00"),
                   c("2026-02-02T22:59:59", "2026-01-31T23:00:00"), c("2026-10-16T09:30:00", "2026-10-16T09:30:00"))
time_days <- lapply(time_pairs, function(p) {
  a <- as.POSIXct(p[1], format = "%Y-%m-%dT%H:%M:%S", tz = "UTC")
  b <- as.POSIXct(p[2], format = "%Y-%m-%dT%H:%M:%S", tz = "UTC")
  list(a = p[1], b = p[2], days = as.integer(trunc(as.numeric(difftime(b, a, units = "days")))))
})

# `time_between` on two times (release 0.8.0, step 8d): the seconds are R's difftime, exact on
# whole seconds; minutes, hours, days and weeks are those seconds in whole units, truncated
# towards zero; months and years are counted as on dates, the moved time keeping its time of day,
# so that a month less a second is no month (as lubridate's `interval(a, b) %/% months(1)` has it).
tb_texts <- c("2026-01-31T23:00:00", "2026-02-02T22:59:59", "2026-02-02T23:00:00", "2026-10-16T09:30:00",
              "2026-01-15T12:00:00", "2026-02-15T06:00:00", "2026-02-15T12:00:00", "2024-02-29T00:00:00",
              "2028-02-28T23:59:59", "2020-06-01T12:00:00", "2026-06-01T11:59:59", "1999-12-31T23:59:59",
              "2000-01-01T00:00:00", "2026-01-31T09:00:00", "2026-02-28T08:59:59", "2026-02-28T09:00:00")
tb <- as.POSIXct(tb_texts, format = "%Y-%m-%dT%H:%M:%S", tz = "UTC")
towards_zero <- function(s, k) as.integer(sign(s) * (abs(s) %/% k))
move_time <- function(t, n) {
  d <- as.Date(t, tz = "UTC")
  as.POSIXct(clamp_month(d, n), tz = "UTC") + (as.numeric(t) - as.numeric(as.POSIXct(d, tz = "UTC")))
}
steps_between <- function(a, b, by) {
  k <- 0L
  if (b >= a) {
    while (isTRUE(move_time(a, by * (k + 1L)) <= b)) k <- k + 1L
    k
  } else {
    while (isTRUE(move_time(a, -by * (k + 1L)) >= b)) k <- k + 1L
    -k
  }
}
time_between_out <- list()
for (i in seq_along(tb)) for (j in seq_along(tb)) {
  secs <- as.integer(as.numeric(difftime(tb[j], tb[i], units = "secs")))
  time_between_out[[length(time_between_out) + 1]] <- list(a = tb_texts[i], b = tb_texts[j],
    second = secs, minute = towards_zero(secs, 60L), hour = towards_zero(secs, 3600L),
    day = towards_zero(secs, 86400L), week = towards_zero(secs, 604800L),
    month = steps_between(tb[i], tb[j], 1L), year = steps_between(tb[i], tb[j], 12L))
}

out <- list(
  reference = reference_block(),
  time_between = time_between_out,
  add_days = add_days_out,
  add_months = add_months_out,
  between = between_out,
  time_days = time_days,
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
