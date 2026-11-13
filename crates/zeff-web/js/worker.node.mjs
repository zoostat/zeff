// The engine's worker as a worker thread of Node: what the tests start. The folder of the
// WebAssembly build (the output of `wasm-bindgen --target nodejs`) and the memory the device
// is said to have come in `workerData`.
import { parentPort, workerData } from "node:worker_threads";
import { createRequire } from "node:module";
import path from "node:path";
import { serve } from "./host.mjs";

const require = createRequire(import.meta.url);
const engine = require(path.resolve(workerData.pkg, "zeff_web.js"));
// A port in the form the browser gives: `postMessage`, and `onmessage` to be set.
const port = {
  postMessage: (message) => parentPort.postMessage(message),
  set onmessage(handler) {
    parentPort.on("message", handler);
  },
};
serve(engine, port, workerData.deviceBytes);
