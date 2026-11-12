//! The PNG of a picture (Language Reference, section 7.6, "Rendering and export"; release
//! 0.7.0, step 22b2): the bytes of a figure drawn in dots, the same on every machine.
//!
//! [`write`] turns a picture ([`crate::raster::Picture`]) into the bytes of a PNG file and is
//! a pure function of it and of the number of dots to the inch. The file holds the size of
//! the picture, a note that its colours are those of a screen (sRGB), how many dots there are
//! to the metre, so that a program that places the figure on a page gives it the size it was
//! drawn for, and the dots, eight bits each of red, green and blue, without transparency.
//! Nothing else is written: no date, no name of a program.
//!
//! **The dots.** Each row is written as what it differs by from what a rule foretells of it
//! (nothing, the dot to the left, the dot above, the mean of the two, or the nearest of the
//! two and the dot above to the left), the rule chosen row by row as the one that leaves the
//! smallest numbers, the first of them where two leave as little. The rows are then
//! compressed by DEFLATE, at a fixed level, as the streams of a PDF are.

use miniz_oxide::deflate::compress_to_vec_zlib;

use crate::raster::Picture;

/// How hard the dots are compressed, of 10: the same on every machine.
const LEVEL: u8 = 6;

/// The eight bytes every PNG file begins with.
const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// The check of a chunk, as the format has it (CRC-32), a byte at a time from a table.
const fn table() -> [u32; 256] {
    let mut table = [0_u32; 256];
    let mut byte = 0;
    while byte < 256 {
        let mut value = byte as u32;
        let mut bit = 0;
        while bit < 8 {
            value = if value & 1 == 1 {
                0xEDB8_8320 ^ (value >> 1)
            } else {
                value >> 1
            };
            bit += 1;
        }
        table[byte] = value;
        byte += 1;
    }
    table
}

const TABLE: [u32; 256] = table();

fn check(parts: &[&[u8]]) -> u32 {
    let mut value = u32::MAX;
    for part in parts {
        for &byte in *part {
            value = TABLE[((value ^ u32::from(byte)) & 0xFF) as usize] ^ (value >> 8);
        }
    }
    !value
}

/// A chunk: its length, its name, its bytes and the check of the name and the bytes.
fn chunk(out: &mut Vec<u8>, name: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(name);
    out.extend_from_slice(data);
    out.extend_from_slice(&check(&[name, data]).to_be_bytes());
}

/// Of the dot to the left, the dot above and the dot above to the left, the one nearest to
/// left plus above less the corner: the left first, then the one above, where two are as
/// near.
fn nearest(left: u8, above: u8, corner: u8) -> u8 {
    let (a, b, c) = (i16::from(left), i16::from(above), i16::from(corner));
    let guess = a + b - c;
    let (from_a, from_b, from_c) = ((guess - a).abs(), (guess - b).abs(), (guess - c).abs());
    if from_a <= from_b && from_a <= from_c {
        left
    } else if from_b <= from_c {
        above
    } else {
        corner
    }
}

/// The rows of a picture, each after the number of the rule it is written by.
fn rows(picture: &Picture) -> Vec<u8> {
    let stride = picture.width * 3;
    let mut out = Vec::with_capacity((stride + 1) * picture.height);
    let none = vec![0_u8; stride];
    let mut tries: [Vec<u8>; 5] = std::array::from_fn(|_| vec![0_u8; stride]);
    for row in 0..picture.height {
        let now = &picture.rgb[row * stride..(row + 1) * stride];
        let above = if row == 0 {
            &none[..]
        } else {
            &picture.rgb[(row - 1) * stride..row * stride]
        };
        for i in 0..stride {
            let (left, corner) = if i >= 3 {
                (now[i - 3], above[i - 3])
            } else {
                (0, 0)
            };
            let up = above[i];
            tries[0][i] = now[i];
            tries[1][i] = now[i].wrapping_sub(left);
            tries[2][i] = now[i].wrapping_sub(up);
            tries[3][i] = now[i].wrapping_sub(((u16::from(left) + u16::from(up)) / 2) as u8);
            tries[4][i] = now[i].wrapping_sub(nearest(left, up, corner));
        }
        // The rule that leaves the smallest numbers, each taken as near zero as it is.
        let mut best = 0;
        let mut least = u64::MAX;
        for (rule, tried) in tries.iter().enumerate() {
            let size: u64 = tried
                .iter()
                .map(|&byte| u64::from((byte as i8).unsigned_abs()))
                .sum();
            if size < least {
                best = rule;
                least = size;
            }
        }
        out.push(best as u8);
        out.extend_from_slice(&tries[best]);
    }
    out
}

/// The PNG file of `picture`, drawn at `dots` to the inch; `None` for a picture of no dots,
/// or one whose bytes are not three to a dot.
pub fn write(picture: &Picture, dots: f64) -> Option<Vec<u8>> {
    let (Ok(width), Ok(height)) = (u32::try_from(picture.width), u32::try_from(picture.height))
    else {
        return None;
    };
    let bytes = picture
        .width
        .checked_mul(picture.height)
        .and_then(|n| n.checked_mul(3))?;
    if width == 0 || height == 0 || picture.rgb.len() != bytes {
        return None;
    }
    let mut out = SIGNATURE.to_vec();
    // The size; eight bits to a part; red, green and blue; the one way of compressing and of
    // foretelling the format has; the rows in order.
    let mut header = Vec::with_capacity(13);
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 2, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &header);
    // The colours are those of a screen, to be shown as the eye takes them.
    chunk(&mut out, b"sRGB", &[0]);
    // How many dots to the metre, across and down.
    let to_the_metre = (dots / 0.0254 + 0.5).floor();
    if to_the_metre >= 1.0 && to_the_metre <= f64::from(u32::MAX) {
        let mut density = Vec::with_capacity(9);
        density.extend_from_slice(&(to_the_metre as u32).to_be_bytes());
        density.extend_from_slice(&(to_the_metre as u32).to_be_bytes());
        density.push(1);
        chunk(&mut out, b"pHYs", &density);
    }
    chunk(
        &mut out,
        b"IDAT",
        &compress_to_vec_zlib(&rows(picture), LEVEL),
    );
    chunk(&mut out, b"IEND", &[]);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_check_of_a_chunk_is_the_one_the_format_names() {
        // The check of the nine digits is the one every description of CRC-32 gives.
        assert_eq!(check(&[b"123456789"]), 0xCBF4_3926);
        assert_eq!(check(&[b"1234", b"56789"]), 0xCBF4_3926);
        assert_eq!(check(&[]), 0);
        // The last chunk of every PNG file: no bytes, and this check of its name.
        let mut end = Vec::new();
        chunk(&mut end, b"IEND", &[]);
        assert_eq!(
            end,
            [0, 0, 0, 0, b'I', b'E', b'N', b'D', 0xAE, 0x42, 0x60, 0x82]
        );
    }

    #[test]
    fn the_nearest_of_three_takes_the_left_first_and_then_the_one_above() {
        assert_eq!(nearest(0, 0, 0), 0);
        assert_eq!(nearest(10, 20, 10), 20);
        assert_eq!(nearest(20, 10, 10), 20);
        assert_eq!(nearest(10, 10, 20), 10);
        assert_eq!(nearest(50, 60, 70), 50);
        assert_eq!(nearest(255, 255, 0), 255);
        assert_eq!(nearest(0, 255, 255), 0);
    }

    #[test]
    fn a_row_is_written_by_the_rule_that_leaves_the_smallest_numbers() {
        let picture = |width: usize, height: usize, rgb: Vec<u8>| Picture { width, height, rgb };
        // One colour everywhere: the first row differs from the dot to its left only at its
        // first dot, and every later row not at all from the row above.
        let flat = rows(&picture(3, 2, [200, 100, 50].repeat(6)));
        assert_eq!(flat[..10], [1, 200, 100, 50, 0, 0, 0, 0, 0, 0]);
        assert_eq!(flat[10..], [2, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        // A white row, by the dot to the left: its first dot, and then nothing.
        let white = rows(&picture(2, 1, vec![255; 6]));
        assert_eq!(white, [1, 255, 255, 255, 0, 0, 0]);
        // A black picture is written as it stands, the first rule of those that leave nothing.
        let black = rows(&picture(2, 2, vec![0; 12]));
        assert_eq!(black, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn what_is_no_picture_is_not_written() {
        let picture = |width: usize, height: usize, rgb: Vec<u8>| Picture { width, height, rgb };
        assert!(write(&picture(0, 1, Vec::new()), 300.0).is_none());
        assert!(write(&picture(1, 0, Vec::new()), 300.0).is_none());
        assert!(write(&picture(2, 2, vec![0; 11]), 300.0).is_none());
        let one = write(&picture(1, 1, vec![1, 2, 3]), 300.0).unwrap();
        assert!(one.starts_with(&SIGNATURE));
        // The size of the picture stands right after the name of the first chunk.
        assert_eq!(
            one[8..24],
            [0, 0, 0, 13, b'I', b'H', b'D', b'R', 0, 0, 0, 1, 0, 0, 0, 1]
        );
        // A number of dots to the inch that is none leaves the file without it.
        let bare = write(&picture(1, 1, vec![1, 2, 3]), f64::NAN).unwrap();
        assert_eq!(bare.len() + 21, one.len());
    }
}
