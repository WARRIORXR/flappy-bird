<div align="center">

# 🐦 Flappy Bird — Rust Edition

**A high-performance, asset-free, feature-rich reimagining of Flappy Bird built entirely in Rust.**

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-dea584.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![Macroquad](https://img.shields.io/badge/Framework-Macroquad%200.4-2575fc.svg?style=for-the-badge)](https://macroquad.rs/)
[![Latest Release](https://img.shields.io/github/v/release/WARRIORXR/flappy-bird?style=for-the-badge&logo=github&color=007ec6)](https://github.com/WARRIORXR/flappy-bird/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux%20%7C%20WASM-34a853.svg?style=for-the-badge)](https://github.com/WARRIORXR/flappy-bird)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=for-the-badge)](https://github.com/WARRIORXR/flappy-bird/pulls)

<p align="center">
  <b>100% Procedural Rendering</b> • <b>Real-Time Audio Synthesis</b> • <b>Dynamic Weather Engine</b><br>
  <b>Persistent Achievements</b> • <b>Power-Up Combat Mechanics</b> • <b>Zero Sprite Sheets or External Audio Assets</b>
</p>

</div>

---

## 📑 Table of Contents

- [Overview & Philosophy](#-overview--philosophy)
- [📸 Game Photos](#-game-photos)
- [📦 Download Executable](#-download-executable)
- [Quick Start](#-quick-start)
- [Controls](#-controls)
- [Deep Gameplay Systems](#-deep-gameplay-systems)
  - [⚡ Power-Up Arsenal](#-power-up-arsenal)
  - [🌧️ Atmospheric Weather Engine](#️-atmospheric-weather-engine)
  - [🏆 Persistent Achievement System](#-persistent-achievement-system)
  - [📊 Difficulty Modes & Scoring Economy](#-difficulty-modes--scoring-economy)
  - [🎨 Procedural Bird Skins](#-procedural-bird-skins)
  - [🌅 Day / Sunset / Night Celestial Cycle](#-day--sunset--night-celestial-cycle)
  - [🔊 In-Memory Procedural Audio Synthesizer](#-in-memory-procedural-audio-synthesizer)
  - [💥 Visual FX & Screen Polish](#-visual-fx--screen-polish)
- [Project Architecture](#-project-architecture)
- [Save Data & Persistence](#-save-data--persistence)
- [Cross-Platform Compilation Guide](#-cross-platform-compilation-guide)
  - [Windows](#windows)
  - [macOS](#macos)
  - [Linux (Ubuntu / Debian)](#linux-ubuntu--debian)
  - [WebAssembly (WASM Browser Build)](#webassembly-wasm-browser-build)
- [Contributing](#-contributing)
- [License](#-license)

---

## 💡 Overview & Philosophy

**Flappy Bird — Rust Edition** is designed around a singular technical philosophy: **absolute self-containment**.

- **Zero Sprite Sheets**: Every element — the bird, flapping wings, beak, blinking eyes, oscillating pipes, layered clouds, parallax ground, chevrons, pebbles, and celestial bodies — is calculated and drawn using geometric primitives and trigonometric functions at runtime.
- **Zero Audio Files**: No `.mp3`, `.wav`, or `.ogg` files are bundled. The game features an integrated software synthesizer that constructs valid RIFF/WAVE byte buffers in memory using sine waves, frequency chirps, arpeggio scales, and noise generators.
- **Sub-Millisecond Execution**: Powered by [macroquad](https://macroquad.rs/) on modern OpenGL/WebGL, delivering locked, responsive 60+ FPS performance with minimal CPU and memory overhead.
- **Virtual Resolution Scaling**: Runs on a crisp, fixed 400×700 virtual canvas that scales and letterboxes cleanly to any window dimensions or display aspect ratio.

---

## 📸 Game Photos

<div align="center">

| Start Screen | Active Gameplay |
|:---:|:---:|
| <img src="screenshots/start_screen.png" width="340" alt="Start Screen with Skin & Difficulty Selection" /> | <img src="screenshots/gameplay_action.png" width="340" alt="Active Gameplay through Oscillating Pipes" /> |
| *Start Screen with Skin, Difficulty & Weather Selector* | *Active Run navigating through Pipe Gaps* |

| Power-Ups Active | Atmospheric Weather |
|:---:|:---:|
| <img src="screenshots/powerups.png" width="340" alt="Energy Shield Bubble & Power-Up Orbs" /> | <img src="screenshots/weather_effects.png" width="340" alt="Rain Weather Particle Simulation" /> |
| *Energy Shield Bubble & Floating Power-Up Orbs* | *Dynamic Rain Particles with Ground Splashes* |

| Game Over Scoreboard | Night Mode Celestial Cycle |
|:---:|:---:|
| <img src="screenshots/game_over.png" width="340" alt="Game Over Scoreboard with Medal Ranks" /> | <img src="screenshots/night_mode.png" width="340" alt="Night Mode with Twinkling Stars & Fireflies" /> |
| *Scoreboard with Medals, Best Scores & Achievements* | *Night Sky Cycle with Twinkling Stars & Moon* |

</div>

---

## 📦 Download Executable

> **Ready to Play?** No Rust compiler or toolchain needed! Download the standalone Windows executable directly:

| Platform | File | Direct Download Link | Size |
|:---|:---|:---:|:---:|
| 🪟 **Windows (x64)** | `flappy-bird-rust.exe` | [📥 **Download Latest .exe**](https://github.com/WARRIORXR/flappy-bird/releases/latest/download/flappy-bird-rust.exe) | ~642 KB |
| 📁 **In-Repo Mirror** | `flappy-bird-rust.exe` | [📥 **flappy-bird-rust.exe**](flappy-bird-rust.exe) | ~642 KB |

*Double-click `flappy-bird-rust.exe` to play instantly!*

---

## 🚀 Quick Start

### Play Immediately (No Installation)
Just download [`flappy-bird-rust.exe`](https://github.com/WARRIORXR/flappy-bird/releases/latest/download/flappy-bird-rust.exe) and double-click to launch!

### Build from Source
Make sure you have the Rust toolchain installed (1.70 or higher). If needed, install via [rustup.rs](https://rustup.rs):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

```bash
# Clone the repository
git clone https://github.com/WARRIORXR/flappy-bird.git
cd flappy-bird/flappy-bird-rust

# Run in optimized release mode
cargo run --release
```

---

## 🎮 Controls

| Key / Input | Context | Action |
|:---:|:---:|:---|
| <kbd>Space</kbd> / <kbd>↑</kbd> / <kbd>Left Click</kbd> | In-Game | **Flap wings** (upward velocity burst) |
| <kbd>Space</kbd> / <kbd>↑</kbd> / <kbd>Left Click</kbd> | Start / Game Over | **Start / Replay game** |
| <kbd>P</kbd> | Playing | **Pause / Unpause game** |
| <kbd>M</kbd> | Any screen | **Toggle audio mute / unmute** |
| <kbd>D</kbd> | Start Screen | **Cycle Difficulty** (`Easy` ➔ `Normal` ➔ `Hard`) |
| <kbd>C</kbd> | Start Screen | **Cycle Bird Skin** (`Classic` ➔ `Blue` ➔ `Red` ➔ `Green` ➔ `Purple`) |
| <kbd>W</kbd> | Start Screen | **Cycle Weather** (`Clear` ➔ `Rain` ➔ `Snow` ➔ `Sakura` ➔ `Firefly`) |
| <kbd>F</kbd> | Any screen | **Toggle real-time FPS overlay** |
| <kbd>R</kbd> | Game Over | **Instant quick restart** |

---

## 🌟 Deep Gameplay Systems

### ⚡ Power-Up Arsenal

Power-ups spawn periodically in the vertical gap between incoming pipes with bobbing physics and a glowing outer halo.

| Power-Up | Symbol | Duration | Color | Aura & Visual Effect | Mechanical Benefit |
|:---|:---:|:---:|:---:|:---|:---|
| **Energy Shield** | `S` | Single Hit | Cyan (`#40E0FF`) | Pulsing concentric forcefield bubble | Absorbs one fatal collision, triggers a shockwave, camera shake, and grants **1.2s mercy invulnerability**. |
| **Coin Magnet** | `M` | 8.0s | Orchid (`#DA70D6`) | Rotating 16-point electric field | Exerts inverse-distance magnetic pull on all onscreen coins, dragging them directly to the bird. |
| **Slow-Mo (Time Warp)** | `T` | 6.0s | Gold (`#FFD700`) | Cyan vignette border | Dilates world time by 40% (0.6x speed), dramatically slowing pipe and obstacle advance for precision maneuvers. |
| **Double Score (2X)** | `2` | 10.0s | OrangeRed (`#FF4500`) | Rotating hexagonal fire aura + warm vignette | **2× Multiplier** on all points earned from passing pipes and collecting coins. |

---

### 🌧️ Atmospheric Weather Engine

The weather system alters atmospheric visuals with drifting particle simulations:

- ☀️ **Clear Sky**: Crisp blue gradients with serene parallax drifting clouds.
- 🌧️ **Rain**: High-speed, slanted rainfall streaks with dynamic ground spatter.
- ❄️ **Snow**: Gentle, meandering snowflakes driven by sinusoidal lateral wind drift.
- 🌸 **Sakura**: Floating Japanese cherry blossom petals with variable rotational torque and wind sway.
- 🪲 **Firefly**: Nighttime bioluminescent orbs drifting in gentle organic paths with breathing dual-layer glow effects.

> 💡 **Dynamic Transitions**: While weather can be manually chosen via <kbd>W</kbd>, it also shifts dynamically as your score progresses: **Score 20 ➔ Rain**, **Score 40 ➔ Snow**, **Score 60 ➔ Sakura**, and **Score 80 ➔ Firefly**!

---

### 🏆 Persistent Achievement System

10 built-in achievements automatically tracked and saved to `achievements.txt`. Unlocking an achievement triggers an animated slide-in golden banner with a trophy icon.

| Icon | Achievement | Objective Requirement | Tracking Mode |
|:---:|:---|:---|:---:|
| 🪽 | **First Flight** | Score your first point | Per-game |
| 💰 | **Coin Collector** | Collect 10 coins in a single run | Per-game |
| 🛡️ | **Shield Master** | Absorb 3 fatal collisions with shields in one run | Per-game |
| 💪 | **Survivor** | Reach an individual score of 25 | Per-game |
| 🏎️ | **Speed Demon** | Score 50 or higher on **Hard** difficulty | Per-game |
| 🎯 | **Perfectionist** | Maintain a 20× flawless pipe combo | Per-game |
| 💯 | **Centurion** | Reach a grand score of 100 points | Per-game |
| 👑 | **Coin King** | Accumulate 50 coins across all lifetime sessions | Lifetime |
| 🎖️ | **Veteran** | Play 50 total games | Lifetime |
| 🌪️ | **Weather Warrior** | Score 30 points in non-Clear weather | Per-game |

---

### 📊 Difficulty Modes & Scoring Economy

| Difficulty | Pipe Gap | Base Speed | Hard Mode Bonus | Target Playstyle |
|:---|:---:|:---:|:---:|:---|
| **Easy** | 180 px | 0.85× | None | Relaxed, generous gap margins for learning physics |
| **Normal** | 150 px | 1.00× | None | Classic arcade balance with progressing challenge |
| **Hard** | 125 px | 1.25× | **+1 bonus point per pipe passed** | Tight clearance, rapid reflexes, high score grinding |

- **Combo Counter**: Passing pipes without collision builds combo streaks (`x3`, `x5`, `x10+`), highlighted with color-coded HUD badges.
- **Coin Value**: Each collected coin awards **2 bonus points** (or **4 points** when Double Score is active).
- **Medal Ranks**:
  - 🥉 **Bronze**: Score 10+
  - 🥈 **Silver**: Score 20+
  - 🥇 **Gold**: Score 30+
  - 💎 **Platinum**: Score 50+

---

### 🎨 Procedural Bird Skins

Cycle skins instantly on the start menu with <kbd>C</kbd>. Each skin alters the bird's primary feathers, belly, wing accents, and outline tone:

| Skin Name | Primary Color | Underbelly | Wing Accent | Outline Tone |
|:---|:---:|:---:|:---:|:---:|
| **Classic** | Goldenrod (`#FFD700`) | Cream (`#FFF3B0`) | Amber (`#F0A500`) | Dark Gold (`#DAA520`) |
| **Blue** | Sky Blue (`#46A0FF`) | Ice Blue (`#AAD2FF`) | Royal Blue (`#2878DC`) | Navy (`#1E5AB4`) |
| **Red** | Crimson (`#E64646`) | Soft Salmon (`#FFAA96`) | Deep Red (`#BE2828`) | Maroon (`#A01E1E`) |
| **Green** | Emerald (`#50C850`) | Mint (`#AAF0AA`) | Forest Green (`#32A032`) | Olive (`#1E781E`) |
| **Purple** | Amethyst (`#AA50DC`) | Lavender (`#D2AAFF`) | Violet (`#8232B4`) | Dark Indigo (`#641E96`) |

---

### 🌅 Day / Sunset / Night Celestial Cycle

The sky environment continuously evaluates score to smoothly morph atmospheric gradients across 14 blended horizon bands:
1. **Day** (Scores 0–14): Vivid cyan skies and soft blue horizon.
2. **Sunset** (Scores 15–26): Fiery orange and warm peach sunset gradients.
3. **Night** (Scores 27–40): Deep cosmic navy featuring 40 procedural twinkling stars.
4. *Repeats every 40 points smoothly.*

---

### 🔊 In-Memory Procedural Audio Synthesizer

The audio engine (`audio.rs`) synthesizes raw PCM audio into standard 22,050 Hz 16-bit mono RIFF/WAVE binary byte sequences at launch:

```
[44-byte RIFF/WAVE Header] ➔ [16-bit PCM Audio Sample Buffer] ➔ macroquad::audio::load_sound_from_bytes
```

- **Flap**: Rapid rising frequency sine wave ($400\text{ Hz} \rightarrow 2400\text{ Hz}$) with linear decay envelope.
- **Score Chime**: Dual-tone musical harmonic ($650\text{ Hz} \rightarrow 920\text{ Hz}$) with lingering ring.
- **Hit**: Descending square buzz ($220\text{ Hz} \rightarrow 50\text{ Hz}$) simulating sudden impact.
- **Power-Up**: Fast 3-step ascending major chord arpeggio ($C_5 \rightarrow E_5 \rightarrow G_5$).
- **Shield Break**: High-frequency harmonic chirp ($800\text{ Hz} \rightarrow 100\text{ Hz}$) mixed with high-entropy white noise crunch.

---

### 💥 Visual FX & Screen Polish

- **Death Tumble**: Bird pops upward with negative velocity, enters uncontrolled rotational spin, draws X-eyes, and plummets beneath the screen.
- **Dynamic Speed Lines**: Cinematic horizontal motion streaks emerge across the screen as pipes exceed baseline speeds.
- **Shockwave Ripples**: Expanding polygon rings on shield breakage and power-up acquisition.
- **Screen Shake**: Decaying random camera jitter on impacts and shield destruction.
- **Vignettes**: Distinct edge overlays signaling active time dilation or score doubling.

---

## 📁 Project Architecture

```
flappy-bird-rust/
├── Cargo.toml            # Project dependencies & crate metadata
├── src/
│   ├── main.rs           # Game loop, state machine, camera matrix, input routing
│   ├── constants.rs      # Physics coefficients, canvas sizing, and layout limits
│   ├── bird.rs           # Kinematics, rotation interpolation, skin palettes, death spin
│   ├── pipes.rs          # Procedural pipe generation, vertical sine oscillation, collision
│   ├── powerups.rs       # Shield, Magnet, SlowMo, DoubleScore mechanics, timers, auras
│   ├── weather.rs        # Rain, Snow, Sakura, Firefly multi-particle atmospheric simulator
│   ├── coins.rs          # Gap coin spawning, bobbing kinematics, inverse-distance magnet
│   ├── achievements.rs   # 10 persistent achievements, disk I/O, animated notification banner
│   ├── ground.rs         # Scrolling parallax ground with procedural grass, chevrons & pebbles
│   ├── clouds.rs         # Multi-layered parallax cloud system with dynamic alpha
│   ├── particles.rs      # Flight trails, explosion bursts, sparkles, shockwave expansion
│   ├── background.rs     # 14-band gradient interpolation & twinkle starfield
│   ├── audio.rs          # Pure mathematical procedural sound synthesizer (WAV header + PCM)
│   └── ui.rs             # Start screen, Game Over scoreboard, medals, HUD badges, vignettes
```

---

## 💾 Save Data & Persistence

All game records are stored locally in the working directory as simple plain-text files:

| File | Content | Format |
|:---|:---|:---|
| `highscore.txt` | Highest score ever achieved | Plain integer (e.g., `42`) |
| `stats.txt` | Lifetime game metrics | Key-value pairs (`games=`, `total_score=`, `total_coins=`) |
| `achievements.txt` | Unlocked achievement identifiers | Newline-delimited list of unlocked achievement names |

---

## 🏗️ Cross-Platform Compilation Guide

### Windows
```bash
cargo build --release
# Executable generated at: target/release/flappy-bird-rust.exe
```

### macOS
```bash
cargo build --release
# Executable generated at: target/release/flappy-bird-rust
```

### Linux (Ubuntu / Debian)
Install standard development and graphics libraries:
```bash
sudo apt-get update
sudo apt-get install -y pkg-config libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev
cargo build --release
```

### WebAssembly (WASM Browser Build)
Compile directly to WebAssembly for seamless in-browser play:
```bash
# 1. Add WASM target
rustup target add wasm32-unknown-unknown

# 2. Build release binary
cargo build --release --target wasm32-unknown-unknown

# 3. Serve via local HTTP server
# (Using python, basic-http-server, or miniserve)
python3 -m http.server 8080
```

---

## 🤝 Contributing

Contributions are warmly welcome! Whether reporting a bug, proposing a new skin, fine-tuning physics, or adding new achievements:

1. **Fork** the repository
2. **Create a branch**: `git checkout -b feature/awesome-feature`
3. **Commit your changes**: `git commit -m "Add awesome feature"`
4. **Push to branch**: `git push origin feature/awesome-feature`
5. **Open a Pull Request**

---

## 📜 License

This project is licensed under the **MIT License** — see the [LICENSE](LICENSE) file for complete details.

<div align="center">
  <sub>Built with ❤️ and 🦀 in Rust using Macroquad.</sub>
</div>
