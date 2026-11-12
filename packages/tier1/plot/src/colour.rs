//! The colours of a plot (Language Reference, section 7.6, "Defaults"): the eight colours of
//! Okabe and Ito for categories, in a fixed order, viridis for continuous values, grey for `NA`,
//! and the names a constant colour may be written by.
//!
//! Every colour is written as `#` and six hexadecimal digits in capitals, red, green and blue in
//! sRGB, and is looked up in a table, never computed, so that the same value has the same colour
//! on every target.

use zeff_diagnostics::did_you_mean;

/// The colours of categories, in the order they are given out, each with its name in words:
/// the palette of Okabe and Ito (2008), which people with the common forms of colour blindness
/// tell apart. The values are those of `palette.colors(palette = "Okabe-Ito")` of R 4.6.0. Black
/// comes last, so that the first categories are not drawn in the colour of a plot without any,
/// and the palette's grey is kept for `NA`.
pub const OKABE_ITO: [(&str, &str); 8] = [
    ("#E69F00", "orange"),
    ("#56B4E9", "sky blue"),
    ("#009E73", "bluish green"),
    ("#F0E442", "yellow"),
    ("#0072B2", "blue"),
    ("#D55E00", "vermilion"),
    ("#CC79A7", "reddish purple"),
    ("#000000", "black"),
];

/// The colour of the category `NA`: the grey of the palette of Okabe and Ito.
pub const NA_COLOUR: (&str, &str) = ("#999999", "grey");

/// The colour of the category at `position`, counted from 0, with its name: the eight colours
/// in turn, and again from the first for a ninth category.
pub fn category(position: usize) -> (&'static str, &'static str) {
    OKABE_ITO[position % OKABE_ITO.len()]
}

/// The entry of a ramp of `steps` colours for the value `v` on a scale from `lo` to `hi`: the
/// share of the way from `lo` to `hi`, times `steps`, rounded down, and the last entry for `hi`
/// itself, so that each entry takes an equal share of the scale (the rule of matplotlib's
/// colour maps). Where the limits are one value, or the share cannot be computed, the middle
/// entry.
///
/// The width of the scale is taken as it is where it can be held, and from the halves of the
/// limits where it, or the distance of the value from the lower limit, would overflow, as
/// from the lowest number to the highest. Halving is exact
/// but for the very smallest numbers, where it would make two limits one; those are never
/// halved, since their width never overflows. Both ways give the same share where both can
/// be computed.
pub fn ramp_entry(v: f64, lo: f64, hi: f64, steps: usize) -> usize {
    if steps == 0 {
        return 0;
    }
    let whole = hi - lo;
    let direct = (v - lo) / whole;
    let (width, share) = if whole.is_finite() && direct.is_finite() {
        (whole, direct)
    } else {
        let width = hi / 2.0 - lo / 2.0;
        (width, (v / 2.0 - lo / 2.0) / width)
    };
    if !(width > 0.0 && share.is_finite()) {
        return steps / 2;
    }
    let entry = (share.clamp(0.0, 1.0) * steps as f64).floor() as usize;
    entry.min(steps - 1)
}

/// Why a constant colour was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotAColour {
    /// `#` followed by something other than three or six hexadecimal digits.
    Digits,
    /// A name that is not in the list, with the nearest name that is, if one is near.
    Name(Option<&'static str>),
}

/// A colour as a script writes it, read as `#RRGGBB` in capitals: `#` and six hexadecimal
/// digits, `#` and three (each standing for itself doubled, so `#08F` is `#0088FF`), or one of
/// the names of [`NAMED`], in small letters.
pub fn parse(written: &str) -> Result<String, NotAColour> {
    if let Some(digits) = written.strip_prefix('#') {
        if !digits.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(NotAColour::Digits);
        }
        let digits = digits.to_ascii_uppercase();
        return match digits.len() {
            6 => Ok(format!("#{digits}")),
            3 => Ok(format!(
                "#{}",
                digits.chars().flat_map(|c| [c, c]).collect::<String>()
            )),
            _ => Err(NotAColour::Digits),
        };
    }
    match NAMED.binary_search_by(|(name, _)| (*name).cmp(written)) {
        Ok(found) => Ok(NAMED[found].1.to_string()),
        Err(_) => Err(NotAColour::Name(did_you_mean(
            written,
            NAMED.iter().map(|(name, _)| *name),
        ))),
    }
}

/// The name of a colour of [`NAMED`] or of the palette, for the printed plot: the palette's own
/// names first (`sky blue`), and `None` for a colour that has no name.
pub fn name_of(hex: &str) -> Option<&'static str> {
    if hex == NA_COLOUR.0 {
        return Some(NA_COLOUR.1);
    }
    OKABE_ITO
        .iter()
        .find(|(value, _)| *value == hex)
        .map(|(_, name)| *name)
}

/// The names a constant colour may be written by: the 148 named colours of the CSS Color Module
/// Level 4 of the W3C (section 6.1, "Named Colors"), in the order of their names, each with its
/// value. `gray` and `grey` are both there wherever one is.
pub const NAMED: [(&str, &str); 148] = [
    ("aliceblue", "#F0F8FF"),
    ("antiquewhite", "#FAEBD7"),
    ("aqua", "#00FFFF"),
    ("aquamarine", "#7FFFD4"),
    ("azure", "#F0FFFF"),
    ("beige", "#F5F5DC"),
    ("bisque", "#FFE4C4"),
    ("black", "#000000"),
    ("blanchedalmond", "#FFEBCD"),
    ("blue", "#0000FF"),
    ("blueviolet", "#8A2BE2"),
    ("brown", "#A52A2A"),
    ("burlywood", "#DEB887"),
    ("cadetblue", "#5F9EA0"),
    ("chartreuse", "#7FFF00"),
    ("chocolate", "#D2691E"),
    ("coral", "#FF7F50"),
    ("cornflowerblue", "#6495ED"),
    ("cornsilk", "#FFF8DC"),
    ("crimson", "#DC143C"),
    ("cyan", "#00FFFF"),
    ("darkblue", "#00008B"),
    ("darkcyan", "#008B8B"),
    ("darkgoldenrod", "#B8860B"),
    ("darkgray", "#A9A9A9"),
    ("darkgreen", "#006400"),
    ("darkgrey", "#A9A9A9"),
    ("darkkhaki", "#BDB76B"),
    ("darkmagenta", "#8B008B"),
    ("darkolivegreen", "#556B2F"),
    ("darkorange", "#FF8C00"),
    ("darkorchid", "#9932CC"),
    ("darkred", "#8B0000"),
    ("darksalmon", "#E9967A"),
    ("darkseagreen", "#8FBC8F"),
    ("darkslateblue", "#483D8B"),
    ("darkslategray", "#2F4F4F"),
    ("darkslategrey", "#2F4F4F"),
    ("darkturquoise", "#00CED1"),
    ("darkviolet", "#9400D3"),
    ("deeppink", "#FF1493"),
    ("deepskyblue", "#00BFFF"),
    ("dimgray", "#696969"),
    ("dimgrey", "#696969"),
    ("dodgerblue", "#1E90FF"),
    ("firebrick", "#B22222"),
    ("floralwhite", "#FFFAF0"),
    ("forestgreen", "#228B22"),
    ("fuchsia", "#FF00FF"),
    ("gainsboro", "#DCDCDC"),
    ("ghostwhite", "#F8F8FF"),
    ("gold", "#FFD700"),
    ("goldenrod", "#DAA520"),
    ("gray", "#808080"),
    ("green", "#008000"),
    ("greenyellow", "#ADFF2F"),
    ("grey", "#808080"),
    ("honeydew", "#F0FFF0"),
    ("hotpink", "#FF69B4"),
    ("indianred", "#CD5C5C"),
    ("indigo", "#4B0082"),
    ("ivory", "#FFFFF0"),
    ("khaki", "#F0E68C"),
    ("lavender", "#E6E6FA"),
    ("lavenderblush", "#FFF0F5"),
    ("lawngreen", "#7CFC00"),
    ("lemonchiffon", "#FFFACD"),
    ("lightblue", "#ADD8E6"),
    ("lightcoral", "#F08080"),
    ("lightcyan", "#E0FFFF"),
    ("lightgoldenrodyellow", "#FAFAD2"),
    ("lightgray", "#D3D3D3"),
    ("lightgreen", "#90EE90"),
    ("lightgrey", "#D3D3D3"),
    ("lightpink", "#FFB6C1"),
    ("lightsalmon", "#FFA07A"),
    ("lightseagreen", "#20B2AA"),
    ("lightskyblue", "#87CEFA"),
    ("lightslategray", "#778899"),
    ("lightslategrey", "#778899"),
    ("lightsteelblue", "#B0C4DE"),
    ("lightyellow", "#FFFFE0"),
    ("lime", "#00FF00"),
    ("limegreen", "#32CD32"),
    ("linen", "#FAF0E6"),
    ("magenta", "#FF00FF"),
    ("maroon", "#800000"),
    ("mediumaquamarine", "#66CDAA"),
    ("mediumblue", "#0000CD"),
    ("mediumorchid", "#BA55D3"),
    ("mediumpurple", "#9370DB"),
    ("mediumseagreen", "#3CB371"),
    ("mediumslateblue", "#7B68EE"),
    ("mediumspringgreen", "#00FA9A"),
    ("mediumturquoise", "#48D1CC"),
    ("mediumvioletred", "#C71585"),
    ("midnightblue", "#191970"),
    ("mintcream", "#F5FFFA"),
    ("mistyrose", "#FFE4E1"),
    ("moccasin", "#FFE4B5"),
    ("navajowhite", "#FFDEAD"),
    ("navy", "#000080"),
    ("oldlace", "#FDF5E6"),
    ("olive", "#808000"),
    ("olivedrab", "#6B8E23"),
    ("orange", "#FFA500"),
    ("orangered", "#FF4500"),
    ("orchid", "#DA70D6"),
    ("palegoldenrod", "#EEE8AA"),
    ("palegreen", "#98FB98"),
    ("paleturquoise", "#AFEEEE"),
    ("palevioletred", "#DB7093"),
    ("papayawhip", "#FFEFD5"),
    ("peachpuff", "#FFDAB9"),
    ("peru", "#CD853F"),
    ("pink", "#FFC0CB"),
    ("plum", "#DDA0DD"),
    ("powderblue", "#B0E0E6"),
    ("purple", "#800080"),
    ("rebeccapurple", "#663399"),
    ("red", "#FF0000"),
    ("rosybrown", "#BC8F8F"),
    ("royalblue", "#4169E1"),
    ("saddlebrown", "#8B4513"),
    ("salmon", "#FA8072"),
    ("sandybrown", "#F4A460"),
    ("seagreen", "#2E8B57"),
    ("seashell", "#FFF5EE"),
    ("sienna", "#A0522D"),
    ("silver", "#C0C0C0"),
    ("skyblue", "#87CEEB"),
    ("slateblue", "#6A5ACD"),
    ("slategray", "#708090"),
    ("slategrey", "#708090"),
    ("snow", "#FFFAFA"),
    ("springgreen", "#00FF7F"),
    ("steelblue", "#4682B4"),
    ("tan", "#D2B48C"),
    ("teal", "#008080"),
    ("thistle", "#D8BFD8"),
    ("tomato", "#FF6347"),
    ("turquoise", "#40E0D0"),
    ("violet", "#EE82EE"),
    ("wheat", "#F5DEB3"),
    ("white", "#FFFFFF"),
    ("whitesmoke", "#F5F5F5"),
    ("yellow", "#FFFF00"),
    ("yellowgreen", "#9ACD32"),
];

/// Viridis, the colours of a continuous scale, from its dark purple at the lowest value to its
/// yellow at the highest: the 256 colours of the colour map of van der Walt and Smith for
/// matplotlib (2015, released into the public domain under CC0), as `viridis(256)` of the R
/// package viridisLite 0.4.3 writes them.
pub const VIRIDIS: [&str; 256] = [
    "#440154", "#440256", "#450457", "#450559", "#46075A", "#46085C", "#460A5D", "#460B5E",
    "#470D60", "#470E61", "#471063", "#471164", "#471365", "#481467", "#481668", "#481769",
    "#48186A", "#481A6C", "#481B6D", "#481C6E", "#481D6F", "#481F70", "#482071", "#482173",
    "#482374", "#482475", "#482576", "#482677", "#482878", "#482979", "#472A7A", "#472C7A",
    "#472D7B", "#472E7C", "#472F7D", "#46307E", "#46327E", "#46337F", "#463480", "#453581",
    "#453781", "#453882", "#443983", "#443A83", "#443B84", "#433D84", "#433E85", "#423F85",
    "#424086", "#424186", "#414287", "#414487", "#404588", "#404688", "#3F4788", "#3F4889",
    "#3E4989", "#3E4A89", "#3E4C8A", "#3D4D8A", "#3D4E8A", "#3C4F8A", "#3C508B", "#3B518B",
    "#3B528B", "#3A538B", "#3A548C", "#39558C", "#39568C", "#38588C", "#38598C", "#375A8C",
    "#375B8D", "#365C8D", "#365D8D", "#355E8D", "#355F8D", "#34608D", "#34618D", "#33628D",
    "#33638D", "#32648E", "#32658E", "#31668E", "#31678E", "#31688E", "#30698E", "#306A8E",
    "#2F6B8E", "#2F6C8E", "#2E6D8E", "#2E6E8E", "#2E6F8E", "#2D708E", "#2D718E", "#2C718E",
    "#2C728E", "#2C738E", "#2B748E", "#2B758E", "#2A768E", "#2A778E", "#2A788E", "#29798E",
    "#297A8E", "#297B8E", "#287C8E", "#287D8E", "#277E8E", "#277F8E", "#27808E", "#26818E",
    "#26828E", "#26828E", "#25838E", "#25848E", "#25858E", "#24868E", "#24878E", "#23888E",
    "#23898E", "#238A8D", "#228B8D", "#228C8D", "#228D8D", "#218E8D", "#218F8D", "#21908D",
    "#21918C", "#20928C", "#20928C", "#20938C", "#1F948C", "#1F958B", "#1F968B", "#1F978B",
    "#1F988B", "#1F998A", "#1F9A8A", "#1E9B8A", "#1E9C89", "#1E9D89", "#1F9E89", "#1F9F88",
    "#1FA088", "#1FA188", "#1FA187", "#1FA287", "#20A386", "#20A486", "#21A585", "#21A685",
    "#22A785", "#22A884", "#23A983", "#24AA83", "#25AB82", "#25AC82", "#26AD81", "#27AD81",
    "#28AE80", "#29AF7F", "#2AB07F", "#2CB17E", "#2DB27D", "#2EB37C", "#2FB47C", "#31B57B",
    "#32B67A", "#34B679", "#35B779", "#37B878", "#38B977", "#3ABA76", "#3BBB75", "#3DBC74",
    "#3FBC73", "#40BD72", "#42BE71", "#44BF70", "#46C06F", "#48C16E", "#4AC16D", "#4CC26C",
    "#4EC36B", "#50C46A", "#52C569", "#54C568", "#56C667", "#58C765", "#5AC864", "#5CC863",
    "#5EC962", "#60CA60", "#63CB5F", "#65CB5E", "#67CC5C", "#69CD5B", "#6CCD5A", "#6ECE58",
    "#70CF57", "#73D056", "#75D054", "#77D153", "#7AD151", "#7CD250", "#7FD34E", "#81D34D",
    "#84D44B", "#86D549", "#89D548", "#8BD646", "#8ED645", "#90D743", "#93D741", "#95D840",
    "#98D83E", "#9BD93C", "#9DD93B", "#A0DA39", "#A2DA37", "#A5DB36", "#A8DB34", "#AADC32",
    "#ADDC30", "#B0DD2F", "#B2DD2D", "#B5DE2B", "#B8DE29", "#BADE28", "#BDDF26", "#C0DF25",
    "#C2DF23", "#C5E021", "#C8E020", "#CAE11F", "#CDE11D", "#D0E11C", "#D2E21B", "#D5E21A",
    "#D8E219", "#DAE319", "#DDE318", "#DFE318", "#E2E418", "#E5E419", "#E7E419", "#EAE51A",
    "#ECE51B", "#EFE51C", "#F1E51D", "#F4E61E", "#F6E620", "#F8E621", "#FBE723", "#FDE725",
];

#[cfg(test)]
mod tests {
    use super::*;

    fn is_hex(colour: &str) -> bool {
        colour.len() == 7
            && colour.starts_with('#')
            && colour[1..]
                .chars()
                .all(|c| c.is_ascii_digit() || ('A'..='F').contains(&c))
    }

    #[test]
    fn the_tables_hold_colours_in_capitals() {
        assert!(OKABE_ITO.iter().all(|(hex, _)| is_hex(hex)));
        assert!(is_hex(NA_COLOUR.0));
        assert!(NAMED.iter().all(|(_, hex)| is_hex(hex)));
        assert!(VIRIDIS.iter().all(|hex| is_hex(hex)));
    }

    #[test]
    fn the_names_are_in_order_and_in_small_letters() {
        assert!(NAMED.windows(2).all(|pair| pair[0].0 < pair[1].0));
        assert!(
            NAMED
                .iter()
                .all(|(name, _)| name.chars().all(|c| c.is_ascii_lowercase()))
        );
    }

    #[test]
    fn the_palette_is_that_of_okabe_and_ito_with_black_last() {
        // palette.colors(palette = "Okabe-Ito") of R 4.6.0: black, the seven colours, grey.
        let r = [
            "#000000", "#E69F00", "#56B4E9", "#009E73", "#F0E442", "#0072B2", "#D55E00", "#CC79A7",
            "#999999",
        ];
        let ours: Vec<&str> = OKABE_ITO.iter().map(|(hex, _)| *hex).collect();
        assert_eq!(ours[..7], r[1..8]);
        assert_eq!(ours[7], r[0]);
        assert_eq!(NA_COLOUR.0, r[8]);
        // No two of the eight are one colour, and none is the grey of NA.
        for (i, a) in ours.iter().enumerate() {
            assert!(ours[i + 1..].iter().all(|b| a != b));
            assert_ne!(*a, NA_COLOUR.0);
        }
    }

    #[test]
    fn a_ninth_category_takes_the_first_colour_again() {
        assert_eq!(category(0), ("#E69F00", "orange"));
        assert_eq!(category(7), ("#000000", "black"));
        assert_eq!(category(8), category(0));
        assert_eq!(category(17), category(1));
    }

    #[test]
    fn viridis_runs_from_purple_to_yellow() {
        // viridisLite::viridis(256), entries 1, 2, 128, 129, 255 and 256.
        assert_eq!(VIRIDIS[0], "#440154");
        assert_eq!(VIRIDIS[1], "#440256");
        assert_eq!(VIRIDIS[127], "#21908D");
        assert_eq!(VIRIDIS[128], "#21918C");
        assert_eq!(VIRIDIS[254], "#FBE723");
        assert_eq!(VIRIDIS[255], "#FDE725");
        // The ramp stands still twice, where two neighbours round to one colour of eight bits
        // (entries 113 and 114, and 130 and 131, of R's 256), and nowhere else.
        let still: Vec<usize> = (0..255).filter(|&i| VIRIDIS[i] == VIRIDIS[i + 1]).collect();
        assert_eq!(still, [112, 129]);
    }

    #[test]
    fn a_value_takes_the_entry_of_its_share_of_the_scale() {
        assert_eq!(ramp_entry(0.0, 0.0, 1.0, 256), 0);
        assert_eq!(ramp_entry(1.0, 0.0, 1.0, 256), 255);
        assert_eq!(ramp_entry(0.5, 0.0, 1.0, 256), 128);
        assert_eq!(ramp_entry(0.25, 0.0, 1.0, 4), 1);
        assert_eq!(ramp_entry(0.249, 0.0, 1.0, 4), 0);
        // Beyond the limits a value takes the nearest end.
        assert_eq!(ramp_entry(-3.0, 0.0, 1.0, 256), 0);
        assert_eq!(ramp_entry(7.0, 0.0, 1.0, 256), 255);
        // One value, or no width: the middle.
        assert_eq!(ramp_entry(5.0, 5.0, 5.0, 256), 128);
        assert_eq!(ramp_entry(f64::NAN, 0.0, 1.0, 256), 128);
        // The widest scale does not overflow.
        assert_eq!(ramp_entry(f64::MAX, -f64::MAX, f64::MAX, 256), 255);
        assert_eq!(ramp_entry(-f64::MAX, -f64::MAX, f64::MAX, 256), 0);
        assert_eq!(ramp_entry(0.0, -f64::MAX, f64::MAX, 256), 128);
        assert_eq!(ramp_entry(1.0, 0.0, 1.0, 0), 0);
        // The narrowest scale there is, from zero to the smallest number above it, found by
        // the property test of the scene on 12 November 2026: its two ends are its two ends.
        assert_eq!(ramp_entry(0.0, 0.0, 5e-324, 256), 0);
        assert_eq!(ramp_entry(5e-324, 0.0, 5e-324, 256), 255);
        assert_eq!(ramp_entry(-5e-324, -5e-324, 5e-324, 256), 0);
        assert_eq!(ramp_entry(0.0, -5e-324, 5e-324, 256), 128);
        assert_eq!(ramp_entry(5e-324, -5e-324, 5e-324, 256), 255);
        // A value so far beyond a limit that its distance overflows still takes the nearest end.
        assert_eq!(ramp_entry(-f64::MAX, f64::MAX / 2.0, f64::MAX, 256), 0);
        assert_eq!(ramp_entry(f64::MAX, -f64::MAX, -f64::MAX / 2.0, 256), 255);
        // Where the width overflows, the halves: the two ways agree where both can be had.
        assert_eq!(ramp_entry(f64::MAX / 2.0, -f64::MAX, f64::MAX, 256), 192);
        for (v, lo, hi) in [(0.3, 0.1, 0.7), (12.5, -3.0, 40.0), (1e300, 1e299, 8e307)] {
            let direct = ((v - lo) / (hi - lo) * 256.0_f64).floor() as usize;
            assert_eq!(ramp_entry(v, lo, hi, 256), direct);
        }
    }

    #[test]
    fn a_colour_is_read_from_its_name_or_its_digits() {
        assert_eq!(parse("steelblue"), Ok("#4682B4".to_string()));
        assert_eq!(parse("grey"), parse("gray"));
        assert_eq!(parse("rebeccapurple"), Ok("#663399".to_string()));
        assert_eq!(parse("#4682b4"), Ok("#4682B4".to_string()));
        assert_eq!(parse("#08f"), Ok("#0088FF".to_string()));
        assert_eq!(parse("#12345"), Err(NotAColour::Digits));
        assert_eq!(parse("#"), Err(NotAColour::Digits));
        assert_eq!(parse("#12345G"), Err(NotAColour::Digits));
        assert_eq!(parse("#ééé"), Err(NotAColour::Digits));
        assert_eq!(
            parse("steel blue"),
            Err(NotAColour::Name(Some("steelblue")))
        );
        assert_eq!(parse("Red"), Err(NotAColour::Name(Some("red"))));
        assert_eq!(parse(""), Err(NotAColour::Name(None)));
        assert_eq!(parse("zzzzzzzzzz"), Err(NotAColour::Name(None)));
    }

    #[test]
    fn the_colours_of_the_palette_have_names_in_words() {
        assert_eq!(name_of("#56B4E9"), Some("sky blue"));
        assert_eq!(name_of("#999999"), Some("grey"));
        assert_eq!(name_of("#4682B4"), None);
    }
}
