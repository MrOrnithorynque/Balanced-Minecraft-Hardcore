# Verification of audit 2 (Totem of Eternity, moon ritual, anvil)

Paths: W = the BMH worktree, H = S/src/horizon (S = the session scratchpad). All Rust paths below are under H/crates/horizon-client/src/game/ unless a full path is given.
Checks done first: W is the toplevel, the branch is horizon-26.3, H is at b880384691b88d317553067109d28e785990bda3.

## Audit 6's V0 (the owner is a hub guest): holds

- `integrated.rs:823` "// No player of its own: the hub runs without a host."
- `integrated.rs:880` "host.hub.lan.connect_local()?"
- `integrated.rs:977` "if let Some(g) = host.hub.guests.iter().find(|g| g.uuid == owner && g.placed) {"
- `lan_actions.rs:711` "self.lan_acting_uuid.replace(g.uuid),"
- `lan_actions.rs:472` "pub(super) fn enter_guest(" and `lan_actions.rs:483` "let i = host.hub.guests.iter().position(|g| g.conn == conn)?;"
All six quotes are on the cited lines. So every "not heard for a LAN host's own player" claim in audit 2 is refuted for single player (the owner is a guest) and for a LAN world opened from single player (same hub).

## Corrected verdict table

| Row | Feature | Audit said | Corrected | Tier | Mechanism | Changed? |
|---|---|---|---|---|---|---|
| 1a | Save from death | HOOK | HOOK (mechanism changed) | N | `living_death` cancel only records the death; the swap runs in the next `player_tick` (player swapped in). `take_items`/`give_stack` fail inside `living_death`. | yes |
| 1b | Save: effects | GAP today | HOOK | N | `run_command` (`effect clear`, `effect give <uuid>`) from `server_tick`, one tick after the cancel. `player_effects` (F) is the clean route. | yes |
| 1c | Save: animation, event 35 | PINNED | PINNED | X | Inject in `Game::update_totem_ready` (totem.rs:53), `apply_totem` (totem.rs:67) and `find_totem` (totem.rs:166). No hook sends event 35. | no |
| 1d | Undying loses Fire Resistance | HOOK | HOOK | N | `modify_default_components` on `minecraft:totem_of_undying`. Mobs lose it too: they read the same component. | no (open question closed) |
| 2 | Items, ids, tooltips, glint, rarity | HOOK | HOOK | N | `items` hook (template, rarity, components), `tooltip` hook, combined .hmod for model and texture. | no |
| 3 | Anvil awakening, gold repair | GAP now, HOOK later | HOOK | F | `anvil_update` is compiled in main (fragment, lan_stations.rs:200), no API number. `prepare_anvil` has no inputs. | yes (tier) |
| 4 | Moon ritual server logic | HOOK, cooldown GAP | HOOK | F | Use hooks as audited. Cooldown is NOT a gap: `minecraft:use_cooldown` component. XP level read needs pending `server_player_state`. | yes |
| 5a | Ritual look, first person | PARTIAL | PARTIAL | X | `render_hand_pose` moves arm and item. Hand light is fixed at frame_geometry.rs:469. Hide-arm-only has no hook. | no |
| 5b | Runes, glint while charging | PARTIAL | PARTIAL | X | Server `send_particles` (N, one position for all). `add_particle` and `send_particles_to` are F. Dynamic glint needs item_model.rs:1216. | no |
| 6a | One per inventory | HOOK | HOOK | N | `player_tick` + `carried_items` + `take_items` + `summon`. The owner is covered (V0). | no |
| 6b | One per container: hoppers | PARTIAL | PARTIAL | X | `inventory_move_item`, `hopper_pickup_item` (N). Droppers call no hook (machines.rs:1766). Entity containers are not readable. | no (droppers resolved) |
| 6c | One per container: menus | GAP | PARTIAL | F | `inventory_click`/`inventory_drag` (F) cancel, but give no container contents. Post-hoc: `player_container_open/close` + `container_slots` + `extract_item` (N). | yes |
| 7 | Client prediction of a use | HOOK | HOOK | N | Hooks run on both sides. A refused start still consumes the click. Client must compute the same start test. | no |

## Row notes

### 1a: the audit's mechanism does not work inside `living_death`; the verdict survives with a deferral
- `living_death` has one call site for players: `lan_actions.rs:4488` "self.native_living_death(" inside `fn guest_die(&mut self, host: &mut LanHost, gi: usize, conn: u32)` (`lan_actions.rs:4471`). The hub is a parameter there. No guest is swapped in (`guest_die` is called at `lan_actions.rs:4438`, after the `as_guest` blocks have closed).
- `native_take_items` (native_server.rs:1278) starts "if self.player.is_none() {" "return 0;". On the integrated and dedicated server `self.player` is None between turns (audit 6 cites `dedicated.rs:1500` "self.player = None;").
- `native_give_stack` (native_server.rs:1328): for a uuid other than the acting one it reaches "let Some(host) = self.lan.as_mut() else {" "return Some(stack);". `self.lan` is taken while the tick runs, so the stack comes back unplaced.
- `native_give_levels` goes through `native_as_player` (native_server.rs:1503), which has "let Some(mut host) = self.lan.take() else {" "return false;". Same failure.
- `native_run_command` (native_server.rs:951) "if self.lan_acting.is_some() || self.lan.is_none() {" "return false;". Same failure inside `living_death`.
- Working route (all N): in `living_death` (cancel = true when the player carries a Totem of Eternity and `damage_type` is not a bypass type) record the entity id. In the next `player_tick` (`lan_actions.rs:4419` "g.native_player_tick();" inside `as_guest`) the guest is swapped in: `take_items` then `give_stack`, or `give_items`. Match the id with `native_acting_player().entity_id` (NativeServerPlayer.entity_id, API 398, which is in the Nightly: API 399 to 411 hooks are in it and `world_hold` API 414 is the first main-only item). The mapping `native_entity_uuid` (native_entity_refs.rs:144-146) calls `command_guest`, which I did not trace: UNVERIFIED inside `guest_die`.
- Health: `lan_actions.rs:4496` "host.hub.guests[gi].health = 1.0;" (the audit cited 4499-4500). Good.
- Order with the Totem of Undying: `health.rs:974-981` `death_protection` returns false for the `BypassesInvulnerability` tag, otherwise sets "self.totem_fired = true;". The game checks hands first. Matches Java.
- Bypass list: the game has the tag test (`health.rs:975` "self.source.immunity_tag(ImmunityTag::BypassesInvulnerability)"). The hook gets only the `damage_type` string, so the mod must hard-code the list. Parity risk stays.
- LAN host claim: refuted by V0. The remaining difference is that the swap lands one tick late.
- Alternative with the same worst tier N and no deferral: none found. `living_incoming_damage` is a function without `self` (native_hooks.rs:9451, "No self: the game is in the middle of the hit"), so it cannot touch an inventory.

### 1b: refuted as a GAP
- `native_run_command` works "Only where no player's state is swapped in (`native_tick`, `native_menu_clicked`)" (native_server.rs:947-949). `server_tick` is that hook: native_lifecycle.rs:239 "self.native_server_tick(&context, phase);".
- `/effect give` targets guests: commands/effects.rs:61-62 "Target::Player | Target::Guest(_) => {" ... "self.for_player(t, |g| g.add_player_effect(instance))". Flags: effects.rs:63 "// `new MobEffectInstance(effect, duration, amplifier, false, particles)`." That equals the totem's list (mobs.rs:1439-1465: regeneration 900/1, absorption 100/1, fire_resistance 800/0, `ambient: false`).
- Plan: `living_death` marks the player; `player_tick` swaps the totem; `server_tick` runs `effect clear <uuid>` and three `effect give <uuid> ...` (45 s, 5 s, 40 s). Effects arrive one or two ticks after the cancel. In that window a second hit finds no Regeneration/Absorption. Medium confidence: I did not confirm `self.lan` is Some inside `server_tick` for the integrated server, but the doc names `native_server_tick` as a place where it works.
- Clean route: `player_effects` fragment (F), `native_add_effect(uuid, effect)`.
- Not needed: `mob_effect_added` (content-mod effects only, native_hooks.rs:9429).

### 1c: PINNED stands, tier X
- `totem.rs:53` "pub(super) fn update_totem_ready(&mut self) {" reads only "self.held()" and "self.offhand".
- `totem.rs:67` "pub(super) fn apply_totem(&mut self) -> Option<Body> {" takes the totem from the hand cell (totem.rs:86 "let main = self.held().is_some_and(|s| self.has_death_protection(s));").
- `totem.rs:166` "fn find_totem(&self) -> ItemStack {" only hands.
- Important fact for a simpler injection: `totem.rs:23` doc "a hand's item carrying the `death_protection` component, whatever item it is". A Totem of Eternity declared with a `minecraft:death_protection` component works as a native totem when it is in a hand. The injection only has to bring an inventory totem into the check.
- No hook emits entity event 35 or the activation animation. I grepped `entity_event|item_activation|activation|totem` in mod_api.rs, native-mods.md and the fragments folder: only `player_totem` (listen, native_hooks.rs:9157).
- N fallback: `play_sound` of `minecraft:item.totem.use` and `send_particles` (type `TotemOfUndying` accepted: UNVERIFIED). No item overlay.
- The injection kinds exist (H/../horizon-mixins README: HEAD, RETURN, OVERWRITE). The README says "atomic Linux staging": whether a macOS build works is UNVERIFIED.

### 1d: HOOK stands; open question 3 closed
- `item_defaults.rs:139` "pub fn native_modify_default_components(item: &str, components: &mut Map<String, Value>) {" ✓. DOC 5083-5086 "Any item can be changed, vanilla's included."
- Mobs: `totem.rs:35-43` `sync_mob_death_protection` copies each mob's hand component into `hand_protection`; `mobs.rs:4358` "match self.extra.hand_protection.clone() {" uses it, and the fixed list at `mobs.rs:1439` is only the fallback "before the game has" read the components. So the edit also strips Fire Resistance from mobs that hold a totem. Parity risk confirmed.
- The reader is `horizon-items/src/items.rs:2768` "pub fn death_effects(component: &Value) -> Vec<ConsumeEffect> {" (the audit wrote items.rs:2765 without the crate).

### 2: HOOK stands
- `native_hooks.rs:3137` "pub(super) fn native_items(items: &mut Vec<horizon_mods::content::Item>) {"; struct at `horizon-mods/src/content.rs:405-418` has `template`, `max_stack_size`, `rarity`, `components`. docs/mods.md:248-250 "any other 26.3 component set or removed in `components`". ✓
- `Text::translate` for the mod's lang keys on the client: UNVERIFIED (not traced).

### 3: tier F
- `lan_stations.rs:200` "self.anvil_update(uuid, &name, left, right, item_name, &mut answer);" and `lan_stations.rs:205` "self.native_prepare_anvil(uuid, &name, gs.pos, &mut result, &mut gs.menu.data[0]);". `anvil_update` is wired in the guest path (lan_stations.rs:200) and the window path (stations.rs:783). The fragment `tools/native-hook-fragments/anvil_update.json` has `"@API:anvil_update@"`: pending, so F, not GAP. The audit's cited lines (stations.rs:699-718, lan_stations.rs:182-196) are about 80 and 18 lines off.
- `prepare_anvil` receives no inputs (native_hooks.rs:6111). `player_stacks` reads only the 36 inventory slots (native_menus.rs:232-243), not the anvil slots, and needs `self.lan` (None in lan_stations). `anvil_craft_pre` fires on take only. No N route for the awakening.
- Gold repair: vanilla must also accept gold block as repair material for the Eternal tools (a `minecraft:repairable` component in `items`), else vanilla gives no output and `anvil_update` must build the whole output. UNVERIFIED which.

### 4: HOOK, tier F
- Start/tick/finish/stop/release/on_use_tick: signatures and docs match (native_hooks.rs:8236-8440). Tick cancel finishes the use: native_hooks.rs:8312 "A cancel sets it to -1." and 8316-8318 "NeoForge's javadoc says a cancel stops the use, but its code finishes it; this follows the code." Code path: using_item.rs:99-103 "if !self.use_item_ticked(&stack, ticks) {" ... "self.complete_using_item(slot, &stack, &info);".
- Cooldown is not a gap. `game.rs:5204` `apply_use_cooldown` reads "i.use_cooldown" of the item (the `minecraft:use_cooldown` component) and runs in `complete_using_item` (item_use.rs:~131) and after `release_using` with `used = true` (item_use.rs:199-202 "if used {" "self.apply_use_cooldown(stack);"). Give the Dormant Totem `use_cooldown` 1 s in the `items` hook. Held-key guard: remote.rs:8921-8924 "an item cooling down passes". Difference to Java: success also starts the cooldown on the Dormant Totem id (harmless, the stack is replaced). A blocked start (hook cancel) starts none, as in Java.
- XP level read: `NativeServerPlayer.xp_level` (mod_api.rs:698) is documented "API @API:server_player_state@" (fragment server_player_state.json): pending, F. The audit used "level" from the catalog description, which is the permission level. Workarounds are weak (`player_level_change` hears changes only). `give_levels` itself is N.
- `effects` for the sight-blocked test (Blindness, Darkness): F (`player_effects`, `living_helpers`).
- Hurt check: `living_incoming_damage` has no `self` and no uuid; match on `entity_id`. `living_damage` is F.
- No moon, no environment attribute, no "dark outside" helper: confirmed (grep `moon` in the fragments folder: no hit; only `day_time` and `weather`).
- Client prediction (open question 7): remote.rs:8930 "if self.with_hand(hand, |g| g.start_mod_use(&held)) {" starts the use on the client; if the server refuses, the client keeps the use until the right button is released (`remote_tick_using`, remote.rs:9271-9277). How the server's refusal reaches the client: UNVERIFIED.
- Particles and sounds on the server: `play_sound` and `send_particles` work "from any server hook" (docs/native-mods.md:5502). ✓

### 5a, 5b: PARTIAL stands
- `render_hand` only hides (native_hooks.rs:2939 "It only hides: the hook has no way to draw a replacement."). `render_hand_pose` adds offsets (native_hooks.rs:2965). ✓
- Hand light: `frame_geometry.rs:469` "let light = self.world.light(" with `eye`, then "block_entity_render::vertex_light(light)". No hook. Injection site X. (The audit said ~460.)
- Glint: `horizon-assets/src/item_model.rs:1216` "fn has_foil(&self) -> bool {" reads only the stack's `enchantment_glint_override`. No dynamic hook: X.
- `client_particles.json`: `add_particle`, F. `send_particles_to.json`: F. Whether `send_particles` accepts `ENCHANT`: UNVERIFIED (native_hooks.rs:10143-10145 excludes only item, trail, vibration).

### 6a: HOOK stands
- `carried_items` counts "take(36)" slots plus the off hand, plain stacks only (native_server.rs:1262-1276). `take_items` is "the main hand first" (native_server.rs:1277). The owner is a guest, so `player_tick` fires for it (V0).

### 6b: PARTIAL stands, droppers resolved
- `native_hooks.rs:6926` `native_inventory_move_item` is called only from machines.rs:1292 and :1407 (hopper paths).
- Droppers: `machines.rs:1735` "Some(into) => Some(self.dropper_insert(&into, one, face))" and `machines.rs:1766` "fn dropper_insert(&mut self, into: &View, one: ItemStack, face: Option<Direction>) -> bool {" call `self.add_item(None, into, one, face)` with no hook. A dropper can put a totem into a container with no mod able to stop it: X (machines.rs:1766). This was UNVERIFIED in the audit.
- Entity containers (chest minecart, chest boat) are "reported at the block it is in"; the helpers "aren't reached yet" (docs/native-mods.md:3829). The mod cannot read their contents.

### 6c: GAP becomes PARTIAL, tier F
- `inventory_click.json`: "native_inventory_click(player, name, window, slot, button, mode, cancel)" (pending). It gives no container contents and no cursor stack, so the mod cannot decide "this click puts a second totem in". `menu_contents` is for mod menus (F, menu_input.json).
- N substitute: `player_container_open` (heard once a tick, menu type and pos), then in `player_tick` read the block's slots with `container_slots` and take the extra with `extract_item` (docs/native-mods.md:3821-3831), give it back with `give_stack`. Works for chests, barrels, shulker block entities. Not UNVERIFIED-free: whether it reaches the ender chest and a double chest's second half. Low confidence.

### 7: HOOK stands
- `start_mod_use` returns true after calling `start_using_item` whatever the hook answered (item_use.rs:99-105 "self.start_using_item();" "true"), so the click is consumed on both sides, as Java's CONSUME. ✓ use_hand.rs:301 and remote.rs:8930 call it.
- The owner is a guest, so server-side hooks fire for it (V0). No change.

## Claims refuted
- 1a: "not heard for a LAN host's own player": refuted by V0 (the single-player owner and a LAN world's host are guests of the integrated hub).
- 1a: "`take_items`/`give_stack` inside `living_death`" does the swap: refuted. `native_take_items` returns 0 with `self.player` None (native_server.rs:1278) and `native_give_stack` returns the stack with `self.lan` None (native_server.rs:1328).
- 1b: "GAP today": refuted. `run_command` from `server_tick` runs `/effect` on guests.
- 4: "20-tick cooldown is a GAP or a parity change": refuted for the failure path. `minecraft:use_cooldown` plus `apply_use_cooldown` give it.
- 4: "`native_server_players()` has `level`" as the XP level: refuted. The field is the permission level; `xp_level` is pending.
- 6b: droppers "UNVERIFIED": resolved, no hook (machines.rs:1766).
- 6c: "GAP": softened to PARTIAL (post-hoc scan, N).
- 1d open question 3: mobs read the component (`hand_protection`).
- 3: "GAP now": it is F (the hook is compiled in).

## Citations that did not match
- `lan_actions.rs:4499-4500` (health = 1.0): actual 4496.
- `lan_actions.rs:4480` (`generic_kill`): actual 4479.
- `lan_actions.rs:4540` (outOfWorld): actual 4551.
- `stations.rs:699-718` and `lan_stations.rs:182-196` (`anvil_update` call): actual `stations.rs:783`, `lan_stations.rs:200`.
- `item_use.rs:96-106` (`mod_use`): `mod_use` is 89-97, `start_mod_use` 99-105.
- `frame_geometry.rs:~460` (hand light): actual 469.
- `totem.rs:90-93` (death_effects loop): actual 110.
- `items.rs:2765`/`2768`: the file is `horizon-items/src/items.rs`, not in horizon-client.
- Not checked (UNVERIFIED): `S/horizon-docs/mod-code.txt:57`.
None of these is a dead finding: each quote exists a few lines away.

## Hooks and helpers the rows need
| Name | Kind | Tier | Catalog |
|---|---|---|---|
| living_death | hook | N | Nightly and main (API 392) |
| player_tick | hook | N | Nightly and main |
| server_tick | hook | N | Nightly and main |
| player_totem | hook (listen) | N | Nightly and main |
| modify_default_components | hook | N | Nightly and main |
| items | hook | N | Nightly and main |
| tooltip | hook | N | Nightly and main |
| use_duration, use_animation | hooks | N | Nightly and main |
| living_entity_use_item_start / _tick / _stop / _finish | hooks | N | Nightly and main |
| on_use_tick, release_using | hooks | N | Nightly and main |
| render_hand, render_hand_pose, hand_drawn | hooks | N | Nightly and main |
| inventory_move_item, hopper_pickup_item | hooks | N | Nightly and main |
| player_container_open, player_container_close | hooks | N | Nightly and main |
| living_incoming_damage | hook | N | Nightly and main |
| prepare_anvil, anvil_craft_pre | hooks | N | Nightly and main |
| take_items, carried_items, give_items, give_stack, give_levels, run_command | helpers | N | Nightly and main |
| entity_uuid, entity_view_vector, day_time, weather, summon, container_slots, extract_item | helpers | N | Nightly and main |
| play_sound, send_particles (Host methods) | helpers | N | via Host |
| NativeServerPlayer.entity_id (API 398), game_mode | field | N | Nightly (API below 412) |
| anvil_update | hook | F | pending fragment |
| player_effects (add_effect, set_effects) | helpers | F | pending fragment |
| server_player_state (xp_level, using_item, selected_slot) | fields | F | pending fragment |
| living_helpers (entity_effects) | helper | F | pending fragment |
| living_damage | hooks | F | pending fragment |
| inventory_click, inventory_drag | hooks | F | pending fragment |
| set_inventory_slot, take_stacks | helpers | F | pending fragment |
| send_particles_to, play_sound_to | helpers | F | pending fragment |
| client_particles (add_particle) | helper | F | pending fragment |
| Game::update_totem_ready, apply_totem, find_totem | injection | X | totem.rs:53, :67, :166 |
| hand light in the first person pass | injection | X | frame_geometry.rs:469 |
| ItemStack foil while in use | injection | X | horizon-assets/src/item_model.rs:1216 |
| dropper insert | injection | X | machines.rs:1766 |

## Step log
1. Checked W, branch and H commit: all as the brief says.
2. Read the brief and the whole audit.
3. Checked audit 6's V0 citations at integrated.rs:823, :880, :977 and lan_actions.rs:472, :483, :711: all match.
4. Read `guest_die` (lan_actions.rs:4471-4500) and its callers; found it runs with the hub as a parameter and no guest swapped in.
5. Read `native_take_items`, `native_give_stack`, `native_give_levels`, `native_as_player`, `native_run_command` (native_server.rs): all need a swapped-in player or `self.lan`.
6. Read totem.rs, health.rs:960-990, mobs.rs:1439 and :4350-4385 for the totem pipeline and mobs.
7. Read commands/effects.rs for `/effect` on guests; read native_lifecycle.rs for `server_tick`.
8. Read item_use.rs, using_item.rs, remote.rs:8895-9290, game.rs:5204 for the ritual hooks and the cooldown.
9. Read the pending fragments by `doc_summary`: anvil_update, player_effects, set_inventory_slot, inventory_click, inventory_drag, server_player_state, send_particles_to, client_particles, container_accepts, menu_input and others.
10. Compared the main and Nightly catalogs: only force_chunk helpers and world_hold, world_warp are main-only.
11. Read lan_stations.rs:100-215, native_menus.rs:230-244, machines.rs:1280-1300, :1720-1775 (anvil and container paths).
12. Read frame_geometry.rs:380-480 and item_model.rs foil code for the look rows.
13. Wrote this file and checked it for em-dashes and en-dashes.
