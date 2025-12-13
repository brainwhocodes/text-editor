use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

slint::include_modules!();

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
