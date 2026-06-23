# Sonic Dash Extreme - OutputBlaster Integration Plan

> **Status:** WORKING - Ticket counter tracked via pointer chain, jackpot/coins all reading correctly
> **Created:** 2026-06-22
> **Last Updated:** 2026-06-23
> **Goal:** Add native Sonic Dash Extreme support to OutputBlaster with LED outputs, scores, and ticket counters

---

## Table of Contents

1. [Architecture Overview](#1-architecture-overview)
2. [Current State Assessment](#2-current-state-assessment)
3. [How OutputBlaster Works (Reference)](#3-how-outputblaster-works-reference)
4. [How TeknoParrot Integration Works](#4-how-tekнопарrot-integration-works)
5. [Sonic Dash Extreme - Game Details](#5-sonic-dash-extreme---game-details)
6. [Memory Offsets & Data Sources](#6-memory-offsets--data-sources)
7. [Implementation Steps](#7-implementation-steps)
8. [Build & Verification](#8-build--verification)
9. [Open Questions / Risks](#9-open-questions--risks)

---

## 1. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     TeknoParrotUI (C#)                          │
│  Reads SonicDashExtreme.xml profile                             │
│  Writes "Enable Outputs=1" to teknoparrot.ini                   │
│  Launches BudgieLoader.exe → TeknoParrot.dll                    │
└──────────────────────────────┬──────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│              TeknoParrot.dll (closed-source loader)              │
│  Reads teknoparrot.ini                                           │
│  If "Enable Outputs=1": LoadLibrary("OutputBlaster.dll")        │
│  Hooks into game process, provides JVS emulation                │
└──────────────────────────────┬──────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│              OutputBlaster.dll (this project)                    │
│  DllMain → CreateThread(OutputsLoop)                            │
│  OutputsLoop:                                                   │
│    1. Sleep(2500) for game init                                 │
│    2. CRC32 first 0x400 bytes of game PE header                 │
│    3. switch(CRC) → instantiate game class                      │
│    4. Game::OutputsGameLoop() → spawn polling thread            │
│    5. Poll game memory → SetValue() → MAMEHooker/TCP outputs    │
└──────────────────────────────┬──────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│              SonicDash_R_Ring.exe (game process)                 │
│  Sega Nu platform game (2015)                                   │
│  Portrait mode (1080x1920)                                      │
│  Uses JVS I/O via TeknoParrot emulation                         │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. Current State Assessment

### What Already Exists
- **OutputBlaster.dll** (1.4MB) is already placed in the game root directory
- **OutputBlaster.ini** is already configured (Sleep=200, OutputsSystem=0)
- **TeknoParrot profile** exists at `TeknoParrotUi.Common\GameProfiles\SonicDashExtreme.xml`
- **40+ Python RE scripts** in the game directory have been used to find memory offsets
- **tickets.json** (250K+ lines) contains SRAM change tracking data

### What's Missing
- **No "Enable Outputs" field** in SonicDashExtreme.xml profile
- **No SonicDashExtreme game class** in OutputBlaster source code
- **No CRC entry** in dllmain.cpp for SonicDash_R_Ring.exe
- **No memory offset definitions** for LED/score/ticket outputs in C++ code
- The existing OutputBlaster.dll is likely a generic build without Sonic Dash support

---

## 3. How OutputBlaster Works (Reference)

### File Structure
```
OutputBlaster/
├── dllmain.cpp                    # Entry point + game detection (CRC switch)
├── Common Files/
│   ├── Game.h                     # Base Game class + Helpers + globals
│   ├── Game.cpp                   # Helpers implementation (memory R/W)
│   ├── CRCCheck.h                 # CRC32 algorithm
│   └── MinHook.h                  # API hooking library
├── Output Files/
│   ├── Outputs.h                  # EOutputs enum (233 outputs) + COutputs class
│   ├── Outputs.cpp                # Output value tracking + name mapping
│   ├── WinOutputs.h/.cpp          # MAMEHooker Win32 message IPC
│   └── NetOutputs.h/.cpp          # TCP/UDP network output
└── Game Files/                    # 41 game implementations (.h + .cpp each)
    ├── SonicAllStarsRacing.h/.cpp # Existing Sega game (reference)
    ├── InitialD8.h/.cpp           # Typical complex game
    └── ...
```

### Game Detection Flow (dllmain.cpp:91-341)
1. `DllMain(DLL_PROCESS_ATTACH)` → `CreateThread(OutputsLoop)` (line 349)
2. `OutputsLoop()` sleeps 2500ms (line 93)
3. Reads first 0x400 bytes of game PE header via `GetModuleHandle(nullptr)` (line 95)
4. Zeroes ImageBase field for ASLR normalization (lines 99-104)
5. Computes CRC32 via `GetCRC32(newCrc, 0x400)` (line 105)
6. `switch(newCrcResult)` matches ~40 known CRCs (lines 107-249)
7. If no CRC match → fallback to fixed-address memory probing (lines 262-322)
8. If game found → `game->OutputsGameLoop()` (line 257 or 327)

### Standard Game Handler Pattern
Every game follows this template (example: SonicAllStarsRacing.cpp):

```cpp
#include "SonicAllStarsRacing.h"

static int WindowsLoop() {
    // 1. Read game memory at known offsets
    UINT8 data = helpers->ReadByte(0x813708, true);   // true = relative to module base
    UINT8 data2 = helpers->ReadByte(0x813709, true);
    UINT8 FFB = helpers->ReadByte(0x5CD864, true);

    // 2. Optional: write to enable features
    helpers->WriteByte(0x5CD858, 0x03, true);  // Enable FFB

    // 3. Map to output signals
    Outputs->SetValue(OutputLampStart, !!(data & 0x80));
    Outputs->SetValue(OutputLampLeader, !!(data & 0x40));
    Outputs->SetValue(OutputLampRed, !!(data2 & 0x08));
    // ... more outputs
    Outputs->SetValue(OutputFFB, FFB);
    return 0;
}

static DWORD WINAPI OutputsAreGo(LPVOID lpParam) {
    while (true) {
        WindowsLoop();
        Sleep(SleepA);  // Configurable polling rate from INI
    }
}

void SonicAllStarsRacing::OutputsGameLoop() {
    if (!init) {
        Outputs = CreateOutputsFromConfig();   // CWinOutputs or CNetOutputs
        m_game.name = "Sonic & Sega All Stars Racing";
        Outputs->SetGame(m_game);
        Outputs->Initialize();
        Outputs->Attached();
        CreateThread(NULL, 0, OutputsAreGo, NULL, 0, NULL);
        while (GetMessage(&Msg1, NULL, NULL, 0)) {
            TranslateMessage(&Msg1);
            DispatchMessage(&Msg1);
        }
        init = true;
    }
}
```

### Key Helpers (Game.h/cpp)
- `helpers->ReadByte(offset, isRelative)` - Read 1 byte from memory
- `helpers->ReadInt32(offset, isRelative)` - Read 4-byte integer
- `helpers->ReadFloat32(offset, isRelative)` - Read 4-byte float
- `helpers->ReadIntPtr(offset, isRelative)` - Read pointer-sized value
- `helpers->WriteByte(offset, val, isRelative)` - Write 1 byte
- When `isRelative=true`: offset is added to `GetModuleHandle(NULL)` (game .exe base)
- When `isRelative=false`: offset is treated as absolute virtual address

### Available Output Signals (Outputs.h:24-233)
Key outputs relevant to Sonic Dash Extreme:
- **Lamps:** OutputLampStart, OutputLampRed/Green/Blue, OutputLampLeader
- **RGB LEDs:** OutputSideRed/Green/Blue, OutputWooferRed/Green/Blue, OutputBillboardRed/Green/Blue, OutputItemRed/Green/Blue
- **Mechanical:** Output1pKnock, Output1pMotor, OutputVibration
- **FFB:** OutputFFB
- **Generic numeric:** OutputRPM, OutputPower, OutputSpeedo, OutputBoost

**Note:** There are NO existing output enum values for "ticket count" or "score". These would need to be added to the `EOutputs` enum if we want to expose them as named outputs.

---

## 4. How TeknoParrot Integration Works

### Pipeline: Profile → INI → Loader → DLL
1. **XML Profile** (`SonicDashExtreme.xml`) defines game settings including "Enable Outputs"
2. **TeknoParrotUI** serializes ConfigValues to `teknoparrot.ini` via `ConfigurationWriter.WriteConfigIni()` (ConfigurationWriter.cs:24-61)
3. **BudgieLoader.exe** launches the game and loads `TeknoParrot.dll`
4. **TeknoParrot.dll** reads `teknoparrot.ini`, if `Enable Outputs=1`, loads `OutputBlaster.dll` into the game process
5. **OutputBlaster.dll** `DllMain` fires, spawns the detection/output thread

### Critical: The "Enable Outputs" Field
Currently **missing** from `SonicDashExtreme.xml`. Must be added:

```xml
<FieldInformation>
    <CategoryName>General</CategoryName>
    <FieldName>Enable Outputs</FieldName>
    <FieldValue>1</FieldValue>
    <FieldType>Bool</FieldType>
    <Hint>Enables loading the outputblaster dll if you have it and want to have your lights controlled by the game.</Hint>
</FieldInformation>
```

This goes inside `<ConfigValues>` in the XML profile. The value `1` means enabled by default.

### OutputBlaster.dll Location
Based on the Aliens Extermination MD5 reference, the DLL is expected in the game directory. It's already there at:
`E:\Games-Roms\Tekno\Sonic Dash Extreme (2015)[Sega Nu][TP]\OutputBlaster.dll`

### teknoparrot.ini Current State
Located at `exe\teknoparrot.ini`:
```ini
[General]
Input API=DirectInput
Windowed=1
Fullscreen Display Rotation=Disabled
Use Custom Resolution=1
Custom Resolution Width=540
Custom Resolution Height=960
```
Missing: `Enable Outputs=1` under `[General]`

---

## 5. Sonic Dash Extreme - Game Details

### Platform
- **Platform:** Sega Nu (2015)
- **Executable:** `exe\SonicDash_R_Ring.exe` (5.9MB, x86)
- **Orientation:** Portrait (1080x1920)
- **Emulation Profile:** SegaJvs
- **Emulator Type:** TeknoParrot
- **Test Menu:** `exe\TestMode_x86.exe` (separate executable)

### Game Directory Structure
```
Sonic Dash Extreme (2015)[Sega Nu][TP]/
├── exe/
│   ├── SonicDash_R_Ring.exe          # Main game executable
│   ├── SonicDash_R_Ring.exe.original # Unpatched backup
│   ├── TestMode_x86.exe              # Test/service menu
│   ├── dinput8.dll                   # Input hooking DLL (138KB)
│   ├── lua5.1.dll                    # Lua scripting (168KB)
│   ├── rotate.exe                    # Screen rotation helper
│   ├── teknoparrot.ini               # TeknoParrot config
│   └── TeknoParrot/                  # EEPROM/SRAM save data (24 .bin files)
├── fs/                               # Game data (afs/, bank/, collision/, compiled/, script/, sfd/, shader/, table/, track/, viewer/)
├── OutputBlaster.dll                 # OutputBlaster (already placed, 1.4MB)
├── OutputBlaster.ini                 # OutputBlaster config (already configured)
├── game.bat                          # Launch script (rotate → game → rotate back)
└── [40+ Python RE scripts]           # Reverse engineering tools
```

### Game Launch Sequence
1. `game.bat` runs `rotate.exe 270 1080 1920` (rotate to portrait)
2. Either `TestMode_x86.exe` (test mode) or `SonicDash_R_Ring.exe` (game) launches
3. When launched via TeknoParrot: BudgieLoader → TeknoParrot.dll → game.exe
4. TeknoParrot.dll hooks JVS I/O, provides coin/start/steering emulation
5. If "Enable Outputs" is set, TeknoParrot.dll loads OutputBlaster.dll

---

## 6. Memory Offsets & Data Sources

### What We Know From RE Scripts

#### Ticket Counters (Primary Goal)
The RE scripts show extensive work on finding ticket counter addresses:

- **`detect_offsets.py`**: Tool to compute relative offsets from absolute addresses
  - Usage: `python detect_offsets.py <addr1> <addr2>` where addr1/addr2 are hex absolute addresses
  - Outputs relative offsets from module base

- **`auto_tickets.py`**: Auto-discovers ticket counter addresses by scanning heap
  - Scans all writable private memory regions for uint16 values ≤200
  - Takes snapshots every 15 seconds, looks for monotonic increases
  - Finds pairs of addresses (likely double-buffered ticket counters)
  - Logs ticket events to `tickets.json`

- **`monitor_tickets.py`**: Similar heap scanner for ticket counter discovery
  - Scans committed RW/WC memory regions
  - Filters for uint16 values ≤100
  - Monitors for monotonic increases over 90 seconds

- **`check_addrs.py`**: Checks specific candidate addresses
  - Target addresses found: `0x07d92258`, `0x07e1eb48`, `0x07e1f958`, `0x07cd7404`
  - These are absolute addresses in the game process

- **`find_ticket_strings.py`**: Searches for ticket-related strings in memory
  - Found: `"TICKETS PAID OUT:"` string in TestMode_x86.exe
  - Also searched for: `"MERCY_TICKETS"`, `"MAXIMUM_TICKETS_DISPENSE"`, `"TICKET_PAY_OUT"`, `"NOTCH SENSOR"`, `"EMPTY ERROR"`
  - These are in the TestMode config string area (~0x00ef68xx - 0x00f00000)

- **`mem_scan.py`**: Before/after snapshot comparison for ticket detection
  - Scans 0x02000000 - 0x20000000 for uint32 values 1-999
  - Takes before/after snapshots during ticket dispenser test

#### SRAM/EEPROM Tracking (Secondary Source)
- **`tracker.py`** and **`ticket_mon.py`**: Monitor SRAM file changes
  - Watch `exe/TeknoParrot/*.bin` files for changes
  - Files: `SRAM_0x00004000.bin`, `SRAM_0x00004400.bin`, various EEPROM files
  - Ticket-related values found at SRAM offsets: `0x28`, `0x30`, `0x38`, `0x3C`, `0x68`, `0x84`
  - Example: SRAM_0x00004000.bin values: `0x28=86656`, `0x30=16338`, `0x38=6`, `0x3C=3`, `0x68=529`

#### Important Notes on Memory Addresses
- The absolute addresses found by the Python scripts (like `0x07d92258`) are **NOT stable** across runs due to ASLR
- The `detect_offsets.py` script was designed to convert these to relative offsets from the module base
- **We need the actual relative offsets** to use in the C++ code
- The Python scripts run on the **TestMode_x86.exe** process, NOT the main game process - this is a different executable

### What We Need To Determine

1. **CRC32 of SonicDash_R_Ring.exe PE header** - Need to compute this to add to the CRC switch in dllmain.cpp
   - The CRC is computed on the first 0x400 bytes with ImageBase zeroed out
   - Must be computed on the actual game executable, not the test mode

2. **Relative memory offsets for LED outputs** - What byte offsets control the cabinet LEDs?
   - The game likely has lamp/LED control bytes similar to other Sega Nu games
   - These may be in the game's data section or controlled via JVS I/O

3. **Relative memory offsets for ticket counters** - Converting absolute addresses from RE to relative
   - Need to run `detect_offsets.py` on the MAIN game process (SonicDash_R_Ring.exe), not TestMode
   - The ticket counters may be uint16 values that monotonically increase as tickets are dispensed

4. **Score data offsets** - If score data is readable from memory

### Existing OutputBlaster.ini Config
```ini
[Settings]
Sleep=200          # 200ms polling (slower than default 16ms - appropriate for this game)
MaxScaleOutput=100 # High scale for numeric outputs
OutputsSystem=0    # Windows Messages (MAMEHooker)
```

---

## 7. Implementation Steps

### Phase 1: Pre-Implementation Requirements (DO FIRST)

- [X] **1.1 Compute CRC32 of SonicDash_R_Ring.exe**
  - Initial Python computation gave `0x821BA425` (incorrect)
  - **Live game test confirmed actual CRC: `0xf4b75de0`**
  - The on-disk PE header differs from the in-memory loaded module (TeknoParrot may modify the header)
  - **Always use the live-game-reported CRC** from `New CRC: XXXXXXXX not implemented` debug message
  - Updated in `dllmain.cpp` switch statement

- [ ] **1.2 Determine LED/lamp memory offsets**
  - Start the game via TeknoParrot
  - Use memory scanning to find bytes that change when LEDs should activate
  - Look for bytes in the game's writable memory that control cabinet outputs
  - Consider: The game may use JVS I/O for outputs, which TeknoParrot emulates
  - Alternative: Check if TeknoParrot.dll exports output data that we can read

- [ ] **1.3 Determine ticket counter memory offsets (relative)**
  - Start the game via TeknoParrot
  - Run the heap scanning approach from auto_tickets.py on the MAIN game process
  - Use detect_offsets.py to convert absolute addresses to relative offsets
  - Verify the offsets are stable across multiple game restarts
  - The counters appear to be uint16 values that increase monotonically

- [ ] **1.4 Determine score data offsets (if available)**
  - Check if score data is exposed in memory
  - May be related to the SRAM values tracked by the RE scripts

### Phase 2: TeknoParrot Profile Modification

- [ ] **2.1 Add "Enable Outputs" to SonicDashExtreme.xml**
  - File: `E:\Projects\TeknoParrotUI\TeknoParrotUi.Common\GameProfiles\SonicDashExtreme.xml`
  - Add inside `<ConfigValues>`:
  ```xml
  <FieldInformation>
      <CategoryName>General</CategoryName>
      <FieldName>Enable Outputs</FieldName>
      <FieldValue>1</FieldValue>
      <FieldType>Bool</FieldType>
      <Hint>Enables loading the outputblaster dll if you have it and want to have your lights controlled by the game.</Hint>
  </FieldInformation>
  ```
  - Default value `1` = enabled by default

- [ ] **2.2 Verify teknoparrot.ini gets updated**
  - After profile change, confirm `[General]` section includes `Enable Outputs=1`
  - Located at `exe\teknoparrot.ini`

### Phase 3: OutputBlaster Code Changes

- [ ] **3.1 Create Game Files/SonicDashExtreme.h**
  ```cpp
  #pragma once
  #include "../Common Files/Game.h"
  class SonicDashExtreme : public Game {
  public:
      void OutputsGameLoop();
  };
  ```

- [ ] **3.2 Create Game Files/SonicDashExtreme.cpp**
  - Implement the standard game handler pattern
  - Use the memory offsets determined in Phase 1
  - Map outputs to appropriate EOutputs enum values
  - Ticket counter: Use a numeric output (or add new enum value if needed)
  - LED outputs: Map cabinet LED bytes to OutputLampRed/Green/Blue, OutputSideRed/Green/Blue, etc.
  - FFB: If available, map to OutputFFB

- [ ] **3.3 Add CRC to dllmain.cpp**
  - Add `#include "Game Files/SonicDashExtreme.h"` in the includes section (around line 70)
  - Add CRC case in the switch statement (around line 246, before default):
  ```cpp
  case 0xXXXXXXXX:  // CRC of SonicDash_R_Ring.exe
      game = new SonicDashExtreme;
      break;
  ```

- [ ] **3.4 (Optional) Add new output enum values for tickets/score**
  - If the existing 233 outputs don't cover ticket counts or scores, add new values
  - File: `Output Files/Outputs.h` - add before `NUM_OUTPUTS`
  - File: `Output Files Outputs.cpp` - add name mapping
  - Consider: `OutputTicketCount`, `OutputScore`, etc.

### Phase 4: Build & Test

- [ ] **4.1 Generate Visual Studio project**
  - Run `premake5.bat` → generates `.sln`/`.vcxproj`

- [ ] **4.2 Build OutputBlaster.dll**
  - Build Release|x86 configuration
  - Output: `bin\x86\Release\OutputBlaster.dll`

- [ ] **4.3 Deploy to game directory**
  - Copy built `OutputBlaster.dll` to game root directory
  - Keep existing `OutputBlaster.ini` (already configured)

- [ ] **4.4 Test with TeknoParrot**
  - Launch game via TeknoParrotUI with "Enable Outputs" checked
  - Verify OutputBlaster.dll is loaded (check with Process Explorer)
  - Verify outputs appear in MAMEHooker
  - Test LED control, FFB, and ticket counting

---

## 8. Build & Verification

### Build Commands
```batch
cd E:\Projects\OutputBlaster
premake5.bat                          # Generates VS2017 project
# Then build from Visual Studio: Release|x86
```

### Verification Checklist
- [ ] CRC32 matches the game executable
- [ ] OutputBlaster.dll loads when "Enable Outputs" is set
- [ ] Game class is instantiated (check Debug output: "Game Found" + CRC value)
- [ ] LED outputs respond to game events
- [ ] Ticket counter increments are detected and exposed
- [ ] No crashes or memory access violations
- [ ] Works with both TeknoParrotUI launch and direct command-line launch

### Debug Build
For debugging, build Debug|x86 and check `OutputDebugStringA` output in DebugView:
- "Game Found" + CRC value = game detected correctly
- "New CRC: XXXXXXXX not implemented" = CRC not in switch statement

---

## 9. Open Questions / Risks

### High Priority (Resolved)
1. **What is the CRC32 of SonicDash_R_Ring.exe?**
   - **RESOLVED: `0xf4b75de0`** - First computed as `0x821BA425` (Python, wrong), corrected by live game test showing `New CRC: f4b75de0 not implemented`
   - The CRC in the DLL has been updated to `0xf4b75de0`

2. **What are the actual relative memory offsets for LED outputs?**
   - NOT YET RESOLVED - needs runtime memory scanning
   - Game debug output shows: `JvsMgr_DebugInit()`, `ArcadeInit: 1 JVS node(s) found`
   - Suggests JVS-based I/O - may need to hook JVS functions or scan memory for output registers

3. **Are the ticket counter offsets stable?**
   - NOT YET RESOLVED - The RE scripts ran against TestMode_x86.exe, not the main game
   - Game debug output shows: `TicketMgr_Init()` - Ticket manager initializes at startup
   - Key trigger: `CUIPayOut::FadeIn()` fires when tickets are rewarded at round end
   - Key trigger: `Stage0 - Scene0 START` fires when a round begins
   - Need to scan heap memory during `CUIPayOut::FadeIn()` to find ticket counter values

### Medium Priority
4. **Does TeknoParrot.dll's "Enable Outputs" work for SegaJvs emulation profile?**
   - The closed-source loader must support this combination
   - The UI code doesn't guarantee it works for all emulation profiles

5. **Do we need new EOutputs enum values?**
   - The existing 233 outputs don't include ticket counts or scores
   - Adding new values requires changes to Outputs.h, Outputs.cpp, and WinOutputs.cpp name mapping

6. **What about the TestMode_x86.exe process?**
   - The RE scripts ran against TestMode, not the main game
   - Ticket counters may behave differently in the actual game process
   - The detect_offsets.py tool needs to be run against the correct process

### Low Priority
7. **OutputBlaster.ini Sleep=200 is already set** - This is 12.5x slower than the default 16ms. Is this intentional for this game?

8. **Should we hook JVS functions instead of polling memory?**
   - Some games use MinHook to intercept I/O functions
   - May be more reliable than memory polling for Sega Nu games

---

## Appendix A: Key File Locations

| File | Path |
|------|------|
| OutputBlaster source | `E:\Projects\OutputBlaster\` |
| dllmain.cpp (game detection) | `E:\Projects\OutputBlaster\dllmain.cpp` |
| Game base class | `E:\Projects\OutputBlaster\Common Files\Game.h` |
| Output enum | `E:\Projects\OutputBlaster\Output Files\Outputs.h` |
| Reference game (Sonic) | `E:\Projects\OutputBlaster\Game Files\SonicAllStarsRacing.cpp` |
| TeknoParrotUI source | `E:\Projects\TeknoParrotUI\` |
| SonicDashExtreme profile | `E:\Projects\TeknoParrotUI\TeknoParrotUi.Common\GameProfiles\SonicDashExtreme.xml` |
| ConfigWriter (INI gen) | `E:\Projects\TeknoParrotUI\TeknoParrotUi\Views\GameRunningCode\Utilities\ConfigurationWriter.cs` |
| Game directory | `E:\Games-Roms\Tekno\Sonic Dash Extreme (2015)[Sega Nu][TP]\` |
| Game executable | `E:\Games-Roms\Tekno\Sonic Dash Extreme (2015)[Sega Nu][TP]\exe\SonicDash_R_Ring.exe` |
| teknoParrot.ini | `E:\Games-Roms\Tekno\Sonic Dash Extreme (2015)[Sega Nu][TP]\exe\teknoparrot.ini` |
| OutputBlaster.ini | `E:\Games-Roms\Tekno\Sonic Dash Extreme (2015)[Sega Nu][TP]\OutputBlaster.ini` |
| OutputBlaster.dll | `E:\Games-Roms\Tekno\Sonic Dash Extreme (2015)[Sega Nu][TP]\OutputBlaster.dll` |
| Offset detection tool | `E:\Games-Roms\Tekno\Sonic Dash Extreme (2015)[Sega Nu][TP]\detect_offsets.py` |
| Auto ticket scanner | `E:\Games-Roms\Tekno\Sonic Dash Extreme (2015)[Sega Nu][TP]\auto_tickets.py` |
| Ticket string finder | `E:\Games-Roms\Tekno\Sonic Dash Extreme (2015)[Sega Nu][TP]\find_ticket_strings.py` |

## Appendix B: Similar Game Implementations for Reference

| Game | File | Notes |
|------|------|-------|
| SonicAllStarsRacing | `Game Files/SonicAllStarsRacing.cpp` | Simple: 13 outputs, LED + FFB |
| InitialD8 | `Game Files/InitialD8.cpp` | Complex: lamp + speedo + FFB outputs |
| BattlePod | `Game Files/BattlePod.cpp` | Uses ReadInt32 for numeric outputs |
| CruisnBlast | `Game Files/CruisnBlast.cpp` | Uses MinHook for function hooking |
| JurassicPark | `Game Files/JurassicPark.cpp` | Multiple lamp hooks, writes to enable upright mode |
| M2Emulator | `Game Files/M2Emulator.cpp` | Pointer chain indirection pattern |

## Appendix C: Complete Findings & Replication Guide

### How the Ticket Counter Was Found (Replication Steps)

1. **Launch game** via TeknoParrotUI with `Enable Outputs=1` in the profile
2. **Attach Cheat Engine** to `SonicDash_R_Ring.exe` process
3. **Scan for tickets**: Play a round, note ticket count at end, scan in Cheat Engine as 4-byte value
4. **Pointer scan**: After finding a stable address (e.g., `0D29F3E8`), Cheat Engine pointer scan reveals:
   ```
   "SonicDash_R_Ring.exe"+0084A070 → [read pointer] → +8 → ticket counter (4 bytes)
   ```
5. **Verify stability**: Restart game, confirm pointer chain resolves to same semantic address every time
6. **Bonus addresses found**:
   - `+84A238` → Ticket Jackpot (4 bytes, increments each round)
   - `+78B3B8` → Coin/Credit counter 1 (4 bytes, decrements per continue)
   - `+7A6788` → Coin/Credit counter 2 (4 bytes, decrements per continue)
   - `+7A678C` → 2nd value at same area (related counter)

### Memory Map (Verified)
```
Base address of SonicDash_R_Ring.exe (GetModuleHandle(NULL))
  ├── +0x84A070 → pointer → +8 = ticket counter (uint32)
  ├── +0x84A238 = ticket jackpot (uint32)
  ├── +0x78B3B8 = coin credit 1 (uint32)
  ├── +0x7A6788 = coin credit 2a (uint32)
  └── +0x7A678C = coin credit 2b (uint32)
```

### Game Event Flow (from DebugView strings)
```
Stage0 - Scene0 START       → Round begins (ticket counter starts at 0)
  ... gameplay ...
  ticket counter climbs 1→5→8→13→...→27 during round
BossBattleSystem::Enable()  → Boss reached (ticket jump: 27→43)
CUIPayOut::FadeIn()         → Round ends, payout shown (ticket resets to 0)
```

### How OutputBlaster Reads the Ticket
```cpp
// In SonicDashExtreme.cpp ResolveTicketPointer():
uintptr_t moduleBase = (uintptr_t)GetModuleHandle(NULL);
uint32_t* ptrField = (uint32_t*)(moduleBase + 0x84A070);
uint32_t ptrVal = *ptrField;
g_TicketAddr = (uint32_t*)(ptrVal + 8);  // ticket address = deref + 8

// Then each loop:
uint32_t ticketNow = *g_TicketAddr;       // read ticket count
uint32_t jackpot = helpers->ReadInt32(0x84A238, true);
uint32_t coins1 = helpers->ReadInt32(0x78B3B8, true);
uint32_t coins2a = helpers->ReadInt32(0x7A6788, true);
uint32_t coins2b = helpers->ReadInt32(0x7A678C, true);
```

### Per-Round Ticket Tracking Logic
- Round start detected when ticket goes from 0 → > 0 (Stage0 Scene0 START)
- Round end detected when ticket resets to 0 (CUIPayOut::FadeIn)
- Peak ticket during round = final ticket count for that round
- BossBattleSystem::Enable = boss was reached (tracked as bool)
- JSON file written to game directory: `tickets_outputblaster.json`

### OutputBlaster EOutputs Added
```
OutputTicketCounter  (#233) "TicketCounter"  - Per-round ticket count
OutputTicketJackpot  (#234) "TicketJackpot"  - Jackpot/round counter
OutputCoin1          (#235) "Coin1"          - Credit counter
OutputCoin2          (#236) "Coin2"          - Related credit counter
```

### To Replicate for Another TeknoParrot Game
1. **Get CRC**: Launch game with OutputBlaster that logs `New CRC: XXXXXXXX not implemented`, copy the hex CRC
2. **Create game files**: Copy `SonicDashExtreme.h/.cpp`, rename, update class name
3. **Add CRC case**: In `dllmain.cpp` switch statement
4. **Find ticket via CheatEngine**: Same pointer scan approach as above
5. **Find lamps/LEDs**: Unknown offsets currently — need runtime memory scanning

### Key Files
| File | Purpose |
|------|---------|
| `Game Files/SonicDashExtreme.h` | Game class declaration |
| `Game Files/SonicDashExtreme.cpp` | Main logic: pointer resolution, value polling, JSON output |
| `Output Files/Outputs.h` | EOutputs enum (add new outputs before `NUM_OUTPUTS`) |
| `Output Files/Outputs.cpp` | Output name strings (must match enum order) |

## Appendix D: Notes for Next Pass

**DO NOT SKIP THESE:**
1. The CRC computation must be done on `SonicDash_R_Ring.exe` (the main game), NOT `TestMode_x86.exe`
2. The RE scripts in the game directory ran against `TestMode_x86.exe` - the memory layout may differ in the main game
3. The absolute addresses in `check_addrs.py` (0x07d92258, etc.) are NOT usable as-is - they must be converted to relative offsets using `detect_offsets.py`
4. The `OutputBlaster.ini` already exists and is configured - don't overwrite it
5. The `teknoparrot.ini` lives in the `exe/` subdirectory, not the game root
6. The game runs in portrait mode (1080x1920) - this may affect output behavior
7. The `Sleep=200` in OutputBlaster.ini is already set to a slower polling rate - this may be intentional for this game
