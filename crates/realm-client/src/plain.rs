use std::io::{self, Write};

use anyhow::Result;
use realm_protocol::{OnlinePlayer, OutputStyle, PlayerSnapshot, ServerMessage};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;

use crate::app::{
    handle_user_input, reconnect_delay, reconnect_limit_reached, run_connection, should_reconnect,
    ClientState, WsEvent,
};

pub struct RoomView {
    pub title: String,
    pub description: String,
    pub exits: String,
    pub entities: Vec<String>,
    pub zone_art: Option<String>,
}

pub async fn run(server_url: &str) -> Result<()> {
    let mut state = ClientState::default();
    let mut stdin = BufReader::new(tokio::io::stdin());
    let mut line = String::new();

    loop {
        let (ws_tx, mut ws_rx) = mpsc::unbounded_channel::<WsEvent>();
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let url = server_url.to_string();
        let conn_state = state.clone();
        let conn_task = tokio::spawn(async move {
            let _ = run_connection(&url, &conn_state, ws_tx, cmd_rx).await;
        });

        let mut connected = false;

        'session: loop {
            tokio::select! {
                event = ws_rx.recv() => {
                    let Some(event) = event else { break 'session };
                    match event {
                        WsEvent::Connected => {
                            connected = true;
                            state.reconnect_attempts = 0;
                        }
                        WsEvent::Message(msg) => {
                            if let Some(prompt) = apply_message(&mut state, msg) {
                                print!("{prompt}");
                                io::stdout().flush().ok();
                            }
                        }
                        WsEvent::Error(err) => {
                            if !connected && state.reconnect_attempts == 0 && !state.authenticated {
                                eprintln!("Connection error: {err}");
                                eprintln!("Is the server running? Start it with: cargo run -p realm-server");
                                conn_task.abort();
                                return Ok(());
                            }
                        }
                        WsEvent::Disconnected => {
                            if state.intentional_disconnect {
                                println!("Farewell, adventurer!");
                                conn_task.abort();
                                return Ok(());
                            }
                            if should_reconnect(&state) {
                                state.reconnect_attempts += 1;
                                println!(
                                    "Connection lost. Reconnecting ({}/{})...",
                                    state.reconnect_attempts, 5
                                );
                                break 'session;
                            }
                            if reconnect_limit_reached(&state) || !state.authenticated {
                                println!("Disconnected from server.");
                                conn_task.abort();
                                return Ok(());
                            }
                        }
                    }
                }
                read = stdin.read_line(&mut line) => {
                    if read? > 0 {
                        let input = line.trim_end().to_string();
                        line.clear();
                        if let Some(msg) = handle_user_input(&mut state, &input) {
                            let _ = cmd_tx.send(msg);
                        } else if !state.authenticated && state.auth_step == crate::app::AuthStep::Mode {
                            // auth helper messages
                            match input.to_lowercase().as_str() {
                                "login" => println!("Enter your username."),
                                "register" => println!("Choose a username (3-16 chars)."),
                                _ if !input.is_empty() => eprintln!("Error: Type \"login\" or \"register\"."),
                                _ => {}
                            }
                        } else if state.auth_step == crate::app::AuthStep::Password && state.pending_password != "__register__" {
                            println!("Enter password.");
                        } else if state.auth_step == crate::app::AuthStep::Class {
                            if !input.is_empty() {
                                eprintln!("Error: Choose: warrior, mage, or rogue");
                            }
                            show_class_select();
                        }
                        print!("{}", state.prompt);
                        io::stdout().flush().ok();
                    } else {
                        conn_task.abort();
                        return Ok(());
                    }
                }
            }
        }

        conn_task.abort();
        reconnect_delay(state.reconnect_attempts).await;
    }
}

fn apply_message(state: &mut ClientState, msg: ServerMessage) -> Option<String> {
    match msg {
        ServerMessage::Banner => {
            show_banner();
            None
        }
        ServerMessage::Output { text, style } => {
            log(&text, style.unwrap_or(OutputStyle::Normal));
            None
        }
        ServerMessage::Room {
            title,
            description,
            exits,
            entities,
            zone_art,
            ..
        } => {
            show_room(&RoomView {
                title,
                description,
                exits,
                entities,
                zone_art,
            });
            None
        }
        ServerMessage::Stats { player } => {
            show_stats(&player);
            None
        }
        ServerMessage::Online { players } => {
            show_online(&players);
            None
        }
        ServerMessage::Flash { .. } => None,
        ServerMessage::Bell => {
            print!("\x07");
            io::stdout().flush().ok();
            None
        }
        ServerMessage::Ticker { text } => {
            println!("› {text}");
            None
        }
        ServerMessage::Motd { text } => {
            println!("\n=== MOTD ===\n{text}\n");
            None
        }
        ServerMessage::Error { text } => {
            eprintln!("Error: {text}");
            None
        }
        ServerMessage::Prompt { text } => {
            if state.authenticated {
                state.prompt = ">".into();
            } else {
                state.prompt = text.trim().to_string();
                if state.prompt.is_empty() {
                    state.prompt = "login or register?".into();
                }
            }
            Some(format!("{} ", state.prompt))
        }
        ServerMessage::Disconnect { reason } => {
            state.intentional_disconnect = true;
            println!("{reason}");
            None
        }
        ServerMessage::Combat { .. } => None,
    }
}

fn show_banner() {
    println!(
        r#"
╔══════════════════════════════════════════════════════════╗
║              R E A L M   O F   E C H O E S              ║
║         A Classic MMO Text Adventure                     ║
╚══════════════════════════════════════════════════════════╝
"#
    );
}

fn log(text: &str, style: OutputStyle) {
    let prefix = match style {
        OutputStyle::Combat => "⚔ ",
        OutputStyle::System => "» ",
        _ => "",
    };
    for line in text.lines() {
        println!("{prefix}{line}");
    }
}

fn show_room(room: &RoomView) {
    println!();
    if let Some(art) = &room.zone_art {
        println!("{art}");
    }
    println!("[ {} ]", room.title);
    println!("{}", room.description);
    if !room.exits.is_empty() {
        println!("Exits: {}", room.exits);
    }
    if !room.entities.is_empty() {
        println!("Also here:");
        for entity in &room.entities {
            println!("  • {entity}");
        }
    }
    println!();
}

fn show_stats(player: &PlayerSnapshot) {
    let duel = if player.in_duel.unwrap_or(false) { " [DUEL]" } else { "" };
    println!(
        " Lv.{} HP {}/{} MP {}/{} {}g{}",
        player.level, player.hp, player.max_hp, player.mp, player.max_mp, player.gold, duel
    );
}

fn show_online(players: &[OnlinePlayer]) {
    println!("-- {} online --", players.len());
    for p in players {
        println!(
            "  {} Lv.{} {} @ {}",
            p.username,
            p.level,
            p.class_name.as_str(),
            p.zone
        );
    }
}

fn show_class_select() {
    println!("\nChoose class: warrior | mage | rogue\n");
}