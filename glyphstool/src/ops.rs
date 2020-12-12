//! Operations for manipulating fonts.

use std::collections::HashMap;

use crate::font::Font;

pub fn merge(font: &mut Font, other: &Font, layer_id: &str) {
    let mut map = HashMap::new();
    for glyph in &other.glyphs {
        for layer in &glyph.layers {
            if layer.layer_id == layer_id {
                map.insert(glyph.glyphname.to_owned(), layer);
            }
        }
    }

    for glyph in &mut font.glyphs {
        for layer in &mut glyph.layers {
            if layer.layer_id == layer_id {
                *layer = (*map[&glyph.glyphname]).clone();
            }
        }
    }
}
