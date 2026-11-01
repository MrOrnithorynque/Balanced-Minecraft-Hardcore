# Changelog

All notable changes to Balanced Minecraft Hardcore. One list for every Minecraft version: each release ships the same changes for 1.20 to 1.20.4.

## [Unreleased]

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
