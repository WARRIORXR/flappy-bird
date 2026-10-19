const http = require('node:http');
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const { ScoreDatabase } = require('./db');

const PORT = parseInt(process.env.PORT, 10) || 3030;
const DB_PATH = path.join(__dirname, '..', 'data', 'flappy_scores.db');
const WEB_ROOT = path.join(__dirname, '..', 'web');

const db = new ScoreDatabase(DB_PATH);

const MIME_TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.js': 'application/javascript; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.wasm': 'application/wasm',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.svg': 'image/svg+xml',
  '.ico': 'image/x-icon',
  '.txt': 'text/plain; charset=utf-8',
  '.wav': 'audio/wav',
};

function getLocalIpAddresses() {
  const interfaces = os.networkInterfaces();
  const addresses = [];
  for (const name of Object.keys(interfaces)) {
    for (const iface of interfaces[name]) {
      if (iface.family === 'IPv4' && !iface.internal) {
        addresses.push(iface.address);
      }
    }
  }
  return addresses;
}

function sendJson(res, statusCode, data) {
  res.writeHead(statusCode, {
    'Content-Type': 'application/json',
    'Access-Control-Allow-Origin': '*',
    'Access-Control-Allow-Methods': 'GET, POST, DELETE, OPTIONS',
    'Access-Control-Allow-Headers': 'Content-Type',
    'Cache-Control': 'no-cache',
  });
  res.end(JSON.stringify(data));
}

function readBody(req) {
  return new Promise((resolve, reject) => {
    let body = '';
    req.on('data', chunk => {
      body += chunk;
      if (body.length > 1e6) {
        req.destroy();
        reject(new Error('Payload too large'));
      }
    });
    req.on('end', () => {
      try {
        resolve(body ? JSON.parse(body) : {});
      } catch (err) {
        reject(err);
      }
    });
    req.on('error', reject);
  });
}

const server = http.createServer(async (req, res) => {
  const urlObj = new URL(req.url, `http://${req.headers.host || 'localhost'}`);
  const pathname = urlObj.pathname;
  const searchParams = urlObj.searchParams;

  // Handle CORS preflight
  if (req.method === 'OPTIONS') {
    res.writeHead(204, {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, POST, DELETE, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type',
      'Access-Control-Max-Age': '86400',
    });
    res.end();
    return;
  }

  // ─── API Routes ─────────────────────────────────────────────────────────────
  if (pathname.startsWith('/api/')) {
    try {
      if (req.method === 'GET' && pathname === '/api/health') {
        return sendJson(res, 200, {
          status: 'ok',
          database: 'sqlite',
          version: '1.0.0',
          uptime: process.uptime(),
        });
      }

      if (req.method === 'GET' && pathname === '/api/scores') {
        const difficulty = searchParams.get('difficulty') || 'all';
        const limit = parseInt(searchParams.get('limit'), 10) || 25;
        const sort = searchParams.get('sort') || 'highest';
        const player = searchParams.get('player') || null;

        const scores = db.getScores({ difficulty, limit, sort, player });
        return sendJson(res, 200, { success: true, scores, count: scores.length });
      }

      if (req.method === 'GET' && pathname === '/api/scores/recent') {
        const limit = parseInt(searchParams.get('limit'), 10) || 15;
        const scores = db.getRecentScores(limit);
        return sendJson(res, 200, { success: true, scores, count: scores.length });
      }

      if (req.method === 'GET' && pathname === '/api/stats') {
        const stats = db.getStats();
        return sendJson(res, 200, { success: true, stats });
      }

      if (req.method === 'GET' && pathname === '/api/players') {
        const limit = parseInt(searchParams.get('limit'), 10) || 20;
        const players = db.getPlayers(limit);
        return sendJson(res, 200, { success: true, players, count: players.length });
      }

      if (req.method === 'POST' && pathname === '/api/scores') {
        const body = await readBody(req);
        if (body.score === undefined || body.score === null) {
          return sendJson(res, 400, { success: false, error: 'Field "score" is required' });
        }

        const record = db.addScore(body);
        return sendJson(res, 201, { success: true, record });
      }

      if (req.method === 'DELETE' && pathname.startsWith('/api/scores/')) {
        const id = pathname.split('/')[3];
        const deleted = db.deleteScore(id);
        return sendJson(res, deleted ? 200 : 404, { success: deleted });
      }

      if (req.method === 'POST' && pathname === '/api/scores/reset') {
        db.reset();
        return sendJson(res, 200, { success: true, message: 'Database reset successfully with seed entries' });
      }

      if (req.method === 'GET' && pathname === '/api/export') {
        const data = db.exportAll();
        return sendJson(res, 200, data);
      }

      return sendJson(res, 404, { success: false, error: 'Endpoint not found' });
    } catch (err) {
      console.error('API Error:', err);
      return sendJson(res, 500, { success: false, error: err.message });
    }
  }

  // ─── Static File Serving ───────────────────────────────────────────────────
  let safePath = path.normalize(pathname).replace(/^(\.\.[\/\\])+/, '');
  if (safePath === '/' || safePath === '\\') {
    safePath = '/index.html';
  }

  let filePath = path.join(WEB_ROOT, safePath);

  // If trying to access directory without trailing index.html
  if (fs.existsSync(filePath) && fs.statSync(filePath).isDirectory()) {
    filePath = path.join(filePath, 'index.html');
  }

  fs.stat(filePath, (err, stats) => {
    if (err || !stats.isFile()) {
      res.writeHead(404, { 'Content-Type': 'text/plain' });
      res.end('404 Not Found');
      return;
    }

    const ext = path.extname(filePath).toLowerCase();
    const contentType = MIME_TYPES[ext] || 'application/octet-stream';

    res.writeHead(200, {
      'Content-Type': contentType,
      'Content-Length': stats.size,
      'Access-Control-Allow-Origin': '*',
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
      'Cache-Control': ext === '.html' ? 'no-cache' : 'public, max-age=3600',
    });

    const stream = fs.createReadStream(filePath);
    stream.pipe(res);
  });
});

server.listen(PORT, '0.0.0.0', () => {
  const ips = getLocalIpAddresses();
  console.log('='.repeat(60));
  console.log('🐦 FLAPPY BIRD - WEB & MOBILE DATABASE SERVER 🚀');
  console.log('='.repeat(60));
  console.log(`Local Access:        http://localhost:${PORT}`);
  ips.forEach(ip => {
    console.log(`Android / Network:   http://${ip}:${PORT}`);
  });
  console.log(`Database File:       ${DB_PATH}`);
  console.log(`Web Assets Root:     ${WEB_ROOT}`);
  console.log('='.repeat(60));
});

module.exports = server;
