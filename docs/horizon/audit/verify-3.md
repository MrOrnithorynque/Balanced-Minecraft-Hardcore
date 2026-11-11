# Verification of audit 3 (items, Eternal tools, abilities, data, assets)

Short names: W = BMH worktree. H = S/src/horizon (main b880384). Nightly = tag nightly-source/2026-11-08-fdc87ae.
All Horizon paths below are under H. I read the Nightly source with `git show` into S/v3/ (read-only copies).
Nightly facts I measured: `NATIVE_HOOKS_API: u32 = 411` (main: 415, native_hooks.rs:1086). The Nightly source also compiles
the pending fragments, but its catalog stamps only the names in native-hooks.nightly-2026-11-08.json. Main has 2 extra hooks
(`world_hold`, `world_warp`) and 3 extra helpers (`force_chunk`, `force_chunk_in`, `forced_chunks`) over the Nightly. None matters here.

V0 check (audit 6): HOLDS. I opened every citation.
- integrated.rs:823 (game/integrated.rs) "// No player of its own: the hub runs without a host."
- integrated.rs:880 "host.hub.lan.connect_local()?" ; lan/mod.rs `pub fn connect_local(&self)` builds `Pipe::pair()`.
- integrated.rs:977 "if let Some(g) = host.hub.guests.iter().find(|g| g.uuid == owner && g.placed) {"
- lan_actions.rs:711 "self.lan_acting_uuid.replace(g.uuid)," ; lan_actions.rs:483 "let i = host.hub.guests.iter().position(|g| g.conn == conn)?;"
- enchant_effects.rs:131 "&& let (Some(uuid), Some(name)) = (self.lan_acting_uuid, self.lan_acting_name.clone())"
- lan_actions.rs:5651-5654 `guest_destroy` -> `as_guest` -> `g.break_block(pos)`; block_breaking.rs:25 `self.damage_held_with_max(per_block, max);`;
  item_durability.rs:201 and :218 "Some(held) => self.wear(&held, amount),".
- The same code is in the Nightly (enchant_effects.rs:99 and integrated.rs:810 hold the same lines).
So the owner of a single-player world is a guest, and every "server player" hook fires for it.

## Corrected verdict table

| Row | Feature | Audit said | Corrected | Tier | Mechanism | Changed? |
|---|---|---|---|---|---|---|
| 1a | Item ids, stack size, rarity, fire resistance | HOOK | HOOK | N | content mod `data/bmh/horizon/item/*.json`, template + `components` patch (content.rs:408-422) | no |
| 1b | Tool stats (33000 durability, damage, speed, enchantability, repair) | HOOK, numbers UNVERIFIED | HOOK | N | netherite template + `components` patch; exact 26.3 modifier ids still UNVERIFIED | no |
| 1c | Item classes with code | PARTIAL | PARTIAL | N | no item class; each behaviour is a hook keyed on the item id (rows 3 to 7) | no |
| 1d | Works on dedicated server | HOOK | HOOK | N | docs/mods.md:105-110 and :112-120; `side = "both"` needed; untested by anyone | no |
| 1e | Vanilla client joins | GAP | GAP | G | conn.rs:1329-1335 and :1425-1430; Horizon would have to give vanilla ids to modded items; BMH never supported it | no |
| 2 | Recipes, tags, lang, sounds.json, item models | HOOK | HOOK | N | vanilla data pack and resource pack formats; `%%` in lang IS handled (text.rs:3) | no |
| 2b | `range_dispatch` on `bmh:north` | GAP as written, PARTIAL with rewrite | PARTIAL | N | 32 leaf tree of `minecraft:condition` nodes with namespaced properties, answered by `item_model_property` (API 385, in Nightly) | no |
| 3a | Tools never break | PARTIAL (PINNED for single player) | HOOK | N | `item_damaged` caps `amount`; damage read from `carried_stack(slot)`, slot kept from `inventory_tick(selected)`; fires for the owner (V0) | YES |
| 3b | Worn out: half damage | HOOK | HOOK | N | `living_incoming_damage_v2` `amount *= 0.5` when `weapon.damage >= max-1` (mobs only) | no |
| 3c | Worn out: stone mining speed | PARTIAL (pending `break_speed`) | HOOK | N | `inventory_tick` puts a `minecraft:tool` patch (speed 4.0) on the stack; dig speed reads it (items.rs:1175). `break_speed` (F) is the fallback | YES |
| 3d | 7 percent death wear | HOOK, API 405 main-only | HOOK | N | `loot_drops` with the dying player's inventory (API 405, in Nightly); edit `stack.damage` | no (tier fixed) |
| 3e | Tooltip lines | HOOK | HOOK | N | `tooltip` (client) gets the full stack; `Text::translate` exists; `item_bar` (API 60) can keep the 1 pixel bar | no |
| 4a | Echo Strike | PARTIAL | PARTIAL | F | `hurt_enemy`/`living_incoming_damage_v2` + `server_tick` queue + `hurt_entity` (F) + mob list from `entity_join_level`; N fallback: `set_mob_health`/`kill_mob` | no |
| 4b | Axe Smash | PARTIAL | PARTIAL | N | v2 `amount *= m`; airborne fall tracked from `player_tick` + `entity_on_ground` (N); `living_fall` halves the landing | no |
| 4c | Echo Sense | GAP for glow, PARTIAL via world_primitives | PARTIAL | F | `destroy_block` + `level_block_state` + mod ore list; glow by client `world_primitives` (F) sent with `net_send_to_player` to every nearby player | no |
| 5 | Eternal Items on the ground | PARTIAL | PARTIAL | N | `item_expire` (extra_life 32766, re-asked each time); `item_spawn` + `entity_merge_nbt` Health 32767s; `damage_resistant` list for fire and cactus | no |
| 6 | Items kept after death | HOOK, main-only extension | HOOK | N | `loot_drops` (405) + `player_respawned` + `give_stack`; all in the Nightly | no (tier fixed) |
| 6b | One Eternal Item per type | HOOK | HOOK | N | `item_picked_up` cancel + `inventory_tick` + `carried_stack` + `summon` | no |
| 7 | Eternal Bundle | PARTIAL (needs guest-only or pending hooks) | PARTIAL | N | template `minecraft:bundle`; eject Eternal items from `StackData.bundle_contents` in `inventory_tick`; drop-all by claiming `use_item` | no (tier fixed) |
| 8 | Creative tab, two sprites | HOOK | HOOK | N | `creative_tab/*.json` (vanilla ids are accepted); `select` on `display_context` decodes | no |

The audit's table has 20 rows, not the 21 its "Counts" line says.

## Row notes

### 3a (changed to HOOK, tier N)
- Audit claim: `item_damaged` is not heard for the single-player owner, so PINNED with an injection into `Game::wear`.
- Evidence against: `game/enchant_effects.rs:131` "&& let (Some(uuid), Some(name)) = (self.lan_acting_uuid, self.lan_acting_name.clone())"
  runs for the owner because the owner is a guest (V0). The doc line `native_hooks.rs:9214` "nor the host of a LAN world" is stale.
- Hook: `native_item_damaged(player: u128, name: &str, item: &str, amount: &mut i32, cancel: &mut bool)` (native_hooks.rs:9207-9226), catalog `item_damaged`, contract `cancel`, Nightly yes.
  Doc: "`amount` is what is left after Unbreaking; change it (0 or less leaves the item whole)".
- The open question "no stack in the hook" has an answer without a pending helper. `inventory_tick(slot, selected)` runs each tick for every carried stack
  (native_hooks.rs:9346-9355) and `native_carried_stack(slot)` is plain field access on the swapped-in inventory (native_hooks.rs:11958-11964
  `0.. if (slot as usize) < self.inventory.len() => self.inventory.get_mut(slot as usize)`). The mod keeps `held_slot[uuid]` from `inventory_tick`, then in `item_damaged`
  reads `carried_stack(held_slot).damage` and sets `amount = min(amount, max-1-damage)`. Armour wear: scan slots 100..103.
- Every durability loss goes through `Game::wear` (callers: item_durability.rs:168, :201, :218, :257; mounts.rs:272; shield.rs:163; enchant_effects.rs:635, :1109; held_items.rs:46).
  The break itself is `apply_held_damage` with `max = items().get(&s.item).max_damage` (item_durability.rs:204-212), so a cap of max-1 holds.
- Parity: none expected. Not heard for creative players (they take no wear), which is also true in Java.

### 3c (changed to HOOK, tier N, medium confidence)
- `break_speed` is a pending fragment: tools/native-hook-fragments/break_speed.json, `"side": "both"`, absent from both catalogs. Confirmed. Its signature has no stack, and
  `held` returns only `(item, count, plain)`; `held_stack` is also pending (tools/native-hook-fragments/player_equipment.json) and "None ... on a client". So a damage test in `break_speed` is awkward.
- The audit's UNVERIFIED alternative holds: `Items::mining_speed` (horizon-items/src/items.rs:1166-1181) reads the STACK first:
  "if let Some(tool) = stack.data().and_then(|d| d.other("minecraft:tool")) { return self.stack_tool_rule(tool, block, "speed")...". The Nightly has the same line (items.rs:1122).
  `StackData::set_other(&mut self, id: &str, value: Option<Tag>)` is public (item_data.rs:197; Nightly :165). `carried_stack` gives `&mut ItemStack`.
- UNVERIFIED: that the client receives the `minecraft:tool` patch with the stack (item_sync.rs not traced). If it does not, only the crack animation prediction differs; the server decides the break.
- Parity risk: the patch must copy the template's rules (drop rules) and change only `speed`; remove the patch when the tool is repaired.

### 3d and 6 (tier N, not main-only)
- Nightly `NATIVE_HOOKS_API` is 411 (S/v3/nightly_native_hooks.rs, grep). The Nightly says: native_hooks.rs:21 "API 405 adds a dying player's inventory to `native_loot_drops`", and nightly mod_api.rs:280-284 "A dying player's inventory is a call of its own too, after vanishing curse items are gone".
- Call site: game.rs:4294 `fn drop_inventory(...)` in the Nightly; in main game/item_entities.rs:601-620 `self.native_loot_drops(&context, &mut stacks);` with `entity: Some((who.1 as u32, "minecraft:player"))`.
  Its only caller is `game.drop_inventory(eyes, (uuid, id))` inside `as_guest` (lan_actions.rs:4614, Nightly :4433). The owner dies through that guest path (V0), so the hook fires for the single-player owner.
- It returns early with keep_inventory (item_entities.rs:602-604), as the audit said. The drops list holds slots 0..36, armour and the off hand (`death_drops`), so the Eternal stacks can be removed whole.
- `player_respawned` (native_hooks.rs:8657-8663): "with their state swapped in", so `give_stack` for the acting uuid takes the branch at native_server.rs:1337-1345 and calls `self.give(stack)`. Called in lan_actions.rs:4208 inside `as_guest`.

### 3b, 4a, 4b: the v2 hook
- `living_incoming_damage_v2` call site: game/mace.rs:216-228 inside `hurt_by_player`, after `refuses_damage` and before armour. `held.as_ref()` is the weapon with its `damage` field, so "worn out" is testable.
- `amount` already includes the attack strength scale: melee.rs:200 `let (amount, scale, crit, magic) = self.melee_damage(&target_kind, true);` then melee.rs:219 `self.hurt_by_player(index, amount, source)`.
  So the Java 0.9 strength test can be approximated as amount versus the weapon's base damage (no helper gives `scale`). `attack_strength_scale()` (melee.rs:6-14) is private. X-tier option: inject into `Game::attack_mob` (game/melee.rs:200) to read `scale`.
- `hurt_enemy` (catalog, Nightly, API 39) says "A player's hit landed on the mob `entity` of type `kind`, holding `stack`: where `Item.hurtEnemy` is called" (native_hooks.rs:9338-9343). It is the cleaner "hit landed" signal for Echo Strike and Axe Smash effects.
- Players as victims are not scaled by v2 (doc: "melee, sweep and spear hits" on mobs). PvP parity gap stays.

### 4a Echo Strike (PARTIAL, tier F)
- `native_hurt_entity(dimension, target: u32, amount: f32, damage_type: &str, attacker: Option<u32>) -> bool` is a pending fragment (tools/native-hook-fragments/hurt_entity.json). The doc says the damage hooks hear it, so the mod needs a re-entrancy flag. Confirmed (native_entities.rs:408-440).
- `native_entities_in` (entities_in.json) is pending. I found a way around it in the Nightly catalog: `entity_join_level`, `entity_leave_level`, `mob_spawned`, `entity_tick_post` are hooks; `entity_position`, `entity_kind`, `entity` are helpers. A mod can keep its own list of hostile mobs and read their positions. Cost: a per-tick hash map. `native_mobs(dimension)` (docs/native-mods.md:3735, native_mobs.rs:233) exists but is in neither catalog and not in the Nightly Host: do not rely on it.
- N-tier fallback for the damage: `set_mob_health` / `kill_mob` (catalog helpers). They skip armour, knockback, the hurt sound and invulnerability frames.
- Dust colour is supported: `ParticleOptions::dust(rgb, scale)` (horizon-client-render/src/particles.rs:181); `mod_particles_ok` (native_hooks.rs:10038-10046) refuses only block-state types, `Item`, `Trail`, `Vibration`. Audit open question 8 is answered. `Host::send_particles` and `Host::play_sound` are in the Nightly mod_api.rs (lines 1027, 1036).

### 4b Axe Smash (PARTIAL, tier N)
- Refutes the audit: `on_ground` is not only in the pending `server_player_state`. Helper `entity_on_ground(entity)` (native_entity_motion.rs:88, API 379, in the Nightly catalog) answers for players: it calls `native_client_player` or `native_target_guest(t).on_ground`.
  The player's entity id comes from `NativeServerPlayer.entity_id` (API 398, nightly mod_api.rs:554).
- No fall-distance reader exists (grep of native_*.rs, docs, fragments: only `native_trample`, `native_block_fall_on`, `living_fall`). The mod tracks peak y while not on ground. Water, ladder and elytra resets are the mod's job, so parity is approximate.
- `living_fall(entity, kind, distance: &mut f64, multiplier: &mut f32, cancel: &mut bool)` (landing.rs:28-39, side both, Nightly) halves the landing. UNVERIFIED: that a guest's landing calls it (motion.rs `FallEvent` path, not traced to the end). Medium confidence.

### 4c Echo Sense (PARTIAL, tier F)
- Glow confirmed absent for displays: text_displays.rs:58-60 "(not the view range, size or glow)"; `mark_outline` has no hit in text_displays.rs. `entity_outline` (fragment) says "make any mob, player, dropped item or projectile glow", so no displays.
- `world_primitives` is a pending client hook (fragment, `"side": "client"`, contract `collect`). `box_outline(...).through_walls()` is documented (docs/native-mods.md:4784).
- Refuted parity risk: the audit says the glow is visible to the miner only. The server can send the ore positions to every player of the dimension within range (`server_players` + `net_send_to_player`, both in the Nightly catalog), so everyone sees it, as in Java. Each player needs the mod on the client.
- Tag check: `native_server_block_in_tag` (native_world_read.rs:67) is in neither catalog nor the Host. Use a hard-coded ore list read from the mod's own `native_data!` copy of `echo_sense_ores.json`.
- `destroy_block` fires before the break (catalog: cancellable, server). Same as the audit said.

### 5 Items on the ground (PARTIAL, tier N)
- `item_expire` (native_hooks.rs:9243-9255, API 325): call site item_entities.rs:150-158 inside `merging` (`self.remote.is_none()`, so authoritative side, the integrated server included). `item.age = 6000 - extra_life.min(32766)`. Confirmed.
- Damage: lava, fire, cactus, explosion paths confirmed (entities.rs:478-520; explosion.rs:722-740 "if &*item.stack.item != "minecraft:nether_star"").
  `resists_fire` (items.rs:693-707) and `resists_cactus` (items.rs:716-740) read the stack's `minecraft:damage_resistant` first. A `types` LIST holding the sources `minecraft:lava`, `minecraft:in_fire`, `minecraft:on_fire` and `minecraft:cactus` would cover fire and cactus; whether a content mod may declare a list there is UNVERIFIED.
- Explosions: `item.health = (item.health as f32 - amount) as i32`, so a Health of 32767 (`entity_nbt.rs:2790-2793` writes a `Short`) is the only lever. `entity_merge_nbt` (API 386, Nightly) can set it. `ItemEntity` has no `invulnerable` field (entities.rs:198-226).
- Parity: a blast removes health; 32767 survives about 6500 normal blasts. Lava is covered if fire resistance is set.

### 6b (HOOK, tier N)
- `item_picked_up` (native_hooks.rs:9229-9236, API 40) is called by `native_pickup_refused` (native_hooks.rs:11853-11866) for the acting player, after `item_picked_up_v2` (pending). Cancel works.

### 7 Eternal Bundle (PARTIAL, tier N)
- Bundle by component confirmed (items.rs:943; bundle.rs:64-69). The insert rule is hard coded: bundle.rs:94-97 `can_be_in_bundle` refuses only "shulker_box". Ender chests and Eternal Items are allowed. No hook edits it. Audit confirmed.
- Refutes the audit's need for `inventory_click` (pending, guests only): `StackData.bundle_contents: Vec<ItemStack>` is public (item_data.rs:58 in Nightly), `carried_stack` gives `&mut ItemStack`, and `inventory_tick` runs for every carried stack. A tick scan can move a forbidden stack out of the bundle and `give_stack` it back. Cost: the item sits inside for up to one tick (a visible flicker). The owner is a guest (V0), so the "single-player host" worry vanishes.
- Drop-all: Horizon follows vanilla 26.1 (use_hand.rs:347-352 "BundleItem.use: startUsingItem, then onUseTick empties it", 200 ticks in item_use.rs:72). To match Java 1.21.1 (drop all at once) claim the click with `use_item` (native_hooks.rs:8192-8197, "Set `claimed` to take the use ... the item's own use doesn't run", server, API 13), empty `bundle_contents` and `summon` the stacks. The audit's "vanilla 26.3 drops one at a time: UNVERIFIED" is now verified.

### 2b (PARTIAL, tier N)
- Foreign `range_dispatch` property reads 0: item_model.rs:507-526 "A property Horizon cannot read is 0". Confirmed. Namespaced `minecraft:condition` is NOT foreign: item_model.rs:435-444 returns `ConditionProperty::Custom(property)` before the foreign branch. Hook `item_model_property` is API 385 ("API 385 adds per-stack conditional item model properties", nightly native_hooks.rs:64) and present in the Nightly catalog, so the doc line "Bundles need hooks 385 once this pending hook is stamped" (docs/native-mods.md:66) is old. The audit's rewrite is sound.
- The hook sees `StackView` (item, count, explicit patch), not the camera. The camera yaw comes from `native_camera` (client). Amulet group owns the rest.

### 8, 1a, 1b, 2
- Creative tab: mod_content.rs:1809-1811 `let known = |id: &str| new.contains(&id) || self.vanilla_items.contains(id);` so `minecraft:bundle` is accepted. Audit's UNVERIFIED is resolved.
- `select` on `minecraft:display_context`: item_model.rs:735 and :187 confirmed.
- Lang `%%`: horizon-client-core/src/text.rs:3 "(`%s`, `%1$s`, `%%`)" and :857 test `format_translation("100%% sure %s", ...)`. Resolved.
- Item struct: content.rs:408-422 confirmed; docs/mods.md:248-262 confirmed; "A component type no vanilla item has is refused" confirmed (docs/mods.md:333).
- Nightly has the same `components: BTreeMap` (content.rs:350 there), `creative_tabs` (mod_content.rs:1729), `ConditionProperty::Custom` (item_model.rs:440).

### 1d, 1e
- docs/mods.md:105-120 quotes confirmed. conn.rs:1329-1335 and :1425-1430 confirmed. Native mods are not compared at join ("Native mods are not in the list yet", docs/mods.md:119-120): a server and client with different native builds are not refused. Add that to the parity notes.

## Claims refuted
1. Audit 3, rows 3a and 12-open-question 2: "`item_damaged` does not fire for the single-player owner (PINNED)". Refuted by V0: enchant_effects.rs:131 plus lan_actions.rs:711 (the owner is a guest).
2. Audit 3, rows 3d and 6: "API 405 is main-only, Nightly unknown". Refuted: Nightly `NATIVE_HOOKS_API` is 411 and the Nightly source carries the 405 text and the call (nightly native_hooks.rs:21, nightly mod_api.rs:280, game.rs:4294-4311).
3. Audit 3, parity risk of 3e: "`item_bar` UNVERIFIED that it can change the width". Refuted: native_hooks.rs:8127-8131 "`bar` arrives holding vanilla's value ... and may be replaced (width `0..=13`, colour `rgb`) or set to `None`".
4. Audit 3, 4c: "Visible to ONE player (the miner) only". Not forced: the server can send to every player (`server_players`, `net_send_to_player`).
5. Audit 3, 4b: "`on_ground` is in the pending `server_player_state`". Incomplete: `entity_on_ground` (native_entity_motion.rs:88, API 379) is stamped and handles players.
6. Audit 3, row 3c: "UNVERIFIED that a per-stack `minecraft:tool` patch is honoured by the dig speed code". Verified at items.rs:1175-1181.
7. Audit 3, row 7: "refusing Eternal Items needs `inventory_click` (pending, guests only)". Not needed: tick scan of `StackData.bundle_contents` (N-tier).
8. Audit 3, row 7: "Not tested whether vanilla 26.3 drops one at a time". Verified: use_hand.rs:347 "BundleItem.use: startUsingItem, then onUseTick empties it".
9. Audit 3, open question 8 (dust colour): `ParticleOptions::dust` exists and is not refused (native_hooks.rs:10038-10046).
10. Audit 3, lang `%%` and translatable `Text` (3e): both exist (text.rs:3, text.rs:239 `Text::translate(key, args)`).
11. Audit 3, step 9: "main-only names (`force_chunk*` only)". Also two hooks: `world_hold`, `world_warp`.
12. Audit 3, 4a: "`entities_in` is the only way to find candidates". A hook-built mob list is possible (`entity_join_level` etc.).

## Citations that did not match
- None of substance. Line drift only: `native_hooks.rs:9535-9547` for the v2 signature is at 9525-9546; docs/native-mods.md:513-526 is 513-517 for the v2 text (and that text still says "receives its API number at merge" though the code stamps 384). The audit's `foreign.rs:1` is `crates/horizon-assets/src/item_model/foreign.rs:1` (correct quote).
- All other cited quotes I opened matched (enchant_effects.rs:119-138, item_entities.rs:149-157, entities.rs:483-520, explosion.rs:728-735, content.rs:408-422, items.rs:693-712 and :943, bundle.rs:64-69, mod_content tab text, native_hooks.rs:9207-9217, :9243-9255, :10119-10128, :10141-10170, mod_api.rs:297-310, :700-701).

## Hooks and helpers the rows need

| Name | Kind | Tier | Catalog |
|---|---|---|---|
| `item_damaged` | hook | N | main + Nightly (API 162) |
| `inventory_tick` | hook | N | both (API 39) |
| `carried_stack` | helper | N | both |
| `living_incoming_damage_v2` | hook | N | both (API 384) |
| `hurt_enemy` | hook | N | both (API 39) |
| `loot_drops` (player inventory, API 405) | hook | N | both; extension in Nightly source |
| `player_respawned` | hook | N | both (API 115) |
| `give_stack` | helper | N | both (API 98) |
| `item_picked_up` | hook | N | both (API 40) |
| `item_expire`, `item_spawn` | hooks | N | both |
| `entity_merge_nbt` | helper | N | both (API 386) |
| `tooltip` | hook (client) | N | both (API 12) |
| `item_bar` | hook (client) | N | both (API 60) |
| `item_model_property` | hook (client) | N | both (API 385) |
| `use_item` | hook | N | both (API 13) |
| `destroy_block` | hook | N | both (API 14) |
| `living_fall` | hook | N | both (API 357) |
| `player_tick`, `server_tick`, `entity_join_level`, `entity_leave_level`, `mob_spawned`, `entities_load` | hooks | N | both |
| `entity_on_ground`, `entity_position`, `entity_kind`, `entity`, `entity_velocity` | helpers | N | both |
| `level_block_state`, `summon`, `remove_entity`, `entity_add_tag`, `entity_tags`, `set_mob_health`, `kill_mob`, `net_send_to_player`, `server_players` | helpers | N | both |
| `Host::play_sound`, `Host::send_particles` | Host methods | N | Nightly mod_api.rs:1027, :1036 (not in the helper catalog) |
| content `items`, `creative_tab`, recipes, tags, lang, sounds, item models | content data | N | Nightly source has them |
| `component_types`, `modify_default_components`, `items` (native) | hooks | N | both; not needed |
| `break_speed` | hook (both) | F | fragment only; fallback for 3c |
| `hurt_entity` | helper | F | fragment only (4a) |
| `entities_in` | helper | F | fragment only; avoidable (4a) |
| `world_primitives` | hook (client) | F | fragment only (4c) |
| `held_stack`, `player_equipment` | helpers | F | fragment only; avoidable |
| `inventory_click` | hook | F | fragment only; not needed now |
| `server_player_state` fields (`on_ground`, `sneaking`...) | struct fields | F | fragment only; `entity_on_ground` replaces `on_ground` |
| `entity_outline` | hook (client) | F | fragment only; draws no displays, no use |
| X option: `Game::attack_mob` (game/melee.rs:200) | injection | X | to read `scale` for Echo Strike exactly |
| `native_server_block_in_tag`, `native_mobs` | game fns | G/unknown | in neither catalog; do not rely on them |

## Step log
1. Tree guard: W printed the expected root and branch `horizon-26.3`; H HEAD printed b880384.
2. Read the brief and the whole audit 3.
3. Opened every V0 citation of audit 6 (list above). All hold.
4. Loaded both catalogs with python3 -I; compared names; listed the 228 fragment files.
5. Read the Nightly sources with `git show` (tag in `${T}:path` form) into S/v3: `NATIVE_HOOKS_API` is 411; 405, 385 and the Host methods are there.
6. Traced `loot_drops` for a dying player to `drop_inventory` and to its single caller in lan_actions.rs.
7. Traced `item_damaged` to `Game::wear` and its callers; checked the stack-read helpers (`carried_stack`, `held`, `player_stacks`, pending `held_stack`).
8. Checked `mining_speed`, `StackData`, `set_other` and the `break_speed` call site (hand.rs:417-440).
9. Traced v2: mace.rs `hurt_by_player`, melee.rs `attack_mob`, `melee_damage`.
10. Checked item entity damage, expiry, NBT merge, `damage_resistant` readers.
11. Checked bundle insert rule, use start, `bundle_contents`, `use_item`.
12. Checked item model decoding (condition, range_dispatch, select), creative tab known ids, lang `%%`, `Text::translate`, `ParticleOptions::dust`.
13. Checked glow code paths (text_displays, `entity_outline`, `world_primitives`) and `entity_on_ground`.
14. Checked the join and content docs (mods.md, conn.rs).
15. Wrote this file. No em-dash or en-dash was typed; no file other than this one was written (copies of Nightly sources sit in S/v3, scratch).
