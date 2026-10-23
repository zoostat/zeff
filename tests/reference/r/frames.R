# R's answers for the frame verbs of release 0.2.0 (steps 15 and 16) on tests/datasets/wages.csv
# and wages_large.csv: the rows head and tail show, the rows where keeps, and the order sort gives
# (stable, NA last, text by code point as Zeff sorts it: R's radix method in the C locale). Row
# numbers are 1-based positions in the file. Run with `Rscript tests/reference/r/frames.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

root <- normalizePath(file.path(here(), "..", "..", ".."))

frame_reference <- function(d, keys) {
  n <- nrow(d)
  out <- list(
    n_rows = n,
    n_cols = ncol(d),
    variables = as.list(names(d)),
    head_3 = as.list(seq_len(min(3, n))),
    tail_3 = as.list(if (n == 0) integer(0) else seq.int(max(1, n - 2), n)),
    head_10 = as.list(seq_len(min(10, n))),
    tail_10 = as.list(if (n == 0) integer(0) else seq.int(max(1, n - 9), n))
  )
  sorts <- list()
  for (key in keys) {
    for (direction in c("asc", "desc")) {
      # order() with method = "radix" is stable and sorts text by code point; NA rows come last
      # in both directions, as Zeff's sort puts them.
      o <- order(d[[key]], decreasing = (direction == "desc"), na.last = TRUE, method = "radix")
      sorts[[paste0(key, "_", direction)]] <- list(
        first_10 = as.list(as.integer(head(o, 10))),
        last_10 = as.list(as.integer(tail(o, 10)))
      )
    }
  }
  out$sort <- sorts
  out
}

wages <- read_dataset(file.path(root, "tests", "datasets", "wages.csv"))
large <- read_dataset(file.path(root, "tests", "datasets", "wages_large.csv"))

large_ref <- frame_reference(large, c("wage", "hours", "region", "educ"))
# Two keys: region by code point, then wage within the region, both ascending.
o <- order(large$region, large$wage, na.last = TRUE, method = "radix")
large_ref$sort[["region_then_wage_asc"]] <- list(first_10 = as.list(as.integer(head(o, 10))),
                                                 last_10 = as.list(as.integer(tail(o, 10))))
# where: rows whose condition is true; rows with NA in the condition are dropped and counted.
kept <- which(!is.na(large$hours) & large$hours > 40)
large_ref$where <- list(
  hours_over_40 = list(count = length(kept), first_10 = as.list(as.integer(head(kept, 10))),
                       na_dropped = sum(is.na(large$hours))),
  female = list(count = sum(large$female %in% c("true", TRUE)), na_dropped = 0L),
  educ_at_least_16_and_wage_under_20 = list(
    count = sum(large$educ >= 16 & large$wage < 20),
    first_10 = as.list(as.integer(head(which(large$educ >= 16 & large$wage < 20), 10))),
    na_dropped = 0L
  )
)

# recode (release 0.4.0, step 6). R's `levels<-` merges levels in place, the merged level standing
# where the first of its old levels stood, which is the rule of question 6 of the release; values
# are mapped by match; a labelled Int's labels follow their codes by the rule written out in
# follow_labels (a code merged into another keeps the label of the first of its old codes). The C
# locale sorts the levels by bytes, as Zeff sorts by code point.
Sys.setlocale("LC_COLLATE", "C")
region <- factor(large$region)
merge_levels <- function(f, from, to) {
  l <- levels(f)
  l[l %in% from] <- to
  levels(f) <- l
  f
}
codes_and_counts <- function(f) {
  list(levels = as.list(levels(f)), first_20 = as.list(as.integer(f)[1:20]),
       counts = as.list(as.integer(table(f))))
}
educ <- large$educ
keys <- c(20L, 12L)
values <- c(19L, 10L)
educ2 <- ifelse(educ %in% keys, values[match(educ, keys)], educ)
hours2 <- large$hours
hours2[is.na(hours2)] <- 0
follow_labels <- function(labels, from, to) {
  codes <- as.integer(names(labels))
  out <- character(0)
  for (i in order(codes)) {
    code <- codes[i]
    new <- if (code %in% from) to[match(code, from)] else code
    if (!is.na(new) && !(as.character(new) %in% names(out))) out[as.character(new)] <- labels[[i]]
  }
  as.list(out[order(as.integer(names(out)))])
}
female_labels <- c("0" = "male", "1" = "female")
large_ref$recode <- list(
  region_europe = codes_and_counts(merge_levels(region, c("Zürich", "Łódź"), "Europe")),
  region_tokyo = codes_and_counts(merge_levels(region, "東京", "Tokyo")),
  region_into_quebec = codes_and_counts(merge_levels(region, "Łódź", "Québec")),
  educ_20_to_19_12_to_10 = list(first_20 = as.list(as.integer(educ2[1:20])), changed = sum(educ2 != educ)),
  hours_na_to_0 = list(changed = sum(is.na(large$hours)), first_20 = as.list(num(hours2[1:20]))),
  labels = list(
    move_0_to_2 = follow_labels(female_labels, 0L, 2L),
    merge_1_into_0 = follow_labels(female_labels, 1L, 0L),
    swap = follow_labels(female_labels, c(0L, 1L), c(1L, 0L))
  )
)

out <- list(
  reference = reference_block(),
  datasets = list(
    wages = c(list(file = "tests/datasets/wages.csv"), frame_reference(wages, c("wage", "hours", "name"))),
    wages_large = c(list(file = "tests/datasets/wages_large.csv"), large_ref)
  )
)
write_json(out, file.path(here(), "frames.json"))
