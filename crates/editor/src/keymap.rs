use std::collections::HashMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Char(char),
    Enter,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Tab,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct KeyChord {
    pub code: KeyCode,
    pub mods: KeyModifiers,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Movement {
    Left,
    Right,
    Up,
    Down,
    WordLeft,
    WordRight,
    LineStart,
    LineEnd,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KeyAction {
    Newline,
    Backspace,
    Delete,
    DeleteWordBackward,
    DeleteWordForward,
    DeleteLine,
    Undo,
    Redo,
    Copy,
    Cut,
    Paste,
    Indent,
    Outdent,
    DuplicateLine,
    ToggleComment,
    Move { movement: Movement, extend: bool },
}

#[derive(Debug, Clone, Default)]
pub struct Keymap {
    bindings: HashMap<KeyChord, KeyAction>,
}

impl Keymap {
    pub fn with_defaults() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert(
            KeyChord { code: KeyCode::Enter, mods: KeyModifiers::default() },
            KeyAction::Newline,
        );
        bindings.insert(
            KeyChord { code: KeyCode::Backspace, mods: KeyModifiers::default() },
            KeyAction::Backspace,
        );
        bindings.insert(
            KeyChord { code: KeyCode::Delete, mods: KeyModifiers::default() },
            KeyAction::Delete,
        );
        bindings.insert(
            KeyChord { code: KeyCode::Left, mods: KeyModifiers::default() },
            KeyAction::Move { movement: Movement::Left, extend: false },
        );
        bindings.insert(
            KeyChord { code: KeyCode::Right, mods: KeyModifiers::default() },
            KeyAction::Move { movement: Movement::Right, extend: false },
        );
        bindings.insert(
            KeyChord { code: KeyCode::Up, mods: KeyModifiers::default() },
            KeyAction::Move { movement: Movement::Up, extend: false },
        );
        bindings.insert(
            KeyChord { code: KeyCode::Down, mods: KeyModifiers::default() },
            KeyAction::Move { movement: Movement::Down, extend: false },
        );
        bindings.insert(
            KeyChord { code: KeyCode::Left, mods: KeyModifiers { ctrl: true, ..KeyModifiers::default() } },
            KeyAction::Move { movement: Movement::WordLeft, extend: false },
        );
        bindings.insert(
            KeyChord { code: KeyCode::Right, mods: KeyModifiers { ctrl: true, ..KeyModifiers::default() } },
            KeyAction::Move { movement: Movement::WordRight, extend: false },
        );
        bindings.insert(
            KeyChord { code: KeyCode::Home, mods: KeyModifiers::default() },
            KeyAction::Move { movement: Movement::LineStart, extend: false },
        );
        bindings.insert(
            KeyChord { code: KeyCode::End, mods: KeyModifiers::default() },
            KeyAction::Move { movement: Movement::LineEnd, extend: false },
        );
        bindings.insert(
            KeyChord { code: KeyCode::Char('z'), mods: KeyModifiers { ctrl: true, ..KeyModifiers::default() } },
            KeyAction::Undo,
        );
        bindings.insert(
            KeyChord { code: KeyCode::Char('y'), mods: KeyModifiers { ctrl: true, ..KeyModifiers::default() } },
            KeyAction::Redo,
        );
        bindings.insert(
            KeyChord { code: KeyCode::Char('c'), mods: KeyModifiers { ctrl: true, ..KeyModifiers::default() } },
            KeyAction::Copy,
        );
        bindings.insert(
            KeyChord { code: KeyCode::Char('x'), mods: KeyModifiers { ctrl: true, ..KeyModifiers::default() } },
            KeyAction::Cut,
        );
        bindings.insert(
            KeyChord { code: KeyCode::Char('v'), mods: KeyModifiers { ctrl: true, ..KeyModifiers::default() } },
            KeyAction::Paste,
        );
        bindings.insert(
            KeyChord { code: KeyCode::Tab, mods: KeyModifiers::default() },
            KeyAction::Indent,
        );
        bindings.insert(
            KeyChord { code: KeyCode::Tab, mods: KeyModifiers { shift: true, ..KeyModifiers::default() } },
            KeyAction::Outdent,
        );
        Self { bindings }
    }

    pub fn resolve(&self, chord: KeyChord) -> Option<KeyAction> {
        self.bindings.get(&chord).copied()
    }
}
