//! A plot as the bytes of a figure (Language Reference, section 7.6, "Rendering and export";
//! release 0.7.0, step 21d): what `export` asks of a plot. The function here is put into
//! every plot value this package makes (`zeff_core::value::Plot::figure`), so that `export`,
//! which belongs to a package below this one, writes a plot without calling upwards (Engine
//! and Project Specification, section 8.1, rule 5).
//!
//! The scene of the plot is laid out on the page asked for under the journal theme
//! ([`crate::layout`]) and written in the format asked for: as SVG ([`crate::svg`]) or PDF
//! ([`crate::pdf`]), drawn with lines, or as PNG ([`crate::png`]), drawn in dots at 300 to
//! the inch ([`crate::raster`]). What cannot be drawn is said in this package's words: a size
//! that is none, a page too small for labels of 8 points, a page larger than a PDF may have,
//! and a page of more dots than a PNG of Zeff holds.

use zeff_core::display::format_float;
use zeff_core::value::{FigureFormat, FigureProblem, FigureRequest, FigureSize, Plot};

use crate::layout::{JOURNAL, LayoutError, MM, SMALLEST_PANEL, Size, layout, layout_with_notes};
use crate::raster::{DOTS, MOST_DOTS, RasterError, render};
use crate::scene::{Scene, scene};
use crate::{pdf, png, svg};

/// A length in millimetres as a message writes it: to a tenth.
fn mm(points: f64) -> String {
    format_float((points / MM * 10.0).round() / 10.0)
}

/// A figure in dots is dots throughout: a layer flagged to be drawn in dots inside a figure
/// of lines (open decision #118, F4) is, in a PNG, a layer like any other, drawn once.
fn all_in_dots(scene: &mut Scene) {
    for panel in &mut scene.panels {
        for layer in &mut panel.layers {
            layer.raster = false;
        }
    }
}

/// The bytes of `plot` as the figure `request` asks for.
pub fn write(plot: &Plot, request: &FigureRequest) -> Result<Vec<u8>, FigureProblem> {
    let size = match request.size {
        FigureSize::OneColumn => Size::OneColumn,
        FigureSize::TwoColumn => Size::TwoColumn,
        FigureSize::Slide => Size::Slide,
        FigureSize::Millimetres(width, height) => Size::Millimetres(width, height),
    };
    let mut drawn = scene(plot);
    if request.format == FigureFormat::Png {
        all_in_dots(&mut drawn);
    }
    let laid = if request.notes {
        layout_with_notes(&drawn, size, &JOURNAL)
    } else {
        layout(&drawn, size, &JOURNAL)
    };
    let not_a_size = || {
        FigureProblem {
        message: "the size of a figure is a width and a height above 0, in millimetres".to_string(),
        label: "not a width and a height".to_string(),
        help: "write `size: [89, 66]` for 89 mm by 66 mm, or one of `one_column`, `two_column` and `slide`".to_string(),
        of_size: true,
    }
    };
    let drawing = laid.map_err(|why| match why {
        LayoutError::NotASize => not_a_size(),
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
    let (wide, high) = size.millimetres();
    match request.format {
        FigureFormat::Svg => Ok(svg::write(&drawing).into_bytes()),
        FigureFormat::Pdf => {
            if drawing.width > pdf::LARGEST_PAGE || drawing.height > pdf::LARGEST_PAGE {
                return Err(FigureProblem {
                    message: format!(
                        "a page of {} by {} mm is larger than a page of a PDF may be: {} mm each way at most",
                        format_float(wide),
                        format_float(high),
                        mm(pdf::LARGEST_PAGE)
                    ),
                    label: "too large a page for a PDF".to_string(),
                    help: "give a smaller `size:`, or write the figure as `.svg`, which has no largest page".to_string(),
                    of_size: true,
                });
            }
            Ok(pdf::write(&drawing))
        }
        FigureFormat::Png => match render(&drawing, DOTS) {
            Ok(picture) => png::write(&picture, DOTS).ok_or_else(not_a_size),
            Err(RasterError::TooLarge { width, height }) => Err(FigureProblem {
                message: format!(
                    "a page of {} by {} mm is {} by {} dots at {} to the inch, and a PNG holds {} million at most",
                    format_float(wide),
                    format_float(high),
                    format_float(width),
                    format_float(height),
                    format_float(DOTS),
                    MOST_DOTS / 1_000_000
                ),
                label: "too many dots".to_string(),
                help: "give a smaller `size:`, or write the figure as `.pdf` or `.svg`, which are drawn with lines and have no dots".to_string(),
                of_size: true,
            }),
            Err(RasterError::NotASize) => Err(not_a_size()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::{start, with_layer};
    use std::rc::Rc;
    use zeff_columns::{Column, Table};
    use zeff_core::value::{Aesthetic, Frame, Mark, PlotLayer, PlotMapping};

    #[test]
    fn a_figure_in_dots_draws_every_layer_once_and_is_the_picture_of_its_drawing() {
        let numbers = |values: &[f64]| Column::floats(values.iter().map(|v| Some(*v))).0;
        let table = Table::new(vec![
            ("dose".to_string(), numbers(&[1.0, 2.0, 3.0])),
            ("response".to_string(), numbers(&[2.0, 3.5, 7.0])),
        ])
        .unwrap();
        let frame = Frame {
            table: Rc::new(table),
            name: Some("d".to_string()),
        };
        let mapped = |aesthetic, name: &str| PlotMapping {
            aesthetic,
            variable: name.to_string(),
            computed: None,
        };
        let plot = start(
            frame,
            vec![
                mapped(Aesthetic::X, "dose"),
                mapped(Aesthetic::Y, "response"),
            ],
            Vec::new(),
        );
        let (plot, mut flagged) = with_layer(
            &plot,
            PlotLayer {
                mark: Mark::Points,
                data: None,
                alpha: 1.0,
                size: None,
                jitter: 0.0,
                stream: None,
                computed: Vec::new(),
            },
        );
        flagged.panels[0].layers[0].raster = true;
        all_in_dots(&mut flagged);
        assert!(!flagged.panels[0].layers[0].raster);
        // The PNG of a plot is the picture of its drawing at 300 dots to the inch.
        let request = FigureRequest {
            format: FigureFormat::Png,
            size: FigureSize::OneColumn,
            notes: false,
        };
        let drawing = layout(&scene(&plot), Size::OneColumn, &JOURNAL).unwrap();
        let picture = render(&drawing, DOTS).unwrap();
        assert_eq!(
            write(&plot, &request).unwrap(),
            png::write(&picture, DOTS).unwrap()
        );
        // And its PDF and its SVG are those of the same drawing.
        let as_pdf = FigureRequest {
            format: FigureFormat::Pdf,
            ..request
        };
        assert_eq!(write(&plot, &as_pdf).unwrap(), pdf::write(&drawing));
        let as_svg = FigureRequest {
            format: FigureFormat::Svg,
            ..request
        };
        assert_eq!(
            write(&plot, &as_svg).unwrap(),
            svg::write(&drawing).into_bytes()
        );
    }
}
