# OutputBlaster — Repository Map

> **Last updated:** 2026-06-24
> **Task:** 0007-frogger-ghostbusters-modular-stats

---

## Repository Structure

```
E:\Projects\OutputBlaster\
├── .git/                          # Git repository
├── .gitignore                     # Ignores .sln, .vcxproj, .vs/, bin/, obj/
├── Agents.md                      # Agent governance rulebook
├── Audits_index.md                # Index of all audit files
├── changelog.md                   # Project changelog
├── LICENSE.txt                    # GPL v3 license
├── README.md                      # Project overview
├── Repo_map.md                    # This file — repo structure map
├── dllmain.cpp                    # DLL entry point + game detection (CRC switch)
├── premake5.lua                   # Premake5 build configuration
├── premake5.bat                   # Premake5 generator launcher
├── PLAN_SonicDashExtreme.md       # Active Sonic Dash Extreme integration plan
│
├── Common Files/                  # Shared infrastructure
│   ├── Game.h                     # Base Game class + helpers + globals
│   ├── Game.cpp                   # Helpers implementation (memory R/W) + CreateOutputsFromConfig
│   ├── CRCCheck.h                 # CRC32 algorithm
│   ├── MinHook.h                  # MinHook API hooking library header
│   └── MinHook/                   # MinHook lib binaries (x86/x64, Debug/Release)
│
├── Game Files/                    # Per-game implementations (41+ games)
│   ├── <GameName>.h               # Game class declaration
│   ├── <GameName>.cpp             # Game output polling implementation
│   ├── SonicDashExtreme.h/.cpp    # Current active development target
│   └── ...                        # (116 files total)
│
├── Output Files/                  # Output signal system
│   ├── Outputs.h                  # EOutputs enum (236+ outputs) + COutputs class
│   ├── Outputs.cpp                # Output value tracking + name mapping
│   ├── WinOutputs.h/.cpp          # MAMEHooker Win32 message IPC
│   ├── NetOutputs.h/.cpp          # TCP/UDP network output
│   ├── BroadcastOutputs.h/.cpp    # Dual-backend wrapper (WinOutputs + NetOutputs simultaneous)
│   └── GameOutput.h               # Game metadata struct
│
├── win-game/                      # Tauri arcade display app
│   ├── index.html                 # Vite entry point (HTML layout)
│   ├── package.json               # Node dependencies
│   ├── vite.config.js             # Vite config (singlefile + base: "./")
│   ├── src/
│   │   ├── main.js                # Frontend JS (LED rendering, TCP polling, debug)
│   │   └── styles.css             # All styles (cabinet, LEDs, modal, debug)
│   ├── src-tauri/
│   │   ├── Cargo.toml             # Rust deps (tauri, tokio, serde, chrono)
│   │   ├── tauri.conf.json        # Window config (1280x800, borderless, wingame://)
│   │   ├── capabilities/
│   │   │   └── default.json       # Tauri IPC permissions (window API)
│   │   ├── build.rs               # Frontend build + embed as byte array in generated.rs
│   │   └── src/
│   │       └── lib.rs             # Backend: TCP client, custom protocol, Tauri commands
│   ├── dist/                      # Built frontend (gitignored)
│   └── target/                    # Rust build artifacts (gitignored)
│
├── docs/
│   └── MASTER_MAP.md              # Complete reference (outputs, memory, architecture, cheat sheet)
│
├── audits/                        # Audit files (governance)
│   ├── 2026-06-23-0001-agents-governance-standard-audit.md
│   ├── 2026-06-23-0002-create-master-reference-map-audit.md
│   ├── 2026-06-24-0003-win-game-tcp-simulator-audit.md
│   ├── 2026-06-24-0004-win-game-drag-simulate-audit.md
│   ├── 2026-06-24-0005-sonic-dash-rings-audit.md
│   └── 2026-06-24-0006-broadcast-outputs-wingame-ui-redesign.md
│
└── tasks/                         # Task files (governance)
    ├── TASK_INDEX.md              # Task index
    ├── 0001-update-agents-governance-standard.md
    ├── 0002-create-master-reference-map.md
    ├── 0003-win-game-tcp-simulator-and-test.md
    ├── 0004-win-game-drag-simulate.md
    ├── 0005-sonic-dash-rings-output.md
    └── 0006-broadcast-outputs-wingame-ui-redesign.md
```

## External System Paths

```
TEKNOPARROT UI (launcher):              C:\Users\robon\Desktop\TPBootstrapper\
TEKNOPARROT GameProfiles (stock):       C:\Users\robon\Desktop\TPBootstrapper\GameProfiles\
TEKNOPARROT UserProfiles (override):    C:\Users\robon\Desktop\TPBootstrapper\UserProfiles\
TEKNOPARROT GameProfiles (source):      E:\Projects\TeknoParrotUI\TeknoParrotUi.Common\GameProfiles\
OUTPUTHOOKER source:                    E:\Projects\OutputHooker\
OUTPUTHOOKER binary:                    E:\Projects\OutputHooker\build\Release\OutputHooker.exe

Game root directories:                  E:\Games-Roms\Tekno\<Game Name>\
  Sonic Dash Extreme:                   E:\Games-Roms\Tekno\Sonic Dash Extreme (2015)[Sega Nu][TP]\
  Frogger:                              E:\Games-Roms\Tekno\Frogger (1.38)(2013-08-30)(China)[Raw Thrills PC][TP]\
  Ghostbusters:                         E:\Games-Roms\Tekno\Ghostbusters (1.17)(2019-02-05)[ICE-RT Linux PC][TP]\
```

**CRITICAL:** XML profiles must be edited in BOTH source and runtime GameProfiles directories.

---

## Major Systems

### 1. Game Detection System (`dllmain.cpp`)
- **Entry point:** `DllMain(DLL_PROCESS_ATTACH)` → `CreateThread(OutputsLoop)`
- **Detection flow:**
  1. Sleep 2500ms for game init
  2. Copy first 0x400 bytes of PE header, zero ImageBase for ASLR normalization
  3. Compute CRC32 → match against known CRC values in switch statement
  4. If matched → instantiate game class → call `OutputsGameLoop()`
  5. If not matched → fallback to fixed-address memory probing (Lindbergh games)
  6. If no game found → debug message `New CRC: XXXXXXXX not implemented`

### 2. Game Handler System (`Game Files/`)
- Each game has a `.h`/`.cpp` pair implementing the `Game` interface.
- `OutputsGameLoop()` pattern: init outputs, spawn polling thread with message loop.
- Polling reads memory at game-specific offsets and maps to EOutputs enum values.

### 3. Output System (`Output Files/`)
- `COutputs` class manages 236+ named output signals.
- Three output backends: `CWinOutputs`, `CNetOutputs`, `CBroadcastOutputs`
- `CBroadcastOutputs` wraps both WinOutputs and NetOutputs — used when `OutputsSystem=1`
- Defaults: `CWinOutputs` when `OutputsSystem=0` or INI not found

### 4. WinGame Arcade Display (`win-game/`)
- Tauri 2.11.3 app with Rust backend + Vite frontend
- TCP client connects to OutputBlaster on port 37520
- Custom `wingame://` protocol serves embedded HTML
- Self-contained binary (frontend embedded via build.rs)
- LED layout: Billboard triangle, Woofer speakers, Side LEDs, Item LEDs, Misc box
- Modular per-game stat boxes (STAT_CONFIGS lookup in main.js)
- Coin/start button lighting based on live output values

### 5. Build System (`premake5.lua`)
- Generates Visual Studio project files
- `premake5.bat` → `MSBuild /p:PlatformToolset=v145`
- Static runtime, C++17, Unicode
- MinHook linked per platform/config

### 6. TeknoParrot Integration (`C:\Users\robon\Desktop\TPBootstrapper\`)
- Writes `Enable Outputs=1` to `teknoparrot.ini` from XML profile
- Launches game process via LLHook/StartEx
- Loads `OutputBlaster.dll` from game root dir when `Enable Outputs=1`
- XML profiles in `GameProfiles\` define ConfigValues (including Enable Outputs)
- Source profiles at `E:\Projects\TeknoParrotUI\TeknoParrotUi.Common\GameProfiles\`
- User-saved overrides in `UserProfiles\` — these take priority over GameProfiles
- **Must sync XML changes to ALL THREE: source stock, runtime stock, runtime user**

---

## Key Source-of-Truth Relationships

| File | Source of Truth For |
|------|-------------------|
| `dllmain.cpp` | Game detection CRC table, DLL lifecycle |
| `Common Files/Game.h` | Base Game interface, memory helpers, globals |
| `Output Files/Outputs.h` | Output signal enum (must stay in sync with Outputs.cpp) |
| `Output Files/Outputs.cpp` | Output name strings (must match enum order) |
| `Output Files/BroadcastOutputs.h/.cpp` | Dual-backend output dispatch |
| `win-game/src-tauri/src/lib.rs` | WinGame backend: TCP client, commands, state |
| `win-game/src/main.js` | WinGame frontend: LED rendering, polling, UI |
| `win-game/src-tauri/build.rs` | Frontend build + embedding pipeline |
| `docs/MASTER_MAP.md` | Complete system reference + cheat sheet |
| `premake5.lua` | Build configuration, platform settings, dependencies |
| `C:\Users\robon\Desktop\TPBootstrapper\GameProfiles\*.xml` | TeknoParrot runtime stock game config (sync from source) |
| `C:\Users\robon\Desktop\TPBootstrapper\UserProfiles\*.xml` | TeknoParrot runtime user overrides (sync from source, overrides stock) |
| `E:\Projects\TeknoParrotUI\TeknoParrotUi.Common\GameProfiles\*.xml` | TeknoParrot XML profile source of truth |

---

## Current Active Development

- **Target games:** Sonic Dash Extreme (working), Frogger (CRC placeholder), Ghostbusters (CRC placeholder)
- **Status:** Sonic fully functional (23 outputs). Frogger + Ghostbusters need CRC confirmation from user.
- **Ports:** TCP 37520, UDP broadcast 37521
- **Dual output:** Both OutputHooker (WinMsg) and WinGame (TCP) receive data simultaneously
- **WinGame features:** Billboard triangle, Woofer glow, Side/Item LEDs, modular per-game stat boxes, coin/start button lighting

## Remaining Work

- [ ] Run Frogger → capture real CRC from DebugView → update FROGGER_CRC in dllmain.cpp
- [ ] Run Ghostbusters → capture real CRC → update GHOSTBUSTERS_CRC
- [ ] Confirm Ghostbusters memory offsets (ghosts separate from score, tickets, P2 coins, shots)
