// The engine's worker, tested under a worker thread of Node (release 0.7.0, step 33e1).
// Usage: ZEFF_WEB_PKG=<output of wasm-bindgen --target nodejs> node --test worker.test.mjs
import test from "node:test";
import assert from "node:assert/strict";
import { Worker } from "node:worker_threads";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const pkg = process.env.ZEFF_WEB_PKG;
assert.ok(pkg, "ZEFF_WEB_PKG names the folder of the WebAssembly build");

// A worker, what it has said since it was last asked, and a way to wait for a message.
function started(deviceBytes) {
  const worker = new Worker(path.join(here, "worker.node.mjs"), {
    workerData: { pkg: path.resolve(pkg), deviceBytes },
  });
  const said = [];
  let wake = null;
  worker.on("message", (message) => {
    said.push(message);
    if (wake) wake();
  });
  // The next message of a type, waited for; the others stay in `said`.
  const next = async (type) => {
    for (;;) {
      const at = said.findIndex((message) => message.type === type);
      if (at >= 0) return said.splice(at, 1)[0];
      await new Promise((resolve) => (wake = resolve));
    }
  };
  return { worker, said, next };
}

// A worker with a project open.
async function opened(deviceBytes) {
  const { worker, said, next } = started(deviceBytes);
  const ready = await next("ready");
  const cancel = new SharedArrayBuffer(4);
  worker.postMessage({ type: "open", name: "demo", cancel });
  await next("opened");
  return { worker, said, next, ready, flag: new Int32Array(cancel) };
}

const bytes = (text) => new TextEncoder().encode(text);

test("a run tells its steps and what it prints, then its outcome", async () => {
  const { worker, said, next, ready } = await opened(0);
  assert.match(ready.version, /^\d+\.\d+\.\d+/);
  worker.postMessage({
    type: "run",
    script: "scripts/main.zeff",
    text: 'print("one")\nlet d = frame(x = [1.0, NA, 3])\nprint(d.mean(x))\n',
  });
  const done = await next("done");
  assert.equal(done.exit_code, 0, done.stderr);
  assert.equal(done.stdout, "one\n2\n");
  assert.match(done.stderr, /warning\[W0201\]/);
  // Before the outcome: each statement started and finished, and the pieces of output, which
  // are the whole of what the run printed and said.
  assert.deepEqual(
    said.map((message) => message.type),
    ["started", "output", "finished", "started", "finished", "started", "output", "finished"],
  );
  assert.deepEqual(said[0], { type: "started", statement: 1, of: 3, line: 1 });
  const pieces = said.filter((message) => message.type === "output");
  assert.equal(pieces.map((piece) => piece.stdout).join(""), done.stdout);
  assert.equal(pieces.map((piece) => piece.stderr).join(""), done.stderr);
  await worker.terminate();
});

test("the files of the project are given by messages", async () => {
  const { worker, next } = await opened(0);
  worker.postMessage({ type: "file", path: "data/pay.csv", bytes: bytes("id,wage\n1,23.5\n2,19\n3,31.25\n") });
  const module =
    "/// Twice a number.\n///\n/// The number doubled.\n///\n/// x: The number.\n/// returns: The number times two, a Float.\n/// > twice(2.0)\n/// 4\nfn twice(x: Float) -> Float {\n    x * 2\n}\n";
  worker.postMessage({ type: "file", path: "functions/clean.zeff", bytes: bytes(module) });
  worker.postMessage({
    type: "run",
    script: "scripts/main.zeff",
    text: 'use functions::clean\nlet pay = import("data/pay.csv")\nprint(pay.n_rows())\nprint(twice(pay.mean(wage)))\n',
  });
  const done = await next("done");
  assert.equal(done.exit_code, 0, done.stderr);
  assert.equal(done.stdout, "3\n49.1667\n");
  await worker.terminate();
});

test("a run is cancelled between two steps through the flag the two sides share", async () => {
  const { worker, next, flag } = await opened(0);
  worker.postMessage({
    type: "run",
    script: "scripts/main.zeff",
    text: 'print("started")\nfor i in 1..200000 {\n    let rows = dataset(wages).where(year == 2015).n_rows()\n}\nprint("never")\n',
  });
  // The loop is under way: the worker is computing and reads no message, so the interface
  // writes the flag.
  await next("pass");
  Atomics.store(flag, 0, 1);
  const done = await next("done");
  assert.equal(done.exit_code, 1);
  assert.equal(done.stdout, "started\n");
  assert.match(done.stderr, /^error\[Z0705\]: the run was cancelled before pass \d+ of this loop\n/);
  // The flag is the interface's: cleared, the next run goes to its end.
  Atomics.store(flag, 0, 0);
  worker.postMessage({ type: "run", script: "scripts/main.zeff", text: "print(1 + 1)\n" });
  const again = await next("done");
  assert.equal(again.exit_code, 0);
  assert.equal(again.stdout, "2\n");
  await worker.terminate();
});

test("the window has a memory budget by default, and the project's own comes first", async () => {
  // A device said to have 4 MB: the budget is half of it, and the shipped wages file,
  // 875 kB counted five times, does not fit.
  const text = 'let w = import("zeff:wages.csv")\nprint(w.n_rows())\n';
  const small = await opened(4_000_000);
  small.worker.postMessage({ type: "run", script: "scripts/main.zeff", text });
  const refused = await small.next("done");
  assert.equal(refused.exit_code, 1);
  assert.match(
    refused.stderr,
    /error\[Z0704\]: this statement may need 4\.4 MB of memory, more than the budget of 2\.0 MB/,
  );
  // The project sets its own budget in zeff.toml, which comes before the default.
  const manifest = `[project]\nname = "demo"\nzeff = "${small.ready.version}"\nedition = "preview"\nmemory = "64MB"\n`;
  small.worker.postMessage({ type: "file", path: "zeff.toml", bytes: bytes(manifest) });
  small.worker.postMessage({ type: "run", script: "scripts/main.zeff", text });
  const ran = await small.next("done");
  assert.equal(ran.exit_code, 0, ran.stderr);
  assert.equal(ran.stdout, "18412\n");
  await small.worker.terminate();
  // A device that reports nothing has the largest default, where the file fits.
  const unknown = await opened(0);
  unknown.worker.postMessage({ type: "run", script: "scripts/main.zeff", text });
  assert.equal((await unknown.next("done")).exit_code, 0);
  await unknown.worker.terminate();
});

test("a message the worker cannot act on is answered, not passed over", async () => {
  const { worker, said, next } = started(0);
  await next("ready");
  worker.postMessage({ type: "run", script: "scripts/main.zeff", text: "print(1)\n" });
  worker.postMessage({ type: "file", path: "data/x.csv", bytes: new Uint8Array() });
  worker.postMessage({ type: "dance" });
  const answers = [await next("error"), await next("error"), await next("error")];
  assert.deepEqual(answers, [
    { type: "error", message: "a run was asked for before a project was opened" },
    { type: "error", message: "a file was sent before a project was opened" },
    { type: "error", message: "no such message: dance" },
  ]);
  assert.deepEqual(said, []);
  await worker.terminate();
});
