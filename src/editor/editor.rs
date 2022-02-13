use std::{path::Path, sync::Arc};

use tokio::sync::Notify;

use crate::{
    clipboard::{self, ClipboardProvider},
    document::DocumentManager,
    git::Repo,
};

pub struct Editor {
    pub documents: DocumentManager,
    pub clipboard: Box<dyn ClipboardProvider>,
    pub repo: Option<Arc<Repo>>,
    pub proxy: Arc<noa_proxy::client::Client>,
    pub render_request: Arc<Notify>,
}

impl Editor {
    pub fn new(workspace_dir: &Path, render_request: Arc<Notify>) -> Editor {
        let repo = match Repo::open(workspace_dir) {
            Ok(repo) => Some(Arc::new(repo)),
            Err(err) => {
                notify_warn!("failed to open the git repository: {}", err);
                None
            }
        };

        Editor {
            documents: DocumentManager::new(),
            clipboard: clipboard::build_provider().unwrap_or_else(clipboard::build_dummy_provider),
            repo,
            proxy: Arc::new(noa_proxy::client::Client::new(workspace_dir)),
            render_request,
        }
    }
}
