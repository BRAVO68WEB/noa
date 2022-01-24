use noa_buffer::display_width::DisplayWidth;
use noa_compositor::{
    canvas::{CanvasViewMut, Decoration, Style},
    line_edit::LineEdit,
    surface::{HandledEvent, KeyEvent, Layout, MouseEvent, RectSize, Surface},
    terminal::{KeyCode, KeyModifiers},
};
use tokio::{sync::oneshot, task};

use crate::{
    clipboard::{ClipboardData, SystemClipboardData},
    editor::Editor,
    notification::Notification,
};

use super::helpers::truncate_to_width;

pub struct BottomLineView {
    search_query: LineEdit,
}

impl BottomLineView {
    pub fn new() -> BottomLineView {
        BottomLineView {
            search_query: LineEdit::new(),
        }
    }
}

impl Surface for BottomLineView {
    type Context = Editor;

    fn name(&self) -> &str {
        "bottom_line"
    }

    fn is_visible(&self, _editor: &mut Editor) -> bool {
        true
    }

    fn layout(&self, _editor: &mut Editor, screen_size: RectSize) -> (Layout, RectSize) {
        (
            Layout::Fixed {
                y: screen_size.height.saturating_sub(2),
                x: 0,
            },
            RectSize {
                height: 2,
                width: screen_size.width,
            },
        )
    }

    fn cursor_position(&self, _editor: &mut Editor) -> Option<(usize, usize)> {
        None
    }

    fn render(&mut self, editor: &mut Editor, canvas: &mut CanvasViewMut<'_>) {
        canvas.clear();

        let doc = editor.documents.current();
        let buffer = doc.buffer();
        let cursor_pos = buffer.main_cursor().selection().start;
        let cursor_pos_str = format!("{}, {}", cursor_pos.y + 1, cursor_pos.x);
        let cursor_pos_width = cursor_pos_str.display_width();
        let filename_max_width = canvas.width() - cursor_pos_width - 2;
        let search_query = self.search_query.text();
        let notification_max_width = canvas.width() - search_query.display_width() - 2;
        let noti = editor
            .notifications
            .last_notification_as_str()
            .unwrap_or_else(|| "".to_string());
        let noti = truncate_to_width(&noti, notification_max_width);

        // File name.
        canvas.write_str(0, 1, truncate_to_width(doc.name(), filename_max_width));
        // Cursor position.
        canvas.write_str(0, canvas.width() - 1 - cursor_pos_width, &cursor_pos_str);
        // The first line.
        canvas.set_decoration(0, 0, canvas.width(), Decoration::inverted());
        // Search query.
        canvas.write_str(0, 1, &search_query);
        // Notification.
        canvas.write_str(0, canvas.width() - 1 - noti.display_width(), noti);
    }

    fn handle_key_event(&mut self, editor: &mut Editor, key: KeyEvent) -> HandledEvent {
        const NONE: KeyModifiers = KeyModifiers::NONE;
        const CTRL: KeyModifiers = KeyModifiers::CONTROL;
        const ALT: KeyModifiers = KeyModifiers::ALT;
        const SHIFT: KeyModifiers = KeyModifiers::SHIFT;

        let mut _doc = editor.documents.current();

        // match (key.code, key.modifiers) {
        // _ => HandledEvent::Ignored,
        // }

        HandledEvent::Ignored
    }

    fn handle_key_batch_event(&mut self, editor: &mut Editor, s: &str) -> HandledEvent {
        HandledEvent::Ignored
    }

    fn handle_mouse_event(&mut self, editor: &mut Editor, _ev: MouseEvent) -> HandledEvent {
        HandledEvent::Ignored
    }
}