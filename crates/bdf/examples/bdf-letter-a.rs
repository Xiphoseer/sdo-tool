use std::{collections::BTreeMap, iter::FromIterator};

use bdf::{
    xfont::{Slant, Spacing},
    BoundingBox, Char, Font, Size, Width,
};

fn props<V: Clone>(p: &[(&str, V)]) -> BTreeMap<String, V> {
    let it = p.iter().cloned().map(|(k, v)| (k.to_string(), v));
    BTreeMap::from_iter(it)
}

/// Generate the BDF example from https://en.wikipedia.org/wiki/Glyph_Bitmap_Distribution_Format
fn main() {
    let font = Font {
        font_descriptor: bdf::xfont::XFontDescriptor {
            foundry: "gnu".to_string(),
            family_name: "unifont".to_string(),
            weight_name: "medium".to_string(),
            slant: Slant::Roman,
            setwidth_name: "normal".to_string(),
            add_style_name: "".to_string(),
            pixel_size: 16,
            point_size: 160,
            resolution_x: 75,
            resolution_y: 75,
            spacing: Spacing::CharCell,
            average_width: 80,
            charset_registry: "iso10646".to_string(),
            charset_encoding: "1".to_string(),
        },
        properties: props(&[("FONT_ASCENT", 14), ("FONT_DESCENT", 2)]),
        size: Size {
            point_size: 16,
            xdpi: 75,
            ydpi: 75,
        },
        chars: &[Char {
            unicode: 0x41,
            encoding: 65,
            scalable_width: Width { x: 500, y: 0 },
            device_width: Width { x: 8, y: 0 },
            bounding_box: BoundingBox {
                width: 8,
                height: 16,
                xoff: 0,
                yoff: -2,
            },
            pixels: &[
                &[0x00],
                &[0x00],
                &[0x00],
                &[0x00],
                &[0x18],
                &[0x24],
                &[0x24],
                &[0x42],
                &[0x42],
                &[0x7E],
                &[0x42],
                &[0x42],
                &[0x42],
                &[0x42],
                &[0x00],
                &[0x00],
            ],
        }],
        bounding_box: BoundingBox {
            width: 16,
            height: 16,
            xoff: 0,
            yoff: -2,
        },
    };
    print!("{}", font);
}
