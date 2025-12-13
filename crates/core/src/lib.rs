use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use tokio::sync::mpsc;

pub type DocumentId = u64;

pub type ConversationId = u64;
pub type PatchProposalId = u64;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("io error: {0}")]
    Io(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    OpenWorkspace { path: PathBuf },
    OpenFile { path: PathBuf },
    SaveFile { document_id: DocumentId },
    CloseFile { document_id: DocumentId },
    CreateFile { path: PathBuf },
    RenamePath { from: PathBuf, to: PathBuf },
    DeletePath { path: PathBuf },
    ChatSend { conversation_id: ConversationId, user_message: String },
    ApplyPatch { document_id: DocumentId, patch: String },
    RejectPatch { proposal_id: PatchProposalId },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    WorkspaceOpened { root: PathBuf },
    WorkspaceTreeUpdated,
    DocumentOpened { document_id: DocumentId, path: PathBuf, text: String },
    DocumentSaved { document_id: DocumentId },
    DocumentClosed { document_id: DocumentId },
    ChatMessageAdded { conversation_id: ConversationId, role: ChatRole, content: String },
    AiStreamDelta { conversation_id: ConversationId, delta: String },
    PatchProposed { proposal_id: PatchProposalId, document_id: DocumentId, patch: String },
    Error { message: String },
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

pub type CommandSender = mpsc::Sender<Command>;
pub type CommandReceiver = mpsc::Receiver<Command>;
pub type EventSender = mpsc::Sender<Event>;
pub type EventReceiver = mpsc::Receiver<Event>;

pub fn new_bus(buffer: usize) -> (CommandSender, CommandReceiver, EventSender, EventReceiver) {
    let (command_tx, command_rx) = mpsc::channel(buffer);
    let (event_tx, event_rx) = mpsc::channel(buffer);
    (command_tx, command_rx, event_tx, event_rx)
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppState {
    pub workspace: WorkspaceState,
    pub editor: EditorState,
    pub chat: ChatState,
    pub diff: DiffState,
    pub theme: ThemeState,
    pub settings: SettingsState,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceState {
    pub root: Option<PathBuf>,
    pub open_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EditorState {
    pub active_document: Option<DocumentId>,
    pub open_documents: Vec<OpenDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenDocument {
    pub document_id: DocumentId,
    pub path: Option<PathBuf>,
    pub is_dirty: bool,
    pub cursor_char_idx: usize,
}

impl Default for OpenDocument {
    fn default() -> Self {
        Self {
            document_id: 0,
            path: None,
            is_dirty: false,
            cursor_char_idx: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatState {
    pub active_conversation: Option<ConversationId>,
    pub conversations: Vec<Conversation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Conversation {
    pub id: ConversationId,
    pub title: String,
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiffState {
    pub proposals: Vec<PatchProposal>,
    pub active_proposal: Option<PatchProposalId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchProposal {
    pub id: PatchProposalId,
    pub document_id: DocumentId,
    pub patch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeState {
    pub theme_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsState {
    pub model_id: String,
}
