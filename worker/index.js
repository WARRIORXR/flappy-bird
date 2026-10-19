// ─── Cloudflare Worker Edge Backend for Flappy Bird Pro ────────────────────
let memoryScores = [
  { id: 1, player_name: 'SkyAce', score: 58, coins: 26, pipes_passed: 52, difficulty: 'Hard', skin: 'Purple', weather: 'Night', medal: 'Platinum', combo: 18, close_shaves: 6, duration_seconds: 94, created_at: new Date().toISOString() },
  { id: 2, player_name: 'FlappyLord', score: 42, coins: 18, pipes_passed: 39, difficulty: 'Normal', skin: 'Classic', weather: 'Sunset', medal: 'Gold', combo: 12, close_shaves: 4, duration_seconds: 70, created_at: new Date().toISOString() },
  { id: 3, player_name: 'RustyWing', score: 34, coins: 14, pipes_passed: 32, difficulty: 'Normal', skin: 'Red', weather: 'Day', medal: 'Gold', combo: 9, close_shaves: 3, duration_seconds: 58, created_at: new Date().toISOString() },
  { id: 4, player_name: 'PixelPilot', score: 25, coins: 10, pipes_passed: 24, difficulty: 'Easy', skin: 'Blue', weather: 'Rain', medal: 'Silver', combo: 7, close_shaves: 2, duration_seconds: 45, created_at: new Date().toISOString() },
  { id: 5, player_name: 'BreezyBird', score: 18, coins: 8, pipes_passed: 17, difficulty: 'Easy', skin: 'Green', weather: 'Day', medal: 'Bronze', combo: 5, close_shaves: 1, duration_seconds: 32, created_at: new Date().toISOString() },
];

function jsonResponse(data, status = 200) {
  return new Response(JSON.stringify(data), {
    status,
    headers: {
      'Content-Type': 'application/json',
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, POST, DELETE, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type',
      'Cache-Control': 'no-cache',
    },
  });
}

export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);

    // Handle CORS preflight
    if (request.method === 'OPTIONS') {
      return new Response(null, {
        status: 204,
        headers: {
          'Access-Control-Allow-Origin': '*',
          'Access-Control-Allow-Methods': 'GET, POST, DELETE, OPTIONS',
          'Access-Control-Allow-Headers': 'Content-Type',
        },
      });
    }

    // ─── API Routes ─────────────────────────────────────────────────────────
    if (url.pathname.startsWith('/api/')) {
      if (url.pathname === '/api/health') {
        return jsonResponse({
          status: 'ok',
          provider: 'cloudflare-workers',
          edgeRegion: request.cf?.colo || 'global',
          timestamp: new Date().toISOString(),
        });
      }

      if (url.pathname === '/api/scores' && request.method === 'GET') {
        const difficulty = url.searchParams.get('difficulty') || 'all';
        const limit = parseInt(url.searchParams.get('limit') || '25', 10);
        let list = [...memoryScores];

        if (difficulty && difficulty.toLowerCase() !== 'all') {
          list = list.filter(s => (s.difficulty || '').toLowerCase() === difficulty.toLowerCase());
        }

        list.sort((a, b) => b.score - a.score || b.coins - a.coins);
        list = list.slice(0, limit).map((s, idx) => ({ ...s, rank: idx + 1 }));

        return jsonResponse({ success: true, scores: list, count: list.length });
      }

      if (url.pathname === '/api/scores/recent' && request.method === 'GET') {
        const limit = parseInt(url.searchParams.get('limit') || '15', 10);
        const list = [...memoryScores].sort((a, b) => new Date(b.created_at) - new Date(a.created_at)).slice(0, limit);
        return jsonResponse({ success: true, scores: list, count: list.length });
      }

      if (url.pathname === '/api/stats' && request.method === 'GET') {
        const totalGames = memoryScores.length;
        const highScore = Math.max(...memoryScores.map(s => s.score), 0);
        const totalCoins = memoryScores.reduce((sum, s) => sum + (s.coins || 0), 0);
        const totalPipes = memoryScores.reduce((sum, s) => sum + (s.pipes_passed || 0), 0);
        const totalScore = memoryScores.reduce((sum, s) => sum + s.score, 0);
        const avgScore = totalGames > 0 ? Math.round((totalScore / totalGames) * 10) / 10 : 0;
        const topPlayer = memoryScores.length > 0 ? [...memoryScores].sort((a, b) => b.score - a.score)[0].player_name : 'None';

        return jsonResponse({
          success: true,
          stats: {
            totalGames,
            highScore,
            totalCoins,
            totalPipes,
            avgScore,
            topPlayer,
            topPlayerScore: highScore,
          },
        });
      }

      if (url.pathname === '/api/scores' && request.method === 'POST') {
        try {
          const body = await request.json();
          const newEntry = {
            id: memoryScores.length + 1,
            player_name: (body.playerName || 'Player').slice(0, 30),
            score: Math.max(0, parseInt(body.score, 10) || 0),
            coins: Math.max(0, parseInt(body.coins, 10) || 0),
            pipes_passed: Math.max(0, parseInt(body.pipesPassed, 10) || 0),
            difficulty: body.difficulty || 'Normal',
            skin: body.skin || 'Classic',
            weather: body.weather || 'Sunny',
            medal: body.medal || 'None',
            combo: Math.max(0, parseInt(body.combo, 10) || 0),
            close_shaves: Math.max(0, parseInt(body.closeShaves, 10) || 0),
            duration_seconds: Math.max(0, parseInt(body.durationSeconds, 10) || 0),
            created_at: new Date().toISOString(),
          };

          memoryScores.push(newEntry);
          const higherCount = memoryScores.filter(s => s.score > newEntry.score).length;
          newEntry.rank = higherCount + 1;

          return jsonResponse({ success: true, record: newEntry }, 201);
        } catch (err) {
          return jsonResponse({ success: false, error: 'Invalid JSON payload' }, 400);
        }
      }

      return jsonResponse({ success: false, error: 'Not found' }, 404);
    }

    // ─── Static Assets Fallback (Cloudflare Static Assets) ───────────────────
    if (env.ASSETS) {
      return env.ASSETS.fetch(request);
    }

    return new Response('Asset not found', { status: 404 });
  },
};
