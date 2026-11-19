// Runs the measuring module built for the browser's target in Node and prints its line, then
// the sizes of the file, raw and compressed (release 0.8.0, step 21b).
// Usage: node run.cjs <file.wasm>
"use strict";
const fs = require("fs");
const zlib = require("zlib");
const file = process.argv[2];
const bytes = fs.readFileSync(file);
const module_ = new WebAssembly.Module(bytes);
const imports = {};
for (const i of WebAssembly.Module.imports(module_)) {
  imports[i.module] = imports[i.module] || {};
  imports[i.module][i.name] = (...args) => { throw new Error(`import called: ${i.module}.${i.name}`); };
}
console.log("imports:", WebAssembly.Module.imports(module_).map((i) => `${i.module}.${i.name}`).join(", ") || "none");
const instance = new WebAssembly.Instance(module_, imports);
const t0 = process.hrtime.bigint();
const ptr = instance.exports.measured_ptr();
const len = instance.exports.measured_len();
const ms = Number(process.hrtime.bigint() - t0) / 1e6;
console.log(Buffer.from(instance.exports.memory.buffer, ptr, len).toString("utf8"), `(${ms.toFixed(0)} ms)`);
const brotli = zlib.brotliCompressSync(bytes, { params: { [zlib.constants.BROTLI_PARAM_QUALITY]: 11, [zlib.constants.BROTLI_PARAM_SIZE_HINT]: bytes.length } }).length;
console.log(`raw ${bytes.length} brotli ${brotli} gzip ${zlib.gzipSync(bytes, { level: 9 }).length}`);
