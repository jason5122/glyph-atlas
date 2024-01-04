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
    pub fn insert(&mut self, string: &str) {
        self.data.insert(self.cursor, string);
        self.move_cursor(Movement::ForwardChar(1))
    }

    pub fn move_cursor(&mut self, movement: Movement) {
        let pos = self.eval_movement(movement);
        self.cursor = pos;

        self.update_offset();
    }

    pub fn move_cursor_vertical(&mut self, movement: VerticalMovement) {
        let pos = self.eval_vertical_movement(movement);
        self.cursor = pos;
    }

    pub fn delete_char_backwards(&mut self) {
        let pos = self.eval_movement(Movement::BackwardChar(1));
        self.data.replace(pos..self.cursor, "");
        self.cursor = pos;

        self.update_offset();
    }

    pub fn delete_line_backwards(&mut self) {
        let pos = self.eval_movement(Movement::StartOfLine);
        self.data.replace(pos..self.cursor, "");
        self.cursor = pos;

        self.update_offset();
    }

    fn update_offset(&mut self) {
        let line = self.data.line_of_byte(self.cursor);
        let line_start = self.data.byte_of_line(line);
        self.cursor_offset = self.cursor - line_start;
    }

    fn eval_movement(&self, movement: Movement) -> usize {
        let data_str = self.data.to_string();

        match movement {
            Movement::BackwardChar(rep) => {
                let mut position = self.cursor;
                for _ in 0..rep {
                    let mut cursor = GraphemeCursor::new(position, data_str.len(), false);
                    if let Ok(Some(pos)) = cursor.prev_boundary(&data_str, 0) {
                        position = pos;
                    } else {
                        break;
                    }
                }
                position
            },
            Movement::ForwardChar(rep) => {
                let mut position = self.cursor;
                for _ in 0..rep {
                    let mut cursor = GraphemeCursor::new(position, data_str.len(), false);
                    if let Ok(Some(pos)) = cursor.next_boundary(&data_str, 0) {
                        position = pos;
                    } else {
                        break;
                    }
                }
                position
            },
            Movement::StartOfLine => {
                let line = self.data.line_of_byte(self.cursor);
                let line_start = self.data.byte_of_line(line);

                line_start
            },
            Movement::EndOfLine => {
                let line = self.data.line_of_byte(self.cursor);

                if line == self.data.line_len() {
                    return self.data.byte_len();
                }

                let line_len = self.data.line(line).byte_len();
                let line_end = self.data.byte_of_line(line) + line_len;

                let mut cursor = GraphemeCursor::new(line_end.saturating_sub(1), line_end, false);
                if let Ok(Some(pos)) = cursor.next_boundary(&data_str, 0) {
                    pos
                } else {
                    self.cursor
                }
            },
        }
    }

    fn eval_vertical_movement(&self, movement: VerticalMovement) -> usize {
        match movement {
            VerticalMovement::UpLine => {
                let line = self.data.line_of_byte(self.cursor);
                let prev_line = line.saturating_sub(1);

                if line == prev_line {
                    return 0;
                }

                let prev_line_start = self.data.byte_of_line(prev_line);

                let prev_line_len = self.data.line(prev_line).byte_len();
                let position = prev_line_start + cmp::min(self.cursor_offset, prev_line_len);
                position
            },
            VerticalMovement::DownLine => {
                let line = self.data.line_of_byte(self.cursor);
                let next_line = line.saturating_add(1);

                if next_line >= self.data.line_len() {
                    return self.data.byte_len();
                }

                let next_line_start = self.data.byte_of_line(next_line);

                let next_line_len = self.data.line(next_line).byte_len();
                let position = next_line_start + cmp::min(self.cursor_offset, next_line_len);
                position
            },
        }
    }

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

#[derive(Debug, Clone, Copy)]
pub enum Movement {
    BackwardChar(usize),
    ForwardChar(usize),
    StartOfLine,
    EndOfLine,
}

pub enum VerticalMovement {
    UpLine,
    DownLine,
}
