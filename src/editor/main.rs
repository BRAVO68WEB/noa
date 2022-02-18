#![feature(test)]
#![feature(vec_retain_mut)]

extern crate test;

#[macro_use]
extern crate log;

use std::{path::PathBuf, sync::Arc, time::Duration};

use clap::Parser;

use noa_common::{logger::install_logger, time_report::TimeReport};
use noa_compositor::{terminal::Event, Compositor};
use theme::parse_default_theme;
use tokio::sync::{mpsc, oneshot, Notify};
use ui::{
    buffer_view::BufferView, completion_view::CompletionView, finder_view::FinderView,
    meta_line_view::MetaLineView, too_small_view::TooSmallView,
};

#[macro_use]
mod notification;

mod clipboard;
mod completion;
mod document;
mod editor;
mod flash;
mod git;
mod linemap;
mod markdown;
mod movement;
mod theme;
mod ui;
mod view;

#[derive(Parser, Debug)]
struct Args {
    #[clap(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

#[tokio::main]
async fn main() {
    let boot_time = TimeReport::new("boot time");

    // Parse the default theme here to print panics in stderr.
    parse_default_theme();

    install_logger("main");
    let args = Args::parse();

    let workspace_dir = args
        .files
        .iter()
        .find(|path| path.is_dir())
        .cloned()
        .unwrap_or_else(|| PathBuf::from("."));

    let render_request = Arc::new(Notify::new());
    let (_update_completion_request_tx, mut update_completion_request_rx) =
        mpsc::unbounded_channel();
    let (notification_tx, mut notification_rx) = mpsc::unbounded_channel();
    let mut editor = editor::Editor::new(&workspace_dir, render_request.clone(), notification_tx);
    let mut compositor = Compositor::new();

    let mut open_finder = true;
    for path in args.files {
        if !path.is_dir() {
            match editor.open_file(&path, None) {
                Ok(id) => {
                    editor.documents.switch_by_id(id);
                }
                Err(err) => {
                    notify_anyhow_error!(err);
                }
            }

            open_finder = false;
        }
    }

    let (quit_tx, mut quit_rx) = oneshot::channel();
    compositor.add_frontmost_layer(Box::new(TooSmallView::new("too small!")));
    compositor.add_frontmost_layer(Box::new(BufferView::new(quit_tx, render_request.clone())));
    compositor.add_frontmost_layer(Box::new(MetaLineView::new()));
    compositor.add_frontmost_layer(Box::new(FinderView::new(
        &editor,
        render_request.clone(),
        &workspace_dir,
    )));
    compositor.add_frontmost_layer(Box::new(CompletionView::new()));

    if open_finder {
        compositor
            .get_mut_surface_by_name::<FinderView>("finder")
            .set_active(true);
    }

    compositor.render_to_terminal(&mut editor);
    drop(boot_time);

    let mut idle_timer = tokio::time::interval(Duration::from_millis(1200));
    loop {
        let mut skip_rendering = false;
        tokio::select! {
            biased;

            _ = &mut quit_rx => {
                break;
            }

            Some(ev) = compositor.recv_terminal_event() => {
                let _event_tick_time = Some(TimeReport::new("I/O event handling"));
                match ev {
                    Event::Input(input) => {
                        compositor.handle_input(&mut editor, input);
                    }
                    Event::Resize { height, width } => {
                        compositor.resize_screen(height, width);
                    }
                }
            }

            Some(noti) = notification_rx.recv() => {
                trace!("proxy notification: {:?}", noti);
                editor.handle_notification(noti);
            }

            Some(doc_id) = update_completion_request_rx.recv() => {
                editor.documents.get_mut_document_by_id(doc_id).unwrap().update_completion();
            }

            _ = render_request.notified() => {
            }

            _ = idle_timer.tick()  => {
                editor.documents.current_mut().idle_job();
                skip_rendering = true;
            }
        }

        if !skip_rendering {
            compositor.render_to_terminal(&mut editor);
        }
        idle_timer.reset();
    }
}
