![Balanced Minecraft Hardcore](https://raw.githubusercontent.com/MrOrnithorynque/Balanced-Minecraft-Hardcore/main/resources/bmh_banner.png)

Hardcore is too punishing: one death and your world is gone. Balanced Minecraft Hardcore (BMH) is the middle ground between normal Minecraft and Hardcore. When you die, you keep your world, but you wake up far away from home.

You can go back to where you were. It takes a long time, and you can die again on the way. Or you can start a new adventure in the same world. The **Eternal Items** you carry stay with you and give you a head start.

**Do not play on hardcore Minecraft: the mod works only on normal Minecraft.**

## Features

### Dying Mechanic

When you die, you respawn far from your bed (or from the world spawn), on a safe spot of solid ground, with only what you could protect. The harder the difficulty, and the more Eternal Items you carry, the farther you wake. You take no damage for 10 seconds after the respawn.

![The journey scene after a distant respawn](https://raw.githubusercontent.com/MrOrnithorynque/Balanced-Minecraft-Hardcore/main/resources/showcase/journey.gif)

![Respawn distance by difficulty and number of Eternal Items](https://raw.githubusercontent.com/MrOrnithorynque/Balanced-Minecraft-Hardcore/main/resources/showcase/respawn_distance.png)

The distances come from the mod config (`minDistanceFromBed` and `maxDistanceFromBed`).

### Finding Your Way

In Survival, the F3 screen hides your coordinates. The Navigator's Amulet is your only way to know where you are and where your places are, after a distant respawn or after a long exploration.

- **Mark your places:** sneak and use the amulet on a bed or a lodestone to name a point there. You keep up to 5 points (mod config `maxAmuletPoints`). The same click on a point renames or removes it.
- **Read the stars:** at night, under open sky and without rain, use the amulet. Your coordinates show on the action bar, and a line of stars points toward each of your points for 30 seconds.
- **Know which line is which:** look at the last star of a line through a spyglass to see the name of its point.
- **Limits:** a reading costs one of 3 starlight charges, which come back at every nightfall. The rest of the time, the needle only points north.

![A star reading: one line of stars toward each named point](https://raw.githubusercontent.com/MrOrnithorynque/Balanced-Minecraft-Hardcore/main/resources/showcase/amulet_points_trace.gif)

![Through a spyglass, the last star of a line shows the name of its point](https://raw.githubusercontent.com/MrOrnithorynque/Balanced-Minecraft-Hardcore/main/resources/showcase/amulet_points_spyglass.gif)

![The naming screen of a point](https://raw.githubusercontent.com/MrOrnithorynque/Balanced-Minecraft-Hardcore/main/resources/showcase/amulet_points_screen.png)

### Eternal Items

Eternal Items stay with you after you die. Each death costs the Eternal tools 7% of their durability, but they never break. They are very very powerful items and you can carry only one of each. Each one you carry also pushes your next respawn farther away.

![The five Eternal Items](https://raw.githubusercontent.com/MrOrnithorynque/Balanced-Minecraft-Hardcore/main/resources/showcase/eternal_items.png)

On the ground, an Eternal Item never despawns, and explosions and cactus do not destroy it. The void still does.

### Totem of Undying

The Totem of Undying still saves you with Regeneration and Absorption, but it no longer gives Fire Resistance.

### Sleeping

Sleeping is now slightly revamped, it can give you buff if you sleep well.

## Items

### Eternal Items

#### Totem of Eternal Life

The heart of every Eternal Item, made in three steps:

1. Craft a **Dormant Totem of Eternal Life** (the old totem recipe).
2. Craft an **Echo Heart** from a nether star, echo shards and crying obsidian.
3. Awaken the totem at an anvil: Dormant Totem on the left, Echo Heart on the right, for 39 levels.

The awakened totem glows like an enchanted item. It is the smithing template of the Eternal tools and the core of the Eternal Bundle.

Carried anywhere in your inventory, the totem saves you from death once, like a Totem of Undying in your hand, and it also gives Fire Resistance. A Totem of Undying in your hand is used first. After it saves you, it turns back into a Dormant Totem: awaken it again with a new Echo Heart. You carry one at most, and a chest, a shulker box or any other container holds one at most.

![The Totem of Eternity saves you from your inventory](https://raw.githubusercontent.com/MrOrnithorynque/Balanced-Minecraft-Hardcore/main/resources/showcase/totem_of_eternity_save.gif)

![dormant_totem_crafting_grid](https://raw.githubusercontent.com/MrOrnithorynque/Balanced-Minecraft-Hardcore/main/resources/dormant_totem_crafting_grid.png)
![echo_heart_crafting_grid](https://raw.githubusercontent.com/MrOrnithorynque/Balanced-Minecraft-Hardcore/main/resources/echo_heart_crafting_grid.png)
![totem_of_eternity_anvil](https://raw.githubusercontent.com/MrOrnithorynque/Balanced-Minecraft-Hardcore/main/resources/totem_of_eternity_anvil.png)

#### Eternal Tools

Upgrade a netherite sword, axe or pickaxe at a smithing table: Totem of Eternal Life in the template slot, then the tool, then an echo shard. The tool keeps its enchantments, its name and its wear. Eternal tools never break. At their last point of durability they are worn out: they deal half damage, mine at the speed of a stone tool and lose their ability. Repair them at an anvil with gold blocks (always 15 levels, and never "Too Expensive") or with Mending. Each one has its own ability:

- **Eternal Sword, Echo Strike:** a full-strength hit echoes onto the nearest other hostile mob within 5 blocks, for half the damage.
- **Eternal Axe, Smash:** a hit while you fall lands harder the longer the fall, up to 3 times the damage, and the blow halves your fall damage.
- **Eternal Pickaxe, Echo Sense:** breaking an ore makes the same ore within 8 blocks glow through walls for 5 seconds.

![eternal_sword_smithing](https://raw.githubusercontent.com/MrOrnithorynque/Balanced-Minecraft-Hardcore/main/resources/eternal_sword_smithing.png)
![eternal_axe_smithing](https://raw.githubusercontent.com/MrOrnithorynque/Balanced-Minecraft-Hardcore/main/resources/eternal_axe_smithing.png)
![eternal_pickaxe_smithing](https://raw.githubusercontent.com/MrOrnithorynque/Balanced-Minecraft-Hardcore/main/resources/eternal_pickaxe_smithing.png)

#### Eternal Bundle

Just like the legacy Bundle but keep with you any item inside it even after you die.

![eternal_bundle_crafting_grid](https://raw.githubusercontent.com/MrOrnithorynque/Balanced-Minecraft-Hardcore/main/resources/eternal_bundle_crafting_grid.png)

### Normal Items

#### Navigator's Amulet

How to use it: see Finding Your Way, under Features.

![navigator_amulet_crafting_grid](https://raw.githubusercontent.com/MrOrnithorynque/Balanced-Minecraft-Hardcore/main/resources/navigator_amulet_crafting_grid.png)

#### Bundle

Legacy minecraft bundle implementation.

![bundle_crafting_grid](https://raw.githubusercontent.com/MrOrnithorynque/Balanced-Minecraft-Hardcore/main/resources/bundle_crafting_grid.png)

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

Available for Forge and Fabric (Fabric needs [Fabric API](https://modrinth.com/mod/fabric-api)), from Minecraft 1.20 to 1.20.4, and 1.21.1. NeoForge: 1.20.1 (the Forge version), 1.20.4 and 1.21.1.
