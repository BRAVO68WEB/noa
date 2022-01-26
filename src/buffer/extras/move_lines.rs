use crate::{
    buffer::Buffer,
    cursor::{Cursor, Range},
};

impl Buffer {
    pub fn move_lines_up(&mut self) {
        self.cursors.foreach(|c, past_cursors| {
            if c.front().y == 0 {
                return;
            }

            let s = c.selection();

            c.select_overlapped_lines();
            let mut text = self.buf.substr(c.selection());
            if !text.ends_with('\n') {
                text.push('\n');
            }
            let prev_line = self
                .buf
                .substr(Range::new(c.front().y - 1, 0, c.front().y, 0));
            dbg!(&prev_line);
            dbg!(&text);
            dbg!(&c.selection());
            text.push_str(&prev_line);

            *c = Cursor::new_selection(s.front().y - 1, s.start.x, s.back().y, s.end.x);
            self.buf.edit_at_cursor(c, past_cursors, &text);
            *c = Cursor::new_selection(s.start.y - 1, s.start.x, s.end.y - 1, s.end.x);
        });
    }

    pub fn move_lines_down(&mut self) {
        self.cursors.foreach(|c, past_cursors| {
            let s = c.selection();

            c.select_overlapped_lines();
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn move_a_line_up() {
        let mut b = Buffer::from_text("");
        b.set_cursors(&[Cursor::new(0, 0)]);
        b.move_lines_up();
        assert_eq!(b.text(), "");
        assert_eq!(b.cursors(), &[Cursor::new(0, 0)]);

        //
        // abcd
        let mut b = Buffer::from_text("\nabcd");
        b.set_cursors(&[Cursor::new(1, 2)]);
        b.move_lines_up();
        assert_eq!(b.text(), "abcd\n");
        assert_eq!(b.cursors(), &[Cursor::new(0, 2)]);

        // abcd
        // xyz
        let mut b = Buffer::from_text("abcd\nxyz");
        b.set_cursors(&[Cursor::new(1, 2)]);
        b.move_lines_up();
        assert_eq!(b.text(), "abcd");
        assert_eq!(b.cursors(), &[Cursor::new(1, 2)]);
    }

    #[test]
    fn move_multiple_lines_up() {
        // ABCD
        // EFGH
    }
}
