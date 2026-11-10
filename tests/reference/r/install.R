# The CRAN packages of the R reference harness (open question 2 of release 0.4.0, decided 23
# October 2026): installed into tests/reference/r/library, a folder Git ignores, from a dated
# snapshot of CRAN so that every machine gets the same versions, which renv.lock records. Nothing
# here is linked into the engine or the apps. Run with `Rscript tests/reference/r/install.R`; the
# reference scripts put the folder first on their library path. fixest (question 4 of release
# 0.6.0, decided 1 November 2026) is the reference for `fe:` and `event`; its dependencies
# dreamerr and stringmagic come with it from the same snapshot.
here <- dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))]))
lib <- file.path(here, "library")
dir.create(lib, showWarnings = FALSE)
.libPaths(c(lib, .libPaths()))
snapshot <- "https://packagemanager.posit.co/cran/2026-10-01"
# car (owner's decision at the start of release 0.7.0; first needed at step 17, 10 November 2026)
# is the reference for the factorial form of `anova`. It brings some sixty packages with it, so on
# Linux, where CRAN has no binaries, the same dated snapshot is read through the package
# manager's path of binaries built for the distribution: the same versions, without an hour of
# compiling on every run of the continuous integration. On macOS the snapshot serves CRAN's own
# binaries as before.
repos <- snapshot
if (Sys.info()[["sysname"]] == "Linux" && file.exists("/etc/os-release")) {
  release <- readLines("/etc/os-release")
  codename <- sub("^VERSION_CODENAME=", "", grep("^VERSION_CODENAME=", release, value = TRUE))
  if (length(codename) == 1 && nzchar(codename)) {
    repos <- sub("/cran/", sprintf("/cran/__linux__/%s/", codename), snapshot, fixed = TRUE)
    # The package manager serves a binary only to a client that says which R it is.
    options(HTTPUserAgent = sprintf("R/%s R (%s)", getRversion(),
                                    paste(getRversion(), R.version$platform, R.version$arch, R.version$os)))
  }
}
wanted <- c(sandwich = "3.1.1", wooldridge = "1.4-7", fixest = "0.14.2", nortest = "1.0-4", car = "3.1-5")
for (name in names(wanted)) {
  present <- requireNamespace(name, quietly = TRUE) && packageVersion(name) == wanted[[name]]
  if (!present) {
    cat("installing", name, wanted[[name]], "into", lib, "from", repos, "\n")
    install.packages(name, lib = lib, repos = repos, quiet = TRUE)
  }
  cat(name, as.character(packageVersion(name, lib.loc = .libPaths())), "\n")
}
