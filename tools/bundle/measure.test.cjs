"use strict";
const test = require("node:test");
const assert = require("node:assert/strict");
const { median, evaluate, markdown } = require("./measure.cjs");

const report = (brotli, ms) => ({ size: { brotli }, cold_start_ms: ms });

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
  assert.match(problems[0], /compressed bundle/);
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
  const size = { files: [{ name: "zeff_web_bg.wasm", raw: 1000, brotli: 300, gzip: 400 }], raw: 1000, brotli: 300, gzip: 400 };
  const runs = [{ load_ms: 10, first_call_ms: 5, report_ms: 0 }];
  const limits = { bytes: 5_000_000, ms: 1000 };
  const web = markdown({ web: true, node: "v1", size, runs, cold_start_ms: 15, report_ms: 0 }, limits, []);
  assert.match(web, /^### The engine's worker build: bundle and cold start\n/);
  assert.match(web, /the first call opens a project and runs the script/);
  assert.doesNotMatch(web, /conformance report/);
  const cases = markdown({ web: false, node: "v1", size, runs, cold_start_ms: 15, report_ms: 20 }, limits, []);
  assert.match(cases, /^### Browser engine bundle and cold start\n/);
  assert.match(cases, /The whole conformance report, for information: 20\.0 ms/);
});
