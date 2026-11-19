//! One fixed document set as PDF through the Typst library, with the engine's three fonts and a
//! fixed date (release 0.8.0, step 21b; open decision #73). A measurement: see `Cargo.toml`.
//!
//! The document is `doc/main.typ`, which places a report of two fits, the lines of a t test and
//! a plot, each as `export` wrote it to a `.typ` file. Nothing is read from the machine: the
//! sources and the fonts are inside the program, no font of the system is looked for, the date
//! is given, and the PDF's creator and identifier are fixed, so that the bytes can depend only
//! on the code that sets them.

pub const MAIN: &str = include_str!("../doc/main.typ");
pub const TABLE: &str = include_str!("../doc/report.typ");
pub const LINES: &str = include_str!("../doc/one.typ");
pub const FIGURE: &str = include_str!("../doc/figure.typ");

/// The hash the engine's own identity jobs use.
pub fn hex(bytes: &[u8]) -> String {
    zeff_hash::hash_hex(bytes)
}

#[cfg(feature = "typst")]
mod set {
    use typst::diag::{FileError, FileResult};
    use typst::foundations::{Bytes, Datetime, Duration, Smart};
    use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
    use typst::text::{Font, FontBook};
    use typst::utils::LazyHash;
    use typst::{Library, LibraryExt, World};
    use typst_layout::PagedDocument;

    const FONTS: [&[u8]; 3] = [
        include_bytes!("../../../packages/tier1/plot/fonts/IBMPlexSans-Regular.ttf"),
        include_bytes!("../../../packages/tier1/plot/fonts/IBMPlexSans-SemiBold.ttf"),
        include_bytes!("../../../packages/tier1/plot/fonts/IBMPlexMono-Regular.ttf"),
    ];

    struct Fixed {
        library: LazyHash<Library>,
        book: LazyHash<FontBook>,
        fonts: Vec<Font>,
        files: Vec<(FileId, &'static str)>,
    }

    fn id(name: &str) -> FileId {
        RootedPath::new(VirtualRoot::Project, VirtualPath::new(name).unwrap()).intern()
    }

    impl World for Fixed {
        fn library(&self) -> &LazyHash<Library> {
            &self.library
        }
        fn book(&self) -> &LazyHash<FontBook> {
            &self.book
        }
        fn main(&self) -> FileId {
            self.files[0].0
        }
        fn source(&self, id: FileId) -> FileResult<Source> {
            self.files
                .iter()
                .find(|(file, _)| *file == id)
                .map(|(file, text)| Source::new(*file, text.to_string()))
                .ok_or(FileError::AccessDenied)
        }
        fn file(&self, _: FileId) -> FileResult<Bytes> {
            Err(FileError::AccessDenied)
        }
        fn font(&self, index: usize) -> Option<Font> {
            self.fonts.get(index).cloned()
        }
        fn today(&self, _: Option<Duration>) -> Option<Datetime> {
            Datetime::from_ymd(2026, 1, 1)
        }
    }

    /// The PDF of the fixed document, or what the compiler said.
    pub fn pdf() -> Result<Vec<u8>, String> {
        let fonts: Vec<Font> = FONTS
            .iter()
            .flat_map(|data| Font::iter(Bytes::new(*data)))
            .collect();
        let world = Fixed {
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(FontBook::from_fonts(&fonts)),
            fonts,
            files: vec![
                (id("main.typ"), super::MAIN),
                (id("report.typ"), super::TABLE),
                (id("one.typ"), super::LINES),
                (id("figure.typ"), super::FIGURE),
            ],
        };
        let compiled = typst::compile::<PagedDocument>(&world);
        let document = compiled.output.map_err(|errors| {
            errors
                .iter()
                .map(|e| e.message.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        })?;
        if !compiled.warnings.is_empty() {
            return Err(format!(
                "warnings: {}",
                compiled
                    .warnings
                    .iter()
                    .map(|w| w.message.to_string())
                    .collect::<Vec<_>>()
                    .join("; ")
            ));
        }
        let options = typst_pdf::PdfOptions {
            ident: Smart::Custom("zeff-typst-measure".to_string()),
            creator: Smart::Custom(Some("Zeff".to_string())),
            timestamp: Datetime::from_ymd_hms(2026, 1, 1, 0, 0, 0)
                .map(typst_pdf::Timestamp::new_utc),
            ..Default::default()
        };
        typst_pdf::pdf(&document, &options).map_err(|errors| {
            errors
                .iter()
                .map(|e| e.message.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        })
    }
}

/// The line the program prints: the size and the hash of the PDF.
#[cfg(feature = "typst")]
pub fn measured() -> String {
    match set::pdf() {
        Ok(pdf) => format!("pdf {} bytes hash {}", pdf.len(), hex(&pdf)),
        Err(said) => format!("failed: {said}"),
    }
}

/// Without Typst: the same sources hashed, so that the baseline does the same reading and
/// hashing and differs by the typesetting alone.
#[cfg(not(feature = "typst"))]
pub fn measured() -> String {
    let all = [MAIN, TABLE, LINES, FIGURE].concat();
    format!("sources {} bytes hash {}", all.len(), hex(all.as_bytes()))
}

#[cfg(feature = "typst")]
pub fn pdf() -> Result<Vec<u8>, String> {
    set::pdf()
}

/// For the browser's target: the printed line as bytes in the module's memory.
static LINE: std::sync::OnceLock<String> = std::sync::OnceLock::new();

#[unsafe(no_mangle)]
pub extern "C" fn measured_ptr() -> *const u8 {
    LINE.get_or_init(measured).as_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn measured_len() -> usize {
    LINE.get_or_init(measured).len()
}
