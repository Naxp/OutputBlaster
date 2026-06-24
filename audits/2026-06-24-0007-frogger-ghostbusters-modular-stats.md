# Audit: Frogger + Ghostbusters Support, WinGame Modular Stats & Button Lighting

**Date:** 2026-06-24
**Task Reference:** tasks/0007-frogger-ghostbusters-modular-stats.md
**Status:** Fresh

## Files Reviewed
- Game Files/Frogger.h, Frogger.cpp
- Game Files/Ghostbusters.h, Ghostbusters.cpp
- dllmain.cpp
- win-game/src/main.js
- win-game/src/styles.css
- win-game/index.html

## Findings
1. **Frogger handler**: Uses module-relative offsets for ticket (base+0x135C80) and coins (base+0x0C91F8, base+0x0C9230). Copies ticket counter to HighScore for WinGame display since Frogger has no score output.
2. **Ghostbusters handler**: Uses absolute address 0x08E16368 for score/ghosts until user can test real module-relative offsets.
3. **CRC detection**: Both games have placeholder CRCs (0xf4b75de1, 0xf4b75de2). _DEBUG guard removed from CRC output so Release builds show `OB: No game match — CRC: XXXXXXXX` in DebugView.
4. **WinGame modular stats**: STAT_CONFIGS lookup table maps game names to 4-stat configurations. Dynamically renders info boxes. Falls back to default config for unknown games.
5. **Button lighting**: Coin buttons get `.coin-active` (gold glow) when Coin1/Coin2 > 0. Start button gets `.start-active` (green glow) when LampStart == 1.
6. **Waiting state**: Buttons dim to 0.3 opacity when disconnected.

## Risks
- **Placeholder CRCs**: Both games will not activate until real CRCs are confirmed via DebugView. One-line fix each.
- **Ghostbusters offsets**: Score and ghosts mapped to same address (0x08E16368). User needs to find separate ghost offset.
- **Ghostbusters EXE**: Unknown EXE name; absolute addresses may need conversion to module-relative.
- **Frogger CWD**: sdaemon.exe in root dir; DLL loaded from same dir. Test needed to verify.

## Decisions
- **Frogger ticket → HighScore**: Since Frogger has no score, ticket counter is copied to HighScore for WinGame display. User can decide later if a dedicated score offset exists.
- **CRC placeholders**: Using descriptive placeholders (FROGGER_CRC = 0xf4b75de1, GHOSTBUSTERS_CRC = 0xf4b75de2) so the case structure is ready; one constant change per game to activate.
- **STAT_CONFIGS in JS**: Client-side lookup avoids backend changes. Easy to extend by editing the map.

## Implementation Notes
- DLL builds clean (Release|x86)
- WinGame builds clean (cargo build --release)
- Deployed to Frogger, Ghostbusters, and Sonic game directories
