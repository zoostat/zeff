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

# Categories (release 0.4.0, step 5): factor levels in the C locale, where R sorts by bytes, the
# same order as Zeff's code points; the codes; relevel; and the four contrast matrices of
# section 4.2 for four levels, which the model matrix of step 8 is held to.
Sys.setlocale("LC_COLLATE", "C")
rows_of <- function(m) lapply(seq_len(nrow(m)), function(i) as.list(unname(num(m[i, ]))))
city <- read_dataset(file.path(here(), "..", "..", "datasets", "unicode_labels.csv"))$city
region <- read_dataset(file.path(here(), "..", "..", "datasets", "wages_large.csv"))$region
fc <- factor(city)
fr <- factor(region)
grades <- factor(c(3, 1, 2, 1, 3))

out <- list(
  reference = reference_block(),
  categories = list(
    city_levels = as.list(levels(fc)),
    city_codes = as.list(as.integer(as.integer(fc))),
    region_levels = as.list(levels(fr)),
    region_first_codes = as.list(as.integer(fr)[1:20]),
    region_relevel = as.list(levels(relevel(fr, "Zürich"))),
    numeric_levels = as.list(levels(grades)),
    numeric_codes = as.list(as.integer(grades))
  ),
  contrasts = list(
    treatment = rows_of(contr.treatment(4)),
    sum = rows_of(contr.sum(4)),
    helmert = rows_of(contr.helmert(4)),
    poly = rows_of(contr.poly(4))
  ),
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
