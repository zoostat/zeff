# The CRAN packages of the R reference harness (open question 2 of release 0.4.0, decided 23
# October 2026): installed into tests/reference/r/library, a folder Git ignores, from a dated
# snapshot of CRAN so that every machine gets the same versions, which renv.lock records. Nothing
# here is linked into the engine or the apps. Run with `Rscript tests/reference/r/install.R`; the
# reference scripts put the folder first on their library path.
here <- dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))]))
lib <- file.path(here, "library")
dir.create(lib, showWarnings = FALSE)
.libPaths(c(lib, .libPaths()))
snapshot <- "https://packagemanager.posit.co/cran/2026-10-01"
wanted <- c(sandwich = "3.1.1")
for (name in names(wanted)) {
  present <- requireNamespace(name, quietly = TRUE) && as.character(packageVersion(name)) == wanted[[name]]
  if (!present) {
    cat("installing", name, wanted[[name]], "into", lib, "\n")
    install.packages(name, lib = lib, repos = snapshot, quiet = TRUE)
  }
  cat(name, as.character(packageVersion(name, lib.loc = .libPaths())), "\n")
}
