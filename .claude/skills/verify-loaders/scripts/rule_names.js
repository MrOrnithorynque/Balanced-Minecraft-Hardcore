// Minecraft 26.1 made the game rules a registry with new ids. The bots keep the old names in their commands, and
// ruleNames turns "gamerule <old name>" into the 26.1 id when the server runs 26.x. Other commands pass unchanged.
const RULES_26 = {
  doDaylightCycle: 'minecraft:advance_time',
  doWeatherCycle: 'minecraft:advance_weather',
  doImmediateRespawn: 'minecraft:immediate_respawn',
  doMobSpawning: 'minecraft:spawn_mobs',
  keepInventory: 'minecraft:keep_inventory',
  randomDeathSpawnPoint: 'bmh:random_death_spawn_point',
  displayTextOnRespawn: 'bmh:display_text_on_respawn',
  foodAndHealthRegenOnWakeUp: 'bmh:food_and_health_regen_on_wake_up',
  onlyCarryOneEternalItemType: 'bmh:only_carry_one_eternal_item_type',
  damageEternalItemWhenDead: 'bmh:damage_eternal_item_when_dead',
  removeCoordinatesInSurvival: 'bmh:remove_coordinates_in_survival'
}

module.exports = function ruleNames (mcv, cmd) {
  if (!/^26\./.test(mcv)) return cmd
  return cmd.replace(/^gamerule (\w+)/, (whole, name) => 'gamerule ' + (RULES_26[name] || name))
}
