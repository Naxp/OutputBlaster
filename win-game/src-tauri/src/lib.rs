use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{Manager, State};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::broadcast;

const MAX_LOG_ENTRIES: usize = 512;

struct AppState {
    outputs: Mutex<HashMap<String, String>>,
    scores: Mutex<Vec<ScoreEntry>>,
    round_active: Mutex<bool>,
    last_tickets: Mutex<i32>,
    last_coin1: Mutex<i32>,
    coins_inserted: Mutex<bool>,
    high_score: Mutex<i32>,
    connected: Mutex<bool>,
    game_name: Mutex<String>,
    player_initials: Mutex<String>,
    logs: Mutex<Vec<String>>,
    tx: broadcast::Sender<String>,
}

fn log(state: &AppState, msg: String) {
    let mut logs = state.logs.lock().unwrap();
    logs.push(msg);
    if logs.len() > MAX_LOG_ENTRIES {
        logs.remove(0);
    }
}

fn log_info(state: &AppState, msg: &str) {
    println!("[WinGame] {}", msg);
    log(state, format!("[INFO] {}", msg));
}

fn log_err(state: &AppState, msg: &str) {
    eprintln!("[WinGame] ERROR: {}", msg);
    log(state, format!("[ERROR] {}", msg));
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
    rings: i32,
    lamps: HashMap<String, bool>,
    raw: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ConnectionStatus {
    connected: bool,
    game_name: String,
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
    let rings = outputs
        .get("Rings")
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
        rings,
        lamps,
        raw: outputs.clone(),
    }
}

#[tauri::command]
fn get_status(state: State<AppState>) -> ConnectionStatus {
    let connected = *state.connected.lock().unwrap();
    let game_name = state.game_name.lock().unwrap().clone();
    ConnectionStatus { connected, game_name }
}

#[tauri::command]
fn get_logs(state: State<AppState>) -> Vec<String> {
    let logs = state.logs.lock().unwrap();
    logs.clone()
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
        initials: initials.clone(),
        score,
        tickets,
        date: now,
    };
    let mut scores = state.scores.lock().unwrap();
    scores.push(entry);
    log_info(&state, &format!("Score submitted: {} {} {}", initials, score, tickets));
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

#[tauri::command]
fn get_initials(state: State<AppState>) -> String {
    state.player_initials.lock().unwrap().clone()
}

#[tauri::command]
fn set_initials(state: State<AppState>, initials: String) {
    let mut pi = state.player_initials.lock().unwrap();
    let trimmed: String = initials.chars().take(3).collect();
    *pi = if trimmed.len() < 3 { format!("{: <3}", trimmed) } else { trimmed };
    log_info(&state, &format!("Player initials set to: {}", *pi));
}

#[tauri::command]
fn close_app() {
    std::process::exit(0);
}

#[tauri::command]
fn simulate(state: State<AppState>) -> ConnectionStatus {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let mut outputs = state.outputs.lock().unwrap();
    outputs.insert("TicketCounter".into(), ((t % 500) + 10).to_string());
    outputs.insert("TicketJackpot".into(), ((t % 10000) + 1000).to_string());
    outputs.insert("Coin1".into(), ((t % 50) + 1).to_string());
    outputs.insert("HighScore".into(), ((t % 999999) + 100000).to_string());
    outputs.insert("Rings".into(), ((t % 300) + 5).to_string());
    outputs.insert("LampStart".into(), if t % 3 == 0 { "1".into() } else { "0".into() });
    outputs.insert("LampLeader".into(), if t % 5 == 0 { "1".into() } else { "0".into() });
    outputs.insert("LampRed".into(), "1".into());
    outputs.insert("LampGreen".into(), if t % 2 == 0 { "1".into() } else { "0".into() });
    outputs.insert("LampBlue".into(), if t % 3 == 0 { "1".into() } else { "0".into() });
    outputs.insert("Billboard Red".into(), if t % 4 == 0 { "1".into() } else { "0".into() });
    outputs.insert("Billboard Green".into(), "1".into());
    outputs.insert("Billboard Blue".into(), if t % 2 == 0 { "1".into() } else { "0".into() });
    outputs.insert("SideLEDRed".into(), "1".into());
    outputs.insert("SideLEDGreen".into(), if t % 3 != 0 { "1".into() } else { "0".into() });
    outputs.insert("SideLEDBlue".into(), if t % 7 == 0 { "1".into() } else { "0".into() });
    outputs.insert("WooferLEDRed".into(), if t % 5 == 0 { "1".into() } else { "0".into() });
    outputs.insert("WooferLEDGreen".into(), if t % 2 == 0 { "1".into() } else { "0".into() });
    outputs.insert("WooferLEDBlue".into(), "1".into());
    outputs.insert("ItemLEDRed".into(), "1".into());
    outputs.insert("ItemLEDGreen".into(), if t % 4 == 0 { "1".into() } else { "0".into() });
    outputs.insert("ItemLEDBlue".into(), if t % 7 == 0 { "1".into() } else { "0".into() });

    *state.connected.lock().unwrap() = true;
    *state.game_name.lock().unwrap() = "Sonic Dash Extreme [SIMULATED]".into();
    *state.last_tickets.lock().unwrap() = ((t % 500) + 10) as i32;
    *state.high_score.lock().unwrap() = ((t % 999999) + 100000) as i32;

    log_info(&state, "Simulated game data injected");
    ConnectionStatus { connected: true, game_name: "Sonic Dash Extreme [SIMULATED]".into() }
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
        match TcpStream::connect("127.0.0.1:37520").await {
            Ok(stream) => {
                let st: State<AppState> = state.state();
                *st.connected.lock().unwrap() = true;
                log_info(&st, "TCP connected to OutputBlaster on port 37520");
                drop(st);
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
                        let st: State<AppState> = state.state();
                        let mut outputs = st.outputs.lock().unwrap();
                        outputs.insert(signal.clone(), value.clone());

                        if signal == "mame_start" {
                            log_info(&st, &format!("Game detected: {}", value));
                            *st.game_name.lock().unwrap() = value.clone();
                        }
                        if signal == "TicketCounter" {
                            let val: i32 = value.parse().unwrap_or(0);
                            let mut last = st.last_tickets.lock().unwrap();
                            let mut round = st.round_active.lock().unwrap();
                            let coins = *st.coins_inserted.lock().unwrap();
                            if val > 0 && *last == 0 && coins {
                                *round = true;
                                *st.coins_inserted.lock().unwrap() = false;
                                log_info(&st, "Round started (coins inserted)");
                            } else if val == 0 && *last > 0 && *round {
                                *round = false;
                                log_info(&st, &format!("Round ended! Tickets: {}", *last));
                                let _ = st.tx.send("round_ended".to_string());
                            }
                            *last = val;
                        }
                        if signal == "HighScore" {
                            let val: i32 = value.parse().unwrap_or(0);
                            let mut hs = st.high_score.lock().unwrap();
                            *hs = val;
                        }
                        if signal == "Coin1" {
                            let val: i32 = value.parse().unwrap_or(0);
                            let mut last = st.last_coin1.lock().unwrap();
                            if val > *last {
                                *st.coins_inserted.lock().unwrap() = true;
                                log_info(&st, &format!("Coin inserted ({} -> {})", *last, val));
                            }
                            *last = val;
                        }
                    }
                }
                let st: State<AppState> = state.state();
                *st.connected.lock().unwrap() = false;
                log_info(&st, "TCP disconnected from OutputBlaster");
            }
            Err(e) => {
                let st: State<AppState> = state.state();
                log_err(&st, &format!("TCP connect failed: {} — retrying in 3s", e));
                drop(st);
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            }
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let scores = load_scores();
    let (tx, _rx) = broadcast::channel::<String>(100);
    let html_bytes: Vec<u8> = HTML_BYTES.to_vec();

    println!("[WinGame] Starting Arcade Output Display v0.1.0");

    tauri::Builder::default()
        .register_uri_scheme_protocol("wingame", move |_app, _req| {
            tauri::http::Response::builder()
                .status(200)
                .header("Content-Type", "text/html")
                .body(html_bytes.clone())
                .unwrap()
        })
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            outputs: Mutex::new(HashMap::new()),
            scores: Mutex::new(scores),
            round_active: Mutex::new(false),
            last_tickets: Mutex::new(0),
            last_coin1: Mutex::new(0),
            coins_inserted: Mutex::new(false),
            high_score: Mutex::new(0),
            connected: Mutex::new(false),
            game_name: Mutex::new(String::new()),
            player_initials: Mutex::new("---".to_string()),
            logs: Mutex::new(Vec::new()),
            tx: tx.clone(),
        })
        .invoke_handler(tauri::generate_handler![
            get_outputs,
            get_status,
            get_logs,
            get_scores,
            submit_score,
            round_ended,
            close_app,
            simulate,
            get_initials,
            set_initials,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            println!("[WinGame] Setup complete — spawning TCP client loop");
            tauri::async_runtime::spawn(async move {
                tcp_client_loop(handle).await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
