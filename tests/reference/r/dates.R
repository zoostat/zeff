# R's dates and times on a fixed set of values (release 0.6.0, step 2): as.Date on ISO text and
# with a format, the day counts since 1970-01-01, comparisons and differences in days, sorting,
# the earliest and latest, as.POSIXct on ISO date-and-time text, and the conversions between a
# date and a time. The rules are those of sections 3.7 and 3.8 of the Language Reference (ISO
# 8601 or a written format, never a guess; a DateTime has no zone and an offset is applied to
# reach UTC); the test holds the engine to these values exactly. Years below 1000 are kept as day
# counts, not text, because R's own printing of them varies between platforms. Run with
# `Rscript tests/reference/r/dates.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
Sys.setlocale("LC_TIME", "C")

iso_texts <- c("2026-10-16", "2024-02-29", "1970-01-01", "1969-12-31", "2000-03-01", "1900-02-28",
               "2023-02-29", "2026-13-01", "2026-02-30", "16/10/2026", "2026-10-16 ", "")
iso <- as.Date(iso_texts, format = "%Y-%m-%d")
# Day counts since the epoch; `NA` where the text is no date.
days <- function(d) ifelse(is.na(d), "NA", as.character(as.integer(d)))
date_text <- function(d) ifelse(is.na(d), "NA", format(d, "%Y-%m-%d"))

# The calendar's ends, as day counts.
ends <- as.Date(c("0001-01-01", "9999-12-31", "0001-12-31", "0100-02-28", "0400-02-29"), format = "%Y-%m-%d")

# Comparisons and differences over the pairs of five dates.
five <- as.Date(c("2026-10-16", "2024-02-29", "1970-01-01", "2026-10-16", "1999-12-31"), format = "%Y-%m-%d")
pairs <- list()
for (i in seq_along(five)) for (j in seq_along(five)) {
  pairs[[length(pairs) + 1]] <- list(
    a = format(five[i], "%Y-%m-%d"), b = format(five[j], "%Y-%m-%d"),
    lt = five[i] < five[j], eq = five[i] == five[j], le = five[i] <= five[j],
    diff = as.integer(five[j] - five[i])
  )
}

# A written format: day first, then month names in the C locale.
dmy_texts <- c("16/10/2026", "29/02/2024", "29/02/2023", "31/12/1999", "2026-10-16", "1/2/2026", "01/02/26")
dmy <- as.Date(dmy_texts, format = "%d/%m/%Y")
bdy_texts <- c("Oct 16, 2026", "February 29, 2024", "oct 16, 2026", "Octo 16, 2026")
bdy <- as.Date(bdy_texts, format = "%b %d, %Y")

# Date and time: seconds since the epoch in UTC, and the ISO text R prints.
dt_texts <- c("2026-10-16T09:30:00", "2026-10-16 09:30:00", "1970-01-01T00:00:00", "1969-12-31T23:59:59",
              "2024-02-29T12:00:00", "2026-10-16T24:00:00", "2026-10-16T09:30")
dt <- as.POSIXct(dt_texts, format = "%Y-%m-%dT%H:%M:%S", tz = "UTC")
dt[2] <- as.POSIXct(dt_texts[2], format = "%Y-%m-%d %H:%M:%S", tz = "UTC")
dt[7] <- as.POSIXct(dt_texts[7], format = "%Y-%m-%dT%H:%M", tz = "UTC")
seconds <- function(t) ifelse(is.na(t), "NA", as.character(as.integer(as.numeric(t))))
dt_text <- function(t) ifelse(is.na(t), "NA", format(t, "%Y-%m-%dT%H:%M:%S", tz = "UTC"))
# An offset applied to reach UTC: the local time less the offset.
local <- as.POSIXct("2026-10-16 09:30:00", format = "%Y-%m-%d %H:%M:%S", tz = "UTC")
offsets <- list(
  list(text = "2026-10-16T09:30:00Z", utc = dt_text(local)),
  list(text = "2026-10-16T09:30:00+02:00", utc = dt_text(local - 2 * 3600)),
  list(text = "2026-10-16T09:30:00-05:30", utc = dt_text(local + 5.5 * 3600)),
  list(text = "2026-10-16T09:30:00+0200", utc = dt_text(local - 2 * 3600))
)

# Between a date and a time: midnight, and the day dropped.
midnight <- as.POSIXct(five, tz = "UTC")
dropped <- as.Date(dt[!is.na(dt)], tz = "UTC")

# Order: sort, unique, the earliest and the latest.
six <- as.Date(c("2026-10-16", "1970-01-01", "2024-02-29", "2026-10-16", "1999-12-31", "0001-01-01"), format = "%Y-%m-%d")

out <- list(
  reference = reference_block(),
  iso_texts = as.list(iso_texts),
  iso_days = as.list(days(iso)),
  iso_text = as.list(date_text(iso)),
  ends_texts = as.list(c("0001-01-01", "9999-12-31", "0001-12-31", "0100-02-28", "0400-02-29")),
  ends_days = as.list(days(ends)),
  ends_span = as.character(as.integer(ends[2] - ends[1])),
  pairs = pairs,
  dmy_texts = as.list(dmy_texts),
  dmy_days = as.list(days(dmy)),
  bdy_texts = as.list(bdy_texts),
  bdy_days = as.list(days(bdy)),
  dt_texts = as.list(dt_texts),
  dt_seconds = as.list(seconds(dt)),
  dt_text = as.list(dt_text(dt)),
  offsets = offsets,
  midnight_seconds = as.list(seconds(midnight)),
  dropped_days = as.list(days(dropped)),
  six_days = as.list(days(six)),
  six_sorted_days = as.list(days(sort(six))),
  six_unique_days = as.list(days(unique(six))),
  six_min_days = days(min(six)),
  six_max_days = days(max(six))
)
write_json(out, file.path(here(), "dates.json"))
