# The fonts of the figures

The three font files in this folder are Atkinson Hyperlegible Next (Regular and SemiBold) and
JetBrains Mono (Regular), copied unchanged from their publishers' repositories. Both families
are licensed under the SIL Open Font License, version 1.1; each family's licence text is beside
its files as it stands in its repository, `LICENSE-AtkinsonHyperlegibleNext.txt` and
`LICENSE-JetBrainsMono.txt`. Neither licence reserves a font name. The files are not modified,
renamed or subset here. They are static TrueType faces, with their outlines in `glyf`; neither
is a variable font.

| File | From | Bytes | SHA-256 |
| --- | --- | --- | --- |
| `AtkinsonHyperlegibleNext-Regular.ttf` | <https://github.com/googlefonts/atkinson-hyperlegible-next>, `fonts/ttf/`, at commit `7925f50f649b3813257faf2f4c0b381011f434f1` (21 February 2025; the repository has no tagged release) | 65,068 | `88ed5c31a71584c7772963b02d04bef1eb7e3d2e9c8b9cb204339b1f82cf432c` |
| `AtkinsonHyperlegibleNext-SemiBold.ttf` | the same repository, folder and commit | 66,680 | `8e2fff779f35ac25239656bb0f3b2bd35959a2009e3dd21a12141c3b9030362b` |
| `JetBrainsMono-Regular.ttf` | <https://github.com/JetBrains/JetBrainsMono>, `fonts/ttf/`, at the tag `v2.304` (commit `cd5227bd1f61dff3bbd6c814ceaf7ffd95e947d9`) | 273,900 | `a0bf60ef0f83c5ed4d7a75d45838548b1f6873372dfac88f71804491898d138f` |

The licence texts: `LICENSE-AtkinsonHyperlegibleNext.txt` is `OFL.txt` of the first repository
at the same commit, 4,431 bytes, SHA-256
`aca6a428580965d2297d1b718042dd427c2a9443ece3b0d02d758e161e0c4030`; `LICENSE-JetBrainsMono.txt`
is `OFL.txt` of the second at the same tag, 4,399 bytes, SHA-256
`30f0c136e3c88e422d0791acd97238870f9054a9729bc34cf2ff0d4ed8cac4ad`. Fetched on 27 November 2026.

Until release 0.9.0 the figures were set in IBM Plex Sans and IBM Plex Mono, from IBM's
releases under the same licence; the faces were changed at release 0.10.0 so that a figure is
set in the fonts of the website and of Zemp Lite (open decision #222).

The renderer of `plot` embeds these files and reads them with its own code (`src/font.rs`); a
test checks each file against its hash, so a changed font cannot pass unnoticed.
