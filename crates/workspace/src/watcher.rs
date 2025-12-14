//! File system watching for workspace changes.

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;
use tokio::sync::broadcast;

/// Events emitted by the file watcher.
#[derive(Debug, Clone)]
pub enum WatchEvent {
    /// File or directory created
    Created(PathBuf),
    /// File or directory modified
    Modified(PathBuf),
    /// File or directory deleted
    Deleted(PathBuf),
    /// File or directory renamed
    Renamed { from: PathBuf, to: PathBuf },
    /// Error occurred
    Error(String),
}

/// File system watcher for a workspace.
pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    event_tx: broadcast::Sender<WatchEvent>,
}

impl FileWatcher {
    /// Create a new file watcher for the given root path.
    pub fn new(root: &Path) -> Result<Self, String> {
        let (event_tx, _) = broadcast::channel(256);
        let tx_clone = event_tx.clone();

        let (sync_tx, sync_rx) = mpsc::channel::<notify::Result<Event>>();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = sync_tx.send(res);
            },
            Config::default().with_poll_interval(Duration::from_secs(1)),
        )
        .map_err(|e| e.to_string())?;

        watcher
            .watch(root, RecursiveMode::Recursive)
            .map_err(|e| e.to_string())?;

        // Spawn thread to process events
        std::thread::spawn(move || {
            while let Ok(res) = sync_rx.recv() {
                match res {
                    Ok(event) => {
                        let watch_events = Self::convert_event(event);
                        for we in watch_events {
                            let _ = tx_clone.send(we);
                        }
                    }
                    Err(e) => {
                        let _ = tx_clone.send(WatchEvent::Error(e.to_string()));
                    }
                }
            }
        });

        Ok(Self {
            _watcher: watcher,
            event_tx,
        })
    }

    /// Subscribe to watch events.
    pub fn subscribe(&self) -> broadcast::Receiver<WatchEvent> {
        self.event_tx.subscribe()
    }

    /// Convert notify event to our watch event.
    fn convert_event(event: Event) -> Vec<WatchEvent> {
        let paths = event.paths;
        match event.kind {
            EventKind::Create(_) => {
                paths.into_iter().map(WatchEvent::Created).collect()
            }
            EventKind::Modify(_) => {
                paths.into_iter().map(WatchEvent::Modified).collect()
            }
            EventKind::Remove(_) => {
                paths.into_iter().map(WatchEvent::Deleted).collect()
            }
            EventKind::Any | EventKind::Access(_) | EventKind::Other => Vec::new(),
        }
    }
}

impl std::fmt::Debug for FileWatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileWatcher").finish()
    }
}
