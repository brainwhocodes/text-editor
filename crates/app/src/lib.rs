//! AI Code Editor application crate.

pub mod events;

pub use events::{
    create_event_bridge, invoke_ui_update, spawn_event_processor,
    EventReceiver, EventSender, ThrottleConfig, UiEvent,
};
