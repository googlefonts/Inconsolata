//! A little logic to apply horizontal stretching to a font.

use kurbo::Affine;

use crate::font::{Font, Glyph, Layer};

fn affine_stretch(stretch: f64) -> Affine {
    Affine::new([stretch, 0., 0., 1., 0., 0.])
}

fn stretch_layer(layer: &mut Layer, stretch: f64) {
    let a = affine_stretch(stretch);
    let a_inv = affine_stretch(stretch.recip());
    layer.width = (layer.width * stretch).round();
    if let Some(ref mut paths) = layer.paths {
        for path in paths {
            for node in &mut path.nodes {
                node.pt = (a * node.pt).round();
            }
        }
    }
    if let Some(ref mut anchors) = layer.anchors {
        for anchor in anchors {
            anchor.position = (a * anchor.position).round();
        }
    }
    if let Some(ref mut guide_lines) = layer.guide_lines {
        for guide_line in guide_lines {
            guide_line.position = (a * guide_line.position).round();
        }
    }
    if let Some(ref mut components) = layer.components {
        for component in components {
            if let Some(ref mut transform) = component.transform {
                // TODO: round the translation component
                *transform = a * *transform * a_inv;
            }
        }
    }
}

fn stretch_glyph(glyph: &mut Glyph, stretch: f64, layer_id: &str) {
    for layer in &mut glyph.layers {
        if layer.layer_id == layer_id {
            stretch_layer(layer, stretch);
        }
    }
}

pub fn stretch(font: &mut Font, stretch: f64, layer_id: &str) {
    for glyph in &mut font.glyphs {
        stretch_glyph(glyph, stretch, layer_id);
    }
}
