# Task 0003: WinGame TCP Simulator and Pipeline Test

**Date:** 2026-06-24  
**Status:** Completed

## Goal
Create a standalone TCP simulator to test the WinGame arcade display app without needing a physical game running, verify the full NetOutputs pipeline works end-to-end.

## Scope
- `win-game/` subdirectory (Tauri app)
- TCP protocol matching OutputBlaster's NetOutputs (port 8000, `Name = Value\r\n`)
- Fix tokio runtime crash in WinGame setup hook

## Checklist
- [x] Read NetOutputs.cpp/.h to understand protocol format
- [x] Read WinGame lib.rs to understand expected signal names and round-detection logic
- [x] Create `win-game/simulate.py` — TCP server with attract→game→boss→payout→reset cycle
- [x] Fix `tokio::spawn` → `tauri::async_runtime::spawn` in lib.rs:235
- [x] Create `win-game/.gitignore` (exclude node_modules, dist, target/)
- [x] Build WinGame release with fix (cargo build --release)
- [x] Test: start simulate.py → launch win-game.exe → verify connection and UI

## Files Reviewed
- `Output Files/NetOutputs.cpp` — TCP protocol format (`SeparatorIdAndValue = " = "`, `FrameEnding = "\r" / "\r\n"`)
- `Output Files/NetOutputs.h` — class definition
- `win-game/src-tauri/src/lib.rs` — Rust TCP client, command handlers
- `win-game/src/main.js` — frontend poll loop, lamp/FW/ticket display
- `Output Files/Outputs.cpp` — output name strings (`s_outputNames[]`)
- `Game Files/SonicDashExtreme.cpp` — actual game outputs for realistic simulation

## Files Changed
| File | Action |
|------|--------|
| `win-game/.gitignore` | Created |
| `win-game/simulate.py` | Created |
| `win-game/src-tauri/src/lib.rs` | Edited (fixed tokio runtime crash) |

## Audit Reference
- `audits/2026-06-24-0003-win-game-tcp-simulator-audit.md`

## Changelog Reference
- `changelog.md` — 2026-06-24, Task 0003

## Completion Notes
- Simulator sends all 22 Sonic Dash Extreme signals with `\r\n` frame endings
- WinGame connects, parses output values, renders lamps/LEDs/ticket counter/scoreboard
- Test verified: WinGame window visible ("Arcade Output Display"), no crash, 26 MB working set
