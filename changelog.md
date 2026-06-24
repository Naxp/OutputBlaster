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
- **Self-contained binary:** Eliminated Tauri's embedded asset system entirely. build.rs now reads `dist/index.html`, `dist/styles.css`, and the Vite JS bundle at compile time, inlines the CSS and JS into the HTML, and generates `$OUT_DIR/generated.rs` containing the combined HTML as a Rust constant. `lib.rs` includes this via `include!()`. At runtime, the setup hook writes the HTML to disk next to the exe and navigates the window to `file://` URL. Window config uses `"url": "about:blank"` to prevent any initial load attempt. No dependency on Tauri's `frontendDist` asset serving. No localhost connection needed — the app is 100% self-contained in a single binary.
- **Files changed:**
  - `win-game/simulate.py` (new)
  - `win-game/.gitignore` (new)
  - `win-game/src-tauri/src/lib.rs` (edited: tokio spawn fix, connected flag, mame_start parsing, get_status, get_logs, in-memory log buffer, console logging, score submit fix)
  - `win-game/src/main.js` (rewritten: get_status usage, clean waiting state, no error messages, debug overlay polling, F12 toggle)
  - `win-game/index.html` (edited: dynamic game name span, debug overlay HTML)
  - `win-game/public/styles.css` (edited: waiting state styles, subtle connection status, debug overlay styles)
  - `win-game/src-tauri/build.rs` (rewritten: generate embedded HTML Rust source)
  - `win-game/src-tauri/src/lib.rs` (edited: include! generated HTML, write to disk, navigate file:// URL)
  - `win-game/src-tauri/tauri.conf.json` (edited: added url: about:blank to prevent initial load)
  - `audits/2026-06-24-0003-win-game-tcp-simulator-audit.md` (new)
  - `tasks/0003-win-game-tcp-simulator-and-test.md` (new)
  - `tasks/TASK_INDEX.md` (updated)
  - `Audits_index.md` (updated)
  - `changelog.md` (updated)
- **Reason for change:** Enable testing of WinGame arcade display app without a physical game; fix runtime crash blocking WinGame launch; document the TCP protocol for future reference. WinGame must never require hardware — it starts silently and waits for any game to begin sending data via TCP.

### 2026-06-24 — Pass 0004: WinGame drag/simulate + GameProfiles XML fix

- **Task reference:** 0004
- **Summary:** Make WinGame window draggable (data-tauri-drag-region), add close button, add Sim Data button for offline testing, fix GameProfiles XML to include Enable Outputs field so TeknoParrot injects OB DLL
- **Files changed:**
  - `win-game/index.html` (added close button, simulate button)
  - `win-game/public/styles.css` (added drag cursor, close-btn, sim-btn styles)
  - `win-game/src/main.js` (added close_app, simulate invoke handlers)
  - `win-game/src-tauri/src/lib.rs` (added close_app + simulate Tauri commands)
  - `C:\Users\robon\Desktop\TPBootstrapper\GameProfiles\SonicDashExtreme.xml` (added Enable Outputs field)
- **Reason for change:** Window was immovable (decorations: false, no drag-region); no way to test UI without live game; OB DLL not injected because GameProfiles template lacked Enable Outputs field

### 2026-06-24 — Pass 0005: Add Rings output to Sonic Dash Extreme

- **Task reference:** 0005
- **Summary:** Add real-time ring count via 6-level pointer chain from Cheat Engine; add OutputRings to enum; deploy updated DLL; add rings display to WinGame; fix VS2026 build toolset (v145)
- **Files changed:**
  - `Output Files/Outputs.h` (added OutputRings before NUM_OUTPUTS)
  - `Output Files/Outputs.cpp` (added "Rings" to s_outputNames)
  - `Game Files/SonicDashExtreme.cpp` (added pointer chain + ring count output)
  - `win-game/src-tauri/src/lib.rs` (added rings to OutputsSnapshot + get_outputs)
  - `win-game/index.html` (added rings info box)
  - `win-game/src/main.js` (added rings display update)
  - `bin/x86/Release/OutputBlaster.dll` (rebuilt + deployed)
- **Reason for change:** User identified ring count pointer chain in Cheat Engine; need real-time ring display in WinGame; OB DLL needed rebuild for new output

### 2026-06-24 — Pass 0006: Broadcast Outputs, WinGame UI Redesign & Port Change

- **Task reference:** 0006
- **Summary:** Created `CBroadcastOutputs` wrapper allowing both WinOutputs (OutputHooker) and NetOutputs (WinGame TCP) to run simultaneously. Changed default ports from 8000/8001 to 37520/37521 to avoid conflicts. Redesigned WinGame LED layout: Billboard triangle with per-color R/G/B, Woofer speaker shapes (2 small + 1 large), Side LEDs left/right columns, Item LEDs, Misc auto-detect box for unknown outputs. Fixed WinGame build.rs to always rebuild frontend (was caching stale HTML). Added persistent player initials with get/set Tauri commands and "Change Initials" button. Added raw outputs HashMap to OutputsSnapshot for Misc box detection.
- **Files changed:**
  - `Output Files/BroadcastOutputs.h` (new)
  - `Output Files/BroadcastOutputs.cpp` (new)
  - `Common Files/Game.cpp` (modified: includes broadcast, dual-backend CreateOutputsFromConfig)
  - `win-game/index.html` (rewritten: new LED layout, Change Initials button, coins-exhausted indicator)
  - `win-game/src/main.js` (rewritten: per-color LED rendering, triangle coloring, woofer glow, Misc box, round-end flow)
  - `win-game/src/styles.css` (rewritten: new grid layout, billboard triangle CSS, woofer circles, color-led dots, misc box)
  - `win-game/src-tauri/src/lib.rs` (modified: raw HashMap in OutputsSnapshot, get_initials/set_initials, player_initials state)
  - `win-game/src-tauri/build.rs` (modified: always build frontend, rerun-if-changed for source files)
  - `win-game/simulate.py` (port changed to 37520)
  - `docs/MASTER_MAP.md` (updated: new cheat sheet, broadcast architecture, port guide, complete system diagram)
  - `changelog.md`, `Repo_map.md`, `Audits_index.md`, `tasks/TASK_INDEX.md` (updated)
  - `audits/2026-06-24-0006-broadcast-outputs-wingame-ui-redesign.md` (new)
  - `tasks/0006-broadcast-outputs-wingame-ui-redesign.md` (new)
- **Reason for change:** Enable OutputHooker + WinGame simultaneously. Fix port conflicts. Improve LED visualization to match physical hardware layout. Fix stale frontend builds in WinGame.

### 2026-06-24 — Task 0007: Frogger + Ghostbusters Support, WinGame Modular Stats & Button Lighting
- **Summary:** Added Frogger and Ghostbusters game handlers. Made WinGame stat boxes modular per game. Added coin/start button lighting. Removed _DEBUG guard on CRC output.
- **Files changed:**
  - `Game Files/Frogger.h` (new)
  - `Game Files/Frogger.cpp` (new)
  - `Game Files/Ghostbusters.h` (new)
  - `Game Files/Ghostbusters.cpp` (new)
  - `dllmain.cpp` (modified: adds Frogger/Ghostbusters includes + CRC cases; removed _DEBUG guard on CRC output)
  - `win-game/index.html` (modified: replaced hardcoded info boxes with dynamic #gameStats container)
  - `win-game/src/main.js` (modified: added STAT_CONFIGS lookup table, dynamic stat rendering, coin/start button lighting)
  - `win-game/src/styles.css` (modified: added .coin-active, .start-active, waiting-state button dimming)
- **Reason for change:** Add support for two new arcade games. Make WinGame display relevant stats per game. Provide visual feedback for coin/start buttons.
