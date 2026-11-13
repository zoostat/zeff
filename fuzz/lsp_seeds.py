"""The first inputs of the `lsp` fuzz target: conversations with the language server on the
scripts of the repository, in the two forms the target reads (the messages under their
headers; and the bodies alone, a NUL byte between two).

Usage: python3 fuzz/lsp_seeds.py <folder to write>   (from the root of the repository)
"""

import glob
import json
import os
import sys


def message(method, params=None, id=None):
    body = {"jsonrpc": "2.0", "method": method}
    if id is not None:
        body["id"] = id
    if params is not None:
        body["params"] = params
    return json.dumps(body, ensure_ascii=False, separators=(",", ":")).encode("utf-8")


def conversation(uri, text, changed):
    document = {"uri": uri, "languageId": "zeff", "version": 1, "text": text}
    part = {
        "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 3}},
        "text": "let",
    }
    return [
        message("initialize", {"capabilities": {}}, id=1),
        message("initialized", {}),
        message("textDocument/didOpen", {"textDocument": document}),
        message(
            "textDocument/didChange",
            {"textDocument": {"uri": uri, "version": 2}, "contentChanges": [{"text": changed}]},
        ),
        message(
            "textDocument/didChange",
            {"textDocument": {"uri": uri, "version": 3}, "contentChanges": [part]},
        ),
        message("textDocument/hover", {"textDocument": {"uri": uri}}, id=2),
        # Hover at a place, and the formatting of the document and of a selection.
        message("textDocument/hover", {"textDocument": {"uri": uri}, "position": {"line": 1, "character": 5}}, id=7),
        message("textDocument/formatting", {"textDocument": {"uri": uri}}, id=8),
        message(
            "textDocument/rangeFormatting",
            {"textDocument": {"uri": uri}, "range": {"start": {"line": 1, "character": 0}, "end": {"line": 4, "character": 2}}},
            id=9,
        ),
        # Completion and signature help at a place inside the document, and past its end.
        message("textDocument/completion", {"textDocument": {"uri": uri}, "position": {"line": 2, "character": 4}}, id=4),
        message("textDocument/signatureHelp", {"textDocument": {"uri": uri}, "position": {"line": 2, "character": 4}}, id=5),
        message("textDocument/completion", {"textDocument": {"uri": uri}, "position": {"line": 9999, "character": 9999}}, id=6),
        message("textDocument/didClose", {"textDocument": {"uri": uri}}),
        message("shutdown", id=3),
        message("exit"),
    ]


def main():
    out = sys.argv[1]
    os.makedirs(out, exist_ok=True)
    scripts = sorted(glob.glob("samples/*/scripts/main.zeff") + glob.glob("samples/*/functions/*.zeff"))
    for at, path in enumerate(scripts):
        with open(path, encoding="utf-8") as file:
            text = file.read()
        # The documents the target's project holds, by turns, and one that is no file.
        uri = ["file:///p/scripts/main.zeff", "file:///p/functions/clean.zeff", "untitled:1"][at % 3]
        bodies = conversation(uri, text, text + "d.mean(nope\n")
        name = path.replace("/", "_")
        with open(os.path.join(out, name + ".framed"), "wb") as file:
            for body in bodies:
                file.write(b"Content-Length: %d\r\n\r\n" % len(body) + body)
        # Without the `initialize`, which the target sends itself in this form.
        with open(os.path.join(out, name + ".bodies"), "wb") as file:
            file.write(b"\x00".join(bodies[1:]))
    # Bytes that are no messages.
    with open(os.path.join(out, "no_messages"), "wb") as file:
        file.write(b"hello\r\n\r\nContent-Length: x\r\n\r\nContent-Length: 3\r\n\r\n{{{\x00[1]\x007")


if __name__ == "__main__":
    main()
