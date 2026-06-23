# OutputBlaster — Repository Map

> **Last updated:** 2026-06-23
> **Task:** 0001-update-agents-governance-standard

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
├── Common Files/                  # Shared/shared infrastructure
│   ├── Game.h                     # Base Game class + helpers + globals
│   ├── Game.cpp                   # Helpers implementation (memory R/W)
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
│   └── GameOutput.h               # Game metadata struct
│
├── audits/                        # Audit files (governance)
│   └── 2026-06-23-0001-agents-governance-standard-audit.md
│
└── tasks/                         # Task files (governance)
    ├── TASK_INDEX.md              # Task index
    └── 0001-update-agents-governance-standard.md
```

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
- **Two detection modes:** PE header CRC (TeknoParrot games) + fixed-address probing (Lindbergh games)

### 2. Game Handler System (`Game Files/`)

- Each game has a `.h`/`.cpp` pair implementing the `Game` interface.
- `OutputsGameLoop()` pattern: init outputs, spawn polling thread with message loop.
- Polling reads memory at game-specific offsets and maps to EOutputs enum values.
- **Reference implementation:** `SonicDashExtreme.cpp` (most complex, with pointer chain resolution and per-round ticket tracking).

### 3. Output System (`Output Files/`)

- `COutputs` class manages 236+ named output signals.
- Two output backends: `CWinOutputs` (Win32 messages for MAMEHooker) and `CNetOutputs` (TCP/UDP).
- Outputs selected via `OutputsSystem` config value in `OutputBlaster.ini`.

### 4. Build System (`premake5.lua`)

- Generates Visual Studio project files from Lua configuration.
- Two platforms: x86, x64. Two configs: Debug, Release.
- Links MinHook.lib per platform/config.
- Static runtime, C++17, Unicode.

---

## Key Source-of-Truth Relationships

| File | Source of Truth For |
|------|-------------------|
| `dllmain.cpp` | Game detection CRC table, DLL lifecycle |
| `Common Files/Game.h` | Base Game interface, memory helpers, globals |
| `Output Files/Outputs.h` | Output signal enum (must stay in sync with Outputs.cpp) |
| `Output Files/Outputs.cpp` | Output name strings (must match enum order) |
| `premake5.lua` | Build configuration, platform settings, dependencies |
| `PLAN_SonicDashExtreme.md` | Active development plan, memory offsets, verified findings |

---

## Current Active Development

- **Target:** Sonic Dash Extreme (Sega Nu, 2015)
- **Status:** WORKING — ticket counter via pointer chain, jackpot/coins reading correctly
- **Plan file:** `PLAN_SonicDashExtreme.md`
- **Missing:** LED/lamp memory offsets, FFB support, score output
