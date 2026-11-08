# Advancements log

Numbered steps, newest last. A fresh agent can resume from here. The plan is `PLAN.md` (copy of the coordinating session's plan).
Branches (local only, nothing pushed): `feat/advancements` (1.20.4, from `origin/main` 89b3e7e), then `feat/advancements-1.20`, `-1.20.1`, `-1.20.2`, `-1.20.3` (cherry-picks) and hand ports `-1.21.1`, `-26.1`.

1. Read the plan and the code. Award points: `RespawnEffectHandler.onPlayerRespawn` (exile, far away), `TotemOfEternityHandler.trySave`, `MoonRitualHandler.succeed`, `EchoStrikeHandler.deliver`, `AxeSmashHandler.onSmash`, `EchoSenseHandler.onOreBroken`, `AmuletReading.record`, `AmuletPoints.put`.
2. Vanilla shapes read from the jars (`~/.gradle/caches/fabric-loom/<version>/minecraft-extracted_server.jar`): 1.20.4 `advancements/`, icon `item`, `inventory_changed` items `[{"items":[ids]}]`. 1.21.1 `advancement/`, icon `{"count":1,"id"}`, items as a string. 26.1.2 `advancement/`, icon `{"id"}`, background `minecraft:gui/advancements/backgrounds/stone` (no `textures/`, no `.png`). Root trigger `minecraft:tick` has no conditions.
3. JSON and lang keys come from a generator, `advancements-gen_adv.py` in the shared notes folder (`gen_adv.py <tree> <1.20|1.21|26>`). It writes the 19 JSON files and the 38 keys in the ten lang files (idempotent).
4. Finding: `ServerAdvancementManager.getAdvancement` returns `Advancement` on 1.20 and 1.20.1. From 1.20.2 it is `get` returning `AdvancementHolder`. So `BmhAdvancements.java` differs on `mc-1.20` and `mc-1.20.1` (the only 1.20.x file that does).
5. Exile distance: `RespawnPositionHandler` keeps the Overworld-block distance of each exile in `exileDistances`, read once by `takeExileDistance` in `RespawnEffectHandler`.
6. Checks written: `advancement_check.py` (data) and `hook/selftest/AdvancementTest.java` (awards), both under `.claude/skills/verify-loaders/scripts/`.
7. Louis asked for a hidden advancement for the tenth death: `journey/here_we_go_again` (title "Shit, here we go again", frame challenge, `"hidden": true`, parent `journey/exile`). `RespawnEffectHandler.onPlayerRespawn` awards it when the vanilla statistic `minecraft:deaths` is 10 or more, at every respawn except the End exit. The self-test sets the statistic to 9, kills the player and checks the award. Now 20 entries.
8. Louis: the death advancements only count where BMH is on in the world. The tenth-death check moved after the exile check, so it needs a distant respawn (the BMH rule on, survival, not hardcore) like `journey/exile` and `journey/far_away`. The statistic still counts the deaths from before BMH was on. The self-test adds a player in a world with the rule off: no award.

## Integration

The homecoming feature (`feat/homecoming[-<version>]`) and this feature are merged on `feat/integrated[-<version>]`. Nothing is pushed.

9. Merge of `feat/homecoming` into the advancements branch. The conflicts were `RespawnEffectHandler`, `RespawnPositionHandler`, eight lang files and `CHANGELOG.md`. The lang files and the changelog keep both sides. The two Java files keep both sides for the merge commit, so that the next commit can be replayed on every version.
10. Reconciliation commit: one `Exile(center, distance)` record from the homecoming feature serves both. The `exileDistances` map and `takeExileDistance` of this feature are gone. `RespawnEffectHandler` reads the distance from the record. `FAR_AWAY_BLOCKS` (30000) moves to `BmhAdvancements`.
11. Wiring commit: `HomecomingHandler.arrive` awards `journey/home_at_last` always and `journey/long_way_home` when the record distance is `FAR_AWAY_BLOCKS` or more. The awards do not depend on the text rule. `HomecomingTest` checks a near exile (5000 blocks) and a far one (40000 blocks) with the text rule off. `advancement_check.py` no longer has the list of entries that are wired later.

