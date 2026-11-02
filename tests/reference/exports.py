"""Checks of the exported sample files (release 0.4.0, step 13).

`cargo run -p xtask -- export-samples` writes `tests/reference/exports/sample.docx` and
`sample.tex` from the engine's own writers. This script opens the Word file with the standard
library's `zipfile`, which verifies every entry's CRC, and checks what the writer promises: three
parts in their order, stored without compression, no extra fields, the fixed timestamp of
1 January 1980, every part well-formed XML, and the document holding the sample table's title,
headings, rows and notes. The LaTeX file is checked for its rules and its escaped characters.
Continuous integration runs it after rebuilding the files and then compares both with the
committed copies. Run with `python3 -I tests/reference/exports.py`.
"""

import os
import zipfile
import xml.etree.ElementTree as ET

here = os.path.dirname(os.path.abspath(__file__))
folder = os.path.join(here, "exports")

W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
TITLE = "reg · log(wage) ~ educ + exper + exper^2 + female"
HEADINGS = ["term", "estimate", "std.error", "t value", "p value", "95% CI"]
NOTES = [
    "N = 18,179 (233 rows dropped for NA in model variables)",
    "standard errors: classical",
    "R² 0.210 · adjusted R² 0.210",
]


def check_docx():
    path = os.path.join(folder, "sample.docx")
    with zipfile.ZipFile(path) as archive:
        assert archive.testzip() is None, "a CRC does not match"
        names = archive.namelist()
        assert names == ["[Content_Types].xml", "_rels/.rels", "word/document.xml"], names
        for info in archive.infolist():
            assert info.compress_type == zipfile.ZIP_STORED, info.filename
            assert info.date_time == (1980, 1, 1, 0, 0, 0), (info.filename, info.date_time)
            assert info.flag_bits == 0, info.filename
            assert info.extra == b"", info.filename
            assert info.comment == b"", info.filename
            assert info.file_size == info.compress_size, info.filename
            ET.fromstring(archive.read(info.filename))  # well-formed, or it raises
        document = ET.fromstring(archive.read("word/document.xml"))
    texts = [t.text for t in document.iter(f"{{{W}}}t")]
    assert texts[0] == TITLE, texts[0]
    for note in NOTES:
        assert note in texts, note
    rows = document.findall(f".//{{{W}}}tr")
    assert len(rows) == 6, len(rows)
    first = [t.text for t in rows[0].iter(f"{{{W}}}t")]
    assert first == HEADINGS, first
    last = [t.text for t in rows[-1].iter(f"{{{W}}}t")]
    assert last == ["female", "-0.1319", "0.0064", "-20.63", "<0.001", "[-0.1445, -0.1194]"], last
    # Numbers to the right: the estimate column's cells carry the right alignment.
    cells = rows[1].findall(f"{{{W}}}tc")
    jc = cells[1].find(f".//{{{W}}}jc")
    assert jc is not None and jc.get(f"{{{W}}}val") == "right"
    assert cells[0].find(f".//{{{W}}}jc") is None
    with open(path, "rb") as f:
        assert f.read(4) == b"PK\x03\x04"


def check_tex():
    path = os.path.join(folder, "sample.tex")
    with open(path, encoding="utf-8") as f:
        text = f.read()
    assert text.startswith("% requires \\usepackage{booktabs}\n")
    for rule in ["\\toprule", "\\midrule", "\\bottomrule"]:
        assert text.count(rule) == 1, rule
    assert "\\begin{tabular}{lrrrrl}" in text
    assert "95\\% CI" in text, "the percent sign is escaped"
    assert "exper\\textasciicircum{}2" in text, "the caret is escaped"
    assert "$<$0.001" in text, "the comparison sign is in maths mode"
    assert "R\\textsuperscript{2} 0.210 $\\cdot$ adjusted" in text
    assert text.count("\\multicolumn{6}{@{}l@{}}") == 3, "three notes under the rules"
    assert text.endswith("\\end{tabular}\n")
    assert text.count(" \\\\\n") == 1 + 5 + 3, "one heading row, five rows, three notes"


check_docx()
check_tex()
print("exports ok: sample.docx and sample.tex are what the writer promises")


def check_csv():
    """The sample frame saved as CSV, TSV and CSV with a byte-order mark and category codes
    (release 0.6.0, step 15): every field read back by the standard library's csv module, under
    the writing rules of section 3.8 of the Language Reference."""
    import csv

    expected = [
        ["1", "1.5", "true", "plain", "1970-01-01", "1970-01-01T00:00:00", "low"],
        ["2", "0.0", "false", "with, comma", "2024-01-01", "2024-01-01T00:00:00", "high"],
        ["3", "0.1", "", 'with "quotes"', "", "", "mid"],
        ["4", "Inf", "true", "two\nlines", "1900-01-01", "2024-01-01T00:00:00.123456", ""],
        ["5", "123456789.12345679", "false", "", "9999-12-31", "1969-12-31T23:59:59", "high"],
        ["6", "", "true", "", "1970-01-02", "1970-01-02T00:00:00", "low"],
        ["7", "1e300", "", " padded ", "1969-12-31", "1970-01-01T00:00:00.000001", "mid"],
        ["8", "120.0", "false", "naïve café", "2019-04-14", "1970-01-01T00:00:00.999999", "high"],
    ]
    header = ["id", "x", "flag", "note", "day", "at", "level"]
    for name, delimiter, bom, codes in (("sample.csv", ",", False, False), ("sample.tsv", "\t", False, False), ("sample_bom_codes.csv", ",", True, True)):
        path = os.path.join(folder, name)
        with open(path, "rb") as f:
            raw = f.read()
        assert b"\r" not in raw, name + ": a carriage return"
        assert raw.startswith("﻿".encode("utf-8")) == bom, name + ": the byte-order mark"
        with open(path, encoding="utf-8-sig", newline="") as f:
            rows = list(csv.reader(f, delimiter=delimiter))
        assert rows[0] == header, (name, rows[0])
        want = [list(r) for r in expected]
        if codes:
            positions = {"low": "1", "mid": "2", "high": "3", "": ""}
            want = [r[:-1] + [positions[r[-1]]] for r in want]
        assert rows[1:] == want, (name, rows[1:])
    print("the sample CSV, TSV and byte-order-mark files read back field for field")


check_csv()
