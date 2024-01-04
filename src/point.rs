use std::cmp;

use crop::Rope;
use unicode_segmentation::GraphemeCursor;

use crate::display::content::{RenderableCell, RenderableCursor};
use crate::display::Rgb;

#[derive(Default, Eq, PartialEq, Copy, Clone)]
pub struct Point {
    pub line: usize,
    pub column: usize,
}

impl Point {
    pub fn new(line: usize, column: usize) -> Point {
        Point { line, column }
    }
}
