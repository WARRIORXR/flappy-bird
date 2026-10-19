const { DatabaseSync } = require('node:sqlite');
const path = require('node:path');
const fs = require('node:fs');

class ScoreDatabase {
  constructor(dbPath) {
    const dataDir = path.dirname(dbPath);
    if (!fs.existsSync(dataDir)) {
      fs.mkdirSync(dataDir, { recursive: true });
    }

    this.db = new DatabaseSync(dbPath);
    this.init();
  }

  init() {
    this.db.exec(`
      CREATE TABLE IF NOT EXISTS scores (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        player_name TEXT NOT NULL,
        score INTEGER NOT NULL,
        coins INTEGER DEFAULT 0,
        pipes_passed INTEGER DEFAULT 0,
        difficulty TEXT NOT NULL DEFAULT 'Normal',
        skin TEXT NOT NULL DEFAULT 'Classic',
        weather TEXT NOT NULL DEFAULT 'Sunny',
        medal TEXT DEFAULT 'None',
        combo INTEGER DEFAULT 0,
        close_shaves INTEGER DEFAULT 0,
        duration_seconds INTEGER DEFAULT 0,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
      );

      CREATE INDEX IF NOT EXISTS idx_scores_score ON scores(score DESC);
      CREATE INDEX IF NOT EXISTS idx_scores_difficulty ON scores(difficulty);
      CREATE INDEX IF NOT EXISTS idx_scores_player ON scores(player_name);

      CREATE TABLE IF NOT EXISTS players (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        player_name TEXT UNIQUE NOT NULL,
        high_score INTEGER DEFAULT 0,
        total_games INTEGER DEFAULT 0,
        total_score INTEGER DEFAULT 0,
        total_coins INTEGER DEFAULT 0,
        favorite_skin TEXT DEFAULT 'Classic',
        last_played_at DATETIME DEFAULT CURRENT_TIMESTAMP
      );

      CREATE INDEX IF NOT EXISTS idx_players_high_score ON players(high_score DESC);
    `);

    // Pre-seed demo records if completely empty
    const countRow = this.db.prepare('SELECT COUNT(*) as count FROM scores').get();
    if (countRow && countRow.count === 0) {
      this.seedInitialData();
    }
  }

  seedInitialData() {
    const demoEntries = [
      { name: 'SkyAce', score: 58, coins: 26, pipes: 52, diff: 'Hard', skin: 'Purple', weather: 'Night', medal: 'Platinum', combo: 18, shaves: 6, dur: 94 },
      { name: 'FlappyLord', score: 42, coins: 18, pipes: 39, diff: 'Normal', skin: 'Classic', weather: 'Sunset', medal: 'Gold', combo: 12, shaves: 4, dur: 70 },
      { name: 'RustyWing', score: 34, coins: 14, pipes: 32, diff: 'Normal', skin: 'Red', weather: 'Day', medal: 'Gold', combo: 9, shaves: 3, dur: 58 },
      { name: 'PixelPilot', score: 25, coins: 10, pipes: 24, diff: 'Easy', skin: 'Blue', weather: 'Rain', medal: 'Silver', combo: 7, shaves: 2, dur: 45 },
      { name: 'BreezyBird', score: 18, coins: 8, pipes: 17, diff: 'Easy', skin: 'Green', weather: 'Day', medal: 'Bronze', combo: 5, shaves: 1, dur: 32 },
    ];

    demoEntries.forEach(entry => {
      this.addScore({
        playerName: entry.name,
        score: entry.score,
        coins: entry.coins,
        pipesPassed: entry.pipes,
        difficulty: entry.diff,
        skin: entry.skin,
        weather: entry.weather,
        medal: entry.medal,
        combo: entry.combo,
        closeShaves: entry.shaves,
        durationSeconds: entry.dur,
      });
    });
  }

  addScore({
    playerName = 'Flapper',
    score = 0,
    coins = 0,
    pipesPassed = 0,
    difficulty = 'Normal',
    skin = 'Classic',
    weather = 'Sunny',
    medal = 'None',
    combo = 0,
    closeShaves = 0,
    durationSeconds = 0,
  }) {
    const trimmedName = (playerName || 'Player').trim().slice(0, 30) || 'Player';
    const numScore = Math.max(0, parseInt(score, 10) || 0);
    const numCoins = Math.max(0, parseInt(coins, 10) || 0);
    const numPipes = Math.max(0, parseInt(pipesPassed, 10) || 0);
    const numCombo = Math.max(0, parseInt(combo, 10) || 0);
    const numShaves = Math.max(0, parseInt(closeShaves, 10) || 0);
    const numDuration = Math.max(0, parseInt(durationSeconds, 10) || 0);

    // Insert score record
    const insertStmt = this.db.prepare(`
      INSERT INTO scores (
        player_name, score, coins, pipes_passed, difficulty, skin, weather,
        medal, combo, close_shaves, duration_seconds
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `);

    const result = insertStmt.run(
      trimmedName,
      numScore,
      numCoins,
      numPipes,
      difficulty,
      skin,
      weather,
      medal,
      numCombo,
      numShaves,
      numDuration
    );

    const insertedId = result.lastInsertRowid;

    // Update or create player aggregate entry
    const existingPlayer = this.db.prepare('SELECT * FROM players WHERE player_name = ?').get(trimmedName);

    if (existingPlayer) {
      const newHighScore = Math.max(existingPlayer.high_score, numScore);
      const newTotalGames = existingPlayer.total_games + 1;
      const newTotalScore = existingPlayer.total_score + numScore;
      const newTotalCoins = existingPlayer.total_coins + numCoins;

      this.db.prepare(`
        UPDATE players
        SET high_score = ?, total_games = ?, total_score = ?, total_coins = ?, favorite_skin = ?, last_played_at = CURRENT_TIMESTAMP
        WHERE player_name = ?
      `).run(newHighScore, newTotalGames, newTotalScore, newTotalCoins, skin, trimmedName);
    } else {
      this.db.prepare(`
        INSERT INTO players (player_name, high_score, total_games, total_score, total_coins, favorite_skin)
        VALUES (?, ?, 1, ?, ?, ?)
      `).run(trimmedName, numScore, numScore, numCoins, skin);
    }

    // Determine current global rank of this score
    const rankRow = this.db.prepare('SELECT COUNT(*) as rank FROM scores WHERE score > ?').get(numScore);
    const rank = (rankRow ? rankRow.rank : 0) + 1;

    return {
      id: Number(insertedId),
      playerName: trimmedName,
      score: numScore,
      coins: numCoins,
      pipesPassed: numPipes,
      difficulty,
      skin,
      weather,
      medal,
      combo: numCombo,
      closeShaves: numShaves,
      durationSeconds: numDuration,
      rank,
      timestamp: new Date().toISOString(),
    };
  }

  getScores({ difficulty = 'all', limit = 25, sort = 'highest', player = null } = {}) {
    const lim = Math.min(Math.max(1, parseInt(limit, 10) || 25), 100);
    let sql = 'SELECT * FROM scores';
    const params = [];
    const whereClauses = [];

    if (difficulty && difficulty.toLowerCase() !== 'all') {
      whereClauses.push('LOWER(difficulty) = LOWER(?)');
      params.push(difficulty);
    }

    if (player) {
      whereClauses.push('LOWER(player_name) = LOWER(?)');
      params.push(player);
    }

    if (whereClauses.length > 0) {
      sql += ' WHERE ' + whereClauses.join(' AND ');
    }

    if (sort === 'recent') {
      sql += ' ORDER BY created_at DESC, id DESC';
    } else {
      sql += ' ORDER BY score DESC, coins DESC, id DESC';
    }

    sql += ' LIMIT ?';
    params.push(lim);

    const rows = this.db.prepare(sql).all(...params);
    return rows.map((row, idx) => ({
      ...row,
      rank: sort === 'highest' ? idx + 1 : undefined,
    }));
  }

  getRecentScores(limit = 15) {
    return this.getScores({ limit, sort: 'recent' });
  }

  getStats() {
    const totalRuns = this.db.prepare('SELECT COUNT(*) as count FROM scores').get().count;
    const highestRow = this.db.prepare('SELECT MAX(score) as max_score FROM scores').get();
    const highScore = highestRow ? highestRow.max_score || 0 : 0;

    const aggregateRow = this.db.prepare(`
      SELECT 
        SUM(score) as total_score,
        SUM(coins) as total_coins,
        SUM(pipes_passed) as total_pipes,
        AVG(score) as avg_score
      FROM scores
    `).get();

    const topPlayerRow = this.db.prepare(`
      SELECT player_name, high_score, total_games
      FROM players
      ORDER BY high_score DESC, total_score DESC
      LIMIT 1
    `).get();

    const diffCounts = this.db.prepare(`
      SELECT difficulty, COUNT(*) as count, MAX(score) as best
      FROM scores
      GROUP BY difficulty
    `).all();

    return {
      totalGames: totalRuns,
      highScore,
      totalScore: aggregateRow.total_score || 0,
      totalCoins: aggregateRow.total_coins || 0,
      totalPipes: aggregateRow.total_pipes || 0,
      avgScore: Math.round((aggregateRow.avg_score || 0) * 10) / 10,
      topPlayer: topPlayerRow ? topPlayerRow.player_name : 'None',
      topPlayerScore: topPlayerRow ? topPlayerRow.high_score : 0,
      difficultyStats: diffCounts,
    };
  }

  getPlayers(limit = 20) {
    const lim = Math.min(Math.max(1, parseInt(limit, 10) || 20), 100);
    return this.db.prepare(`
      SELECT player_name, high_score, total_games, total_score, total_coins, favorite_skin, last_played_at
      FROM players
      ORDER BY high_score DESC, total_score DESC
      LIMIT ?
    `).all(lim);
  }

  deleteScore(id) {
    const numId = parseInt(id, 10);
    if (!numId) return false;
    const res = this.db.prepare('DELETE FROM scores WHERE id = ?').run(numId);
    return res.changes > 0;
  }

  reset() {
    this.db.exec('DELETE FROM scores; DELETE FROM players;');
    this.seedInitialData();
    return true;
  }

  exportAll() {
    const allScores = this.db.prepare('SELECT * FROM scores ORDER BY id ASC').all();
    const allPlayers = this.db.prepare('SELECT * FROM players ORDER BY id ASC').all();
    return {
      exportedAt: new Date().toISOString(),
      scores: allScores,
      players: allPlayers,
    };
  }
}

module.exports = { ScoreDatabase };
