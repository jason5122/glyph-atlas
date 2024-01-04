use std::cmp;

use crop::Rope;
use unicode_segmentation::GraphemeCursor;

use crate::display::content::{RenderableCell, RenderableCursor};
use crate::display::Rgb;

#[derive(Default)]
pub struct Buffer {
    data: Rope,
    cursor: usize,
    cursor_offset: usize,
}

impl Buffer {
    pub fn get_renderables(&self) -> (Vec<RenderableCell>, RenderableCursor) {
        let mut cells = Vec::new();

        let s = "Hello world!";
        for (column, character) in s.chars().enumerate() {
            let cell = RenderableCell {
                character,
                line: 10,
                column,
                bg_alpha: 1.0,
                fg: Rgb::new(0x33, 0x33, 0x33),
                bg: Rgb::new(0xfc, 0xfd, 0xfd),
                underline: Rgb::new(0x33, 0x33, 0x33),
            };
            cells.push(cell);
        }

        let cursor_point = Point::new(10, 3);
        let cursor = RenderableCursor { point: cursor_point, color: Rgb::new(0x5f, 0xb4, 0xb4) };

        (cells, cursor)
    }
}

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
