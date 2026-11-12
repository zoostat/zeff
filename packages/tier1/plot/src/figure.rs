//! A plot as the bytes of a figure (Language Reference, section 7.6, "Rendering and export";
//! release 0.7.0, step 21d): what `export` asks of a plot. The function here is put into
//! every plot value this package makes (`zeff_core::value::Plot::figure`), so that `export`,
//! which belongs to a package below this one, writes a plot without calling upwards (Engine
//! and Project Specification, section 8.1, rule 5).
//!
//! The scene of the plot is laid out on the page asked for under the journal theme
//! ([`crate::layout`]) and written as SVG ([`crate::svg`]). What cannot be drawn is said in
//! this package's words: a size that is none, and a page too small for labels of 8 points.

use zeff_core::display::format_float;
use zeff_core::value::{FigureFormat, FigureProblem, FigureRequest, FigureSize, Plot};

use crate::layout::{JOURNAL, LayoutError, MM, SMALLEST_PANEL, Size, layout, layout_with_notes};
use crate::scene::scene;
use crate::svg;

/// A length in millimetres as a message writes it: to a tenth.
fn mm(points: f64) -> String {
    format_float((points / MM * 10.0).round() / 10.0)
}

/// The bytes of `plot` as the figure `request` asks for.
pub fn write(plot: &Plot, request: &FigureRequest) -> Result<Vec<u8>, FigureProblem> {
    let size = match request.size {
        FigureSize::OneColumn => Size::OneColumn,
        FigureSize::TwoColumn => Size::TwoColumn,
        FigureSize::Slide => Size::Slide,
        FigureSize::Millimetres(width, height) => Size::Millimetres(width, height),
    };
    let drawn = scene(plot);
    let laid = if request.notes {
        layout_with_notes(&drawn, size, &JOURNAL)
    } else {
        layout(&drawn, size, &JOURNAL)
    };
    let drawing = laid.map_err(|why| match why {
        LayoutError::NotASize => FigureProblem {
            message: "the size of a figure is a width and a height above 0, in millimetres"
                .to_string(),
            label: "not a width and a height".to_string(),
            help: "write `size: [89, 66]` for 89 mm by 66 mm, or one of `one_column`, `two_column` and `slide`".to_string(),
            of_size: true,
        },
        LayoutError::TooSmall { panel } => {
            let (wide, high) = size.millimetres();
            FigureProblem {
                message: format!(
                    "a page of {} by {} mm is too small for this plot: its labels, never set below 8 points, leave {} by {} mm for the panel",
                    format_float(wide),
                    format_float(high),
                    mm(panel.0.max(0.0)),
                    mm(panel.1.max(0.0))
                ),
                label: "too small a page".to_string(),
                help: format!(
                    "a panel needs {} mm each way; give a larger `size:`, as `one_column` (89 by 66 mm) or `two_column` (183 by 110 mm), or shorten the labels",
                    mm(SMALLEST_PANEL)
                ),
                of_size: true,
            }
        }
        LayoutError::Undrawn(why) => FigureProblem {
            message: why,
            label: "a plot this version does not draw".to_string(),
            help: "write the figure with the version of Zeff that made the plot".to_string(),
            of_size: false,
        },
    })?;
    Ok(match request.format {
        FigureFormat::Svg => svg::write(&drawing).into_bytes(),
    })
}
