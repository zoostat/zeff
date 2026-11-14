// The engine's worker in the browser: a module worker, started by the interface with
// `new Worker("worker.js", { type: "module" })`. The WebAssembly build beside it is the output
// of `wasm-bindgen --target web --out-dir pkg`. The messages are those of `host.mjs`.
//
// Two tests run the messages: `worker.test.mjs` runs `host.mjs`, where they are handled, under
// a worker thread of Node (`worker.node.mjs`); `browser.test.mjs` loads this very file in
// headless Chrome, from a page that is cross-origin isolated.
import init, * as engine from "./pkg/zeff_web.js";
import { serve } from "./host.mjs";

await init();
// The memory the device reports, in bytes; browsers give it in gigabytes, rounded, or not at all.
const deviceBytes = (self.navigator?.deviceMemory ?? 0) * 2 ** 30;
serve(engine, self, deviceBytes);
