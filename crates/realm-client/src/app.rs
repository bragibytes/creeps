use std::time::Duration;

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use realm_core::types::HOTKEY_COMMANDS;
use realm_protocol::{ClassName, ClientMessage, ServerMessage};
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message};

const MAX_RECONNECT: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthStep {
    Mode,
    Username,
    Password,
    Class,
}

#[derive(Clone)]
pub struct ClientState {
    pub authenticated: bool,
    pub auth_step: AuthStep,
    pub pending_username: String,
    pub pending_password: String,
    pub saved_username: String,
    pub saved_password: String,
    pub reconnect_attempts: u32,
    pub intentional_disconnect: bool,
    pub prompt: String,
    pub hidden_input: bool,
}

impl Default for ClientState {
    fn default() -> Self {
        Self {
            authenticated: false,
            auth_step: AuthStep::Mode,
            pending_username: String::new(),
            pending_password: String::new(),
            saved_username: String::new(),
            saved_password: String::new(),
            reconnect_attempts: 0,
            intentional_disconnect: false,
            prompt: "login or register?".into(),
            hidden_input: false,
        }
    }
}

pub fn expand_input(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let lower = trimmed.to_lowercase();
    if !lower.contains(' ') {
        if let Some(cmd) = HOTKEY_COMMANDS.get(&lower) {
            return cmd.clone();
        }
    }
    trimmed.to_string()
}

pub fn handle_auth_input(state: &mut ClientState, input: &str) -> Option<ClientMessage> {
    let lower = input.to_lowercase();

    match state.auth_step {
        AuthStep::Mode => {
            if lower == "login" {
                state.auth_step = AuthStep::Username;
                state.prompt = "username:".into();
                state.hidden_input = false;
                return None;
            }
            if lower == "register" {
                state.auth_step = AuthStep::Username;
                state.pending_password = "__register__".into();
                state.prompt = "username:".into();
                state.hidden_input = false;
                return None;
            }
            state.prompt = "login or register?".into();
            return None;
        }
        AuthStep::Username => {
            state.pending_username = input.to_string();
            state.auth_step = AuthStep::Password;
            state.prompt = "password:".into();
            state.hidden_input = true;
            return None;
        }
        AuthStep::Password => {
            if state.pending_password == "__register__" {
                state.pending_password = input.to_string();
                state.auth_step = AuthStep::Class;
                state.prompt = "class:".into();
                state.hidden_input = false;
                return None;
            }
            state.saved_username = state.pending_username.clone();
            state.saved_password = input.to_string();
            state.authenticated = true;
            state.auth_step = AuthStep::Mode;
            state.prompt = ">".into();
            state.hidden_input = false;
            return Some(ClientMessage::Login {
                username: state.pending_username.clone(),
                password: input.to_string(),
            });
        }
        AuthStep::Class => {
            let class_name = match lower.as_str() {
                "warrior" => ClassName::Warrior,
                "mage" => ClassName::Mage,
                "rogue" => ClassName::Rogue,
                _ => {
                    state.prompt = "class:".into();
                    return None;
                }
            };
            state.saved_username = state.pending_username.clone();
            state.saved_password = state.pending_password.clone();
            state.authenticated = true;
            state.auth_step = AuthStep::Mode;
            state.prompt = ">".into();
            state.hidden_input = false;
            return Some(ClientMessage::Register {
                username: state.pending_username.clone(),
                password: state.pending_password.clone(),
                class_name,
            });
        }
    }
}

pub fn handle_user_input(state: &mut ClientState, raw: &str) -> Option<ClientMessage> {
    let input = expand_input(raw);
    if input.is_empty() {
        if state.authenticated {
            state.prompt = ">".into();
        }
        return None;
    }

    let lower = input.to_lowercase();
    if lower == "quit" || lower == "exit" {
        if state.authenticated {
            state.intentional_disconnect = true;
            return Some(ClientMessage::Command { input });
        }
        return None;
    }

    if !state.authenticated {
        return handle_auth_input(state, &input);
    }

    Some(ClientMessage::Command { input })
}

pub enum WsEvent {
    Connected,
    Message(ServerMessage),
    Disconnected,
    Error(String),
}

pub async fn run_connection(
    server_url: &str,
    state: &ClientState,
    out_tx: mpsc::UnboundedSender<WsEvent>,
    mut in_rx: mpsc::UnboundedReceiver<ClientMessage>,
) -> Result<()> {
    let (ws, _) = connect_async(server_url)
        .await
        .with_context(|| format!("connect to {server_url}"))?;

    let (mut write, mut read) = ws.split();
    let _ = out_tx.send(WsEvent::Connected);

    if state.authenticated && !state.saved_username.is_empty() && !state.saved_password.is_empty() {
        let msg = ClientMessage::Login {
            username: state.saved_username.clone(),
            password: state.saved_password.clone(),
        };
        let json = serde_json::to_string(&msg)?;
        write.send(Message::Text(json)).await?;
    }

    loop {
        tokio::select! {
            incoming = read.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ServerMessage>(&text) {
                            Ok(msg) => {
                                if matches!(msg, ServerMessage::Prompt { .. }) && state.authenticated {
                                    // prompt handled by UI
                                }
                                let _ = out_tx.send(WsEvent::Message(msg));
                            }
                            Err(_) => {
                                let _ = out_tx.send(WsEvent::Error("Invalid server message.".into()));
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        let _ = out_tx.send(WsEvent::Disconnected);
                        break;
                    }
                    Some(Err(err)) => {
                        let _ = out_tx.send(WsEvent::Error(err.to_string()));
                        let _ = out_tx.send(WsEvent::Disconnected);
                        break;
                    }
                    _ => {}
                }
            }
            outgoing = in_rx.recv() => {
                match outgoing {
                    Some(msg) => {
                        let json = serde_json::to_string(&msg)?;
                        if write.send(Message::Text(json)).await.is_err() {
                            let _ = out_tx.send(WsEvent::Disconnected);
                            break;
                        }
                    }
                    None => break,
                }
            }
        }
    }

    Ok(())
}

pub async fn reconnect_delay(attempt: u32) {
    if attempt > 0 {
        sleep(Duration::from_secs(2)).await;
    }
}

pub fn should_reconnect(state: &ClientState) -> bool {
    state.authenticated
        && !state.intentional_disconnect
        && state.reconnect_attempts < MAX_RECONNECT
}

pub fn reconnect_limit_reached(state: &ClientState) -> bool {
    state.authenticated
        && !state.intentional_disconnect
        && state.reconnect_attempts >= MAX_RECONNECT
}