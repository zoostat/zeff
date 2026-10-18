# Shared by the R reference scripts (testing handbook, section 1.1). Base R only: no package is
# needed yet, so renv.lock pins the R version alone; the first script that needs a package adds it.
#
# Numbers are written as text with 15 significant digits, the precision R itself prints at its
# fullest and the precision the comparison tests read, so that a last-bit difference between the
# maths libraries of two platforms never changes the committed file; `NA`, `Inf` and `-Inf` are
# written as those words. Counts and positions are written as plain JSON integers.

required_r <- "4.6.0"
if (getRversion() != required_r) {
  stop(sprintf("the references are made with R %s; this is R %s", required_r, getRversion()))
}

num <- function(x) {
  ifelse(is.na(x), "NA", ifelse(is.infinite(x), ifelse(x > 0, "Inf", "-Inf"), sprintf("%.15g", x)))
}

json_escape <- function(s) {
  s <- gsub("\\", "\\\\", s, fixed = TRUE)
  s <- gsub("\"", "\\\"", s, fixed = TRUE)
  s <- gsub("\n", "\\n", s, fixed = TRUE)
  paste0("\"", s, "\"")
}

# A JSON text for a nested list: a named list is an object, an unnamed one an array; a character
# scalar is a string, an integer scalar a number, a logical scalar true or false; vectors of
# length other than one are arrays. Doubles must be passed through num() first.
to_json <- function(x, indent = 0) {
  pad <- strrep(" ", indent)
  if (is.null(x)) return("null")
  if (is.list(x)) {
    if (length(x) == 0) return(if (is.null(names(x))) "[]" else "{}")
    if (!is.null(names(x))) {
      items <- mapply(function(k, v) paste0(pad, " ", json_escape(k), ": ", to_json(v, indent + 1)),
                      names(x), x, USE.NAMES = FALSE)
      return(paste0("{\n", paste(items, collapse = ",\n"), "\n", pad, "}"))
    }
    items <- vapply(x, function(v) paste0(pad, " ", to_json(v, indent + 1)), "")
    return(paste0("[\n", paste(items, collapse = ",\n"), "\n", pad, "]"))
  }
  if (length(x) != 1) return(to_json(as.list(x), indent))
  if (is.character(x)) return(json_escape(x))
  if (is.logical(x)) return(if (x) "true" else "false")
  if (is.integer(x)) return(as.character(x))
  stop("a double must be written through num(): ", x)
}

write_json <- function(x, file) {
  con <- file(file, open = "w", encoding = "UTF-8")
  writeLines(to_json(x), con, useBytes = FALSE)
  close(con)
  cat("wrote", file, "\n")
}

reference_block <- function() {
  list(software = R.version.string, note = "base R; numbers written with 15 significant digits")
}

here <- function() {
  args <- commandArgs(trailingOnly = FALSE)
  file <- sub("^--file=", "", args[grepl("^--file=", args)])
  if (length(file) == 0) return(".")
  dirname(normalizePath(file[1]))
}

read_dataset <- function(path) {
  read.csv(path, encoding = "UTF-8", na.strings = c("NA", ""), check.names = FALSE,
           stringsAsFactors = FALSE)
}
