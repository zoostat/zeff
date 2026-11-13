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
    /// A frame of the last run as the bytes of an Arrow stream, or nothing where no binding of
    /// that name held a frame. The bytes arrive in a buffer of their own, which the worker
    /// script passes to the interface without a copy.
    pub fn frame(&self, name: &str) -> Option<Vec<u8>> {
        self.host.frame(name)
    }
}

/// The version of the engine in the worker.
#[wasm_bindgen]
pub fn version() -> String {
    zeff_engine::VERSION.to_string()
}
