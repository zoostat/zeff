# R's values for the pattern functions of release 0.7.0, step 6: grepl(), gsub(), regmatches()
# with regexpr(), and strsplit(), each with perl = TRUE where a pattern is read, on patterns
# that PCRE and Rust's regex crate read alike (classes, anchors, alternation, groups,
# repetition, escapes; ASCII under ignore.case). Every value is exact, so the comparison is
# exact. A trailing separator and an empty match are left to the Rust program, where R's
# conventions differ from Zeff's. Run with `Rscript tests/reference/r/text_regex.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

texts <- c("Hello, World", "abc123def45", "a.b.c", "café au lait", "  two  words ", "x", "", "2026-10-16",
           "日本語 123", "aaa")

# contains: literal (fixed), regular expression, and case-insensitive regular expression.
literal_patterns <- c("World", "world", "c1", ".", "", "é", "aa", "-")
regex_patterns <- c("[0-9]+", "^H", "d$", "a|x", "[aeiou]", "(ab)+", "\\.", "a{2,3}", "\\s+", "^$", "^\\d{4}-\\d{2}-\\d{2}$", "W.r")
contains_cases <- list()
for (s in texts) {
  for (p in literal_patterns) contains_cases[[length(contains_cases) + 1]] <- list(text = s, pattern = p, regex = FALSE, ignore = FALSE, out = grepl(p, s, fixed = TRUE))
  for (p in regex_patterns) {
    contains_cases[[length(contains_cases) + 1]] <- list(text = s, pattern = p, regex = TRUE, ignore = FALSE, out = grepl(p, s, perl = TRUE))
    contains_cases[[length(contains_cases) + 1]] <- list(text = s, pattern = p, regex = TRUE, ignore = TRUE, out = grepl(p, s, perl = TRUE, ignore.case = TRUE))
  }
}
for (p in c("HELLO", "WORLD", "DEF", "Caf")) {
  for (s in texts) contains_cases[[length(contains_cases) + 1]] <- list(text = s, pattern = p, regex = TRUE, ignore = TRUE, out = grepl(p, s, perl = TRUE, ignore.case = TRUE))
}

# replace: every occurrence, literal and regular expression; the replacement is plain text.
replace_specs <- list(
  list("Hello, World", "o", "0", FALSE), list("a.b.c", ".", "-", FALSE), list("a.b.c", ".", "-", TRUE), list("abc123def45", "[0-9]+", "#", TRUE),
  list("  two  words ", "\\s+", " ", TRUE), list("aaa", "a", "bb", FALSE), list("aaa", "aa", "b", FALSE), list("café", "é", "e", FALSE),
  list("Hello, World", "xyz", "!", FALSE), list("2026-10-16", "-", "/", FALSE), list("x1y22z333", "\\d", "", TRUE),
  list("Hello, World", "(l+)", "[L]", TRUE), list("", "a", "b", FALSE)
)
replace_cases <- lapply(replace_specs, function(s) list(
  text = s[[1]], from = s[[2]], to = s[[3]], regex = s[[4]],
  out = if (s[[4]]) gsub(s[[2]], s[[3]], s[[1]], perl = TRUE) else gsub(s[[2]], s[[3]], s[[1]], fixed = TRUE)
))

# extract: the first match, NA when there is none.
extract_specs <- list(
  list("abc123def45", "[0-9]+"), list("abc123def45", "[a-z]+$"), list("Hello, World", "W\\w+"), list("a.b.c", "\\."), list("2026-10-16", "\\d{2}$"),
  list("Hello, World", "xyz"), list("café au lait", "au \\w+"), list("", "a"), list("aaa", "a*"), list("x", ".")
)
extract_cases <- lapply(extract_specs, function(s) {
  m <- regmatches(s[[1]], regexpr(s[[2]], s[[1]], perl = TRUE))
  list(text = s[[1]], pattern = s[[2]], found = length(m) == 1, out = if (length(m) == 1) m else "")
})

# split: on a pattern, no trailing separator in these texts (R drops a trailing empty piece,
# Zeff keeps it; the Rust program covers that case by hand).
split_specs <- list(
  list("a,b,c", ","), list("a.b.c", "\\."), list("one  two three", "\\s+"), list("2026-10-16", "-"), list("abc", "x"),
  list("a1b22c333d", "[0-9]+"), list("key=value", "="), list("café|au|lait", "\\|"), list(",a,b", ",")
)
split_cases <- lapply(split_specs, function(s) list(text = s[[1]], sep = s[[2]], out = as.list(strsplit(s[[1]], s[[2]], perl = TRUE)[[1]])))

write_json(list(
  reference = reference_block(),
  contains = contains_cases,
  replaces = replace_cases,
  extracts = extract_cases,
  splits = split_cases
), file.path(here(), "text_regex.json"))
