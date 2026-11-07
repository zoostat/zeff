# R's values for the text functions of release 0.7.0, step 5: toupper(), tolower(), trimws(),
# startsWith(), endsWith(), substr(), formatC(width) and paste(collapse) on fixed strings, with
# accented, combining and non-Latin text among them. Every value is exact, so the comparison is
# exact. The case mappings are those R's platform gives for these characters, which agree with
# Unicode simple case mapping; `trimws` trims ASCII white space only, so the Unicode spaces that
# `trim` also removes are tested in the Rust program by hand. Run with
# `Rscript tests/reference/r/text.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

# Strings: plain, accented, a combining mark, German sharp s, a ligature, Greek, Cyrillic, CJK,
# emoji, a flag, white space around, empty.
strings <- c("Hello, World", "café", "ÉCOLE", "é", "Straße", "ﬁsh", "Σίσυφος",
             "Привет", "日本語", "\U0001F600", "\U0001F1EC\U0001F1E7", "  padded  ",
             " spaces and\nlines \n", "", "MiXeD 123")

case_cases <- lapply(strings, function(s) list(
  text = s,
  upper = toupper(s),
  lower = tolower(s),
  trimmed = trimws(s),
  len = nchar(s, type = "chars")
))

# Prefixes and suffixes, with the empty text and a longer candidate among them.
affix_pairs <- list(
  c("Hello, World", "Hello"), c("Hello, World", "hello"), c("Hello, World", "World"), c("Hello, World", ""),
  c("café", "caf"), c("café", "é"), c("café", "cafés"), c("", ""), c("", "a"),
  c("日本語", "日"), c("日本語", "語"), c("é", "e"), c("é", "́")
)
affix_cases <- lapply(affix_pairs, function(p) list(
  text = p[1], affix = p[2],
  starts = startsWith(p[1], p[2]),
  ends = endsWith(p[1], p[2])
))

# Slices: positions count characters, from 1; beyond the end they give what is there.
slice_specs <- list(
  list("Hello, World", 1L, 5L), list("Hello, World", 8L, 5L), list("Hello, World", 8L, 50L), list("Hello, World", 13L, 2L),
  list("Hello, World", 1L, 0L), list("café", 4L, 1L), list("café", 2L, 2L), list("é", 2L, 1L),
  list("日本語", 2L, 2L), list("\U0001F1EC\U0001F1E7", 2L, 1L), list("", 1L, 3L), list("abc", 3L, 1L)
)
slice_cases <- lapply(slice_specs, function(s) list(
  text = s[[1]], start = s[[2]], len = s[[3]],
  out = substr(s[[1]], s[[2]], s[[2]] + s[[3]] - 1L)
))

# Padding with spaces on the left, to a width in characters (ASCII text, where R's width is
# the character count); a width below the length leaves the text as it is.
pad_specs <- list(list("abc", 6L), list("abc", 3L), list("abc", 1L), list("", 2L), list("12.5", 8L), list("x", 0L))
pad_cases <- lapply(pad_specs, function(s) list(text = s[[1]], width = s[[2]], out = formatC(s[[1]], width = s[[2]])))

# Joining with a separator, an empty list and an NA item among them.
join_specs <- list(
  list(c("a", "b", "c"), ", "), list(c("a", "b", "c"), ""), list(c("one"), "-"), list(character(0), ", "),
  list(c("café", "日本"), " | "), list(c("a", NA, "c"), "/")
)
join_cases <- lapply(join_specs, function(s) list(
  items = lapply(s[[1]], function(x) if (is.na(x)) "NA" else x),
  na = as.list(is.na(s[[1]])),
  sep = s[[2]],
  out = paste(s[[1]], collapse = s[[2]])
))

write_json(list(
  reference = reference_block(),
  cases = case_cases,
  affixes = affix_cases,
  slices = slice_cases,
  pads = pad_cases,
  joins = join_cases
), file.path(here(), "text.json"))
