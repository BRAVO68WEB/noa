use std::{path::Path, sync::Arc};

use anyhow::Result;
use noa_compositor::terminal::Event;
use tokio::sync::{oneshot, Notify};

use crate::{
    clipboard::{self, ClipboardProvider},
    document::{Document, DocumentManager},
    git::Repo,
    minimap::MiniMap,
    theme::Theme,
    ui::buffer_view::BufferView,
};

pub struct Editor {
    pub theme: Theme,
    pub documents: DocumentManager,
    pub clipboard: Box<dyn ClipboardProvider>,
    pub repo: Option<Arc<Repo>>,
    pub render_request: Arc<Notify>,
}

impl Editor {
    pub fn new(workspace_dir: &Path, render_request: Arc<Notify>) -> Editor {
        let repo = match Repo::open(workspace_dir) {
            Ok(repo) => Some(Arc::new(repo)),
            Err(err) => {
                warn!("failed to open the git repository: {}", err);
                notify_warn!("Not in a Git repo");
                None
            }
        };

        Editor {
            theme: Theme::default(),
            documents: DocumentManager::new(),
            clipboard: clipboard::build_provider().unwrap_or_else(clipboard::build_dummy_provider),
            repo,
            render_request,
        }
    }
}
