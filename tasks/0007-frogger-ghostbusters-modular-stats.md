# Task 0007: Frogger + Ghostbusters Support, WinGame Modular Stats & Button Lighting

**Date:** 2026-06-24
**Status:** Completed

## Goal
- Add Frogger and Ghostbusters game handlers to OutputBlaster DLL
- Make WinGame stat boxes modular per game (config-based lookup)
- Add coin/start button lighting based on live output values
- Deploy to game directories

## Scope
- DLL: New Frogger handler with ticket/coin memory offsets
- DLL: New Ghostbusters handler with score/ghost offsets
- DLL: CRC placeholders for both games (user confirms via DebugView)
- DLL: CRC debug output always visible (removed _DEBUG guard)
- WinGame: STAT_CONFIGS lookup table mapping game names to stat definitions
- WinGame: Dynamic stat box rendering from config
- WinGame: Coin button lights up when Coin1 or Coin2 > 0
- WinGame: Start button lights up when LampStart == 1
- Deploy DLL, INI, WinGame to Frogger and Ghostbusters game roots

## Checklist
- [x] Create Frogger.h/.cpp with ticket (base+0x135C80), coin P1 (base+0x0C91F8), coin P2 (base+0x0C9230)
- [x] Create Ghostbusters.h/.cpp with score P1 (0x08E16368), ghosts P1 (placeholder)
- [x] Add includes and CRC cases in dllmain.cpp
- [x] Remove _DEBUG guard on CRC output for Release builds
- [x] Regenerate premake5 project to include new .cpp files
- [x] Build DLL (Release|x86)
- [x] Add STAT_CONFIGS lookup in main.js (Sonic, Frogger, Ghostbusters)
- [x] Replace hardcoded stat boxes with dynamic rendering from config
- [x] Add coin-active CSS class toggle on Coin1/Coin2 > 0
- [x] Add start-active CSS class toggle on LampStart
- [x] Add waiting-state CSS for buttons when disconnected
- [x] Build WinGame (cargo build --release)
- [x] Deploy DLL, INI, win-game.exe to Frogger game root
- [x] Deploy DLL, INI, win-game.exe to Ghostbusters game root
- [x] Update win-game.exe in Sonic exe dir
- [x] Update changelog, task index, audit

## Files Reviewed
- Game Files/SonicDashExtreme.h/.cpp (reference pattern)
- Common Files/Game.h (base Game class)
- Common Files/Game.cpp (CreateOutputsFromConfig, OutputsAreGo)
- Output Files/Outputs.h (EOutputs enum)
- dllmain.cpp (CRC switch structure)
- win-game/src/main.js (render loop)
- win-game/src/styles.css (button styles)
- win-game/index.html (HTML structure)

## Files Changed
- Game Files/Frogger.h (new)
- Game Files/Frogger.cpp (new)
- Game Files/Ghostbusters.h (new)
- Game Files/Ghostbusters.cpp (new)
- dllmain.cpp (modified: Frogger/Ghostbusters includes + CRC cases, removed _DEBUG guard)
- win-game/index.html (modified: replaced hardcoded info boxes with #gameStats container)
- win-game/src/main.js (modified: added STAT_CONFIGS, dynamic stat rendering, button lighting)
- win-game/src/styles.css (modified: added .coin-active, .start-active, waiting button styles)
- premake5.bat (regenerated .vcxproj to include new .cpp files — gitignored)

## Audit Reference
- audits/2026-06-24-0007-frogger-ghostbusters-modular-stats.md

## Completion Status
- Complete
