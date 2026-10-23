# R's conversions and type names on a fixed set of values (release 0.4.0, step 4): as.integer,
# as.numeric and as.logical on texts, as.character and as.integer on numbers and Bools, and
# class() of values. The rules of `to` are those of section 3.7 of the Language Reference, which
# the Rust test applies; where R reads what Zeff refuses (hexadecimal, a leading plus, a bare
# point, truncation of a fraction) or writes a number differently (15 significant digits, `e+`),
# the test names the departure. Run with `Rscript tests/reference/r/types.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

texts <- c("42", "-7", " 8 ", "4.2e1", "1_000", "1,000", "abc", "", "true", "FALSE", "TRUE ",
           "0x10", "Inf", "-Inf", "2.5", "1e400", "+5", ".5", "5.", "1E3")
# "NaN" is left out: R's as.integer of the text "NaN" is the C cast of a NaN, 0 on one processor
# and NA on another, so the file would differ between platforms.
numbers <- c(2.5, 2, -0, 1e300, 0.1, 1 / 3, 1e15, 1e16, 123456789012, -7.25, 1e-7, 0.5, 100000)
dec <- function(x) sprintf("%.17g", x)
logical_text <- function(x) ifelse(is.na(x), "NA", ifelse(x, "true", "false"))

out <- list(
  reference = reference_block(),
  texts = as.list(texts),
  text_to_int = as.list(num(suppressWarnings(as.integer(texts)))),
  text_to_float = as.list(num(suppressWarnings(as.numeric(texts)))),
  text_to_bool = as.list(logical_text(as.logical(texts))),
  numbers = as.list(dec(numbers)),
  number_to_str = as.list(as.character(numbers)),
  number_to_int = as.list(num(suppressWarnings(as.integer(numbers)))),
  bool_to_int = as.list(num(as.integer(c(TRUE, FALSE)))),
  bool_to_str = as.list(as.character(c(TRUE, FALSE))),
  classes = list(int = class(1L), float = class(1), str = class("a"), bool = class(TRUE),
                 na = class(NA), list = class(list(1)), frame = class(data.frame()))
)
write_json(out, file.path(here(), "types.json"))
