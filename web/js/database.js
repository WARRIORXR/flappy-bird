// ─── Flappy Bird Hybrid Database Client (SQLite Server + Offline IndexedDB) ──
class ScoreDatabaseClient {
  constructor() {
    this.dbName = 'FlappyScoresDB';
    this.dbVersion = 1;
    this.db = null;
    this.serverOnline = false;
    this.serverUrl = window.location.origin;
    this.listeners = [];

    this.initIndexedDB();
    this.checkServer();
    setInterval(() => this.checkServer(), 10000);
  }

  onStatusChange(fn) {
    this.listeners.push(fn);
  }

  notifyStatus() {
    this.listeners.forEach(fn => fn({ online: this.serverOnline }));
  }

  async checkServer() {
    try {
      const res = await fetch(`${this.serverUrl}/api/health`, {
        method: 'GET',
        cache: 'no-cache',
      });
      const wasOnline = this.serverOnline;
      this.serverOnline = res.ok;
      if (this.serverOnline !== wasOnline) {
        this.notifyStatus();
        if (this.serverOnline) {
          this.syncPendingScores();
        }
      }
    } catch {
      if (this.serverOnline) {
        this.serverOnline = false;
        this.notifyStatus();
      }
    }
  }

  initIndexedDB() {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(this.dbName, this.dbVersion);

      request.onupgradeneeded = event => {
        const db = event.target.result;
        if (!db.objectStoreNames.contains('scores')) {
          const store = db.createObjectStore('scores', { keyPath: 'id', autoIncrement: true });
          store.createIndex('score', 'score', { unique: false });
          store.createIndex('difficulty', 'difficulty', { unique: false });
          store.createIndex('playerName', 'playerName', { unique: false });
          store.createIndex('createdAt', 'createdAt', { unique: false });
        }
      };

      request.onsuccess = event => {
        this.db = event.target.result;
        resolve(this.db);
      };

      request.onerror = err => {
        console.warn('IndexedDB unavailable, falling back to localStorage:', err);
        resolve(null);
      };
    });
  }

  getPlayerName() {
    return localStorage.getItem('flappy_player_name') || 'SkyFlapper';
  }

  setPlayerName(name) {
    const clean = (name || 'SkyFlapper').trim().slice(0, 25);
    localStorage.setItem('flappy_player_name', clean);
    return clean;
  }

  async saveScore(entry) {
    const data = {
      playerName: entry.playerName || this.getPlayerName(),
      score: Math.max(0, parseInt(entry.score, 10) || 0),
      coins: Math.max(0, parseInt(entry.coins, 10) || 0),
      pipesPassed: Math.max(0, parseInt(entry.pipesPassed, 10) || 0),
      difficulty: entry.difficulty || 'Normal',
      skin: entry.skin || 'Classic',
      weather: entry.weather || 'Sunny',
      medal: entry.medal || 'None',
      combo: Math.max(0, parseInt(entry.combo, 10) || 0),
      closeShaves: Math.max(0, parseInt(entry.closeShaves, 10) || 0),
      durationSeconds: Math.max(0, parseInt(entry.durationSeconds, 10) || 0),
      createdAt: new Date().toISOString(),
      synced: false,
    };

    // 1. Save locally in IndexedDB
    let localId = null;
    if (this.db) {
      localId = await new Promise(resolve => {
        const tx = this.db.transaction('scores', 'readwrite');
        const store = tx.objectStore('scores');
        const req = store.add(data);
        req.onsuccess = () => resolve(req.result);
        req.onerror = () => resolve(null);
      });
    }

    // 2. Post to SQLite server
    try {
      const response = await fetch(`${this.serverUrl}/api/scores`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data),
      });

      if (response.ok) {
        const result = await response.json();
        this.serverOnline = true;
        this.notifyStatus();

        if (localId && this.db) {
          const tx = this.db.transaction('scores', 'readwrite');
          const store = tx.objectStore('scores');
          const req = store.get(localId);
          req.onsuccess = () => {
            const item = req.result;
            if (item) {
              item.synced = true;
              item.serverId = result.record?.id;
              store.put(item);
            }
          };
        }

        return { ...data, id: localId, rank: result.record?.rank, synced: true };
      }
    } catch {
      this.serverOnline = false;
      this.notifyStatus();
    }

    return { ...data, id: localId, synced: false };
  }

  async syncPendingScores() {
    if (!this.db || !this.serverOnline) return;

    const tx = this.db.transaction('scores', 'readwrite');
    const store = tx.objectStore('scores');
    const req = store.getAll();

    req.onsuccess = async () => {
      const records = req.result || [];
      const unsynced = records.filter(r => !r.synced);

      for (const item of unsynced) {
        try {
          const res = await fetch(`${this.serverUrl}/api/scores`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(item),
          });
          if (res.ok) {
            item.synced = true;
            const updateTx = this.db.transaction('scores', 'readwrite');
            updateTx.objectStore('scores').put(item);
          }
        } catch {
          break; // server went offline
        }
      }
    };
  }

  async getLeaderboard(difficulty = 'all', limit = 25) {
    // If online, fetch authoritative rankings from SQLite
    if (this.serverOnline) {
      try {
        const query = new URLSearchParams({ difficulty, limit, sort: 'highest' });
        const res = await fetch(`${this.serverUrl}/api/scores?${query.toString()}`);
        if (res.ok) {
          const json = await res.json();
          if (json.scores && json.scores.length > 0) {
            return json.scores.map(s => ({
              id: s.id,
              playerName: s.player_name,
              score: s.score,
              coins: s.coins,
              pipesPassed: s.pipes_passed,
              difficulty: s.difficulty,
              skin: s.skin,
              weather: s.weather,
              medal: s.medal,
              combo: s.combo,
              createdAt: s.created_at,
              rank: s.rank,
            }));
          }
        }
      } catch {
        this.serverOnline = false;
        this.notifyStatus();
      }
    }

    // Fallback to local IndexedDB
    return this.getLocalLeaderboard(difficulty, limit);
  }

  getLocalLeaderboard(difficulty = 'all', limit = 25) {
    return new Promise(resolve => {
      if (!this.db) return resolve([]);
      const tx = this.db.transaction('scores', 'readonly');
      const store = tx.objectStore('scores');
      const req = store.getAll();

      req.onsuccess = () => {
        let list = req.result || [];
        if (difficulty && difficulty.toLowerCase() !== 'all') {
          list = list.filter(item => (item.difficulty || '').toLowerCase() === difficulty.toLowerCase());
        }
        list.sort((a, b) => b.score - a.score || b.coins - a.coins);
        list = list.slice(0, limit);
        list.forEach((item, index) => {
          item.rank = index + 1;
        });
        resolve(list);
      };

      req.onerror = () => resolve([]);
    });
  }

  async getRecentGames(limit = 15) {
    if (this.serverOnline) {
      try {
        const res = await fetch(`${this.serverUrl}/api/scores/recent?limit=${limit}`);
        if (res.ok) {
          const json = await res.json();
          if (json.scores) {
            return json.scores.map(s => ({
              id: s.id,
              playerName: s.player_name,
              score: s.score,
              coins: s.coins,
              difficulty: s.difficulty,
              skin: s.skin,
              weather: s.weather,
              medal: s.medal,
              combo: s.combo,
              createdAt: s.created_at,
            }));
          }
        }
      } catch {
        this.serverOnline = false;
        this.notifyStatus();
      }
    }

    return new Promise(resolve => {
      if (!this.db) return resolve([]);
      const tx = this.db.transaction('scores', 'readonly');
      const store = tx.objectStore('scores');
      const req = store.getAll();

      req.onsuccess = () => {
        let list = req.result || [];
        list.sort((a, b) => new Date(b.createdAt) - new Date(a.createdAt));
        resolve(list.slice(0, limit));
      };

      req.onerror = () => resolve([]);
    });
  }

  async getStats() {
    if (this.serverOnline) {
      try {
        const res = await fetch(`${this.serverUrl}/api/stats`);
        if (res.ok) {
          const json = await res.json();
          if (json.stats) return json.stats;
        }
      } catch {
        this.serverOnline = false;
        this.notifyStatus();
      }
    }

    // Local stats
    return new Promise(resolve => {
      if (!this.db) return resolve({ totalGames: 0, highScore: 0, totalCoins: 0, avgScore: 0 });
      const tx = this.db.transaction('scores', 'readonly');
      const store = tx.objectStore('scores');
      const req = store.getAll();

      req.onsuccess = () => {
        const list = req.result || [];
        if (list.length === 0) {
          return resolve({ totalGames: 0, highScore: 0, totalCoins: 0, avgScore: 0, topPlayer: 'None' });
        }

        const totalScore = list.reduce((sum, item) => sum + (item.score || 0), 0);
        const totalCoins = list.reduce((sum, item) => sum + (item.coins || 0), 0);
        const highScore = Math.max(...list.map(i => i.score || 0), 0);
        const avgScore = Math.round((totalScore / list.length) * 10) / 10;

        resolve({
          totalGames: list.length,
          highScore,
          totalScore,
          totalCoins,
          avgScore,
          topPlayer: this.getPlayerName(),
        });
      };

      req.onerror = () => resolve({ totalGames: 0, highScore: 0, totalCoins: 0, avgScore: 0 });
    });
  }

  async exportAll() {
    if (this.serverOnline) {
      try {
        const res = await fetch(`${this.serverUrl}/api/export`);
        if (res.ok) {
          const data = await res.json();
          this.downloadJson(data, 'flappy_scores_database_server.json');
          return;
        }
      } catch {}
    }

    if (!this.db) return;
    const tx = this.db.transaction('scores', 'readonly');
    const store = tx.objectStore('scores');
    const req = store.getAll();

    req.onsuccess = () => {
      const data = {
        exportedAt: new Date().toISOString(),
        source: 'IndexedDB',
        scores: req.result || [],
      };
      this.downloadJson(data, 'flappy_scores_local_backup.json');
    };
  }

  downloadJson(obj, filename) {
    const blob = new Blob([JSON.stringify(obj, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  async resetDatabase() {
    if (this.serverOnline) {
      try {
        await fetch(`${this.serverUrl}/api/scores/reset`, { method: 'POST' });
      } catch {}
    }

    if (this.db) {
      const tx = this.db.transaction('scores', 'readwrite');
      tx.objectStore('scores').clear();
    }
  }
}

// Global singleton instance
window.scoreDB = new ScoreDatabaseClient();
