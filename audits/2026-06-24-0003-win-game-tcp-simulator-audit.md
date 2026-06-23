# Audit: WinGame TCP Simulator and Pipeline Test

**Date:** 2026-06-24  
**Task:** 0003-win-game-tcp-simulator-and-test  
**Freshness:** Fresh

## Files Reviewed
- `Output Files/NetOutputs.cpp`
- `Output Files/NetOutputs.h`
- `Output Files/Outputs.cpp`
- `Game Files/SonicDashExtreme.cpp`
- `win-game/src-tauri/src/lib.rs`
- `win-game/src/main.js`
- `win-game/src-tauri/Cargo.toml`

## Findings

1. **NetOutputs TCP protocol:**
   - `SeparatorIdAndValue = " = "` (space-equals-space)
   - `FrameEnding = "\r"` by default; `"\r\n"` when `NetOutputsWithLF=1`
   - On client connect: sends `mame_start = <GameName>\r\n`, then all output values as `<Name> = <Value>\r\n`
   - On value change: sends `<Name> = <Value>\r\n` to all registered clients only
   - WinGame INI is configured with `NetOutputsWithLF=1` → `\r\n` line endings

2. **WinGame Rust backend** (lib.rs) correctly parses `Name = Value` lines, maps to:
   - Numeric: `TicketCounter`, `TicketJackpot`, `Coin1`, `Coin2`, `HighScore`
   - Lamps: `LampStart`, `LampLeader`, `LampRed/Green/Blue`, `Billboard Red/Green/Blue`, `SideLEDRed/Green/Blue`, `WooferLEDRed/Green/Blue`, `ItemLEDRed/Green/Blue`
   - Round detection: `TicketCounter` 0→positive = round start, positive→0 = round end

3. **Crash bug:** `tokio::spawn` called in Tauri 2's `setup` hook where no tokio runtime exists. Tauri 2.11.3 does not provide automatic tokio context. Fix: `tauri::async_runtime::spawn` — the runtime-agnostic equivalent.

4. **Simulator script** (simulate.py) implements full attract→game→boss→payout→reset cycle with realistic timing and lamp patterns matching Sonic Dash Extreme signal names.

## Risks
- Simulator runs as a single-threaded blocking script (fine for testing, not production)
- WinGame will freeze if TCP connection stalls — no timeout/retry on read (low risk, only occurs if OutputBlaster crashes)

## Decisions
- Use `tauri::async_runtime::spawn` not `tokio::spawn` (doesn't require tauri tokio feature)
- Simulator uses `\r\n` line endings matching the `NetOutputsWithLF=1` INI setting
- All 22 Sonic Dash outputs included for realistic visual feedback

## Implementation Notes
- simulate.py uses `threading.Thread` to handle one client at a time (no need for asyncio since it's sequential blocking sends)
- WinGame rebuild from scratch: `cargo build --release` — 1m 27s, successful
- Test verified: WinGame process running at 26 MB with visible window title "Arcade Output Display"
