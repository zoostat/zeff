"""Reference parses of CSV files, from Python's `csv` module.

RFC 4180 leaves room for interpretation at the edges (quoted line breaks, doubled quotes, a last
line without a line break, empty fields, a byte-order mark). Python's `csv` module is the
reference for how the fields of a file are split; the types of the columns are Zeff's own rules
and are tested in Rust. Run with `python3 -I tests/reference/csv_cases.py`; it writes the files
to `tests/reference/csv/` and the expected fields to `tests/reference/csv_cases.json`.

The Rust test skips blank lines (empty, or white space only) as the reader does, and so does this
script when it writes the expected rows.
"""

import csv
import io
import json
import os
import platform

here = os.path.dirname(os.path.abspath(__file__))
folder = os.path.join(here, "csv")
os.makedirs(folder, exist_ok=True)

# name: (bytes of the file, delimiter)
cases = {
    "plain.csv": (b"id,wage,female\n1,23.4,true\n2,19.0,false\n3,,true\n", ","),
    "quoted.csv": (
        b'name,note,n\n"Smith, John","He said ""hello""",1\n"two\nlines","plain",2\n"",,3\n',
        ",",
    ),
    "crlf.csv": (b"a,b\r\n1,2\r\n3,4\r\n", ","),
    "no_final_newline.csv": (b"a,b\n1,2\n3,4", ","),
    "bom.csv": ("﻿a,b\n1,2\n".encode("utf-8"), ","),
    "semicolon.csv": (b"a;b;c\n1;2;3\n4;5;6\n", ";"),
    "tab.tsv": (b"a\tb\n1\t2\n3\t4\n", "\t"),
    "unicode.csv": (
        "Gross Sales (total),café,日本語,`case`\n1,naïve,東京,x\n2,😀,大阪,y\n".encode("utf-8"),
        ",",
    ),
    "empty_fields.csv": (b"a,b,c\n,,\n1,,3\n,2,\n", ","),
    "trailing_delimiter.csv": (b"a,b,\n1,2,\n", ","),
    "blank_lines.csv": (b"a,b\n\n1,2\n\n\n3,4\n", ","),
    "single_column.csv": (b"a\n1\n\n2\n", ","),
    "spaces.csv": (b"a, b ,c\n 1 ,2, 3\n", ","),
    "quoted_delimiters.csv": (b'a;b\n"1;2";3\n"x";"y;z"\n', ";"),
    "quotes_in_unquoted_field.csv": (b'a,b\nit"s,1\n', ","),
    "escaped_quote_only.csv": (b'a\n""""\n', ","),
    "long_quoted_block.csv": (
        ("a,b\n" + '"' + "line\n" * 5 + '",1\n').encode("utf-8"),
        ",",
    ),
}


def rows_of(data, delimiter):
    text = data.decode("utf-8-sig")  # strips a byte-order mark, as the reader does
    rows = list(csv.reader(io.StringIO(text, newline=""), delimiter=delimiter))
    return [r for r in rows if not (len(r) == 0 or (len(r) == 1 and r[0].strip() == ""))]


out = {
    "reference": {"software": "Python csv module (standard library)", "python": platform.python_version()},
    "files": {},
}
for name, (data, delimiter) in cases.items():
    with open(os.path.join(folder, name), "wb") as f:
        f.write(data)
    out["files"][name] = {"delimiter": delimiter, "rows": rows_of(data, delimiter)}

with open(os.path.join(here, "csv_cases.json"), "w", encoding="utf-8") as f:
    json.dump(out, f, indent=1, ensure_ascii=False)
    f.write("\n")
print("wrote", len(cases), "files and csv_cases.json")
