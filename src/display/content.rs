use crate::display::Rgb;

/// Cell ready for rendering.
#[derive(Clone, Debug)]
pub struct RenderableCell {
    pub character: char,
    pub line: usize,
    pub column: usize,
    pub fg: Rgb,
    pub bg: Rgb,
    pub bg_alpha: f32,
    pub font_key: usize,
}
