"""HarfBuzz's answers for the text of a figure (release 0.7.0, step 21a): each text of the list
below, set in each of the three embedded faces by `hb-shape`, as the glyph of each character and
how far it advances, in font units.

The engine sets a text by one fixed rule (packages/tier1/plot/src/font.rs): the text in its
composed form, each character by the glyph the font's map gives it, each advance corrected by
the kerning with the glyph after it, and nothing else. HarfBuzz is asked for the same: the
features that put another glyph in the place of a character (ligatures, contextual and
localised forms) and those that attach marks are turned off, and `kern` is left on; HarfBuzz
composes a letter and its mark by itself, whatever the features. With them on, HarfBuzz draws `fi` and `fl`
as one glyph each, which the engine does not; the file records both widths of such a text, so
that the difference the rule makes is on record.

A line of a figure is set in two faces where one is not enough (release 0.10.0, step 4a; open
decision #234): a character the face of the line lacks is taken from the mono face, letter by
letter. For that rule the file records, for each of the two sans faces and each text of the
second list below, the stretches of one face the line falls into and HarfBuzz's answer for each
stretch in the face that draws it. Which face draws a character is asked of HarfBuzz too, one
character at a time: the first face unless it answers with the glyph 0, then the mono face
unless that answers so as well. None of those texts holds a letter and a mark the faces would
draw by parts, so that a character is one glyph in either face.

Run with `python3 -I tests/reference/fonts/shaping.py`; it needs `hb-shape` (HarfBuzz, from
Homebrew) and writes `shaping.json` beside itself. The file is committed and is not written
again by the continuous integration, which has no HarfBuzz; the tests compare the engine with
it on every target.
"""
import json
import os
import subprocess

HERE = os.path.dirname(os.path.abspath(__file__))
FONTS = os.path.normpath(os.path.join(HERE, "..", "..", "..", "packages", "tier1", "plot", "fonts"))
FACES = ["AtkinsonHyperlegibleNext-Regular.ttf", "AtkinsonHyperlegibleNext-SemiBold.ttf", "JetBrainsMono-Regular.ttf"]
PLAIN = "-liga,-clig,-calt,-rlig,-ccmp,-locl,-mark,-mkmk"

TEXTS = [
    # What a figure writes: titles of axes, tick labels, legends.
    "Hourly wage", "Years of schooling", "log(wage)", "wage / hours", "exper^2",
    "Potential labour-market experience", "Coefficient and 95% confidence interval",
    "0", "0.5", "1.0", "10", "100", "1,000", "18,201", "-0.25", "−0.25", "1e-05", "2.5e+06",
    "0 20 40 60 80 100", "8, 10, 12, 14, 16, 18, 20", "p < 0.001", "R² = 0.47", "n = 18,201",
    "Agriculture", "Manufacturing", "Public administration", "Transport", "Finance", "NA",
    "female", "male", "union: yes", "union: no", "difference", "office", "final", "affine", "fjord",
    # Pairs a font kerns.
    "AV", "AVATAR", "WAVE", "To", "Ty", "Tr", "Te", "Ta", "Yo", "Ye", "Va", "Vo", "We", "LT", "LY",
    "P.", "F,", "r.", "y,", "v.", "w,", "T.", "Y.", "V,", "7.", "1.", "f)", "(j", "\"A\"", "'T'",
    "AT", "TA", "VA", "AY", "YA", "ff", "rt", "ry", "ky", "ox", "xo", "/7", "7/", "L'", "A'",
    # Every printable character of ASCII, in order and against its neighbours.
    "".join(chr(c) for c in range(32, 127)),
    "The quick brown fox jumps over the lazy dog",
    "THE QUICK BROWN FOX JUMPS OVER THE LAZY DOG",
    "Pack my box with five dozen liquor jugs.",
    # Letters with marks, other alphabets, signs.
    "éèêë ñ üöä ß Åøæ ç łąę",
    "École, Straße, Malmö, Łódź, São Paulo, Île-de-France",
    "α β γ δ ε μ σ Σ χ² Ω",
    "Средняя зарплата",
    "± × ÷ ≤ ≥ ≠ ≈ ∞ µ ° % ‰",
    "€ £ ¥ $ ¢", "– — … ‘a’ “b” ′ ″",
    "x² y³ a¹ ½ ¼ ¾", "a\u00a0b", "  two  spaces  ", " ", "",
    # A mark written after its letter, which is joined to it; a joined letter the faces lack,
    # drawn by its parts; a mark alone; and characters no face has.
    "e\u0301", "n\u0303o", "cafe\u0301", "\u01e7", "\u01e6a\u01e7", "g\u030c", "\u0301",
    "\u4e2d\u6587", "wage \u2192 hours", "\U0001F600", "a\tb",
]


def shape(face, text, features=None):
    command = ["hb-shape", "--output-format=json", "--no-glyph-names", "--no-clusters"]
    if features:
        command.append("--features=" + features)
    command += [os.path.join(FONTS, face), "--text=" + text]
    out = subprocess.run(command, check=True, capture_output=True, text=True).stdout
    return json.loads(out) if out.strip() else []


# Lines that may need two faces: Latin text with signs, Greek, arrows, superscripts, Cyrillic
# and characters that only a mono face has, or that neither has.
MIXED = [
    "Hourly wage", "",
    "Estimated coefficient \u03b2 and its 95% interval", "\u03c3\u00b2 = 2.1, \u03b1 = 0.05",
    "\u03c7\u00b2 = 12.4 on 3 df", "\u03b1 \u03b2 \u03b3 \u03b4 \u03b5 \u03b8 \u03bb \u03bc \u03c0 \u03c1 \u03c3 \u03c4 \u03c6 \u03c7 \u03c9",
    "\u0393 \u0394 \u0398 \u039b \u03a3 \u03a6 \u03a8 \u03a9", "training \u2192 tenure \u2192 productivity",
    "a \u2190 b \u2194 c", "x\u2074 + x\u2075, 10\u207b\u2076", "H\u2080: \u03bc\u2081 = \u03bc\u2082",
    "\u0421\u0440\u0435\u0434\u043d\u044f\u044f \u0437\u0430\u0440\u043f\u043b\u0430\u0442\u0430",
    "mean x\u203e", "end \u2400 of \u2409 text", "\u2400\u2401\u2402", "A\u2192V", "\u2192AV\u2192", "To \u03b2 Ty",
    "\u4e2d\u6587 \u03b2", "\U0001F600 wage", "\u2208 and \u21d2",
]


def drawn_by(first, character):
    """The face that draws a character of a line set in `first`."""
    for face in (first, FACES[2]):
        answer = shape(face, character, PLAIN)
        if answer and all(glyph["g"] != 0 for glyph in answer):
            return face
    return first


def mixed(first, text):
    stretches = []
    for character in text:
        face = drawn_by(first, character)
        if stretches and stretches[-1][0] == face:
            stretches[-1][1] += character
        else:
            stretches.append([face, character])
    runs = []
    for face, part in stretches:
        plain = shape(face, part, PLAIN)
        for glyph in plain:
            assert glyph.get("dx", 0) == 0 and glyph.get("dy", 0) == 0 and glyph.get("ay", 0) == 0, (face, part)
        runs.append({
            "face": face,
            "text": part,
            "glyphs": [g["g"] for g in plain],
            "advances": [g["ax"] for g in plain],
            "width": sum(g["ax"] for g in plain),
        })
    return {"text": text, "runs": runs, "width": sum(run["width"] for run in runs)}


def main():
    version = subprocess.run(["hb-shape", "--version"], check=True, capture_output=True, text=True).stdout.split("\n")[0]
    faces = {}
    for face in FACES:
        cases = []
        for text in TEXTS:
            plain = shape(face, text, PLAIN)
            for glyph in plain:
                assert glyph.get("dx", 0) == 0 and glyph.get("dy", 0) == 0 and glyph.get("ay", 0) == 0, (face, text)
            usual = shape(face, text)
            cases.append({
                "text": text,
                "glyphs": [g["g"] for g in plain],
                "advances": [g["ax"] for g in plain],
                "width": sum(g["ax"] for g in plain),
                "width_with_ligatures": sum(g["ax"] for g in usual),
            })
        faces[face] = cases
    lines = {first: [mixed(first, text) for text in MIXED] for first in FACES[:2]}
    reference = {"harfbuzz": version, "features": PLAIN, "texts": len(TEXTS), "lines": len(MIXED)}
    with open(os.path.join(HERE, "shaping.json"), "w", encoding="utf-8") as out:
        json.dump({"reference": reference, "faces": faces, "lines": lines}, out, ensure_ascii=True, indent=1)
        out.write("\n")
    print("wrote", os.path.join(HERE, "shaping.json"), len(TEXTS), "texts,", len(FACES), "faces,", len(MIXED), "lines in two faces")


main()
