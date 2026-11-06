// Upgrade test (upgrade.sh): what a 1.4.0 player's items become under the redesign.
//   BOT_MODULES=<node_modules> node upgrade_bot.js <port> <mc version> old|new     (RCON on port + 100, password bmh)
// old: on a 1.4.0 server, give the items a long-time player has (an enchanted amulet worn by use, an enchanted worn
//      Eternal Sword, the Totem of Eternity, a filled Eternal Bundle), use the amulet three times, dump the inventory.
// new: on the redesign server, same world and same player: dump the inventory, check what carried over, take a reading.
const MODULES = process.env.BOT_MODULES
if (!MODULES) { console.log('BOT_MODULES is not set: npm install --ignore-scripts --prefix <dir> mineflayer@4.39.0, then BOT_MODULES=<dir>/node_modules'); process.exit(2) }
const RCON_PY = __dirname + '/rcon.py'
const ruleNames = require(__dirname + '/rule_names.js')
const mineflayer = require(MODULES + '/mineflayer')
const { execFileSync } = require('child_process')

const PORT = parseInt(process.argv[2], 10)
const MCV = process.argv[3]
const MODE = process.argv[4]
const NAME = 'upgradebot'
const C = /^1\.21/.test(MCV)
const results = []
const sleep = (ms) => new Promise((r) => setTimeout(r, ms))
process.on('uncaughtException', (e) => console.log('uncaught', e && e.message))
process.on('unhandledRejection', (e) => console.log('unhandled', e && e.message))

function rcon (cmd) {
  try {
    return execFileSync('python3', ['-I', RCON_PY, String(PORT + 100), ruleNames(MCV, cmd)], { encoding: 'utf8', timeout: 30000 }).trim()
  } catch (e) {
    return 'RCON-ERROR ' + e.message
  }
}
function check (name, ok, detail) {
  results.push({ name, ok })
  console.log((ok ? 'PASS ' : 'FAIL ') + name + (detail ? '  [' + String(detail).slice(0, 400) + ']' : ''))
}
function topLevel (entry) { // an SNBT compound without its nested compounds and lists
  let s = entry.slice(1, -1); let prev
  do { prev = s; s = s.replace(/\{[^{}]*\}/g, '').replace(/\[[^\[\]]*\]/g, '') } while (s !== prev)
  return s
}
function slot (inv, id) { // the whole inventory entry whose own id is <id> (a bundle's contents do not count)
  const list = inv.slice(inv.indexOf('['))
  let depth = 0; let start = -1
  for (let j = 0; j < list.length; j++) {
    if (list[j] === '{') { if (depth === 0) start = j; depth++ }
    if (list[j] === '}') {
      depth--
      if (depth === 0 && start >= 0) {
        const entry = list.slice(start, j + 1)
        if (topLevel(entry).includes('id: "' + id + '"')) return entry
        start = -1
      }
    }
  }
  return null
}

async function main () {
  const bot = mineflayer.createBot({ host: '127.0.0.1', port: PORT, username: NAME, auth: 'offline', version: MCV, checkTimeoutInterval: 180000 })
  bot.on('error', (e) => console.log('bot error', e.message))
  const bars = []
  bot.on('message', (msg, position) => { if (position === 'game_info') bars.push(JSON.stringify(msg.json !== undefined ? msg.json : msg)) })
  await new Promise((resolve, reject) => { bot.once('spawn', resolve); setTimeout(() => reject(new Error('no spawn after 60 s')), 60000) })
  await sleep(4000)
  for (const c of ['gamerule doDaylightCycle false', 'gamerule doWeatherCycle false', 'weather clear', 'difficulty peaceful',
    'gamemode survival ' + NAME]) rcon(c)

  if (MODE === 'old') {
    rcon('clear ' + NAME)
    rcon('time set midnight')
    if (C) {
      rcon('give ' + NAME + ' bmh:navigator_amulet[minecraft:enchantments={levels:{"minecraft:unbreaking":1}}]')
      rcon('give ' + NAME + ' bmh:eternal_sword[minecraft:enchantments={levels:{"minecraft:sharpness":5}},minecraft:damage=500]')
      rcon('give ' + NAME + ' bmh:totem_of_eternity')
      rcon('give ' + NAME + ' bmh:eternal_bundle[minecraft:bundle_contents=[{id:"minecraft:diamond",count:3}]]')
    } else {
      rcon('give ' + NAME + ' bmh:navigator_amulet{Enchantments:[{id:"minecraft:unbreaking",lvl:1s}]}')
      rcon('give ' + NAME + ' bmh:eternal_sword{Enchantments:[{id:"minecraft:sharpness",lvl:5s}],Damage:500}')
      rcon('give ' + NAME + ' bmh:totem_of_eternity')
      rcon('give ' + NAME + ' bmh:eternal_bundle{Items:[{id:"minecraft:diamond",Count:3b}]}')
    }
    await sleep(1000)
    for (let i = 0; i < 3; i++) { bot.activateItem(); await sleep(60); bot.deactivateItem(); await sleep(1500) }
    const inv = rcon('data get entity ' + NAME + ' Inventory')
    console.log('OLD INVENTORY ' + inv)
    check('old amulet worn by use (damage) or stamped (LastUseTime)', /LastUseTime|[Dd]amage/.test(slot(inv, 'bmh:navigator_amulet') || ''), slot(inv, 'bmh:navigator_amulet'))
  } else {
    const inv = rcon('data get entity ' + NAME + ' Inventory')
    console.log('NEW INVENTORY ' + inv)
    const am = slot(inv, 'bmh:navigator_amulet'); const sw = slot(inv, 'bmh:eternal_sword')
    const bu = slot(inv, 'bmh:eternal_bundle'); const to = slot(inv, 'bmh:totem_of_eternity')
    check('the amulet is still there', !!am, am)
    check('the Eternal Sword kept Sharpness V and its wear', !!sw && /sharpness/.test(sw) && /500/.test(sw), sw)
    check('the Totem of Eternity is still there', !!to, to)
    check('the Eternal Bundle kept its 3 diamonds', !!bu && /diamond/.test(bu), bu)
    check('the old amulet dropped its last-use stamp', !!am && !/LastUseTime/.test(am), am)
    check('the old amulet dropped its wear', !!am && !/[Dd]amage/.test(am), am)
    check('the old amulet dropped Unbreaking', !!am && !/unbreaking/.test(am), am)
    rcon('time set midnight')
    // the old phase gave the amulet first: it sits in hotbar slot 0
    bars.length = 0
    bot.setQuickBarSlot(0)
    await sleep(500)
    bot.activateItem(); await sleep(60); bot.deactivateItem()
    await sleep(1500)
    const after = slot(rcon('data get entity ' + NAME + ' Inventory'), 'bmh:navigator_amulet')
    check('a star reading works with the old amulet (a charge spent)', /bmh_starlight: 2/.test(after || ''), after)
    check('the reading showed coordinates on the action bar', bars.some((b) => /bmh\.amulet\.reading|X .* Y .* Z/.test(b)), bars.slice(-2).join(' | '))
  }
  bot.quit()
  await sleep(1500)
  const passed = results.filter((r) => r.ok).length
  console.log('UPGRADE ' + MODE + ' ' + passed + '/' + results.length)
  process.exit(0)
}
main().catch((e) => { console.log('UPGRADE crashed: ' + e.message); process.exit(2) })
