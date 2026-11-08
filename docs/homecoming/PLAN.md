# Plan: the homecoming

Status: planned 2026-11-07, not started. Base: `origin/main` (1.9.0, Minecraft 1.20.4), then every version branch.

## Goal

A distant respawn has a cinematic for the departure and nothing for the end. The homecoming is that end: when you reach a home after an exile, you get a short moment of reward. The text says how far you went and for how long.

It stays small on purpose. Phase 1 is server code only: vanilla title packets, a sound and particles. No client class, no mixin on the client, no network channel. A custom client visual is out of scope until the user has seen phase 1 and asks for more.

## What counts as a home

The homes are fixed at the moment of the exile, in `RespawnEffectHandler.onPlayerRespawn` (after `takeExiled` is true):

- the center of the exile (`bedLocation` in `RespawnPositionHandler.findRespawnPosition`: the bed or anchor, or the world spawn when there is none in that dimension);
- every named point of the Navigator's Amulet at that moment (`AmuletPoints.get`).

Why fixed: if homes were read at check time, a player could place a bed or name a point under their feet and get the reward at once. The exile center has to leave `RespawnPositionHandler` (today only `exiledPlayers` leaves it, a `Set`): turn it into a `Map<UUID, BlockPos>` or add a second map, taken once with the exile flag.

A home is reached when the player is in the same dimension as the exile and within 24 blocks (horizontal distance) of it. The check runs once per second.

The configured minimum exile distance is 5000 blocks (`BMHConfigLimits.MIN_DISTANCE_FROM_BED`), far above the radius. So a respawn never starts inside a home, and the plan needs no "armed" state.

## Data

One record per player, saved with the player and kept through death and relog. Mirror how `AmuletPoints` does it (`mixin/C_ServerPlayerMixin.java` and the `Holder` interface). On 26.1 the save uses `ValueOutput`/`ValueInput` codecs: write every field, never `optionalFieldOf` with a default (see the 26.1 section of `.claude/CLAUDE.md`).

Fields: dimension id, arrival game time (long), exile distance in blocks (int, as `calculateDistanceFromBed` gave it, Overworld-equivalent), and the list of homes (position and name; the exile center has the name `""`).

Rules for the record:
- A new exile replaces the old record. A player who dies on the way home starts over.
- No record in a hardcore world, in Creative or Spectator, or when `randomDeathSpawnPoint` is off. That is already the case for `takeExiled`.
- While the player is in another dimension, the record stays and the check does nothing.

## The moment

At the home, once:
- Title: "Home" (or the point name when the home is a named point). Subtitle: "{blocks} blocks and {nights} nights since you left". Singular keys for 1 night. Under one night: use a key with no nights. Use vanilla `ClientboundSetTitlesAnimationPacket`, `ClientboundSetTitleTextPacket` and `ClientboundSetSubtitleTextPacket`. Keys start with `bmh.homecoming.`: `JourneyCinematic.onTitle` and `PointSignal.onTitle` ignore other keys, so vanilla shows these. Check this on every branch.
- Nights = whole days between the arrival game time and now (24000 ticks per day).
- The text obeys the game rule `displayTextOnRespawn`. The sound and the particles do not.
- Sound: the BMH bell (`SoundInit.BELL_SOUND`), for the player and the players nearby, like the respawn bell. The session may pick a softer pitch.
- Particles: a slow ring around the player, sent from the server (`ServerLevel.sendParticles`). Pick something warm and small (END_ROD or HAPPY_VILLAGER). A quick look in the client is enough, the user judges the feel.
- Then the record is deleted.

Translations: every file in `common/src/main/resources/assets/bmh/lang/` (ten languages today) needs the new keys. English is the reference. Machine-quality translations are fine for the other nine, but keep the same placeholders.

## Where the code goes

- New `handlers/HomecomingHandler.java`: the record type, `onExile(ServerPlayer, BlockPos center)`, `tick(ServerPlayer)` and the moment.
- `E_ServerPlayerMixin` already has a `ServerPlayer.tick` hook with a clock. Add the one-second check there, or add a tiny new mixin. Do not add a loader tick event.
- `RespawnEffectHandler.onPlayerRespawn` calls `HomecomingHandler.onExile`.
- Save and restore with the player next to `AmuletPoints` data.
- `CHANGELOG.md`, `## [Unreleased]`, one line under `### Added`, for players, in ASD-STE100.
- `.claude/CLAUDE.md` code map: one bullet for the homecoming.

## Rollout across branches

Follow `.claude/CLAUDE.md` ("Every change goes to every branch"):
1. Build it on a branch cut from `origin/main` (1.20.4): `feat/homecoming`.
2. Cherry-pick onto `feat/homecoming-1.20`, `-1.20.1`, `-1.20.2`, `-1.20.3`, each cut from `origin/mc-<version>`. The sources of all 1.20.x are identical, so the empty-diff check in CLAUDE.md applies.
3. Port by hand to `feat/homecoming-1.21.1` (from `origin/mc-1.21.1`) and `feat/homecoming-26.1` (from `origin/mc-26.1`). Expect these differences: player NBT API on 1.21.1, `ValueOutput` codecs and `Identifier` on 26.1, the respawn mixin differs per codebase.
4. `compile_check.sh` on every version, a full build per branch, then verify-loaders.

## Done means

- `selftest.sh` has a new cell (model it on `RespawnTest.java`; mock players with a real connection): record written at exile with the right homes; no trigger before 24 blocks; trigger once at 24 blocks, record deleted; a second death replaces the record; the record survives a save and load, and survives death; a bed placed after the exile is not a home; the record is not read in another dimension.
- A real-death client run with a screenshot of the title (the `death_client.sh` pattern, a temporary hook never committed) on at least one loader per codebase (1.20.4, 1.21.1, 26.1). The user judges the look.
- Green on Forge, Fabric and NeoForge where the branch builds them.
- Nothing pushed, no version bump, no release.
