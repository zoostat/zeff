// The engine's worker in a browser (release 0.7.0, step 39d): the module worker of
// `worker.js`, with the WebAssembly build of `wasm-bindgen --target web`, loaded by headless
// Chromium from a page that is cross-origin isolated, as the interface will load it. The page
// (`browser.page.html`) runs the worker through its messages and posts what it saw; this file
// serves the page, starts the browser, and holds what was seen to what the tests under Node
// hold of the same worker (`worker.test.mjs`).
//
// Usage: ZEFF_WEB_PKG_WEB=<output of wasm-bindgen --target web> node --test browser.test.mjs
// The browser is the one `ZEFF_BROWSER` names, or the first of Chrome, Chromium and Edge found
// in their usual places. Where none is found the test is skipped, unless
// `ZEFF_BROWSER_REQUIRED` is set, as it is in CI.
import test from "node:test";
import assert from "node:assert/strict";
import http from "node:http";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawn, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const pkg = process.env.ZEFF_WEB_PKG_WEB;

// The browser to start: a path given, or one found.
function browser() {
  if (process.env.ZEFF_BROWSER) return process.env.ZEFF_BROWSER;
  const places = [
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Chromium.app/Contents/MacOS/Chromium",
    "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
  ];
  for (const place of places) if (fs.existsSync(place)) return place;
  for (const name of ["google-chrome", "google-chrome-stable", "chromium", "chromium-browser"]) {
    const found = spawnSync("which", [name], { encoding: "utf8" });
    if (found.status === 0 && found.stdout.trim()) return found.stdout.trim();
  }
  return null;
}

// The files the page needs, each with its type; nothing else is served.
const served = () => ({
  "/": [path.join(here, "browser.page.html"), "text/html; charset=utf-8"],
  "/worker.js": [path.join(here, "worker.js"), "text/javascript"],
  "/host.mjs": [path.join(here, "host.mjs"), "text/javascript"],
  "/pkg/zeff_web.js": [path.join(pkg, "zeff_web.js"), "text/javascript"],
  "/pkg/zeff_web_bg.wasm": [path.join(pkg, "zeff_web_bg.wasm"), "application/wasm"],
});

// A server on this machine alone that answers with the two headers of cross-origin isolation,
// and a promise of what the page posts.
function serve() {
  const files = served();
  let tell;
  const seen = new Promise((resolve) => (tell = resolve));
  const asked = [];
  const server = http.createServer((request, response) => {
    const headers = {
      "Cross-Origin-Opener-Policy": "same-origin",
      "Cross-Origin-Embedder-Policy": "require-corp",
      "Cache-Control": "no-store",
    };
    if (request.method === "POST" && request.url === "/seen") {
      let body = "";
      request.on("data", (piece) => (body += piece));
      request.on("end", () => {
        response.writeHead(204, headers).end();
        tell(JSON.parse(body));
      });
      return;
    }
    asked.push(request.url);
    const file = files[request.url];
    if (request.method !== "GET" || file === undefined) {
      response.writeHead(404, headers).end();
      return;
    }
    response.writeHead(200, { ...headers, "Content-Type": file[1] }).end(fs.readFileSync(file[0]));
  });
  return new Promise((resolve) => {
    server.listen(0, "127.0.0.1", () => resolve({ server, seen, asked, port: server.address().port }));
  });
}

const found = browser();
const required = Boolean(process.env.ZEFF_BROWSER_REQUIRED);

test(
  "the worker runs, is cancelled, keeps its budget and passes a frame in a browser",
  { skip: found === null && !required ? "no Chromium-family browser on this machine" : false },
  async () => {
    assert.ok(pkg, "ZEFF_WEB_PKG_WEB names the folder of the WebAssembly build for the web");
    assert.ok(found, "ZEFF_BROWSER_REQUIRED is set and no browser was found");
    const { server, seen, asked, port } = await serve();
    const profile = fs.mkdtempSync(path.join(os.tmpdir(), "zeff-browser-"));
    const flags = [
      "--headless=new",
      "--disable-gpu",
      "--no-first-run",
      "--no-default-browser-check",
      "--disable-extensions",
      `--user-data-dir=${profile}`,
    ];
    // A runner of CI has no user namespace for the browser's sandbox; the page is this
    // repository's own, served from this machine.
    if (process.platform === "linux") flags.push("--no-sandbox");
    const child = spawn(found, [...flags, `http://127.0.0.1:${port}/`], { stdio: "ignore" });
    let timer;
    const late = new Promise((_, reject) => {
      timer = setTimeout(
        () => reject(new Error(`the page said nothing in two minutes; it asked for ${asked.join(", ") || "nothing"}`)),
        120_000,
      );
    });
    let page;
    try {
      page = await Promise.race([seen, late]);
    } finally {
      clearTimeout(timer);
      child.kill();
      server.close();
      fs.rmSync(profile, { recursive: true, force: true, maxRetries: 5 });
    }
    assert.equal(page.failure, undefined, page.failure);
    // The page is cross-origin isolated, which is what gives it memory to share with the
    // worker: the cancel flag.
    assert.equal(page.isolated, true);
    assert.equal(page.shared, "function");
    assert.match(page.version, /^\d+\.\d+\.\d+/);
    // A run: its outcome, and before it each statement started and finished and the pieces
    // of output, which are the whole of what it printed.
    assert.equal(page.run.exit_code, 0, page.run.stderr);
    assert.equal(page.run.stdout, "one\n2\n");
    assert.match(page.run.stderr, /warning\[W0201\]/);
    assert.deepEqual(page.run.steps, [
      "started", "output", "finished", "started", "finished", "started", "output", "finished",
    ]);
    assert.deepEqual(page.run.first, { type: "started", statement: 1, of: 3, line: 1 });
    assert.equal(page.run.printed, page.run.stdout);
    // A cancellation through the flag, while the worker computes and reads no message; the
    // flag cleared, the next run goes to its end.
    assert.equal(page.cancel.exit_code, 1);
    assert.equal(page.cancel.stdout, "started\n");
    assert.match(page.cancel.stderr, /^error\[Z0705\]: the run was cancelled before pass \d+ of this loop\n/);
    assert.deepEqual(page.cancel.again, { exit_code: 0, stdout: "2\n" });
    // The memory budget: the project's own refuses the shipped wages at 1 MB and admits them
    // at 64 MB; the window's default, from what the browser says of the device, admits them.
    assert.equal(page.budget.refused.exit_code, 1);
    assert.match(
      page.budget.refused.stderr,
      /error\[Z0704\]: this statement may need 4\.4 MB of memory, more than the budget of 1\.0 MB/,
    );
    assert.equal(page.budget.ran.exit_code, 0, page.budget.ran.stderr);
    assert.equal(page.budget.ran.stdout, "18412\n");
    assert.equal(page.budget.usual.exit_code, 0, page.budget.usual.stderr);
    // A frame passed as an Arrow stream: bytes of the page's own, the stream's mark first, the
    // variables' names in its schema, and nothing else said by the worker, which checks that
    // the buffer left it.
    assert.deepEqual(page.frame.frames, [{ name: "pay", rows: 3, variables: 2 }]);
    assert.equal(page.frame.name, "pay");
    assert.equal(page.frame.typed, true);
    assert.ok(page.frame.length > 200, `only ${page.frame.length} bytes`);
    assert.deepEqual(page.frame.head, [0xff, 0xff, 0xff, 0xff]);
    assert.equal(page.frame.shared, false);
    assert.equal(page.frame.names, true);
    assert.deepEqual(page.frame.none, { type: "error", message: "no frame called `n` in the last run" });
    assert.equal(page.frame.besides, 0);
  },
);
