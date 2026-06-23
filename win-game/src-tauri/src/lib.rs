use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{Manager, State};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::broadcast;

struct AppState {
    outputs: Mutex<HashMap<String, String>>,
    scores: Mutex<Vec<ScoreEntry>>,
    round_active: Mutex<bool>,
    last_tickets: Mutex<i32>,
    high_score: Mutex<i32>,
    tx: broadcast::Sender<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ScoreEntry {
    initials: String,
    score: i32,
    tickets: i32,
    date: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct OutputsSnapshot {
    ticket_counter: i32,
    ticket_jackpot: i32,
    coin1: i32,
    coin2: i32,
    high_score: i32,
    lamps: HashMap<String, bool>,
}

#[tauri::command]
fn get_outputs(state: State<AppState>) -> OutputsSnapshot {
    let outputs = state.outputs.lock().unwrap();
    let ticket_counter = outputs
        .get("TicketCounter")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let ticket_jackpot = outputs
        .get("TicketJackpot")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let coin1 = outputs
        .get("Coin1")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let coin2 = outputs
        .get("Coin2")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let high_score = outputs
        .get("HighScore")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let mut lamps = HashMap::new();
    for lamp in [
        "LampStart",
        "LampLeader",
        "LampRed",
        "LampGreen",
        "LampBlue",
        "Billboard Red",
        "Billboard Green",
        "Billboard Blue",
        "SideLEDRed",
        "SideLEDGreen",
        "SideLEDBlue",
        "WooferLEDRed",
        "WooferLEDGreen",
        "WooferLEDBlue",
        "ItemLEDRed",
        "ItemLEDGreen",
        "ItemLEDBlue",
    ] {
        lamps.insert(
            lamp.to_string(),
            outputs.get(lamp).map(|v| v == "1").unwrap_or(false),
        );
    }

    OutputsSnapshot {
        ticket_counter,
        ticket_jackpot,
        coin1,
        coin2,
        high_score,
        lamps,
    }
}

#[tauri::command]
fn get_scores(state: State<AppState>) -> Vec<ScoreEntry> {
    let scores = state.scores.lock().unwrap();
    let mut sorted = scores.clone();
    sorted.sort_by(|a, b| b.score.cmp(&a.score));
    sorted.truncate(10);
    sorted
}

#[tauri::command]
fn submit_score(
    state: State<AppState>,
    initials: String,
    score: i32,
    tickets: i32,
) -> Vec<ScoreEntry> {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    let entry = ScoreEntry {
        initials,
        score,
        tickets,
        date: now,
    };
    let mut scores = state.scores.lock().unwrap();
    scores.push(entry);
    let mut sorted = scores.clone();
    sorted.sort_by(|a, b| b.score.cmp(&a.score));
    sorted.truncate(10);
    *scores = sorted.clone();
    save_scores(&scores);
    sorted
}

fn save_scores(scores: &[ScoreEntry]) {
    if let Ok(data) = serde_json::to_string(scores) {
        let _ = std::fs::write("scores.json", data);
    }
}

fn load_scores() -> Vec<ScoreEntry> {
    if let Ok(data) = std::fs::read_to_string("scores.json") {
        if let Ok(scores) = serde_json::from_str(&data) {
            return scores;
        }
    }
    Vec::new()
}

#[tauri::command]
fn round_ended(state: State<AppState>) -> Option<(i32, i32)> {
    let mut round = state.round_active.lock().unwrap();
    if *round {
        let outputs = state.outputs.lock().unwrap();
        let tickets = outputs
            .get("TicketCounter")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let score = state.high_score.lock().unwrap().clone();
        *round = false;
        if tickets > 0 || score > 0 {
            return Some((score, tickets));
        }
    }
    None
}

async fn tcp_client_loop(state: tauri::AppHandle) {
    loop {
        match TcpStream::connect("127.0.0.1:8000").await {
            Ok(stream) => {
                println!("Connected to OutputBlaster at 127.0.0.1:8000");
                let reader = BufReader::new(stream);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let line = line.trim().to_string();
                    if line.is_empty() {
                        continue;
                    }
                    if let Some((signal, value)) = line.split_once(" = ") {
                        let signal = signal.trim().to_string();
                        let value = value.trim().to_string();
                        let state: State<AppState> = state.state();
                        let mut outputs = state.outputs.lock().unwrap();
                        outputs.insert(signal.clone(), value.clone());

                        if signal == "TicketCounter" {
                            let val: i32 = value.parse().unwrap_or(0);
                            let mut last = state.last_tickets.lock().unwrap();
                            let mut round = state.round_active.lock().unwrap();
                            if val > 0 && *last == 0 {
                                *round = true;
                                println!("Round started");
                            } else if val == 0 && *last > 0 && *round {
                                *round = false;
                                println!("Round ended! Tickets: {}", *last);
                                let _ = state.tx.send("round_ended".to_string());
                            }
                            *last = val;
                        }
                        if signal == "HighScore" {
                            let val: i32 = value.parse().unwrap_or(0);
                            let mut hs = state.high_score.lock().unwrap();
                            *hs = val;
                        }
                    }
                }
                println!("Disconnected from OutputBlaster");
            }
            Err(e) => {
                println!("Failed to connect to OutputBlaster: {}. Retrying in 3s...", e);
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let scores = load_scores();
    let (tx, _rx) = broadcast::channel::<String>(100);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            outputs: Mutex::new(HashMap::new()),
            scores: Mutex::new(scores),
            round_active: Mutex::new(false),
            last_tickets: Mutex::new(0),
            high_score: Mutex::new(0),
            tx: tx.clone(),
        })
        .invoke_handler(tauri::generate_handler![
            get_outputs,
            get_scores,
            submit_score,
            round_ended,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tcp_client_loop(handle).await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
