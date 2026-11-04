// Nether exile check (server_only.sh <label> <mc version> <loader> exile_bot.js exile): the exile distance is counted in
// world blocks, so a respawn anchor in the Nether sends you 8 times fewer Nether blocks away than a bed sends you
// Overworld blocks.
//   BOT_MODULES=<node_modules> node exile_bot.js <port> <mc version>      (RCON on port + 100, password bmh)
// Normal difficulty, default config (5000 to 60000), no Eternal Item: 23333 world blocks, so about 23333 blocks in the
// Overworld and about 2917 in the Nether. Prints PASS/FAIL lines, then "EXILE <passed>/<total>".
const MODULES = process.env.BOT_MODULES
if (!MODULES) { console.log('BOT_MODULES is not set: npm install --ignore-scripts --prefix <dir> mineflayer@4.39.0, then BOT_MODULES=<dir>/node_modules'); process.exit(2) }
const RCON_PY = __dirname + '/rcon.py'
const mineflayer = require(MODULES + '/mineflayer')
const { execFileSync } = require('child_process')

const PORT = parseInt(process.argv[2], 10)
const MCV = process.argv[3]
const NAME = 'exilebot'
const results = []
const sleep = (ms) => new Promise((r) => setTimeout(r, ms))
process.on('uncaughtException', (e) => console.log('uncaught', e && e.message))
process.on('unhandledRejection', (e) => console.log('unhandled', e && e.message))

function rcon (cmd) {
  try {
    return execFileSync('python3', ['-I', RCON_PY, String(PORT + 100), cmd], { encoding: 'utf8', timeout: 30000 }).trim()
  } catch (e) {
    return 'RCON-ERROR ' + e.message
  }
}
function check (name, ok, detail) {
  results.push({ name, ok })
  console.log((ok ? 'PASS ' : 'FAIL ') + name + (detail ? '  [' + String(detail).slice(0, 300) + ']' : ''))
}
function where () {
  const p = rcon('data get entity ' + NAME + ' Pos').match(/\[(-?[\d.]+)d, (-?[\d.]+)d, (-?[\d.]+)d\]/)
  const d = rcon('data get entity ' + NAME + ' Dimension').match(/"([^"]+)"/)
  return p ? { x: parseFloat(p[1]), z: parseFloat(p[3]), dim: d ? d[1] : '?' } : null
}

// Die and wait until the respawn has landed far from (0, 0) or in another dimension; returns the new place.
async function dieAndWait () {
  rcon('kill ' + NAME)
  let w = null
  for (let i = 0; i < 60; i++) {
    await sleep(1000)
    w = where()
    if (w && Math.hypot(w.x, w.z) > 500) break
  }
  return w
}

async function main () {
  const bot = mineflayer.createBot({ host: '127.0.0.1', port: PORT, username: NAME, auth: 'offline', version: MCV, checkTimeoutInterval: 180000 })
  bot.on('error', (e) => console.log('bot error', e.message))
  await new Promise((resolve, reject) => {
    bot.once('spawn', resolve)
    setTimeout(() => reject(new Error('no spawn after 60 s')), 60000)
  })
  await sleep(4000)
  for (const c of ['gamerule randomDeathSpawnPoint true', 'gamerule doImmediateRespawn true', 'difficulty normal',
    'gamemode survival ' + NAME, 'clear ' + NAME]) rcon(c)

  // Overworld: a forced spawn point at the origin
  rcon('spawnpoint ' + NAME + ' 0 100 0')
  const a = await dieAndWait()
  const da = a ? Math.round(Math.hypot(a.x, a.z)) : null
  check('Overworld respawn about 23333 blocks from the bed (got ' + da + ')', a && a.dim === 'minecraft:overworld' && da > 21000 && da < 25700, JSON.stringify(a))

  // Nether: a charged respawn anchor at the origin
  rcon('execute in minecraft:the_nether run setblock 0 99 0 minecraft:obsidian')
  rcon('execute in minecraft:the_nether run setblock 0 100 0 minecraft:respawn_anchor[charges=4]')
  rcon('execute in minecraft:the_nether run spawnpoint ' + NAME + ' 0 100 0')
  const n = await dieAndWait()
  const dn = n ? Math.round(Math.hypot(n.x, n.z)) : null
  check('Nether anchor respawn in the Nether (got ' + (n && n.dim) + ')', n && n.dim === 'minecraft:the_nether', JSON.stringify(n))
  check('Nether exile about 2917 Nether blocks from the anchor, not 23333 (got ' + dn + ')', dn !== null && dn > 2400 && dn < 3500, JSON.stringify(n))

  rcon('difficulty peaceful')
  bot.quit()
  await sleep(1000)
  const passed = results.filter((r) => r.ok).length
  console.log('EXILE ' + passed + '/' + results.length)
  process.exit(passed === results.length ? 0 : 1)
}

main().catch((e) => { console.log('EXILE crashed: ' + e.message); process.exit(2) })
