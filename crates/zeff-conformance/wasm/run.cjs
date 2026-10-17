// Run the conformance cases of the WebAssembly build under Node and write the report.
// Usage: node run.cjs <wasm-bindgen output folder> <report file>
const fs = require("fs");
const path = require("path");
const [folder, output] = process.argv.slice(2);
const engine = require(path.resolve(folder, "zeff_conformance.js"));
fs.mkdirSync(path.dirname(path.resolve(output)), { recursive: true });
fs.writeFileSync(output, engine.report());
