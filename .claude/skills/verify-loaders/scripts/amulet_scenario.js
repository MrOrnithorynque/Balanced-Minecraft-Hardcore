// Drives a dedicated server like a (vanilla) client would, to check the Navigator's Amulet's server side (28 checks):
// BOT_MODULES=<node_modules folder that holds mineflayer> node amulet_scenario.js <port> <mc version>
// (the same checks as docs/redesign/c-amulet-tools/amulet_scenario.js). The server needs online-mode=false and RCON on
// <port> + 100 with password bmh. The bot sees nothing of the modded items; it only sends "use item" / "use item on block"
// packets and reads the action bar. Prints PASS/FAIL lines, then "<passed>/<total> checks passed".
const MODULES = process.env.BOT_MODULES || (__dirname + '/node_modules')
const RCON_PY = __dirname + '/rcon.py'
const mineflayer = require(MODULES + '/mineflayer')
const { Vec3 } = require(MODULES + '/vec3')
const { execFileSync } = require('child_process')

const PORT = parseInt(process.argv[2], 10)
const MCV = process.argv[3]
const NAME = 'amuletbot'
const results = []
const sleep = (ms) => new Promise((r) => setTimeout(r, ms))

function rcon (cmd) {
  try {
    return execFileSync('python3', [RCON_PY, String(PORT + 100), cmd], { encoding: 'utf8', timeout: 30000 }).trim()
  } catch (e) {
    return 'RCON-ERROR ' + e.message
  }
}

function check (name, ok, detail) {
  results.push({ name, ok })
  console.log((ok ? 'PASS ' : 'FAIL ') + name + (detail ? '  [' + detail + ']' : ''))
}

// action bar messages seen, newest last: { t, text }
let bars = []
function listen (bot) {
  bot.on('message', (msg, position) => {
    if (position === 'game_info') {
      const raw = JSON.stringify(msg.json !== undefined ? msg.json : msg)
      bars.push({ t: Date.now(), text: raw })
    }
  })
}

// The server answers a click within a few ms: only messages from before the click are ignored (lastUse), not before the wait.
let lastUse = 0
async function waitBar (key, ms) {
  const start = Date.now()
  while (Date.now() - start < ms) {
    const hit = bars.find((b) => b.t >= lastUse - 30 && b.text.includes(key))
    if (hit) return hit
    await sleep(100)
  }
  return null
}

function stack () { return rcon('data get entity ' + NAME + ' SelectedItem') }
function num (re, s) { const m = s.match(re); return m ? parseFloat(m[1]) : null }
function pos (bot) { return bot.entity.position }

async function joinBot () {
  const bot = mineflayer.createBot({ host: '127.0.0.1', port: PORT, username: NAME, auth: 'offline', version: MCV, checkTimeoutInterval: 180000 })
  bot.on('error', (e) => console.log('bot error', e.message))
  bot.on('kicked', (r) => console.log('bot kicked', JSON.stringify(r)))
  listen(bot)
  await new Promise((resolve, reject) => {
    bot.once('spawn', resolve)
    bot.once('end', (r) => reject(new Error('ended before spawn: ' + r)))
    setTimeout(() => reject(new Error('no spawn after 60 s')), 60000)
  })
  return bot
}

async function use (bot) { lastUse = Date.now(); bot.activateItem(); await sleep(50); bot.deactivateItem() }
async function click (bot, block) { lastUse = Date.now(); await bot.activateBlock(block) }

async function main () {
  let bot = await joinBot()
  console.log('joined as', bot.username, 'at', pos(bot))
  await sleep(1500)

  rcon('gamerule doDaylightCycle false')
  rcon('gamerule doWeatherCycle false')
  rcon('gamerule randomDeathSpawnPoint false')
  rcon('gamerule doImmediateRespawn true')
  rcon('weather clear')
  rcon('time set day')
  rcon('gamemode survival ' + NAME)
  rcon('clear ' + NAME)
  console.log(rcon('give ' + NAME + ' bmh:navigator_amulet'))
  await sleep(500)
  const s0 = stack()
  console.log(s0)
  check('give works and the item is in the selected slot', s0.includes('bmh:navigator_amulet'))

  // 1. by day: refused, nothing written
  bars = []
  await use(bot)
  check('day: action bar says the stars are hidden (day)', !!(await waitBar('bmh.amulet.hidden.day', 1500)))
  check('day: nothing spent or written', !stack().includes('bmh_reading') && !stack().includes('bmh_starlight'))

  // 2. a reading at midnight under open sky
  rcon('time set midnight')
  await sleep(600)
  bars = []
  const where = pos(bot).clone()
  await use(bot)
  const reading = await waitBar('bmh.amulet.reading', 2000)
  check('night: a reading shows on the action bar', !!reading, reading && reading.text)
  await sleep(300)
  let s = stack()
  console.log(s)
  check('night: charge spent (2 left)', num(/bmh_starlight: (\d+)/, s) === 2)
  check('night: night index written', num(/bmh_night: (\d+)L/, s) === 1)
  check('night: reading has azimuth and time', /bmh_reading: \{[^}]*azimuth: [-\d.]+f/.test(s) && /time: \d+L/.test(s))

  // 3. cooldown: a second use right away does nothing
  bars = []
  await use(bot)
  await sleep(900)
  check('cooldown: second use within 2 s writes no message', bars.length === 0, JSON.stringify(bars))
  check('cooldown: still 2 charges', num(/bmh_starlight: (\d+)/, stack()) === 2)

  // 4. spend the rest, then the empty message
  await sleep(1500)
  await use(bot); await sleep(3000)
  check('second reading leaves 1', num(/bmh_starlight: (\d+)/, stack()) === 1)
  await use(bot); await sleep(3000)
  check('third reading leaves 0', num(/bmh_starlight: (\d+)/, stack()) === 0)
  bars = []
  await use(bot)
  check('empty: "no starlight left tonight"', !!(await waitBar('bmh.amulet.empty', 1500)))
  check('empty: stays at 0', num(/bmh_starlight: (\d+)/, stack()) === 0)

  // 5. refill at the next nightfall
  rcon('time add 24000')
  await sleep(1800)
  s = stack()
  check('refill: next night gives 3 charges back (inventoryTick)', num(/bmh_starlight: (\d+)/, s) === 3 && num(/bmh_night: (\d+)L/, s) === 2, s)

  // 6. weather and roof
  rcon('weather rain')
  await sleep(2500)
  bars = []
  await use(bot)
  check('rain: stars hidden behind the clouds', !!(await waitBar('bmh.amulet.hidden.rain', 1500)))
  rcon('weather clear')
  await sleep(6000)
  const p = pos(bot)
  rcon('setblock ' + Math.floor(p.x) + ' ' + (Math.floor(p.y) + 2) + ' ' + Math.floor(p.z) + ' minecraft:stone')
  await sleep(1500)
  bars = []
  await use(bot)
  check('roof: stars hidden (open sky needed)', !!(await waitBar('bmh.amulet.hidden.roof', 1500)))
  rcon('setblock ' + Math.floor(p.x) + ' ' + (Math.floor(p.y) + 2) + ' ' + Math.floor(p.z) + ' minecraft:air')
  await sleep(1500)

  // 7. Home on a lodestone: sneak and use
  const hx = Math.floor(p.x) + 1; const hy = Math.floor(p.y); const hz = Math.floor(p.z)
  rcon('setblock ' + hx + ' ' + hy + ' ' + hz + ' minecraft:lodestone')
  await sleep(800)
  bot.setControlState('sneak', true)
  await sleep(500)
  bars = []
  const lode = bot.blockAt(new Vec3(hx, hy, hz))
  check('lodestone is there for the bot', !!lode && lode.name === 'lodestone', lode && lode.name)
  await click(bot, lode)
  check('lodestone: Home set message', !!(await waitBar('bmh.amulet.home_set.lodestone', 2000)))
  const home1 = rcon('data get entity ' + NAME + ' bmh_home')
  console.log(home1)
  check('Home saved on the player (x, y, z of the lodestone)', home1.includes('x: ' + hx) && home1.includes('y: ' + hy) && home1.includes('z: ' + hz), home1)
  bot.setControlState('sneak', false)

  // 8. Home on a bed
  await sleep(1200)
  const bx = Math.floor(p.x) + 2; const bz = Math.floor(p.z) + 2
  rcon('setblock ' + bx + ' ' + hy + ' ' + (bz + 1) + ' minecraft:red_bed[facing=north,part=foot]')
  rcon('setblock ' + bx + ' ' + hy + ' ' + bz + ' minecraft:red_bed[facing=north,part=head]')
  await sleep(800)
  bot.setControlState('sneak', true)
  await sleep(500)
  bars = []
  const bed = bot.blockAt(new Vec3(bx, hy, bz))
  await click(bot, bed)
  check('bed: Home set message', !!(await waitBar('bmh.amulet.home_set.bed', 2000)))
  const home2 = rcon('data get entity ' + NAME + ' bmh_home')
  check('Home moved to the bed', home2.includes('x: ' + bx) && home2.includes('z: ' + bz), home2)
  bot.setControlState('sneak', false)
  await sleep(1200)

  // 9. azimuth, from four sides of Home (creative: free readings)
  rcon('gamemode creative ' + NAME)
  await sleep(500)
  const H = { x: bx + 0.5, z: bz + 0.5 }
  const cases = [['south of Home', 0, 30], ['west of Home', -30, 0], ['north of Home', 0, -30], ['east of Home', 30, 0]]
  for (const [label, dx, dz] of cases) {
    const px = H.x + dx; const pz = H.z + dz
    rcon('tp ' + NAME + ' ' + px + ' ' + (hy + 0) + ' ' + pz)
    await sleep(1200)
    await use(bot)
    await sleep(500)
    const az = num(/azimuth: ([-\d.]+)f/, stack())
    const expected = ((Math.atan2(H.x - px, pz - H.z) * 180 / Math.PI) % 360 + 360) % 360
    check('azimuth ' + label + ': ' + expected.toFixed(1), az !== null && Math.abs(az - expected) < 0.01, 'got ' + az)
    await sleep(2000)
  }
  rcon('gamemode survival ' + NAME)

  // 10. death and respawn keep Home
  rcon('kill ' + NAME)
  await sleep(3000)
  const after = rcon('data get entity ' + NAME + ' bmh_home')
  check('Home survives death (restoreFrom)', after.includes('x: ' + bx) && after.includes('z: ' + bz), after)

  // 11. relog keeps Home
  bot.quit()
  await sleep(2500)
  bot = await joinBot()
  await sleep(1500)
  const relog = rcon('data get entity ' + NAME + ' bmh_home')
  check('Home survives a relog (save and read)', relog.includes('x: ' + bx) && relog.includes('z: ' + bz), relog)

  // 12. the Nether: no stars, and Home refuses
  rcon('gamemode creative ' + NAME)
  rcon('execute in minecraft:the_nether run tp ' + NAME + ' 0 120 0')
  await sleep(4000)
  console.log(stack())
  bars = []
  rcon('clear ' + NAME)
  rcon('give ' + NAME + ' bmh:navigator_amulet')
  await sleep(600)
  await use(bot)
  check('nether: stars only answer in the Overworld', !!(await waitBar('bmh.amulet.hidden.dimension', 2000)))
  bot.quit()
  await sleep(500)
  const bad = rcon('list')
  console.log(bad)
  const failed = results.filter((r) => !r.ok)
  console.log('\n' + (results.length - failed.length) + '/' + results.length + ' checks passed')
  process.exit(failed.length ? 1 : 0)
}

main().catch((e) => { console.log('SCENARIO ERROR', e.stack || e); process.exit(2) })
