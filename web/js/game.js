// ─── Flappy Bird Pro: HTML5 Canvas Game Engine ──────────────────────────────
(function() {
  'use strict';

  const GAME_WIDTH = 400;
  const GAME_HEIGHT = 700;
  const GROUND_HEIGHT = 112;
  const BIRD_WIDTH = 34;
  const BIRD_HEIGHT = 24;

  const canvas = document.getElementById('gameCanvas');
  const ctx = canvas.getContext('2d', { alpha: false, desynchronized: true });

  // Set logical resolution
  canvas.width = GAME_WIDTH;
  canvas.height = GAME_HEIGHT;
  ctx.imageSmoothingEnabled = false;

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

  // ─── Game Modes ─────────────────────────────────────────────────────────────
  const MODE = {
    SOLO: 'solo',
    DUEL: 'duel',
  };
  let currentMode = MODE.SOLO;

  // ─── Game Variables ─────────────────────────────────────────────────────────
  let score = 0;
  let score1 = 0;
  let score2 = 0;
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
  let duelWins1 = 0;
  let duelWins2 = 0;
  let hasAnnouncedRecord = false;

  // ─── Camera Shake & Speed Lines ─────────────────────────────────────────────
  let screenShake = 0;
  let speedLines = [];

  // ─── Popups & Particles Object Pools (Zero-GC Optimization) ─────────────────
  const MAX_POPUPS = 16;
  const popupPool = new Array(MAX_POPUPS);
  for (let i = 0; i < MAX_POPUPS; i++) {
    popupPool[i] = { active: false, x: 0, y: 0, text: '', color: '#fff', life: 0, maxLife: 1.0 };
  }
  let nextPopupIdx = 0;

  const MAX_PARTICLES = 128;
  const particlePool = new Array(MAX_PARTICLES);
  for (let i = 0; i < MAX_PARTICLES; i++) {
    particlePool[i] = { active: false, x: 0, y: 0, vx: 0, vy: 0, life: 0, maxLife: 0, color: '#fff', size: 3 };
  }
  let nextParticleIdx = 0;

  function spawnParticle(x, y, vx, vy, life, color, size) {
    const p = particlePool[nextParticleIdx];
    nextParticleIdx = (nextParticleIdx + 1) % MAX_PARTICLES;
    p.active = true;
    p.x = x;
    p.y = y;
    p.vx = vx;
    p.vy = vy;
    p.life = life;
    p.maxLife = life;
    p.color = color || '#fff';
    p.size = size || 3;
    return p;
  }

  let weatherParticles = [];
  let stars = [];

  // ─── Bird Entity Factory ────────────────────────────────────────────────────
  function createBird(startX, defaultSkinIdx, label, isP2) {
    return {
      x: startX,
      startX: startX,
      y: 260,
      velY: 0,
      rotation: 0,
      wingAngle: 0,
      wingDir: 1,
      alive: true,
      tumbleSpeed: 0,
      skinIndex: defaultSkinIdx,
      label: label,
      isP2: !!isP2,

      reset(startX) {
        if (startX !== undefined) this.startX = startX;
        this.x = this.startX;
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
        if (this.isP2) {
          window.soundEngine.playFlapP2();
        } else {
          window.soundEngine.playFlapP1();
        }

        if (navigator.vibrate) {
          navigator.vibrate(15);
        }

        const skin = SKINS[this.skinIndex % SKINS.length];
        for (let i = 0; i < 3; i++) {
          spawnParticle(
            this.x + 5,
            this.y + 15,
            -40 - Math.random() * 40,
            (Math.random() - 0.5) * 50,
            0.35,
            skin.belly,
            3 + Math.random() * 2
          );
        }
      },

      update(dt) {
        if (currentState === STATE.START) {
          const offset = this.startX === 70 ? 0 : Math.PI;
          this.y = 260 + Math.sin(Date.now() / 250 + offset) * 8;
          this.rotation = Math.sin(Date.now() / 250 + offset) * 0.08;
          this.wingAngle += dt * 8 * this.wingDir;
          if (Math.abs(this.wingAngle) > 0.4) this.wingDir *= -1;
          return;
        }

        if (this.alive) {
          this.velY += 980 * dt;
          if (this.velY > 520) this.velY = 520;
          this.y += this.velY * dt;

          const targetRot = Math.min(Math.PI / 2.2, Math.max(-0.45, this.velY * 0.0028));
          this.rotation += (targetRot - this.rotation) * dt * 10;

          if (this.velY < 0) {
            this.wingAngle += dt * 18 * this.wingDir;
            if (Math.abs(this.wingAngle) > 0.6) this.wingDir *= -1;
          } else {
            this.wingAngle += (0 - this.wingAngle) * dt * 8;
          }
        } else {
          // Dramatic death tumble
          this.velY += 1200 * dt;
          this.y += this.velY * dt;
          this.rotation += this.tumbleSpeed * dt;
        }
      },

      draw() {
        ctx.save();
        ctx.translate(this.x + BIRD_WIDTH / 2, this.y + BIRD_HEIGHT / 2);
        ctx.rotate(this.rotation);

        const skin = SKINS[this.skinIndex % SKINS.length];

        // Draw Shield Aura if active
        if ((this.label === 'P1' || currentMode === MODE.SOLO) && powerUps.shield.active) {
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
        if ((this.label === 'P1' || currentMode === MODE.SOLO) && powerUps.magnet.active) {
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
          ctx.lineWidth = 2;
          ctx.strokeStyle = '#222';
          ctx.beginPath();
          ctx.moveTo(eyeX - 4, eyeY - 4);
          ctx.lineTo(eyeX + 4, eyeY + 4);
          ctx.moveTo(eyeX + 4, eyeY - 4);
          ctx.lineTo(eyeX - 4, eyeY + 4);
          ctx.stroke();
        } else {
          ctx.beginPath();
          ctx.ellipse(eyeX, eyeY, 6, 7, 0, 0, Math.PI * 2);
          ctx.fillStyle = '#FFFFFF';
          ctx.fill();
          ctx.lineWidth = 1.5;
          ctx.strokeStyle = skin.outline;
          ctx.stroke();

          ctx.beginPath();
          ctx.arc(eyeX + 2, eyeY, 2.5, 0, Math.PI * 2);
          ctx.fillStyle = '#000000';
          ctx.fill();

          ctx.beginPath();
          ctx.arc(eyeX + 3, eyeY - 2, 1, 0, Math.PI * 2);
          ctx.fillStyle = '#FFFFFF';
          ctx.fill();
        }

        // Cheek blush
        ctx.beginPath();
        ctx.arc(4, 3, 3, 0, Math.PI * 2);
        ctx.fillStyle = skin.blush || '#FF8A80';
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
      }
    };
  }

  const bird1 = createBird(80, 0, 'P1', false);
  const bird2 = createBird(110, 1, 'P2', true);
  const bird = bird1; // alias for solo compatibility

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

        if (currentMode === MODE.DUEL) {
          // Player 1 scoring
          if (!p.passed1 && bird1.alive && p.x + 52 < bird1.x) {
            p.passed1 = true;
            const pts = p.golden ? 5 : 1;
            score1 += pts;
            window.soundEngine.playScore();
            addPopup(bird1.x + 10, bird1.y - 20, p.golden ? 'P1 +5!' : 'P1 +1', '#FFD700');
          }
          // Player 2 scoring
          if (!p.passed2 && bird2.alive && p.x + 52 < bird2.x) {
            p.passed2 = true;
            const pts = p.golden ? 5 : 1;
            score2 += pts;
            window.soundEngine.playScore();
            addPopup(bird2.x + 10, bird2.y - 20, p.golden ? 'P2 +5!' : 'P2 +1', '#46A0FF');
          }
          score = Math.max(score1, score2);
        } else {
          // Solo passing pipe check
          if (!p.passed && p.x + 52 < bird1.x) {
            p.passed = true;
            pipesPassed++;
            combo++;
            if (combo > maxCombo) maxCombo = combo;

            let points = 1 + diff.bonus;
            if (p.golden) {
              points = 5;
              window.soundEngine.playGolden();
              addPopup(bird1.x, bird1.y - 30, '+5 GOLDEN!', '#FFD700');
              for (let k = 0; k < 12; k++) {
                spawnParticle(
                  p.x + 26,
                  p.gapY,
                  (Math.random() - 0.5) * 160,
                  (Math.random() - 0.5) * 160,
                  0.6,
                  '#FFD700',
                  4 + Math.random() * 3
                );
              }
            } else {
              window.soundEngine.playScore();
              addPopup(bird1.x, bird1.y - 20, points > 1 ? `+${points}` : '+1', '#FFFFFF');
            }

            if (powerUps.doubleScore.active) {
              points *= 2;
            }

            score += points;

            if (score > bestScore && bestScore > 0 && !hasAnnouncedRecord) {
              hasAnnouncedRecord = true;
              addPopup(GAME_WIDTH / 2, 210, '🏆 NEW RECORD!', '#FFD700');
              window.soundEngine.playGolden();
              screenShake = 6;
              for (let k = 0; k < 18; k++) {
                spawnParticle(
                  bird1.x,
                  bird1.y,
                  (Math.random() - 0.5) * 140,
                  (Math.random() - 0.5) * 140,
                  0.7,
                  k % 2 === 0 ? '#FFD700' : '#00E5FF',
                  3.5
                );
              }
            }

            if (combo >= 5 && combo % 5 === 0) {
              addPopup(GAME_WIDTH / 2, 180, `🔥 ${combo}X COMBO!`, '#FF9800');
            }
          }

          // Close-shave near-miss check (skimming pipe lip within 14px)
          if (!p.closeShaved && Math.abs(p.x + 26 - (bird1.x + BIRD_WIDTH / 2)) < 26) {
            const topLipY = p.gapY - p.gap / 2;
            const botLipY = p.gapY + p.gap / 2;
            const birdTop = bird1.y;
            const birdBot = bird1.y + BIRD_HEIGHT;

            const distTop = Math.abs(birdTop - topLipY);
            const distBot = Math.abs(birdBot - botLipY);

            if ((distTop < 14 || distBot < 14) && birdTop > topLipY - 5 && birdBot < botLipY + 5) {
              p.closeShaved = true;
              closeShaves++;
              score += 2;
              window.soundEngine.playCloseShave();
              addPopup(bird1.x, bird1.y - 35, '⚡ CLOSE SHAVE! +2', '#00E5FF');

              for (let k = 0; k < 8; k++) {
                spawnParticle(
                  bird1.x + BIRD_WIDTH,
                  distTop < 14 ? birdTop : birdBot,
                  50 + Math.random() * 80,
                  (Math.random() - 0.5) * 80,
                  0.4,
                  '#00E5FF',
                  2.5
                );
              }
            }
          }
        }

        // Magnet attraction for coin
        const activeTarget = (currentMode === MODE.DUEL && !bird1.alive && bird2.alive) ? bird2 : bird1;
        if (p.hasCoin && !p.coinCollected) {
          const coinX = p.x + 26;
          const coinY = p.gapY;
          if (powerUps.magnet.active) {
            const dist = Math.hypot(coinX - (activeTarget.x + BIRD_WIDTH / 2), coinY - (activeTarget.y + BIRD_HEIGHT / 2));
            if (dist < 140) {
              p.coinOffset = (p.coinOffset || 0) + (activeTarget.x - coinX) * dt * 4;
            }
          }

          // Coin pickup collision
          const bx = activeTarget.x + BIRD_WIDTH / 2;
          const by = activeTarget.y + BIRD_HEIGHT / 2;
          if (Math.hypot(coinX + (p.coinOffset || 0) - bx, coinY - by) < 22) {
            p.coinCollected = true;
            coins++;
            const coinPts = powerUps.doubleScore.active ? 4 : 2;
            score += coinPts;
            if (currentMode === MODE.DUEL) {
              if (activeTarget === bird2) score2 += coinPts;
              else score1 += coinPts;
            }
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
    ctx.beginPath();
    for (let x = -groundScroll; x < GAME_WIDTH + 24; x += 20) {
      ctx.moveTo(x, groundY + 22);
      ctx.lineTo(x - 8, groundY + 36);
    }
    ctx.stroke();

    // Top border line
    ctx.strokeStyle = '#543847';
    ctx.lineWidth = 2.5;
    ctx.beginPath();
    ctx.moveTo(0, groundY);
    ctx.lineTo(GAME_WIDTH, groundY);
    ctx.stroke();
  }

  // ─── Celestial Background (Day / Sunset / Night / Dawn) ────────────────────
  const GRADIENT_CACHE = {};
  function initGradients() {
    let g = ctx.createLinearGradient(0, 0, 0, GAME_HEIGHT - GROUND_HEIGHT);
    g.addColorStop(0, '#4ec0ca');
    g.addColorStop(1, '#a3e5eb');
    GRADIENT_CACHE['Day'] = g;

    g = ctx.createLinearGradient(0, 0, 0, GAME_HEIGHT - GROUND_HEIGHT);
    g.addColorStop(0, '#2C3E50');
    g.addColorStop(0.4, '#E74C3C');
    g.addColorStop(0.8, '#F39C12');
    g.addColorStop(1, '#F1C40F');
    GRADIENT_CACHE['Sunset'] = g;

    g = ctx.createLinearGradient(0, 0, 0, GAME_HEIGHT - GROUND_HEIGHT);
    g.addColorStop(0, '#0B0C1E');
    g.addColorStop(0.6, '#181E3B');
    g.addColorStop(1, '#272F55');
    GRADIENT_CACHE['Night'] = g;

    g = ctx.createLinearGradient(0, 0, 0, GAME_HEIGHT - GROUND_HEIGHT);
    g.addColorStop(0, '#3F2B96');
    g.addColorStop(0.5, '#A8C0FF');
    g.addColorStop(1, '#FEE140');
    GRADIENT_CACHE['Dawn'] = g;
  }
  initGradients();

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

    ctx.fillStyle = GRADIENT_CACHE[mode] || GRADIENT_CACHE['Day'];
    ctx.fillRect(0, 0, GAME_WIDTH, GAME_HEIGHT);

    if (mode === 'Day') {
      // Sun
      ctx.beginPath();
      ctx.arc(330, 90, 32, 0, Math.PI * 2);
      ctx.fillStyle = '#FFF8A0';
      ctx.shadowColor = '#FFF8A0';
      ctx.shadowBlur = 18;
      ctx.fill();
      ctx.shadowBlur = 0;
    } else if (mode === 'Sunset') {
      // Sinking Sun
      ctx.beginPath();
      ctx.arc(280, 220, 36, 0, Math.PI * 2);
      ctx.fillStyle = '#FF6B6B';
      ctx.fill();
    } else if (mode === 'Night') {
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
    }

    // Parallax Clouds
    drawParallaxClouds();
  }

  function drawParallaxClouds() {
    const y = 380;
    ctx.fillStyle = 'rgba(255, 255, 255, 0.45)';
    ctx.beginPath();
    for (let i = 0; i < 3; i++) {
      const cx = (i * 180 - cloudScroll) % (GAME_WIDTH + 180) - 60;
      ctx.ellipse(cx, y, 45, 20, 0, 0, Math.PI * 2);
      ctx.ellipse(cx + 25, y - 8, 35, 22, 0, 0, Math.PI * 2);
      ctx.ellipse(cx + 50, y, 40, 18, 0, 0, Math.PI * 2);
    }
    ctx.fill();
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
    if (weather === 'Rain') {
      ctx.strokeStyle = 'rgba(180, 220, 255, 0.6)';
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      weatherParticles.forEach(p => {
        p.vy = 420;
        p.vx = -60;
        p.x += p.vx * dt;
        p.y += p.vy * dt;
        ctx.moveTo(p.x, p.y);
        ctx.lineTo(p.x - 3, p.y + 12);
        if (p.y >= GAME_HEIGHT - GROUND_HEIGHT) {
          p.y = -10;
          p.x = Math.random() * (GAME_WIDTH + 60);
        }
      });
      ctx.stroke();
    } else if (weather === 'Snow') {
      ctx.fillStyle = 'rgba(255, 255, 255, 0.85)';
      ctx.beginPath();
      weatherParticles.forEach(p => {
        p.vy = 60 + Math.sin(p.angle) * 20;
        p.vx = Math.sin(p.angle) * 30;
        p.angle += dt * 3;
        p.x += p.vx * dt;
        p.y += p.vy * dt;
        ctx.moveTo(p.x + p.size, p.y);
        ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2);
        if (p.y >= GAME_HEIGHT - GROUND_HEIGHT) {
          p.y = -10;
          p.x = Math.random() * GAME_WIDTH;
        }
      });
      ctx.fill();
    } else if (weather === 'Sakura') {
      ctx.fillStyle = 'rgba(255, 183, 197, 0.85)';
      ctx.beginPath();
      weatherParticles.forEach(p => {
        p.vy = 45;
        p.vx = 40 + Math.sin(p.angle) * 30;
        p.angle += dt * 2.5;
        p.x += p.vx * dt;
        p.y += p.vy * dt;
        ctx.ellipse(p.x, p.y, 4, 2.5, p.angle, 0, Math.PI * 2);
        if (p.y >= GAME_HEIGHT - GROUND_HEIGHT || p.x > GAME_WIDTH + 20) {
          p.y = -10;
          p.x = Math.random() * GAME_WIDTH;
        }
      });
      ctx.fill();
    } else if (weather === 'Firefly') {
      weatherParticles.forEach(p => {
        p.angle += dt * 2;
        p.x += Math.sin(p.angle) * 15 * dt;
        p.y += Math.cos(p.angle * 0.7) * 12 * dt;
        const glow = 0.4 + Math.sin(p.angle * 2) * 0.4;
        ctx.fillStyle = `rgba(255, 240, 100, ${glow})`;
        ctx.beginPath();
        ctx.arc(p.x, p.y, 2.5, 0, Math.PI * 2);
        ctx.fill();
      });
    }
    ctx.restore();
  }

  // ─── Popups & Particle Effects (Batched & Zero-GC) ──────────────────────────
  function addPopup(x, y, text, color) {
    const p = popupPool[nextPopupIdx];
    nextPopupIdx = (nextPopupIdx + 1) % MAX_POPUPS;
    p.active = true;
    p.x = x;
    p.y = y;
    p.text = text;
    p.color = color || '#fff';
    p.life = 1.0;
    p.maxLife = 1.0;
  }

  function updateAndDrawPopups(dt) {
    ctx.save();
    ctx.font = 'bold 15px Arial';
    ctx.textAlign = 'center';
    for (let i = 0; i < MAX_POPUPS; i++) {
      const p = popupPool[i];
      if (!p.active) continue;

      p.life -= dt;
      if (p.life <= 0) {
        p.active = false;
        continue;
      }

      p.y -= dt * 35;
      const alpha = Math.max(0, p.life / p.maxLife);
      ctx.globalAlpha = alpha;
      ctx.fillStyle = '#000';
      ctx.fillText(p.text, p.x + 1, p.y + 1); // shadow
      ctx.fillStyle = p.color;
      ctx.fillText(p.text, p.x, p.y);
    }
    ctx.restore();
  }

  function updateAndDrawParticles(dt) {
    ctx.save();
    for (let i = 0; i < MAX_PARTICLES; i++) {
      const p = particlePool[i];
      if (!p.active) continue;

      p.life -= dt;
      if (p.life <= 0) {
        p.active = false;
        continue;
      }

      p.x += p.vx * dt;
      p.y += p.vy * dt;

      const alpha = Math.max(0, p.life / p.maxLife);
      ctx.globalAlpha = alpha;
      ctx.fillStyle = p.color;
      ctx.beginPath();
      ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.restore();
  }

  // ─── Collisions & Hit Detection ─────────────────────────────────────────────
  function isBirdColliding(b) {
    const margin = 3.5;
    const bx = b.x + margin;
    const by = b.y + margin;
    const bw = BIRD_WIDTH - margin * 2;
    const bh = BIRD_HEIGHT - margin * 2;

    if (by + bh >= GAME_HEIGHT - GROUND_HEIGHT || by <= -20) {
      return true;
    }

    for (const p of pipes.list) {
      if (bx + bw > p.x && bx < p.x + 52) {
        const topLipY = p.gapY - p.gap / 2;
        const botLipY = p.gapY + p.gap / 2;
        if (by < topLipY || by + bh > botLipY) {
          return true;
        }
      }
    }
    return false;
  }

  function checkCollision() {
    if (currentMode === MODE.DUEL) {
      if (bird1.alive && isBirdColliding(bird1)) {
        bird1.alive = false;
        bird1.velY = -220;
        bird1.tumbleSpeed = 14;
        screenShake = 10;
        window.soundEngine.playHit();
        addPopup(bird1.x + 10, bird1.y - 20, 'P1 DOWN!', '#FF5252');
        for (let k = 0; k < 12; k++) {
          spawnParticle(
            bird1.x + BIRD_WIDTH / 2,
            bird1.y + BIRD_HEIGHT / 2,
            (Math.random() - 0.5) * 120,
            (Math.random() - 0.5) * 120,
            0.5,
            SKINS[bird1.skinIndex % SKINS.length].body,
            3 + Math.random() * 3
          );
        }
      }

      if (bird2.alive && isBirdColliding(bird2)) {
        bird2.alive = false;
        bird2.velY = -220;
        bird2.tumbleSpeed = 14;
        screenShake = 10;
        window.soundEngine.playHit();
        addPopup(bird2.x + 10, bird2.y - 20, 'P2 DOWN!', '#46A0FF');
        for (let k = 0; k < 12; k++) {
          spawnParticle(
            bird2.x + BIRD_WIDTH / 2,
            bird2.y + BIRD_HEIGHT / 2,
            (Math.random() - 0.5) * 120,
            (Math.random() - 0.5) * 120,
            0.5,
            SKINS[bird2.skinIndex % SKINS.length].body,
            3 + Math.random() * 3
          );
        }
      }

      if (!bird1.alive && !bird2.alive) {
        triggerDeath();
      }
    } else {
      if (bird1.alive && isBirdColliding(bird1)) {
        if (powerUps.shield.active) {
          powerUps.shield.active = false;
          window.soundEngine.playShieldBreak();
          screenShake = 10;
          addPopup(bird1.x, bird1.y, '💥 SHIELD SAVED YOU!', '#FF5252');
          for (let k = 0; k < 16; k++) {
            spawnParticle(
              bird1.x + BIRD_WIDTH / 2,
              bird1.y + BIRD_HEIGHT / 2,
              (Math.random() - 0.5) * 180,
              (Math.random() - 0.5) * 180,
              0.5,
              '#46A0FF',
              3 + Math.random() * 3
            );
          }
          return;
        }

        triggerDeath();
      }
    }
  }

  function triggerDeath() {
    if (currentState !== STATE.PLAYING) return;

    currentState = STATE.DYING;
    if (currentMode === MODE.SOLO) {
      bird1.alive = false;
      bird1.velY = -240;
      bird1.tumbleSpeed = 16;
    } else if (currentMode === MODE.DUEL) {
      if (score1 > score2) duelWins1++;
      else if (score2 > score1) duelWins2++;
    }
    screenShake = 12;
    gameDuration = Math.round((Date.now() - gameStartTime) / 1000);

    window.soundEngine.playHit();

    if (navigator.vibrate) {
      navigator.vibrate([40, 30, 40]);
    }

    const effectiveScore = currentMode === MODE.DUEL ? Math.max(score1, score2) : score;
    let medal = 'None';
    if (effectiveScore >= 50) medal = 'Platinum';
    else if (effectiveScore >= 30) medal = 'Gold';
    else if (effectiveScore >= 20) medal = 'Silver';
    else if (effectiveScore >= 10) medal = 'Bronze';

    if (effectiveScore > bestScore) {
      bestScore = effectiveScore;
      localStorage.setItem('flappy_best_score', bestScore.toString());
    }

    const scoreData = {
      playerName: window.scoreDB.getPlayerName(),
      score: effectiveScore,
      coins,
      pipesPassed,
      difficulty: DIFFICULTIES[currentDiffIndex].name,
      skin: currentMode === MODE.DUEL ? '2P Duel' : SKINS[currentSkinIndex].name,
      weather: WEATHERS[currentWeatherIndex],
      medal,
      combo: maxCombo,
      closeShaves,
      durationSeconds: gameDuration,
    };

    window.scoreDB.saveScore(scoreData).then(rec => {
      lastScoreRecord = rec;
      if (typeof window.refreshLeaderboardView === 'function') {
        window.refreshLeaderboardView();
      }
    });

    setTimeout(() => {
      currentState = STATE.GAMEOVER;
      const duelOverlay = document.getElementById('duelTouchOverlay');
      if (duelOverlay) duelOverlay.style.display = 'none';
    }, 650);
  }

  // ─── UI Rendering (Start Screen, HUD, Game Over) ────────────────────────────
  function drawStartScreen() {
    ctx.save();
    ctx.textAlign = 'center';

    // Title Drop Shadow & Text
    ctx.font = 'bold 36px "Trebuchet MS", sans-serif';
    ctx.fillStyle = '#543847';
    ctx.fillText('FLAPPY BIRD', GAME_WIDTH / 2 + 3, 133);
    ctx.fillStyle = '#FFFFFF';
    ctx.fillText('FLAPPY BIRD', GAME_WIDTH / 2, 130);

    ctx.font = 'bold 15px Arial';
    ctx.fillStyle = currentMode === MODE.DUEL ? '#00E5FF' : '#FFE066';
    ctx.fillText(currentMode === MODE.DUEL ? '⚔️ 2 PLAYERS DUEL BATTLE ⚔️' : '★ PRO ARCADE EDITION ★', GAME_WIDTH / 2, 156);

    // Instructions Box
    ctx.fillStyle = 'rgba(0, 0, 0, 0.45)';
    roundRect(ctx, 35, 335, 330, 215, 14, true);

    ctx.font = 'bold 15px Arial';
    ctx.fillStyle = '#FFFFFF';
    if (currentMode === MODE.DUEL) {
      ctx.fillText('TAP OR PRESS SPACE TO START DUEL', GAME_WIDTH / 2, 362);
      ctx.font = 'bold 12px Arial';
      ctx.fillStyle = '#FFD700';
      ctx.fillText('Player 1: Space / W / Left Screen Tap', GAME_WIDTH / 2, 390);
      ctx.fillStyle = '#46A0FF';
      ctx.fillText('Player 2: Up Arrow / K / Right Screen Tap', GAME_WIDTH / 2, 412);
    } else {
      ctx.fillText('TAP OR PRESS SPACE TO FLAP', GAME_WIDTH / 2, 362);
      ctx.font = '12px Arial';
      ctx.fillStyle = '#E0E0E0';
      ctx.fillText(`🎮 Difficulty: ${DIFFICULTIES[currentDiffIndex].name} [Tap or 'D']`, GAME_WIDTH / 2, 390);
      ctx.fillText(`🎨 Bird Skin: ${SKINS[currentSkinIndex].name} [Tap or 'C']`, GAME_WIDTH / 2, 412);
    }

    ctx.font = '12px Arial';
    ctx.fillStyle = '#E0E0E0';
    ctx.fillText(`⚔️ Game Mode: ${currentMode === MODE.DUEL ? '2P DUEL' : 'SOLO'} [Press '2']`, GAME_WIDTH / 2, 436);
    ctx.fillText(`🌧️ Weather: ${WEATHERS[currentWeatherIndex]} ['W']  |  🌙 Sky: ${CYCLES[currentCycleIndex]} ['N']`, GAME_WIDTH / 2, 460);
    ctx.fillText(`🏆 High Scores & Database ['L']`, GAME_WIDTH / 2, 484);
    ctx.fillText(`P: Pause | M: Mute Sound`, GAME_WIDTH / 2, 508);

    // Pilot Tag
    ctx.font = 'bold 13px Arial';
    ctx.fillStyle = '#FFFDE7';
    ctx.fillText(`👤 Pilot: ${window.scoreDB.getPlayerName()}`, GAME_WIDTH / 2, 575);

    ctx.restore();
  }

  function drawHUD() {
    ctx.save();

    if (currentMode === MODE.DUEL) {
      // 2-Player HUD
      // Player 1 Card (Top Left)
      ctx.fillStyle = 'rgba(0, 0, 0, 0.6)';
      roundRect(ctx, 15, 14, 110, 52, 10, true);
      ctx.lineWidth = 2;
      ctx.strokeStyle = '#FFD700';
      roundRect(ctx, 15, 14, 110, 52, 10, false);

      ctx.fillStyle = bird1.alive ? '#FFD700' : '#888';
      ctx.font = 'bold 12px Arial';
      ctx.textAlign = 'left';
      const p1Title = bird1.alive ? (score1 > score2 ? '👑 P1 (GOLD)' : 'P1 (GOLD)') : 'P1 (OUT)';
      ctx.fillText(p1Title, 25, 33);

      ctx.fillStyle = '#FFFFFF';
      ctx.font = 'bold 24px "Impact", Arial';
      ctx.fillText(score1.toString(), 25, 57);

      // VS
      ctx.textAlign = 'center';
      ctx.font = 'bold 20px "Impact", Arial';
      ctx.fillStyle = '#FFE066';
      ctx.fillText('VS', GAME_WIDTH / 2, 48);

      // Series Tally
      ctx.font = 'bold 11px Arial';
      ctx.fillStyle = '#FFE066';
      ctx.fillText(`P1 [${duelWins1}] - [${duelWins2}] P2`, GAME_WIDTH / 2, 68);

      // Player 2 Card (Top Right)
      ctx.fillStyle = 'rgba(0, 0, 0, 0.6)';
      roundRect(ctx, GAME_WIDTH - 125, 14, 110, 52, 10, true);
      ctx.lineWidth = 2;
      ctx.strokeStyle = '#46A0FF';
      roundRect(ctx, GAME_WIDTH - 125, 14, 110, 52, 10, false);

      ctx.fillStyle = bird2.alive ? '#46A0FF' : '#888';
      ctx.font = 'bold 12px Arial';
      ctx.textAlign = 'right';
      const p2Title = bird2.alive ? (score2 > score1 ? '👑 P2 (BLUE)' : 'P2 (BLUE)') : 'P2 (OUT)';
      ctx.fillText(p2Title, GAME_WIDTH - 25, 33);

      ctx.fillStyle = '#FFFFFF';
      ctx.font = 'bold 24px "Impact", Arial';
      ctx.fillText(score2.toString(), GAME_WIDTH - 25, 57);

      // Coins
      ctx.textAlign = 'center';
      ctx.font = 'bold 14px Arial';
      ctx.fillStyle = '#FFD700';
      ctx.fillText(`🪙 ${coins}`, GAME_WIDTH / 2, 85);
    } else {
      // Solo HUD
      ctx.textAlign = 'center';
      ctx.font = 'bold 44px "Impact", "Arial Black", sans-serif';
      ctx.fillStyle = '#000000';
      ctx.fillText(score.toString(), GAME_WIDTH / 2 + 2, 62);
      ctx.fillStyle = '#FFFFFF';
      ctx.fillText(score.toString(), GAME_WIDTH / 2, 60);

      // Coins Counter
      ctx.textAlign = 'left';
      ctx.font = 'bold 16px Arial';
      ctx.fillStyle = '#FFD700';
      ctx.fillText(`🪙 ${coins}`, 20, 35);

      // High Score Target
      if (bestScore > 0) {
        ctx.textAlign = 'right';
        ctx.font = 'bold 13px Arial';
        ctx.fillStyle = score > bestScore ? '#00E5FF' : 'rgba(255, 255, 255, 0.75)';
        ctx.fillText(`🏆 BEST: ${bestScore}`, GAME_WIDTH - 20, combo >= 3 ? 55 : 35);
      }

      // Combo Streak
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
    }

    ctx.restore();
  }

  function drawGameOver() {
    ctx.save();
    ctx.textAlign = 'center';

    if (currentMode === MODE.DUEL) {
      // 2P Duel Game Over Banner
      ctx.font = 'bold 32px "Trebuchet MS", sans-serif';
      ctx.fillStyle = '#543847';
      ctx.fillText('DUEL FINISHED!', GAME_WIDTH / 2 + 2, 132);
      ctx.fillStyle = '#FFE066';
      ctx.fillText('DUEL FINISHED!', GAME_WIDTH / 2, 130);

      // Winner text
      const winner = score1 > score2 ? '👑 PLAYER 1 WINS!' : score2 > score1 ? '👑 PLAYER 2 WINS!' : '🤝 IT\'S A DRAW!';
      const winColor = score1 > score2 ? '#FFD700' : score2 > score1 ? '#46A0FF' : '#FFFFFF';
      ctx.font = 'bold 22px Arial';
      ctx.fillStyle = winColor;
      ctx.fillText(winner, GAME_WIDTH / 2, 165);

      // Duel Scorecard
      const px = 35;
      const py = 185;
      const pw = 330;
      const ph = 230;

      ctx.fillStyle = '#DED895';
      roundRect(ctx, px, py, pw, ph, 14, true);
      ctx.lineWidth = 3;
      ctx.strokeStyle = '#543847';
      roundRect(ctx, px, py, pw, ph, 14, false);

      // P1 Column
      ctx.textAlign = 'center';
      ctx.font = 'bold 16px Arial';
      ctx.fillStyle = '#B8860B';
      ctx.fillText('PLAYER 1', px + 80, py + 35);
      ctx.font = 'bold 36px "Impact", Arial';
      ctx.fillStyle = '#000000';
      ctx.fillText(score1.toString(), px + 80, py + 80);
      if (score1 >= score2 && score1 > 0) {
        ctx.font = 'bold 13px Arial';
        ctx.fillStyle = '#2E7D32';
        ctx.fillText('★ WINNER ★', px + 80, py + 108);
      }

      // Divider
      ctx.beginPath();
      ctx.moveTo(px + pw / 2, py + 15);
      ctx.lineTo(px + pw / 2, py + 130);
      ctx.strokeStyle = '#BDB76B';
      ctx.lineWidth = 2;
      ctx.stroke();

      // P2 Column
      ctx.font = 'bold 16px Arial';
      ctx.fillStyle = '#1E5AB4';
      ctx.fillText('PLAYER 2', px + pw - 80, py + 35);
      ctx.font = 'bold 36px "Impact", Arial';
      ctx.fillStyle = '#000000';
      ctx.fillText(score2.toString(), px + pw - 80, py + 80);
      if (score2 >= score1 && score2 > 0) {
        ctx.font = 'bold 13px Arial';
        ctx.fillStyle = '#2E7D32';
        ctx.fillText('★ WINNER ★', px + pw - 80, py + 108);
      }

      // Series total
      ctx.font = 'bold 13px Arial';
      ctx.fillStyle = '#8B4513';
      ctx.fillText(`⚔️ SERIES STANDING: P1 [${duelWins1}] - [${duelWins2}] P2`, GAME_WIDTH / 2, py + 140);

      // Bottom info
      ctx.font = '13px Arial';
      ctx.fillStyle = '#555555';
      ctx.fillText(`🪙 Coins: ${coins}  |  Difficulty: ${DIFFICULTIES[currentDiffIndex].name}`, GAME_WIDTH / 2, py + 162);

      // Database status
      ctx.font = 'bold 12px Arial';
      if (lastScoreRecord && lastScoreRecord.synced) {
        ctx.fillStyle = '#2E7D32';
        ctx.fillText(`✅ Duel Result Recorded in SQLite Database`, GAME_WIDTH / 2, py + 188);
      } else {
        ctx.fillStyle = '#F57F17';
        ctx.fillText(`💾 Duel Result Saved Locally`, GAME_WIDTH / 2, py + 188);
      }

      // Rematch prompt
      ctx.font = 'bold 16px Arial';
      ctx.fillStyle = '#FFFFFF';
      ctx.fillText('TAP OR PRESS SPACE / R FOR REMATCH', GAME_WIDTH / 2, 460);
      ctx.font = '13px Arial';
      ctx.fillStyle = '#FFE066';
      ctx.fillText('Press [2] to switch to Solo Mode', GAME_WIDTH / 2, 490);
    } else {
      // Solo Game Over Banner
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
      ctx.fillText('TAP OR PRESS SPACE / R TO RESTART', GAME_WIDTH / 2, 465);

      ctx.font = '13px Arial';
      ctx.fillStyle = '#FFE066';
      ctx.fillText('Press [2] to play 2-Player Duel Mode', GAME_WIDTH / 2, 495);
    }

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
  function setMode(mode) {
    currentMode = mode;
    const canvasWrap = document.getElementById('canvasWrapper');
    const duelOverlay = document.getElementById('duelTouchOverlay');
    const btnSolo = document.getElementById('btnModeSolo');
    const btnDuel = document.getElementById('btnModeDuel');

    if (btnSolo && btnDuel) {
      if (mode === MODE.DUEL) {
        btnSolo.classList.remove('active');
        btnDuel.classList.add('active');
        if (canvasWrap) canvasWrap.classList.add('duel-active');
      } else {
        btnSolo.classList.add('active');
        btnDuel.classList.remove('active');
        if (canvasWrap) canvasWrap.classList.remove('duel-active');
      }
    }

    if (duelOverlay) {
      duelOverlay.style.display = (mode === MODE.DUEL && currentState === STATE.PLAYING) ? 'flex' : 'none';
    }

    if (currentState === STATE.START || currentState === STATE.GAMEOVER) {
      resetGameForMode();
    }
  }

  function resetGameForMode() {
    if (currentMode === MODE.DUEL) {
      bird1.skinIndex = currentSkinIndex;
      bird2.skinIndex = (currentSkinIndex + 1) % SKINS.length;
      bird1.reset(70);
      bird2.reset(105);
    } else {
      bird1.skinIndex = currentSkinIndex;
      bird1.reset(80);
    }
  }

  function handleActionP1() {
    window.soundEngine.ensureContext();
    if (currentState === STATE.START || currentState === STATE.GAMEOVER) {
      startNewGame();
    } else if (currentState === STATE.PLAYING) {
      if (bird1.alive) bird1.jump();
    }
  }

  function handleActionP2() {
    window.soundEngine.ensureContext();
    if (currentState === STATE.START || currentState === STATE.GAMEOVER) {
      startNewGame();
    } else if (currentState === STATE.PLAYING) {
      if (currentMode === MODE.DUEL && bird2.alive) {
        bird2.jump();
      } else if (currentMode === MODE.SOLO && bird1.alive) {
        bird1.jump();
      }
    }
  }

  function startNewGame() {
    currentState = STATE.PLAYING;
    score = 0;
    score1 = 0;
    score2 = 0;
    coins = 0;
    pipesPassed = 0;
    combo = 0;
    maxCombo = 0;
    closeShaves = 0;
    gameStartTime = Date.now();
    lastScoreRecord = null;
    hasAnnouncedRecord = false;

    resetGameForMode();
    pipes.reset();
    powerUps.reset();

    bird1.jump();
    if (currentMode === MODE.DUEL) {
      bird2.jump();
    }

    const duelOverlay = document.getElementById('duelTouchOverlay');
    if (duelOverlay) {
      duelOverlay.style.display = currentMode === MODE.DUEL ? 'flex' : 'none';
    }
  }

  // Keyboard controls
  window.addEventListener('keydown', e => {
    if (e.repeat && (e.code === 'Space' || e.code === 'KeyW' || e.code === 'ArrowUp' || e.code === 'KeyK' || e.code === 'Digit2' || e.code === 'Tab')) {
      return;
    }
    if (e.code === 'Space' || e.code === 'KeyW') {
      e.preventDefault();
      handleActionP1();
    } else if (e.code === 'ArrowUp' || e.code === 'KeyK') {
      e.preventDefault();
      handleActionP2();
    } else if (e.code === 'Digit2' || e.code === 'Tab') {
      e.preventDefault();
      setMode(currentMode === MODE.SOLO ? MODE.DUEL : MODE.SOLO);
      addPopup(GAME_WIDTH / 2, 330, `MODE: ${currentMode === MODE.DUEL ? '2P DUEL' : 'SOLO'}`, '#00E5FF');
    } else if (e.code === 'KeyD') {
      cycleDifficulty();
    } else if (e.code === 'KeyC') {
      cycleSkin();
    } else if (e.code === 'KeyW' && currentState !== STATE.PLAYING) {
      cycleWeather();
    } else if (e.code === 'KeyN') {
      cycleCycle();
    } else if (e.code === 'KeyM') {
      window.soundEngine.toggleMute();
      updateMuteButton();
    } else if (e.code === 'KeyP' && currentState === STATE.PLAYING) {
      isPaused = !isPaused;
    } else if (e.code === 'KeyR' && (currentState === STATE.GAMEOVER || currentState === STATE.DYING)) {
      e.preventDefault();
      startNewGame();
    } else if (e.code === 'KeyL') {
      if (typeof window.openLeaderboardModal === 'function') {
        window.openLeaderboardModal();
      }
    }
  });

  // Touch & Pointer controls (Mobile / Android / Click)
  canvas.addEventListener('pointerdown', e => {
    e.preventDefault();
    const rect = canvas.getBoundingClientRect();
    const clickX = e.clientX - rect.left;

    if (currentMode === MODE.DUEL && currentState === STATE.PLAYING) {
      if (clickX < rect.width / 2) {
        handleActionP1();
      } else {
        handleActionP2();
      }
    } else {
      handleActionP1();
    }
  }, { passive: false });

  function cycleDifficulty() {
    currentDiffIndex = (currentDiffIndex + 1) % DIFFICULTIES.length;
    addPopup(GAME_WIDTH / 2, 330, `DIFFICULTY: ${DIFFICULTIES[currentDiffIndex].name}`, DIFFICULTIES[currentDiffIndex].color);
  }

  function cycleSkin() {
    currentSkinIndex = (currentSkinIndex + 1) % SKINS.length;
    bird1.skinIndex = currentSkinIndex;
    bird2.skinIndex = (currentSkinIndex + 1) % SKINS.length;
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

      // 5. Bird(s)
      bird1.update(dt);
      bird1.draw();
      if (currentMode === MODE.DUEL) {
        bird2.update(dt);
        bird2.draw();
      }

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
    handleAction: handleActionP1,
    handleActionP1,
    handleActionP2,
    setMode,
    getMode: () => currentMode,
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
