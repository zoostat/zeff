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
FACES = ["IBMPlexSans-Regular.ttf", "IBMPlexSans-SemiBold.ttf", "IBMPlexMono-Regular.ttf"]
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
    reference = {"harfbuzz": version, "features": PLAIN, "texts": len(TEXTS)}
    with open(os.path.join(HERE, "shaping.json"), "w", encoding="utf-8") as out:
        json.dump({"reference": reference, "faces": faces}, out, ensure_ascii=True, indent=1)
        out.write("\n")
    print("wrote", os.path.join(HERE, "shaping.json"), len(TEXTS), "texts,", len(FACES), "faces")


main()
