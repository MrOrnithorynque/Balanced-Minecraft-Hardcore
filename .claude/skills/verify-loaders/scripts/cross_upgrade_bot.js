// Cross-version upgrade test (cross_upgrade.sh): a world of an older Minecraft version (old, 1.20.5 or later: the gives
// use item components) opened by a newer one (new). Old: a player with a point saved through the real command, worn
// Eternal tools, a filled Eternal Bundle, the Totem of Eternity, and BMH and vanilla game rules changed. New: checks
// what carried over (items, point, every rule value), then a distant respawn with no death wear.
//   BOT_MODULES=<node_modules> node cross_upgrade_bot.js <port> <bot version> old|new   (RCON on port + 100, password bmh)
// Commands use the rule names from before 26.1: rule_names.js turns them into the 26.1 ids when the version is 26.x.
const MODULES = process.env.BOT_MODULES
const RCON_PY = __dirname + '/rcon.py'
const ruleNames = require(__dirname + '/rule_names.js')
const mineflayer = require(MODULES + '/mineflayer')
const { execFileSync } = require('child_process')
const PORT = parseInt(process.argv[2], 10)
const MCV = process.argv[3]
const MODE = process.argv[4]
const NAME = 'upgradebot'
const results = []
const sleep = (ms) => new Promise((r) => setTimeout(r, ms))
process.on('uncaughtException', (e) => console.log('uncaught', e && e.message))
process.on('unhandledRejection', (e) => console.log('unhandled', e && e.message))
function rcon (cmd) {
  try { return execFileSync('python3', ['-I', RCON_PY, String(PORT + 100), ruleNames(MCV, cmd)], { encoding: 'utf8', timeout: 30000 }).trim() } catch (e) { return 'RCON-ERROR ' + e.message }
}
function check (name, ok, detail) { results.push(ok); console.log((ok ? 'PASS ' : 'FAIL ') + name + (detail ? '  [' + String(detail).slice(0, 500) + ']' : '')) }
function pos () { const m = rcon('data get entity ' + NAME + ' Pos').match(/\[(-?[\d.]+)d, (-?[\d.]+)d, (-?[\d.]+)d\]/); return m ? { x: +m[1], y: +m[2], z: +m[3] } : null }

async function main () {
  const bot = mineflayer.createBot({ host: '127.0.0.1', port: PORT, username: NAME, auth: 'offline', version: MCV, checkTimeoutInterval: 180000 })
  bot.on('error', (e) => console.log('bot error', e.message))
  bot.on('kicked', (r) => console.log('bot kicked', JSON.stringify(r).slice(0, 300)))
  await new Promise((resolve, reject) => { bot.once('spawn', resolve); setTimeout(() => reject(new Error('no spawn after 60 s')), 60000) })
  await sleep(4000)
  for (const c of ['gamerule doDaylightCycle false', 'gamerule doWeatherCycle false', 'weather clear', 'difficulty peaceful', 'gamemode survival ' + NAME]) rcon(c)
  if (MODE === 'old') {
    rcon('clear ' + NAME)
    rcon('gamerule keepInventory true')
    rcon('gamerule randomDeathSpawnPoint false')
    rcon('gamerule damageEternalItemWhenDead false')
    rcon('give ' + NAME + ' bmh:navigator_amulet')
    await sleep(500)
    const p = bot.entity.position.floored()
    const lx = p.x + 1; const ly = p.y; const lz = p.z
    rcon('setblock ' + lx + ' ' + ly + ' ' + lz + ' minecraft:lodestone')
    await sleep(1000)
    bot.chat('/bmh point save ' + lx + ' ' + ly + ' ' + lz + ' Camp')
    await sleep(2000)
    console.log('LODESTONE ' + lx + ' ' + ly + ' ' + lz)
    rcon('give ' + NAME + ' bmh:eternal_sword[minecraft:damage=40]')
    rcon('give ' + NAME + ' bmh:eternal_pickaxe[minecraft:damage=10]')
    rcon('give ' + NAME + ' bmh:eternal_bundle[minecraft:bundle_contents=[{id:"minecraft:dirt",count:5}]]')
    rcon('give ' + NAME + ' bmh:totem_of_eternity')
    await sleep(1000)
    const pts = rcon('data get entity ' + NAME + ' bmh_points')
    check('old: point Camp saved', pts.includes('Camp'), pts)
    console.log('OLD INVENTORY ' + rcon('data get entity ' + NAME + ' Inventory'))
    console.log('OLD RULES keepInventory=' + rcon('gamerule keepInventory') + ' | randomDeathSpawnPoint=' + rcon('gamerule randomDeathSpawnPoint'))
  } else {
    const inv = rcon('data get entity ' + NAME + ' Inventory')
    console.log('NEW INVENTORY ' + inv)
    check('new: amulet carried over', inv.includes('bmh:navigator_amulet'), inv)
    check('new: Eternal Sword carried over with damage 40', /id: "bmh:eternal_sword"[^\]]*?"minecraft:damage": 40|"minecraft:damage": 40[^\]]*?id: "bmh:eternal_sword"/.test(inv), inv)
    check('new: Eternal Pickaxe carried over with damage 10', /"minecraft:damage": 10/.test(inv) && inv.includes('bmh:eternal_pickaxe'), inv)
    check('new: Eternal Bundle still holds 5 dirt', inv.includes('bmh:eternal_bundle') && /bundle_contents": \[\{count: 5, id: "minecraft:dirt"\}\]|bundle_contents": \[\{id: "minecraft:dirt", count: 5\}\]/.test(inv), inv)
    check('new: Totem of Eternity carried over', inv.includes('bmh:totem_of_eternity'), inv)
    const pts = rcon('data get entity ' + NAME + ' bmh_points')
    check('new: point Camp still on the player, all three coordinates', pts.includes('Camp') && /x: -?\d+/.test(pts) && /y: -?\d+/.test(pts) && /z: -?\d+/.test(pts), pts)
    const keep = rcon('gamerule keepInventory')
    check('new: vanilla rule keepInventory carried over as keep_inventory=true', /true/.test(keep), keep)
    const bmh = rcon('gamerule randomDeathSpawnPoint')
    check('new: BMH rule random_death_spawn_point kept its old value (false)', /false/.test(bmh), bmh)
    const wear = rcon('gamerule damageEternalItemWhenDead')
    check('new: BMH rule damage_eternal_item_when_dead kept its old value (false)', /false/.test(wear), wear)
    const text = rcon('gamerule displayTextOnRespawn')
    check('new: a BMH rule the old world never changed is still true', /true/.test(text), text)
    rcon('gamerule randomDeathSpawnPoint true')
    // The upgraded world still works: a distant respawn
    rcon('gamerule doImmediateRespawn true')
    rcon('gamerule keepInventory false')
    const before = pos()
    rcon('kill ' + NAME)
    let after = null
    for (let i = 0; i < 40; i++) { await sleep(1000); after = pos(); if (after && before && Math.hypot(after.x - before.x, after.z - before.z) > 1000) break }
    const dist = after && before ? Math.round(Math.hypot(after.x - before.x, after.z - before.z)) : null
    check('new: distant respawn works in the upgraded world (> 1000 blocks)', dist !== null && dist > 1000, 'distance ' + dist)
    const inv2 = rcon('data get entity ' + NAME + ' Inventory')
    check('new: the Eternal Sword stayed after the death', inv2.includes('bmh:eternal_sword'), inv2)
    check('new: no death wear, the rule is still off (damage 40)', /"minecraft:damage": 40[^\]]*?id: "bmh:eternal_sword"/.test(inv2), inv2)
  }
  console.log('UPGRADE ' + MODE + ' ' + results.filter(Boolean).length + '/' + results.length)
  bot.quit(); await sleep(1000); process.exit(0)
}
main().catch((e) => { console.log('UPGRADE crashed: ' + e.message); process.exit(1) })
