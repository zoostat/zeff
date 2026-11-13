// The browser build in the benchmark suite (testing handbook, section 6; release 0.7.0, step
// 33e3). Each case is a script run through the engine as a browser loads it, on the suite's
// generated file. Before a case is timed, what it prints and says is compared, to the byte,
// with what the native engine gave for the same script: a time counts only for the right
// answer. The times are reported, not judged: they vary with the machine.
//
// Usage: node bench.mjs <worker build> <folder written by bench_browser> [--runs N]
// The worker build is the output of `wasm-bindgen --target nodejs`; the folder holds
// `cases.json` and the generated files. Prints a table (also to $GITHUB_STEP_SUMMARY), writes
// `browser-bench.json` in the folder, and exits 1 when a case prints something else.
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";

const [pkg, folder] = process.argv.slice(2);
if (!pkg || !folder) {
  console.error("usage: node bench.mjs <worker build> <folder written by bench_browser> [--runs N]");
  process.exit(2);
}
const at = process.argv.indexOf("--runs");
const runs = at === -1 ? 3 : Number(process.argv[at + 1]);

// What the engine calls while it runs: nobody listens here, and nothing is cancelled.
globalThis.zeffHost = { event() {}, cancelled: () => false };
const require = createRequire(import.meta.url);
const engine = require(path.resolve(pkg, "zeff_web.js"));
const cases = JSON.parse(fs.readFileSync(path.join(folder, "cases.json"), "utf8"));

const median = (numbers) => {
  const sorted = [...numbers].sort((a, b) => a - b);
  const middle = sorted.length >> 1;
  return sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
};

const results = [];
const wrong = [];
const files = new Map();
for (const one of cases) {
  if (!files.has(one.file)) files.set(one.file, fs.readFileSync(path.join(folder, one.file)));
  const times = [];
  for (let run = 0; run < runs; run++) {
    // A project of its own each time, as a window that has just opened the file.
    const project = new engine.Engine("bench");
    project.add_file("data/bench.csv", files.get(one.file));
    const before = process.hrtime.bigint();
    const done = JSON.parse(project.run("scripts/main.zeff", one.script, 0));
    times.push(Number(process.hrtime.bigint() - before) / 1e6);
    project.free();
    if (done.stdout + done.stderr !== one.expected) {
      wrong.push(`${one.name} at ${one.rows} rows prints something else than the native engine`);
      break;
    }
  }
  results.push({ name: one.name, rows: one.rows, ms: median(times), runs: times });
}

const lines = [
  "### The browser build in the benchmark suite",
  "",
  `The engine as a browser loads it, under Node ${process.version}; each case a script that reads the generated file and prints its result, checked to the byte against the native engine; the median of ${runs} runs.`,
  "",
  "| Case | Rows | Time |",
  "| --- | ---: | ---: |",
  ...results.map((r) => `| ${r.name} | ${r.rows.toLocaleString("en-US")} | ${r.ms.toFixed(1)} ms |`),
  "",
  wrong.length ? `**Wrong:** ${wrong.join("; ")}.` : "Every case printed what the native engine prints.",
  "",
];
const table = lines.join("\n");
console.log(table);
if (process.env.GITHUB_STEP_SUMMARY) fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, table + "\n");
fs.writeFileSync(path.join(folder, "browser-bench.json"), JSON.stringify({ node: process.version, runs, results, wrong }, null, 2) + "\n");
if (wrong.length) process.exit(1);
