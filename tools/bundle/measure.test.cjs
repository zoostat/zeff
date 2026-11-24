"use strict";
const test = require("node:test");
const assert = require("node:assert/strict");
const { median, evaluate, markdown } = require("./measure.cjs");

// The size held to the limit is the one the host sends; Brotli at its highest is beside it.
const report = (sent, ms) => ({ size: { sent, brotli: Math.round(sent * 0.7) }, cold_start_ms: ms });

test("the median of an odd and an even count", () => {
  assert.equal(median([5, 1, 3]), 3);
  assert.equal(median([4, 1, 3, 2]), 2.5);
});

test("both limits holding gives no problem", () => {
  assert.deepEqual(evaluate(report(4_999_999, 999), { bytes: 5_000_000, ms: 1000 }), []);
  assert.deepEqual(evaluate(report(5_000_000, 1000), { bytes: 5_000_000, ms: 1000 }), []);
});

test("a bundle over the size limit is a problem", () => {
  const problems = evaluate(report(5_000_001, 10), { bytes: 5_000_000, ms: 1000 });
  assert.equal(problems.length, 1);
  assert.match(problems[0], /compressed bundle is 5000001 bytes as the host sends it/);
});

test("the limit is held to what the host sends, not to Brotli at its highest", () => {
  // A bundle that the highest setting brings under the limit and the host's does not.
  const over = { size: { sent: 5_200_000, brotli: 3_700_000 }, cold_start_ms: 10 };
  assert.equal(evaluate(over, { bytes: 5_000_000, ms: 1000 }).length, 1);
  // And the other way round does not arise, but would pass: the limit reads one figure.
  const under = { size: { sent: 4_100_000, brotli: 5_300_000 }, cold_start_ms: 10 };
  assert.deepEqual(evaluate(under, { bytes: 5_000_000, ms: 1000 }), []);
});

test("a file's size as the host sends it lies between its highest compression and its raw size", () => {
  const text = Buffer.from("the engine as a browser loads it, ".repeat(4000));
  const { sent, brotli } = require("./measure.cjs");
  assert.ok(brotli(text) <= sent(text), `${brotli(text)} against ${sent(text)}`);
  assert.ok(sent(text) < text.length);
  assert.equal(sent(text), brotli(text, 2));
});

test("a slow cold start is a problem", () => {
  const problems = evaluate(report(10, 1000.5), { bytes: 5_000_000, ms: 1000 });
  assert.equal(problems.length, 1);
  assert.match(problems[0], /cold start/);
});

test("both limits exceeded give two problems", () => {
  assert.equal(evaluate(report(9_000_000, 5000), { bytes: 5_000_000, ms: 1000 }).length, 2);
});

test("the table of the worker build says what its first call is, and has no report", () => {
  const size = {
    files: [{ name: "zeff_web_bg.wasm", raw: 1000, sent: 410, brotli: 300, gzip: 400 }],
    raw: 1000,
    sent: 410,
    brotli: 300,
    gzip: 400,
  };
  const runs = [{ load_ms: 10, first_call_ms: 5, report_ms: 0 }];
  const limits = { bytes: 5_000_000, ms: 1000 };
  const web = markdown({ web: true, node: "v1", size, runs, cold_start_ms: 15, report_ms: 0 }, limits, []);
  assert.match(web, /^### The engine's worker build: bundle and cold start\n/);
  assert.match(web, /the first call opens a project and runs the script/);
  assert.doesNotMatch(web, /conformance report/);
  assert.match(web, /\| \*\*Total\*\* \| 1\.0 kB \| \*\*0\.4 kB\*\* \| 0\.3 kB \| 0\.4 kB \|/);
  assert.match(web, /As the host sends it: Brotli at setting 2/);
  assert.match(web, /Limits: 5000\.0 kB as the host sends it \(risk R4\)/);
  const cases = markdown({ web: false, node: "v1", size, runs, cold_start_ms: 15, report_ms: 20 }, limits, []);
  assert.match(cases, /^### Browser engine bundle and cold start\n/);
  assert.match(cases, /The whole conformance report, for information: 20\.0 ms/);
});
