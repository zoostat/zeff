// The messages of the engine's worker (Engine and Project Specification, section 8.3,
// point 7). The interface and the worker speak in messages; this file is the worker's side,
// the same in a Web Worker of the browser and in a worker thread of Node, where it is tested.
//
// Messages in:
//   { type: "open", name, cancel }      a project called `name`; `cancel` is a
//                                       SharedArrayBuffer of four bytes, the cancel flag
//   { type: "file", path, bytes }       a file of the project, as a Uint8Array
//   { type: "run", script, text }       run the text of a script of the project
//   { type: "frame", name }             a frame of the last run, by the name of its binding
// Messages out:
//   { type: "ready", version }          once, when the engine is loaded
//   { type: "opened", name }
//   { type: "started" | "finished" | "pass" | "output", ... }   while a run goes
//   { type: "done", exit_code, stdout, stderr, frames }         its outcome, with the frames
//                                       its bindings hold: [{ name, rows, variables }]
//   { type: "frame", name, bytes }      the frame as an Arrow stream, in a Uint8Array whose
//                                       buffer is passed to the interface and not copied
//   { type: "error", message }          a message the worker could not act on
//
// A run is cancelled by the interface writing 1 into the flag: a worker that is computing
// reads no message, so the engine reads the flag before each step. The flag is the
// interface's: it clears it before it asks for a run, and the worker never writes it.

export function serve(engine, port, deviceBytes) {
  let project = null;
  let flag = null;
  // What the engine calls while it runs.
  globalThis.zeffHost = {
    event(json) {
      port.postMessage(JSON.parse(json));
    },
    cancelled() {
      return flag !== null && Atomics.load(flag, 0) === 1;
    },
  };
  port.onmessage = (message) => {
    const said = message.data ?? message;
    switch (said.type) {
      case "open":
        project = new engine.Engine(said.name);
        flag = said.cancel ? new Int32Array(said.cancel) : null;
        port.postMessage({ type: "opened", name: said.name });
        break;
      case "file":
        if (project === null) {
          port.postMessage({ type: "error", message: "a file was sent before a project was opened" });
          break;
        }
        project.add_file(said.path, said.bytes);
        break;
      case "run":
        if (project === null) {
          port.postMessage({ type: "error", message: "a run was asked for before a project was opened" });
          break;
        }
        port.postMessage(JSON.parse(project.run(said.script, said.text, deviceBytes ?? 0)));
        break;
      case "frame": {
        const bytes = project === null ? undefined : project.frame(said.name);
        if (bytes === undefined) {
          port.postMessage({ type: "error", message: `no frame called \`${String(said.name)}\` in the last run` });
          break;
        }
        // The buffer goes to the interface: named in the transfer list it is moved, and the
        // worker is left with nothing, which is how it is known not to have been copied.
        port.postMessage({ type: "frame", name: said.name, bytes }, [bytes.buffer]);
        if (bytes.byteLength !== 0) {
          port.postMessage({ type: "error", message: `the frame \`${said.name}\` was copied, not passed` });
        }
        break;
      }
      default:
        port.postMessage({ type: "error", message: `no such message: ${String(said.type)}` });
    }
  };
  port.postMessage({ type: "ready", version: engine.version() });
}
