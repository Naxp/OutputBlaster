# Audit 2026-06-24-0006: Broadcast Outputs & WinGame UI Redesign

**Date:** 2026-06-24
**Task Reference:** 0006-broadcast-outputs-wingame-ui-redesign

## Files Reviewed
- Common Files/Game.h, Game.cpp
- Output Files/Outputs.h, Outputs.cpp, Outputs.h
- Output Files/WinOutputs.h, WinOutputs.cpp
- Output Files/NetOutputs.h, NetOutputs.cpp
- Game Files/SonicDashExtreme.cpp
- win-game/index.html, src/main.js, src/styles.css
- win-game/src-tauri/src/lib.rs, build.rs
- win-game/simulate.py
- Existing docs: MASTER_MAP.md, changelog.md, Repo_map.md

## Findings

### Architecture Issue: Single Output Backend
`CreateOutputsFromConfig()` returned either `CWinOutputs` OR `CNetOutputs` — never both. This meant:
- With `OutputsSystem=1` (NetOutputs), OutputHooker couldn't receive data via WinMsg protocol
- With `OutputsSystem=0` (WinOutputs), WinGame couldn't receive data via TCP

**Fix:** Created `CBroadcastOutputs` wrapper class that holds both backends and forwards `SendOutput()`, `Initialize()`, and `Attached()` calls to both. Now `OutputsSystem=1` enables both simultaneously.

### WinGame build.rs Caching Bug
`build.rs` had `if !dist.exists()` guard — only rebuilt frontend when `dist/index.html` was missing. Source file changes went undetected because `cargo:rerun-if-changed` only watched `dist/index.html`, not the source files.

**Fix:** Removed the guard — always runs `npm run build`. Added `cargo:rerun-if-changed` for `../index.html`, `../src/main.js`, `../src/styles.css`.

### Port Numbering Conflicts
Port 8000 is widely used (many dev tools, web servers). Port 8001 also common.

**Fix:** Changed default ports to 37520 (TCP) and 37521 (UDP broadcast) — in the dynamic/private range, unlikely to conflict.

### INI Discovery Issue
`Game.cpp` uses `settingsFilename = TEXT(".\\OutputBlaster.ini")` which is relative to the current working directory. When TeknoParrot sets CWD to the `exe\` subdirectory instead of the game root, the INI is not found, causing `OutputsSystem` to default to 0.

**Fix:** Copy OutputBlaster.ini to BOTH the game root AND the `exe\` subdirectory.

### WinGame UI Layout
Previous layout used abstract LED bars — not representing actual hardware layout. User requested:
- Billboard: triangle shape with per-color R/G/B indicators
- Woofers: 3 speaker shapes (2 small, 1 large center)
- Side LEDs: Left/Right columns with individual R/G/B dots
- Item LEDs: individual colored dots
- Misc box: auto-detects any active output not in the layout, shows correct color

**Fix:** Complete rewrite of frontend HTML/CSS/JS.

## Risks
- `CBroadcastOutputs` creates orphaned `COutputs*` objects in the error path if one Initialize fails — but the destructor handles cleanup
- The `raw` HashMap in `OutputsSnapshot` includes ALL outputs including internal ones (`pause`, `mame_start`) — the Misc box in JS filters these out
- If the frontend npm build fails, the entire Rust build fails — this is intentional but may be annoying during rapid iteration

## Decisions
1. **Broadcast over single backend**: The wrapper approach is minimal code change. Alternative (refactoring to a vector of outputs) would require changing all game handlers.
2. **Always rebuild frontend**: Trade-off of ~5s build time vs. silent failures from cached builds. Worth it.
3. **Player initials in AppState**: Simple Mutex<String> with default "---". Commands get/set. Persistence across sessions via scores.json only (initials reset on app restart).
4. **Ports 37520/37521**: Chosen from IANA dynamic range (49152-65535 is actually the official range, but 37520 is well above common use).

## Implementation Notes
- `BroadcastOutputs.h/.cpp` is in `Output Files/` — auto-included by premake5 wildcard glob
- WinGame `vite.config.js` has `base: "./"` and uses `vite-plugin-singlefile` — ensures all CSS/JS inlined into single HTML page
- The `wooferOut` element is inside `woofer-large` div — colored dots render inside the largest speaker circle
- Billboard triangle uses CSS border trick (`border-bottom` color changes based on active LED)
- Misc box uses `buildMiscBox(o.raw)` which excludes known outputs and shows unknown active ones with auto-detected color
- `roundEndPending` flag in JS prevents multiple initials modals per session

## Freshness
Fresh — created during Pass 0006.
