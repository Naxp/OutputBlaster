# Changelog

> **Last updated:** 2026-06-24

---

## 2026-06-23 — Task 0001: Update Agents Governance Standard

- **Summary:** Created complete repo governance structure: `Agents.md`, `Repo_map.md`, `Audits_index.md`, `changelog.md`, `audits/`, `tasks/`, `tasks/TASK_INDEX.md`, and `tasks/0001-update-agents-governance-standard.md`.
- **Files changed:**
  - `Agents.md` (new)
  - `changelog.md` (new)
  - `Audits_index.md` (new)
  - `Repo_map.md` (new)
  - `audits/2026-06-23-0001-agents-governance-standard-audit.md` (new)
  - `tasks/TASK_INDEX.md` (new)
  - `tasks/0001-update-agents-governance-standard.md` (new)
- **Reason for change:** Establish repo-standard governance workflow with mandatory pre-read, pass types, task tracking, audit tracking, repo mapping, changelog, Git behavior, and end-of-pass summary requirements.

---

## 2026-06-23 — Task 0002: Create Master Reference Map

- **Summary:** Created comprehensive `docs/MASTER_MAP.md` documenting all 215 EOutputs, full CRC table (62 games), Sonic Dash Extreme memory map, step-by-step game-add guide, TeknoParrot integration, OutputHooker architecture, build commands, and debug tools. Updated `Repo_map.md` and `AGENTS.md` with project specifics.
- **Files changed:**
  - `docs/MASTER_MAP.md` (new)
  - `Repo_map.md` (updated)
  - `AGENTS.md` (updated)
  - `audits/2026-06-23-0002-create-master-reference-map-audit.md` (new)
- **Reason for change:** Single authoritative reference for all project knowledge; eliminates tribal knowledge and reduces drift across AI/dev passes.

---

## 2026-06-24 — Task 0003: WinGame TCP Simulator and Pipeline Test

- **Summary:** Created `win-game/simulate.py` — standalone TCP server simulating OutputBlaster NetOutputs protocol (port 8000, `Name = Value\r\n`) with attract→game→boss→payout→reset cycle. Fixed `tokio::spawn` → `tauri::async_runtime::spawn` crash in WinGame setup hook. Created `win-game/.gitignore`. Verified full pipeline: simulator + WinGame connect, render arcade UI with lamps/LEDs/scores/ticket animations.
- **UX overhaul:** WinGame now starts silently with dimmed cabinet, "Waiting" status, no error messages. No longer requires game/OutputBlaster to be running. Dynamic game name display via `mame_start` signal. Added `connected` flag + `get_status` Tauri command. Replaced hardcoded marquee with dynamic `#gameName` element. Added `.right-panel.waiting` CSS for dimmed idle state. Removed `println!` error messages from TCP client.
- **Debug overlay:** In-memory log ring buffer (512 entries) with `get_logs` Tauri command. Debug panel toggled with F12 showing real-time log output. `println!`/`eprintln!` console logging for terminal visibility.
- **Bug fixes:** Fixed `initials` borrow-after-move in `submit_score`. Ensured dist/ is always rebuilt before Rust build.
- **Build system fix:** `build.rs` now auto-detects missing `dist/` and runs `npm run build` before compilation. Previously, `cargo build --release` without `npm run build` first would compile successfully but produce a binary with no embedded frontend → WebView showed "ERR_CONNECTION_REFUSED". Tested end-to-end: removed dist/, ran `cargo build --release` → build.rs auto-built frontend → binary works.
- **Files changed:**
  - `win-game/simulate.py` (new)
  - `win-game/.gitignore` (new)
  - `win-game/src-tauri/src/lib.rs` (edited: tokio spawn fix, connected flag, mame_start parsing, get_status, get_logs, in-memory log buffer, console logging, score submit fix)
  - `win-game/src/main.js` (rewritten: get_status usage, clean waiting state, no error messages, debug overlay polling, F12 toggle)
  - `win-game/index.html` (edited: dynamic game name span, debug overlay HTML)
  - `win-game/public/styles.css` (edited: waiting state styles, subtle connection status, debug overlay styles)
  - `win-game/src-tauri/build.rs` (rewritten: auto-build frontend if dist/ missing)
  - `audits/2026-06-24-0003-win-game-tcp-simulator-audit.md` (new)
  - `tasks/0003-win-game-tcp-simulator-and-test.md` (new)
  - `tasks/TASK_INDEX.md` (updated)
  - `Audits_index.md` (updated)
  - `changelog.md` (updated)
- **Reason for change:** Enable testing of WinGame arcade display app without a physical game; fix runtime crash blocking WinGame launch; document the TCP protocol for future reference. WinGame must never require hardware — it starts silently and waits for any game to begin sending data via TCP.
