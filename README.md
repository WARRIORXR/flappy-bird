# 🐦 Flappy Bird — Rust Edition

A feature-rich Flappy Bird game built entirely in **Rust** using the [macroquad](https://github.com/not-fl3/macroquad) game framework. No external assets or sprite sheets required — all graphics, animations, particles, and sound effects are procedurally drawn and synthesized at runtime!

![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-blue)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS%20%7C%20WASM-green)

---

## 🚀 Quick Start

```bash
# Clone the repository
git clone https://github.com/WARRIORXR/flappy-bird.git
cd flappy-bird/flappy-bird-rust

# Run the game
cargo run --release
```

> **Prerequisites:** Rust 1.70+ and Cargo. Install via [rustup.rs](https://rustup.rs).

---

## 🎮 Controls

| Key | Action |
|-----|--------|
| `Space` / `↑` / `Left Click` | Flap / Start / Restart |
| `P` | Pause / Resume |
| `M` | Mute / Unmute Audio |
| `D` | Cycle Difficulty (`Easy`, `Normal`, `Hard`) on start screen |
| `C` | Cycle Bird Skin (`Classic`, `Blue`, `Red`, `Green`, `Purple`) on start screen |
| `W` | Cycle Weather (`Clear`, `Rain`, `Snow`, `Sakura`, `Firefly`) on start screen |
| `F` | Toggle FPS overlay |
| `R` | Quick restart (Game Over screen) |

---

## ✨ Features

### ⚡ Power-Ups System
- 🛡️ **Energy Shield** — Absorbs one fatal collision! Breaks with a shockwave burst, camera shake, and grants 1.2s of mercy invulnerability so you can recover.
- 🧲 **Coin Magnet** — Pulls all coins within range directly toward the bird with magnetic physics for 8 seconds.
- ⏳ **Slow-Mo (Time Warp)** — Dilation effect that slows pipe speed and world scrolling by 40% for 6 seconds, complete with a cyan border vignette.
- 🔥 **Double Score (2X)** — Doubles all points earned from pipes and coins for 10 seconds! Features an orange fire aura and border vignette.

### 🌧️ Dynamic Weather System
- ☀️ **Clear Sky** — Classic bright skies.
- 🌧️ **Rain** — Angled rain streaks drifting across the screen.
- ❄️ **Snow** — Gentle swirling snowflakes floating down.
- 🌸 **Sakura** — Drifting, rotating cherry blossom petals.
- 🪲 **Firefly** — Glowing fireflies that float and bob with pulsing bioluminescent glow effects.
- *Weather transitions automatically at score milestones (20, 40, 60, 80) or can be toggled manually with `W`.*

### 🏆 Achievement System
10 unlockable achievements that persist across sessions:
| Achievement | Requirement |
|---|---|
| 🪽 **First Flight** | Score your first point |
| 💰 **Coin Collector** | Collect 10 coins in one game |
| 🛡️ **Shield Master** | Use 3 shields in one game |
| 💪 **Survivor** | Score 25 points |
| 🏎️ **Speed Demon** | Score 50 on Hard difficulty |
| 🎯 **Perfectionist** | Achieve a 20x combo |
| 💯 **Centurion** | Score 100 points |
| 👑 **Coin King** | Collect 50 lifetime coins |
| 🎖️ **Veteran** | Play 50 games |
| 🌪️ **Weather Warrior** | Score 30 in non-Clear weather |

### 🪙 Economy & Scoring
- 🪙 **Coin Collection** — Gold coins spawn inside pipe gaps with bobbing and shimmer animations; each coin gives 2 bonus points (4 with 2X Score).
- 🔥 **Combo System** — Consecutive pipes passed without dying build combo multipliers (`x3`, `x5`, `x10+`) with escalating HUD colors.
- 📊 **Difficulty Bonus** — Hard mode awards +1 bonus point per pipe passed.
- ⚡ **Score Flash & Milestones** — White screen flash on points scored, plus celebration banners at scores 10, 25, 50, 75, 100 (`NICE!`, `AMAZING!`, `LEGENDARY!`, `INSANE!`, `GOD MODE!`).
- 🏅 **Tiered Medals** — Bronze (10+), Silver (20+), Gold (30+), and Platinum (50+) awarded on the Game Over screen.

### 🎨 Visuals & Bird Customization
- 🐦 **5 Bird Skins** — `Classic` (yellow), `Blue`, `Red`, `Green`, `Purple` with custom body, belly, wing, and outline palettes.
- 💀 **Death Tumble Animation** — On fatal impact, bird pops up, tumbles with rotational spin, displays X-eyes, and plummets off screen before the Game Over menu appears.
- 🌅 **Day / Sunset / Night Cycle** — Sky gradient smoothly transitions from daylight to sunset orange to starry night as you score.
- ✨ **Particle FX & Shockwaves** — Flight trail, explosion debris, score sparkles, and circular expanding shockwaves on shield break / powerup pickup.
- 💨 **Dynamic Speed Lines** — Cinematic horizontal motion streaks appear at high velocities.
- 📳 **Screen Shake** — Camera impact shake on crash and shield break.
- 🔥 **Power-Up Auras** — Each active power-up shows a unique visual aura around the bird (shield bubble, magnet ring, fire hexagon).
- 🌈 **Power-Up Vignettes** — Cyan border for Slow-Mo, orange border for Double Score.

### 🔊 Procedural Audio & Mute
- 🎵 **Synthesized Sound Effects** — Pure in-memory procedural WAV synthesis:
  - Rising frequency sine wave on flap
  - Two-tone melodic chime on score
  - Descending frequency buzz on collision
  - 3-step arpeggio chord on power-up pickup
  - Glass crunch / crack sound on shield break
- 🔇 **Audio Mute Toggle** — Press `M` anytime to mute/unmute audio.

### 📊 Persistence & Polish
- 💾 **High Score Saving** — Automatically persists best score to `highscore.txt`.
- 📈 **Lifetime Statistics** — Total games played, cumulative score, and total coins saved in `stats.txt` and displayed on the start menu.
- 🏆 **Achievement Persistence** — Unlocked achievements saved to `achievements.txt` and persist across sessions.
- 🖥️ **Letterboxed Virtual Resolution** — Sharp 400×700 virtual canvas perfectly centered and scaled to fit any window aspect ratio.
- ⏸️ **Pause & FPS Counter** — Press `P` to pause and `F` to view real-time FPS.

---

## 📁 Project Structure

```
flappy-bird-rust/
├── Cargo.toml           # Dependencies (macroquad 0.4)
├── .gitignore
└── src/
    ├── main.rs           # Game loop, state machine, camera, input, achievement checks
    ├── constants.rs      # Physics and dimension tuning constants
    ├── bird.rs           # Bird physics, skins, invulnerability & death animation
    ├── pipes.rs          # Pipe spawning, vertical oscillation, collision
    ├── powerups.rs       # Shield, Magnet, Slow-Mo, and Double Score power-up logic & auras
    ├── weather.rs        # Rain, Snow, Sakura, Firefly weather simulation & particle render
    ├── coins.rs          # Coin spawning, bobbing animation & magnetic pull
    ├── ground.rs         # Parallax ground strip, chevrons & pebbles
    ├── clouds.rs         # Multi-layered parallax cloud system
    ├── particles.rs      # Trail, explosion, sparkle, and shockwave effects
    ├── background.rs     # Day/night sky gradient + twinkling stars
    ├── audio.rs          # Procedural WAV synthesis & mute management
    ├── achievements.rs   # 10-achievement unlock system with persistence & notification UI
    └── ui.rs             # Start screen, game over, HUD, medals, combo, vignettes
```

---

## 🏗️ Cross-Platform Build

### Desktop (Windows, macOS, Linux)
```bash
cargo build --release
```

### WebAssembly (WASM for Web)
```bash
# Add WebAssembly target
rustup target add wasm32-unknown-unknown

# Build release WASM binary
cargo build --release --target wasm32-unknown-unknown
```

---

## 📜 License

Open source under the [MIT License](LICENSE).
