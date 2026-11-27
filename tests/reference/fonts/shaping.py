"""HarfBuzz's answers for the text of a figure (release 0.7.0, step 21a): each text of the list
below, set in each of the three embedded faces by `hb-shape`, as the glyph of each character,
how far it advances, and how far from the place its advance gives it the glyph is drawn, in
font units.

The engine sets a text by one fixed rule (packages/tier1/plot/src/font.rs): the text in its
composed form, each character by the glyph the font's map gives it, each advance corrected by
the kerning with the glyph after it, each mark put on its letter or on the mark before it at
the point the font keeps for the two (from release 0.10.0, step 4c), and nothing else.
HarfBuzz is asked for the same: the features that put another glyph in the place of a
character (ligatures, contextual and localised forms, and the compositions that take the dot
off an i under a mark) are turned off, and `kern`, `mark` and `mkmk` are left on; HarfBuzz
composes a letter and its mark by itself, whatever the features. With them on, HarfBuzz draws
`fi` and `fl` as one glyph each, which the engine does not; the file records both widths of
such a text, so that the difference the rule makes is on record.

In one case the engine goes beyond HarfBuzz, and the tests say where, from a second reader
of the font files and not from this file: a mark drawn above the small letters, for which the
font has no point on the letter before it, is raised by as much as that letter rises above
the small letters. JetBrains Mono has no point for a mark on most Greek letters, and leaves
the hat of β̂ inside the top of the β; HarfBuzz's answer here is the font's, unraised.

HarfBuzz is told that the script is Latin. Left to guess, it takes the script of the first
letter, and for a text that opens with a Greek letter it then finds no feature for marks in
these fonts, which list them under Latin alone; the engine reads a feature wherever the font
lists it, whatever the text.

A line of a figure is set in two faces where one is not enough (release 0.10.0, step 4a; open
decision #234): a character the face of the line lacks is taken from the mono face, letter by
letter. For that rule the file records, for each of the two sans faces and each text of the
second list below, the stretches of one face the line falls into and HarfBuzz's answer for each
stretch in the face that draws it. Which face draws a character is asked of HarfBuzz too, one
letter at a time, a letter being a character with the marks written after it: the first face
when it draws the whole letter without the glyph 0 and moves every mark of it from where its
advance leaves it (in these fonts a mark that is put on a letter is always moved, since its
outline is kept clear of the letter before it); else the mono face when that draws the whole
letter; else the first face when it draws it whole; else each character by the first of the
two that draws it.

Run with `python3 -I tests/reference/fonts/shaping.py`; it needs `hb-shape` (HarfBuzz, from
Homebrew) and writes `shaping.json` beside itself. The file is committed and is not written
again by the continuous integration, which has no HarfBuzz; the tests compare the engine with
it on every target.
"""
import json
import os
import subprocess
import unicodedata

HERE = os.path.dirname(os.path.abspath(__file__))
FONTS = os.path.normpath(os.path.join(HERE, "..", "..", "..", "packages", "tier1", "plot", "fonts"))
FACES = ["AtkinsonHyperlegibleNext-Regular.ttf", "AtkinsonHyperlegibleNext-SemiBold.ttf", "JetBrainsMono-Regular.ttf"]
PLAIN = "-liga,-clig,-calt,-rlig,-ccmp,-locl"

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
    # A mark Unicode has no joined letter for, put on its letter by the font: what a statistician
    # writes, marks above and below, two and three marks on one letter, a mark between two
    # letters that are kerned, a mark on a capital, on a space, and after a character no face has.
    "x\u0304", "x\u0304 and y\u0304", "X\u0304 = 13.4", "p\u0302 = 0.21", "y\u0303", "\u03b2\u0302",
    "\u03bc\u0302 and \u03c0\u0302", "\u03c3\u0302\u00b2", "\u03b8\u0302", "x\u0304\u0302", "x\u0302\u0304\u0307",
    "a\u0328\u0304", "q\u0327\u0301", "T\u0304o", "A\u0302V", "T\u0327\u0304y", "W\u0303a", "i\u0304", "j\u0304",
    " \u0304", "\u4e2d\u0304", "x\u0304\u0304\u0304", "g\u0304", "\u00e9\u0304", "e\u0304\u0301",
]


def shape(face, text, features=None):
    command = ["hb-shape", "--output-format=json", "--no-glyph-names", "--no-clusters", "--script=latn"]
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
    # Letters with marks: on a letter of the first face, on a letter of the mono face, on a
    # letter the first face has and has no point for (μ, π), a mark the mono face alone has,
    # two marks, a mark that opens the line, and a mark after a character neither face has.
    "Means x\u0304 and y\u0304", "\u03b2\u0302 and \u03bc\u0302", "\u03c3\u0302\u00b2 unknown, \u03c0\u0302 = 0.3",
    "X\u0304 \u2192 \u03bc", "\u03b5\u0302 = y \u2212 y\u0302", "x\u0309 and x\u0304\u0309", "\u03b2\u0302\u0304 To",
    "\u0304x", "\u4e2d\u0304 x\u0304", "T\u0304o \u03b8\u0303 Ty",
]


def offsets(answer):
    for glyph in answer:
        assert glyph.get("ay", 0) == 0, glyph
    return [g.get("dx", 0) for g in answer], [g.get("dy", 0) for g in answer]


def whole(face, letter):
    """HarfBuzz's answer for a letter in a face when the face draws all of it, else None."""
    answer = shape(face, letter, PLAIN)
    return answer if answer and all(glyph["g"] != 0 for glyph in answer) else None


def letters(text):
    """The text in its composed form, a letter at a time: a character and the marks after it."""
    out = []
    for character in unicodedata.normalize("NFC", text):
        if out and unicodedata.category(character).startswith("M"):
            out[-1] += character
        else:
            out.append(character)
    return out


def drawn_by(first, letter):
    """The faces that draw the characters of a letter of a line set in `first`."""
    mono = FACES[2]
    lone = unicodedata.category(letter[0]).startswith("M")
    by_first = whole(first, letter)
    if by_first and (lone or all(g.get("dx", 0) != 0 or g.get("dy", 0) != 0 for g in by_first[1:])):
        return [first] * len(letter)
    if whole(mono, letter):
        return [mono] * len(letter)
    if by_first:
        return [first] * len(letter)
    return [first if whole(first, character) or not whole(mono, character) else mono for character in letter]


def mixed(first, text):
    stretches = []
    for letter in letters(text):
        for face, character in zip(drawn_by(first, letter), letter):
            if stretches and stretches[-1][0] == face:
                stretches[-1][1] += character
            else:
                stretches.append([face, character])
    runs = []
    for face, part in stretches:
        plain = shape(face, part, PLAIN)
        dx, dy = offsets(plain)
        runs.append({
            "face": face,
            "text": part,
            "glyphs": [g["g"] for g in plain],
            "advances": [g["ax"] for g in plain],
            "dx": dx,
            "dy": dy,
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
            dx, dy = offsets(plain)
            usual = shape(face, text)
            cases.append({
                "text": text,
                "glyphs": [g["g"] for g in plain],
                "advances": [g["ax"] for g in plain],
                "dx": dx,
                "dy": dy,
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
