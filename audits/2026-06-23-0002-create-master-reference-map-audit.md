# Audit: Create Master Reference Map

- **Date:** 2026-06-23
- **Task:** 0002-create-master-reference-map
- **Pass Type:** Audit + Implementation (combined)

## Files Reviewed
- All key source files listed in task 0002
- Full repo governance structure (Agents.md, Repo_map.md, etc.)
- OutputHooker source (architectural understanding)
- Sonic Dash Extreme INI, game directory structure, memory offsets

## Findings

1. **Project is well-structured** — The OutputBlaster codebase follows a consistent pattern across all 48+ games. The Game class hierarchy with OutputsGameLoop() pattern is clean and repeatable.

2. **Output system is comprehensive** — 215 outputs covering lamps, RGB LEDs, mechanical outputs, FFB, speedo segments, boost segments, and numeric counters. Recent additions (OutputTicketCounter, OutputTicketJackpot, OutputCoin1, OutputCoin2, OutputHighScore) were added at the end of the enum.

3. **Sonic Dash Extreme is the most feature-rich handler** — It's the only game that uses pointer chain resolution, per-round ticket tracking with boss detection, JVS shared memory polling, and JSON output files. This makes it the ideal reference implementation.

4. **Two game detection modes** — CRC32-based (TeknoParrot games, 48 games) and fixed-address probing (Lindbergh games, 14 games). The fallback probing is fragile but necessary for platforms without reliable CRCs.

5. **OutputHooker is a mature application** — Qt6-based, supports 7+ output driver modules (WinMsg, TCP, COM port, USB HID, LED-Wiz, PacDrive, SDL3), fully INI-configurable per game.

## Risks

1. **CRC table maintenance** — The CRC table in dllmain.cpp has no comments explaining which games/game versions correspond to each CRC. New CRCs are added ad-hoc without documentation of the source game executable.

2. **Name string ordering** — The name strings in Outputs.cpp must exactly match the enum order in Outputs.h. There's no compile-time check for this alignment, and reordering would silently break output mapping.

3. **Numeric value truncation** — All numeric values (ticket counter, high score) are cast to UINT8 (0-255). Values exceeding 255 will wrap, losing data. The `MaxScaleOutput` INI setting mitigates this for some games but isn't universally applied.

4. **Sleep=200 for Sonic Dash** — The polling rate is 12.5x slower than default. If the game ever needs faster response (e.g., pulse-width modulated LEDs, real-time FFB), this will need adjustment.

## Decisions

1. **Master map goes in `docs/MASTER_MAP.md`** — This creates the `docs/` directory as the documentation home, consistent with standard project structure and the governance rulebook (Section 9.8).

2. **Full output reference included** — All 215 outputs are documented by category with their enum name, string name, type, and description. This makes it a single-source reference for game developers.

3. **Sonic Dash Extreme is the canonical example** — Its handler code is the most complex and feature-rich, making it the best reference for adding extra outputs (ticket counter, high score, coins) to other games.

## Implementation Notes

- Created `docs/MASTER_MAP.md` — comprehensive reference document
- Updated `Repo_map.md` — added `docs/` directory entry
- Updated `changelog.md` — added 2026-06-23 entry
- Updated `tasks/TASK_INDEX.md` — added task 0002 entry
- Created `tasks/0002-create-master-reference-map.md`
- Created `audits/2026-06-23-0002-create-master-reference-map-audit.md`
- No source code files were modified

## Freshness Status
- **Fresh** (created 2026-06-23)
