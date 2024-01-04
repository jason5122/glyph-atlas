use crate::editor::buffer::Buffer;

pub mod buffer;

#[derive(Default)]
pub struct Editor {
    buffer: Buffer,
}

impl Editor {
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }
}
