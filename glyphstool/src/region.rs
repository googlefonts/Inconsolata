//! A data structure representing the union of rectangles.

use std::collections::BTreeSet;

use kurbo::{Point, Rect};

use crate::font::{NodeType, Path};

#[derive(Default, Debug)]
pub struct Region {
    slices: Vec<Slice>,
}

#[derive(Clone, PartialEq, Debug)]
struct Slice {
    y0: f64,
    y1: f64,
    intervals: Vec<Interval>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct Interval {
    x0: f64,
    x1: f64,
}

impl From<(f64, f64)> for Interval {
    fn from(xs: (f64, f64)) -> Interval {
        Interval { x0: xs.0, x1: xs.1 }
    }
}

impl Slice {
    fn from_rect(rect: Rect) -> Slice {
        Slice {
            y0: rect.y0,
            y1: rect.y1,
            intervals: vec![(rect.x0, rect.x1).into()],
        }
    }

    // Trims the slice to the height of the rect, unions the rect into the intervals.
    fn add_rect(&self, rect: impl Into<Rect>) -> Slice {
        let rect = rect.into();
        let mut iv = Interval::from((rect.x0, rect.x1));
        let mut intervals = Vec::new();
        let mut i = 0;
        while i < self.intervals.len() && self.intervals[i].x1 < iv.x0 {
            intervals.push(self.intervals[i]);
            i += 1;
        }
        while i < self.intervals.len() && self.intervals[i].x0 <= iv.x1 {
            iv.x0 = iv.x0.min(self.intervals[i].x0);
            iv.x1 = iv.x1.max(self.intervals[i].x1);
            i += 1;
        }
        intervals.push(iv);
        intervals.extend_from_slice(&self.intervals[i..]);

        Slice {
            y0: rect.y0,
            y1: rect.y1,
            intervals,
        }
    }

    fn slice(&self, y0: f64, y1: f64) -> Slice {
        Slice {
            y0,
            y1,
            intervals: self.intervals.clone(),
        }
    }
}

impl Region {
    fn push(&mut self, slice: Slice) {
        if let Some(last) = self.slices.last_mut() {
            if last.y1 == slice.y0 && last.intervals == slice.intervals {
                last.y1 = slice.y1;
                return;
            }
        }
        self.slices.push(slice);
    }

    pub fn add(&self, rect: impl Into<Rect>) -> Region {
        let rect = rect.into();
        let mut result = Region::default();
        let mut i = 0;
        while i < self.slices.len() && self.slices[i].y1 <= rect.y0 {
            result.push(self.slices[i].clone());
            i += 1;
        }
        if i < self.slices.len() && self.slices[i].y0 < rect.y0 {
            result.push(self.slices[i].slice(self.slices[i].y0, rect.y0));
        }
        let mut y = rect.y0;
        while i < self.slices.len() && self.slices[i].y0 < rect.y1 {
            if self.slices[i].y0 > y {
                let trim_rect = Rect::new(rect.x0, y, rect.x1, self.slices[i].y0);
                result.push(Slice::from_rect(trim_rect));
                y = self.slices[i].y0;
            }
            let y1 = self.slices[i].y1.min(rect.y1);
            let trim_rect = Rect::new(rect.x0, y, rect.x1, y1);
            result.push(self.slices[i].add_rect(trim_rect));
            y = y1;
            if y < self.slices[i].y1 {
                result.push(self.slices[i].slice(y, self.slices[i].y1));
            }
            i += 1;
        }
        if y < rect.y1 {
            let trim_rect = Rect::new(rect.x0, y, rect.x1, rect.y1);
            result.push(Slice::from_rect(trim_rect));
        }
        while i < self.slices.len() {
            result.push(self.slices[i].clone());
            i += 1;
        }
        result
    }

    /*
    // This is the dumb version, for reference
    pub fn to_paths(&self) -> Vec<Path> {
        // TODO: generate more optimized path
        let mut result = Vec::new();
        for slice in &self.slices {
            for iv in &slice.intervals {
                let mut path = Path::new(true);
                path.add((iv.x0, slice.y0), NodeType::Line);
                path.add((iv.x1, slice.y0), NodeType::Line);
                path.add((iv.x1, slice.y1), NodeType::Line);
                path.add((iv.x0, slice.y1), NodeType::Line);
                result.push(path);
            }
        }
        result
    }
    */

    pub fn to_paths(&self) -> Vec<Path> {
        let mut tracer = PathTracer::default();
        for i in 0..self.slices.len() {
            let slice = &self.slices[i];
            let (y0, y1) = (slice.y0, slice.y1);
            if i == 0 || self.slices[i - 1].y1 != y0 {
                tracer.process_line(&[], &slice.intervals, y0);
            } else {
                tracer.process_line(&self.slices[i - 1].intervals, &slice.intervals, y0);
            }
            if i == self.slices.len() - 1 || self.slices[i + 1].y0 != y1 {
                tracer.process_line(&slice.intervals, &[], y1);
            }
        }
        tracer.to_paths()
    }
}

type VertexIx = usize;
type EdgeIx = usize;

#[derive(Default)]
struct PathTracer {
    vertices: Vec<Vertex>,
    edges: Vec<(VertexIx, VertexIx)>,
    prev_verts: Vec<VertexIx>,
    next_verts: Vec<VertexIx>,

    pending_edges: BTreeSet<EdgeIx>,
}

struct Vertex {
    pt: Point,
    pred: EdgeIx,
    succ: EdgeIx,
}

const NIL: usize = !0;

impl PathTracer {
    /// Create a new vertex (no edges) and return its index.
    fn new_vertex(&mut self, x: f64, y: f64) -> VertexIx {
        let v = self.vertices.len();
        self.vertices.push(Vertex {
            pt: Point::new(x, y),
            pred: NIL,
            succ: NIL,
        });
        v
    }

    /// Draw an edge from `v0` to `v1`.
    fn edge(&mut self, v0: VertexIx, v1: VertexIx) {
        let e = self.edges.len();
        self.edges.push((v0, v1));
        self.vertices[v0].succ = e;
        self.vertices[v1].pred = e;
    }

    fn process_line(&mut self, prev: &[Interval], next: &[Interval], y: f64) {
        fn get(ivs: &[Interval], i: usize) -> f64 {
            let iv = ivs[i / 2];
            if (i & 1) == 0 {
                iv.x0
            } else {
                iv.x1
            }
        }
        std::mem::swap(&mut self.prev_verts, &mut self.next_verts);
        self.next_verts.clear();
        // These are indices to 2x interval + 1 if right edge
        let imax = prev.len() * 2;
        let jmax = next.len() * 2;
        let mut i = 0;
        let mut j = 0;
        let mut last_v: Option<usize> = None;
        while i < imax || j < jmax {
            if j >= jmax || (i < imax && get(prev, i) <= get(next, j)) {
                // Process a point from prev.
                let x = get(prev, i);
                if let Some(last_v) = last_v.take() {
                    let last_x = self.vertices[last_v].pt.x;
                    if x > last_x {
                        let v = self.new_vertex(x, y);
                        if (i & 1) == 0 {
                            self.edge(last_v, v);
                            self.edge(v, self.prev_verts[i]);
                        } else {
                            self.edge(self.prev_verts[i], v);
                            self.edge(v, last_v);
                        }
                    } else {
                        if (i & 1) == 0 {
                            self.edge(last_v, self.prev_verts[i]);
                        } else {
                            self.edge(self.prev_verts[i], last_v);
                        }
                    }
                } else {
                    let v = self.new_vertex(x, y);
                    if (i & 1) == 0 {
                        self.edge(v, self.prev_verts[i]);
                    } else {
                        self.edge(self.prev_verts[i], v);
                    }
                    last_v = Some(v);
                }
                i += 1;
            } else {
                // Process a point from next.
                let x = get(next, j);
                if let Some(last_v) = last_v.take() {
                    let last_x = self.vertices[last_v].pt.x;
                    if x > last_x {
                        let v = self.new_vertex(x, y);
                        self.next_verts.push(v);
                        if (i & 1) == 0 {
                            self.edge(last_v, v);
                        } else {
                            self.edge(v, last_v);
                        }
                    } else {
                        self.next_verts.push(last_v);
                    }
                } else {
                    let v = self.new_vertex(x, y);
                    self.next_verts.push(v);
                    last_v = Some(v);
                }
                j += 1;
            }
        }
    }

    fn trace_path(&mut self) -> Option<Path> {
        if let Some(e) = self.pending_edges.iter().next() {
            let mut e = *e;
            let mut path = Path::new(true);
            let mut last_pt: Option<Point> = None;
            while self.pending_edges.remove(&e) {
                let edge = self.edges[e];
                let pt = self.vertices[edge.0].pt;
                if let Some(last_pt) = last_pt {
                    if last_pt.x != pt.x {
                        path.add(last_pt, NodeType::Line);
                        path.add(pt, NodeType::Line);
                    }
                }
                last_pt = Some(pt);
                e = self.vertices[edge.1].succ;
            }
            // Note: should probably rotate by two for clockwise, to make
            // it match what glyphs does for "correct path direction".
            path.rotate_left(1);
            Some(path)
        } else {
            None
        }
    }

    fn to_paths(&mut self) -> Vec<Path> {
        self.pending_edges = (0..self.edges.len()).collect();
        let mut result = Vec::new();
        while let Some(path) = self.trace_path() {
            result.push(path);
        }
        result
    }
}
