<img src="https://github.com/MrOrnithorynque/Balanced-Minecraft-Hardcore/blob/main/resources/bmh_banner.png" align="center"> </h1>
<h1 align="center">BALANCED MINECRAFT HARDCORE 1.20.4 <img src="https://github.com/MrOrnithorynque/Balanced-Minecraft-Hardcore/blob/main/resources/totem_of_eternity.gif" align="center" width="30"><br>
	<a href="https://minecraft.curseforge.com/projects/balanced-minecraft-hardcore/files"><img src="https://cf.way2muchnoise.eu/versions/For%20MC_982343_all.svg" alt="Supported Versions"></a>
	<a href="https://github.com/MrOrnithorynque/Balanced-Minecraft-Hardcore/blob/main/LICENSE"><img src="https://img.shields.io/github/license/MrOrnithorynque/Balanced-Minecraft-Hardcore?style=flat&color=900c3f" alt="License"></a>
	<a href="https://discord.gg/Czn9JQxMjT"><img src="https://img.shields.io/badge/Discord-Join-5865f2?logo=discord&logoColor=white&style=flat" alt="Discord"></a>
	<a href="https://www.curseforge.com/minecraft/mc-mods/balanced-minecraft-hardcore"><img src="http://cf.way2muchnoise.eu/full_982343.svg" alt="CF"></a>
  <a href="https://modrinth.com/mod/balanced-minecraft-hardcore"><img src="https://img.shields.io/modrinth/dt/balanced-minecraft-hardcore?logo=modrinth&label=&suffix=%20&style=flat&color=242629&labelColor=5ca424&logoColor=1c1c1c" alt="Modrinth"></a>
    <br><br>
</h1>

Hardcore is too punishing: one death and your world is gone. Balanced Minecraft Hardcore (BMH) is the middle ground between normal Minecraft and Hardcore. When you die, you keep your world, but you wake up far away from home.

You can go back to where you were. It takes a long time, and you can die again on the way. Or you can start a new adventure in the same world. The **Eternal Items** you carry stay with you and give you a head start.

**Do not play on hardcore Minecraft: the mod works only on normal Minecraft.**

[![Discord](https://img.shields.io/badge/Discord-Join-5865f2?logo=discord&logoColor=white&style=flat)](https://discord.gg/Czn9JQxMjT) [![GitHub](https://img.shields.io/badge/GitHub-Source-24292f?logo=github&logoColor=white&style=flat)](https://github.com/MrOrnithorynque/Balanced-Minecraft-Hardcore)

## Features

### Dying Mechanic

When you die, you respawn far from your bed (or from the world spawn), on a safe spot of solid ground, with only what you could protect. The harder the difficulty, and the more Eternal Items you carry, the farther you wake. You take no damage for 10 seconds after the respawn.

<p align="center">
  <img src="resources/showcase/journey.gif" alt="The journey scene after a distant respawn" width="640">
</p>

<p align="center">
  <img src="resources/showcase/respawn_distance.png" alt="Respawn distance by difficulty and number of Eternal Items" width="640">
</p>

The distances come from the mod config (`minDistanceFromBed` and `maxDistanceFromBed`).

### Finding Your Way

In Survival, the F3 screen hides your coordinates. The Navigator's Amulet is your only way to know where you are and where your places are, after a distant respawn or after a long exploration.

- **Mark your places:** sneak and use the amulet on a bed or a lodestone to name a point there. You keep up to 5 points (mod config `maxAmuletPoints`). The same click on a point renames or removes it. Sneak and use the amulet anywhere else to see the list of your points and remove one, even when its bed or lodestone is gone.
- **Read the stars:** at night, under open sky and without rain, use the amulet. Your coordinates show on the action bar, and a line of stars points toward each of your points and, a little lighter, toward the world spawn (the first place of your world) for 30 seconds.
- **Know which line is which:** look at the last star of a line through a spyglass to see the name of its point. The line toward the world spawn has no name.
- **Limits:** a reading costs one of 3 starlight charges, which come back at every nightfall. The rest of the time, the needle only points north.

<p align="center">
  <img src="resources/showcase/amulet_points_trace.gif" alt="A star reading: one line of stars toward each named point" width="640">
</p>

<p align="center">
  <img src="resources/showcase/amulet_points_spyglass.gif" alt="Through a spyglass, the last star of a line shows the name of its point" width="640">
</p>

<p align="center">
  <img src="resources/showcase/amulet_points_screen.png" alt="The naming screen of a point" width="480">
</p>

### Eternal Items

Eternal Items stay with you after you die. Each death costs the Eternal tools 7% of their durability, but they never break. They are very very powerful items and you can carry only one of each. Each one you carry also pushes your next respawn farther away.

<p align="center">
  <img src="resources/showcase/eternal_items.png" alt="The five Eternal Items" width="640">
</p>

On the ground, an Eternal Item never despawns, and explosions and cactus do not destroy it. The void still does.

### Totem of Undying

The Totem of Undying still saves you with Regeneration and Absorption, but it no longer gives Fire Resistance.

### Sleeping

Sleeping is now slightly revamped, it can give you buff if you sleep well.

## Items

### Eternal Items

#### Totem of Eternal Life

The heart of every Eternal Item, made in three steps:

1. Craft a **Hollow Totem of Eternal Life** (the old totem recipe).
2. Craft an **Echo Heart** from a nether star, echo shards and crying obsidian.
3. Awaken the totem at an anvil: Hollow Totem on the left, Echo Heart on the right, for 39 levels.

The awakened totem glows like an enchanted item. It is the smithing template of the Eternal tools and the core of the Eternal Bundle.

Carried anywhere in your inventory, the totem saves you from death once, like a Totem of Undying in your hand, and it also gives Fire Resistance. A Totem of Undying in your hand is used first. You carry one at most, and a chest, a shulker box or any other container holds one at most.

After it saves you, it falls asleep: it becomes a **Dormant Totem of Eternal Life**. The Dormant Totem cannot save you, but it is still an Eternal Item: it stays with you when you die. Only the moon can wake it. At night, under open sky, hold use with the Dormant Totem and look at the moon. For 15 seconds your levels drain, 30 levels in all, while a chime climbs and the totem trembles more and more, and then the totem is awake again. The ritual must be done in one go: if you let go, look away, get hurt, lose the moon or run out of levels, the totem stays asleep and the levels already drained are gone.

<p align="center">
  <img src="resources/showcase/totem_of_eternity_save.gif" alt="The Totem of Eternity saves you from your inventory" width="640">
</p>

<p align="center">
  <img src="resources/showcase/moon_ritual.gif" alt="The moon ritual: the Dormant Totem raised under the moon, runes flow into it, the levels drain, the totem wakes" width="640">
</p>

<p align="center">
  <img src="resources/dormant_totem_crafting_grid.png" alt="dormant_totem_crafting_grid" width="200" style="margin-right: 10px;">
  <img src="resources/echo_heart_crafting_grid.png" alt="echo_heart_crafting_grid" width="200" style="margin-right: 10px;">
  <img src="resources/totem_of_eternity_anvil.png" alt="totem_of_eternity_anvil" width="200">
</p>

#### Eternal Tools

Upgrade a netherite sword, axe or pickaxe at a smithing table: Totem of Eternal Life in the template slot, then the tool, then an echo shard. The tool keeps its enchantments, its name and its wear. Eternal tools never break. At their last point of durability they are worn out: they deal half damage, mine at the speed of a stone tool and lose their ability. Repair them at an anvil with gold blocks (always 15 levels, and never "Too Expensive") or with Mending. Each one has its own ability:

- **Eternal Sword, Echo Strike:** a full-strength hit echoes onto the nearest other hostile mob within 5 blocks, for half the damage.
- **Eternal Axe, Smash:** a hit while you fall lands harder the longer the fall, up to 3 times the damage, and the blow halves your fall damage.
- **Eternal Pickaxe, Echo Sense:** breaking an ore makes the same ore within 8 blocks glow through walls for 5 seconds.

<p align="center">
  <img src="resources/eternal_sword_smithing.png" alt="eternal_sword_smithing" width="200" style="margin-right: 10px;">
  <img src="resources/eternal_axe_smithing.png" alt="eternal_axe_smithing" width="200" style="margin-right: 10px;">
  <img src="resources/eternal_pickaxe_smithing.png" alt="eternal_pickaxe_smithing" width="200">
</p>

#### Eternal Bundle

Just like the legacy Bundle but keep with you any item inside it even after you die.

<p align="center">
  <img src="resources/eternal_bundle_crafting_grid.png" alt="eternal_bundle_crafting_grid" width="200">
</p>

### Normal Items

#### Navigator's Amulet

How to use it: see Finding Your Way, under Features.

<p align="center">
  <img src="resources/navigator_amulet_crafting_grid.png" alt="navigator_amulet_crafting_grid" width="200">
</p>

#### Bundle

Legacy minecraft bundle implementation.

<p align="center">
  <img src="resources/bundle_crafting_grid.png" alt="bundle_crafting_grid" width="200">
</p>


## GameRules

- randomDeathSpawnPoint (default : true)
- displayTextOnRespawn (default : true)
- foodAndHealthRegenOnWakeUp (default : true)
- onlyCarryOneEternalItemType (default : true)
- damageEternalItemWhenDead (default : true)
- removeCoordinatesInSurvival (default : true)

## More Information

Most of the features are not actives when in creative mod.

## Loaders

Available for Forge and Fabric (Fabric needs [Fabric API](https://modrinth.com/mod/fabric-api)), from Minecraft 1.20 to 1.20.4, 1.21.1, and 26.1 to 26.1.2. NeoForge: 1.20.1 (the Forge version), 1.20.4, 1.21.1 and 26.1 to 26.1.2.
