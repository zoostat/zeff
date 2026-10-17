"use strict";
const test = require("node:test");
const assert = require("node:assert/strict");
const { median, evaluate } = require("./measure.cjs");

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
