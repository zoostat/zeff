# The fonts of the figures

The three font files in this folder are IBM Plex, copied unchanged from the releases of IBM's
repository <https://github.com/IBM/plex>. They are licensed under the SIL Open Font License,
version 1.1, whose text is `LICENSE.txt` beside them, as it came in both archives. "Plex" is a
Reserved Font Name of IBM Corp.; the files are not modified, renamed or subset here.

| File | From | Bytes | SHA-256 |
| --- | --- | --- | --- |
| `IBMPlexSans-Regular.ttf` | `ibm-plex-sans.zip` of the release `@ibm/plex-sans@1.1.0`, `fonts/complete/ttf/` | 200,500 | `975dcda37d80f038dcd143c22e33ca2d97a0cc5a929aace1c749153b0fe1afa5` |
| `IBMPlexSans-SemiBold.ttf` | the same archive and folder | 202,632 | `a20caf8286023a6a7a85e40b1d2a4ae9fc3e3b1f9eda8f4c542dd4986af67bb1` |
| `IBMPlexMono-Regular.ttf` | `ibm-plex-mono.zip` of the release `@ibm/plex-mono@2.5.0`, `fonts/complete/ttf/` | 173,052 | `7c6fbddca4b700be918f5f6183d9bd4464fa427fe435f0b480d77fe2bb8c5a43` |

The archives: `ibm-plex-sans.zip`, 9,921,777 bytes, SHA-256
`fb365d910566e6d199cc2c15579a7dd9a267128e18431a394ed81f1970c69200`; `ibm-plex-mono.zip`,
6,940,652 bytes, SHA-256 `6d23f01257663d8cc49a0d64c22ced630b79e0e2a0ac08a0da86e9a38bbc481c`.
Downloaded on 12 November 2026.

The renderer of `plot` embeds these files and reads them with its own code (`src/font.rs`); a
test checks each file against the hash above, so a changed font cannot pass unnoticed.
