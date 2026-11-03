# Changelog

All notable changes to Balanced Minecraft Hardcore. One list for every Minecraft version: each release ships the same changes for 1.20 to 1.21.1.

## [Unreleased]

### Added
- Dormant Totem of Eternity and Echo Heart. The Totem of Eternity is now made in steps: the old Totem of Eternity recipe makes a Dormant Totem, the Echo Heart is crafted from 4 crying obsidian, 4 echo shards and a nether star, and the two meet at an anvil (Dormant Totem on the left, Echo Heart on the right, 39 levels). The name field is ignored.
- The Totem of Eternity glows like an enchanted item and says "Eternity" in its tooltip. It is not a real enchantment, so a grindstone cannot remove it.
- Every Eternal Item now says in its tooltip that you wake farther while you carry it. The Eternal Sword, Axe and Pickaxe also say that each death costs them 7% durability.

### Changed
- The Totem of Undying no longer gives Fire Resistance when it saves you. You still get Regeneration and Absorption.
- The exile from a respawn anchor in the Nether is now as long, in world terms, as one from a bed in the Overworld. Before, it was eight times longer, because a Nether block counts for 8 Overworld blocks.
- On 1.21.1, mining a decorated pot with an Eternal Sword, Axe or Pickaxe now breaks it into sherds like with the other tools (it used to drop whole). Silk Touch still keeps it whole.
- The Eternal Sword, Axe and Pickaxe are now made at the smithing table: Totem of Eternity in the template slot, your netherite sword, axe or pickaxe, and an echo shard. The tool keeps its enchantments, name and wear. The dragon egg is no longer needed, and the old crafting table recipes are gone.
- New Eternal Bundle recipe: string, gold ingots, leather, netherite scrap and a Totem of Eternity. It no longer needs a vanilla bundle.
- Eternal tools are repaired with gold blocks at the anvil instead of a Totem of Eternity.
- The Eternal Bundle wears a small gold ring charm on its drawstring, empty or full.

### Fixed
- The Eternal Bundle now shows its filled texture when it holds items.
- On 1.21.1, the Eternal Sword, Axe and Pickaxe no longer refuse enchantments (Mending, Unbreaking and the rest of what their netherite versions take).

## [1.4.0] - 2026-11-02

### Added
- NeoForge version for Minecraft 1.20.4 and 1.21.1. On 1.20.1, NeoForge runs the Forge version.

## [1.3.0] - 2026-11-01

### Added
- Minecraft 1.21.1, on Forge and Fabric (Fabric needs Fabric API).

### Known issues
- Moving a world from 1.20.x to 1.21.1: Eternal Bundles look empty afterwards. Empty them before you move the world.

## [1.2.0] - 2026-11-01

### Added
- Fabric version, for every Minecraft version from 1.20 to 1.20.4. It needs Fabric API. Its settings live in `config/bmh.properties`.

### Fixed
- The mod no longer stops a dedicated server from starting.
- On a LAN world, the Navigator's Amulet no longer shows a guest's coordinates on the host's screen. On-screen texts still only show in singleplayer and for the LAN host.

## [1.1.0] - 2026-11-01

### Added
- "Balanced Hardcore: ON/OFF" button on the Create New World screen (Game tab). Turning it off disables every BMH game rule for that world. It is greyed out in Hardcore mode.

### Fixed
- You no longer respawn in the middle of an ocean or a river.
- Coming back from the End no longer throws you thousands of blocks away from your bed.
- In multiplayer, each player now respawns at their own distance instead of the distance of whoever died last.
- You no longer respawn on the Nether roof, on top of leaves, or in water, lava or magma.
- The respawn distance now grows smoothly with difficulty and Eternal Items, instead of jumping straight to the maximum.
- Breaking your bed now removes your spawn point again, like in vanilla.
- The respawn bell and "new journey" message now play, and only when the distant respawn is on and you are not in Creative or Spectator.
- The Eternal Pouch no longer deletes Eternal Items.
- The Navigator's Amulet can be crafted again (its recipe failed to load).

## [1.0.0] - 2024-03-06

First release.
