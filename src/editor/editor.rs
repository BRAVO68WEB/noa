use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use arc_swap::ArcSwap;
use noa_buffer::{
    cursor::{Position, Range},
    raw_buffer::RawBuffer,
    undoable_raw_buffer::Change,
};
use noa_common::oops::OopsExt;
use noa_languages::language::Language;
use noa_proxy::{client::Client as ProxyClient, lsp_types::TextEdit, protocol::Notification};
use tokio::sync::{
    mpsc::{self, UnboundedSender},
    Notify,
};

use crate::{
    clipboard::{self, ClipboardProvider},
    document::{Document, DocumentId, DocumentManager},
    git::Repo,
    minimap::MiniMap,
};

pub struct Editor {
    pub documents: DocumentManager,
    pub clipboard: Box<dyn ClipboardProvider>,
    pub repo: Option<Arc<Repo>>,
    pub proxy: Arc<noa_proxy::client::Client>,
    pub render_request: Arc<Notify>,
}

impl Editor {
    pub fn new(
        workspace_dir: &Path,
        render_request: Arc<Notify>,
        notification_tx: UnboundedSender<Notification>,
    ) -> Editor {
        let repo = match Repo::open(workspace_dir) {
            Ok(repo) => Some(Arc::new(repo)),
            Err(err) => {
                notify_warn!("failed to open the git repository: {}", err);
                None
            }
        };

        let proxy = Arc::new(noa_proxy::client::Client::new(
            workspace_dir,
            notification_tx,
        ));

        Editor {
            documents: DocumentManager::new(),
            clipboard: clipboard::build_provider(),
            repo,
            proxy,
            render_request,
        }
    }

    pub fn open_file(&mut self, path: &Path, cursor_pos: Option<Position>) -> Result<()> {
        let mut doc = Document::new(path)?;

        let (lsp_sync_tx, lsp_sync_rx) = mpsc::unbounded_channel();
        tokio::spawn(lsp_file_sync_task(
            lsp_sync_rx,
            doc.id(),
            self.proxy.clone(),
            doc.raw_buffer().clone(),
            doc.path().to_owned(),
            doc.buffer().language(),
        ));

        let (git_diff_tx, git_diff_rx) = mpsc::unbounded_channel();
        tokio::task::spawn(git_diff_task(
            git_diff_rx,
            self.repo.clone(),
            doc.minimap().clone(),
            doc.path().to_owned(),
            self.render_request.clone(),
        ));

        doc.set_post_update_hook(move |version, raw_buffer, changes| {
            let _ = lsp_sync_tx.send((version, changes));
            let _ = git_diff_tx.send(raw_buffer.clone());
        });

        if let Some(pos) = cursor_pos {
            doc.buffer_mut().move_main_cursor_to_pos(pos);
            doc.flashes_mut().flash(Range::from_positions(pos, pos));
        }

        self.documents.add(doc);
        Ok(())
    }

    pub fn handle_notification(&mut self, notification: Notification) {
        match notification {
            Notification::Diagnostics { diags, path } => {
                if path != self.documents.current().path() {
                    return;
                }

                if let Some(diag) = diags.first() {
                    notify_warn!("{}: {:?}", diag.range.start.line + 1, diag.message);
                }
            }
        }
    }
}

/// Synchronizes the latest buffer text with the LSP server.
async fn lsp_file_sync_task(
    mut rx: mpsc::UnboundedReceiver<(usize, Vec<Change>)>,
    _doc_id: DocumentId,
    proxy: Arc<ProxyClient>,
    initial_buffer: RawBuffer,
    path: PathBuf,
    lang: &'static Language,
) {
    proxy
        .open_file(lang, &path, &initial_buffer.text())
        .await
        .oops();

    let path = path.clone();
    while let Some((version, mut changes)) = rx.recv().await {
        let edits = changes
            .drain(..)
            .map(|change| TextEdit {
                range: change.range.into(),
                new_text: change.insert_text,
            })
            .collect();

        proxy
            .incremental_update_file(lang, &path, edits, version)
            .await
            .oops();
    }
}

async fn git_diff_task(
    mut rx: mpsc::UnboundedReceiver<RawBuffer>,
    repo: Option<Arc<Repo>>,
    minimap: Arc<ArcSwap<MiniMap>>,
    path: PathBuf,
    render_request: Arc<Notify>,
) {
    while let Some(raw_buffer) = rx.recv().await {
        if let Some(repo) = &repo {
            let buffer_text = raw_buffer.text();
            let mut new_minimap = MiniMap::new();
            new_minimap.update_git_line_statuses(repo, &path, &buffer_text);
            minimap.store(Arc::new(new_minimap));
            render_request.notify_one();
        }
    }
}
