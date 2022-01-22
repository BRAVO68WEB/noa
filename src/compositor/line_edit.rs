use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use noa_buffer::buffer::Buffer;

pub struct LineEdit {
    buffer: Buffer,
    scroll: usize,
}

impl LineEdit {
    pub fn new() -> LineEdit {
        LineEdit {
            buffer: Buffer::new(),
            scroll: 0,
        }
    }

    pub fn from_text(text: &str) -> LineEdit {
        let mut le = LineEdit::new();
        le.insert(text);
        le
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.buffer.line_len(0)
    }

    pub fn text(&self) -> String {
        self.buffer.text()
    }

    pub fn insert(&mut self, s: &str) {
        self.buffer.insert(&s.replace('\n', " "));
    }

    pub fn insert_char(&mut self, c: char) {
        if c == '\n' {
            warn!("newline is not supported in LineEdit");
            return;
        }

        self.buffer.insert_char(c);
    }

    pub fn backspace(&mut self) {
        self.buffer.backspace();
    }

    pub fn delete(&mut self) {
        self.buffer.delete();
    }

    pub fn relocate_scroll(&mut self, width: usize) {
        let x = self.buffer.cursors()[0].selection().start.x;

        // Scroll Right.
        if x > self.scroll + width {
            self.scroll = x - width;
        }

        // Scroll Left.
        if x < self.scroll {
            self.scroll = x;
        }
    }

    pub fn move_left(&mut self) {
        todo!()
    }

    pub fn move_right(&mut self) {
        todo!()
    }

    pub fn move_to_end_of_line(&mut self) {
        todo!()
    }

    pub fn move_to_beginning_of_line(&mut self) {
        todo!()
    }

    pub fn move_to_next_word(&mut self) {
        todo!()
    }

    pub fn move_to_prev_word(&mut self) {
        todo!()
    }

    pub fn consume_key_event(&mut self, key: KeyEvent) -> bool {
        const NONE: KeyModifiers = KeyModifiers::NONE;
        const CTRL: KeyModifiers = KeyModifiers::CONTROL;
        const ALT: KeyModifiers = KeyModifiers::ALT;
        const SHIFT: KeyModifiers = KeyModifiers::SHIFT;

        match (key.modifiers, key.code) {
            (NONE, KeyCode::Char(ch)) => {
                self.insert_char(ch);
            }
            (SHIFT, KeyCode::Char(ch)) => {
                self.insert_char(ch);
            }
            (NONE, KeyCode::Backspace) => {
                self.backspace();
            }
            (NONE, KeyCode::Delete) | (CTRL, KeyCode::Char('d')) => {
                self.delete();
            }
            (NONE, KeyCode::Left) => {
                self.move_left();
            }
            (NONE, KeyCode::Right) => {
                self.move_right();
            }
            (CTRL, KeyCode::Char('a')) => {
                self.move_to_beginning_of_line();
            }
            (CTRL, KeyCode::Char('e')) => {
                self.move_to_end_of_line();
            }
            (ALT, KeyCode::Char('f')) => {
                self.move_to_next_word();
            }
            (ALT, KeyCode::Char('b')) => {
                self.move_to_prev_word();
            }
            _ => {
                return false;
            }
        }

        true
    }
}

impl Default for LineEdit {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relocate_scroll() {
        let mut le = LineEdit::new();
        le.insert("abcde");
        le.relocate_scroll(5);
        assert_eq!(le.scroll, 0);

        le.relocate_scroll(4);
        assert_eq!(le.scroll, 1);

        le.insert_char('f');
        le.relocate_scroll(4);
        assert_eq!(le.scroll, 2);

        le.move_to_beginning_of_line();
        le.relocate_scroll(4);
        assert_eq!(le.scroll, 0);
    }
}