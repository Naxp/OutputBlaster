# Task 0006: Broadcast Outputs, WinGame UI Redesign & Port Change

**Date:** 2026-06-24
**Status:** Completed

## Goal
- Enable both WinOutputs (OutputHooker) and NetOutputs (WinGame) simultaneously
- Change ports from 8000/8001 to 37520/37521 to avoid conflicts
- Redesign WinGame LED layout per user request (Billboard triangle, Woofer speakers, per-color LEDs, Misc box)
- Fix WinGame build.rs to always rebuild frontend when sources change
- Fix high score flow with persistent initials and change button

## Scope
- DLL: New `CBroadcastOutputs` class that wraps both CWinOutputs and CNetOutputs
- DLL: `Game.cpp` CreateOutputsFromConfig now always creates CWinOutputs, and additionally creates CNetOutputs for OutputsSystem=1
- Default ports changed from 8000/8001 to 37520/37521
- WinGame frontend: complete rewrite of main.js, styles.css, index.html with new layout
- WinGame backend: added raw HashMap to OutputsSnapshot, get_initials/set_initials commands, player_initials in AppState
- WinGame build.rs: always runs npm run build (no caching), added rerun-if-changed for source files

## Checklist
- [x] Create BroadcastOutputs.h/.cpp wrapper class
- [x] Modify CreateOutputsFromConfig to use both output backends
- [x] Change default ports to 37520/37521
- [x] Update INI files in both game root and exe dirs
- [x] Redesign WinGame frontend HTML (index.html)
- [x] Rewrite WinGame main.js with new LED layout logic
- [x] Rewrite WinGame styles.css for new layout
- [x] Add raw outputs HashMap to OutputsSnapshot
- [x] Add get_initials/set_initials Tauri commands
- [x] Fix build.rs to always rebuild frontend
- [x] Build DLL (Release|x86)
- [x] Build WinGame (cargo build --release)
- [x] Deploy both to game directories
- [x] Update docs/MASTER_MAP.md with new cheat sheet
- [x] Update changelog, task index, audit index

## Files Reviewed
- Common Files/Game.h, Game.cpp
- Output Files/Outputs.h, Outputs.cpp
- Output Files/WinOutputs.h, WinOutputs.cpp
- Output Files/NetOutputs.h, NetOutputs.cpp
- Game Files/SonicDashExtreme.cpp
- win-game/index.html
- win-game/src/main.js
- win-game/src/styles.css
- win-game/src-tauri/src/lib.rs
- win-game/src-tauri/build.rs
- win-game/simulate.py
- docs/MASTER_MAP.md
- changelog.md
- Repo_map.md

## Files Changed
- Output Files/BroadcastOutputs.h (new)
- Output Files/BroadcastOutputs.cpp (new)
- Common Files/Game.cpp (modified: includes broadcast, CreateOutputsFromConfig uses both outputs)
- win-game/index.html (rewritten)
- win-game/src/main.js (rewritten)
- win-game/src/styles.css (rewritten)
- win-game/src-tauri/src/lib.rs (modified: raw HashMap, initials, port fix)
- win-game/src-tauri/build.rs (modified: always build frontend, rerun-if-changed)
- win-game/simulate.py (port changed to 37520)
- docs/MASTER_MAP.md (updated: port changes, new cheat sheet, broadcast architecture)
- changelog.md (updated)
- Repo_map.md (updated)
- Audits_index.md (updated)
- tasks/TASK_INDEX.md (updated)
- audits/2026-06-24-0006-broadcast-outputs-wingame-ui-redesign.md (new)
- tasks/0006-broadcast-outputs-wingame-ui-redesign.md (this file)

## Audit Reference
- `audits/2026-06-24-0006-broadcast-outputs-wingame-ui-redesign.md`

## Completion Status
- Complete
