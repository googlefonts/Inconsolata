//! A bit of scripting to automate a bunch of Inconsolata vf work.
//!
//! Note that this is a submodule of main, rather than in the lib, as it is not
//! generally useful. But it's very likely that logic in here can be adapted into
//! a more general tool.

use std::collections::HashMap;

use kurbo::{Affine, Point, Rect};

use glyphstool::{Component, Font, Glyph, Layer, Node, NodeType, Path, Region};

#[derive(Default)]
struct LayerMap {
    params_to_id: HashMap<(i64, i64), String>,
    id_to_params: HashMap<String, (i64, i64)>,
}

impl LayerMap {
    fn add(&mut self, wght: i64, wdth: i64, id: &str) {
        self.params_to_id.insert((wght, wdth), id.to_string());
        self.id_to_params.insert(id.to_string(), (wght, wdth));
    }

    fn get_id(&self, wght: i64, wdth: i64) -> &str {
        &self.params_to_id[&(wght, wdth)]
    }

    fn get_params(&self, id: &str) -> Option<(i64, i64)> {
        self.id_to_params.get(id).copied()
    }
}

fn affine_stretch(stretch: f64) -> Affine {
    Affine::new([stretch, 0., 0., 1., 0., 0.])
}

fn simple_lerp_path(path0: &Path, path1: &Path, t: f64) -> Path {
    let nodes = path0
        .nodes
        .iter()
        .zip(path1.nodes.iter())
        .map(|(n0, n1)| Node {
            pt: n0.pt.lerp(n1.pt, t),
            node_type: n0.node_type,
        })
        .collect();
    Path {
        closed: path0.closed,
        nodes,
    }
}

fn fix_path(path0: &Path, path1: &Path, t: f64, a: Affine) -> Path {
    let nodes = path0
        .nodes
        .iter()
        .zip(path1.nodes.iter())
        .map(|(n0, n1)| Node {
            pt: (a * n0.pt.lerp(n1.pt, t)).round(),
            node_type: n0.node_type,
        })
        .collect();
    Path {
        closed: path0.closed,
        nodes,
    }
}

fn fix_glyph(glyph: &mut Glyph, layers: &LayerMap) {
    let paths0 = glyph
        .get_layer(layers.get_id(400, 100))
        .unwrap()
        .paths
        .clone();
    // This is actually the 700 from the master, but is stored in 900.
    let paths1 = glyph
        .get_layer(layers.get_id(900, 100))
        .unwrap()
        .paths
        .clone();
    println!("processing glyph {}", glyph.glyphname);
    for layer in &mut glyph.layers {
        if let Some((wght, wdth)) = layers.get_params(&layer.layer_id) {
            let t = (wght as f64 - 400.0) / 300.0;
            let stretch = wdth as f64 / 100.0;
            let a = affine_stretch(stretch);
            println!("  touching layer {}, t = {}", layer.layer_id, t);
            if let Some(ref p0) = paths0 {
                let paths = p0
                    .iter()
                    .zip(paths1.as_ref().unwrap().iter())
                    .map(|(p0, p1)| fix_path(p0, p1, t, a))
                    .collect();
                layer.paths = Some(paths);
            }
            layer.width = wdth as f64 * 5.0;

            // Possibly TODO: lerp the affine from the masters, rather than
            // doing the processing in-place. Not clear whether it makes much
            // difference.
            let a_inv = affine_stretch(stretch.recip());

            if let Some(ref mut anchors) = layer.anchors {
                for anchor in anchors {
                    anchor.position = (a * anchor.position).round();
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
    }
}

fn get_layer_map(font: &Font) -> LayerMap {
    let mut layers = LayerMap::default();
    for master in &font.font_master {
        let wght = master.weight_value;
        let wdth = master.width_value.unwrap_or(100);
        println!("{}: wght {}, wdth {}", master.id, wght, wdth);
        layers.add(wght, wdth, &master.id);
    }
    layers
}

pub fn inco_fix(font: &mut Font) {
    let layers = get_layer_map(font);
    let layer_400_narrow_id = layers.get_id(400, 50);
    for glyph in &mut font.glyphs {
        let narrow = glyph.get_layer(layer_400_narrow_id).unwrap();
        if narrow.width != 250. && !glyph.glyphname.starts_with("_corner") {
            fix_glyph(glyph, &layers);
        }
    }
}

/// Scaling of small alphanumerics follows

const NUM_PAIRS: &[(&str, &str)] = &[
    ("zero", "zerosuperior"),
    ("zero.ss02", "zerosuperior.ss02"),
    // Note: zero form isn't here because it's made by composition
    ("one", "onesuperior"),
    ("two", "twosuperior"),
    ("three", "threesuperior"),
    ("four", "foursuperior"),
    ("five", "fivesuperior"),
    ("six", "sixsuperior"),
    ("seven", "sevensuperior"),
    ("eight", "eightsuperior"),
    ("nine", "ninesuperior"),
];

const ORD_PAIRS: &[(&str, &str)] = &[("a", "ordfeminine"), ("o", "ordmasculine")];

const FRACS: &[(&str, &str, &str)] = &[
    ("one", "two", "onehalf"),
    ("one", "four", "onequarter"),
    ("three", "four", "threequarters"),
];

const CARONS: &[(&str, &str)] = &[("d", "dcaron")];

fn lerp_layers(glyph: &Glyph, weight: f64, width: f64, a: Affine, layers: &LayerMap) -> Vec<Path> {
    let (wt0, wt1, wtt) = if weight < 400.0 {
        (200, 400, (weight - 200.0) / 200.0)
    } else {
        (400, 900, (weight - 400.0) / 500.0)
    };
    let (wd0, wd1, wdt) = if width < 100.0 {
        (50, 100, (width - 50.0) / 50.0)
    } else {
        (100, 200, (width - 100.0) / 100.0)
    };
    let paths00 = glyph
        .get_layer(layers.get_id(wt0, wd0))
        .unwrap()
        .paths
        .clone();
    let paths01 = glyph
        .get_layer(layers.get_id(wt0, wd1))
        .unwrap()
        .paths
        .clone();
    let paths10 = glyph
        .get_layer(layers.get_id(wt1, wd0))
        .unwrap()
        .paths
        .clone();
    let paths11 = glyph
        .get_layer(layers.get_id(wt1, wd1))
        .unwrap()
        .paths
        .clone();
    if let Some(ref p0) = paths00 {
        return p0
            .iter()
            .zip(paths10.as_ref().unwrap().iter())
            .zip(paths01.as_ref().unwrap().iter())
            .zip(paths11.as_ref().unwrap().iter())
            .map(|(((p00, p10), p01), p11)| {
                let p0 = simple_lerp_path(p00, p01, wdt);
                let p1 = simple_lerp_path(p10, p11, wdt);
                fix_path(&p0, &p1, wtt, a)
            })
            .collect();
    }
    // This shouldn't happen.
    Vec::new()
}

fn add_ord_dash(paths: &mut Vec<Path>, wght: i64, wdth: i64) {
    let mut path = Path::new(true);
    let thickness = match wght {
        200 => 24.,
        400 => 54.,
        900 => 107.,
        _ => panic!("unexpected weight"),
    };
    let thickness_fudge = match wdth {
        50 => 0.9,
        100 => 1.0,
        200 => 1.05,
        _ => panic!("unexpected width"),
    };
    let thickness = thickness * thickness_fudge;
    let mut xw = (wght as f64 - 400.0) * 0.025;
    if wdth == 200 {
        xw -= 25.0;
    }
    let x0 = wdth as f64 * 0.76 - xw;
    let x1 = wdth as f64 * 4.25 + xw;
    let yc = 195.0f64;
    let y0 = (yc - 0.7 * thickness).round();
    let y1 = (yc + 0.3 * thickness).round();
    path.add((x0, y0), NodeType::Line);
    path.add((x1, y0), NodeType::Line);
    path.add((x1, y1), NodeType::Line);
    path.add((x0, y1), NodeType::Line);
    paths.push(path);
}

fn add_fraction_ref(layer: &mut Layer) {
    let component = Component {
        name: "fraction".to_string(),
        transform: None,
        other_stuff: Default::default(),
    };
    layer.components = Some(vec![component]);
}

fn add_caron(layer: &mut Layer, wdth: i64) {
    let a = Affine::translate((wdth as f64 * 1.7, 0.0));
    let component = Component {
        name: "caroncomb.alt".to_string(),
        transform: Some(a),
        other_stuff: Default::default(),
    };
    layer.components = Some(vec![component]);
}

pub fn inco_scale(font: &mut Font, subcmd: i32) {
    let layers = get_layer_map(font);

    // This is very cut'n'pasty, reflecting the development process. Obviously this
    // would be cleaned up for a reusable tool.
    match subcmd {
        // small numerics
        0 => {
            for (src, dst) in NUM_PAIRS {
                println!("{} -> {}", src, dst);
                let src_glyph = font.get_glyph(src).expect("glyph not found");
                let mut glyph = src_glyph.clone();
                glyph.glyphname = dst.to_string();
                for layer in &mut glyph.layers {
                    if let Some((wght, wdth)) = layers.get_params(&layer.layer_id) {
                        let dst_wght = (wght as f64 * 1.3).min(1000.0);
                        let dst_wdth = wdth as f64 * 1.1;
                        let x = (wdth as f64 - dst_wdth * 0.62) * 5.0 * 0.5;
                        let a = Affine::new([0.62, 0.0, 0.0, 0.62, x, 246.0]);
                        let paths = lerp_layers(src_glyph, dst_wght, dst_wdth, a, &layers);
                        layer.paths = Some(paths);
                    }
                }
                let dst_glyph = font.get_glyph_mut(dst).expect("dst glyph not found");
                glyph.other_stuff = dst_glyph.other_stuff.clone();
                *dst_glyph = glyph;
            }
        }
        // ordfeminine, ordmasculine
        1 => {
            for (src, dst) in ORD_PAIRS {
                println!("{} -> {}", src, dst);
                let src_glyph = font.get_glyph(src).expect("glyph not found");
                let mut glyph = src_glyph.clone();
                glyph.glyphname = dst.to_string();
                for layer in &mut glyph.layers {
                    if let Some((wght, wdth)) = layers.get_params(&layer.layer_id) {
                        let dst_wght = (wght as f64 * 1.2).min(1000.0);
                        let dst_wdth = wdth as f64 * 0.95;
                        let x = (wdth as f64 - dst_wdth * 0.77) * 5.0 * 0.5;
                        let a = Affine::new([0.77, 0.0, 0.0, 0.77, x, 267.0]);
                        let mut paths = lerp_layers(src_glyph, dst_wght, dst_wdth, a, &layers);
                        add_ord_dash(&mut paths, wght, wdth);
                        layer.paths = Some(paths);
                    }
                }
                let dst_glyph = font.get_glyph_mut(dst).expect("dst glyph not found");
                glyph.other_stuff = dst_glyph.other_stuff.clone();
                *dst_glyph = glyph;
            }
        }
        // fractions
        2 => {
            for (num, denom, dst) in FRACS {
                println!("{} / {} -> {}", num, denom, dst);
                let num_glyph = font.get_glyph(num).expect("glyph not found");
                let denom_glyph = font.get_glyph(denom).expect("glyph not found");
                let mut glyph = num_glyph.clone();
                glyph.glyphname = dst.to_string();
                for layer in &mut glyph.layers {
                    if let Some((wght, wdth)) = layers.get_params(&layer.layer_id) {
                        let dst_wght = (wght as f64 * 1.5).min(1100.0);
                        let dst_wdth = wdth as f64 * 1.1;
                        let x = (wdth as f64 - dst_wdth * 0.49) * 5.0 * 0.5;
                        let dx = (wdth as f64) * 1.4;
                        let dx2 = match *denom {
                            "two" => dx * 0.93,
                            "four" => dx * 0.85,
                            _ => 0.0,
                        };
                        let a = Affine::new([0.49, 0.0, 0.0, 0.49, x - dx, 380.0]);
                        let mut paths = lerp_layers(num_glyph, dst_wght, dst_wdth, a, &layers);
                        let a = Affine::new([0.49, 0.0, 0.0, 0.49, x + dx2, -70.0]);
                        let denom_paths = lerp_layers(denom_glyph, dst_wght, dst_wdth, a, &layers);
                        paths.extend(denom_paths);
                        layer.paths = Some(paths);
                        add_fraction_ref(layer);
                    }
                }
                let dst_glyph = font.get_glyph_mut(dst).expect("dst glyph not found");
                glyph.other_stuff = dst_glyph.other_stuff.clone();
                *dst_glyph = glyph;
            }
        }
        // ascender caron letters
        3 => {
            for (src, dst) in CARONS {
                println!("{} -> {}", src, dst);
                let src_glyph = font.get_glyph(src).expect("glyph not found");
                let mut glyph = src_glyph.clone();
                glyph.glyphname = dst.to_string();
                for layer in &mut glyph.layers {
                    if let Some((wght, wdth)) = layers.get_params(&layer.layer_id) {
                        let dst_wght = (wght as f64 * 1.05).min(1000.0);
                        let dst_wdth = wdth as f64 * 0.8;
                        let x = (wdth as f64 - dst_wdth * 1.0) * 5.0 * 0.0;
                        let a = Affine::new([1.0, 0.0, 0.0, 1.0, x, 0.0]);
                        let paths = lerp_layers(src_glyph, dst_wght, dst_wdth, a, &layers);
                        layer.paths = Some(paths);
                        add_caron(layer, wdth);
                        layer.anchors = None;
                    }
                }
                let dst_glyph = font.get_glyph_mut(dst).expect("dst glyph not found");
                glyph.other_stuff = dst_glyph.other_stuff.clone();
                *dst_glyph = glyph;
            }
        }
        _ => {
            panic!("unknown subcmd");
        }
    }
}

/// Bézier point distance for circular arcs.
const KAPPA: f64 = 4.0 * (std::f64::consts::SQRT_2 - 1.0) / 3.0;

// A way to organize state for box drawing
struct BoxDraw {
    wght: i64,
    wdth: i64,

    region: Region,
}

#[derive(Clone, Copy)]
enum BoxType {
    Empty,
    Light,
    Double,
    Heavy,
}

impl BoxDraw {
    fn new(wght: i64, wdth: i64) -> BoxDraw {
        let region = Default::default();
        BoxDraw { wght, wdth, region }
    }

    // Line width of light line
    fn light(&self) -> f64 {
        let thickness = match self.wght {
            200 => 60.,
            400 => 120.,
            900 => 180.,
            _ => panic!("unexpected weight"),
        };
        let thickness_fudge = match self.wdth {
            50 => 0.6,
            100 => 1.0,
            200 => 1.1,
            _ => panic!("unexpected width"),
        };
        thickness * thickness_fudge
    }

    fn heavy(&self) -> f64 {
        self.light() * 2.0
    }

    // here, x and y are percent of width, y and w in actual units
    fn hline(&mut self, x0: f64, x1: f64, y: f64, w: f64) {
        let sx0 = (self.wdth as f64 * x0 * 0.05).round();
        let sx1 = (self.wdth as f64 * x1 * 0.05).round();
        let y0 = (y - 0.5 * w).round();
        let y1 = (y + 0.5 * w).round();
        self.rect(sx0, y0, sx1, y1);
    }

    // Based on Source Code Pro box drawing logic
    fn dashed_hline(&mut self, step: i64, thickness: f64) {
        let width = self.wdth as f64 * 5.0;
        let step_length = width / (step as f64);
        let gap = step_length / (step as f64);
        let yc = 300.0;
        let y0 = (yc - 0.5 * thickness).round();
        let y1 = (yc + 0.5 * thickness).round();
        for i in 0..step {
            let x0 = i as f64 * step_length + gap / 2.0;
            let x1 = x0 + step_length - gap;
            self.rect(x0, y0, x1, y1);
        }
    }

    fn dashed_vline(&mut self, step: i64, thickness: f64) {
        let height = 1400.0;
        let step_length = height / (step as f64);
        let gap = step_length / (step as f64);
        let xc = self.wdth as f64 * 2.5;
        let x0 = (xc - 0.5 * thickness).round();
        let x1 = (xc + 0.5 * thickness).round();
        for i in 0..step {
            let y0 = -400.0 + i as f64 * step_length + gap / 2.0;
            let y1 = y0 + step_length - gap;
            self.rect(x0, y0, x1, y1);
        }
    }

    // Assume left->right dir. Based on Source Code Pro box drawing logic
    fn diagonal(&self, start: Point, end: Point, width: f64) -> Path {
        let mut path = Path::new(true);
        let diag = (end - start).hypot();
        let angle = ((end.x - start.x) / diag).asin();
        let dx = (width * 0.5 / angle.cos()).round();
        let dy = (width * 0.5 / angle.sin()).round().copysign(end.y - start.y);
        path.add((start.x + dx, start.y), NodeType::Line);
        path.add((end.x, end.y - dy), NodeType::Line);
        path.add(end, NodeType::Line);
        path.add((end.x - dx, end.y), NodeType::Line);
        path.add((start.x, start.y + dy), NodeType::Line);
        path.add(start, NodeType::Line);
        if end.y < start.y {
            path.rotate_left(3);
            path.reverse();
        }
        path
    }

    fn arc(&self, flip_x: bool, flip_y: bool) -> Vec<Path> {
        let h = (self.light() * 0.5).round(); // half-width
        let r = (self.wdth as f64 * 2.5).round();
        let y0 = -400.0;
        let yc = 300.0;
        let x0 = self.wdth as f64 * -0.8;
        let xc = self.wdth as f64 * 2.5;
        let mut path = Path::new(true);
        let mut p = |x: f64, y: f64, nt| {
            let x = if flip_x { 2.0 * xc - x } else { x };
            let y = if flip_y { 2.0 * yc - y } else { y };
            path.add((x.round(), y.round()), nt);
        };
        let (ls, cs) = if flip_x == flip_y {
            (NodeType::LineSmooth, NodeType::CurveSmooth)
        } else {
            (NodeType::CurveSmooth, NodeType::LineSmooth)
        };
        p(xc - h, y0, NodeType::Line);
        p(xc + h, y0, NodeType::Line);
        p(xc + h, yc - r, ls);
        p(xc + h, yc - r + KAPPA * (r + h), NodeType::OffCurve);
        p(xc - r + KAPPA * (r + h), yc + h, NodeType::OffCurve);
        p(xc - r, yc + h, cs);
        p(x0, yc + h, NodeType::Line);
        p(x0, yc - h, NodeType::Line);
        p(xc - r, yc - h, ls);
        p(xc - r + KAPPA * (r - h), yc - h, NodeType::OffCurve);
        p(xc - h, yc - r + KAPPA * (r - h), NodeType::OffCurve);
        p(xc - h, yc - r, cs);
        if flip_x != flip_y {
            path.reverse();
        }
        vec![path]
    }

    // Used for "up" also
    fn dnblock(&mut self, y0: i64, y1: i64) {
        self.quadrant(0, 8, y0, y1)
    }

    fn lrblock(&mut self, x0: i64, x1: i64) {
        self.quadrant(x0, x1, 0, 8)
    }

    fn quadrant(&mut self, x0: i64, x1: i64, y0: i64, y1: i64) {
        let w = (y1 - y0) as f64 * 175.0;
        let y = (y1 + y0) as f64 * 175.0 * 0.5 - 400.0;
        self.hline(x0 as f64 * 12.5, x1 as f64 * 12.5, y, w);
    }

    fn rect(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) {
        self.region = self.region.add(Rect::new(x0, y0, x1, y1).round());
    }

    /// The general function for drawing most boxes
    fn bxd(&mut self, top: BoxType, left: BoxType, right: BoxType, bot: BoxType) {
        use BoxType::*;
        let light = self.light() * 0.5; // half-width!
        let heavy = self.heavy() * 0.5;
        let mut dbly = light;
        if self.wdth == 50 && self.wght == 900 {
            dbly *= 0.8;
        }
        let dblx = dbly;
        let wdth = self.wdth as f64;
        let xc = wdth * 2.5;
        let yc = 300.0;
        let yb = -400.0;
        let yt = 1000.0;
        let xl = -0.8 * wdth;
        let xr = 5.8 * wdth;
        let dblspy = wdth * 1.2;
        let dblspx = dblspy;
        // counterclockwise starting from this arm
        let yoff = |a, b, c, d| match (a, b, c, d) {
            (Double, Double, _, _) => dbly - dblspy,
            (Double, Empty, Empty, Double) => dbly + dblspy,
            (Light, Double, Empty, Empty) => dbly + dblspy,
            (Light, Empty, Empty, Double) => dbly + dblspy,
            (Light, Double, Empty, Double) => dbly - dblspy,
            (Light, _, _, _) => light,
            (Heavy, Heavy, _, _) => heavy,
            (Heavy, _, _, Heavy) => heavy,
            (Heavy, _, _, _) => light,
            _ => 0.0,
        };
        let xoff = |a, b, c, d| match (a, b, c, d) {
            (Double, Double, _, _) => dblx - dblspx,
            (Double, Empty, Empty, Double) => dblx + dblspx,
            (Light, Double, Empty, Empty) => dblx + dblspx,
            (Light, Empty, Empty, Double) => dblx + dblspx,
            (Light, Double, Empty, Double) => dblx - dblspx,
            (Light, _, _, _) => light,
            (Heavy, Heavy, _, _) => heavy,
            (Heavy, _, _, Heavy) => heavy,
            (Heavy, _, _, _) => light,
            _ => 0.0,
        };
        match top {
            Empty => (),
            Light => {
                let y = yc - yoff(top, left, bot, right);
                self.rect(xc - light, y, xc + light, yt);
            }
            Heavy => {
                let y = yc - yoff(top, left, bot, right);
                self.rect(xc - heavy, y, xc + heavy, yt);
            }
            Double => {
                let y = yc - yoff(top, left, bot, right);
                self.rect(xc - dblx - dblspx, y, xc + dblx - dblspx, yt);
                let y = yc - yoff(top, right, bot, left);
                self.rect(xc - dblx + dblspx, y, xc + dblx + dblspx, yt);
            }
        }
        match left {
            Empty => (),
            Light => {
                let x = xc + xoff(left, bot, right, top);
                self.rect(xl, yc - light, x, yc + light);
            }
            Heavy => {
                let x = xc + xoff(left, bot, right, top);
                self.rect(xl, yc - heavy, x, yc + heavy);
            }
            Double => {
                let x = xc + xoff(left, bot, right, top);
                self.rect(xl, yc - dbly - dblspy, x, yc + dbly - dblspy);
                let x = xc + xoff(left, top, right, bot);
                self.rect(xl, yc - dbly + dblspy, x, yc + dbly + dblspy);
            }
        }
        match right {
            Empty => (),
            Light => {
                let x = xc - xoff(right, top, left, bot);
                self.rect(x, yc - light, xr, yc + light);
            }
            Heavy => {
                let x = xc - xoff(right, top, left, bot);
                self.rect(x, yc - heavy, xr, yc + heavy);
            }
            Double => {
                let x = xc - xoff(right, top, left, bot);
                self.rect(x, yc - dbly + dblspy, xr, yc + dbly + dblspy);
                let x = xc - xoff(right, bot, left, top);
                self.rect(x, yc - dbly - dblspy, xr, yc + dbly - dblspy);
            }
        }
        match bot {
            Empty => (),
            Light => {
                let y = yc + yoff(bot, right, top, left);
                self.rect(xc - light, yb, xc + light, y);
            }
            Heavy => {
                let y = yc + yoff(bot, right, top, left);
                self.rect(xc - heavy, yb, xc + heavy, y);
            }
            Double => {
                let y = yc + yoff(bot, right, top, left);
                self.rect(xc - dblx + dblspx, yb, xc + dblx + dblspx, y);
                let y = yc + yoff(bot, left, top, right);
                self.rect(xc - dblx - dblspx, yb, xc + dblx - dblspx, y);
            }
        }
    }

    fn shade(&mut self, w200: f64, w400: f64, w900: f64) {
        let unit = match self.wght {
            200 => w200,
            400 => w400,
            900 => w900,
            _ => panic!("unexpected weight"),
        };
        let half = 0.5 * unit; // TODO: adjust based on weight
        let halfx = half * (self.wdth as f64) / 100.0;
        for j in 0..14 {
            for i in 0..3 {
                let xc = ((4 * i + 2 * (j & 1) + 1) * self.wdth) as f64 * 5.0 / 12.0;
                let yc = j as f64 * 100.0 - 350.0;
                self.rect(xc - halfx, yc - half, xc + halfx, yc + half);
            }
        }
    }

    fn draw(&mut self, glyphname: &str) -> Option<Vec<Path>> {
        use BoxType::*;

        match glyphname {
            "dneighthblock" => self.dnblock(0, 1),
            "dnquarterblock" => self.dnblock(0, 2),
            "dnthreeeighthsblock" => self.dnblock(0, 3),
            "dnhalfblock" => self.dnblock(0, 4),
            "dnfiveeighthsblock" => self.dnblock(0, 5),
            "dnthreequartersblock" => self.dnblock(0, 6),
            "dnseveneighthsblock" => self.dnblock(0, 7),
            "fullblock" => self.dnblock(0, 8),
            "uphalfblock" => self.dnblock(4, 8),
            "upeighthblock" => self.dnblock(7, 8),
            "lefteighthblock" => self.lrblock(0, 1),
            "leftquarterblock" => self.lrblock(0, 2),
            "leftthreeeighthsblock" => self.lrblock(0, 3),
            "lefthalfblock" => self.lrblock(0, 4),
            "leftfiveeighthsblock" => self.lrblock(0, 5),
            "leftthreequartersblock" => self.lrblock(0, 6),
            "leftseveneighthsblock" => self.lrblock(0, 7),
            "righthalfblock" => self.lrblock(4, 8),
            "righteighthblock" => self.lrblock(7, 8),
            "dnleftquadrant" => self.quadrant(0, 4, 0, 4),
            "dnrightquadrant" => self.quadrant(4, 8, 0, 4),
            "upleftquadrant" => self.quadrant(0, 4, 4, 8),
            "uprightquadrant" => self.quadrant(4, 8, 4, 8),
            "upleftdnrightquadrant" => {
                self.quadrant(0, 4, 4, 8);
                self.quadrant(4, 8, 0, 4);
            }
            "uprightdnleftquadrant" => {
                self.quadrant(4, 8, 4, 8);
                self.quadrant(0, 4, 0, 4);
            }
            "upleftdnleftdnrightquadrant" => {
                self.quadrant(0, 4, 4, 8);
                self.quadrant(0, 4, 0, 4);
                self.quadrant(4, 8, 0, 4);
            }
            "upleftuprightdnleftquadrant" => {
                self.quadrant(0, 4, 4, 8);
                self.quadrant(4, 8, 4, 8);
                self.quadrant(0, 4, 0, 4);
            }
            "upleftuprightdnrightquadrant" => {
                self.quadrant(0, 4, 4, 8);
                self.quadrant(4, 8, 4, 8);
                self.quadrant(4, 8, 0, 4);
            }
            "uprightdnleftdnrightquadrant" => {
                self.quadrant(4, 8, 4, 8);
                self.quadrant(0, 4, 0, 4);
                self.quadrant(4, 8, 0, 4);
            }
            // TODO: three quadrant pieces (L shapes)
            "lightdnbxd" => self.bxd(Empty, Empty, Empty, Light),
            "lightdnhorzbxd" => self.bxd(Empty, Light, Light, Light),
            "lightdnleftbxd" => self.bxd(Empty, Light, Empty, Light),
            "lightdnrightbxd" => self.bxd(Empty, Empty, Light, Light),
            "lightvertbxd" => self.bxd(Light, Empty, Empty, Light),
            "lighthorzbxd" => self.bxd(Empty, Light, Light, Empty),
            "lightleftbxd" => self.bxd(Empty, Light, Empty, Empty),
            "lightrightbxd" => self.bxd(Empty, Empty, Light, Empty),
            "lightupbxd" => self.bxd(Light, Empty, Empty, Empty),
            "lightuphorzbxd" => self.bxd(Light, Light, Light, Empty),
            "lightupleftbxd" => self.bxd(Light, Light, Empty, Empty),
            "lightuprightbxd" => self.bxd(Light, Empty, Light, Empty),
            "lightverthorzbxd" => self.bxd(Light, Light, Light, Light),
            "lightvertleftbxd" => self.bxd(Light, Light, Empty, Light),
            "lightvertrightbxd" => self.bxd(Light, Empty, Light, Light),

            "heavydnbxd" => self.bxd(Empty, Empty, Empty, Heavy),
            "heavydnhorzbxd" => self.bxd(Empty, Heavy, Heavy, Heavy),
            "heavydnleftbxd" => self.bxd(Empty, Heavy, Empty, Heavy),
            "heavydnrightbxd" => self.bxd(Empty, Empty, Heavy, Heavy),
            "heavyvertbxd" => self.bxd(Heavy, Empty, Empty, Heavy),
            "heavyhorzbxd" => self.bxd(Empty, Heavy, Heavy, Empty),
            "heavyleftbxd" => self.bxd(Empty, Heavy, Empty, Empty),
            "heavyrightbxd" => self.bxd(Empty, Empty, Heavy, Empty),
            "heavyupbxd" => self.bxd(Heavy, Empty, Empty, Empty),
            "heavyuphorzbxd" => self.bxd(Heavy, Heavy, Heavy, Empty),
            "heavyupleftbxd" => self.bxd(Heavy, Heavy, Empty, Empty),
            "heavyuprightbxd" => self.bxd(Heavy, Empty, Heavy, Empty),
            "heavyverthorzbxd" => self.bxd(Heavy, Heavy, Heavy, Heavy),
            "heavyvertleftbxd" => self.bxd(Heavy, Heavy, Empty, Heavy),
            "heavyvertrightbxd" => self.bxd(Heavy, Empty, Heavy, Heavy),

            "dbldnhorzbxd" => self.bxd(Empty, Double, Double, Double),
            "dbldnleftbxd" => self.bxd(Empty, Double, Empty, Double),
            "dbldnrightbxd" => self.bxd(Empty, Empty, Double, Double),
            "dblvertbxd" => self.bxd(Double, Empty, Empty, Double),
            "dblhorzbxd" => self.bxd(Empty, Double, Double, Empty),
            "dbluphorzbxd" => self.bxd(Double, Double, Double, Empty),
            "dblupleftbxd" => self.bxd(Double, Double, Empty, Empty),
            "dbluprightbxd" => self.bxd(Double, Empty, Double, Empty),
            "dblverthorzbxd" => self.bxd(Double, Double, Double, Double),
            "dblvertleftbxd" => self.bxd(Double, Double, Empty, Double),
            "dblvertrightbxd" => self.bxd(Double, Empty, Double, Double),

            "dndblhorzsngbxd" => self.bxd(Empty, Light, Light, Double),
            "dndblleftsngbxd" => self.bxd(Empty, Light, Empty, Double),
            "dndblrightsngbxd" => self.bxd(Empty, Empty, Light, Double),
            "dnsnghorzdblbxd" => self.bxd(Empty, Double, Double, Light),
            "dnsngleftdblbxd" => self.bxd(Empty, Double, Empty, Light),
            "dnsngrightdblbxd" => self.bxd(Empty, Empty, Double, Light),
            "updblhorzsngbxd" => self.bxd(Double, Light, Light, Empty),
            "updblleftsngbxd" => self.bxd(Double, Light, Empty, Empty),
            "updblrightsngbxd" => self.bxd(Double, Empty, Light, Empty),
            "upsnghorzdblbxd" => self.bxd(Light, Double, Double, Empty),
            "upsngleftdblbxd" => self.bxd(Light, Double, Empty, Empty),
            "upsngrightdblbxd" => self.bxd(Light, Empty, Double, Empty),
            "vertdblhorzsngbxd" => self.bxd(Double, Light, Light, Double),
            "vertdblleftsngbxd" => self.bxd(Double, Light, Empty, Double),
            "vertdblrightsngbxd" => self.bxd(Double, Empty, Light, Double),
            "vertsnghorzdblbxd" => self.bxd(Light, Double, Double, Light),
            "vertsngleftdblbxd" => self.bxd(Light, Double, Empty, Light),
            "vertsngrightdblbxd" => self.bxd(Light, Empty, Double, Light),

            "dnheavyhorzlightbxd" => self.bxd(Empty, Light, Light, Heavy),
            "dnheavyleftlightbxd" => self.bxd(Empty, Light, Empty, Heavy),
            "dnheavyleftuplightbxd" => self.bxd(Light, Light, Empty, Heavy),
            "dnheavyrightlightbxd" => self.bxd(Empty, Empty, Light, Heavy),
            "dnheavyrightuplightbxd" => self.bxd(Light, Empty, Light, Heavy),
            "dnheavyuphorzlightbxd" => self.bxd(Light, Light, Light, Heavy),
            "dnlighthorzheavybxd" => self.bxd(Empty, Heavy, Heavy, Light),
            "dnlightleftheavybxd" => self.bxd(Empty, Heavy, Empty, Light),
            "dnlightrightheavybxd" => self.bxd(Empty, Empty, Heavy, Light),
            "dnlightleftupheavybxd" => self.bxd(Heavy, Heavy, Empty, Light),
            "dnlightrightupheavybxd" => self.bxd(Heavy, Empty, Heavy, Light),
            "dnlightuphorzheavybxd" => self.bxd(Heavy, Heavy, Heavy, Light),
            "heavyleftlightrightbxd" => self.bxd(Empty, Heavy, Light, Empty),
            "heavyuplightdnbxd" => self.bxd(Heavy, Empty, Empty, Light),
            "leftdnheavyrightuplightbxd" => self.bxd(Light, Heavy, Light, Heavy),
            "leftheavyrightdnlightbxd" => self.bxd(Empty, Heavy, Light, Light),
            "leftheavyrightuplightbxd" => self.bxd(Light, Heavy, Light, Empty),
            "leftheavyrightvertlightbxd" => self.bxd(Light, Heavy, Light, Light),
            "leftlightrightdnheavybxd" => self.bxd(Empty, Light, Heavy, Heavy),
            "leftlightrightupheavybxd" => self.bxd(Heavy, Light, Heavy, Empty),
            "leftlightrightvertheavybxd" => self.bxd(Heavy, Light, Heavy, Heavy),
            "leftupheavyrightdnlightbxd" => self.bxd(Heavy, Heavy, Light, Light),
            "lightleftheavyrightbxd" => self.bxd(Empty, Light, Heavy, Empty),
            "lightupheavydnbxd" => self.bxd(Light, Empty, Empty, Heavy),
            "rightdnheavyleftuplightbxd" => self.bxd(Light, Light, Heavy, Heavy),
            "rightheavyleftdnlightbxd" => self.bxd(Empty, Light, Heavy, Light),
            "rightheavyleftuplightbxd" => self.bxd(Light, Light, Heavy, Empty),
            "rightheavyleftvertlightbxd" => self.bxd(Light, Light, Heavy, Light),
            "rightlightleftdnheavybxd" => self.bxd(Empty, Heavy, Light, Heavy),
            "rightlightleftupheavybxd" => self.bxd(Heavy, Heavy, Light, Empty),
            "rightlightleftvertheavybxd" => self.bxd(Heavy, Heavy, Light, Heavy),
            "rightupheavyleftdnlightbxd" => self.bxd(Heavy, Light, Heavy, Light),
            "upheavydnhorzlightbxd" => self.bxd(Heavy, Light, Light, Light),
            "upheavyhorzlightbxd" => self.bxd(Heavy, Light, Light, Empty),
            "upheavyleftdnlightbxd" => self.bxd(Heavy, Light, Empty, Light),
            "upheavyleftlightbxd" => self.bxd(Heavy, Light, Empty, Empty),
            "upheavyrightdnlightbxd" => self.bxd(Heavy, Empty, Light, Light),
            "upheavyrightlightbxd" => self.bxd(Heavy, Empty, Light, Empty),
            "uplightdnhorzheavybxd" => self.bxd(Light, Heavy, Heavy, Heavy),
            "uplighthorzheavybxd" => self.bxd(Light, Heavy, Heavy, Empty),
            "uplightleftdnheavybxd" => self.bxd(Light, Heavy, Empty, Heavy),
            "uplightleftheavybxd" => self.bxd(Light, Heavy, Empty, Empty),
            "uplightrightdnheavybxd" => self.bxd(Light, Empty, Heavy, Heavy),
            "uplightrightheavybxd" => self.bxd(Light, Empty, Heavy, Empty),
            "vertheavyhorzlightbxd" => self.bxd(Heavy, Light, Light, Heavy),
            "vertheavyleftlightbxd" => self.bxd(Heavy, Light, Empty, Heavy),
            "vertheavyrightlightbxd" => self.bxd(Heavy, Empty, Light, Heavy),
            "vertlighthorzheavybxd" => self.bxd(Light, Heavy, Heavy, Light),
            "vertlightleftheavybxd" => self.bxd(Light, Heavy, Empty, Light),
            "vertlightrightheavybxd" => self.bxd(Light, Empty, Heavy, Light),

            "lightshade" => self.shade(40.0, 50.0, 70.0),
            "mediumshade" => self.shade(50.0, 80.0, 90.0),
            // Maybe TODO: clip the dark one to the glyph box
            "darkshade" => self.shade(110.0, 120.0, 130.0),

            "heavydbldashhorzbxd" => self.dashed_hline(2, self.heavy()),
            "heavytrpldashhorzbxd" => self.dashed_hline(3, self.heavy()),
            "heavyquaddashhorzbxd" => self.dashed_hline(4, self.heavy()),
            "lightdbldashhorzbxd" => self.dashed_hline(2, self.light()),
            "lighttrpldashhorzbxd" => self.dashed_hline(3, self.light()),
            "lightquaddashhorzbxd" => self.dashed_hline(4, self.light()),
            "heavydbldashvertbxd" => self.dashed_vline(2, self.heavy()),
            "heavytrpldashvertbxd" => self.dashed_vline(3, self.heavy()),
            "heavyquaddashvertbxd" => self.dashed_vline(4, self.heavy()),
            "lightdbldashvertbxd" => self.dashed_vline(2, self.light()),
            "lighttrpldashvertbxd" => self.dashed_vline(3, self.light()),
            "lightquaddashvertbxd" => self.dashed_vline(4, self.light()),

            "lightdiaguprightdnleftbxd" => {
                let start = Point::new(0.0, -300.0);
                let end = Point::new(self.wdth as f64 * 5.0, 900.0);
                return Some(vec![self.diagonal(start, end, self.light())]);
            }
            "lightdiagupleftdnrightbxd" => {
                let start = Point::new(0.0, 900.0);
                let end = Point::new(self.wdth as f64 * 5.0, -300.0);
                return Some(vec![self.diagonal(start, end, self.light())]);
            }
            "lightdiagcrossbxd" => {
                // Note: it would probably be more efficient to use components
                let start = Point::new(0.0, -300.0);
                let end = Point::new(self.wdth as f64 * 5.0, 900.0);
                let path1 = self.diagonal(start, end, self.light());
                let start = Point::new(0.0, 900.0);
                let end = Point::new(self.wdth as f64 * 5.0, -300.0);
                let path2 = self.diagonal(start, end, self.light());
                return Some(vec![path1, path2]);
            }
            "lightarcdnleftbxd" => return Some(self.arc(false, false)),
            "lightarcdnrightbxd" => return Some(self.arc(true, false)),
            "lightarcupleftbxd" => return Some(self.arc(false, true)),
            "lightarcuprightbxd" => return Some(self.arc(true, true)),
            _ => return None,
        }
        Some(self.region.to_paths())
    }
}

/// Create symbols, mostly box-drawing.
///
/// Note that it should be practical to adapt this into a fairly general tool.
pub fn inco_syms(font: &mut Font) {
    let layers = get_layer_map(font);

    for glyph in &mut font.glyphs {
        for layer in &mut glyph.layers {
            if let Some((wght, wdth)) = layers.get_params(&layer.layer_id) {
                let mut box_draw = BoxDraw::new(wght, wdth);
                if let Some(paths) = box_draw.draw(&glyph.glyphname) {
                    layer.paths = Some(paths);
                } else {
                    break;
                }
            }
        }
    }
}
