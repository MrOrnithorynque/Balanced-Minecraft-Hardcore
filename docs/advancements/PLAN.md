# Plan: advancements

Status: planned 2026-11-07, not started. Base: `origin/main` (1.9.0, Minecraft 1.20.4), then every version branch.

## Goal

BMH has no advancement today (checked: no file with "advancement" in its path on `origin/mc-1.21.1`). The chain of the Eternal Items (hollow totem, Echo Heart, awakening, save, moon ritual) is invisible without JEI. A small advancement tab shows the path and rewards the first time you do each thing.

No rewards (no XP, no loot): the mod is about balance.

## Design

Two kinds of advancement, both without a custom trigger. A custom `CriterionTrigger` needs a registry call that differs on 1.20, 1.21 and 26.1. These two kinds do not.

1. Vanilla triggers in the JSON only: `minecraft:tick` for the root, `minecraft:inventory_changed` for "you got the item".
2. `minecraft:impossible` in the JSON, awarded from the server code when the event happens. One helper, `handlers/BmhAdvancements.java`: `award(ServerPlayer player, String path)` looks the advancement up and calls `player.getAdvancements().award(advancement, "done")`. The lookup API differs per version (1.20.x: `Advancement` from `server.getAdvancements().getAdvancement(id)`; 1.21.1 and 26.1: `AdvancementHolder` from `.get(id)`). Wrap this in the helper so no handler sees it. Do nothing if the advancement is missing (a data pack can remove it).

Data folder: `data/bmh/advancements/` on the 1.20.x branches, `data/bmh/advancement/` on `mc-1.21.1` and `mc-26.1` (the folder was renamed in 1.21: a file in the old folder is silently ignored). The icon field is `{"item": ...}` before 1.20.5 and `{"id": ...}` after. The `inventory_changed` item predicate differs too. Copy the exact shapes from the vanilla data of each version (the vanilla jar in the Gradle cache), never from memory.

Registry ids to know (they are not what the names suggest): the Hollow Totem is registered as `bmh:dormant_totem`, the Dormant Totem of Eternal Life as `bmh:sleeping_totem`, the Totem of Eternity as `bmh:totem_of_eternity`, the Echo Heart as `bmh:echo_heart`. Read `init/ItemInit.java`.

## The list

Tab root: `bmh:root`, title "Balanced Hardcore", icon the Navigator's Amulet, `minecraft:tick`, a background texture (a vanilla one such as `minecraft:textures/gui/advancements/backgrounds/stone.png` is fine).

Journey branch
| id | title | frame | how |
|---|---|---|---|
| `journey/exile` | Not the end | task | award in `RespawnEffectHandler.onPlayerRespawn` after `takeExiled` |
| `journey/far_away` | Far, far away | goal | award there when the exile distance is 30000 blocks or more |
| `journey/home_at_last` | Home at last | task | first homecoming (`HomecomingHandler`, see below) |
| `journey/long_way_home` | The long way home | challenge | homecoming after an exile of 30000 blocks or more (`HomecomingHandler`) |

Eternity branch
| id | title | frame | how |
|---|---|---|---|
| `eternity/hollow_totem` | A hollow promise | task | `inventory_changed`, Hollow Totem |
| `eternity/echo_heart` | Heart of an echo | task | `inventory_changed`, Echo Heart |
| `eternity/awakening` | Eternity awakens | goal | `inventory_changed`, Totem of Eternity |
| `eternity/saved` | Not today | task | award in `TotemOfEternityHandler.trySave` when it saves the player |
| `eternity/moon_ritual` | Moon-touched | goal | award in `MoonRitualHandler` when a ritual completes |
| `eternity/eternal_tool` | Heavy with forever | task | `inventory_changed`, any of the Eternal Sword, Axe or Pickaxe |
| `eternity/echo_strike` | An echo answers | task | award in `EchoStrikeHandler` on the first echo |
| `eternity/smash` | From above | task | award in `AxeSmashHandler` on the first boosted hit |
| `eternity/echo_sense` | Ore that glows | task | award in `EchoSenseHandler` on the first glow |
| `eternity/everything_stays` | Everything that stays | challenge | `inventory_changed` needing all five at once: sword, axe, pickaxe, bundle, Totem of Eternity |

Navigation branch
| id | title | frame | how |
|---|---|---|---|
| `navigation/amulet` | North is a promise | task | `inventory_changed`, Navigator's Amulet |
| `navigation/stargazer` | Reading the sky | task | award in `AmuletReading` on the first successful reading |
| `navigation/named_place` | A name for a place | task | award in `AmuletPoints.put` the first time |
| `navigation/own_sky` | A sky of your own | goal | award in `AmuletPoints.put` when the player holds 3 points |

Titles and descriptions are a first draft. The session may trim the list (drop any entry whose award point turns out awkward on a version) and must report what it dropped. Keep descriptions short and in plain words. Chat announcement on for goals and challenges only (`announce_to_chat` false for tasks).

Where several handlers award for a hand-held ability, check the real event fires on the server for all three loaders: the handlers are common code, so one hook is enough, but confirm in the self-test.

## The two homecoming advancements

`journey/home_at_last` and `journey/long_way_home` belong to the other feature (see `docs/homecoming/PLAN.md`). They are awarded by `HomecomingHandler.arrive`, when the homecoming record is deleted: `home_at_last` always, `long_way_home` when the exile distance is `BmhAdvancements.FAR_AWAY_BLOCKS` (30000) or more. The text rule `displayTextOnRespawn` does not change the awards.

## Other files

- Lang keys: `advancements.bmh.<path>.title` and `.description` style, or whatever the vanilla key for advancement text is on each version (use plain `title` and `description` component objects with `translate`). Add them to all ten files in `common/src/main/resources/assets/bmh/lang/`.
- `CHANGELOG.md`, `## [Unreleased]`, `### Added`: one line for the tab, in ASD-STE100.
- `.claude/CLAUDE.md` code map: one bullet for `BmhAdvancements` and the data folder rule.
- A test `selftest` cell (model on `EternalWearTest.java` or `RespawnTest.java`): all advancements load (the server advancement manager knows every id), awarding one through the helper sets it done on the mock player, and a second award does nothing. Also a data check in the build: every JSON file parses, and every title key exists in `en_us.json` (a small script under `.claude/skills/verify-loaders/scripts/` is fine).

## Rollout across branches

1. Build it on `feat/advancements`, cut from `origin/main` (1.20.4).
2. Cherry-pick to `feat/advancements-1.20`, `-1.20.1`, `-1.20.2`, `-1.20.3` (cut from `origin/mc-<version>`). The empty-diff check of CLAUDE.md applies.
3. Port by hand to `feat/advancements-1.21.1` and `feat/advancements-26.1`: folder name, icon field, predicate shape, award API, `Identifier` on 26.1.
4. `compile_check.sh` on every version, a full build per branch, then verify-loaders.

## Done means

- Self-test cell green on every branch and loader that builds.
- A production-style server boot per loader shows no "Couldn't parse advancement" line in the log (`grep -i advancement` on `latest.log`).
- One screenshot of the advancement tab on each of 1.20.4, 1.21.1 and 26.1 (client hook, never committed), so the user can judge the look.
- Nothing pushed, no version bump, no release.
