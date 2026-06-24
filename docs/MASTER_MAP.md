# OutputBlaster — Master Reference Map

> **Version:** 2.0  
> **Last updated:** 2026-06-24  
> **Purpose:** Complete reference for all outputs, memory maps, architecture, integration patterns, and new-game cheat sheet.

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [System Architecture](#2-system-architecture)
3. [Output Signal Reference (EOutputs)](#3-output-signal-reference-eoutputs)
4. [Sonic Dash Extreme — Complete Reference](#4-sonic-dash-extreme--complete-reference)
5. [Game Detection Reference](#5-game-detection-reference)
6. [OutputBlaster.ini Configuration](#6-outputblasterini-configuration)
7. [How to Add a New Game](#7-how-to-add-a-new-game)
8. [How to Add Extra Outputs to an Existing Game](#8-how-to-add-extra-outputs-to-an-existing-game)
9. [TeknoParrot Integration Guide](#9-teknoparrot-integration-guide)
10. [OutputHooker Architecture & Connection](#10-outputhooker-architecture--connection)
11. [WinGame Arcade Display](#11-wingame-arcade-display)
12. [Memory Access Helpers Reference](#12-memory-access-helpers-reference)
13. [CRC32 Game Detection Table](#13-crc32-game-detection-table)
14. [Complete System Integration Cheat Sheet](#14-complete-system-integration-cheat-sheet)
15. [New Game Addition Checklist](#15-new-game-addition-checklist)
16. [Appendices](#16-appendices)

---

## 1. Project Overview

OutputBlaster is an open-source C++ DLL (GPL v3) that adds output support (LEDs, FFB, ticket counters, coin counters, numeric displays) to arcade games running under TeknoParrot emulation.

### Four-Component System

| Component | Location | Role |
|-----------|----------|------|
| **TeknoParrot** | `C:\Users\robon\Desktop\TPBootstrapper\` | Launches arcade games, writes `Enable Outputs=1` to `teknoparrot.ini`, injects `OutputBlaster.dll` into game process |
| **OutputBlaster** (`OutputBlaster.dll`) | `E:\Projects\OutputBlaster\bin\x86\Release\` + deployed to each `<game_root>\` | C++ DLL injected by TeknoParrot. Reads game memory at CRC-detected offsets, publishes output values via dual backends |
| **OutputHooker** (`OutputHooker.exe`) | `E:\Projects\OutputHooker\build\Release\` | C++ Qt6 app (standalone). Receives outputs via WinMsg protocol, routes to hardware/drivers, provides visual display |
| **WinGame** (`win-game.exe`) | `E:\Projects\OutputBlaster\win-game\src-tauri\target\release\` + deployed to each `<game_root>\` | Rust/Tauri app (standalone). Receives outputs via TCP, renders arcade cabinet display with LEDs, tickets, high scores |

### Key Concepts

- **Game Detection:** CRC32 of the PE header (first 0x400 bytes with ImageBase zeroed) — computed in-memory at runtime
- **Output Mapping:** Each game maps memory bytes to named signals via `Outputs->SetValue(OutputName, value)`
- **Two Output Backends:** WinOutputs (MAMEHooker-compatible Windows messages) or NetOutputs (TCP/UDP on configurable ports)
- **Polling:** Configurable polling rate per game via `Sleep=XX` in `OutputBlaster.ini`
- **OutputHooker:** Receives outputs from either backend and routes to hardware (LED-Wiz, PacDrive, SDL, HID, COM ports)

---

## 2. System Architecture

### Data Flow

```
                    TEKNOPARROT (C:\Users\robon\Desktop\TPBootstrapper\)
                         │
                         │ Launches game via LLHook/StartEx
                         │ Injects OutputBlaster.dll
                         │ Writes EXE dir as CWD
                         ▼
                    ┌──────────────────────────────────┐
                    │     Game Process (e.g.,           │
                    │  SonicDash_R_Ring.exe)            │
                    │                                   │
                    │  Memory at known offsets:         │
                    │   ├── lamp bytes                  │
                    │   ├── ticket counter              │
                    │   ├── coin counters               │
                    │   └── high score / jackpot        │
                    └──────────┬────────────────────────┘
                               │
                               │ OutputBlaster.dll injected by TeknoParrot
                               ▼
              ┌───────────────────────────────────────────┐
              │           OutputBlaster.dll                │
              │                                            │
              │  1. DllMain → CreateThread(OutputsLoop)   │
              │  2. CRC32 detection → switch(game)        │
              │  3. Game::OutputsGameLoop()                │
              │     → Create polling thread               │
              │     → ReadByte/ReadInt32 at offsets       │
              │     → Outputs->SetValue(name, value)      │
              │                                            │
              │  CBroadcastOutputs forwards to BOTH:      │
              │  ┌──────────────┐  ┌──────────────────┐  │
              │  │  CWinOutputs  │  │  CNetOutputs     │  │
              │  │  (WinMsg)     │  │  (TCP 37520 +    │  │
              │  │               │  │   UDP 37521)     │  │
              │  └──────┬───────┘  └────────┬─────────┘  │
              └─────────┼────────────────────┼────────────┘
                        │                    │
                        ▼                    ▼
          ┌──────────────────────┐  ┌──────────────────────────────┐
          │    OutputHooker      │  │     WinGame (win-game.exe)   │
          │   WinMsgModule       │  │                              │
          │   (MAMEHooker        │  │  TCP client on 127.0.0.1:   │
          │    protocol)         │  │  37520                       │
          │                      │  │                              │
          │  Routes to:          │  │  Renders arcade cabinet:    │
          │   ├─ LED-Wiz         │  │   ├─ Billboard triangle     │
          │   ├─ PacDrive        │  │   ├─ Speaker woofers        │
          │   ├─ SDL3 controllers│  │   ├─ Side LEDs L/R          │
          │   ├─ HID devices     │  │   ├─ Item LEDs              │
          │   ├─ COM ports       │  │   ├─ Misc box               │
          │   ├─ TCP/UDP/HTTP    │  │   ├─ Ticket counter + anim  │
          │   └─ Sound effects   │  │   ├─ High score leaderboard │
          └──────────────────────┘  │   ├─ Modular stat boxes     │
                                    │   ├─ Coin button lighting   │
                                    │   ├─ Start button lighting  │
                                    │   └─ Initials modal         │
                                    └──────────────────────────────┘

### Hardware/Direct Output Mapping (INI Files)

OutputHooker maps signals to physical outputs using `.ini` files in `<exedir>/ini/`:

| File | Role |
|------|------|
| `dllmain.cpp` | DLL entry point, game detection CRC switch, lifecycle |
| `Common Files/Game.h` | Base `Game` class, `Helpers` memory R/W, global vars |
| `Common Files/Game.cpp` | Helper implementation (`ReadByte`, `ReadInt32`, etc.) |
| `Common Files/CRCCheck.h` | CRC32 algorithm |
| `Output Files/Outputs.h` | `EOutputs` enum (236+ signal names) + `COutputs` base class |
| `Output Files/Outputs.cpp` | Output value tracking, name↔id mapping |
| `Output Files/WinOutputs.h/.cpp` | Win32 message-based MAMEHooker output backend |
| `Output Files/NetOutputs.h/.cpp` | TCP/UDP network output backend |
| `Game Files/<Game>.h/.cpp` | Per-game handler: memory offsets → output mapping |
| `OutputBlaster.ini` | Config per game root directory |

---

## 3. Output Signal Reference (EOutputs)

### Complete Enum (236 outputs, EOutputs.h)

The enum in `Output Files/Outputs.h` defines every output signal. Each entry has a matching name string in `Output Files/Outputs.cpp` that must stay in order.

#### Lamps (Binary, 0/1)
| # | Enum | Name String | Description |
|---|------|-------------|-------------|
| 0 | `OutputPause` | `"pause"` | Pause state |
| 1 | `OutputLampStart` | `"LampStart"` | Start button lamp |
| 2 | `OutputLampView1` | `"LampView1"` | View 1 button lamp |
| 3 | `OutputLampView2` | `"LampView2"` | View 2 button lamp |
| 4 | `OutputLampView3` | `"LampView3"` | View 3 button lamp |
| 5 | `OutputLampView4` | `"LampView4"` | View 4 button lamp |
| 6 | `OutputLampLeader` | `"LampLeader"` | Leader lamp |
| 7 | `OutputLampLeader2` | `"LampLeader2"` | Leader lamp 2 |
| 8 | `OutputRawDrive` | `"RawDrive"` | Raw drive output |
| 9 | `OutputRawLamps` | `"RawLamps"` | Raw lamp output |
| 10 | `OutputLampAction` | `"LampAction"` | Action button lamp |
| 11 | `OutputLampSelectUp` | `"LampSelectUp"` | Select up lamp |
| 12 | `OutputLampSelectDown` | `"LampSelectDown"` | Select down lamp |
| 13 | `OutputLampSelectLeft` | `"LampSelectLeft"` | Select left lamp |
| 14 | `OutputLampSelectRight` | `"LampSelectRight"` | Select right lamp |
| 15 | `OutputLampHazard` | `"LampHazard"` | Hazard lamp |
| 16 | `OutputLampKey` | `"LampKey"` | Key lamp |
| 17 | `OutputLampOnline` | `"LampOnline"` | Online status lamp |
| 18 | `OutputLampOverrev` | `"LampOverrev"` | Overrev lamp |

#### RGB Lamps (Binary, 0/1 for each channel)
| # | Enum | Name String | Description |
|---|------|-------------|-------------|
| 19 | `OutputLampRed` | `"LampRed"` | Red lamp channel |
| 20 | `OutputLampGreen` | `"LampGreen"` | Green lamp channel |
| 21 | `OutputLampBlue` | `"LampBlue"` | Blue lamp channel |
| 22 | `OutputLampYellow` | `"LampYellow"` | Yellow lamp |
| 23 | `OutputLampCyan` | `"LampCyan"` | Cyan lamp |
| 24 | `OutputLampMagneta` | `"LampMagneta"` | Magenta lamp |
| 25 | `OutputLampWhite` | `"LampWhite"` | White lamp |
| 26 | `OutputLampPatoButtonR` | `"LampPatoButtonR"` | Pato button R |
| 27 | `OutputLampPatoButtonB` | `"LampPatoButtonB"` | Pato button B |
| 28 | `OutputLampPato` | `"LampPato"` | Pato lamp |

#### RGB LED Zones (Binary, 0/1 for each channel)
| # | Enum | Name String | Description |
|---|------|-------------|-------------|
| 29 | `OutputWooferRed` | `"WooferLEDRed"` | Woofer LED red |
| 30 | `OutputWooferGreen` | `"WooferLEDGreen"` | Woofer LED green |
| 31 | `OutputWooferBlue` | `"WooferLEDBlue"` | Woofer LED blue |
| 32 | `OutputRearRed` | `"RearLEDRed"` | Rear LED red |
| 33 | `OutputRearGreen` | `"RearLEDGreen"` | Rear LED green |
| 34 | `OutputRearBlue` | `"RearLEDBlue"` | Rear LED blue |
| 35 | `OutputSideRed` | `"SideLEDRed"` | Side LED red |
| 36 | `OutputSideGreen` | `"SideLEDGreen"` | Side LED green |
| 37 | `OutputSideBlue` | `"SideLEDBlue"` | Side LED blue |
| 38 | `OutputItemRed` | `"ItemLEDRed"` | Item LED red |
| 39 | `OutputItemGreen` | `"ItemLEDGreen"` | Item LED green |
| 40 | `OutputItemBlue` | `"ItemLEDBlue"` | Item LED blue |
| 41 | `OutputDriverLampL` | `"LampDriverLeft"` | Driver lamp left |
| 42 | `OutputDriverLampR` | `"LampDriverRight"` | Driver lamp right |

#### Mechanical Outputs (Binary, 0/1)
| # | Enum | Name String | Description |
|---|------|-------------|-------------|
| 43 | `Output1pKnock` | `"1pKnock"` | 1P knocker |
| 44 | `Output1pMotor` | `"1pMotor"` | 1P motor |
| 45 | `Output2pKnock` | `"2pKnock"` | 2P knocker |
| 46 | `Output2pMotor` | `"2pMotor"` | 2P motor |
| 47 | `Output2pLampStart` | `"2pLampStart"` | 2P start lamp |
| 48 | `OutputVisualChange3D` | `"VisualChange2D/3D"` | 2D/3D visual toggle |
| 49 | `Output1pAirFront` | `"1pAirFront"` | 1P air front |
| 50 | `Output1pAirRear` | `"1pAirRear"` | 1P air rear |
| 51 | `Output2pAirFront` | `"2pAirFront"` | 2P air front |
| 52 | `Output2pAirRear` | `"2pAirRear"` | 2P air rear |

#### Additional Lamps
| # | Enum | Name String | Description |
|---|------|-------------|-------------|
| 53 | `OutputInterruption` | `"LampInterruptionButton"` | Interruption button lamp |
| 54 | `OutputSideLamp` | `"LampSide"` | Side lamp |
| 55 | `OutputSideLamp2` | `"LampSide2"` | Side lamp 2 |
| 56 | `OutputVibration` | `"Vibration"` | Vibration output |
| 57 | `OutputPower` | `"Power"` | Power indicator |
| 58 | `OutputRearCover` | `"LEDRearCover"` | Rear cover LED |
| 59 | `OutputPanelLamp` | `"PanelLamp"` | Panel lamp |
| 60-66 | `OutputSlot1Lamp` → `OutputSlot3Lamp` | `"SlotLamp1"`→`"SlotLamp3"` | Slot lamps |
| 67-73 | `OutputSeat1Lamp` → `OutputSeat7Lamp` | `"SeatLamp1"`→`"SeatLamp7"` | Seat lamps |

#### Billboard/Item RGB (Binary)
| # | Enum | Name String |
|---|------|-------------|
| 74 | `OutputBillboardRed` | `"Billboard Red"` |
| 75 | `OutputBillboardGreen` | `"Billboard Green"` |
| 76 | `OutputBillboardBlue` | `"Billboard Blue"` |
| 77 | `OutputBillboardWhite` | `"Billboard White"` |
| 78 | `OutputItemButton` | `"Item Button"` |
| 79 | `OutputMarioButton` | `"Mario Button"` |
| 80 | `OutputSideWhite` | `"SideLEDWhite"` |

#### Recoil/Holder (Binary)
| # | Enum | Name String |
|---|------|-------------|
| 81 | `Output1pRecoil` | `"1pRecoil"` |
| 82 | `Output1pHolderLamp` | `"1pHolderLamp"` |
| 83 | `Output2pRecoil` | `"2pRecoil"` |
| 84 | `Output2pHolderLamp` | `"2pHolderLamp"` |
| 85 | `OutputBillboardLamp` | `"BillboardLamp"` |

#### Boost/RGB2
| # | Enum | Name String |
|---|------|-------------|
| 86 | `OutputBoost` | `"Boost Lamp"` |
| 87 | `OutputBoostRed` | `"Boost Lamp Red"` |
| 88 | `OutputBoostGreen` | `"Boost Lamp Green"` |
| 89 | `OutputBoostBlue` | `"Boost Lamp Blue"` |
| 90 | `OutputLampRed2` | `"LampRed2"` |
| 91 | `OutputLampGreen2` | `"LampGreen2"` |
| 92 | `OutputLampBlue2` | `"LampBlue2"` |

#### Force Feedback
| # | Enum | Name String |
|---|------|-------------|
| 93 | `OutputFFB` | `"FFB"` |
| 94 | `OutputFFB1` | `"FFB1"` |
| 95 | `OutputFFB2` | `"FFB2"` |
| 96 | `OutputFFB3` | `"FFB3"` |
| 97 | `OutputFFB4` | `"FFB4"` |

#### Shooters/Ammo/Health (Binary)
| # | Enum | Name String |
|---|------|-------------|
| 98 | `OutputAmmo1pA` | `"Ammo1pA"` |
| 99 | `OutputAmmo1pB` | `"Ammo1pB"` |
| 100 | `OutputAmmo2pA` | `"Ammo2pA"` |
| 101 | `OutputAmmo2pB` | `"Ammo2pB"` |
| 102 | `OutputFlame1pBool` | `"Flame1pBool"` |
| 103 | `OutputFlame2pBool` | `"Flame2pBool"` |
| 104 | `OutputHealth1pBool` | `"Health1pBool"` |
| 105 | `OutputHealth2pBool` | `"Health2pBool"` |
| 106 | `OutputShoot1p` | `"Shoot1p"` |
| 107 | `OutputShoot2p` | `"Shoot2p"` |

#### Controller Lamps
| # | Enum | Name String |
|---|------|-------------|
| 108 | `OutputControllerLamp1p` | `"1p Controller Lamp"` |
| 109 | `OutputControllerLamp2p` | `"2p Controller Lamp"` |
| 110 | `OutputBrakeLamp1p` | `"1p Brake Lamp"` |
| 111 | `OutputBrakeLamp2p` | `"2p Brake Lamp"` |

#### Vehicle Emblem/Intake/Base
| # | Enum | Name String |
|---|------|-------------|
| 112-114 | `OutputEmblemRed`→`OutputEmblemBlue` | `"Emblem Lamp Red/Green/Blue"` |
| 115-117 | `OutputIntakeLeft`→`OutputIntakeRight` | `"Intake Left/Center/Right"` |
| 118-121 | `OutputBase0Left`→`OutputBase1Right` | `"Base0/1 Left/Right"` |
| 122-123 | `OutputSeatLeft`→`OutputSeatRight` | `"Seat Left/Right"` |

#### Infinity/Indicators
| # | Enum | Name String |
|---|------|-------------|
| 124-126 | `OutputInfinity1`→`OutputInfinity3` | `"Infinity 1/2/3"` |
| 127-130 | `OutputLeftIndicator1`→`OutputLeftIndicator4` | `"Left Indicator 1-4"` |
| 131-134 | `OutputRightIndicator1`→`OutputRightIndicator4` | `"Right Indicator 1-4"` |

#### Speakers/Fog Lights
| # | Enum | Name String |
|---|------|-------------|
| 135-137 | `OutputSpeaker1`→`OutputSpeaker3` | `"Speaker 1/2/3"` |
| 138-141 | `OutputFogLight1`→`OutputFogLight4` | `"Fog Light 1-4"` |

#### Dials/RPM
| # | Enum | Name String |
|---|------|-------------|
| 142-144 | `OutputLargeDials`→`OutputSmallDialRight` | `"Large Dials"`, `"Small Dial Left/Right"` |
| 145 | `OutputBase` | `"Base"` |
| 146 | `OutputExtra` | `"Extra"` |
| 147 | `OutputDash` | `"Dash"` |
| 148 | `OutputRPM` | `"RPM"` |
| 149-151 | `OutputThrottle1`→`OutputThrottle3` | `"Throttle1/2/3"` |
| 152 | `OutputKeypad` | `"Keypad"` |
| 153 | `OutputSpeedo` | `"Speedo"` |

#### Speedo Digits (0-9 numeric)
| # | Enum | Name String |
|---|------|-------------|
| 154-157 | `OutputDigit1Speedo`→`OutputDigit4Speedo` | `"Digit1-4Speed"` |
| 158-161 | `OutputDigit1Brightness`→`OutputDigit4Brightness` | `"Digit1-4Speed Brightness"` |

#### Speedo Segment Brightness (24 segments)
| # | Enum (162-185) | Name String `"Speedo1-24 Brightness"` |
|---|------|-------------|
| 162-185 | `OutputSpeedo1Brightness`→`OutputSpeedo24Brightness` | Speedo bar graph brightness (0-255) |

#### Boost Segment Brightness (24 segments)
| # | Enum (186-209) | Name String `"Boost1-24 Brightness"` |
|---|------|-------------|
| 186-209 | `OutputBoost1Brightness`→`OutputBoost24Brightness` | Boost bar graph brightness (0-255) |

#### Numeric/Mechanical Counters (0-255, typically uint8)
| # | Enum | Name String | Description |
|---|------|-------------|-------------|
| **210** | `OutputTicketCounter` | `"TicketCounter"` | Per-round ticket count (uint8 from uint32) |
| **211** | `OutputTicketJackpot` | `"TicketJackpot"` | Jackpot/round bonus counter (uint8 from uint32) |
| **212** | `OutputCoin1` | `"Coin1"` | Coin/credit counter 1 (uint8 from uint32) |
| **213** | `OutputCoin2` | `"Coin2"` | Coin/credit counter 2 (uint8 from uint32) |
| **214** | `OutputHighScore` | `"HighScore"` | High score value (uint8 from uint32) |

**Total: 215 outputs** (indices 0-214, `NUM_OUTPUTS` = 215)

### Output Value Semantics

| Type | Range | Semantics |
|------|-------|-----------|
| **Binary (Lamps/LEDs)** | 0 or 1 | Off/On |
| **Numeric (Counters)** | 0-255 | Scaled from larger int (e.g., uint32 → cast to UINT8) |
| **Brightness** | 0-255 | PWM value |
| **FFB** | 0-255 | Force feedback intensity |

---

## 4. Sonic Dash Extreme — Complete Reference

### Game Identity

| Property | Value |
|----------|-------|
| **Platform** | Sega Nu (2015) |
| **Executable** | `SonicDash_R_Ring.exe` |
| **CRC32** | `0xf4b75de0` |
| **Orientation** | Portrait (1080×1920 → rotated 270°) |
| **Emulation Profile** | SegaJvs |
| **Test Menu** | `TestMode_x86.exe` (separate process) |

### Memory Map (All offsets relative to `GetModuleHandle(NULL)`)

#### LED/Lamp Data (Byte-wide bitfields)
```
Base +0x9C4B20 → lampData1 (UINT8)
  Bit 7 (0x80) → OutputLampStart     (Start button lamp)
  Bit 6 (0x40) → OutputLampLeader    (Leader/event indicator)
  Bit 3 (0x08) → OutputLampRed       (Marquee red)
  Bit 2 (0x04) → OutputLampGreen     (Marquee green)
  Bit 1 (0x02) → OutputLampBlue      (Marquee blue)
  Bit 0 (0x01) → OutputItemRed       (Item LED red)

Base +0x9C4B21 → lampData2 (UINT8)
  Bit 3 (0x08) → OutputBillboardRed    (Billboard LED red)
  Bit 2 (0x04) → OutputBillboardGreen  (Billboard LED green)
  Bit 1 (0x02) → OutputBillboardBlue   (Billboard LED blue)

Base +0x9C4B22 → sideData (UINT8)
  Bit 3 (0x08) → OutputSideRed       (Side LED red)
  Bit 2 (0x04) → OutputSideGreen     (Side LED green)
  Bit 1 (0x02) → OutputSideBlue      (Side LED blue)
  Bit 0 (0x01) → OutputItemGreen     (Item LED green)

Base +0x9C4B23 → wooferData (UINT8)
  Bit 3 (0x08) → OutputWooferRed     (Woofer LED red)
  Bit 2 (0x04) → OutputWooferGreen   (Woofer LED green)
  Bit 1 (0x02) → OutputWooferBlue    (Woofer LED blue)
  Bit 0 (0x01) → OutputItemBlue      (Item LED blue)
```

**Visual LED Layout (Arcade Cabinet):**
```
        ┌─────────────────────────┐
        │  BILLBOARD (RGB)        │  ← OutputBillboardRed/Green/Blue
        │  "SONIC DASH EXTREME"   │      (lampData2 bits 3,2,1)
        ├─────────────────────────┤
        │  MARQUEE (RGB)          │  ← OutputLampRed/Green/Blue
        │  (cabinet top panel)    │      (lampData1 bits 3,2,1)
        ├─────────────────────────┤
        │  [START] button lamp    │  ← OutputLampStart (lampData1 bit 7)
        ├─────────────────────────┤
        │                         │
        │   SIDE LEDS (RGB)       │  ← OutputSideRed/Green/Blue
        │   (left/right strips)   │      (sideData bits 3,2,1)
        │                         │
        ├─────────────────────────┤
        │  WOOFER LED (RGB)       │  ← OutputWooferRed/Green/Blue
        │  (subwoofer ring)       │      (wooferData bits 3,2,1)
        ├─────────────────────────┤
        │  ITEM LED (RGB)         │  ← OutputItemRed/Green/Blue
        │  (collectible/item)     │      (lampData1 bit 0, sideData bit 0,
        │                         │       wooferData bit 0)
        └─────────────────────────┘
```

#### Numeric Counter Data (uint32, relative offsets)
```
Base +0x84A070 → [pointer] → +8 → TicketCounter (uint32)
  Value: Monotonically increasing per-round ticket count
  Resets to 0 at round end
  Boss detection: jump ≥ 10 in single poll

Base +0x84A238 → TicketJackpot (uint32)
  Value: Jackpot/bonus counter, increments at round completion

Base +0x78B3B8 → Coin1 (uint32)
  Value: Coin/credit slot 1, decrements per continue

Base +0x7A6788 → Coin2a (uint32)
  Value: Coin/credit slot 2a

Base +0x7A678C → Coin2b (uint32)
  Value: Coin/credit slot 2b
  Note: OutputCoin2 = max(Coin2a, Coin2b)

Base +0x84A678 → HighScore (uint32)
  Value: Current high score
```

### Ticket Tracking Logic

The game handler implements per-round ticket tracking with boss detection:

```
Event Flow:
  Stage0 - Scene0 START         → Round begins (ticket=0)
    Gameplay...                   Ticket climbs: 1→5→8→13→...→27
  BossBattleSystem::Enable()     → Boss reached (ticket jump ≥ 10: 27→43)
  CUIPayOut::FadeIn()            → Round ends (ticket resets to 0)
    ↓
  WriteRoundJson() writes to tickets_outputblaster.json:
  [{"round":1,"tickets":43,"jackpot":5,"boss":true,"time":"..."}]
```

### Debug Output (OutputDebugStringA)

Key debug messages emitted by the handler:
```
OB: [N] ticket=XX jackpot=YY coin1=ZZ coin2a=AA coin2b=BB high=HH
OB: Round N START (ticket=XX)
OB: Boss detected! Ticket jump=+NN
OB: Round N ended: NN tickets (boss=Y/N) -> tickets_outputblaster.json
JVS: <64-byte hexdump> <changed bytes>
```

### Outputs Summary

| Output | Source Memory | Type | Notes |
|--------|--------------|------|-------|
| `OutputLampStart` | `+0x9C4B20` bit 7 | Binary | Start button lamp |
| `OutputLampLeader` | `+0x9C4B20` bit 6 | Binary | Leader indicator |
| `OutputLampRed` | `+0x9C4B20` bit 3 | Binary | Marquee red |
| `OutputLampGreen` | `+0x9C4B20` bit 2 | Binary | Marquee green |
| `OutputLampBlue` | `+0x9C4B20` bit 1 | Binary | Marquee blue |
| `OutputBillboardRed` | `+0x9C4B21` bit 3 | Binary | Billboard red |
| `OutputBillboardGreen` | `+0x9C4B21` bit 2 | Binary | Billboard green |
| `OutputBillboardBlue` | `+0x9C4B21` bit 1 | Binary | Billboard blue |
| `OutputSideRed` | `+0x9C4B22` bit 3 | Binary | Side strip red |
| `OutputSideGreen` | `+0x9C4B22` bit 2 | Binary | Side strip green |
| `OutputSideBlue` | `+0x9C4B22` bit 1 | Binary | Side strip blue |
| `OutputWooferRed` | `+0x9C4B23` bit 3 | Binary | Woofer red |
| `OutputWooferGreen` | `+0x9C4B23` bit 2 | Binary | Woofer green |
| `OutputWooferBlue` | `+0x9C4B23` bit 1 | Binary | Woofer blue |
| `OutputItemRed` | `+0x9C4B20` bit 0 | Binary | Item red |
| `OutputItemGreen` | `+0x9C4B22` bit 0 | Binary | Item green |
| `OutputItemBlue` | `+0x9C4B23` bit 0 | Binary | Item blue |
| `OutputTicketCounter` | `[*+0x84A070]+8` | Numeric | 0-255 (from uint32) |
| `OutputTicketJackpot` | `+0x84A238` | Numeric | 0-255 (from uint32) |
| `OutputCoin1` | `+0x78B3B8` | Numeric | 0-255 (from uint32) |
| `OutputCoin2` | `max(+0x7A6788, +0x7A678C)` | Numeric | 0-255 (from uint32) |
| `OutputHighScore` | `+0x84A678` | Numeric | 0-255 (from uint32) |

**Total outputs used by Sonic Dash Extreme: 22 outputs**

### Game Handler Source Map

| File | Lines | Purpose |
|------|-------|---------|
| `Game Files/SonicDashExtreme.h` | 1-22 | Class declaration |
| `Game Files/SonicDashExtreme.cpp` | 1-286 | Main implementation |
| `SonicDashExtreme.cpp:23-48` | `ResolveTicketPointer()` | Pointer chain resolution for ticket counter |
| `SonicDashExtreme.cpp:50-70` | `JvsInit()` | JVS shared memory map init |
| `SonicDashExtreme.cpp:72-111` | `JvsPoll()` | JVS state polling for debug |
| `SonicDashExtreme.cpp:113-156` | `WriteRoundJson()` | Per-round ticket JSON output |
| `SonicDashExtreme.cpp:158-258` | `WindowsLoop()` | Main polling loop |
| `SonicDashExtreme.cpp:260-267` | `OutputsAreGo()` | Thread wrapper |
| `SonicDashExtreme.cpp:269-286` | `OutputsGameLoop()` | Init + message loop |

### INI Configuration

File: `OutputBlaster.ini` (in game root directory):
```ini
[Settings]
Sleep=200               # Polling interval (ms) — slower rate for this game
MaxScaleOutput=100       # Max scaling for numeric outputs
OutputsSystem=0          # 0=WinOutputs (MAMEHooker), 1=NetOutputs (TCP)
NetOutputsWithLF=0       # 0=\r frame ending, 1=\r\n
NetOutputsTCPPort=8000   # TCP server port
NetOutputsUDPBroadcastPort=8001  # UDP broadcast port
AutoRecoilPulse=0        # Auto recoil pulse enable
PulseRate=300            # Pulse rate (ms)
```

---

## 5. Game Detection Reference

### CRC Algorithm (dllmain.cpp:96-106)

```cpp
// 1. Copy first 0x400 bytes of PE header
memcpy(newCrc, GetModuleHandle(nullptr), 0x400);

// 2. Get PE header offset from DOS header
DWORD pePTR = *(DWORD*)(newCrc + 0x3C);

// 3. Zero ImageBase (x86 offset +0x18-0x1C) for ASLR normalization
*(DWORD*)(newCrc + pePTR + 0x18) = 0x00000000;
*(DWORD*)(newCrc + pePTR + 0x18 + 4) = 0x00000000;

// 4. CRC32 the normalized header
uint32_t newCrcResult = GetCRC32(newCrc, 0x400);
```

**Key insight:** The CRC is computed on the **in-memory** PE header after ImageBase is zeroed, NOT on the on-disk file. This means:
- ASLR doesn't affect the CRC (ImageBase is zeroed)
- TeknoParrot in-memory modifications ARE reflected (which is why the CRC may differ between disk and memory)
- The CRC can only be confirmed by running the game and checking the debug output `New CRC: XXXXXXXX not implemented`

### How to Get a CRC for a New Game

1. Build OutputBlaster with a debug build that outputs `New CRC: XXXXXXXX not implemented` for unknown games
2. Launch the game via TeknoParrot with `Enable Outputs=1`
3. Capture the CRC from DebugView output
4. Add the CRC to the switch statement in `dllmain.cpp`
5. Create the game handler class

### Detection Flow

```
DllMain (DLL_PROCESS_ATTACH)
  → CreateThread(OutputsLoop)
    → Sleep(2500ms) — wait for game to initialize
    → CRC32 detection
      ├── Matched → instantiate game class → OutputsGameLoop()
      └── Not matched
            ├── CRC32 = 0xf3d3f699 → BattlePod launcher (return 0)
            └── Fallback: fixed-address memory probing
                  ├── ReadWithoutCrashing(0x804CA44) == 0x82EED98 → AfterburnerClimax
                  ├── ReadWithoutCrashing(0x804CA44) == 0x454825FF → GhostSquadEvo
                  ├── ReadWithoutCrashing(0x804CA44) == 0x62726F76 → WalkingDead
                  ├── ReadWithoutCrashing(0x804B850) == 0x82642C8 → Outrun2SP
                  ├── ReadWithoutCrashing(0x804B840) == 0x0002A68 → MotoGP
                  ├── ReadWithoutCrashing(0x804B840) == 0x0000012 → DeadHeat
                  ├── ReadWithoutCrashing(0x804A908) == 0x12EE → SRTV
                  ├── ReadWithoutCrashing(0x804CF84) == 0x1B09 → InitialD4
                  ├── ReadWithoutCrashing(0x804D258) == 0x1C5F → InitialD5
                  ├── ReadWithoutCrashing(0x0804E8F8) == 0x08479718 → RTuned
                  ├── ReadWithoutCrashing(0x0832572E) == 0xAAAA03C7 → HOTD4VerA
                  ├── ReadWithoutCrashing(0x08320C69) == 0xAAAA03C7 → HOTD4VerC
                  ├── ReadWithoutCrashing(0x8320C69) == 0xC70000A4 → CruisnBlast
                  ├── ReadWithoutCrashing(0x8320C69) == 0x000004B8 → JurassicPark
                  └── ReadWithoutCrashing(0x8320C69) == 0x81DB3153 → HummerExtreme
```

### Two Detection Modes

| Mode | Method | Used For | Reliability |
|------|--------|----------|-------------|
| **CRC (primary)** | PE header CRC32 | TeknoParrot games (~48 games) | High — unique per game version |
| **Fixed-address (fallback)** | Memory probe at known addresses | Lindbergh games (~14 games) | Medium — depends on fixed memory layout |

---

## 6. OutputBlaster.ini Configuration

### File Location
Placed in **BOTH** the game root directory AND the `exe\` subdirectory (the DLL may load with CWD set to either location).

### Reason for Dual Placement
The DLL uses `settingsFilename = TEXT(".\\OutputBlaster.ini")` which is relative to the current working directory. TeknoParrot may set CWD to either the game root or the `exe/` subdirectory. To guarantee the INI is found, copy it to both:
- `<game_root>\OutputBlaster.ini`
- `<game_root>\exe\OutputBlaster.ini`

### Format

```ini
[Settings]
```

### All Options

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `Sleep` | int | 16 | Polling interval in milliseconds |
| `MaxScaleOutput` | int | 16 | Maximum scale value for numeric outputs |
| `OutputsSystem` | int (0/1) | 0 | 0 = WinOutputs only (OutputHooker), 1 = Broadcast (both WinOutputs + NetOutputs TCP) |
| `NetOutputsWithLF` | int (0/1) | 0 | Use `\r\n` instead of `\r` for frame ending |
| `NetOutputsTCPPort` | int | 37520 | TCP server port |
| `NetOutputsUDPBroadcastPort` | int | 37521 | UDP broadcast announce port |
| `AutoRecoilPulse` | int (0/1) | 0 | Auto-recoil pulse enable |
| `PulseRate` | int | 300 | Pulse rate in milliseconds |
| `AutoLaunchWinGame` | int (0/1) | 1 | Auto-launch win-game.exe when game starts |

### Typical Sonic Dash Extreme INI

```ini
[Settings]
Sleep=200
MaxScaleOutput=100
OutputsSystem=1
NetOutputsWithLF=1
NetOutputsTCPPort=37520
NetOutputsUDPBroadcastPort=37521
AutoLaunchWinGame=1
```

### Critical Notes
- `OutputsSystem=1` now enables **both** WinOutputs (for OutputHooker) AND NetOutputs (for WinGame TCP) via `CBroadcastOutputs`
- If `OutputsSystem=0` or INI is not found, only WinOutputs is used (OutputHooker works, WinGame does not receive data)
- AutoLaunchWinGame search paths (relative to game EXE dir): `win-game.exe`, `..\win-game.exe`, `..\..\win-game.exe`, `..\win-game\src-tauri\target\release\win-game.exe`

---

## 7. How to Add a New Game

### Prerequisites
- Game running under TeknoParrot
- `Enable Outputs=1` in `teknoparrot.ini`
- `OutputBlaster.dll` in game root directory
- DebugView or equivalent to capture CRC on first launch

### Step-by-Step

#### 1. Get the CRC
```bash
# Build OutputBlaster (Release|x86)
# Deploy to game root
# Launch game
# Check DebugView for: "New CRC: XXXXXXXX not implemented"
```

#### 2. Find Memory Offsets
Use Cheat Engine while the game is running:
- **Binary lamps:** Scan for bytes that change when lamps activate/deactivate
- **Numeric values:** Scan for 4-byte values that change (tickets, coins, score)
- **Pointer chains:** For dynamic addresses, use pointer scan in Cheat Engine

#### 3. Create Game Handler Files

**`Game Files/<GameName>.h`:**
```cpp
#pragma once
#include "../Common Files/Game.h"
class GameName : public Game {
public:
    void OutputsGameLoop();
};
```

**`Game Files/<GameName>.cpp`:**
```cpp
#include "GameName.h"

static int WindowsLoop()
{
    // Read game memory at known offsets
    UINT8 data = helpers->ReadByte(0xOFFSET, true);
    UINT32 score = helpers->ReadInt32(0xOFFSET2, true);
    
    // Map to outputs
    Outputs->SetValue(OutputLampStart, !!(data & 0x80));
    Outputs->SetValue(OutputLampLeader, !!(data & 0x40));
    Outputs->SetValue(OutputHighScore, (UINT8)score);
    
    return 0;
}

static DWORD WINAPI OutputsAreGo(LPVOID lpParam)
{
    while (true)
    {
        WindowsLoop();
        Sleep(SleepA);
    }
}

void GameName::OutputsGameLoop()
{
    if (!init)
    {
        Outputs = CreateOutputsFromConfig();
        m_game.name = "Game Display Name";
        Outputs->SetGame(m_game);
        Outputs->Initialize();
        Outputs->Attached();
        CreateThread(NULL, 0, OutputsAreGo, NULL, 0, NULL);
        while (GetMessage(&Msg1, NULL, NULL, 0))
        {
            TranslateMessage(&Msg1);
            DispatchMessage(&Msg1);
        }
        init = true;
    }
}
```

#### 4. Add to dllmain.cpp

```cpp
// Add include (around line 71)
#include "Game Files/NewGame.h"

// Add CRC case (around line 248, before default:)
case 0xXXXXXXXX:  // CRC from step 1
    game = new NewGame;
    break;
```

#### 5. Build & Deploy
```bash
premake5.bat
# Build Release|x86 in Visual Studio
# Copy OutputBlaster.dll to game root
# Create/edit OutputBlaster.ini if needed
```

---

## 8. How to Add Extra Outputs to an Existing Game

### Adding Ticket Counter (if not present)

For games without ticket counter support, follow the Sonic Dash Extreme pattern:

1. **Find ticket memory address** via Cheat Engine:
   - Play a round, scan for ticket count as 4-byte value
   - Use pointer scan to find a stable pointer chain
   - Convert to relative offset from module base

2. **Add new EOutputs enum values** (if needed):
   ```cpp
   // In Output Files/Outputs.h, before NUM_OUTPUTS:
   OutputTicketCounter,
   OutputTicketJackpot,
   OutputCoin1,
   OutputCoin2,
   OutputHighScore,
   NUM_OUTPUTS  // Update count
   ```

3. **Add name strings** in `Output Files/Outputs.cpp`:
   ```cpp
   // Must match enum order, add at same position:
   "TicketCounter",
   "TicketJackpot",
   "Coin1",
   "Coin2",
   "HighScore",
   ```

4. **Read in game handler**:
   ```cpp
   // Pointer chain for ticket counter:
   uintptr_t moduleBase = (uintptr_t)GetModuleHandle(NULL);
   uint32_t* ptrField = (uint32_t*)(moduleBase + 0xXXXXXXXX);
   uint32_t ptrVal = *ptrField;
   uint32_t* ticketAddr = (uint32_t*)(ptrVal + OFFSET);
   uint32_t ticketNow = *ticketAddr;
   
   // Direct read for static offsets:
   uint32_t highScore = helpers->ReadInt32(0xYYYYYYYY, true);
   
   // Set outputs:
   Outputs->SetValue(OutputTicketCounter, (UINT8)ticketNow);
   Outputs->SetValue(OutputHighScore, (UINT8)highScore);
   ```

### Adding Generic Numeric Outputs (lives, ammo, health, etc.)

The existing enum has many unused outputs that can be repurposed:
- `OutputAmmo1pA/B` — ammo counters
- `OutputHealth1pBool` / `OutputHealth2pBool` — health indicators
- `OutputShoot1p` / `OutputShoot2p` — shooting indicators
- `OutputFlame1pBool` / `OutputFlame2pBool` — flame/spin indicators
- `OutputRPM` — numeric RPM value
- `OutputPower` — numeric power value
- `OutputSpeedo` — numeric speed value

For completely new concepts (e.g., "Lives"), add new enum values before `NUM_OUTPUTS`.

### Adding Per-Round Tracking (JSON output)

Copy the pattern from `SonicDashExtreme.cpp`:
- Track round start/end via ticket counter going 0→>0 and >0→0
- Write round summary to `<gameroot>/tickets_outputblaster.json`
- Track boss detection via threshold ticket jumps

---

## 9. TeknoParrot Integration Guide

### TeknoParrot Location

```
TeknoParrot UI:          C:\Users\robon\Desktop\TPBootstrapper\TeknoParrotUi.exe
GameProfiles (XML):      C:\Users\robon\Desktop\TPBootstrapper\GameProfiles\
Source GameProfiles:     E:\Projects\TeknoParrotUI\TeknoParrotUi.Common\GameProfiles\
```

**CRITICAL:** When editing XML profiles, you MUST copy to BOTH locations:
1. `E:\Projects\TeknoParrotUI\TeknoParrotUi.Common\GameProfiles\` (source of truth)
2. `C:\Users\robon\Desktop\TPBootstrapper\GameProfiles\` (runtime)

### How OutputBlaster Gets Loaded

1. **XML Profile** defines `Enable Outputs` field
2. **TeknoParrotUI** writes `Enable Outputs=1` to `teknoparrot.ini`
3. **TeknoParrot.dll** reads the INI, calls `LoadLibrary("OutputBlaster.dll")`
4. **OutputBlaster.dll** `DllMain` fires → game detection → output loop

### Required XML Profile Field

In the game's XML profile at `<CategoryName>General</CategoryName>`:
```xml
<FieldInformation>
    <CategoryName>General</CategoryName>
    <FieldName>Enable Outputs</FieldName>
    <FieldValue>1</FieldValue>
    <FieldType>Bool</FieldType>
    <Hint>Enables loading the outputblaster dll.</Hint>
</FieldInformation>
```

### teknoparrot.ini (after TeknoParrotUI writes it)
```ini
[GlobalHotkeys]
ExitKey=0x1B
PauseKey=0x13
[General]
Input API=DirectInput
Windowed=1
Fullscreen Display Rotation=Disabled
Use Custom Resolution=1
Custom Resolution Width=540
Custom Resolution Height=960
Enable Outputs=1
```

### OutputBlaster.dll Deployment
- Place in game root directory (where `OutputBlaster.ini` lives)
- Also copy to `exe/` subdirectory (TeknoParrot may look there)

---

## 10. OutputHooker Architecture & Connection

### Overview

OutputHooker is a Qt6-based C++ application that receives game output signals and routes them to hardware/drivers.

### Connection Types

| Type | Protocol | OutputBlaster Backend | OutputHooker Module | Port |
|------|----------|----------------------|-------------------|------|
| **WinMsg** | Windows Messages (MAMEHooker protocol) | `CWinOutputs` | `WinMsgModule` | N/A (Window message) |
| **TCP** | Text-based socket | `CNetOutputs` | `TCPSocketModule` | 37520 |

### WinMsg (MAMEHooker) Connection Flow

```
1. OutputHooker starts, creates hidden receiver window
2. Timer every 1000ms: FindWindow(L"MAMEOutput", L"MAMEOutput")
3. OutputBlaster creates MAMEOutput hidden window (when game is attached)
4. OutputHooker finds it → sends OM_MAME_REGISTER_CLIENT (client ID: 2323)
5. OutputBlaster sends all current output states via PostMessage(OM_MAME_UPDATE_STATE)
6. OutputHooker requests output name strings via OM_MAME_GET_ID_STRING
7. OutputBlaster replies via WM_COPYDATA with the name string
8. Subsequent changes arrive as OM_MAME_UPDATE_STATE (wParam=outputId+1, lParam=value)
```

### TCP Connection Flow

```
1. OutputHooker timer every 2000ms: connectToHost(LocalHost, 37520)
2. OutputBlaster TCP server accepts connection
3. OutputBlaster sends: "mame_start = Game Name\r" + current output states
4. Updates arrive as: "SignalName = Value\r"
5. Disconnect → reconnect timer restarts
```

### Signal Data Format

Both backends produce the same logical data:
```
SignalName = Value
```

Where `SignalName` matches the name string from `Outputs.cpp` and `Value` is an integer 0-255.

### Both Output Systems Run Simultaneously

Since Pass 0006, `CBroadcastOutputs` forwards every `SetValue()` call to both `CWinOutputs` (WinMsg) and `CNetOutputs` (TCP) simultaneously when `OutputsSystem=1`. This means:
- **OutputHooker** receives data via WinMsg (no changes needed)
- **WinGame** receives data via TCP on port 37520
- **Both work at the same time** — no more choosing one or the other

### Hardware/Direct Output Mapping (INI Files)

OutputHooker maps signals to physical outputs using `.ini` files in `<exedir>/ini/`:
- **Default:** `ini/default.ini`
- **Per-game:** `ini/<GameName>.ini`

INI commands include:
- `LEDWizSet <id> <pin> <signal>` — LED-Wiz output
- `PacDriveSet <id> <pin> <signal>` — PacDrive/Ultimarc output
- `SDLSetRumble <id> <signal>` — SDL3 controller rumble
- `COMPortWrite <port> <dat>` — Serial COM port
- `USBHIDWrite <player> <vid> <pid> <signal>` — HID device
- `TCPSend <socket>` — TCP command
- `UDPSend <type> <addr> <port>` — UDP command
- `HTTPPost <url> <content>` — HTTP request
- `PlayWav <file>` — Audio playback
- `LaunchApp <exe>` — Application launcher

---

## 11. WinGame Arcade Display

### Overview

WinGame (`win-game.exe`) is a Rust/Tauri 2.11.3 application that connects to OutputBlaster's TCP server (port 37520) and renders a live arcade cabinet display with:

- **Billboard triangle** — color changes per active R/G/B channels
- **Woofer speakers** — 3 speaker shapes (2 small, 1 large center) that glow when active
- **Side LEDs** — Left/right columns with individual Red/Green/Blue dots
- **Item LEDs** — Individual colored dots
- **Lamps** — Start (green), Leader (amber), Red/Green/Blue
- **Misc box** — Auto-detects active outputs not in the layout, shows correct colors
- **Ticket counter** — Large animated display with falling ticket animation on round end
- **High score leaderboard** — Top 10 scores with initials
- **Coins exhausted indicator** — Flashes when coins hit 0
- **Fireworks** — Particle burst on round end

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    win-game.exe                              │
│                                                              │
│  ┌──────────────────────────────────────────────────┐       │
│  │           Rust Backend (lib.rs)                   │       │
│  │                                                    │       │
│  │  TCP client ──── connect 127.0.0.1:37520 ──────►  │       │
│  │                                                    │       │
│  │  State (Mutex):                                    │       │
│  │   ├─ outputs: HashMap<String, String>              │       │
│  │   ├─ scores: Vec<ScoreEntry>                       │       │
│  │   ├─ round_active, last_tickets, coins_inserted   │       │
│  │   ├─ player_initials, high_score                   │       │
│  │   ├─ connected, game_name                          │       │
│  │   └─ logs: ring buffer (512 entries)               │       │
│  │                                                    │       │
│  │  Tauri Commands:                                   │       │
│  │   ├─ get_status → {connected, game_name}          │       │
│  │   ├─ get_outputs → OutputsSnapshot (values + raw)  │       │
│  │   ├─ get_scores / submit_score                     │       │
│  │   ├─ get_initials / set_initials                   │       │
│  │   ├─ round_ended → Option<(score, tickets)>       │       │
│  │   ├─ simulate (inject test data)                   │       │
│  │   ├─ get_logs / close_app                          │       │
│  │   └─ Custom protocol: wingame://localhost/index.html│       │
│  └──────────────────────────────────────────────────┘       │
│                                                              │
│  ┌──────────────────────────────────────────────────┐       │
│  │          Frontend (Vite + vanilla JS)             │       │
│  │                                                    │       │
│  │  polls every 200ms: get_status + get_outputs       │       │
│  │  renders: cabinet frame, LED positions, tickets   │       │
│  │  Debug overlay: F12 toggle, log from get_logs     │       │
│  │  Window: borderless, draggable, minimize/close    │       │
│  │  build.rs embeds dist/index.html as byte array    │       │
│  └──────────────────────────────────────────────────┘       │
└─────────────────────────────────────────────────────────────┘
```

### Build Process

```batch
cd win-game
npm install                    # Install Vite + plugin-singlefile
npm run build                  # Build frontend → dist/index.html
cd src-tauri
cargo build --release          # Build.rs auto-runs npm build + embeds
:: Output: src-tauri/target/release/win-game.exe
```

### TCP Protocol (Match OutputBlaster NetOutputs)

Lines received on port 37520:
```
mame_start = Sonic Dash Extreme
LampStart = 1
Billboard Red = 1
TicketCounter = 42
...
```

### Deployment

- Copy `win-game.exe` to `<game_root>\` for DLL auto-launch (`AutoLaunchWinGame=1`)
- DLL searches relative to game EXE: `win-game.exe`, `..\win-game.exe`, etc.
- Can also be launched manually before the game

---

## 12. Memory Access Helpers Reference

### Reading Memory (from `Common Files/Game.cpp`)

| Function | Signature | Description |
|----------|-----------|-------------|
| `ReadByte` | `UINT8 ReadByte(INT_PTR offset, bool isRelative)` | Read 1 byte |
| `ReadInt32` | `int ReadInt32(INT_PTR offset, bool isRelative)` | Read 4-byte integer |
| `ReadFloat32` | `float ReadFloat32(INT_PTR offset, bool isRelative)` | Read 4-byte float |
| `ReadIntPtr` | `INT_PTR ReadIntPtr(INT_PTR offset, bool isRelative)` | Read pointer-sized value |

### Writing Memory

| Function | Signature | Description |
|----------|-----------|-------------|
| `WriteByte` | `UINT8 WriteByte(INT_PTR offset, UINT8 val, bool isRelative)` | Write 1 byte |
| `WriteFloat32` | `float WriteFloat32(INT_PTR offset, float val, bool isRelative)` | Write 4-byte float |
| `WriteIntPtr` | `INT_PTR WriteIntPtr(INT_PTR offset, INT_PTR val, bool isRelative)` | Write pointer |
| `WriteNop` | `UINT8 WriteNop(INT_PTR offset, bool isRelative)` | Write 0x90 (NOP) |

### isRelative Semantics

| `isRelative` | Address Calculation | Use Case |
|-------------|-------------------|----------|
| `true` | `GetModuleHandle(NULL) + offset` | Offsets within the game EXE module |
| `false` | `offset` used as absolute virtual address | System addresses, heap, or injected DLL regions |

### Pointer Chain Pattern

For counters that use pointer indirection:
```cpp
uintptr_t base = (uintptr_t)GetModuleHandle(NULL);
uint32_t* ptrField = (uint32_t*)(base + 0xINDIRECT_ADDR);
if (*ptrField != 0 && *ptrField != 0xFFFFFFFF) {
    uint32_t* valueAddr = (uint32_t*)(*ptrField + 0xOFFSET);
    uint32_t value = *valueAddr;
}
```

### Safe Reading (Avoiding AVs)

Use `__try/__except` for potentially invalid pointers:
```cpp
__try {
    value = *addr;
} __except (EXCEPTION_EXECUTE_HANDLER) {
    value = 0; // Fallback on access violation
}
```

---

## 13. CRC32 Game Detection Table

### CRC-Matched Games (TeknoParrot, PE header CRC)

| CRC32 | Game Class | Game Name |
|-------|-----------|-----------|
| `0x4904b14d` | OperationGhost | Operation Ghost |
| `0xf26ecfa9` | WackyRaces | Wacky Races |
| `0x1adfb24b` | Machstorm | Mach Storm |
| `0x7787da54` | MarioKartGPDXJP110 | Mario Kart GP DX (JP 1.10) |
| `0x6c9038be` | MarioKart118 | Mario Kart 1.18 |
| `0x533f1a71` | SonicAllStarsRacing | Sonic & Sega All-Stars Racing |
| `0x2c8b0265` | LGI3D | LGI 3D |
| `0xd400a3f5` | LGI | LGI |
| `0x80900efd` | VirtuaTennis4 | Virtua Tennis 4 |
| `0x92b5b16b` | InitialD8 | Initial D 8 |
| `0x4fd57346` | InitialD7 | Initial D 7 |
| `0x08d4bace` | InitialD6Update | Initial D 6 (v1.2) |
| `0x715aaebf` | InitialD6 | Initial D 6 (v1.0) |
| `0x1b61779e` | GTIClubSuperminiFesta | GTI Club Supermini Festa! |
| `0x6844eee1` | ChaseHQ2 | Chase HQ 2 |
| `0x47641574` | SegaRacingClassic | Sega Racing Classic |
| `0xb6e0de95` | DaytonaChampionshipUSANSE | Daytona USA (NSE) |
| `0x5a468d9e` | DaytonaChampionshipUSA | Daytona Championship USA |
| `0xbafaca7b` | BattleGear4Tuned | Battle Gear 4 Tuned |
| `0x97994382` | M2Emulator | M2 Emulator |
| `0xE7BC4D6B` | AliensExtermination | Aliens Extermination |
| `0xdc693790` | Transformers | Transformers (multiple versions) |
| `0x7dcef927` | Transformers | Transformers (alt version) |
| `0x8073dbb9` | Transformers | Transformers (alt version) |
| `0xbd8c984d` | BattleGear4 | Battle Gear 4 |
| `0xed9b5740` | H2Overdrive | H2Overdrive |
| `0xfac8a714` | Cars | Cars (multiple versions) |
| `0x01a76797` | Cars | Cars (alt version) |
| `0x8456EEC1` | DirtyDrivin | Dirty Drivin |
| `0xc484002f` | SRG | Sega Racing Game |
| `0x08f14845` | SR3 | Sega Rally 3 |
| `0xc68bcd2f` | InitialD0V131 | Initial D Zero (v1.31) |
| `0x89da99ee` | InitialD0V211 | Initial D Zero (v2.11) |
| `0xe75a6a44` | WMMT5 | Wangan Midnight Maximum Tune 5 |
| `0xDD61E0BA` | WMMT5DX | WMMT5 DX |
| `0x1BB6F051` | WMMT5DXPlus | WMMT5 DX Plus |
| `0x0761cc11` | WMMT6 | WMMT 6 |
| `0xa447f2ef` | WMMT6R | WMMT 6R |
| `0xbfa0c985` | GRID | GRID |
| `0xdb7c9b6e` | FNFDrift | Fast & Furious: Drift |
| `0x259812d7` | FNFSupercars | Fast & Furious: Supercars |
| `0x790b4172` | CrazyRide | Crazy Ride |
| `0xc205c6Ac` | ArcticThunder | Arctic Thunder |
| `0xf3d3f699` | (none) | BattlePod launcher (ignore) |
| `0x8505c794` | BattlePod | Battle Pod |
| `0x55f66578` | TransformersShadowsRising | Transformers: Shadows Rising |
| `0xf4b75de0` | SonicDashExtreme | **Sonic Dash Extreme** |

### Fixed-Address Games (Lindbergh, memory probing)

| Matching Condition | Game Class | Game Name |
|-------------------|-----------|-----------|
| `0x804CA44 == 0x82EED98` | AfterburnerClimax | After Burner Climax |
| `0x804CA44 == 0x454825FF` | GhostSquadEvo | Ghost Squad Evolution |
| `0x804CA44 == 0x62726F76` | WalkingDead | The Walking Dead |
| `0x804B850 == 0x82642C8` | Outrun2SP | OutRun 2 SP |
| `0x804B840 == 0x0002A68` | MotoGP | MotoGP |
| `0x804B840 == 0x0000012` | DeadHeat | Dead Heat |
| `0x804A908 == 0x12EE` | SRTV | Sega RaceTV |
| `0x804CF84 == 0x1B09` | InitialD4 | Initial D 4 |
| `0x804D258 == 0x1C5F` | InitialD5 | Initial D 5 |
| `0x0804E8F8 == 0x08479718` | RTuned | R-Tuned |
| `0x0832572E == 0xAAAA03C7` | HOTD4VerA | House of the Dead 4 (Ver A) |
| `0x08320C69 == 0xAAAA03C7` | HOTD4VerC | House of the Dead 4 (Ver C) |
| `0x8320C69 == 0xC70000A4` | CruisnBlast | Cruis'n Blast |
| `0x8320C69 == 0x000004B8` | JurassicPark | Jurassic Park |
| `0x8320C69 == 0x81DB3153` | HummerExtreme | Hummer Extreme |

---

## 14. Complete System Integration Cheat Sheet

### How Everything Connects

```
                    TEKNOPARROT
                        │
                        │ Launches game via LLHook/StartEx
                        │ Writes EXE dir as CWD
                        ▼
              ┌─────────────────────┐
              │   Game Process       │
              │   (exe/Game.exe)     │
              │                      │
              │ Loads via:           │
              │  DllMain()           │
              │  → CreateThread()    │
              │  → CRC32 detect     │
              │  → Game::Outputs     │
              │    GameLoop()        │
              └────────┬────────────┘
                       │
                       │ CBroadcastOutputs
                       ├────────────────────────────┐
                       ▼                            ▼
              ┌─────────────────┐    ┌──────────────────────────┐
              │  CWinOutputs    │    │     CNetOutputs          │
              │  (WinMsg)       │    │  TCP server :37520       │
              │                 │    │  UDP broadcast :37521    │
              │  Hidden window  │    └──────────┬───────────────┘
              │  "MAMEOutput"   │               │
              └────────┬────────┘               │
                       │ POSTMSG                │ TCP connect
                       ▼                        ▼
              ┌─────────────────┐    ┌──────────────────────────┐
              │ OutputHooker    │    │      WinGame              │
              │ WinMsgModule    │    │  127.0.0.1:37520         │
              │                 │    │                          │
              │ Finds MAMEOutput│    │ Parses "Name = Value\r\n"│
              │ Register client │    │ Renders arcade display  │
              │ Routes to HW    │    │ Shows LEDs/scores/tix    │
              └─────────────────┘    └──────────────────────────┘
```

### File Placement Map

```
TEKNOPARROT:                             C:\Users\robon\Desktop\TPBootstrapper\
├── TeknoParrotUi.exe                    TeknoParrot UI launcher
└── GameProfiles\                        XML profiles — Must match source!
    ├── SonicDashExtreme.xml             [Has Enable Outputs]
    ├── Frogger.xml                      [MUST have Enable Outputs field]
    ├── Ghostbusters.xml                 [MUST have Enable Outputs field]
    └── ... (30+ more profiles)

OUTPUT HOOKER:                           E:\Projects\OutputHooker\build\Release\
└── OutputHooker.exe                     WinMsg receiver + hardware router

EACH GAME ROOT:                          E:\Games-Roms\Tekno\<Game Name>\
├── OutputBlaster.dll                    [REQUIRED] The OB DLL (TeknoParrot loads this)
├── OutputBlaster.ini                    [REQUIRED] Config (OutputsSystem=1, TCP port, Sleep, etc.)
├── win-game.exe                         [OPTIONAL] WinGame binary (AutoLaunchWinGame=1)
│
└── exe\                                 [if exists]
    ├── <Game.exe>                       [GAME] The actual game EXE
    ├── OutputBlaster.dll                [REQUIRED] Same DLL copy! (CWD may be exe\)
    ├── OutputBlaster.ini                [REQUIRED] Same INI copy! (CWD may be exe\)
    └── dinput8.dll                      [TeknoParrot proxy, already present]
```

**Rule of thumb:** ALWAYS copy `OutputBlaster.dll` AND `OutputBlaster.ini` to BOTH `<game_root>\` and `<game_root>\exe\`.

### Configuration Values (Current Working Setup)

| Setting | Value | Why |
|---------|-------|-----|
| `OutputsSystem` | `1` | Enables Broadcast mode (both OutputHooker + WinGame) |
| `NetOutputsTCPPort` | `37520` | Unique TCP port, no conflicts |
| `NetOutputsUDPBroadcastPort` | `37521` | Unique UDP port |
| `NetOutputsWithLF` | `1` | Use `\r\n` frame ending (required by WinGame parser) |
| `AutoLaunchWinGame` | `1` | DLL auto-launches win-game.exe on game start |
| `Sleep` | `200` | Slower polling for Sonic Dash (adjust per game: 16 for racing/lightgun) |

### Start Order

1. **Launch OutputHooker** (optional — for hardware routing + visual debug) from `E:\Projects\OutputHooker\build\Release\OutputHooker.exe`
2. **Launch TeknoParrot** from `C:\Users\robon\Desktop\TPBootstrapper\TeknoParrotUi.exe` → select game → **Start**
3. **TeknoParrot** injects OutputBlaster.dll into game process
4. **OutputBlaster.dll** detects game by CRC, starts polling thread
5. **NetOutputs** TCP server starts on port 37520
6. **AutoLaunchWinGame**: DLL searches for and launches `win-game.exe`
7. **WinGame** TCP client connects to 127.0.0.1:37520
8. **OutputHooker** finds MAMEOutput window, registers as client
9. **Both** receive live output data simultaneously

### Ports Summary

| Port | Protocol | Purpose | Used By |
|------|----------|---------|---------|
| 37520 | TCP | OutputBlaster output stream | WinGame TCP client |
| 37521 | UDP | OutputBlaster broadcast announce | Network discovery |

---

## 15. New Game Addition Checklist

### Phase 1: Get the CRC

- [ ] Build OutputBlaster (Release|x86)
- [ ] Copy `OutputBlaster.dll` to `<game_root>\` AND `<game_root>\exe\`
- [ ] Copy `OutputBlaster.ini` to both directories (with `OutputsSystem=1`)
- [ ] Add `Enable Outputs=1` to game XML profile in BOTH locations:
  - [ ] Source: `E:\Projects\TeknoParrotUI\TeknoParrotUi.Common\GameProfiles\<Game>.xml`
  - [ ] Runtime: `C:\Users\robon\Desktop\TPBootstrapper\GameProfiles\<Game>.xml`
- [ ] Add `Enable Outputs=1` to `teknoparrot.ini` if game already has one
- [ ] Launch game from TeknoParrot (`C:\Users\robon\Desktop\TPBootstrapper\TeknoParrotUi.exe`)
- [ ] Check DebugView for: `OB: No game match — CRC: XXXXXXXX`
- [ ] Note the CRC hex value

### Phase 2: Find Memory Offsets

- [ ] Open Cheat Engine, attach to game process
- [ ] Find binary lamps: scan for byte values that change with button presses/LED activity
- [ ] Find numeric values: scan for 4-byte int (tickets, coins, score)
- [ ] For pointer-based values: use pointer scan in Cheat Engine
- [ ] Verify each offset is relative to module base (not absolute)

### Phase 3: Create Game Handler

- [ ] Create `Game Files/<GameName>.h` (class inheriting Game, OutputsGameLoop declaration)
- [ ] Create `Game Files/<GameName>.cpp`:
  - [ ] WindowsLoop() polling function
  - [ ] ReadByte/ReadInt32 at known offsets
  - [ ] Outputs->SetValue() mapping
  - [ ] OutputsAreGo thread wrapper
  - [ ] OutputsGameLoop() init with AutoLaunchWinGame()
- [ ] Add `#include` and CRC `case` to `dllmain.cpp`

### Phase 4: Update Outputs (if new signals needed)

- [ ] Add new enum values before `NUM_OUTPUTS` in `Output Files/Outputs.h`
- [ ] Add matching name strings in `Output Files/Outputs.cpp` (same position)
- [ ] Increment `NUM_OUTPUTS` if adding after the last entry

### Phase 5: Build and Deploy

- [ ] Run `premake5.bat` (only if new .cpp/.h files added to Output Files/)
- [ ] Build: `MSBuild OutputBlaster.sln /p:Configuration=Release /p:Platform=Win32 /p:PlatformToolset=v145`
- [ ] Copy DLL to both game root directories
- [ ] Copy/update INI in both directories

### Phase 6: Update WinGame

- [ ] If new outputs should appear in WinGame, add them to:
  - [ ] `lib.rs`: `get_outputs()` command — add to OutputsSnapshot
  - [ ] `main.js`: `updateDisplay()` — add updateLED() call
  - [ ] `index.html`: add container element in the LED grid
- [ ] Rebuild: `cd src-tauri && cargo build --release`
- [ ] Deploy `win-game.exe` to `<game_root>\`

### Phase 7: Verify

- [ ] Launch game via TeknoParrot
- [ ] Check DebugView for CRC match and polling messages
- [ ] Verify OutputHooker receives all mapped outputs
- [ ] Verify WinGame connects and shows live values
- [ ] Test ticket/coin counters update correctly
- [ ] Test round detection (ticket rising/falling)

---

## 16. Appendices

### A. Key File Paths

| Item | Path |
|------|------|
| OutputBlaster source root | `E:\Projects\OutputBlaster\` |
| OutputBlaster DLL entry point | `E:\Projects\OutputBlaster\dllmain.cpp` |
| Game base class | `E:\Projects\OutputBlaster\Common Files\Game.h` |
| Output enum + COutputs class | `E:\Projects\OutputBlaster\Output Files\Outputs.h` |
| Output name strings | `E:\Projects\OutputBlaster\Output Files\Outputs.cpp` |
| WinOutputs (MAMEHooker) | `E:\Projects\OutputBlaster\Output Files\WinOutputs.h/.cpp` |
| NetOutputs (TCP/UDP) | `E:\Projects\OutputBlaster\Output Files\NetOutputs.h/.cpp` |
| Broadcast outputs (dual backend) | `E:\Projects\OutputBlaster\Output Files\BroadcastOutputs.h/.cpp` |
| Sonic Dash Extreme handler | `E:\Projects\OutputBlaster\Game Files\SonicDashExtreme.h/.cpp` |
| Frogger handler | `E:\Projects\OutputBlaster\Game Files\Frogger.h/.cpp` |
| Ghostbusters handler | `E:\Projects\OutputBlaster\Game Files\Ghostbusters.h/.cpp` |
| Build config | `E:\Projects\OutputBlaster\premake5.lua` |
| Governance rules | `E:\Projects\OutputBlaster\Agents.md` |
| Active plan | `E:\Projects\OutputBlaster\PLAN_SonicDashExtreme.md` |
| Master map | `E:\Projects\OutputBlaster\docs\MASTER_MAP.md` |
| **TeknoParrot UI** | **`C:\Users\robon\Desktop\TPBootstrapper\`** |
| **TeknoParrot GameProfiles (runtime)** | **`C:\Users\robon\Desktop\TPBootstrapper\GameProfiles\`** |
| **TeknoParrot GameProfiles (source)** | **`E:\Projects\TeknoParrotUI\TeknoParrotUi.Common\GameProfiles\`** |
| Game directory (Sonic) | `E:\Games-Roms\Tekno\Sonic Dash Extreme (2015)[Sega Nu][TP]\` |
| Game directory (Frogger) | `E:\Games-Roms\Tekno\Frogger (1.38)(2013-08-30)(China)[Raw Thrills PC][TP]\` |
| Game directory (Ghostbusters) | `E:\Games-Roms\Tekno\Ghostbusters (1.17)(2019-02-05)[ICE-RT Linux PC][TP]\` |
| Game executable (Sonic) | `E:\Games-Roms\...\exe\SonicDash_R_Ring.exe` |
| Game executable (Frogger) | `E:\Games-Roms\...\sdaemon.exe` |
| OutputBlaster.ini | `<game_root>\OutputBlaster.ini` AND `<game_root>\exe\OutputBlaster.ini` |
| teknoparrot.ini | `<game_root>\exe\teknoparrot.ini` or `<game_root>\pm\teknoparrot.ini` |
| WinGame source | `E:\Projects\OutputBlaster\win-game\` |
| WinGame backend | `E:\Projects\OutputBlaster\win-game\src-tauri\src\lib.rs` |
| WinGame frontend | `E:\Projects\OutputBlaster\win-game\src\main.js` |
| WinGame styles | `E:\Projects\OutputBlaster\win-game\src\styles.css` |
| WinGame executable | `E:\Projects\OutputBlaster\win-game\src-tauri\target\release\win-game.exe` |
| OutputHooker source root | `E:\Projects\OutputHooker\` |
| OutputHooker executable | `E:\Projects\OutputHooker\build\Release\OutputHooker.exe` |

### B. Build Commands

```batch
:: Generate Visual Studio project
cd E:\Projects\OutputBlaster
premake5.bat

:: Build (in VS or via MSBuild)
msbuild OutputBlaster.sln /p:Configuration=Release /p:Platform=x86

:: Deploy
copy bin\x86\Release\OutputBlaster.dll "<game_root>\"
copy bin\x86\Release\OutputBlaster.dll "<game_root>\exe\"

:: Build WinGame (Tauri app)
cd win-game
npm install
cd src-tauri
cargo build --release
:: Output: src-tauri\target\release\win-game.exe

:: Deploy WinGame
copy src-tauri\target\release\win-game.exe "<game_root>\"
```

### C. OutputHooker Build Commands

```batch
cd E:\Projects\OutputHooker
cmake -G "Visual Studio 18 2026" -A x64 -DCMAKE_TOOLCHAIN_FILE=C:\vcpkg\scripts\buildsystems\vcpkg.cmake -DCMAKE_BUILD_TYPE=Release -B build
cmake --build build --config Release
```

### D. Debug Tools

| Tool | Purpose | How to Use |
|------|---------|------------|
| **DebugView** (`dbgview.exe`) | Capture `OutputDebugStringA` output | Run while game is loaded |
| **Cheat Engine** | Find memory offsets | Attach to game process, scan for value changes |
| **Process Explorer** | Verify DLL is loaded | Check game process → DLL list for OutputBlaster.dll |
| **OutputHooker** | Visual output display (WinMsg) | Launch before game; shows live signal values |
| **WinGame** | Arcade cabinet display (TCP/37520) | Auto-launched by DLL or run manually; F12 for debug log |
| **WinGame Simulator** | Test without game | `python win-game/simulate.py` (TCP server on 37520) |
| **netstat** | Verify TCP server | `netstat -ano | findstr 37520` should show LISTENING |
| **Python RE scripts** | Memory scanning | In game directory: `auto_tickets.py`, `monitor_tickets.py`, etc. |

### E. Standard Output Pattern Reference

Quick reference for common output mapping patterns:

```cpp
// Binary lamp from bitfield
Outputs->SetValue(OutputLampStart, !!(byteVal & 0x80));

// RGB from separate bytes
Outputs->SetValue(OutputSideRed, !!(data & 0x04));
Outputs->SetValue(OutputSideGreen, !!(data & 0x02));
Outputs->SetValue(OutputSideBlue, !!(data & 0x01));

// Numeric counter
Outputs->SetValue(OutputTicketCounter, (UINT8)int32Val);

// FFB from byte
Outputs->SetValue(OutputFFB, ffbByte);
```

### F. Future Enhancement Ideas

| Idea | Description |
|------|-------------|
| **Video simulation app** | Qt/OpenGL app that shows an arcade cabinet with virtual LEDs, ticket dispenser, coin slots — driven by OutputBlaster output signals |
| **Extra outputs for all games** | Add ticket counter, high score, coin counter, lives, and ammo to every game that has them in memory |
| **Configurable output mapping** | INI-based mapping of memory offsets to output signals (no recompile needed) |
| **Shared memory output** | Third output backend using named shared memory for performance |
| **Web dashboard** | Real-time output display via web browser |
| **Output recording/playback** | Record output sequences for testing without the game |
