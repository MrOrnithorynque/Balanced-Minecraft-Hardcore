// Smoke bot: a vanilla mineflayer client plus RCON, run once per dedicated server of every branch and loader (branch.sh),
// after the amulet scenario. Checks that every item and recipe loads, the totem nerf, Echo Sense, Echo Strike, the distant
// respawn and the anvil step:
//   BOT_MODULES=<node_modules> SMOKE_ITEMS="a b" SMOKE_RECIPES="x y" node smoke_bot.js <port> <mc version>
// RCON on <port> + 100, password bmh (rcon.py next to this file). Prints PASS/FAIL lines, then "SMOKE <passed>/<total>".
const MODULES = process.env.BOT_MODULES
if (!MODULES) { console.log('BOT_MODULES is not set: npm install --ignore-scripts --prefix <dir> mineflayer@4.39.0, then BOT_MODULES=<dir>/node_modules'); process.exit(2) }
const RCON_PY = __dirname + '/rcon.py'
const mineflayer = require(MODULES + '/mineflayer')
const { Vec3 } = require(MODULES + '/vec3')
const { execFileSync } = require('child_process')

const PORT = parseInt(process.argv[2], 10)
const MCV = process.argv[3]
const NAME = 'smokebot'
const ITEMS = (process.env.SMOKE_ITEMS || '').split(/\s+/).filter(Boolean)
const RECIPES = (process.env.SMOKE_RECIPES || '').split(/\s+/).filter(Boolean)
const COMPONENTS = /^1\.21/.test(MCV) // 1.20.5+ item NBT
const results = []
const sleep = (ms) => new Promise((r) => setTimeout(r, ms))
// a throw inside mineflayer's own handlers (unknown modded items) must not lose the results gathered so far
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

// Effects on a living entity: "active_effects" with ids since 1.20.2, "ActiveEffects" with numeric Id before.
function effects (target) {
  const a = rcon('data get entity ' + target + ' active_effects')
  if (a.includes('[')) return a
  return rcon('data get entity ' + target + ' ActiveEffects')
}
function hasEffect (text, name, numericId) {
  return text.includes('minecraft:' + name) || new RegExp('Id: ' + numericId + 'b?[,}]').test(text)
}
function num (text) { const m = text.match(/data: (-?[\d.]+)/); return m ? parseFloat(m[1]) : null }
function rconPos (target) {
  const t = rcon('data get entity ' + target + ' Pos')
  const m = t.match(/\[(-?[\d.]+)d, (-?[\d.]+)d, (-?[\d.]+)d\]/)
  return m ? new Vec3(parseFloat(m[1]), parseFloat(m[2]), parseFloat(m[3])) : null
}

async function joinBot () {
  const bot = mineflayer.createBot({ host: '127.0.0.1', port: PORT, username: NAME, auth: 'offline', version: MCV, checkTimeoutInterval: 180000 })
  bot.on('error', (e) => console.log('bot error', e.message))
  bot.on('kicked', (r) => console.log('bot kicked', JSON.stringify(r)))
  await new Promise((resolve, reject) => {
    bot.once('spawn', resolve)
    bot.once('end', (r) => reject(new Error('ended before spawn: ' + r)))
    setTimeout(() => reject(new Error('no spawn after 60 s')), 60000)
  })
  return bot
}

async function main () {
  const bot = await joinBot()
  check('bot joined (every server-side mixin target loaded without a crash)', true)
  await sleep(4000) // a joining player ignores damage for 60 ticks (spawnInvulnerableTime)
  for (const c of ['gamerule doDaylightCycle false', 'gamerule doWeatherCycle false', 'gamerule doImmediateRespawn true',
    'gamerule randomDeathSpawnPoint false', 'time set day', 'weather clear', 'difficulty peaceful',
    'gamemode survival ' + NAME, 'clear ' + NAME, 'effect clear ' + NAME]) rcon(c)

  // 1. Registry and data packs: every item gives, every recipe file parsed into a recipe.
  for (const id of ITEMS) {
    const r = rcon('give ' + NAME + ' bmh:' + id)
    check('item bmh:' + id + ' exists', /^Gave 1/.test(r), r)
  }
  rcon('clear ' + NAME)
  for (const id of RECIPES) {
    const r = rcon('recipe give ' + NAME + ' bmh:' + id)
    check('recipe bmh:' + id + ' loaded', /Unlocked 1 recipe/.test(r), r)
  }

  // 2. Part A: a player's totem keeps Regeneration and Absorption but loses Fire Resistance; a cow's keeps all three.
  rcon('effect clear ' + NAME)
  rcon('item replace entity ' + NAME + ' weapon.offhand with minecraft:totem_of_undying')
  rcon('damage ' + NAME + ' 1000 minecraft:generic')
  await sleep(1500)
  const pe = effects(NAME)
  const health = num(rcon('data get entity ' + NAME + ' Health'))
  check('player totem: popped (gone from the off hand)', !rcon('data get entity ' + NAME + ' Inventory').includes('totem_of_undying'))
  check('player totem: survived', health !== null && health > 0, health)
  check('player totem: Regeneration', hasEffect(pe, 'regeneration', 10), pe)
  check('player totem: Absorption', hasEffect(pe, 'absorption', 22), pe)
  check('player totem: no Fire Resistance', !hasEffect(pe, 'fire_resistance', 12), pe)
  const p0 = bot.entity.position.floored()
  const hand = COMPONENTS ? '{id:"minecraft:totem_of_undying",count:1}' : '{id:"minecraft:totem_of_undying",Count:1b}'
  rcon('summon minecraft:cow ' + (p0.x + 3) + ' ' + p0.y + ' ' + p0.z + ' {NoAI:1b,Tags:["smokecow"],HandItems:[' + hand + ',{}]}')
  await sleep(1000)
  rcon('damage @e[type=minecraft:cow,tag=smokecow,limit=1] 1000 minecraft:generic')
  await sleep(1500)
  const ce = effects('@e[type=minecraft:cow,tag=smokecow,limit=1]')
  check('cow totem: still has Fire Resistance', hasEffect(ce, 'fire_resistance', 12), ce)
  rcon('kill @e[type=minecraft:cow,tag=smokecow]')

  // 3. Part E, Echo Sense: breaking an iron ore with the Eternal Pickaxe lights the iron ores around it.
  rcon('clear ' + NAME)
  rcon('effect clear ' + NAME)
  rcon('item replace entity ' + NAME + ' weapon.mainhand with bmh:eternal_pickaxe')
  const p1 = bot.entity.position.floored()
  rcon('setblock ' + (p1.x + 1) + ' ' + p1.y + ' ' + p1.z + ' minecraft:iron_ore')
  rcon('setblock ' + (p1.x + 3) + ' ' + p1.y + ' ' + (p1.z + 3) + ' minecraft:iron_ore')
  rcon('setblock ' + (p1.x - 3) + ' ' + (p1.y + 1) + ' ' + (p1.z + 2) + ' minecraft:deepslate_iron_ore')
  await sleep(1000)
  const ore = bot.blockAt(new Vec3(p1.x + 1, p1.y, p1.z))
  try {
    await bot.dig(ore, true)
  } catch (e) {
    console.log('dig error', e.message)
  }
  await sleep(1000)
  const lit = rcon('execute if entity @e[type=minecraft:block_display]')
  check('Echo Sense: ores light up (block displays)', /count: [1-9]/.test(lit), lit)
  rcon('kill @e[type=minecraft:block_display]')

  // 4. Part E, Echo Strike: a full hit with the Eternal Sword echoes onto the other husk (NoAI dummies, easy difficulty).
  rcon('clear ' + NAME)
  rcon('difficulty easy')
  const p2 = bot.entity.position.floored()
  rcon('summon minecraft:husk ' + (p2.x + 2) + ' ' + p2.y + ' ' + p2.z + ' {NoAI:1b,PersistenceRequired:1b,Tags:["h1"]}')
  rcon('summon minecraft:husk ' + (p2.x + 2) + ' ' + p2.y + ' ' + (p2.z + 3) + ' {NoAI:1b,PersistenceRequired:1b,Tags:["h2"]}')
  rcon('item replace entity ' + NAME + ' weapon.mainhand with bmh:eternal_sword')
  await sleep(1500)
  const husk = bot.nearestEntity((e) => e.name === 'husk')
  if (husk) {
    await bot.lookAt(husk.position.offset(0, 1.5, 0), true)
    await sleep(200)
    bot.attack(husk)
  }
  await sleep(2000)
  const h1 = num(rcon('data get entity @e[type=minecraft:husk,tag=h1,limit=1] Health'))
  const h2 = num(rcon('data get entity @e[type=minecraft:husk,tag=h2,limit=1] Health'))
  check('Echo Strike: the struck husk was hit', husk !== null && h1 !== null && h1 < 20, 'h1=' + h1)
  check('Echo Strike: the other husk took the echo', h2 !== null && h2 < 20, 'h2=' + h2)
  rcon('kill @e[type=minecraft:husk]')
  rcon('difficulty peaceful')

  // 5. The core loop: a distant respawn keeps the Eternal Sword and drops the rest.
  rcon('clear ' + NAME)
  rcon('give ' + NAME + ' bmh:eternal_sword')
  rcon('give ' + NAME + ' minecraft:dirt 5')
  rcon('gamerule randomDeathSpawnPoint true')
  const before = rconPos(NAME)
  rcon('kill ' + NAME)
  let after = null
  for (let i = 0; i < 40; i++) {
    await sleep(1000)
    after = rconPos(NAME)
    if (after && before && Math.hypot(after.x - before.x, after.z - before.z) > 1000) break
  }
  const dist = after && before ? Math.round(Math.hypot(after.x - before.x, after.z - before.z)) : null
  check('death: respawned far away (> 1000 blocks)', dist !== null && dist > 1000, 'distance ' + dist)
  const inv = rcon('data get entity ' + NAME + ' Inventory')
  check('death: the Eternal Sword stayed', inv.includes('bmh:eternal_sword'), inv)
  check('death: the dirt was dropped', !inv.includes('minecraft:dirt'), inv)
  rcon('gamerule randomDeathSpawnPoint false')

  // 6. Part B: an anvil awakens the Dormant Totem with the Echo Heart for 39 levels. Raw clicks: the bot cannot name
  // modded items, but `give` fills hotbar slots 0 and 1, which are window slots 30 and 31 of an anvil (3 own slots).
  if (ITEMS.includes('dormant_totem') && ITEMS.includes('echo_heart')) {
    rcon('clear ' + NAME)
    rcon('xp set ' + NAME + ' 0 levels')
    rcon('xp add ' + NAME + ' 40 levels')
    rcon('give ' + NAME + ' bmh:dormant_totem')
    rcon('give ' + NAME + ' bmh:echo_heart')
    const pa = bot.entity.position.floored()
    rcon('setblock ' + (pa.x + 1) + ' ' + pa.y + ' ' + pa.z + ' minecraft:anvil')
    await sleep(1500)
    try {
      const win = await bot.openBlock(bot.blockAt(new Vec3(pa.x + 1, pa.y, pa.z)))
      await bot.clickWindow(30, 0, 0)
      await bot.clickWindow(0, 0, 0)
      await bot.clickWindow(31, 0, 0)
      await bot.clickWindow(1, 0, 0)
      await sleep(1000)
      await bot.clickWindow(2, 0, 1)
      await sleep(1000)
      bot.closeWindow(win)
    } catch (e) {
      console.log('anvil error', e.message)
    }
    await sleep(1000)
    const inv2 = rcon('data get entity ' + NAME + ' Inventory')
    const levels = num(rcon('data get entity ' + NAME + ' XpLevel'))
    check('anvil: Dormant Totem + Echo Heart give the Totem of Eternity', inv2.includes('bmh:totem_of_eternity') && !inv2.includes('bmh:dormant_totem'), inv2)
    check('anvil: it cost 39 levels', levels === 1, 'levels left ' + levels)
  }

  bot.quit()
  await sleep(1000)
  const passed = results.filter((r) => r.ok).length
  console.log('SMOKE ' + passed + '/' + results.length)
  process.exit(passed === results.length ? 0 : 1)
}

main().catch((e) => {
  console.log('SMOKE crashed: ' + e.message)
  const passed = results.filter((r) => r.ok).length
  console.log('SMOKE ' + passed + '/' + (results.length + 1))
  process.exit(2)
})
