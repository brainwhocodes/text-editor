use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

slint::slint! {
    import { Button, ComboBox, LineEdit, ScrollView } from "std-widgets.slint";

    component PanelHeader inherits Rectangle {
        in property <string> title;
        in property <bool> collapsible: false;
        in-out property <bool> collapsed: false;
        
        height: 28px;
        background: #252526;

        HorizontalLayout {
            padding-left: 8px;
            padding-right: 8px;
            spacing: 4px;

            if (root.collapsible) : Text {
                text: root.collapsed ? "▶" : "▼";
                color: #cccccc;
                font-size: 10px;
                vertical-alignment: center;
            }

            Text {
                text: root.title;
                color: #cccccc;
                font-size: 11px;
                font-weight: 700;
                vertical-alignment: center;
            }
        }

        TouchArea {
            clicked => {
                if (root.collapsible) {
                    root.collapsed = !root.collapsed;
                }
            }
        }
    }

    component FileTreeItem inherits Rectangle {
        in property <string> name;
        in property <string> icon: "📄";
        in property <int> indent: 0;
        in property <bool> is_folder: false;
        in property <bool> expanded: false;
        in property <bool> selected: false;
        callback clicked();

        height: 22px;
        background: selected ? #094771 : transparent;

        HorizontalLayout {
            padding-left: 8px + root.indent * 12px;
            spacing: 4px;

            Text {
                text: is_folder ? (expanded ? "▼" : "▶") : " ";
                color: #858585;
                font-size: 9px;
                width: 12px;
                vertical-alignment: center;
            }

            Text {
                text: root.icon;
                font-size: 12px;
                vertical-alignment: center;
            }

            Text {
                text: root.name;
                color: root.selected ? #ffffff : #cccccc;
                font-size: 12px;
                vertical-alignment: center;
                overflow: elide;
            }
        }

        TouchArea {
            clicked => { root.clicked(); }
        }
    }

    component ExplorerPanel inherits Rectangle {
        in property <string> workspace_name: "text-editor";
        in-out property <int> selected_file: 0;
        callback file_selected(int);

        background: #252526;

        VerticalLayout {
            spacing: 0px;

            PanelHeader {
                title: root.workspace_name;
                collapsible: true;
            }

            ScrollView {
                min-height: 0px;
                vertical-stretch: 1;

                VerticalLayout {
                    spacing: 0px;
                    padding-top: 4px;

                    FileTreeItem {
                        name: "src";
                        icon: "📁";
                        is_folder: true;
                        expanded: true;
                        indent: 0;
                    }

                    FileTreeItem {
                        name: "main.rs";
                        icon: "🦀";
                        indent: 1;
                        selected: root.selected_file == 0;
                        clicked => { root.selected_file = 0; root.file_selected(0); }
                    }

                    FileTreeItem {
                        name: "lib.rs";
                        icon: "🦀";
                        indent: 1;
                        selected: root.selected_file == 1;
                        clicked => { root.selected_file = 1; root.file_selected(1); }
                    }

                    FileTreeItem {
                        name: "editor";
                        icon: "📁";
                        is_folder: true;
                        expanded: true;
                        indent: 1;
                    }

                    FileTreeItem {
                        name: "engine.rs";
                        icon: "🦀";
                        indent: 2;
                        selected: root.selected_file == 2;
                        clicked => { root.selected_file = 2; root.file_selected(2); }
                    }

                    FileTreeItem {
                        name: "Cargo.toml";
                        icon: "📦";
                        indent: 0;
                        selected: root.selected_file == 3;
                        clicked => { root.selected_file = 3; root.file_selected(3); }
                    }

                    FileTreeItem {
                        name: "README.md";
                        icon: "📝";
                        indent: 0;
                        selected: root.selected_file == 4;
                        clicked => { root.selected_file = 4; root.file_selected(4); }
                    }
                }
            }
        }
    }

    component EditorTab inherits Rectangle {
        in property <string> filename;
        in property <bool> active: false;
        in property <bool> dirty: false;
        callback activated();
        callback closed();

        width: 140px;
        height: 32px;
        background: active ? #1e1e1e : #2d2d2d;
        border-width: active ? 0px : 0px;

        HorizontalLayout {
            padding-left: 12px;
            padding-right: 4px;
            spacing: 4px;

            Text {
                text: root.filename;
                color: active ? #ffffff : #969696;
                font-size: 12px;
                vertical-alignment: center;
                overflow: elide;
                horizontal-stretch: 1;
            }

            if (root.dirty) : Text {
                text: "●";
                color: #cccccc;
                font-size: 10px;
                vertical-alignment: center;
            }

            Rectangle {
                width: 20px;
                height: 20px;
                border-radius: 4px;
                background: ta-close.has-hover ? #3c3c3c : transparent;

                ta-close := TouchArea {
                    clicked => { root.closed(); }
                }

                Text {
                    text: "✕";
                    color: ta-close.has-hover ? #ffffff : #858585;
                    font-size: 11px;
                    horizontal-alignment: center;
                    vertical-alignment: center;
                }
            }
        }

        TouchArea {
            clicked => { root.activated(); }
        }
    }

    component LineNumber inherits Rectangle {
        in property <int> line: 1;
        in property <bool> current: false;

        width: 48px;
        height: 20px;
        background: transparent;

        HorizontalLayout {
            padding-right: 12px;

            Text {
                text: root.line;
                color: current ? #c6c6c6 : #6e7681;
                font-size: 12px;
                font-family: "Consolas";
                horizontal-alignment: right;
                vertical-alignment: center;
            }
        }
    }

    component EditorLine inherits Rectangle {
        in property <int> line_num: 1;
        in property <string> content;
        in property <bool> is_current: false;

        height: 20px;
        background: is_current ? #282828 : transparent;

        HorizontalLayout {
            spacing: 0px;

            LineNumber {
                line: root.line_num;
                current: root.is_current;
            }

            Rectangle {
                width: 1px;
                background: #3c3c3c;
            }

            Rectangle {
                horizontal-stretch: 1;

                HorizontalLayout {
                    padding-left: 8px;

                    Text {
                        text: root.content;
                        color: #cccccc;
                        font-size: 12px;
                        font-family: "Consolas";
                        vertical-alignment: center;
                    }
                }
            }
        }
    }

    component DiffLine inherits Rectangle {
        in property <string> line_text;
        in property <bool> is_added: false;
        in property <bool> is_removed: false;

        height: 22px;
        background: is_added ? #234023 : (is_removed ? #402323 : transparent);

        HorizontalLayout {
            spacing: 0px;

            Rectangle {
                width: 24px;
                background: is_added ? #2d4a2d : (is_removed ? #4a2d2d : #1e1e1e);

                Text {
                    text: is_added ? "+" : (is_removed ? "-" : " ");
                    color: is_added ? #73c973 : (is_removed ? #c97373 : #858585);
                    font-size: 12px;
                    font-family: "Consolas";
                    horizontal-alignment: center;
                    vertical-alignment: center;
                }
            }

            Rectangle {
                horizontal-stretch: 1;

                HorizontalLayout {
                    padding-left: 8px;

                    Text {
                        text: root.line_text;
                        color: is_added ? #8fdf8f : (is_removed ? #df8f8f : #cccccc);
                        font-size: 12px;
                        font-family: "Consolas";
                        vertical-alignment: center;
                    }
                }
            }
        }
    }

    component DiffHunk inherits Rectangle {
        in property <string> hunk_header: "@@ -10,5 +10,7 @@";
        callback accept();
        callback reject();

        background: #252526;
        border-radius: 4px;

        VerticalLayout {
            spacing: 0px;

            Rectangle {
                height: 32px;
                background: #2d2d2d;
                border-radius: 4px;

                HorizontalLayout {
                    padding-left: 12px;
                    padding-right: 8px;
                    spacing: 8px;

                    Text {
                        text: root.hunk_header;
                        color: #6a9fb5;
                        font-size: 11px;
                        font-family: "Consolas";
                        vertical-alignment: center;
                    }

                    Rectangle {
                        horizontal-stretch: 1;
                    }

                    Rectangle {
                        width: 70px;
                        height: 24px;
                        background: #2d4a2d;
                        border-radius: 4px;
                        y: 4px;

                        TouchArea {
                            clicked => { root.accept(); }
                        }

                        Text {
                            text: "Accept";
                            color: #8fdf8f;
                            font-size: 11px;
                            horizontal-alignment: center;
                            vertical-alignment: center;
                        }
                    }

                    Rectangle {
                        width: 70px;
                        height: 24px;
                        background: #4a2d2d;
                        border-radius: 4px;
                        y: 4px;

                        TouchArea {
                            clicked => { root.reject(); }
                        }

                        Text {
                            text: "Reject";
                            color: #df8f8f;
                            font-size: 11px;
                            horizontal-alignment: center;
                            vertical-alignment: center;
                        }
                    }
                }
            }

            VerticalLayout {
                padding: 4px;
                spacing: 0px;

                DiffLine { line_text: "    let x = 10;"; }
                DiffLine { line_text: "    let y = 20;"; is_removed: true; }
                DiffLine { line_text: "    let y = calculate_value();"; is_added: true; }
                DiffLine { line_text: "    let result = x + y;"; is_removed: true; }
                DiffLine { line_text: "    let result = process(x, y);"; is_added: true; }
                DiffLine { line_text: "    return result;"; is_added: true; }
                DiffLine { line_text: "    println!(\"{}\", result);"; }
            }
        }
    }

    component EditorWidget inherits Rectangle {
        in-out property <bool> has_pending_diff: true;
        in-out property <int> current_line: 3;

        background: #1e1e1e;

        ScrollView {
            min-height: 0px;
            vertical-stretch: 1;

            VerticalLayout {
                spacing: 0px;
                padding-top: 4px;

                EditorLine { line_num: 1; content: "use std::io;"; }
                EditorLine { line_num: 2; content: ""; }
                EditorLine { line_num: 3; content: "fn main() {"; is_current: root.current_line == 3; }
                EditorLine { line_num: 4; content: "    println!(\"Hello, world!\");"; }
                EditorLine { line_num: 5; content: "}"; }
                EditorLine { line_num: 6; content: ""; }

                if (root.has_pending_diff) : DiffHunk {
                    hunk_header: "@@ -7,3 +7,5 @@ fn main()";
                    accept => { }
                    reject => { }
                }

                EditorLine { line_num: 7; content: "fn calculate(x: i32, y: i32) -> i32 {"; }
                EditorLine { line_num: 8; content: "    x + y"; }
                EditorLine { line_num: 9; content: "}"; }
                EditorLine { line_num: 10; content: ""; }

                if (root.has_pending_diff) : DiffHunk {
                    hunk_header: "@@ -11,2 +13,4 @@ fn calculate()";
                    accept => { }
                    reject => { }
                }

                EditorLine { line_num: 11; content: "fn other_function() {"; }
                EditorLine { line_num: 12; content: "    // TODO: implement"; }
                EditorLine { line_num: 13; content: "}"; }
            }
        }
    }

    component StatusBar inherits Rectangle {
        in property <string> cursor_position: "Ln 3, Col 1";
        in property <string> language: "Rust";
        in property <string> encoding: "UTF-8";
        in property <string> line_ending: "LF";
        in property <string> ai_model: "gpt-4o-mini";

        height: 24px;
        background: #007acc;

        HorizontalLayout {
            padding-left: 12px;
            padding-right: 12px;
            spacing: 16px;

            Text {
                text: root.cursor_position;
                color: #ffffff;
                font-size: 11px;
                vertical-alignment: center;
            }

            Text {
                text: root.encoding;
                color: #ffffff;
                font-size: 11px;
                vertical-alignment: center;
            }

            Text {
                text: root.line_ending;
                color: #ffffff;
                font-size: 11px;
                vertical-alignment: center;
            }

            Rectangle {
                horizontal-stretch: 1;
            }

            Text {
                text: root.language;
                color: #ffffff;
                font-size: 11px;
                vertical-alignment: center;
            }

            Rectangle {
                width: 1px;
                height: 14px;
                background: #ffffff40;
                y: 5px;
            }

            Text {
                text: "AI: " + root.ai_model;
                color: #ffffff;
                font-size: 11px;
                vertical-alignment: center;
            }
        }
    }

    component EditorArea inherits Rectangle {
        in-out property <bool> has_pending_diff: true;
        in-out property <int> active_tab: 0;
        in property <string> cursor_position: "Ln 3, Col 1";
        in property <string> language: "Rust";
        in property <string> ai_model: "gpt-4o-mini";
        callback tab_activated(int);
        callback tab_closed(int);
        callback accept_all();
        callback reject_all();
        
        background: #1e1e1e;

        VerticalLayout {
            spacing: 0px;

            Rectangle {
                height: 36px;
                background: #2d2d2d;

                HorizontalLayout {
                    padding-left: 4px;
                    padding-right: 8px;
                    spacing: 0px;

                    EditorTab {
                        filename: "main.rs";
                        active: root.active_tab == 0;
                        dirty: true;
                        activated => { root.active_tab = 0; root.tab_activated(0); }
                        closed => { root.tab_closed(0); }
                    }

                    EditorTab {
                        filename: "lib.rs";
                        active: root.active_tab == 1;
                        dirty: false;
                        activated => { root.active_tab = 1; root.tab_activated(1); }
                        closed => { root.tab_closed(1); }
                    }

                    EditorTab {
                        filename: "engine.rs";
                        active: root.active_tab == 2;
                        dirty: false;
                        activated => { root.active_tab = 2; root.tab_activated(2); }
                        closed => { root.tab_closed(2); }
                    }

                    Rectangle {
                        horizontal-stretch: 1;
                    }

                    if (root.has_pending_diff) : Rectangle {
                        width: 90px;
                        height: 26px;
                        background: #2d4a2d;
                        border-radius: 4px;
                        y: 5px;

                        TouchArea {
                            clicked => { root.accept_all(); }
                        }

                        Text {
                            text: "Accept All";
                            color: #8fdf8f;
                            font-size: 11px;
                            horizontal-alignment: center;
                            vertical-alignment: center;
                        }
                    }

                    if (root.has_pending_diff) : Rectangle {
                        width: 8px;
                    }

                    if (root.has_pending_diff) : Rectangle {
                        width: 90px;
                        height: 26px;
                        background: #4a2d2d;
                        border-radius: 4px;
                        y: 5px;

                        TouchArea {
                            clicked => { root.reject_all(); }
                        }

                        Text {
                            text: "Reject All";
                            color: #df8f8f;
                            font-size: 11px;
                            horizontal-alignment: center;
                            vertical-alignment: center;
                        }
                    }
                }
            }

            EditorWidget {
                has_pending_diff: root.has_pending_diff;
                vertical-stretch: 1;
            }

            StatusBar {
                cursor_position: root.cursor_position;
                language: root.language;
                ai_model: root.ai_model;
            }
        }
    }

    component SidePanel inherits Rectangle {
        in-out property <string> chat_input;
        in-out property <string> chat_output;
        in-out property <string> api_key_input;
        in-out property <string> key_status;
        in-out property <string> model_id;
        in-out property <string> model_status;
        callback send_chat(message: string);
        callback save_api_key(key: string);
        callback remove_api_key();
        callback save_model(model: string);

        background: #1e1e1e;

        VerticalLayout {
            spacing: 0px;

            PanelHeader {
                title: "AI CHAT";
            }

            Rectangle {
                background: #1e1e1e;

                VerticalLayout {
                    padding: 16px;
                    spacing: 16px;

                    VerticalLayout {
                        spacing: 12px;

                        Text {
                            text: "AI Settings";
                            color: #cccccc;
                            font-size: 13px;
                            font-weight: 600;
                        }

                        VerticalLayout {
                            spacing: 8px;

                            Text {
                                text: root.model_status;
                                color: #858585;
                                font-size: 11px;
                            }

                            ComboBox {
                                model: [
                                    "openai/gpt-4o-mini",
                                    "openai/gpt-4o",
                                    "anthropic/claude-3.5-sonnet",
                                    "google/gemini-2.0-flash-001",
                                ];
                                current-value <=> root.model_id;
                                selected(value) => { root.save_model(value); }
                            }
                        }

                        VerticalLayout {
                            spacing: 8px;

                            Text {
                                text: root.key_status;
                                color: #858585;
                                font-size: 11px;
                            }

                            LineEdit {
                                text <=> root.api_key_input;
                                placeholder-text: "OpenRouter API Key";
                                input-type: password;
                            }

                            HorizontalLayout {
                                spacing: 8px;

                                Button {
                                    text: "Save";
                                    clicked => { root.save_api_key(root.api_key_input); }
                                }

                                Button {
                                    text: "Remove";
                                    clicked => { root.remove_api_key(); }
                                }
                            }
                        }
                    }

                    Rectangle {
                        height: 1px;
                        background: #3e3e42;
                    }

                    ScrollView {
                        min-height: 0px;
                        vertical-stretch: 1;

                        VerticalLayout {
                            padding: 8px;

                            Text {
                                text: root.chat_output;
                                color: #cccccc;
                                font-size: 12px;
                                wrap: word-wrap;
                            }
                        }
                    }

                    VerticalLayout {
                        spacing: 8px;

                        LineEdit {
                            text <=> root.chat_input;
                            placeholder-text: "Ask AI assistant...";
                        }

                        Button {
                            text: "Send";
                            clicked => { root.send_chat(root.chat_input); }
                        }
                    }
                }
            }
        }
    }

    export component AppWindow inherits Window {
        in-out property <string> chat_input;
        in-out property <string> chat_output;
        in-out property <string> api_key_input;
        in-out property <string> key_status;
        in-out property <string> model_id;
        in-out property <string> model_status;
        callback send_chat(message: string);
        callback save_api_key(key: string);
        callback remove_api_key();
        callback save_model(model: string);

        title: "AI Code Editor";
        width: 1280px;
        height: 800px;
        background: #1e1e1e;

        HorizontalLayout {
            spacing: 0px;

            ExplorerPanel {
                width: 260px;
            }

            Rectangle {
                width: 1px;
                background: #3e3e42;
            }

            EditorArea {
                horizontal-stretch: 1;
            }

            Rectangle {
                width: 1px;
                background: #3e3e42;
            }

            SidePanel {
                width: 380px;
                chat_input <=> root.chat_input;
                chat_output <=> root.chat_output;
                api_key_input <=> root.api_key_input;
                key_status <=> root.key_status;
                model_id <=> root.model_id;
                model_status <=> root.model_status;
                send_chat(msg) => { root.send_chat(msg); }
                save_api_key(key) => { root.save_api_key(key); }
                remove_api_key() => { root.remove_api_key(); }
                save_model(model) => { root.save_model(model); }
            }
        }
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let client = ai::OpenRouterClient::new().unwrap();
    let username = std::env::var("USERNAME").unwrap_or_else(|_| "user".to_string());
    let key_store = ai::KeyStore::new("ai-code-editor", username);
    let ai_service = ai::AiService::new(client, key_store.clone());

    let window = AppWindow::new()?;
    let weak = window.as_weak();
    let ai_service = std::sync::Arc::new(ai_service);
    let handle = rt.handle().clone();

    let initial_model = load_config().model;
    window.set_model_id(initial_model.clone().into());
    window.set_model_status(format!("Model: {initial_model}").into());

    {
        let weak_model = weak.clone();
        let handle_model = handle.clone();
        window.on_save_model(move |model: slint::SharedString| {
            let model: String = model.into();
            let weak_model = weak_model.clone();

            handle_model.spawn(async move {
                let status = tokio::task::spawn_blocking(move || {
                    let cfg = AppConfig { model: model.clone() };
                    match save_config(&cfg) {
                        Ok(()) => Ok(model),
                        Err(e) => Err(e),
                    }
                })
                .await
                .ok();

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(w) = weak_model.upgrade() {
                        match status {
                            Some(Ok(model)) => {
                                w.set_model_id(model.clone().into());
                                w.set_model_status(format!("Model: {model}").into());
                            }
                            Some(Err(e)) => {
                                w.set_model_status(format!("Model: error ({e})").into());
                            }
                            None => {
                                w.set_model_status("Model: error".into());
                            }
                        }
                    }
                });
            });
        });
    }

    window.set_key_status("API key: checking...".into());
    {
        let weak_init = weak.clone();
        let key_store_init = key_store.clone();
        handle.spawn(async move {
            let result = tokio::task::spawn_blocking(move || key_store_init.get_openrouter_key())
                .await
                .ok();

            let status = match result {
                Some(Ok(Some(_))) => "API key: set".to_string(),
                Some(Ok(None)) => "API key: not set".to_string(),
                Some(Err(e)) => format!("API key: error ({e})"),
                None => "API key: error".to_string(),
            };

            let _ = slint::invoke_from_event_loop(move || {
                if let Some(w) = weak_init.upgrade() {
                    w.set_key_status(status.into());
                }
            });
        });
    }

    {
        let weak_save = weak.clone();
        let key_store_save = key_store.clone();
        let handle_save = handle.clone();
        window.on_save_api_key(move |key: slint::SharedString| {
            let key: String = key.into();
            let weak_save = weak_save.clone();
            let key_store_save = key_store_save.clone();

            handle_save.spawn(async move {
                let result = tokio::task::spawn_blocking(move || key_store_save.set_openrouter_key(&key))
                    .await
                    .ok();

                let (status, clear_input) = match result {
                    Some(Ok(())) => ("API key: set".to_string(), true),
                    Some(Err(e)) => (format!("API key: error ({e})"), false),
                    None => ("API key: error".to_string(), false),
                };

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(w) = weak_save.upgrade() {
                        w.set_key_status(status.into());
                        if clear_input {
                            w.set_api_key_input("".into());
                        }
                    }
                });
            });
        });
    }

    {
        let weak_remove = weak.clone();
        let key_store_remove = key_store.clone();
        let handle_remove = handle.clone();
        window.on_remove_api_key(move || {
            let weak_remove = weak_remove.clone();
            let key_store_remove = key_store_remove.clone();

            handle_remove.spawn(async move {
                let result = tokio::task::spawn_blocking(move || key_store_remove.remove_openrouter_key())
                    .await
                    .ok();

                let status = match result {
                    Some(Ok(())) => "API key: not set".to_string(),
                    Some(Err(e)) => format!("API key: error ({e})"),
                    None => "API key: error".to_string(),
                };

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(w) = weak_remove.upgrade() {
                        w.set_key_status(status.into());
                        w.set_api_key_input("".into());
                    }
                });
            });
        });
    }

    window.on_send_chat(move |message: slint::SharedString| {
        let message: String = message.into();
        let ai_service = ai_service.clone();
        let weak = weak.clone();

        let mut model = default_model();

        if let Some(w) = weak.upgrade() {
            let current = w.get_chat_output().to_string();
            w.set_chat_output(format!("{current}You: {message}\n\n").into());
            w.set_chat_input("".into());
            model = w.get_model_id().to_string();
        }

        handle.spawn(async move {
            let weak_root = weak.clone();
            let request = ai::ChatCompletionsRequest {
                model,
                messages: vec![ai::ChatMessage {
                    role: "user".to_string(),
                    content: message,
                }],
                temperature: None,
                max_tokens: None,
                stream: Some(true),
            };

            let mut rx = match ai_service.send_chat_stream(request, 128).await {
                Ok(rx) => rx,
                Err(e) => {
                    let weak_err = weak_root.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(w) = weak_err.upgrade() {
                            let current = w.get_chat_output().to_string();
                            w.set_chat_output(format!("{current}Error: {e}\n").into());
                        }
                    });
                    return;
                }
            };

            let weak_assistant = weak_root.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(w) = weak_assistant.upgrade() {
                    let current = w.get_chat_output().to_string();
                    w.set_chat_output(format!("{current}Assistant: ").into());
                }
            });

            while let Some(item) = rx.recv().await {
                match item {
                    Ok(delta) => {
                        let weak2 = weak_root.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(w) = weak2.upgrade() {
                                let current = w.get_chat_output().to_string();
                                w.set_chat_output(format!("{current}{delta}").into());
                            }
                        });
                    }
                    Err(e) => {
                        let weak2 = weak_root.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(w) = weak2.upgrade() {
                                let current = w.get_chat_output().to_string();
                                w.set_chat_output(format!("{current}\nError: {e}\n").into());
                            }
                        });
                        return;
                    }
                }
            }

            let weak_done = weak_root.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(w) = weak_done.upgrade() {
                    let current = w.get_chat_output().to_string();
                    w.set_chat_output(format!("{current}\n\n").into());
                }
            });
        });
    });

    window.run()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppConfig {
    model: String,
}

fn default_model() -> String {
    "openai/gpt-4o-mini".to_string()
}

fn config_path() -> Option<PathBuf> {
    let dirs = ProjectDirs::from("dev", "text_editor", "ai_code_editor")?;
    Some(dirs.config_dir().join("config.json"))
}

fn load_config() -> AppConfig {
    let Some(path) = config_path() else {
        return AppConfig {
            model: default_model(),
        };
    };

    let data = std::fs::read_to_string(path);
    match data {
        Ok(s) => serde_json::from_str::<AppConfig>(&s)
            .ok()
            .unwrap_or(AppConfig {
                model: default_model(),
            }),
        Err(_) => AppConfig {
            model: default_model(),
        },
    }
}

fn save_config(cfg: &AppConfig) -> Result<(), String> {
    let path = config_path().ok_or("no config directory")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}
