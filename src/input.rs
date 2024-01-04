use winit::event::{ElementState, KeyEvent, Modifiers};
use winit::keyboard::{Key, ModifiersState};
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
use winit::window::WindowId;

use crate::display::window::Window;
use crate::display::{Display, SizeInfo};
use crate::editor::buffer::{Movement, VerticalMovement};
use crate::editor::Editor;

/// Processes input from winit.
///
/// An escape sequence may be emitted in case specific keys or key combinations
/// are activated.
pub struct Processor<A: ActionContext> {
    pub ctx: A,
}

pub trait ActionContext {
    fn mark_dirty(&mut self) {}
    fn size_info(&self) -> SizeInfo;
    fn modifiers(&mut self) -> &mut Modifiers;
    fn window(&mut self) -> &mut Window;
    fn display(&mut self) -> &mut Display;
    fn editor(&self) -> &Editor;
    fn editor_mut(&mut self) -> &mut Editor;
    fn create_new_window(&mut self) {}
    fn close_window(&mut self, _window_id: WindowId) {}
    fn redraw_editor(&mut self, _window_id: WindowId) {}
}

impl<A: ActionContext> Processor<A> {
    pub fn new(ctx: A) -> Self {
        Self { ctx }
    }

    /// Modifier state change.
    pub fn modifiers_input(&mut self, modifiers: Modifiers) {
        *self.ctx.modifiers() = modifiers;
    }

    /// Process key input.
    pub fn key_input(&mut self, key: KeyEvent) {
        // IME input will be applied on commit and shouldn't trigger key bindings.
        if key.state == ElementState::Released {
            return;
        }

        let text = key.text_with_all_modifiers().unwrap_or_default();

        // Key bindings suppress the character input.
        if self.process_key_bindings(&key) {
            return;
        }

        if text.is_empty() {
            return;
        }

        let mods = self.ctx.modifiers().state();
        let editor = &mut self.ctx.editor_mut();
        match (mods, key.key_without_modifiers().as_ref()) {
            (ModifiersState::CONTROL, Key::Character("a")) => {
                editor.buffer_mut().move_cursor(Movement::StartOfLine)
            },
            (ModifiersState::CONTROL, Key::Character("e")) => {
                editor.buffer_mut().move_cursor(Movement::EndOfLine)
            },
            (ModifiersState::SUPER, Key::ArrowLeft) => {
                editor.buffer_mut().move_cursor(Movement::StartOfLine)
            },
            (ModifiersState::SUPER, Key::ArrowRight) => {
                editor.buffer_mut().move_cursor(Movement::EndOfLine)
            },
            (ModifiersState::SUPER, Key::Backspace) => editor.buffer_mut().delete_line_backwards(),
            (_, Key::Backspace) => editor.buffer_mut().delete_char_backwards(),
            (_, Key::ArrowLeft) => editor.buffer_mut().move_cursor(Movement::BackwardChar(1)),
            (_, Key::ArrowRight) => editor.buffer_mut().move_cursor(Movement::ForwardChar(1)),
            (_, Key::ArrowUp) => editor.buffer_mut().move_cursor_vertical(VerticalMovement::UpLine),
            (_, Key::ArrowDown) => {
                editor.buffer_mut().move_cursor_vertical(VerticalMovement::DownLine)
            },
            (_, Key::Enter) => editor.buffer_mut().insert("\n"),
            (_, _) => editor.buffer_mut().insert(text),
        };

        self.ctx.mark_dirty();

        let window_id = self.ctx.window().id();
        self.ctx.redraw_editor(window_id);
    }

    fn process_key_bindings(&mut self, key: &KeyEvent) -> bool {
        let mods = self.ctx.modifiers().state();

        // Don't suppress char if no bindings were triggered.
        let mut suppress_chars = true;

        match (mods, key.key_without_modifiers().as_ref()) {
            (ModifiersState::SUPER, Key::Character("n")) => self.ctx.create_new_window(),
            (ModifiersState::SUPER, Key::Character("w")) => {
                let window_id = self.ctx.window().id();
                self.ctx.close_window(window_id);
            },
            (ModifiersState::SUPER, Key::Character("m")) => self.ctx.window().set_minimized(true),
            (ModifiersState::SUPER, Key::Character("j")) => self.ctx.window().select_previous_tab(),
            (ModifiersState::SUPER, Key::Character("k")) => self.ctx.window().select_next_tab(),
            (ModifiersState::SUPER, Key::Character("1")) => self.ctx.window().select_tab(1),
            (ModifiersState::SUPER, Key::Character("2")) => self.ctx.window().select_tab(2),
            (ModifiersState::SUPER, Key::Character("3")) => self.ctx.window().select_tab(3),
            (ModifiersState::SUPER, Key::Character("4")) => self.ctx.window().select_tab(4),
            (ModifiersState::SUPER, Key::Character("5")) => self.ctx.window().select_tab(5),
            (ModifiersState::SUPER, Key::Character("6")) => self.ctx.window().select_tab(6),
            (ModifiersState::SUPER, Key::Character("7")) => self.ctx.window().select_tab(7),
            (ModifiersState::SUPER, Key::Character("8")) => self.ctx.window().select_tab(8),
            (ModifiersState::SUPER, Key::Character("9")) => self.ctx.window().select_tab(9),
            (_, _) => suppress_chars = false,
        };

        suppress_chars
    }
}
