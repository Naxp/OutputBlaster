# OutputBlaster — Agent Governance

> **Last updated:** 2026-06-23
> **Type:** Permanent repo rulebook
> **Scope:** All AI/dev passes on this repository

---

## Project Overview

OutputBlaster is an open-source C++ DLL (GPL v3) that adds output support (LEDs, FFB, ticket counters, etc.) to arcade games running under TeknoParrot emulation. It uses CRC-based game detection to identify the running game, then polls memory at known offsets and exposes values through MAMEHooker-compatible outputs.

### Key Stack

- **Language:** C++17 (`/std:c++17`)
- **Build:** premake5 → Visual Studio project (`.sln`/`.vcxproj` generated, gitignored)
- **Platform:** Windows, x86/x64 (SharedLib/DLL)
- **Profile:** Release + Debug, StaticRuntime, Unicode character set
- **Dependency:** MinHook (bundled at `Common Files/MinHook/`)
- **SDK:** Windows 10 SDK 10.0.26100.0

---

## 1. Mandatory Pre-Read

Every AI/dev pass **must** start by reading this file (`Agents.md`).

If a task touches a known system, the AI/dev must also read the relevant:
- Task files (`tasks/`)
- Audit files (`audits/`)
- Repo map sections (`Repo_map.md`)
- Source files before editing

---

## 2. Pass Types

Every normal pass is a **combined audit + implementation pass**.

Exceptions are allowed only when the user explicitly requests one of:

| Type | Behavior |
|------|----------|
| **audit-only** | Must not implement changes. Read-only review. |
| **question-only** | Must not edit any files. Answer-only. |
| **mapping-only** | Must update `Repo_map.md` but must not implement feature changes unless explicitly requested. |

---

## 3. Task Tracking

- Every pass must create or update the current task index **before** implementation begins.
- The repo-level task index lives at `tasks/TASK_INDEX.md`.
- Every task, large or small, must have an entry in `tasks/TASK_INDEX.md`.
- Every task must have its own task file in `tasks/`.
- Each task file must contain:
  - task name
  - date
  - goal
  - scope
  - checklist
  - files reviewed
  - files changed
  - audit reference
  - changelog reference
  - completion status
- Completed tasks must have their checklist marked complete before the pass ends.

---

## 4. Audit Tracking

- Audit files belong in `audits/`.
- The repo-level audit index lives at `Audits_index.md`.
- Every audit must be indexed in `Audits_index.md`, even if small.
- Every combined audit + implementation pass must create or update an audit entry.
- Every audit file must include:
  - date
  - task reference
  - files reviewed
  - findings
  - risks
  - decisions
  - implementation notes if applicable
  - freshness status
- Any audit older than **3 days** is stale for implementation proof.
- Stale audits may be used only as guidance, not as evidence that the current code is fresh.
- If an old audit is used, the relevant files must be rechecked before implementation.

---

## 5. Repo Mapping

- Mapping work must be recorded in `Repo_map.md`.
- If a mapping pass is requested, all maps must go into `Repo_map.md`.
- `Repo_map.md` must describe the current repo structure, major systems, important files, and known ownership/source-of-truth relationships.
- If implementation changes affect architecture, file ownership, or source-of-truth relationships, update `Repo_map.md`.

---

## 6. Changelog

- Any file, code, config, documentation, or repo-structure change requires a `changelog.md` entry.
- A pass is not complete unless `changelog.md` is updated.
- Changelog entries must include:
  - date
  - task reference
  - summary
  - files changed
  - reason for change

---

## 7. Git Behavior

- If the repo is Git-controlled, every completed implementation pass must be committed locally.
- The commit must include a clear summary and description.
- Never push.
- The user handles all pushes manually.
- If the working tree has unrelated user changes, do not overwrite them. Document them and only stage files changed by this pass.

---

## 8. End-of-Pass Summary

Every completed pass must end with a concise summary containing:

```text
Before:
- ...

After:
- ...

What changed:
- ...

Why:
- ...

Files reviewed:
- ...

Files changed:
- ...

Audit:
- ...

Task:
- ...

Changelog:
- ...

Git:
- Commit: <hash or "not committed because ...">
- Push: not pushed
```

---

## 9. Project-Specific Rules

### 9.1 Game Detection & CRC

- Game detection uses CRC32 of the PE header (first 0x400 bytes with ImageBase zeroed).
- Each game has a unique CRC case in the `dllmain.cpp` switch statement.
- CRCs are computed in-memory at runtime (not pre-computed from disk).
- To add new game support: compute CRC, create game class, add CRC case.
- New game CRCs should be confirmed via live game output (`New CRC: XXXXXXXX not implemented`).

### 9.2 Game Handler Pattern

Every game class follows this pattern:

```cpp
class GameName : public Game {
public:
    void OutputsGameLoop();
};
```

- `OutputsGameLoop()` initializes outputs, then spawns a polling thread.
- Polling reads memory at known offsets via `helpers->ReadByte/ReadInt32/ReadFloat32/ReadIntPtr`.
- Offsets can be relative (to module base) or absolute.
- Outputs mapped via `Outputs->SetValue(OutputLampName, value)`.
- Configurable polling rate via `Sleep(SleepA)` from INI file.

### 9.3 Output Signals

- 236+ outputs defined in `Output Files/Outputs.h` (EOutputs enum).
- Common categories: Lamps, RGB LEDs, Mechanical, FFB, Generic numeric, Ticket/Coin counters.
- Output name strings in `Output Files/Outputs.cpp` must match enum order.
- New enum values must be added before `NUM_OUTPUTS`.

### 9.4 Memory Access Helpers (Game.h/cpp)

| Helper | Description |
|--------|-------------|
| `ReadByte(offset, isRelative)` | Read 1 byte |
| `ReadInt32(offset, isRelative)` | Read 4-byte integer |
| `ReadFloat32(offset, isRelative)` | Read 4-byte float |
| `ReadIntPtr(offset, isRelative)` | Read pointer-sized value |
| `WriteByte(offset, val, isRelative)` | Write 1 byte |
| When `isRelative=true`: offset + `GetModuleHandle(NULL)` | |
| When `isRelative=false`: absolute virtual address | |

### 9.5 Build Process

```batch
premake5.bat            # Generates VS2017 project files
# Then build from Visual Studio: Release|x86 (or Debug|x86)
```

- Generated `.sln`/`.vcxproj`/`.vcxproj.filters` are gitignored.
- Output DLL goes to `bin/x86/Release/OutputBlaster.dll`.
- Debug build outputs to `OutputDebugStringA` — capture with DebugView.

### 9.6 TeknoParrot Integration

- XML profiles live at the TeknoParrotUI project directory.
- `Enable Outputs=1` must be set in the XML profile for OutputBlaster to load.
- `teknoparrot.ini` is written by TeknoParrotUI from the XML profile.
- OutputBlaster.dll must be placed in the game root directory.
- OutputBlaster.ini in the game root configures polling rate and output system.

### 9.7 Code Style & License

- GPL v3 licensed. All source files must carry the license header.
- C++17 standard. Unicode character set.
- No external dependencies beyond MinHook (bundled) and Windows SDK.
- No comments in generated/agent-written code unless explicitly requested.

### 9.8 File Structure Convention

```
/
├── Common Files/          # Shared: Game.h, Game.cpp, CRCCheck.h, MinHook
├── Game Files/            # Per-game .h/.cpp pairs (41+ games)
├── Output Files/          # Output system: Outputs.h/.cpp, WinOutputs, NetOutputs
├── docs/                  # Project documentation (if applicable)
├── audits/                # Audit files (governance)
├── tasks/                 # Task files (governance)
├── Agents.md              # This file
├── Repo_map.md            # Repo mapping
├── Audits_index.md        # Audit index
├── changelog.md           # Changelog
├── TASK_INDEX.md          # Task index
├── dllmain.cpp            # Entry point + game detection
├── premake5.lua           # Build configuration
├── premake5.bat           # Premake generator launcher
└── PLAN_SonicDashExtreme.md # Active game integration plan
```

### 9.9 Preservation Rule

- Existing repo-specific rules must be preserved.
- Do not remove current project rules, stack rules, security rules, deployment rules, architecture rules, naming rules, or workflow rules unless they are clearly obsolete or directly conflict with this standard.
- If a conflict exists, prefer the stricter rule.
- If unsure, keep both and add a clarification note.
- Do not simplify away important project context.
