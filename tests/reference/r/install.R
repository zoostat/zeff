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
# DescTools and emmeans (release 0.7.0, step 19, 11 November 2026) are references for `posthoc`:
# ScheffeTest() for Scheffe's method, and pairs() with adjust = "sidak" for Sidak's, which
# p.adjust() does not have. DescTools brings some thirty packages with it and emmeans one.
# dagitty, psych and pROC (release 0.10.0, question 4 of its opening, taken by the owner on 27
# November 2026): dagitty is the reference the specification names for causal graphs (adjustment
# sets, implied independencies and their tests, instruments), and brings V8 with it; psych's
# alpha() is the reference for `reliability`, with mnormt and GPArotation; pROC's ci.auc() is the
# reference for the interval of the area under the ROC curve. All three are under the GPL and
# are reference tooling only, as fixest is.
wanted <- c(sandwich = "3.1.1", wooldridge = "1.4-7", fixest = "0.14.2", nortest = "1.0-4", car = "3.1-5",
            DescTools = "0.99.60", emmeans = "2.0.4", dagitty = "0.3-4", psych = "2.6.9", pROC = "1.19.1")
# A dated snapshot serves one version of a package, the one current on its day, and that is not
# always the one the references were made with: on 1 October 2026 it serves sandwich 3.1-3, and
# the references are of 3.1-1 (found on 21 November 2026, when the continuous integration
# installed the one and renv.lock named the other; release 0.9.0, step 42a). A version named
# here is taken from the snapshot's archive, as a source package, after the snapshot's own has
# brought its dependencies; sandwich has no compiled code, so this needs no compiler.
archived <- c(sandwich = "3.1-1")
for (name in names(wanted)) {
  present <- requireNamespace(name, quietly = TRUE) && packageVersion(name) == wanted[[name]]
  if (!present) {
    cat("installing", name, wanted[[name]], "into", lib, "from", repos, "\n")
    install.packages(name, lib = lib, repos = repos, quiet = TRUE)
    if (packageVersion(name, lib.loc = lib) != wanted[[name]] && name %in% names(archived)) {
      file <- sprintf("%s/src/contrib/Archive/%s/%s_%s.tar.gz", snapshot, name, name, archived[[name]])
      cat("installing", name, archived[[name]], "from the archive of the snapshot,", file, "\n")
      install.packages(file, lib = lib, repos = NULL, type = "source", quiet = TRUE)
    }
  }
  cat(name, as.character(packageVersion(name, lib.loc = .libPaths())), "\n")
}
# The versions are the ones renv.lock records, or the references are not rebuilt at all: a
# reference made with another version is not the reference.
found <- vapply(names(wanted), function(name) as.character(packageVersion(name, lib.loc = .libPaths())), "")
differ <- names(wanted)[package_version(found) != package_version(unname(wanted))]
if (length(differ) > 0) {
  stop("not the versions of renv.lock: ",
       paste(sprintf("%s is %s, wanted %s", differ, found[differ], wanted[differ]), collapse = "; "))
}
