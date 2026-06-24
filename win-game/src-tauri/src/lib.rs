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
    high_score: Mutex<i32>,
    connected: Mutex<bool>,
    game_name: Mutex<String>,
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
        match TcpStream::connect("127.0.0.1:8000").await {
            Ok(stream) => {
                let st: State<AppState> = state.state();
                *st.connected.lock().unwrap() = true;
                log_info(&st, "TCP connected to OutputBlaster on port 8000");
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
                            if val > 0 && *last == 0 {
                                *round = true;
                                log_info(&st, "Round started (TicketCounter 0 -> positive)");
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

fn write_html() -> Option<String> {
    let path = std::env::current_exe().ok()?
        .parent()?
        .join("wingame.html");
    match std::fs::write(&path, HTML) {
        Ok(_) => {
            println!("[WinGame] HTML written to: {}", path.display());
            Some(path.to_string_lossy().to_string())
        }
        Err(e) => {
            eprintln!("[WinGame] ERROR: Failed to write HTML: {}", e);
            None
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let scores = load_scores();
    let (tx, _rx) = broadcast::channel::<String>(100);

    println!("[WinGame] Starting Arcade Output Display v0.1.0");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            outputs: Mutex::new(HashMap::new()),
            scores: Mutex::new(scores),
            round_active: Mutex::new(false),
            last_tickets: Mutex::new(0),
            high_score: Mutex::new(0),
            connected: Mutex::new(false),
            game_name: Mutex::new(String::new()),
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
        ])
        .setup(|app| {
            if let Some(path) = write_html() {
                if let Some(window) = app.get_webview_window("main") {
                    let url = format!("file:///{}", path.replace('\\', "/"));
                    println!("[WinGame] Setting window URL to: {}", url);
                    let _ = window.navigate(tauri::Url::parse(&url).unwrap());
                }
            }
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
