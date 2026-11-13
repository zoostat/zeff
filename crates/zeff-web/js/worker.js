// The engine's worker in the browser: a module worker, started by the interface with
// `new Worker("worker.js", { type: "module" })`. The WebAssembly build beside it is the output
// of `wasm-bindgen --target web --out-dir pkg`. The messages are those of `host.mjs`.
//
// This file is not run by the tests, which have no browser: they run `host.mjs`, where the
// messages are handled, under a worker thread of Node (`worker.node.mjs`).
import init, * as engine from "./pkg/zeff_web.js";
import { serve } from "./host.mjs";

await init();
// The memory the device reports, in bytes; browsers give it in gigabytes, rounded, or not at all.
const deviceBytes = (self.navigator?.deviceMemory ?? 0) * 2 ** 30;
serve(engine, self, deviceBytes);
