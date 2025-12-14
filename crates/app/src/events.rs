//! Event bridge for UI-thread integration.
//!
//! Provides a channel-based system for background services to communicate
//! with the Slint UI thread safely.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// Events that can be sent to the UI thread.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum UiEvent {
    /// Editor content changed - includes line range for partial updates
    EditorContentChanged {
        start_line: usize,
        end_line: usize,
    },
    /// Cursor position changed
    CursorMoved {
        line: usize,
        column: usize,
    },
    /// File loaded into editor
    FileLoaded {
        filename: String,
        language: String,
    },
    /// File save status changed
    FileSaveStatus {
        filename: String,
        is_dirty: bool,
    },
    /// Explorer tree needs refresh
    ExplorerRefresh,
    /// AI chat response chunk received
    ChatResponseChunk {
        content: String,
    },
    /// AI chat response complete
    ChatResponseComplete,
    /// AI chat error
    ChatError {
        message: String,
    },
    /// Diff hunks available for review
    DiffAvailable {
        hunk_count: usize,
    },
    /// Diff hunk accepted/rejected
    DiffHunkResolved {
        remaining: usize,
    },
    /// Status bar update
    StatusUpdate {
        message: String,
    },
}

/// Configuration for event throttling.
#[derive(Debug, Clone)]
pub struct ThrottleConfig {
    /// Minimum interval between editor repaint events
    pub editor_repaint_interval: Duration,
    /// Minimum interval between cursor update events  
    pub cursor_update_interval: Duration,
}

impl Default for ThrottleConfig {
    fn default() -> Self {
        Self {
            editor_repaint_interval: Duration::from_millis(16), // ~60fps
            cursor_update_interval: Duration::from_millis(50),  // 20 updates/sec
        }
    }
}

/// Tracks last event times for throttling.
struct ThrottleState {
    last_editor_repaint: Option<Instant>,
    last_cursor_update: Option<Instant>,
}

impl ThrottleState {
    fn new() -> Self {
        Self {
            last_editor_repaint: None,
            last_cursor_update: None,
        }
    }

    fn should_emit_editor_repaint(&mut self, config: &ThrottleConfig) -> bool {
        let now = Instant::now();
        match self.last_editor_repaint {
            Some(last) if now.duration_since(last) < config.editor_repaint_interval => false,
            _ => {
                self.last_editor_repaint = Some(now);
                true
            }
        }
    }

    fn should_emit_cursor_update(&mut self, config: &ThrottleConfig) -> bool {
        let now = Instant::now();
        match self.last_cursor_update {
            Some(last) if now.duration_since(last) < config.cursor_update_interval => false,
            _ => {
                self.last_cursor_update = Some(now);
                true
            }
        }
    }
}

/// Sender side of the event bridge - used by background services.
#[derive(Clone)]
pub struct EventSender {
    tx: mpsc::Sender<UiEvent>,
    #[allow(dead_code)]
    config: Arc<ThrottleConfig>,
}

impl EventSender {
    /// Send an event to the UI thread.
    pub async fn send(&self, event: UiEvent) -> Result<(), mpsc::error::SendError<UiEvent>> {
        self.tx.send(event).await
    }

    /// Try to send an event without blocking.
    #[allow(dead_code)]
    pub fn try_send(&self, event: UiEvent) -> Result<(), mpsc::error::TrySendError<UiEvent>> {
        self.tx.try_send(event)
    }

    /// Get the throttle configuration.
    #[allow(dead_code)]
    pub fn config(&self) -> &ThrottleConfig {
        &self.config
    }
}

/// Receiver side of the event bridge - used by the UI thread.
pub struct EventReceiver {
    rx: mpsc::Receiver<UiEvent>,
    throttle_state: ThrottleState,
    config: Arc<ThrottleConfig>,
}

impl EventReceiver {
    /// Receive the next event, applying throttling rules.
    pub async fn recv(&mut self) -> Option<UiEvent> {
        loop {
            let event = self.rx.recv().await?;
            
            // Apply throttling based on event type
            let should_emit = match &event {
                UiEvent::EditorContentChanged { .. } => {
                    self.throttle_state.should_emit_editor_repaint(&self.config)
                }
                UiEvent::CursorMoved { .. } => {
                    self.throttle_state.should_emit_cursor_update(&self.config)
                }
                // All other events pass through without throttling
                _ => true,
            };

            if should_emit {
                return Some(event);
            }
            // If throttled, continue to next event
        }
    }

    /// Try to receive an event without blocking.
    #[allow(dead_code)]
    pub fn try_recv(&mut self) -> Result<UiEvent, mpsc::error::TryRecvError> {
        loop {
            let event = self.rx.try_recv()?;
            
            let should_emit = match &event {
                UiEvent::EditorContentChanged { .. } => {
                    self.throttle_state.should_emit_editor_repaint(&self.config)
                }
                UiEvent::CursorMoved { .. } => {
                    self.throttle_state.should_emit_cursor_update(&self.config)
                }
                _ => true,
            };

            if should_emit {
                return Ok(event);
            }
        }
    }
}

/// Create a new event bridge channel pair.
///
/// # Arguments
/// * `buffer_size` - Size of the channel buffer
/// * `config` - Optional throttle configuration (uses defaults if None)
///
/// # Returns
/// A tuple of (EventSender, EventReceiver)
pub fn create_event_bridge(
    buffer_size: usize,
    config: Option<ThrottleConfig>,
) -> (EventSender, EventReceiver) {
    let (tx, rx) = mpsc::channel(buffer_size);
    let config = Arc::new(config.unwrap_or_default());

    let sender = EventSender {
        tx,
        config: Arc::clone(&config),
    };

    let receiver = EventReceiver {
        rx,
        throttle_state: ThrottleState::new(),
        config,
    };

    (sender, receiver)
}

/// Helper to invoke UI updates from the event loop.
/// 
/// This wraps `slint::invoke_from_event_loop` with proper error handling.
pub fn invoke_ui_update<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    let _ = slint::invoke_from_event_loop(f);
}

/// Spawns a background task that processes events and updates the UI.
///
/// # Arguments
/// * `receiver` - The event receiver
/// * `weak_window` - Weak reference to the Slint window
/// * `handler` - Function to handle each event and update UI state
#[allow(dead_code)]
pub fn spawn_event_processor<W, F>(
    mut receiver: EventReceiver,
    weak_window: slint::Weak<W>,
    handler: F,
) -> tokio::task::JoinHandle<()>
where
    W: slint::ComponentHandle + 'static,
    F: Fn(&W, UiEvent) + Send + Sync + Clone + 'static,
{
    tokio::spawn(async move {
        while let Some(event) = receiver.recv().await {
            let weak = weak_window.clone();
            let handler = handler.clone();
            
            invoke_ui_update(move || {
                if let Some(window) = weak.upgrade() {
                    handler(&window, event);
                }
            });
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_bridge_basic() {
        let (sender, mut receiver) = create_event_bridge(16, None);

        sender.send(UiEvent::StatusUpdate {
            message: "test".to_string(),
        }).await.unwrap();

        let event = receiver.recv().await.unwrap();
        match event {
            UiEvent::StatusUpdate { message } => assert_eq!(message, "test"),
            _ => panic!("unexpected event"),
        }
    }

    #[tokio::test]
    async fn test_throttling() {
        let config = ThrottleConfig {
            cursor_update_interval: Duration::from_millis(100),
            ..Default::default()
        };
        let (sender, mut receiver) = create_event_bridge(16, Some(config));

        // Send multiple cursor events rapidly
        for i in 0..5 {
            sender.send(UiEvent::CursorMoved { line: i, column: 0 }).await.unwrap();
        }

        // Only first should pass through immediately due to throttling
        let event = receiver.try_recv().unwrap();
        match event {
            UiEvent::CursorMoved { line: 0, .. } => {}
            _ => panic!("unexpected event"),
        }
    }
}
