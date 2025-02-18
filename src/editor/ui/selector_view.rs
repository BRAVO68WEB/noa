use std::cmp::min;

use noa_buffer::{cursor::Position, display_width::DisplayWidth};

use noa_compositor::{
    canvas::{CanvasViewMut, Color, Style},
    line_edit::LineEdit,
    surface::{HandledEvent, Layout, RectSize, Surface},
    terminal::{KeyCode, KeyEvent, KeyModifiers},
    Compositor,
};

use crate::{config::theme_for, editor::Editor};

use super::helpers::truncate_to_width;

const HEIGHT_MAX: usize = 16;

pub type SelectedCallback = dyn FnOnce(&mut Editor, &mut Compositor<Editor>) + Send;
pub type ChangedCallback = dyn FnMut(&mut Editor, &str) + Send;

pub enum SelectorContent {
    Normal {
        label: String,
        sub_label: Option<String>,
    },
    SearchMatch {
        path: String,
        pos: Position,
        line_text: String,
        byte_range: std::ops::Range<usize>,
    },
}

pub struct SelectorItem {
    pub content: SelectorContent,
    pub selected: Box<SelectedCallback>,
}

pub struct SelectorView {
    opened_by: &'static str,
    active: bool,
    cursor_pos: (usize, usize),
    items: Vec<SelectorItem>,
    selected_index: usize,
    scroll: usize,
    items_height: usize,
    input: Option<LineEdit>,
    changed_callback: Option<Box<ChangedCallback>>,
}

impl SelectorView {
    pub fn new() -> SelectorView {
        SelectorView {
            opened_by: "",
            active: false,
            cursor_pos: (0, 0),
            items: Vec::new(),
            selected_index: 0,
            scroll: 0,
            items_height: 0,
            input: None,
            changed_callback: None,
        }
    }

    pub fn opened_by(&self) -> &'static str {
        self.opened_by
    }

    pub fn open(
        &mut self,
        opened_by: &'static str,
        eanble_input: bool,
        changed_callback: Option<Box<ChangedCallback>>,
    ) {
        self.opened_by = opened_by;
        self.active = true;
        self.selected_index = 0;
        self.scroll = 0;
        self.items = Vec::new();
        self.changed_callback = changed_callback;

        if eanble_input {
            self.input = Some(LineEdit::new());
        } else {
            self.input = None;
        }
    }

    pub fn close(&mut self) {
        self.active = false;
    }

    pub fn set_items(&mut self, items: Vec<SelectorItem>) {
        self.items = items;
        self.selected_index = min(self.selected_index, self.items.len().saturating_sub(1));
        self.adjust_scroll();
    }

    pub fn select_item(
        &mut self,
        editor: &mut Editor,
        compositor: &mut Compositor<Editor>,
        index: usize,
    ) {
        if index >= self.items.len() {
            notify_warn!("out of bound selection");
            return;
        }

        let item = self.items.remove(index);
        (item.selected)(editor, compositor);
        self.close();
    }

    pub fn adjust_scroll(&mut self) {
        while self.scroll + self.items_height <= self.selected_index {
            self.scroll += 1;
        }

        while self.scroll > self.selected_index {
            self.scroll = self.scroll.saturating_sub(1);
        }
    }
}

impl Surface for SelectorView {
    type Context = Editor;

    fn name(&self) -> &str {
        "selector"
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn is_active(&self, _editor: &mut Editor) -> bool {
        self.active
    }

    fn layout(&mut self, _editor: &mut Editor, screen_size: RectSize) -> (Layout, RectSize) {
        let height = min(
            self.items.len() + if self.input.is_some() { 1 } else { 0 },
            min(HEIGHT_MAX, screen_size.height),
        );

        self.cursor_pos = match self.input.as_ref() {
            Some(input) => (height.saturating_sub(1), 1 + input.cursor_position()),
            None => (0, 0),
        };

        (
            Layout::Fixed {
                y: screen_size.height.saturating_sub(height + 2),
                x: 0,
            },
            RectSize {
                height,
                width: screen_size.width,
            },
        )
    }

    fn cursor_position(&self, _editor: &mut Editor) -> Option<(usize, usize)> {
        if self.active {
            Some(self.cursor_pos)
        } else {
            None
        }
    }

    fn render(&mut self, _editor: &mut Editor, canvas: &mut CanvasViewMut<'_>) {
        canvas.clear();

        self.items_height =
            canvas
                .height()
                .saturating_sub(if self.input.is_some() { 1 } else { 0 });

        for (y, (i, item)) in self
            .items
            .iter()
            .enumerate()
            .skip(self.scroll)
            .take(self.items_height)
            .enumerate()
        {
            let prefix = if i < 9 {
                format!("[F{}] ", i + 1)
            } else {
                "     ".to_string()
            };
            canvas.write_str(y, 1, &prefix);

            let x = 1 + prefix.display_width();
            let max_width = canvas.width().saturating_sub(x);
            match &item.content {
                SelectorContent::Normal { label, sub_label } => {
                    let leftside_width = label.display_width();
                    if let Some(sub_label) = sub_label {
                        let rightside_max_width = max_width.saturating_sub(leftside_width + 3);
                        let rightside_width = min(sub_label.display_width(), rightside_max_width);
                        let rightside_x = canvas.width().saturating_sub(rightside_width);

                        canvas.write_str(
                            y,
                            rightside_x,
                            truncate_to_width(sub_label, rightside_max_width),
                        );
                    }

                    canvas.write_str(y, x, truncate_to_width(label, max_width));
                }
                SelectorContent::SearchMatch {
                    path,
                    pos,
                    line_text,
                    byte_range: range,
                } => {
                    let before_text = &line_text[..range.start];
                    let matched_text = &line_text[range.start..range.end];
                    let after_text = &line_text[range.end..];
                    let leftside = format!("{before_text}{matched_text}{after_text}",);

                    let file_name = path.rfind('/').map_or("", |i| &path[i + 1..]);
                    let rightside =
                        format!("{path}:{lineno}", path = file_name, lineno = pos.y + 1);

                    let leftside_width = min(leftside.display_width(), max_width);
                    let rightside_max_width = max_width.saturating_sub(leftside_width + 1);
                    let rightside_width = min(rightside.display_width(), rightside_max_width);
                    let rightside_x = canvas.width().saturating_sub(rightside_width);

                    canvas.write_str(y, x, truncate_to_width(&leftside, leftside_width));
                    canvas.write_str(
                        y,
                        rightside_x,
                        truncate_to_width(&rightside, rightside_max_width),
                    );

                    let x_off = before_text.display_width();
                    canvas.apply_style(
                        y,
                        x + x_off,
                        x + min(leftside_width, x_off + matched_text.display_width()),
                        Style {
                            fg: Color::Red,
                            ..Default::default()
                        },
                    );
                }
            }

            if self.scroll + y == self.selected_index {
                canvas.apply_style(y, 0, canvas.width(), theme_for("selector.selected"));
            }
        }

        if let Some(input) = self.input.as_mut() {
            input.relocate_scroll(canvas.width());
            canvas.write_str(
                self.items_height,
                1,
                truncate_to_width(&input.text(), canvas.width() - 2),
            );
            canvas.apply_style(
                self.items_height,
                1,
                canvas.width() - 2,
                theme_for("selector.input"),
            );
        }
    }

    fn handle_key_event(
        &mut self,
        editor: &mut Editor,
        compositor: &mut Compositor<Self::Context>,
        key: KeyEvent,
    ) -> HandledEvent {
        const NONE: KeyModifiers = KeyModifiers::NONE;
        const CTRL: KeyModifiers = KeyModifiers::CONTROL;
        // const ALT: KeyModifiers = KeyModifiers::ALT;
        // const SHIFT: KeyModifiers = KeyModifiers::SHIFT;

        match (key.code, key.modifiers) {
            (KeyCode::Enter, NONE) => {
                self.select_item(editor, compositor, self.selected_index);
            }
            (KeyCode::Down, NONE) => {
                self.selected_index =
                    min(self.selected_index + 1, self.items.len().saturating_sub(1));
                self.adjust_scroll();
            }
            (KeyCode::Up, NONE) => {
                self.selected_index = self.selected_index.saturating_sub(1);
                self.adjust_scroll();
            }
            (KeyCode::F(n), NONE) if n < 10 => {
                self.select_item(editor, compositor, (n - 1) as usize);
            }
            (KeyCode::Char('q'), CTRL) => {
                self.close();
            }
            _ => {
                if let Some(input) = self.input.as_mut() {
                    let prev_text = input.text();
                    input.consume_key_event(key);
                    let text = input.text();
                    if prev_text != text {
                        if let Some(callback) = self.changed_callback.as_mut() {
                            callback(editor, &text);
                        }
                    }
                }
            }
        }

        HandledEvent::Consumed
    }

    fn handle_key_batch_event(
        &mut self,
        _editor: &mut Editor,
        _compositor: &mut Compositor<Editor>,
        text: &str,
    ) -> HandledEvent {
        if let Some(input) = self.input.as_mut() {
            input.insert(&text.replace('\n', " "));
        }

        HandledEvent::Consumed
    }

    fn handle_mouse_event(
        &mut self,
        _ctx: &mut Self::Context,
        _compositor: &mut Compositor<Self::Context>,
        _kind: noa_compositor::terminal::MouseEventKind,
        _modifiers: noa_compositor::terminal::KeyModifiers,
        _surface_y: usize,
        _surface_x: usize,
    ) -> HandledEvent {
        HandledEvent::Consumed
    }
}
