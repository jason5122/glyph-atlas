use crate::display::{Rgb, SizeInfo};
use crate::renderer::rects::RenderRect;

/// Cell ready for rendering.
#[derive(Clone, Debug)]
pub struct RenderableCell {
    pub character: char,
    pub line: usize,
    pub column: usize,
    pub fg: Rgb,
    pub bg: Rgb,
    pub bg_alpha: f32,
    pub underline: Rgb,
}

/// Cursor storing all information relevant for rendering.
#[derive(Eq, PartialEq, Copy, Clone)]
pub struct RenderableCursor {
    pub point: Point,
    pub color: Rgb,
}

impl RenderableCursor {
    pub fn rects(self, size_info: &SizeInfo, thickness: f32) -> RenderRect {
        let x = self.point.column as f32 * size_info.cell_width + size_info.padding_x;
        let y = self.point.line as f32 * size_info.cell_height + size_info.padding_y;

        let width = size_info.cell_width;
        let height = size_info.cell_height;

        let thickness = (thickness * width).round().max(1.);

        RenderRect::new(x, y, thickness, height, self.color, 1.).into()
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
