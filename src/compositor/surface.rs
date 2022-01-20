use super::canvas::CanvasViewMut;
pub use crossterm::event::{KeyEvent, MouseEvent};

#[derive(Clone, Copy, Debug)]
pub enum Layout {
    Fixed { y: usize, x: usize },
    Center,
    AroundCursor,
}

#[derive(Clone, Copy, Debug)]
pub struct RectSize {
    pub height: usize,
    pub width: usize,
}

#[derive(Clone, Copy, Debug)]
pub enum HandledEvent {
    Consumed,
    Ignored,
}

pub trait Surface {
    type Context;

    fn name(&self) -> &str;
    fn is_visible(&self, ctx: &mut Self::Context) -> bool;
    fn layout(&self, ctx: &mut Self::Context, screen_size: RectSize) -> (Layout, RectSize);
    /// Returns the cursor position in surface-local `(y, x)`. `None` if the cursor
    /// is hidden.
    fn cursor_position(&self, ctx: &mut Self::Context) -> Option<(usize, usize)>;
    /// Render its contents into the canvas. It must fill the whole canvas; the
    /// canvas can be the newly created one due to, for example, screen resizing.
    fn render(&mut self, ctx: &mut Self::Context, canvas: CanvasViewMut<'_>);
    fn handle_key_event(&mut self, ctx: &mut Self::Context, key: KeyEvent) -> HandledEvent;
    fn handle_mouse_event(&mut self, _ctx: &mut Self::Context, _ev: MouseEvent) -> HandledEvent {
        HandledEvent::Ignored
    }
    fn handle_key_batch_event(&mut self, ctx: &mut Self::Context, input: &str) -> HandledEvent;
}
