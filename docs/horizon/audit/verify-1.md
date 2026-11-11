# Verification of audit 1

Paths: W = the BMH worktree, S = the scratchpad, H = S/src/horizon (main at b880384691b88d317553067109d28e785990bda3). Nightly = S/horizon-repo-docs/native-hooks.nightly-2026-11-08.json (source tag nightly-source/2026-11-08-fdc87ae).
Guard check passed: W is the toplevel, the branch is `horizon-26.3`, H is at b880384.
Paths below are relative to H/crates/horizon-client/src/game/ ("G/") unless a prefix says otherwise.

Tier key: N = all in the Nightly catalog. M = in main's catalog only. F = needs a pending fragment (compiled in, no API number). X = needs an injection into a game function. G = not buildable without a change in Horizon.

## Corrected verdict table

| Row | Feature | Audit said | Corrected | Tier | Mechanism | Changed? |
|---|---|---|---|---|---|---|
| 1a | Capture at death | PARTIAL | PARTIAL | F | `loot_drops` (N) or a per-tick cache of the count; difficulty read is pending `level_difficulty`; no hardcore read | no |
| 1b | Distant position inside the respawn | PINNED | PINNED | F | pending `player_respawn_position`, called at G/lan_actions.rs:4083 (main) before `hub.respawn`; in the Nightly source but no API number | no |
| 1c | Background landing search, unloaded chunks | PARTIAL | PARTIAL | F | `force_chunk_in` (M) + `surface_heights` (N); border test needs pending `world_border`; `native_biome` is itself pending | no |
| 1d | Hold the Respawn request | PINNED | PARTIAL | N (client hold) / X+F (exact server hold) | client mod cancels the death screen click with `screen_mouse_button_pressed_pre` and key hook; server tells "ready" by `net_send_to_player` | YES |
| 1e | Exile bookkeeping, End exit excluded | HOOK | HOOK | N | `player_respawned` is not called for the End exit; input data comes from 1b | no |
| 2a | 200 ticks immunity | HOOK | HOOK | N | `living_incoming_damage` cancel + `player_tick` countdown, key by `acting_player().entity_id` | no |
| 2b | Bell and respawn sound for others | PARTIAL | HOOK | N | `run_command("playsound ... @a[name=!X,distance=..16]")` from `server_tick` | YES |
| 2c | Journey signal server to client | HOOK | HOOK | N | `net_send_to_player` / `net_from_server` | no |
| 2d | Journey advancements from code | HOOK | HOOK | N | `run_command("advancement grant X only bmh:journey/exile done")`; `trigger_criterion` Host form is F | no (tier fixed) |
| 3 | Homecoming | HOOK | HOOK | N | attachment `copy_on_death`, `server_tick`, `server_players`, `run_command title`/`playsound`/`advancement grant` | no |
| 4 | Mod game rules | GAP | GAP | G | `RULES` is a `const` table, `rule()` strips only `minecraft:` | no |
| 5a | Config file | HOOK | HOOK | N | `register_configs` Common | no |
| 5b | Config screen with text fields | PARTIAL | PARTIAL | N | mod screen (API 111) + `screen_char_typed_pre`/`screen_key_pressed_pre` drawing its own fields | no |
| 6 | `/bmh point ...` | HOOK | HOOK | N | `register_commands` + `perform_command` | no |
| 7 | Advancement tab | HOOK | HOOK | N | content-mod JSON + `advancement grant` command (keeps `minecraft:impossible`) | no (tier fixed) |
| 8a | SleepHandler | PINNED | PINNED | F | `player_bed_leave` N; health/food/effects need pending `set_player_status`, `add_effect` | no |
| 8b | KeepItemAfterDeath | HOOK | HOOK | N | take stacks out of `loot_drops` (API 405 is in the Nightly), `give_stack` in `player_respawned` | no |
| 8c | EternalItemHandler | PARTIAL | HOOK | N | `player_tick`/`inventory_tick` + `item_picked_up` cancel + `summon` of `minecraft:item` | YES |
| 8d | DisableDebugInfo | PARTIAL | PARTIAL | F | force by `run_command gamerule`: N; restore needs pending `level_game_rule` | no |

Counts over the 19 rows: HOOK 11 (1e, 2a, 2b, 2c, 2d, 3, 5a, 6, 7, 8b, 8c), PARTIAL 5 (1a, 1c, 1d, 5b, 8d), PINNED 2 (1b, 8a), GAP 1 (4).

## V0 check (owner of a single-player world is a guest)

V0 holds. Citations opened:
- G/integrated.rs:823 `// No player of its own: the hub runs without a host.` with `integrated: true`: matches.
- G/integrated.rs:880 `host.hub.lan.connect_local()?`: matches (the line reads `host.hub.lan.connect_local()?` inside the owner setup).
- G/integrated.rs:878 `host.hub.singleplayer_owner = Some(owner);` shows the hub knows the owner as a guest owner, not as a host body.
- `guest_respawn` (G/lan_actions.rs:4019) takes `host.hub.guests` by `conn` with no owner test, so the owner respawns through it. `lan_actions.rs:3089` `S::ClientCommand(c) if c.action == 0 => self.guest_respawn(host, conn),` has no host-player arm.
- G/native_hooks.rs doc of `native_player_load_from_file` says "for the owner of an integrated server as for anyone else". Doc of `native_player_save_to_file` says "for an integrated server's owner when the world saves them".
Effect on audit 1: the audit already treats the owner as a guest, so no verdict of this report flips because of V0. One citation of the audit that claims it does not say so (see "Citations that did not match").
New parity risk from V0 that audit 1 did not list: in the window's world the guest "has no level of its own". `lan_actions.rs:4056-4064` sets `in_frame_bed = own.filter(|c| !server && c.dim == super::Dim::Overworld)` and `elsewhere = own.filter(|_| server)`. A Nether respawn anchor is ignored for a single-player or LAN owner. The Java mod centres an exile on a Nether anchor and divides by 8. On Horizon that case does not exist off a dedicated server.

## Row notes

### 1b: PINNED stands, tier F
- Fragment: `tools/native-hook-fragments/player_respawn_position.json`: `"name": "player_respawn_position"`, `"side": "server"`, `"neoforge": "PlayerRespawnPositionEvent"`. Matches.
- Doc, G/native_hooks.rs: "A dead server player is about to respawn ... once the Respawn click came and before they are placed". It also says "No player's state is swapped in, and the hub is out of `self.lan`". Matches the audit.
- Call site: G/lan_actions.rs `self.native_player_respawn_position(uuid, &name, &mut dimension, &mut at, yaw, pitch);` comes right after the comment `// PlayerRespawnPositionEvent: a point the mods choose is taken as it is.`. It is before `host.hub.respawn(i, spawn, rotation, &ctx);` which is the first send. Order verified. (The audit's :4083 is right for main. In the Nightly source the call is at lan_actions.rs:3960.)
- Catalog: `grep -c respawn_position` gives 0 in native-hooks.json and in the Nightly JSON. The Nightly SOURCE already holds the function (`git show nightly-source/2026-11-08-fdc87ae:...lan_actions.rs` line 3960) and the fragment. So the Nightly build compiles it, but it has no API number. A mod can use it only through a raw `target` injection, which "pins `horizon_revision`" (docs/native-mods.md, "A raw `target` like `tick_mobs` is not a hook point and may change in any commit: such a mod pins `horizon_revision`").
- Searched for a stamped alternative: `player_clone` (N) is called at lan_actions.rs:4135 before the placement, but its doc says "Not cancellable", no position argument, no player state swapped in. `player_spawn_point_set` (N, cancellable) changes the bed point, not the respawn target. `living_death` (N, cancellable) cancels the death, it does not move the respawn. No stamped hook can set the position before `hub.respawn`. PINNED stands.
- Parity risks added: Nether anchor on single-player/LAN (see V0 section); the hook hands over the old bed position when the bed is gone (audit already says it).

### 1a: PARTIAL stands, tier F
- `loot_drops` for a dying player: `G/item_entities.rs` `drop_inventory` builds `NativeLootDrops { ... entity: Some((who.1 as u32, "minecraft:player")), player: Some(who.0) ... }` and calls `self.native_loot_drops(&context, &mut stacks)`. Matches. It returns early when `keep_inventory` is true (`if self.ui.rules.bool("keep_inventory") { return; }`). Matches.
- docs/native-mods.md line 1813 (API 405): "adds a dying player's inventory to `native_loot_drops` ... A mod that takes every stack out of `drops` keeps the inventory." API 405 is in the Nightly docs (`git show` of docs/native-mods.md at the tag finds line 1776 "API 405 adds a dying player's inventory"). Tier N for that part.
- Refuted risk: "keep_inventory true: no Eternal count at death". A per-tick cache fixes it. `player_tick` (N) runs for every living player with the state swapped in, and `native_player_stacks` (G/native_menus.rs:229-243: `(!g.dead && self.lan_acting != Some(g.conn))`) reads an alive player's 36 slots. A mod can count Eternal slots every second and use the last value at death. The Java mod recomputes from the current inventory too when no stored value exists (audit's own spec, RespawnPositionHandler "else recompute from the current inventory").
- Difficulty: `difficulty_changed` (N) hears changes only (G/native_hooks.rs doc: "told only when the difficulty is another one"). The read is `native_level_difficulty` in the pending fragment `level_world.json`. No stamped read exists. Tier F.
- Hardcore: `grep -i hardcore` in G/native_*.rs and mod_api.rs finds only the game-mode comment (native_hooks.rs:4488-4493) and test fixtures (`hardcore: false` in native_entity_refs.rs). No read helper. UNVERIFIED: a mod could read the flag from the world files with `world_dir` (N) but I did not find the file layout. Partial inference: `player_game_mode_changed` (N, cancellable) fires for "a hardcore world's respawn into spectator", before `player_respawned` (lan_actions.rs hardcore branch is before the `as_guest(... native_player_respawned ...)` call). A mod learns hardcore at the first death only, after the departure signal went out.

### 1c: PARTIAL stands, tier F
- `force_chunk_in`: G/native_chunk_tickets.rs `pub(super) fn native_force_chunk_in(owner, dimension, chunk, hold)`, "API 413". It only inserts the ticket and ranks it (`streaming.rank_forced`), so it does not block. In main's catalog, not in the Nightly (S: helpers 177 vs 174, the three extra are `force_chunk`, `force_chunk_in`, `forced_chunks`). Tier M. Matches the audit.
- Alternative refuted: `/forceload add` through `run_command` (N). `cmd_forceload` (G/commands/terrain_cmds.rs:168-200) ends with `self.wait_for_forced_chunks(...)`, which loops up to `FORCED_CHUNK_WAIT = 120 s` on the server thread (terrain_cmds.rs:13, :102-115). That is the freeze the Java search avoids. Not a substitute.
- Border: `force_chunk_in` refuses only past 30,000,000 blocks (`in_world = chunk.iter().all(|c| (-1_875_000..1_875_000).contains(c))`, native_chunk_tickets.rs), not past the world border. The border read `world_border` is pending (fragment `world_border.json`). So the border test is F. Without it the row is M.
- `native_biome` is a pending helper too: `tools/native-hook-fragments/session_state.json` lists helpers `biome`, `data_dir` and the docs placeholder `@API:session_state@` sits on its doc (G/native_hooks.rs:~11397). The audit presents it as a plain helper. Its doc says "None where that chunk isn't loaded", so the GAP for the cheap ocean/river pre-filter stands.
- A replacement for the biome pre-filter exists once the chunk is loaded: `surface_heights` with `OceanFloor` and `WorldSurface` (both in `NativeHeightmap`, G/native_world_edit.rs:12-17) gives the water depth of a column. It needs the chunk loaded, so the cost parity risk stands.
- `surface_heights` limits verified: G/native_world_columns.rs `if !self.native_level_open(dimension) { return Err(format!("{dimension} is not the level being ticked")); }`, `Option<i32>` per column, "none for a column not loaded", API 127, in the Nightly.

### 1d: PINNED becomes PARTIAL, tier N (client hold)
Refutation attempt succeeded in part. The audit says no stamped hook can hold the click. The click starts on the client.
- Hook: `screen_mouse_button_pressed_pre` is in the Nightly, side `c`, cancellable (hooks-compact.txt: `screen_mouse_button_pressed_pre [c! v1] ScreenEvent.MouseButtonPressed.Pre`). Doc, G/native_hooks.rs:3753-3766: "Set `canceled` and the press goes no further: no slot, no screen button ..." and "while a screen of the game's or the title's is open".
- The death screen is such a screen. `screen_hooks_open` (native_hooks.rs, before `screen_view_now`) is true when `self.screen != Screen::Playing` and no book, sign or prompt is open. The death screen is `Screen::Dead` (G/remote.rs:4627-4630 `self.screen = Screen::Dead;`). docs/native-mods.md:4152 lists `Death (the death screen)` as a screen kind. The hook is called first in the window event handler (G/../game.rs:10312-10327 per `self.native_screen_mouse_button_pressed_pre(&screen, button, modifiers, &mut canceled)`; `if canceled { return Ok(()); }`).
- Keyboard: `screen_key_pressed_pre` is also N and cancellable (`[c! v1]`).
- Server to client "ready" message: `net_send_to_player` and `net_from_server` (N). Optionally the client replays the request: `send_packet` (N, API 409, G/native_net.rs) sends any serverbound play packet.
- Bypasses that remain: the gamepad and the keyboard-focus click ("A gamepad click or a click the keyboard's focus sends is not the mouse and fires nothing", native_hooks.rs:3764-3765), and the `immediate_respawn` path that calls `respawn_player()` straight from the death packet when `!show_death_screen` (remote.rs:4634). A client without the mod also bypasses it. So the server must still cope with an unfinished search inside the 1b hook. That is the reason this row is PARTIAL, not HOOK.
- Exact server-side hold stays X+F: a cancellable HEAD injection on `Game::guest_respawn` (G/lan_actions.rs:4019) plus pending `respawn_player` (fragment `respawn_player.json`, G/native_travel.rs `native_respawn_player`, which pushes `NativeMove::Respawn`, handled at native_travel.rs:438-451 by calling `guest_respawn`). Docs for the injection: S/src/horizon-mixins/docs/advanced-injections.md "## Cancellable HEAD" (line 109). That file is not in H but in S/src/horizon-mixins, so the audit's citation is real.

### 1e: HOOK stands, tier N
`G/native_hooks.rs:8660` `/// death that ends in a respawn, on a server; not for the End-exit one.` Matches. The data it consumes (center, distance) comes from 1b, so 1e is useful only with 1b.
Question 4 of the audit (copy_on_death on End exit) answered: `copy_on_death()` is called only at lan_actions.rs:4032 (death path). The End exit (server_travel.rs:376 `self.native_player_clone(uuid, &name, false)`) does no copy, so the attachments stay as they are. The homecoming record survives the End exit.

### 2a: HOOK stands, tier N
- G/native_hooks.rs doc of `native_living_incoming_damage`: "Change `amount`, or set `canceled`, and nothing of `hurtServer` itself happens ... No `self`". Matches.
- It covers players: G/native_hooks.rs `hurt_server` is the single caller for the damage code and `Hurting::Player(i32, HurtSource)` exists (S/src/horizon/crates/horizon-client/src/health.rs:45-47).
- The entity id of the respawned player: `native_acting_player()` (G/native_server.rs:873-894) returns `entity_id: self.native_acting_entity() as i32` inside a player hook, so `player_respawned` can read it.
- Not verified: whether void and `/kill` damage reach this hook (`damage_type` names). UNVERIFIED, same as the audit's open question 6.
- The audit's parity risk "Horizon's own respawn grace may stack": I found no such code. `grep spawn_invulnerable|spawnInvulnerable` in H/crates/horizon-client/src gives only mob `invulnerable_time` fields (mobs.rs:1541-1543). CLAUDE.md of BMH says 26.1 removed `spawnInvulnerableTime`. Risk not found.

### 2b: PARTIAL becomes HOOK, tier N
- `native_run_command` (G/native_server.rs:944-952): "Runs `line` ... as the server's console does ... Only where no player's state is swapped in (`native_tick` ...): false inside a player's hook". API 21, in the Nightly. Matches.
- The audit left selector support UNVERIFIED. Checked: S/src/horizon/crates/horizon-command/src/selector.rs:12-13 lists the options `"name"` and `"distance"`, and :405 parses `"name"` (negation is the usual vanilla form). `cmd_playsound` (G/commands/world_cmds.rs:614-700) takes `targets` through `self.entities(ctx, "targets", true)` and sends each guest a `sound_packet` (`Target::Guest(conn)`), which is the owner's case under V0. So `@a[name=!Alex,distance=..16]` excludes the respawned player.
- Queue the call in `player_respawned` and run it from `server_tick`: the sound plays one tick later. Custom sounds still need `assets/bmh/sounds.json` in the content part.
- Parity: `range = if volume > 1.0 { 16.0 * volume } else { 16.0 }` in cmd_playsound; the Java call is `level.playSeededSound`. Close, not bit-equal.

### 2c: HOOK stands, tier N
G/native_net.rs doc: "Sends `data` on `channel` (`mod_id:name`, at most 30 000 bytes, at most 32 a player a tick) to `player`'s client, at the tick's end, from any server hook. ... API 40." Matches. `remote_native_net` routes the payload to `native_net_from_server` in the window, which also serves the single-player owner (V0: the window joins by the memory pipe).

### 2d and 7: HOOK stands, tier N (changed from "stamped API 69")
Refuted claim: the audit rates `native_trigger_criterion` as stamped HOOK. The Game method is API 69 (G/native_hooks.rs:765-766 and native_criteria.rs doc "API 69"). But a mod reaches it through the `Host` bridge `trigger_criterion`, and that bridge is in a pending fragment: `tools/native-hook-fragments/host_bridge.json:16` `{"name": "trigger_criterion", "target": "horizon_client::game::native_criteria::Game<'_>::native_trigger_mod_criterion"}`. It is in neither catalog. So the typed route is tier F.
Stamped route that needs no trigger: `cmd_advancement` (G/commands.rs:3480). With `grant ... only <advancement> <criterion>` it calls `g.grant_advancement(i, Some(criterion))` for each target's own turn. `minecraft:impossible` is a known trigger (S/H/crates/horizon-client/src/advancements.rs:1973 test JSON `"trigger": "minecraft:impossible"`), `minecraft:tick` and `minecraft:inventory_changed` too (advancements.rs:389, :2892). So the BMH JSON can stay unchanged and the awards run as `run_command("advancement grant <Name> only bmh:journey/exile done")` from `server_tick`. `run_command` is silent (`silent: true`), so the console prints nothing. The toast comes from `grant_advancement` (UNVERIFIED that it shows the toast for the command path; the same function awards for the game's own triggers).
Folder name `data/bmh/advancement/` (singular) and content-mod advancements: docs/mods.md:31 lists "advancements" in data a content mod ships. Matches.

### 3: HOOK stands, tier N
- Attachment survives death: `lan_actions.rs:4031-4034` `attachments.copy_on_death()` and the test `player_attachments_survive_playerdata_and_respawn` (lan/playerdata.rs:917-944, verified). `docs/native-mods.md:5449` "Player UUIDs cover online guests and ...". Matches.
- Title: the stamped route is `run_command("title <Name> times 10 70 20")`, `subtitle`, `title` with translatable JSON (`cmd_title` exists, commands.rs:1240 region). The pending `title` helper is not needed.
- The 24-block radius check: `server_players()` gives `uuid`, `dimension`, `position`, `game_mode`, `health` in the base set (mod_api.rs:662-690). Fields after `latency` (`xp_level`, abilities and so on) are `@API:server_player_state@`, pending, but not used here.
- The bell to everybody (including the player) is `run_command("playsound ... @a")`.

### 4: GAP stands, tier G
- `crates/horizon-client-core/src/gamerules.rs:50` `pub const RULES: &[Rule] = &[` and `Rule { pub name: &'static str, ... }`. `rule(name)` strips only `minecraft:` (gamerules.rs:138-141 `name.strip_prefix("minecraft:")`) and searches `available_rules(true)`. No registration path. Searched native_hooks.rs, native-mods.md, mods.md, native-hooks.json and all fragments for `game_rule`/`gamerule`: only `world_game_rule_change` (N, hears the screen), and the pending `level_game_rule`/`set_game_rule` reads.
- Replacement: `Server` config (`serverconfig/` per world) or a Level attachment plus a `/bmh rule` command. All N. It loses `/gamerule bmh:...`, completion and the screen. A mixin on `rule()`/`available_rules()` (a different crate, `horizon-client-core`, which "ordinary mixin helpers" may target) is a speculative X: `available_rules` returns an opaque `impl Iterator`, so I did not find a clean RETURN injection. UNVERIFIED, not rated.

### 5a and 5b
- 5a: G/native_configs.rs `NativeConfig::new` gives `format!("{mod_id}-{}.toml", kind.suffix())` and `Common` suffix is `common`: `config/bmh-common.toml`. Matches. HOOK, N (`register_configs`, `config_loading`, `config_reloading` are in the Nightly).
- 5b: the audit left the text field UNVERIFIED. `NativeTextField` (G/native_text_field.rs) is the argument of the typing hooks over an existing box, not a widget a mod draws. A mod screen is `native_mod_screen_open` (API 111, G/native_hooks.rs:11290-11302): "the mod draws it in `native_screen_render_post` and takes its clicks, scroll, keys and typing there". So the mod draws its own boxes and reads `screen_char_typed_pre`/`screen_key_pressed_pre` (both N). Feasible, looks different, no vanilla `EditBox`. `screen_key_pressed_pre` is not yet in the typed trait ("will join a later slice", native-mods.md ~5518-5521), so use the injection form. PARTIAL stands.

### 6: HOOK stands, tier N
G/native_hooks.rs:4347-4372 matches the audit (`native_register_commands`, `native_perform_command`, "The player's state is swapped in for the call"). mod_api.rs:743-760 kinds include `integer`, `block_pos`, `text`; `NativeCommand::new` is for every player. `block_pos` with `~` is UNVERIFIED (the audit says so). The client sends the command with `send_chat` (N).

### 8a: PINNED stands, tier F
Fragments: `player_status.json` (`player_food`, `set_player_status`), `player_effects.json` (`add_effect` with `NativeEffectV2.show_icon`). Both pending. G/native_player_state.rs doc of `native_set_player_status`: "False, and nothing set, for a player not reached, a dead one". A stamped fallback gives only approximate numbers: `run_command("effect give <Name> minecraft:instant_health ...")` heals 4 or 8, not 6 or 3; no stamped way lowers food. So the fallback is not parity.

### 8b: HOOK stands, tier N
`ItemStack` is `horizon_items::ItemStack { item, count, damage: u32, extra }` (crates/horizon-items/src/items.rs:340-346), so the 7% wear is a write to `damage` inside `drops` with no component helper. `native_give_stack` doc (G/native_server.rs:1319-1328): "Gives `stack` whole (its damage, name, enchantments... kept) ... API 98." In `player_respawned` the player is alive and swapped in, so the first branch (`uuid == self.native_acting_id()`) runs `self.give(stack)`. `keep_inventory` true skips `drop_inventory` (nothing taken), as the audit says.

### 8c: PARTIAL becomes HOOK, tier N
The audit found no stamped helper that spawns an item. `summon` is in the Nightly. `native_summon(dimension, kind, pos, nbt)` (G/native_entities.rs:~215) calls `spawn_entity_from_nbt`. `ItemEntity::load_nbt` accepts `"minecraft:item"` (G/entity.rs:1236 `if !(kind == "minecraft:item") { return Err(tag); }`), and `/summon item` is tested (G/chat.rs:2319 `assert!(summonable.contains(&"minecraft:item".to_string()))`). So a mod drops the extra copy with NBT `{Item:{...},PickupDelay:40}`. Confidence medium: I did not run it, and the NBT shape for custom components is UNVERIFIED. Scan and shrink: `player_tick`/`inventory_tick` (N). Pickup block: `item_picked_up` (N, cancellable).

### 8d: PARTIAL stands, tier F
Forcing the rule works every tick: G/lan.rs:2796-2797 `host.hub.set_reduced_debug_info(self.ui.rules.bool("reduced_debug_info"));` and lan/hub.rs:871-880 sends entity event 22/23 to every placed guest when it changes. The old value needs pending `level_game_rule`. `world_game_rule_change` hears only the game rule screen, so a command change by an operator is not heard (doc: "after an operator changes a world game rule through the server's game rule screen"). PARTIAL stands.

## Claims refuted

1. Audit 2b "selector support (`@a[name=!X]`, `distance=`) UNVERIFIED": S/src/horizon/crates/horizon-command/src/selector.rs:12-13 lists `name` and `distance`, and `cmd_playsound` takes a `targets` selector (G/commands/world_cmds.rs:623-627), so the exclusion works and 2b is N.
2. Audit 1d "Horizon has no hook before `guest_respawn` that can refuse the request": the client hook `screen_mouse_button_pressed_pre` (N, `[c!]`, G/native_hooks.rs:3753-3766) cancels the death screen click, so the hold needs no injection (with the bypasses listed in row 1d).
3. Audit 2d/7 "HOOK on the strength of the API number" for `trigger_criterion`: the typed Host bridge is the pending fragment `tools/native-hook-fragments/host_bridge.json:16`, so it is tier F; the tier N route is `cmd_advancement` (G/commands.rs:3480) through `run_command`.
4. Audit 8c "I found no stamped helper that spawns an item entity": `summon` (N) with `minecraft:item` NBT works (G/entity.rs:1236, G/native_entities.rs `native_summon`).
5. Audit parity risk "`keep_inventory` true: no Eternal count at death": a `player_tick` cache plus `native_player_stacks` (G/native_menus.rs:229) gives the count without `loot_drops`.
6. Audit 2a parity risk "Horizon's own respawn grace may stack": no respawn immunity code found (`spawn_invulnerable` has no hit in H/crates/horizon-client/src; BMH 26.1 removed it from vanilla too).
7. Audit open question 4 (End exit and `copy_on_death`): `copy_on_death()` is called only in `guest_respawn` (lan_actions.rs:4032); the End exit keeps the attachments untouched (server_travel.rs:376 only calls `native_player_clone(.., false)`).
8. Audit vocabulary paragraph "229 fragments are unstamped": `ls tools/native-hook-fragments/*.json | wc -l` gives 228 (the audit says 228 elsewhere).
9. Audit 1c lists `native_biome(pos)` as a helper without saying it is pending: it sits in `session_state.json`, so it is tier F.

## Citations that did not match

- "(native_hooks.rs:7432-7434 and docs line 1647) say the owner is told like anyone else" (audit section 1, "Where the respawn is decided"). G/native_hooks.rs:7432-7434 is the middle of the doc of `native_player_respawn_position` ("`dimension` ... and `pos` are their respawn point ...") and does not mention the owner. docs/native-mods.md:1647 is the API 323 entry about `native_player_load_from_file` ("the owner of an integrated server too"), not about respawn. The claim itself is true by V0, but these two lines do not carry it.
- mod_api.rs:677 `0 survival, 1 creative, 2 adventure, 3 spectator.` is at line 676 (one line off).
- native_hooks.rs:11398-11402 for `native_biome` is at 11395-11396 plus the signature after (a few lines off). Content matches.
- The audit says `native_trigger_criterion` is "stamped API 69 ... a mixin helper can call it" (open question 9). The API 69 comment is real (native_hooks.rs:765-766), but the claim that a mod can call it today is not backed by a citation: the Host form is pending.
- Not opened (time): audit citations to S/horizon-docs/mod-code.txt and make-a-mod.txt beyond one spot check (mod-code.txt:11 and :69 exist with the quoted topics); hub.rs:5025-5100 spot checked (`respawn` at hub.rs:5022-5030 matches the quoted doc).

## Hooks and helpers the rows need (name, kind, tier, catalog)

| Name | Kind | Tier | Catalog |
|---|---|---|---|
| living_death | hook | N | main + Nightly |
| loot_drops (player death, API 405) | hook | N | main + Nightly (API 405 is in the Nightly docs) |
| player_died | hook | N | main + Nightly |
| player_tick | hook | N | main + Nightly |
| player_respawned | hook | N | main + Nightly |
| player_clone | hook | N | main + Nightly |
| server_tick | hook | N | main + Nightly |
| living_incoming_damage | hook | N | main + Nightly |
| player_game_mode_changed | hook | N | main + Nightly |
| difficulty_changed | hook | N | main + Nightly |
| world_game_rule_change | hook | N | main + Nightly |
| net_from_server, net_from_client | hooks | N | main + Nightly |
| screen_mouse_button_pressed_pre | hook (client) | N | main + Nightly |
| screen_key_pressed_pre, screen_char_typed_pre, screen_render_post | hooks (client) | N | main + Nightly (first one not in typed trait yet) |
| register_configs, config_loading, config_reloading | hooks | N | main + Nightly |
| register_commands, perform_command | hooks | N | main + Nightly |
| player_bed_leave | hook | N | main + Nightly |
| inventory_tick, item_picked_up | hooks | N | main + Nightly |
| player_advancement_done | hook | N | main + Nightly |
| run_command | helper | N | main + Nightly (API 21) |
| net_send_to_player, send_packet, send_chat | helpers | N | main + Nightly |
| attachments | helper | N | main + Nightly |
| server_players, acting_player, held, block, give_stack, summon, action_bar | helpers | N | main + Nightly |
| surface_heights, read_columns, block_collision, level_height, level_game_time | helpers | N | main + Nightly |
| world_dir, config_dir, mod_screen_open, mod_screen, mod_screen_close, fill, text, text_width | helpers | N | main + Nightly |
| force_chunk_in, force_chunk, forced_chunks | helpers | M | main only (API 412, 413) |
| player_respawn_position | hook | F | fragment player_respawn_position.json (in the Nightly source, no number) |
| respawn_player | helper | F | fragment respawn_player.json |
| trigger_criterion (Host form) | helper | F | fragment host_bridge.json |
| set_player_status, player_food | helpers | F | fragment player_status.json |
| add_effect, effects, set_effects, effects_v2 | helpers | F | fragment player_effects.json |
| level_difficulty, level_game_rule, set_game_rule | helpers | F | fragment level_world.json |
| world_border | helper | F | fragment world_border.json |
| biome, data_dir | helpers | F | fragment session_state.json |
| play_sound_to | helper | F | fragment play_sound_to.json (not needed after 2b) |
| title | helper | F | fragment title.json (not needed after row 3) |
| player_stat | helper | F | fragment player_records.json |
| chunk_load_request, chunk_load_state, chunk_is_loaded, chunk_load_release | helpers | F | fragment chunk_load_async.json (not in the Nightly source; not needed) |
| item_picked_up_v2 | hook | F | fragment item_picked_up_v2.json (not needed) |
| injection on `Game::guest_respawn` (G/lan_actions.rs:4019) | X | X | needs a pinned `horizon_revision`; only for the exact server hold of 1d |
| rule registration | none | G | missing in Horizon (gamerules.rs:50) |

## Step log

1. Read the brief and the audit (two reads, 390 lines).
2. Ran the guard commands: W, branch `horizon-26.3`, H at b880384.
3. Checked V0: opened integrated.rs:823 and :880, lan_actions.rs `guest_respawn`, the dispatch arm at :3089, the doc lines of `player_load_from_file` and `player_save_to_file`.
4. Opened `player_respawn_position`: the fragment, the doc, the call site and the first send (`hub.respawn`). Found the same call in the Nightly source (lan_actions.rs:3960) with `git show` using `${T}:` (zsh `$T:c` trap noted).
5. Searched the catalogs with a python parse for every hook and helper of the report: membership in main and the Nightly.
6. Read `guest_die`, `drop_inventory`, `native_player_stacks`, `native_server_players`, `native_acting_player`.
7. Searched for a hold of the Respawn click: found the client screen hooks and checked `screen_hooks_open`, `Screen::Dead`, the hook call order in game.rs.
8. Read `native_run_command`, `cmd_playsound`, `cmd_advancement`, `cmd_forceload`, the selector options.
9. Found `trigger_criterion` only in the pending fragment host_bridge.json; checked docs table at native-mods.md:5501.
10. Checked chunk tickets, column reads, `native_biome` fragment, `world_border` fragment, forceload blocking wait.
11. Checked game rules table and `rule()`; searched docs and fragments for a registration path.
12. Checked status, effects, item stack, summon and item NBT load for rows 8a, 8b, 8c; checked the reduced-debug-info push path for 8d.
13. Checked config, command, mod-screen and text-field sources for rows 5a, 5b, 6.
14. Checked the audit's open questions 4 and 9 against code; counted fragments (228).
15. Wrote this file. No em-dash or en-dash used.
