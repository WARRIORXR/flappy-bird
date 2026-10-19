// ─── Flappy Bird Pro: HTML5 Canvas Game Engine ──────────────────────────────
(function() {
  'use strict';

  const GAME_WIDTH = 400;
  const GAME_HEIGHT = 700;
  const GROUND_HEIGHT = 112;
  const BIRD_WIDTH = 34;
  const BIRD_HEIGHT = 24;

  const canvas = document.getElementById('gameCanvas');
  const ctx = canvas.getContext('2d');

  // Set logical resolution
  canvas.width = GAME_WIDTH;
  canvas.height = GAME_HEIGHT;

  // ─── Difficulty Configurations ──────────────────────────────────────────────
  const DIFFICULTIES = [
    { name: 'Normal', gap: 150, speedMul: 1.0, bonus: 0, color: '#4CAF50' },
    { name: 'Hard', gap: 125, speedMul: 1.25, bonus: 1, color: '#F44336' },
    { name: 'Easy', gap: 180, speedMul: 0.85, bonus: 0, color: '#2196F3' },
  ];
  let currentDiffIndex = 0;

  // ─── Bird Skins ─────────────────────────────────────────────────────────────
  const SKINS = [
    { name: 'Classic', body: '#FFD700', belly: '#FFF3B0', wing: '#F0A500', outline: '#543847', blush: '#FF8A80' },
    { name: 'Blue', body: '#46A0FF', belly: '#AAD2FF', wing: '#2878DC', outline: '#1E5AB4', blush: '#82B1FF' },
    { name: 'Red', body: '#E64646', belly: '#FFAA96', wing: '#BE2828', outline: '#A01E1E', blush: '#FF8A80' },
    { name: 'Green', body: '#50C850', belly: '#AAF0AA', wing: '#32A032', outline: '#1E781E', blush: '#B9F6CA' },
    { name: 'Purple', body: '#AA50DC', belly: '#D2AAFF', wing: '#8232B4', outline: '#641E96', blush: '#EA80FC' },
  ];
  let currentSkinIndex = 0;

  // ─── Weather Types ──────────────────────────────────────────────────────────
  const WEATHERS = ['Sunny', 'Rain', 'Snow', 'Sakura', 'Firefly'];
  let currentWeatherIndex = 0;

  // ─── Day / Night Cycles ─────────────────────────────────────────────────────
  const CYCLES = ['Auto', 'Day', 'Sunset', 'Night', 'Dawn'];
  let currentCycleIndex = 0;

  // ─── Game State Machine ─────────────────────────────────────────────────────
  const STATE = {
    START: 0,
    PLAYING: 1,
    DYING: 2,
    GAMEOVER: 3,
  };
  let currentState = STATE.START;

  // ─── Game Variables ─────────────────────────────────────────────────────────
  let score = 0;
  let coins = 0;
  let pipesPassed = 0;
  let combo = 0;
  let maxCombo = 0;
  let closeShaves = 0;
  let gameStartTime = 0;
  let gameDuration = 0;
  let bestScore = parseInt(localStorage.getItem('flappy_best_score') || '0', 10);
  let isPaused = false;
  let lastScoreRecord = null;

  // ─── Camera Shake & Speed Lines ─────────────────────────────────────────────
  let screenShake = 0;
  let speedLines = [];

  // ─── Popups & Particles ─────────────────────────────────────────────────────
  let popups = [];
  let particles = [];
  let weatherParticles = [];
  let stars = [];

  // ─── Bird Entity ────────────────────────────────────────────────────────────
  const bird = {
    x: 80,
    y: 260,
    velY: 0,
    rotation: 0,
    wingAngle: 0,
    wingDir: 1,
    jumpTimer: 0,
    alive: true,
    tumbleSpeed: 0,

    reset() {
      this.x = 80;
      this.y = 260;
      this.velY = 0;
      this.rotation = 0;
      this.wingAngle = 0;
      this.alive = true;
      this.tumbleSpeed = 0;
    },

    jump() {
      if (!this.alive) return;
      this.velY = -320;
      this.wingAngle = -0.5;
      window.soundEngine.playFlap();

      // Mobile Haptic vibration
      if (navigator.vibrate) {
        navigator.vibrate(15);
      }

      // Feather puff particles
      for (let i = 0; i < 3; i++) {
        particles.push({
          x: this.x + 5,
          y: this.y + 15,
          vx: -40 - Math.random() * 40,
          vy: (Math.random() - 0.5) * 50,
          life: 0.35,
          maxLife: 0.35,
          color: SKINS[currentSkinIndex].belly,
          size: 3 + Math.random() * 2,
        });
      }
    },

    update(dt) {
      if (currentState === STATE.START) {
        // Gentle bobbing on start screen
        this.y = 260 + Math.sin(Date.now() / 250) * 8;
        this.rotation = Math.sin(Date.now() / 250) * 0.08;
        this.wingAngle += dt * 8 * this.wingDir;
        if (Math.abs(this.wingAngle) > 0.4) this.wingDir *= -1;
        return;
      }

      if (currentState === STATE.PLAYING) {
        this.velY += 980 * dt;
        if (this.velY > 520) this.velY = 520;
        this.y += this.velY * dt;

        // Rotation interpolation
        const targetRot = Math.min(Math.PI / 2.2, Math.max(-0.45, this.velY * 0.0028));
        this.rotation += (targetRot - this.rotation) * dt * 10;

        // Wing flapping
        if (this.velY < 0) {
          this.wingAngle += dt * 18 * this.wingDir;
          if (Math.abs(this.wingAngle) > 0.6) this.wingDir *= -1;
        } else {
          this.wingAngle += (0 - this.wingAngle) * dt * 8;
        }
      } else if (currentState === STATE.DYING) {
        // Dramatic death spin
        this.velY += 1200 * dt;
        this.y += this.velY * dt;
        this.rotation += this.tumbleSpeed * dt;
      }
    },

    draw() {
      ctx.save();
      ctx.translate(this.x + BIRD_WIDTH / 2, this.y + BIRD_HEIGHT / 2);
      ctx.rotate(this.rotation);

      const skin = SKINS[currentSkinIndex];

      // Draw Shield Aura if active
      if (powerUps.shield.active) {
        const pulse = 1 + Math.sin(Date.now() / 100) * 0.08;
        ctx.beginPath();
        ctx.arc(0, 0, 24 * pulse, 0, Math.PI * 2);
        ctx.fillStyle = 'rgba(70, 160, 255, 0.25)';
        ctx.fill();
        ctx.lineWidth = 3;
        ctx.strokeStyle = '#46A0FF';
        ctx.stroke();
      }

      // Draw Magnet Aura if active
      if (powerUps.magnet.active) {
        ctx.beginPath();
        ctx.arc(0, 0, 28, 0, Math.PI * 2);
        ctx.strokeStyle = 'rgba(255, 215, 0, 0.4)';
        ctx.lineWidth = 2;
        ctx.setLineDash([4, 4]);
        ctx.stroke();
        ctx.setLineDash([]);
      }

      // Bird Body Ellipse
      ctx.beginPath();
      ctx.ellipse(0, 0, BIRD_WIDTH / 2, BIRD_HEIGHT / 2, 0, 0, Math.PI * 2);
      ctx.fillStyle = skin.body;
      ctx.fill();
      ctx.lineWidth = 2;
      ctx.strokeStyle = skin.outline;
      ctx.stroke();

      // Belly
      ctx.beginPath();
      ctx.ellipse(-2, 4, BIRD_WIDTH / 3, BIRD_HEIGHT / 3, 0, 0, Math.PI);
      ctx.fillStyle = skin.belly;
      ctx.fill();

      // Wing
      ctx.save();
      ctx.translate(-6, 0);
      ctx.rotate(this.wingAngle);
      ctx.beginPath();
      ctx.ellipse(0, 0, 10, 6, 0, 0, Math.PI * 2);
      ctx.fillStyle = skin.wing;
      ctx.fill();
      ctx.lineWidth = 1.5;
      ctx.strokeStyle = skin.outline;
      ctx.stroke();
      ctx.restore();

      // Eye
      const eyeX = 8;
      const eyeY = -4;
      if (!this.alive) {
        // X-eyes on death
        ctx.lineWidth = 2;
        ctx.strokeStyle = '#222';
        ctx.beginPath();
        ctx.moveTo(eyeX - 4, eyeY - 4);
        ctx.lineTo(eyeX + 4, eyeY + 4);
        ctx.moveTo(eyeX + 4, eyeY - 4);
        ctx.lineTo(eyeX - 4, eyeY + 4);
        ctx.stroke();
      } else {
        // Large retro eye
        ctx.beginPath();
        ctx.ellipse(eyeX, eyeY, 6, 7, 0, 0, Math.PI * 2);
        ctx.fillStyle = '#FFFFFF';
        ctx.fill();
        ctx.lineWidth = 1.5;
        ctx.strokeStyle = skin.outline;
        ctx.stroke();

        // Pupil
        ctx.beginPath();
        ctx.arc(eyeX + 2, eyeY, 2.5, 0, Math.PI * 2);
        ctx.fillStyle = '#000000';
        ctx.fill();

        // Highlight
        ctx.beginPath();
        ctx.arc(eyeX + 3, eyeY - 2, 1, 0, Math.PI * 2);
        ctx.fillStyle = '#FFFFFF';
        ctx.fill();
      }

      // Cheek blush
      ctx.beginPath();
      ctx.arc(4, 3, 3, 0, Math.PI * 2);
      ctx.fillStyle = skin.blush;
      ctx.globalAlpha = 0.6;
      ctx.fill();
      ctx.globalAlpha = 1.0;

      // Beak / Lips
      ctx.beginPath();
      ctx.moveTo(10, -1);
      ctx.lineTo(20, 2);
      ctx.lineTo(10, 5);
      ctx.closePath();
      ctx.fillStyle = '#FF5722';
      ctx.fill();
      ctx.lineWidth = 1.5;
      ctx.strokeStyle = skin.outline;
      ctx.stroke();

      ctx.restore();
    },
  };

  // ─── Power-Up System ────────────────────────────────────────────────────────
  const powerUps = {
    shield: { active: false, timer: 0 },
    magnet: { active: false, timer: 0 },
    slowmo: { active: false, timer: 0 },
    doubleScore: { active: false, timer: 0 },

    items: [],

    reset() {
      this.shield.active = false;
      this.shield.timer = 0;
      this.magnet.active = false;
      this.magnet.timer = 0;
      this.slowmo.active = false;
      this.slowmo.timer = 0;
      this.doubleScore.active = false;
      this.doubleScore.timer = 0;
      this.items = [];
    },

    activate(type) {
      window.soundEngine.playPowerup();
      if (type === 'shield') {
        this.shield.active = true;
        this.shield.timer = 999; // lasts until hit
        addPopup(bird.x, bird.y - 20, '🛡️ SHIELD READY!', '#46A0FF');
      } else if (type === 'magnet') {
        this.magnet.active = true;
        this.magnet.timer = 10.0;
        addPopup(bird.x, bird.y - 20, '🧲 COIN MAGNET!', '#FFD700');
      } else if (type === 'slowmo') {
        this.slowmo.active = true;
        this.slowmo.timer = 8.0;
        addPopup(bird.x, bird.y - 20, '⏱️ SLOW MOTION!', '#00E5FF');
      } else if (type === 'doubleScore') {
        this.doubleScore.active = true;
        this.doubleScore.timer = 12.0;
        addPopup(bird.x, bird.y - 20, '2️⃣ DOUBLE SCORE!', '#FF9100');
      }
    },

    update(dt) {
      if (this.magnet.active) {
        this.magnet.timer -= dt;
        if (this.magnet.timer <= 0) this.magnet.active = false;
      }
      if (this.slowmo.active) {
        this.slowmo.timer -= dt;
        if (this.slowmo.timer <= 0) this.slowmo.active = false;
      }
      if (this.doubleScore.active) {
        this.doubleScore.timer -= dt;
        if (this.doubleScore.timer <= 0) this.doubleScore.active = false;
      }

      // Update floating orbs
      const speed = 140 * DIFFICULTIES[currentDiffIndex].speedMul * (this.slowmo.active ? 0.6 : 1.0);
      for (let i = this.items.length - 1; i >= 0; i--) {
        const item = this.items[i];
        item.x -= speed * dt;
        item.bob += dt * 4;

        // Collision with bird
        const dist = Math.hypot(item.x - (bird.x + BIRD_WIDTH / 2), item.y + Math.sin(item.bob) * 6 - (bird.y + BIRD_HEIGHT / 2));
        if (dist < 26) {
          this.activate(item.type);
          this.items.splice(i, 1);
          continue;
        }

        if (item.x < -40) {
          this.items.splice(i, 1);
        }
      }
    },

    draw() {
      this.items.forEach(item => {
        const y = item.y + Math.sin(item.bob) * 6;
        ctx.save();
        ctx.beginPath();
        ctx.arc(item.x, y, 14, 0, Math.PI * 2);
        ctx.fillStyle = item.color;
        ctx.shadowColor = item.color;
        ctx.shadowBlur = 10;
        ctx.fill();
        ctx.lineWidth = 2;
        ctx.strokeStyle = '#FFFFFF';
        ctx.stroke();
        ctx.shadowBlur = 0;

        // Icon inside orb
        ctx.font = '14px Arial';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText(item.icon, item.x, y);
        ctx.restore();
      });
    },
  };

  // ─── Pipes Manager ──────────────────────────────────────────────────────────
  const pipes = {
    list: [],
    spawnTimer: 0,

    reset() {
      this.list = [];
      this.spawnTimer = 0;
    },

    update(dt) {
      const diff = DIFFICULTIES[currentDiffIndex];
      const timeScale = powerUps.slowmo.active ? 0.6 : 1.0;
      const speed = 140 * diff.speedMul * timeScale;

      this.spawnTimer += dt * timeScale;
      if (this.spawnTimer > 1.6 / diff.speedMul) {
        this.spawnTimer = 0;
        this.spawnPipe();
      }

      for (let i = this.list.length - 1; i >= 0; i--) {
        const p = this.list[i];
        p.x -= speed * dt;

        // Vertical sine oscillation
        if (p.oscillates) {
          p.oscTime += dt * 2;
          p.gapY = p.baseGapY + Math.sin(p.oscTime) * 35;
        }

        // Passed pipe check
        if (!p.passed && p.x + 52 < bird.x) {
          p.passed = true;
          pipesPassed++;
          combo++;
          if (combo > maxCombo) maxCombo = combo;

          let points = 1 + diff.bonus;
          if (p.golden) {
            points = 5;
            window.soundEngine.playGolden();
            addPopup(bird.x, bird.y - 30, '+5 GOLDEN!', '#FFD700');
            // Golden burst particles
            for (let k = 0; k < 12; k++) {
              particles.push({
                x: p.x + 26,
                y: p.gapY,
                vx: (Math.random() - 0.5) * 160,
                vy: (Math.random() - 0.5) * 160,
                life: 0.6,
                maxLife: 0.6,
                color: '#FFD700',
                size: 4 + Math.random() * 3,
              });
            }
          } else {
            window.soundEngine.playScore();
            addPopup(bird.x, bird.y - 20, points > 1 ? `+${points}` : '+1', '#FFFFFF');
          }

          if (powerUps.doubleScore.active) {
            points *= 2;
          }

          score += points;

          // Combo notification
          if (combo >= 5 && combo % 5 === 0) {
            addPopup(GAME_WIDTH / 2, 180, `🔥 ${combo}X COMBO!`, '#FF9800');
          }
        }

        // Close-shave near-miss check (skimming pipe lip within 14px)
        if (!p.closeShaved && Math.abs(p.x + 26 - (bird.x + BIRD_WIDTH / 2)) < 26) {
          const topLipY = p.gapY - p.gap / 2;
          const botLipY = p.gapY + p.gap / 2;
          const birdTop = bird.y;
          const birdBot = bird.y + BIRD_HEIGHT;

          const distTop = Math.abs(birdTop - topLipY);
          const distBot = Math.abs(birdBot - botLipY);

          if ((distTop < 14 || distBot < 14) && birdTop > topLipY - 5 && birdBot < botLipY + 5) {
            p.closeShaved = true;
            closeShaves++;
            score += 2;
            window.soundEngine.playCloseShave();
            addPopup(bird.x, bird.y - 35, '⚡ CLOSE SHAVE! +2', '#00E5FF');

            // Spark particles
            for (let k = 0; k < 8; k++) {
              particles.push({
                x: bird.x + BIRD_WIDTH,
                y: distTop < 14 ? birdTop : birdBot,
                vx: 50 + Math.random() * 80,
                vy: (Math.random() - 0.5) * 80,
                life: 0.4,
                maxLife: 0.4,
                color: '#00E5FF',
                size: 2.5,
              });
            }
          }
        }

        // Magnet attraction for coin
        if (p.hasCoin && !p.coinCollected) {
          const coinX = p.x + 26;
          const coinY = p.gapY;
          if (powerUps.magnet.active) {
            const dist = Math.hypot(coinX - (bird.x + BIRD_WIDTH / 2), coinY - (bird.y + BIRD_HEIGHT / 2));
            if (dist < 140) {
              p.coinOffset = (p.coinOffset || 0) + (bird.x - coinX) * dt * 4;
            }
          }

          // Coin pickup collision
          const bx = bird.x + BIRD_WIDTH / 2;
          const by = bird.y + BIRD_HEIGHT / 2;
          if (Math.hypot(coinX + (p.coinOffset || 0) - bx, coinY - by) < 22) {
            p.coinCollected = true;
            coins++;
            const coinPts = powerUps.doubleScore.active ? 4 : 2;
            score += coinPts;
            window.soundEngine.playCoin();
            addPopup(bx, by - 20, `+${coinPts} 🪙`, '#FFD700');
          }
        }

        if (p.x < -60) {
          this.list.splice(i, 1);
        }
      }
    },

    spawnPipe() {
      const diff = DIFFICULTIES[currentDiffIndex];
      const gap = diff.gap;
      const minGapY = 120 + gap / 2;
      const maxGapY = (GAME_HEIGHT - GROUND_HEIGHT - 120) - gap / 2;
      const gapY = minGapY + Math.random() * (maxGapY - minGapY);

      const isGolden = Math.random() < 0.15; // 15% chance
      const oscillates = score > 15 && Math.random() < 0.35; // moving pipes on higher scores

      this.list.push({
        x: GAME_WIDTH + 20,
        gapY,
        baseGapY: gapY,
        gap,
        golden: isGolden,
        oscillates,
        oscTime: Math.random() * Math.PI,
        passed: false,
        closeShaved: false,
        hasCoin: Math.random() < 0.7,
        coinCollected: false,
        coinOffset: 0,
      });

      // Spawn Powerup Orb occasionally
      if (Math.random() < 0.22 && powerUps.items.length < 2) {
        const types = [
          { type: 'shield', color: '#2979FF', icon: '🛡️' },
          { type: 'magnet', color: '#FFD700', icon: '🧲' },
          { type: 'slowmo', color: '#00E5FF', icon: '⏱️' },
          { type: 'doubleScore', color: '#FF9100', icon: '2️⃣' },
        ];
        const selected = types[Math.floor(Math.random() * types.length)];
        powerUps.items.push({
          x: GAME_WIDTH + 140,
          y: gapY + (Math.random() - 0.5) * 40,
          bob: 0,
          ...selected,
        });
      }
    },

    draw() {
      this.list.forEach(p => {
        const topLipY = p.gapY - p.gap / 2;
        const botLipY = p.gapY + p.gap / 2;

        const mainColor = p.golden ? '#FFD700' : '#73bf2e';
        const darkColor = p.golden ? '#B8860B' : '#558022';
        const lightColor = p.golden ? '#FFF4B8' : '#9ce659';
        const outlineColor = p.golden ? '#8B6508' : '#2d4710';

        // ─── Top Pipe ─────────────────────────────────────────────────────────
        ctx.fillStyle = mainColor;
        ctx.fillRect(p.x, 0, 52, topLipY - 24);

        // Highlight stripe
        ctx.fillStyle = lightColor;
        ctx.fillRect(p.x + 4, 0, 6, topLipY - 24);
        ctx.fillStyle = darkColor;
        ctx.fillRect(p.x + 44, 0, 8, topLipY - 24);

        // Top Pipe Lip
        ctx.fillStyle = mainColor;
        ctx.fillRect(p.x - 3, topLipY - 24, 58, 24);
        ctx.fillStyle = lightColor;
        ctx.fillRect(p.x, topLipY - 24, 6, 24);
        ctx.fillStyle = darkColor;
        ctx.fillRect(p.x + 47, topLipY - 24, 8, 24);

        // Outlines
        ctx.lineWidth = 2;
        ctx.strokeStyle = outlineColor;
        ctx.strokeRect(p.x, 0, 52, topLipY - 24);
        ctx.strokeRect(p.x - 3, topLipY - 24, 58, 24);

        // ─── Bottom Pipe ──────────────────────────────────────────────────────
        const botH = GAME_HEIGHT - GROUND_HEIGHT - botLipY;
        ctx.fillStyle = mainColor;
        ctx.fillRect(p.x, botLipY + 24, 52, botH - 24);

        ctx.fillStyle = lightColor;
        ctx.fillRect(p.x + 4, botLipY + 24, 6, botH - 24);
        ctx.fillStyle = darkColor;
        ctx.fillRect(p.x + 44, botLipY + 24, 8, botH - 24);

        // Bottom Pipe Lip
        ctx.fillStyle = mainColor;
        ctx.fillRect(p.x - 3, botLipY, 58, 24);
        ctx.fillStyle = lightColor;
        ctx.fillRect(p.x, botLipY, 6, 24);
        ctx.fillStyle = darkColor;
        ctx.fillRect(p.x + 47, botLipY, 8, 24);

        ctx.strokeRect(p.x, botLipY + 24, 52, botH - 24);
        ctx.strokeRect(p.x - 3, botLipY, 58, 24);

        // Draw Gap Coin if available
        if (p.hasCoin && !p.coinCollected) {
          const cx = p.x + 26 + (p.coinOffset || 0);
          const cy = p.gapY + Math.sin(Date.now() / 200) * 4;
          ctx.beginPath();
          ctx.ellipse(cx, cy, 9, 12, 0, 0, Math.PI * 2);
          ctx.fillStyle = '#FFD700';
          ctx.fill();
          ctx.lineWidth = 1.5;
          ctx.strokeStyle = '#B8860B';
          ctx.stroke();

          // Coin inner detail
          ctx.beginPath();
          ctx.ellipse(cx, cy, 5, 8, 0, 0, Math.PI * 2);
          ctx.strokeStyle = '#FFF3B0';
          ctx.stroke();
        }
      });
    },
  };

  // ─── Ground & Clouds ────────────────────────────────────────────────────────
  let groundScroll = 0;
  let cloudScroll = 0;

  function drawGround(dt) {
    const timeScale = powerUps.slowmo.active ? 0.6 : 1.0;
    if (currentState === STATE.PLAYING || currentState === STATE.START) {
      groundScroll = (groundScroll + 140 * dt * timeScale) % 24;
      cloudScroll = (cloudScroll + 20 * dt * timeScale) % GAME_WIDTH;
    }

    const groundY = GAME_HEIGHT - GROUND_HEIGHT;

    // Grass Top Bar
    ctx.fillStyle = '#5ee270';
    ctx.fillRect(0, groundY, GAME_WIDTH, 14);
    ctx.fillStyle = '#48b857';
    ctx.fillRect(0, groundY + 14, GAME_WIDTH, 4);

    // Dirt base
    ctx.fillStyle = '#ded895';
    ctx.fillRect(0, groundY + 18, GAME_WIDTH, GROUND_HEIGHT - 18);

    // Diagonal grass chevrons pattern
    ctx.strokeStyle = '#cbd070';
    ctx.lineWidth = 3;
    for (let x = -groundScroll; x < GAME_WIDTH + 24; x += 20) {
      ctx.beginPath();
      ctx.moveTo(x, groundY + 22);
      ctx.lineTo(x - 8, groundY + 36);
      ctx.stroke();
    }

    // Top border line
    ctx.strokeStyle = '#543847';
    ctx.lineWidth = 2.5;
    ctx.beginPath();
    ctx.moveTo(0, groundY);
    ctx.lineTo(GAME_WIDTH, groundY);
    ctx.stroke();
  }

  // ─── Celestial Background (Day / Sunset / Night / Dawn) ────────────────────
  function initStars() {
    stars = [];
    for (let i = 0; i < 50; i++) {
      stars.push({
        x: Math.random() * GAME_WIDTH,
        y: Math.random() * (GAME_HEIGHT - GROUND_HEIGHT - 40),
        size: Math.random() * 2 + 0.8,
        twinkle: Math.random() * Math.PI * 2,
      });
    }
  }
  initStars();

  function drawBackground(dt) {
    let mode = CYCLES[currentCycleIndex];
    if (mode === 'Auto') {
      const cycleIdx = Math.floor((score / 25) % 4);
      mode = ['Day', 'Sunset', 'Night', 'Dawn'][cycleIdx];
    }

    const grad = ctx.createLinearGradient(0, 0, 0, GAME_HEIGHT - GROUND_HEIGHT);

    if (mode === 'Day') {
      grad.addColorStop(0, '#4ec0ca');
      grad.addColorStop(1, '#a3e5eb');
      ctx.fillStyle = grad;
      ctx.fillRect(0, 0, GAME_WIDTH, GAME_HEIGHT);

      // Sun
      ctx.beginPath();
      ctx.arc(330, 90, 32, 0, Math.PI * 2);
      ctx.fillStyle = '#FFF8A0';
      ctx.shadowColor = '#FFF8A0';
      ctx.shadowBlur = 18;
      ctx.fill();
      ctx.shadowBlur = 0;
    } else if (mode === 'Sunset') {
      grad.addColorStop(0, '#2C3E50');
      grad.addColorStop(0.4, '#E74C3C');
      grad.addColorStop(0.8, '#F39C12');
      grad.addColorStop(1, '#F1C40F');
      ctx.fillStyle = grad;
      ctx.fillRect(0, 0, GAME_WIDTH, GAME_HEIGHT);

      // Sinking Sun
      ctx.beginPath();
      ctx.arc(280, 220, 36, 0, Math.PI * 2);
      ctx.fillStyle = '#FF6B6B';
      ctx.fill();
    } else if (mode === 'Night') {
      grad.addColorStop(0, '#0B0C1E');
      grad.addColorStop(0.6, '#181E3B');
      grad.addColorStop(1, '#272F55');
      ctx.fillStyle = grad;
      ctx.fillRect(0, 0, GAME_WIDTH, GAME_HEIGHT);

      // Twinkling stars
      stars.forEach(s => {
        s.twinkle += dt * 3;
        const alpha = 0.4 + Math.sin(s.twinkle) * 0.4;
        ctx.fillStyle = `rgba(255, 255, 255, ${alpha})`;
        ctx.fillRect(s.x, s.y, s.size, s.size);
      });

      // Crescent Moon
      ctx.save();
      ctx.translate(320, 80);
      ctx.beginPath();
      ctx.arc(0, 0, 22, 0, Math.PI * 2);
      ctx.fillStyle = '#FFFDE7';
      ctx.shadowColor = '#FFFDE7';
      ctx.shadowBlur = 12;
      ctx.fill();
      ctx.shadowBlur = 0;

      // Cutout to make crescent
      ctx.beginPath();
      ctx.arc(8, -4, 18, 0, Math.PI * 2);
      ctx.fillStyle = '#0B0C1E';
      ctx.fill();
      ctx.restore();
    } else if (mode === 'Dawn') {
      grad.addColorStop(0, '#3F2B96');
      grad.addColorStop(0.5, '#A8C0FF');
      grad.addColorStop(1, '#FEE140');
      ctx.fillStyle = grad;
      ctx.fillRect(0, 0, GAME_WIDTH, GAME_HEIGHT);
    }

    // Parallax Clouds
    drawParallaxClouds();
  }

  function drawParallaxClouds() {
    const y = 380;
    ctx.fillStyle = 'rgba(255, 255, 255, 0.45)';
    for (let i = 0; i < 3; i++) {
      const cx = (i * 180 - cloudScroll) % (GAME_WIDTH + 180) - 60;
      ctx.beginPath();
      ctx.ellipse(cx, y, 45, 20, 0, 0, Math.PI * 2);
      ctx.ellipse(cx + 25, y - 8, 35, 22, 0, 0, Math.PI * 2);
      ctx.ellipse(cx + 50, y, 40, 18, 0, 0, Math.PI * 2);
      ctx.fill();
    }
  }

  // ─── Dynamic Weather Particle Simulation ───────────────────────────────────
  function initWeather() {
    weatherParticles = [];
    const count = 40;
    for (let i = 0; i < count; i++) {
      weatherParticles.push({
        x: Math.random() * GAME_WIDTH,
        y: Math.random() * GAME_HEIGHT,
        vx: (Math.random() - 0.5) * 30,
        vy: 100 + Math.random() * 150,
        size: Math.random() * 2 + 1,
        angle: Math.random() * Math.PI * 2,
      });
    }
  }
  initWeather();

  function updateAndDrawWeather(dt) {
    const weather = WEATHERS[currentWeatherIndex];
    if (weather === 'Sunny') return;

    ctx.save();
    weatherParticles.forEach(p => {
      if (weather === 'Rain') {
        p.vy = 420;
        p.vx = -60;
        p.x += p.vx * dt;
        p.y += p.vy * dt;

        ctx.strokeStyle = 'rgba(180, 220, 255, 0.6)';
        ctx.lineWidth = 1.5;
        ctx.beginPath();
        ctx.moveTo(p.x, p.y);
        ctx.lineTo(p.x - 3, p.y + 12);
        ctx.stroke();

        // Splash on ground
        if (p.y >= GAME_HEIGHT - GROUND_HEIGHT) {
          ctx.beginPath();
          ctx.arc(p.x, GAME_HEIGHT - GROUND_HEIGHT, 3, 0, Math.PI);
          ctx.stroke();
          p.y = -10;
          p.x = Math.random() * (GAME_WIDTH + 60);
        }
      } else if (weather === 'Snow') {
        p.vy = 60 + Math.sin(p.angle) * 20;
        p.vx = Math.sin(p.angle) * 30;
        p.angle += dt * 3;
        p.x += p.vx * dt;
        p.y += p.vy * dt;

        ctx.fillStyle = 'rgba(255, 255, 255, 0.85)';
        ctx.beginPath();
        ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2);
        ctx.fill();

        if (p.y >= GAME_HEIGHT - GROUND_HEIGHT) {
          p.y = -10;
          p.x = Math.random() * GAME_WIDTH;
        }
      } else if (weather === 'Sakura') {
        p.vy = 45;
        p.vx = 40 + Math.sin(p.angle) * 30;
        p.angle += dt * 2.5;
        p.x += p.vx * dt;
        p.y += p.vy * dt;

        ctx.fillStyle = 'rgba(255, 183, 197, 0.85)';
        ctx.beginPath();
        ctx.ellipse(p.x, p.y, 4, 2.5, p.angle, 0, Math.PI * 2);
        ctx.fill();

        if (p.y >= GAME_HEIGHT - GROUND_HEIGHT || p.x > GAME_WIDTH + 20) {
          p.y = -10;
          p.x = Math.random() * GAME_WIDTH;
        }
      } else if (weather === 'Firefly') {
        p.angle += dt * 2;
        p.x += Math.sin(p.angle) * 15 * dt;
        p.y += Math.cos(p.angle * 0.7) * 12 * dt;

        const glow = 0.4 + Math.sin(p.angle * 2) * 0.4;
        ctx.fillStyle = `rgba(255, 240, 100, ${glow})`;
        ctx.shadowColor = '#FFEB3B';
        ctx.shadowBlur = 8;
        ctx.beginPath();
        ctx.arc(p.x, p.y, 2.5, 0, Math.PI * 2);
        ctx.fill();
        ctx.shadowBlur = 0;
      }
    });
    ctx.restore();
  }

  // ─── Popups & Particle Effects ──────────────────────────────────────────────
  function addPopup(x, y, text, color) {
    popups.push({ x, y, text, color, life: 1.0, maxLife: 1.0 });
  }

  function updateAndDrawPopups(dt) {
    for (let i = popups.length - 1; i >= 0; i--) {
      const p = popups[i];
      p.life -= dt;
      p.y -= dt * 35;

      const alpha = Math.max(0, p.life / p.maxLife);
      ctx.save();
      ctx.globalAlpha = alpha;
      ctx.font = 'bold 15px Arial';
      ctx.textAlign = 'center';
      ctx.fillStyle = '#000';
      ctx.fillText(p.text, p.x + 1, p.y + 1); // shadow
      ctx.fillStyle = p.color;
      ctx.fillText(p.text, p.x, p.y);
      ctx.restore();

      if (p.life <= 0) {
        popups.splice(i, 1);
      }
    }
  }

  function updateAndDrawParticles(dt) {
    for (let i = particles.length - 1; i >= 0; i--) {
      const p = particles[i];
      p.life -= dt;
      p.x += p.vx * dt;
      p.y += p.vy * dt;

      const alpha = Math.max(0, p.life / p.maxLife);
      ctx.save();
      ctx.globalAlpha = alpha;
      ctx.fillStyle = p.color;
      ctx.beginPath();
      ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2);
      ctx.fill();
      ctx.restore();

      if (p.life <= 0) {
        particles.splice(i, 1);
      }
    }
  }

  // ─── Collisions & Hit Detection ─────────────────────────────────────────────
  function checkCollision() {
    const margin = 3.5;
    const bx = bird.x + margin;
    const by = bird.y + margin;
    const bw = BIRD_WIDTH - margin * 2;
    const bh = BIRD_HEIGHT - margin * 2;

    // Ground & Ceiling
    if (by + bh >= GAME_HEIGHT - GROUND_HEIGHT || by <= -20) {
      triggerDeath();
      return;
    }

    // Pipe Collisions
    for (const p of pipes.list) {
      if (bx + bw > p.x && bx < p.x + 52) {
        const topLipY = p.gapY - p.gap / 2;
        const botLipY = p.gapY + p.gap / 2;

        if (by < topLipY || by + bh > botLipY) {
          // If shield is active, absorb hit!
          if (powerUps.shield.active) {
            powerUps.shield.active = false;
            window.soundEngine.playShieldBreak();
            screenShake = 10;
            addPopup(bird.x, bird.y, '💥 SHIELD SAVED YOU!', '#FF5252');

            // Burst shield particles
            for (let k = 0; k < 16; k++) {
              particles.push({
                x: bird.x + BIRD_WIDTH / 2,
                y: bird.y + BIRD_HEIGHT / 2,
                vx: (Math.random() - 0.5) * 180,
                vy: (Math.random() - 0.5) * 180,
                life: 0.5,
                maxLife: 0.5,
                color: '#46A0FF',
                size: 3 + Math.random() * 3,
              });
            }
            return;
          }

          triggerDeath();
          return;
        }
      }
    }
  }

  function triggerDeath() {
    if (currentState !== STATE.PLAYING) return;

    currentState = STATE.DYING;
    bird.alive = false;
    bird.velY = -240;
    bird.tumbleSpeed = 16;
    screenShake = 12;
    gameDuration = Math.round((Date.now() - gameStartTime) / 1000);

    window.soundEngine.playHit();

    // Android Haptic vibration on crash
    if (navigator.vibrate) {
      navigator.vibrate([40, 30, 40]);
    }

    // Determine Medal Rank
    let medal = 'None';
    if (score >= 50) medal = 'Platinum';
    else if (score >= 30) medal = 'Gold';
    else if (score >= 20) medal = 'Silver';
    else if (score >= 10) medal = 'Bronze';

    if (score > bestScore) {
      bestScore = score;
      localStorage.setItem('flappy_best_score', bestScore.toString());
    }

    // Save score asynchronously to SQLite & IndexedDB Database
    const scoreData = {
      playerName: window.scoreDB.getPlayerName(),
      score,
      coins,
      pipesPassed,
      difficulty: DIFFICULTIES[currentDiffIndex].name,
      skin: SKINS[currentSkinIndex].name,
      weather: WEATHERS[currentWeatherIndex],
      medal,
      combo: maxCombo,
      closeShaves,
      durationSeconds: gameDuration,
    };

    window.scoreDB.saveScore(scoreData).then(rec => {
      lastScoreRecord = rec;
      // Refresh database modal data if open
      if (typeof window.refreshLeaderboardView === 'function') {
        window.refreshLeaderboardView();
      }
    });

    setTimeout(() => {
      currentState = STATE.GAMEOVER;
    }, 650);
  }

  // ─── UI Rendering (Start Screen, HUD, Game Over) ────────────────────────────
  function drawStartScreen() {
    // Title Banner
    ctx.save();
    ctx.textAlign = 'center';

    // Title Drop Shadow
    ctx.font = 'bold 36px "Trebuchet MS", sans-serif';
    ctx.fillStyle = '#543847';
    ctx.fillText('FLAPPY BIRD', GAME_WIDTH / 2 + 3, 143);
    ctx.fillStyle = '#FFFFFF';
    ctx.fillText('FLAPPY BIRD', GAME_WIDTH / 2, 140);

    ctx.font = 'bold 16px Arial';
    ctx.fillStyle = '#FFE066';
    ctx.fillText('★ PRO EDITION ★', GAME_WIDTH / 2, 168);

    // Instructions Box
    ctx.fillStyle = 'rgba(0, 0, 0, 0.4)';
    roundRect(ctx, 40, 350, 320, 190, 12, true);

    ctx.font = 'bold 15px Arial';
    ctx.fillStyle = '#FFFFFF';
    ctx.fillText('TAP OR PRESS SPACE TO FLAP', GAME_WIDTH / 2, 380);

    ctx.font = '13px Arial';
    ctx.fillStyle = '#E0E0E0';
    ctx.fillText(`🎮 Difficulty: ${DIFFICULTIES[currentDiffIndex].name} [Tap or 'D']`, GAME_WIDTH / 2, 412);
    ctx.fillText(`🎨 Bird Skin: ${SKINS[currentSkinIndex].name} [Tap or 'C']`, GAME_WIDTH / 2, 438);
    ctx.fillText(`🌧️ Weather: ${WEATHERS[currentWeatherIndex]} [Tap or 'W']`, GAME_WIDTH / 2, 464);
    ctx.fillText(`🌙 Sky Cycle: ${CYCLES[currentCycleIndex]} ['N']`, GAME_WIDTH / 2, 490);
    ctx.fillText(`🏆 Press 'L' for High Score Database`, GAME_WIDTH / 2, 516);

    // Player Gamer Tag Badge
    ctx.font = 'bold 14px Arial';
    ctx.fillStyle = '#FFFDE7';
    ctx.fillText(`👤 Player: ${window.scoreDB.getPlayerName()}`, GAME_WIDTH / 2, 570);

    ctx.restore();
  }

  function drawHUD() {
    // Current Score Big Counter
    ctx.save();
    ctx.textAlign = 'center';
    ctx.font = 'bold 44px "Impact", "Arial Black", sans-serif';

    // Drop shadow
    ctx.fillStyle = '#000000';
    ctx.fillText(score.toString(), GAME_WIDTH / 2 + 2, 62);
    ctx.fillStyle = '#FFFFFF';
    ctx.fillText(score.toString(), GAME_WIDTH / 2, 60);

    // Coins Counter (Top Left)
    ctx.textAlign = 'left';
    ctx.font = 'bold 16px Arial';
    ctx.fillStyle = '#FFD700';
    ctx.fillText(`🪙 ${coins}`, 20, 35);

    // Combo Streak (Top Right)
    if (combo >= 3) {
      ctx.textAlign = 'right';
      ctx.fillStyle = '#FF9800';
      ctx.fillText(`🔥 ${combo}x`, GAME_WIDTH - 20, 35);
    }

    // Active Powerups Bar
    let badgeX = 20;
    const badgeY = 52;
    if (powerUps.shield.active) {
      drawPill(ctx, badgeX, badgeY, '🛡️ Shield', '#2979FF');
      badgeX += 80;
    }
    if (powerUps.magnet.active) {
      drawPill(ctx, badgeX, badgeY, `🧲 ${Math.ceil(powerUps.magnet.timer)}s`, '#FFD700');
      badgeX += 70;
    }
    if (powerUps.slowmo.active) {
      drawPill(ctx, badgeX, badgeY, `⏱️ ${Math.ceil(powerUps.slowmo.timer)}s`, '#00E5FF');
      badgeX += 70;
    }
    if (powerUps.doubleScore.active) {
      drawPill(ctx, badgeX, badgeY, `2️⃣ ${Math.ceil(powerUps.doubleScore.timer)}s`, '#FF9100');
    }

    ctx.restore();
  }

  function drawGameOver() {
    ctx.save();
    ctx.textAlign = 'center';

    // Game Over Banner
    ctx.font = 'bold 36px "Trebuchet MS", sans-serif';
    ctx.fillStyle = '#543847';
    ctx.fillText('GAME OVER', GAME_WIDTH / 2 + 3, 143);
    ctx.fillStyle = '#FF5252';
    ctx.fillText('GAME OVER', GAME_WIDTH / 2, 140);

    // Scoreboard Panel
    const px = 40;
    const py = 180;
    const pw = 320;
    const ph = 240;

    ctx.fillStyle = '#DED895';
    roundRect(ctx, px, py, pw, ph, 14, true);
    ctx.lineWidth = 3;
    ctx.strokeStyle = '#543847';
    roundRect(ctx, px, py, pw, ph, 14, false);

    // Inner score details
    ctx.textAlign = 'left';
    ctx.font = 'bold 15px Arial';
    ctx.fillStyle = '#8B4513';
    ctx.fillText('SCORE', px + 24, py + 45);
    ctx.textAlign = 'right';
    ctx.font = 'bold 24px "Impact", Arial';
    ctx.fillStyle = '#000000';
    ctx.fillText(score.toString(), px + pw - 24, py + 46);

    ctx.textAlign = 'left';
    ctx.font = 'bold 15px Arial';
    ctx.fillStyle = '#8B4513';
    ctx.fillText('BEST SCORE', px + 24, py + 85);
    ctx.textAlign = 'right';
    ctx.font = 'bold 24px "Impact", Arial';
    ctx.fillStyle = score >= bestScore && score > 0 ? '#E91E63' : '#000000';
    ctx.fillText(bestScore.toString(), px + pw - 24, py + 86);

    // Medal
    let medalEmoji = '⚪ None';
    let medalColor = '#757575';
    if (score >= 50) { medalEmoji = '💎 Platinum'; medalColor = '#00E5FF'; }
    else if (score >= 30) { medalEmoji = '🥇 Gold'; medalColor = '#FFD700'; }
    else if (score >= 20) { medalEmoji = '🥈 Silver'; medalColor = '#C0C0C0'; }
    else if (score >= 10) { medalEmoji = '🥉 Bronze'; medalColor = '#CD7F32'; }

    ctx.textAlign = 'left';
    ctx.font = 'bold 15px Arial';
    ctx.fillStyle = '#8B4513';
    ctx.fillText('MEDAL EARNED', px + 24, py + 125);
    ctx.textAlign = 'right';
    ctx.fillStyle = medalColor;
    ctx.fillText(medalEmoji, px + pw - 24, py + 125);

    // Stats breakdown row
    ctx.textAlign = 'center';
    ctx.font = '12px Arial';
    ctx.fillStyle = '#555555';
    ctx.fillText(`🪙 Coins: ${coins}  |  🔥 Max Combo: ${maxCombo}x  |  ⚡ Shaves: ${closeShaves}`, GAME_WIDTH / 2, py + 165);

    // Database Sync status
    ctx.font = 'bold 12px Arial';
    if (lastScoreRecord && lastScoreRecord.synced) {
      ctx.fillStyle = '#2E7D32';
      const rankText = lastScoreRecord.rank ? ` [Rank #${lastScoreRecord.rank}]` : '';
      ctx.fillText(`✅ Saved to SQLite Database${rankText}`, GAME_WIDTH / 2, py + 200);
    } else {
      ctx.fillStyle = '#F57F17';
      ctx.fillText(`💾 Saved to Local Storage (Offline)`, GAME_WIDTH / 2, py + 200);
    }

    // Restart prompt
    ctx.font = 'bold 16px Arial';
    ctx.fillStyle = '#FFFFFF';
    ctx.fillText('TAP OR PRESS SPACE TO RESTART', GAME_WIDTH / 2, 465);

    ctx.font = '13px Arial';
    ctx.fillStyle = '#FFE066';
    ctx.fillText('Press [L] to view Leaderboard & Database', GAME_WIDTH / 2, 495);

    ctx.restore();
  }

  function drawPill(ctx, x, y, text, color) {
    ctx.save();
    ctx.fillStyle = color;
    roundRect(ctx, x, y, 62, 20, 8, true);
    ctx.fillStyle = '#000000';
    ctx.font = 'bold 11px Arial';
    ctx.textAlign = 'center';
    ctx.fillText(text, x + 31, y + 14);
    ctx.restore();
  }

  function roundRect(ctx, x, y, w, h, r, fill) {
    ctx.beginPath();
    ctx.moveTo(x + r, y);
    ctx.arcTo(x + w, y, x + w, y + h, r);
    ctx.arcTo(x + w, y + h, x, y + h, r);
    ctx.arcTo(x, y + h, x, y, r);
    ctx.arcTo(x, y, x + w, y, r);
    ctx.closePath();
    if (fill) ctx.fill();
    else ctx.stroke();
  }

  // ─── Input & Event Handling ────────────────────────────────────────────────
  function handleAction() {
    window.soundEngine.ensureContext();

    if (currentState === STATE.START) {
      currentState = STATE.PLAYING;
      score = 0;
      coins = 0;
      pipesPassed = 0;
      combo = 0;
      maxCombo = 0;
      closeShaves = 0;
      gameStartTime = Date.now();
      lastScoreRecord = null;
      bird.reset();
      pipes.reset();
      powerUps.reset();
      bird.jump();
    } else if (currentState === STATE.PLAYING) {
      bird.jump();
    } else if (currentState === STATE.GAMEOVER) {
      currentState = STATE.START;
      bird.reset();
      pipes.reset();
      powerUps.reset();
    }
  }

  // Keyboard controls
  window.addEventListener('keydown', e => {
    if (e.code === 'Space' || e.code === 'ArrowUp') {
      e.preventDefault();
      handleAction();
    } else if (e.code === 'KeyD') {
      cycleDifficulty();
    } else if (e.code === 'KeyC') {
      cycleSkin();
    } else if (e.code === 'KeyW') {
      cycleWeather();
    } else if (e.code === 'KeyN') {
      cycleCycle();
    } else if (e.code === 'KeyM') {
      window.soundEngine.toggleMute();
      updateMuteButton();
    } else if (e.code === 'KeyP' && currentState === STATE.PLAYING) {
      isPaused = !isPaused;
    } else if (e.code === 'KeyL') {
      if (typeof window.openLeaderboardModal === 'function') {
        window.openLeaderboardModal();
      }
    }
  });

  // Touch & Pointer controls (Mobile / Android)
  canvas.addEventListener('pointerdown', e => {
    e.preventDefault();
    handleAction();
  });

  function cycleDifficulty() {
    currentDiffIndex = (currentDiffIndex + 1) % DIFFICULTIES.length;
    addPopup(GAME_WIDTH / 2, 330, `DIFFICULTY: ${DIFFICULTIES[currentDiffIndex].name}`, DIFFICULTIES[currentDiffIndex].color);
  }

  function cycleSkin() {
    currentSkinIndex = (currentSkinIndex + 1) % SKINS.length;
    addPopup(GAME_WIDTH / 2, 330, `SKIN: ${SKINS[currentSkinIndex].name}`, SKINS[currentSkinIndex].body);
  }

  function cycleWeather() {
    currentWeatherIndex = (currentWeatherIndex + 1) % WEATHERS.length;
    addPopup(GAME_WIDTH / 2, 330, `WEATHER: ${WEATHERS[currentWeatherIndex]}`, '#00E5FF');
  }

  function cycleCycle() {
    currentCycleIndex = (currentCycleIndex + 1) % CYCLES.length;
    addPopup(GAME_WIDTH / 2, 330, `SKY: ${CYCLES[currentCycleIndex]}`, '#FFD700');
  }

  window.cycleDifficulty = cycleDifficulty;
  window.cycleSkin = cycleSkin;
  window.cycleWeather = cycleWeather;

  // ─── Main Game Loop ────────────────────────────────────────────────────────
  let lastTime = performance.now();

  function gameLoop(now) {
    const dt = Math.min((now - lastTime) / 1000, 0.1);
    lastTime = now;

    if (!isPaused) {
      // Screen shake decay
      if (screenShake > 0) {
        screenShake -= dt * 25;
        if (screenShake < 0) screenShake = 0;
      }

      ctx.save();
      if (screenShake > 0) {
        ctx.translate((Math.random() - 0.5) * screenShake, (Math.random() - 0.5) * screenShake);
      }

      // 1. Draw Sky & Celestial background
      drawBackground(dt);

      // 2. Weather simulation
      updateAndDrawWeather(dt);

      // 3. Pipes & Coins
      if (currentState === STATE.PLAYING) {
        pipes.update(dt);
        powerUps.update(dt);
      }
      pipes.draw();
      powerUps.draw();

      // 4. Ground
      drawGround(dt);

      // 5. Bird
      bird.update(dt);
      bird.draw();

      // 6. Collisions
      if (currentState === STATE.PLAYING) {
        checkCollision();
      }

      // 7. Popups & Particles
      updateAndDrawParticles(dt);
      updateAndDrawPopups(dt);

      // 8. State UI
      if (currentState === STATE.START) {
        drawStartScreen();
      } else if (currentState === STATE.PLAYING) {
        drawHUD();
      } else if (currentState === STATE.GAMEOVER) {
        drawGameOver();
      }

      // Slow-Mo Cyan Vignette
      if (powerUps.slowmo.active) {
        ctx.fillStyle = 'rgba(0, 229, 255, 0.08)';
        ctx.fillRect(0, 0, GAME_WIDTH, GAME_HEIGHT);
      }

      // Double-Score Gold Vignette
      if (powerUps.doubleScore.active) {
        ctx.fillStyle = 'rgba(255, 215, 0, 0.08)';
        ctx.fillRect(0, 0, GAME_WIDTH, GAME_HEIGHT);
      }

      ctx.restore();
    }

    requestAnimationFrame(gameLoop);
  }

  requestAnimationFrame(gameLoop);

  // Expose methods for UI buttons
  window.flappyGame = {
    handleAction,
    cycleDifficulty,
    cycleSkin,
    cycleWeather,
    getDifficultyName: () => DIFFICULTIES[currentDiffIndex].name,
    getSkinName: () => SKINS[currentSkinIndex].name,
    getWeatherName: () => WEATHERS[currentWeatherIndex],
    isPaused: () => isPaused,
    togglePause: () => { isPaused = !isPaused; },
  };
})();
