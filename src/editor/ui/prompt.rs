use std::ops::ControlFlow;

use noa_common::fuzzyvec::FuzzyVec;
use noa_compositor::{line_edit::LineEdit, Compositor};

use crate::{
    editor::Editor,
    ui::prompt_view::{PromptMode, PromptView},
};

pub fn prompt<S, F, C>(
    compositor: &mut Compositor<Editor>,
    editor: &mut Editor,
    mode: PromptMode,
    title: S,
    mut enter_callback: F,
    mut completion_callback: C,
) where
    S: Into<String>,
    F: FnMut(&mut Compositor<Editor>, &mut Editor, Option<String>) -> ControlFlow<()> + 'static,
    C: FnMut(&mut Editor, &LineEdit) -> Option<FuzzyVec<String>> + 'static,
{
    let title = title.into();
    let enter_cb = {
        let title = title.clone();
        editor.register_callback(move |compositor, editor| {
            let prompt_view: &mut PromptView = compositor.get_mut_surface_by_name("prompt");

            let result = if prompt_view.is_canceled() {
                Some(prompt_view.input().text())
            } else {
                None
            };

            match enter_callback(compositor, editor, result) {
                ControlFlow::Continue(()) => {}
                ControlFlow::Break(()) => {
                    let prompt_view: &mut PromptView = compositor.get_mut_surface_by_name("prompt");
                    prompt_view.deactivate();
                }
            }
        })
    };

    let completion_cb = {
        let title = title.clone();
        editor.register_callback(move |compositor, editor| {
            let prompt_view: &mut PromptView = compositor.get_mut_surface_by_name("prompt");
            if let Some(items) = completion_callback(editor, prompt_view.input()) {
                // prompt_view.set_completion_items(items);
            }
        })
    };

    let prompt_view: &mut PromptView = compositor.get_mut_surface_by_name("prompt");
    prompt_view.activate(PromptMode::SingleChar, title, enter_cb);
}
