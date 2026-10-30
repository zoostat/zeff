// Measure the browser engine bundle: its compressed size and its cold-start time under Node.
// Risks R2 and R4: the compressed bundle must stay under 5 MB until 0.7.0, and a cold start must
// stay inside its budget (docs/decisions.md records the thresholds).
//
// Usage: node measure.cjs <wasm-bindgen output folder> <entry module name> <output json>
//          [--limit-bytes N] [--limit-ms N] [--runs N]
// Writes a JSON report and a Markdown table (to $GITHUB_STEP_SUMMARY when set), prints both, and
// exits 1 when a limit is exceeded.
"use strict";

const fs = require("fs");
const path = require("path");
const zlib = require("zlib");
const { spawnSync } = require("child_process");

const DEFAULT_LIMIT_BYTES = 5_000_000; // risk R4: 5 MB compressed, before 0.7.0
const DEFAULT_LIMIT_MS = 1000; // half of Zemp Lite web's 2 s start-up target
const DEFAULT_RUNS = 5;

function median(numbers) {
  if (numbers.length === 0) throw new Error("no numbers");
  const sorted = [...numbers].sort((a, b) => a - b);
  const middle = sorted.length >> 1;
  return sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
}

function brotli(buffer) {
  return zlib.brotliCompressSync(buffer, {
    params: { [zlib.constants.BROTLI_PARAM_QUALITY]: 11, [zlib.constants.BROTLI_PARAM_SIZE_HINT]: buffer.length },
  }).length;
}

function gzip(buffer) {
  return zlib.gzipSync(buffer, { level: 9 }).length;
}

// Sizes of every .wasm and .js file in the folder, raw and compressed.
function sizes(folder) {
  const files = fs
    .readdirSync(folder)
    .filter((name) => name.endsWith(".wasm") || name.endsWith(".js"))
    .sort();
  const rows = files.map((name) => {
    const buffer = fs.readFileSync(path.join(folder, name));
    return { name, raw: buffer.length, brotli: brotli(buffer), gzip: gzip(buffer) };
  });
  const total = (key) => rows.reduce((sum, row) => sum + row[key], 0);
  return { files: rows, raw: total("raw"), brotli: total("brotli"), gzip: total("gzip") };
}

// The problems (empty when both limits hold).
function evaluate(report, limits) {
  const problems = [];
  if (report.size.brotli > limits.bytes) {
    problems.push(`the compressed bundle is ${report.size.brotli} bytes, over the limit of ${limits.bytes}`);
  }
  if (report.cold_start_ms > limits.ms) {
    problems.push(`the cold start takes ${report.cold_start_ms.toFixed(1)} ms, over the limit of ${limits.ms} ms`);
  }
  return problems;
}

function kilobytes(bytes) {
  return `${(bytes / 1000).toFixed(1)} kB`;
}

function markdown(report, limits, problems) {
  const lines = [
    "### Browser engine bundle and cold start",
    "",
    "| File | Raw | Brotli | Gzip |",
    "| --- | ---: | ---: | ---: |",
    ...report.size.files.map((f) => `| ${f.name} | ${kilobytes(f.raw)} | ${kilobytes(f.brotli)} | ${kilobytes(f.gzip)} |`),
    `| **Total** | ${kilobytes(report.size.raw)} | **${kilobytes(report.size.brotli)}** | ${kilobytes(report.size.gzip)} |`,
    "",
    `Cold start under Node ${report.node}: **${report.cold_start_ms.toFixed(1)} ms** (median of ${report.runs.length} fresh processes: load ${median(report.runs.map((r) => r.load_ms)).toFixed(1)} ms, first call ${median(report.runs.map((r) => r.first_call_ms)).toFixed(1)} ms, a one-line script).`,
    "",
    `The whole conformance report, for information: ${report.report_ms.toFixed(1)} ms (not limited; it grows with every case).`,
    "",
    `Limits: ${kilobytes(limits.bytes)} compressed (risk R4), ${limits.ms} ms cold start. ` +
      (problems.length ? `**Exceeded:** ${problems.join("; ")}.` : "Both hold."),
    "",
  ];
  return lines.join("\n");
}

// One cold start, in a fresh process: load (compile and instantiate) and the first call, which
// is the engine's start on a one-line script (`start()`); the whole conformance report is timed
// after it for information, since its length grows with every case and says nothing about a start.
function once(folder, entry) {
  const t0 = process.hrtime.bigint();
  const engine = require(path.resolve(folder, `${entry}.js`));
  const t1 = process.hrtime.bigint();
  engine.start();
  const t2 = process.hrtime.bigint();
  engine.report();
  const t3 = process.hrtime.bigint();
  process.stdout.write(
    JSON.stringify({ load_ms: Number(t1 - t0) / 1e6, first_call_ms: Number(t2 - t1) / 1e6, report_ms: Number(t3 - t2) / 1e6 }),
  );
}

function coldStarts(folder, entry, count) {
  const runs = [];
  for (let i = 0; i < count; i++) {
    const child = spawnSync(process.execPath, [__filename, "--once", folder, entry], { encoding: "utf8" });
    if (child.status !== 0) throw new Error(`a cold-start run failed:\n${child.stderr}`);
    runs.push(JSON.parse(child.stdout));
  }
  return runs;
}

function option(args, name, fallback) {
  const at = args.indexOf(name);
  return at === -1 ? fallback : Number(args[at + 1]);
}

function main(args) {
  if (args[0] === "--once") return once(args[1], args[2]);
  const [folder, entry, output] = args;
  if (!folder || !entry || !output) {
    console.error("usage: node measure.cjs <folder> <entry module> <output json> [--limit-bytes N] [--limit-ms N] [--runs N]");
    process.exit(2);
  }
  const limits = { bytes: option(args, "--limit-bytes", DEFAULT_LIMIT_BYTES), ms: option(args, "--limit-ms", DEFAULT_LIMIT_MS) };
  const runs = coldStarts(folder, entry, option(args, "--runs", DEFAULT_RUNS));
  const report = {
    node: process.version,
    size: sizes(folder),
    runs,
    cold_start_ms: median(runs.map((r) => r.load_ms + r.first_call_ms)),
    report_ms: median(runs.map((r) => r.report_ms)),
    limits,
  };
  const problems = evaluate(report, limits);
  report.problems = problems;
  fs.mkdirSync(path.dirname(path.resolve(output)), { recursive: true });
  fs.writeFileSync(output, JSON.stringify(report, null, 2) + "\n");
  const table = markdown(report, limits, problems);
  console.log(table);
  if (process.env.GITHUB_STEP_SUMMARY) fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, table + "\n");
  if (problems.length) {
    for (const problem of problems) console.error(`over the limit: ${problem}`);
    process.exit(1);
  }
}

module.exports = { median, evaluate, sizes, markdown };
if (require.main === module) main(process.argv.slice(2));
