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

# Date arithmetic (step 4; one function `add_time` from release 0.8.0): days added, months added with the clamping of section 3.8 (`%m+%` of
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
# Weeks and years added (release 0.8.0, step 8g, for `add_time`): a week is seven days, a year
# twelve months with the same clamping.
week_steps <- c(1L, -1L, 52L, -53L, 1000L)
year_steps <- c(1L, -1L, 4L, -100L, 8000L)
add_weeks_out <- list()
add_years_out <- list()
for (i in seq_along(arith)) {
  for (n in week_steps) {
    add_weeks_out[[length(add_weeks_out) + 1]] <- list(d = arith_texts[i], n = n, out = days(arith[i] + 7L * n))
  }
  for (n in year_steps) {
    moved <- tryCatch(clamp_month(arith[i], 12L * n), error = function(e) as.Date(NA))
    add_years_out[[length(add_years_out) + 1]] <- list(d = arith_texts[i], n = n, out = days(moved))
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

# The time of day, Unix time and the earliest and latest (release 0.8.0, step 8e). The hour, the
# minute and the whole second of each of the sixteen times, as `format` writes them; numbers of
# seconds and of milliseconds since 1970-01-01 00:00:00 UTC read with `as.POSIXct(origin =)`, the
# fractions chosen exact in binary so that R's own printing of them is exact; and `min` and `max`
# of dates and of times with missing values left out.
clock_out <- lapply(seq_along(tb), function(i) list(t = tb_texts[i],
  hour = as.integer(format(tb[i], "%H", tz = "UTC")),
  minute = as.integer(format(tb[i], "%M", tz = "UTC")),
  second = as.integer(format(tb[i], "%S", tz = "UTC"))))
iso6 <- function(t) {
  whole <- format(t, "%Y-%m-%dT%H:%M:%S", tz = "UTC")
  frac <- round((as.numeric(t) - floor(as.numeric(t))) * 1e6)
  ifelse(frac == 0, whole, sprintf("%s.%06d", whole, as.integer(frac)))
}
epoch_seconds <- c(0, 1763200800, -1, 951782400, 253402300799, 1763200800.25, -1.5, 0.125, 86399.5)
# The last millisecond of the calendar, 253402300799999, is left out: a double of seconds cannot
# hold it (R prints .998993), and the engine reads an Int of milliseconds exactly, which its own
# test holds.
epoch_millis <- c(0, 1763200800250, -1, 1500, 951782400000, 0.5, 253402300799000)
epoch_out <- list(
  seconds = lapply(epoch_seconds, function(x) list(x = num(x), text = iso6(as.POSIXct(x, origin = "1970-01-01", tz = "UTC")))),
  milliseconds = lapply(epoch_millis, function(x) list(x = num(x), text = iso6(as.POSIXct(x / 1000, origin = "1970-01-01", tz = "UTC"))))
)
ex_dates <- as.Date(c("2026-03-02", NA, "2024-02-29", "2026-03-04", "1999-12-31", "2026-03-04"), format = "%Y-%m-%d")
ex_times <- as.POSIXct(c("2026-03-02T09:15:00", "2026-03-02T17:40:00", NA, "2026-03-03T08:00:00", "2026-03-01T23:59:59", "2026-03-03T08:00:00"),
                       format = "%Y-%m-%dT%H:%M:%S", tz = "UTC")
ex_group <- c("a", "a", "b", "b", "b", "a")
extremes_out <- list(
  dates = as.list(ifelse(is.na(ex_dates), "NA", format(ex_dates, "%Y-%m-%d"))),
  times = as.list(ifelse(is.na(ex_times), "NA", format(ex_times, "%Y-%m-%dT%H:%M:%S", tz = "UTC"))),
  group = as.list(ex_group),
  min_date = format(min(ex_dates, na.rm = TRUE), "%Y-%m-%d"),
  max_date = format(max(ex_dates, na.rm = TRUE), "%Y-%m-%d"),
  min_time = format(min(ex_times, na.rm = TRUE), "%Y-%m-%dT%H:%M:%S", tz = "UTC"),
  max_time = format(max(ex_times, na.rm = TRUE), "%Y-%m-%dT%H:%M:%S", tz = "UTC"),
  by_group = lapply(sort(unique(ex_group)), function(g) {
    d <- ex_dates[ex_group == g]; t <- ex_times[ex_group == g]
    list(group = g,
         min_date = format(min(d, na.rm = TRUE), "%Y-%m-%d"), max_date = format(max(d, na.rm = TRUE), "%Y-%m-%d"),
         min_time = format(min(t, na.rm = TRUE), "%Y-%m-%dT%H:%M:%S", tz = "UTC"),
         max_time = format(max(t, na.rm = TRUE), "%Y-%m-%dT%H:%M:%S", tz = "UTC"))
  })
)

# A date from numbers and the start of a period (release 0.8.0, step 8f). `ISOdate` names the day
# of a year, a month and a day, or `NA` when they name none. The start of the day, the week from
# Monday, the month, the quarter and the year of each date of the working grid inside the years
# R prints, and of each of the sixteen times, with the start of the hour: the day counts and the
# ISO texts, written out in base R (`lubridate::floor_date` gives the same, with
# `week_start = 1`).
parts_grid <- list(c(2026, 3, 15), c(2026, 3, 1), c(2024, 2, 29), c(2023, 2, 29), c(2026, 2, 30), c(2026, 13, 1),
                   c(2026, 0, 1), c(2026, 4, 31), c(1999, 12, 31), c(2000, 2, 29), c(1900, 2, 29), c(2026, 1, 0),
                   c(9999, 12, 31), c(1970, 1, 1))
from_parts_out <- lapply(parts_grid, function(p) {
  d <- suppressWarnings(as.Date(ISOdate(p[1], p[2], p[3], tz = "UTC")))
  list(year = as.integer(p[1]), month = as.integer(p[2]), day = as.integer(p[3]), days = days(d))
})
start_day <- function(d, unit) {
  lt <- as.POSIXlt(d, tz = "UTC")
  switch(unit,
    day = d,
    week = d - (as.integer(format(d, "%u")) - 1L),
    month = as.Date(sprintf("%04d-%02d-01", lt$year + 1900L, lt$mon + 1L), format = "%Y-%m-%d"),
    quarter = as.Date(sprintf("%04d-%02d-01", lt$year + 1900L, (lt$mon %/% 3L) * 3L + 1L), format = "%Y-%m-%d"),
    year = as.Date(sprintf("%04d-01-01", lt$year + 1900L), format = "%Y-%m-%d"))
}
start_units <- c("day", "week", "month", "quarter", "year")
start_grid <- grid[as.integer(format(grid, "%Y")) >= 1000]
start_dates_out <- lapply(seq_along(start_grid), function(i) {
  starts <- lapply(start_units, function(u) days(start_day(start_grid[i], u)))
  names(starts) <- start_units
  c(list(d = format(start_grid[i], "%Y-%m-%d")), starts)
})
start_times_out <- lapply(seq_along(tb), function(i) {
  day <- as.Date(tb[i], tz = "UTC")
  starts <- lapply(start_units, function(u) paste0(format(start_day(day, u), "%Y-%m-%d"), "T00:00:00"))
  names(starts) <- start_units
  c(list(t = tb_texts[i], hour = format(tb[i], "%Y-%m-%dT%H:00:00", tz = "UTC")), starts)
})

out <- list(
  reference = reference_block(),
  from_parts = from_parts_out,
  start_dates = start_dates_out,
  start_times = start_times_out,
  clock = clock_out,
  epoch = epoch_out,
  extremes = extremes_out,
  time_between = time_between_out,
  add_days = add_days_out,
  add_months = add_months_out,
  add_weeks = add_weeks_out,
  add_years = add_years_out,
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
