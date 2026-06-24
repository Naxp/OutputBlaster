# Audit 0004 — WinGame draggable + simulate + GameProfiles XML

- **Date:** 2026-06-24
- **Task reference:** 0004
- **Files reviewed:**
  - `win-game/index.html`
  - `win-game/public/styles.css`
  - `win-game/src/main.js`
  - `win-game/src-tauri/src/lib.rs`
  - `C:\Users\robon\Desktop\TPBootstrapper\GameProfiles\SonicDashExtreme.xml`
  - `C:\Users\robon\Desktop\TPBootstrapper\UserProfiles\SonicDashExtreme.xml`
  - `E:\Games-Roms\Tekno\Sonic Dash Extreme (2015)[Sega Nu][TP]\OutputBlaster.ini`
  - `E:\Games-Roms\Tekno\Sonic Dash Extreme (2015)[Sega Nu][TP]\oxGetHwInfo.ini`
- **Findings:**

### 1. Root cause: OB DLL not injected

Despite `Enable Outputs=1` in UserProfiles XML and correct `OutputBlaster.ini`, the DLL is NOT loaded in SonicDash_R_Ring.exe (PID 8432). Two issues:

**a) GameProfiles template missing Enable Outputs field**
`GameProfiles/SonicDashExtreme.xml` did not define an `Enable Outputs` `FieldInformation` entry. TeknoParrot may ignore fields not present in the template, even if the UserProfiles has them. FIX: Added `FieldInformation` for `Enable Outputs` with type `Bool` and default value `1`.

**b) Game was launched before profile update**
The game process (PID 8432) was started before the UserProfiles was updated with `Enable Outputs=1`. The profile change only takes effect on game relaunch via TeknoParrot.

### 2. Windows borderless window immovable

`decorations: false` + `alwaysOnTop: true` in tauri.conf.json made the window undraggable. FIX: Added `data-tauri-drag-region` attribute to the marquee `<div>` and corresponding cursor CSS.

### 3. No way to test WinGame UI without live game

Added `simulate` Tauri command that injects time-varying fake data (scores, tickets, lamps, coins). Added "Sim Data" button in the right-panel area.

### 4. No graceful close mechanism

Added close button (✕) in marquee + `close_app` Rust command (`std::process::exit(0)`).

- **Risks:**
  - GameProfiles XML edit may be overwritten by TeknoParrot updates
  - `simulate` command bypasses real TCP flow — only for UI testing
  - `close_app` via `exit(0)` is abrupt but acceptable for kiosk-mode app
- **Decisions:**
  - Use `data-tauri-drag-region` instead of re-enabling `decorations: true` (preserves borderless aesthetic)
  - Inject simulated data directly into AppState HashMap rather than through TCP path
  - Game restart is required to test OB DLL injection fix
- **Implementation notes:**
  - Simulated data updates every poll cycle (200ms) with time-based pseudo-random values
  - User must kill WinGame via Task Manager or close button (Alt+F4 still works)
  - GameProfiles XML edit only affects future TeknoParrot launches
- **Freshness status:** ✓ Fresh
