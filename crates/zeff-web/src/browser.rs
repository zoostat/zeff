//! The bindings of the WebAssembly build: what the worker script calls, and what it must
//! provide.
//!
//! `wasm-bindgen` generates code that uses `unsafe`, which the rest of the workspace denies;
//! the permission is given here and in the conformance crate's entry, and nowhere else
//! (docs/decisions.md).
#![allow(unsafe_code)]

use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
extern "C" {
    /// The worker script's `zeffHost.event`: one event of the run, as a line of JSON.
    #[wasm_bindgen(js_namespace = zeffHost, js_name = event)]
    fn host_event(json: &str);

    /// The worker script's `zeffHost.cancelled`: whether the interface has set the flag the
    /// two share.
    #[wasm_bindgen(js_namespace = zeffHost, js_name = cancelled)]
    fn host_cancelled() -> bool;
}

fn cancelled() -> bool {
    host_cancelled()
}

/// A project in the worker's memory.
#[wasm_bindgen]
pub struct Engine {
    host: crate::Host,
}

#[wasm_bindgen]
impl Engine {
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str) -> Engine {
        Engine {
            host: crate::Host::new(name),
        }
    }

    /// Put a file into the project, by its path from the project's folder.
    pub fn add_file(&self, path: &str, bytes: &[u8]) {
        self.host.add_file(path, bytes);
    }

    /// Run a script. `device_bytes` is the memory the device reports, or 0 where it reports
    /// none. The events go to `zeffHost.event`; the outcome comes back as a line of JSON.
    pub fn run(&self, script: &str, text: &str, device_bytes: f64) -> String {
        let device = (device_bytes > 0.0).then_some(device_bytes as u64);
        self.host
            .run(script, text, device, cancelled, &|json| host_event(json))
    }
}

#[wasm_bindgen]
impl Engine {
    /// "Report a problem": the bug report of a script as it stands, as a line of JSON of type
    /// `report`, or of type `error` where there is nothing to report. `line` is the line of a
    /// statement whose result is believed wrong, or 0; with it come the name and the text of
    /// the reference it should agree with. Nothing is written and nothing is sent.
    pub fn report_bug(
        &self,
        script: &str,
        text: &str,
        line: u32,
        reference_name: Option<String>,
        reference_text: Option<String>,
        device_bytes: f64,
    ) -> String {
        let device = (device_bytes > 0.0).then_some(device_bytes as u64);
        let reference = match (&reference_name, &reference_text) {
            (Some(name), Some(text)) => Some((name.as_str(), text.as_str())),
            _ => None,
        };
        self.host.report_bug(
            script,
            text,
            (line > 0).then_some(line),
            reference,
            "WebAssembly, in a browser",
            device,
        )
    }
}

#[wasm_bindgen]
impl Engine {
    /// A frame of the last run as the bytes of an Arrow stream, or nothing where no binding of
    /// that name held a frame. The bytes arrive in a buffer of their own, which the worker
    /// script passes to the interface without a copy.
    pub fn frame(&self, name: &str) -> Option<Vec<u8>> {
        self.host.frame(name)
    }
}

/// The version of the engine in the worker.
#[wasm_bindgen]
impl Engine {
    /// A plot of the last run laid out at a size in points, as a line of JSON holding its SVG
    /// (`Host::plot_svg`); `theme` is left out for the plot's own.
    pub fn plot(&self, index: u32, width: f64, height: f64, theme: Option<String>) -> String {
        self.host
            .plot_svg(index as usize, width, height, theme.as_deref())
    }

    /// What a statement of the last run printed, made again at a width, as a line of JSON
    /// (`Host::print_at`).
    pub fn print_at(&self, statement: u32, width: u32) -> String {
        self.host.print_at(statement as usize, width as usize)
    }

    /// A plot of the last run as its scene, a line of JSON (`Host::plot_scene`).
    pub fn scene(&self, index: u32) -> String {
        self.host.plot_scene(index as usize)
    }

    /// A file of the project read back (`Host::file`).
    pub fn file(&self, path: &str) -> Option<Vec<u8>> {
        self.host.file(path)
    }
}

#[wasm_bindgen]
pub fn version() -> String {
    zeff_engine::VERSION.to_string()
}

/// The explained page of a diagnostic code, as a line of JSON of type `explained`, or of type
/// `error` for a code no page explains.
#[wasm_bindgen]
pub fn explain(code: &str) -> String {
    crate::explain(code)
}

/// Every code with an explained page, by family, as a line of JSON of type `codes`.
#[wasm_bindgen]
pub fn explained_codes() -> String {
    crate::explained_codes()
}
