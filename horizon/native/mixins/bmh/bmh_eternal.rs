// Area eternal: the edge between the game and `bmh_logic::eternal`.
// It holds the Eternal Items kept through death (C11), the rewards of a night of sleep (C13), the
// hidden coordinates of the debug screen (C14), the Eternal Items on the ground (E10), the rule
// "one Eternal Item of each type" (E11) and the Eternal Bundle (E12). Every decision is in
// `bmh_logic::eternal`; this file reads the game and carries out the answers.
// Owner: the eternal agent. Log: docs/horizon/log/eternal.md.
//
// The flow, on the server:
// - C11. Before vanilla drops the inventory, the mod counts and wears the Eternal Items.
//   When the inventory drops, the items wait in the compound that the mod keeps on the
//   player: that is the one source of truth, so a restart while the player is dead does not lose
//   them and a stack never leaves the world where the player died. `SERVER.kept` holds a list
//   only when the game refused the compound, and the area forgets it when the world changes. The
//   `player_respawned` hook gives the stacks back with `give_stack`. It clears the compound only
//   with the stacks that did not fit and could not drop.
// - C14. `server_tick` (end) keeps the vanilla game rule `reduced_debug_info` on while a player is in
//   survival. The debug screen of Horizon hides the coordinates when it is on.
// - C13. `player_bed_leave` fires where the player is out of reach of the helpers (a guest waking
//   from a packet or a night skip), so it only queues the player. The next `server_tick` end gives
//   the reward.
// - E10. Ground damage hooks protect Eternal Items. The expiry hook renews their lifetime.
// - E11. `inventory_tick` drops a stack when an earlier slot holds the same Eternal Item.
//   `item_picked_up` refuses a second one.
// - E12. `inventory_tick` moves an Eternal Item or an Ender Chest out of an Eternal Bundle.
//   `use_item` drops all its contents at once.

use bmh_logic::contract::Rule;
use bmh_logic::eternal as logic;
use bmh_logic::ids;
use horizon_client::items::ItemStack;
use horizon_world::nbt::{Compound, JavaString, Tag};
use std::collections::BTreeSet;
use std::sync::{Mutex, MutexGuard};

/// The `minecraft:overworld` dimension id, where the game rules are read.
const OVERWORLD: &str = "minecraft:overworld";

/// What the area keeps between hooks. The kept stacks are also in the player's compound.
struct Server {
    /// The Eternal stacks that wait for a respawn, by player (KeepItemAfterDeath.java:24), only
    /// for a player whose compound the game refused. Tied to the world (see `Fallback`).
    kept: logic::Fallback<horizon_client::mod_api::NativeWorldEpoch, ItemStack>,
    /// The players who woke up since the last tick end and wait for a reward.
    woke: BTreeSet<u128>,
    /// The forcing of `reduced_debug_info` (DisableDebugInfo.java:25-26).
    debug: logic::DebugForce,
    /// The world that `debug` belongs to (DisableDebugInfo.java:37-38).
    debug_epoch: Option<horizon_client::mod_api::NativeWorldEpoch>,
    /// The last value that the area set, for a game that cannot read the rule.
    debug_set: Option<bool>,
    /// The area told that the rule command failed in this world.
    debug_warned: bool,
    rng: bmh_logic::respawn::Rng,
    seeded: bool,
}

static SERVER: Mutex<Server> = Mutex::new(Server {
    kept: logic::Fallback::new(),
    woke: BTreeSet::new(),
    debug: logic::DebugForce::new(),
    debug_epoch: None,
    debug_set: None,
    debug_warned: false,
    rng: bmh_logic::respawn::Rng::new(0x45_5445_524e_414c),
    seeded: false,
});

/// The server state. A lock that a panic left behind still holds valid data.
fn server() -> MutexGuard<'static, Server> {
    SERVER.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The next random value in [0, 1). The source mixes in the clock once.
fn random() -> f64 {
    let mut s = server();
    if !s.seeded {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos() as u64);
        s.rng.mix(nanos);
        s.seeded = true;
    }
    s.rng.next_f64()
}

// ---------------------------------------------------------------------------------------------
// Helpers for the stacks of a player
// ---------------------------------------------------------------------------------------------

/// Drops `stack` from the acting player as vanilla `Player.drop` does: 0.3 below the eyes, a pickup
/// delay of 40 ticks, thrown along the view or in a random direction. An Eternal Item gets the
/// largest health in its data (E10). True when the item entity exists. The stack is not lost on a
/// failure: the caller keeps it.
pub(super) fn drop_stack(game: &mut crate::game::Game<'_>, stack: &ItemStack, randomly: bool) -> bool {
    let Some(me) = game.native_acting_player() else {
        return false;
    };
    drop_at_player(game, &me, stack, randomly)
}

pub(super) fn drop_at_player(
    game: &mut crate::game::Game<'_>,
    me: &horizon_client::mod_api::NativeServerPlayer,
    stack: &ItemStack,
    randomly: bool,
) -> bool {
    let at = logic::drop_position(me.position, f64::from(me.eye_height));
    let velocity = logic::drop_velocity(me.yaw, me.pitch, randomly, &mut random);
    let mut nbt = Compound::new();
    nbt.insert("Item".into(), horizon_client::save::item_to_nbt(stack));
    nbt.insert("PickupDelay".into(), Tag::Short(logic::DROP_PICKUP_DELAY));
    nbt.insert(
        "Motion".into(),
        Tag::List(velocity.iter().map(|v| Tag::Double(v.to_bits())).collect()),
    );
    game.native_summon(&me.dimension, "minecraft:item", at, nbt).is_some()
}

/// The ids of the stacks in the given slots of the acting player (only non-empty stacks).
fn items_in(game: &mut crate::game::Game<'_>, slots: Vec<i32>) -> Vec<std::sync::Arc<str>> {
    slots
        .into_iter()
        .filter_map(|slot| {
            game.native_carried_stack(slot)
                .filter(|stack| stack.count > 0)
                .map(|stack| stack.item.clone())
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// C11: Eternal Items kept through death
// ---------------------------------------------------------------------------------------------

/// Saves the kept stacks in the compound that the mod keeps on the player. An empty list removes
/// the key. False when the game refused.
fn write_persisted(game: &mut crate::game::Game<'_>, player: u128, stacks: &[ItemStack]) -> bool {
    let Some(mut data) = game.native_player_persistent_data(player) else {
        return false;
    };
    let Tag::Compound(map) = &mut data else {
        return false;
    };
    let key = JavaString::from(logic::KEPT_KEY);
    if stacks.is_empty() {
        map.remove(&key);
    } else {
        map.insert(key, Tag::List(stacks.iter().map(horizon_client::save::item_to_nbt).collect()));
    }
    game.native_set_player_persistent_data(player, data)
}

/// Reads the kept stacks from the compound of the player, and leaves them there. Empty when there
/// are none or the game does not answer.
fn read_persisted(game: &mut crate::game::Game<'_>, player: u128) -> Vec<ItemStack> {
    let Some(Tag::Compound(map)) = game.native_player_persistent_data(player) else {
        return Vec::new();
    };
    match map.get(&JavaString::from(logic::KEPT_KEY)) {
        Some(Tag::List(list)) => list.iter().filter_map(horizon_client::save::item_from_nbt).collect(),
        _ => Vec::new(),
    }
}

/// The stacks that wait for the player: the memory list when the game refused the compound, else
/// the compound.
fn waiting_for(game: &mut crate::game::Game<'_>, player: u128) -> Vec<ItemStack> {
    let in_memory = server().kept.get(player).cloned();
    in_memory.unwrap_or_else(|| read_persisted(game, player))
}

/// Replaces the list that waits for the player. The compound is the home of the list. The list
/// stays in memory only when the game refuses the compound, so no path loses a stack silently.
fn store_waiting(game: &mut crate::game::Game<'_>, player: u128, stacks: Vec<ItemStack>) {
    if write_persisted(game, player, &stacks) {
        server().kept.clear(player);
        return;
    }
    log::info!(
        "[BalancedMcHardcoreMain] The kept Eternal Items of {player:032x} wait in memory only"
    );
    server().kept.put(player, stacks);
}

/// Record the Eternal Items before vanilla checks the keep-inventory rule or the Curse of Vanishing.
pub(super) fn before_inventory_drops(game: &mut crate::game::Game<'_>, player: u128) {
    let wear = game.native_acting_player().is_some_and(|me| me.game_mode == logic::SURVIVAL)
        && crate::game::bmh_settings::rule(game, Rule::DamageEternalItemWhenDead);
    let mut kept = Vec::new();
    let mut count = 0;
    for slot in game.inventory.iter_mut().chain(game.armor.iter_mut().rev()).chain([&mut game.offhand]) {
        let Some(stack) = slot.as_mut() else { continue };
        if stack.count == 0 || !ids::is_eternal(&stack.item) {
            continue;
        }
        count += 1;
        if wear && bmh_logic::tools::tool_of(&stack.item).is_some() {
            let limit = crate::game::bmh_tools::max_damage(stack);
            stack.damage = bmh_logic::tools::death_wear(stack.damage, limit);
        }
        kept.extend(slot.take());
    }
    crate::game::bmh_respawn::record_death_inventory(player, count);
    if !kept.is_empty() {
        let before = waiting_for(game, player);
        store_waiting(game, player, logic::keep_after_death(before, kept));
    }
}

// C11: give back the Eternal Items kept at death (KeepItemAfterDeath.java:58-74). Java adds each
// stack with `Inventory.add`; `give_stack` places it the same way. A stack that does not fit is
// dropped at the feet of the player, so it is not lost (Java lost it). The list is cleared only
// after the gives: a stack that cannot drop waits again, in the compound.
pub(super) fn on_player_respawned(game: &mut crate::game::Game<'_>, player: u128, name: &str) {
    let _ = name;
    let stacks = waiting_for(game, player);
    if stacks.is_empty() {
        return;
    }
    let mut failed = Vec::new();
    for stack in stacks {
        let Some(left) = game.native_give_stack(player, stack) else {
            continue;
        };
        if !drop_stack(game, &left, false) {
            failed.push(left);
        }
    }
    if !failed.is_empty() {
        log::warn!(
            "[BalancedMcHardcoreMain] {} kept Eternal Item(s) of {player:032x} did not fit and could not drop",
            failed.len()
        );
    }
    store_waiting(game, player, failed);
}

// ---------------------------------------------------------------------------------------------
// C14 and C13: the end of the server tick
// ---------------------------------------------------------------------------------------------

/// C14 (DisableDebugInfo.java:29-60): keep `reduced_debug_info` on while the world rule is on and a
/// player is in survival. The rule is read with the pending helper `level_game_rule`; when it
/// answers nothing the area uses the last value that it set (GAPS.md C14).
fn hide_coordinates(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLifecycleContext,
) {
    let rule_on = crate::game::bmh_settings::rule(game, Rule::RemoveCoordinatesInSurvival);
    let any_survival =
        game.native_server_players().iter().any(|player| player.game_mode == logic::SURVIVAL);
    let read = game
        .native_level(OVERWORLD)
        .and_then(|level| game.native_level_game_rule(level, logic::REDUCED_DEBUG_INFO))
        .and_then(|value| match value {
            horizon_client::mod_api::GameRuleValue::Bool(on) => Some(on),
            horizon_client::mod_api::GameRuleValue::Int(_) => None,
        });
    let action = {
        let mut s = server();
        if s.debug_epoch != Some(context.epoch) {
            s.debug.forget();
            s.debug_set = None;
            s.debug_warned = false;
            s.debug_epoch = Some(context.epoch);
        }
        let current = logic::current_rule(read, s.debug_set);
        s.debug.step(rule_on, any_survival, current)
    };
    let logic::DebugAction::Set(value) = action else {
        return;
    };
    if game.native_run_command(&logic::rule_command(value)) {
        server().debug_set = Some(value);
    } else if !std::mem::replace(&mut server().debug_warned, true) {
        log::warn!(
            "[BalancedMcHardcoreMain] The command \"{}\" did not run, the coordinates stay as they are",
            logic::rule_command(value)
        );
    }
}

/// C13 (SleepHandler.java:35-77): the reward of each player who woke up since the last tick end.
/// The helpers reach a player outside a player's turn, so this runs in `server_tick`.
fn wake_rewards(game: &mut crate::game::Game<'_>) {
    let woke = std::mem::take(&mut server().woke);
    if woke.is_empty() {
        return;
    }
    let online = game.native_server_players();
    for uuid in woke {
        let Some(me) = online.iter().find(|player| player.uuid == uuid) else {
            continue;
        };
        let Some(food) = game.native_player_food(uuid) else {
            continue;
        };
        let Some(reward) = logic::sleep_reward(food.0, me.health) else {
            continue;
        };
        let (health, new_food) = logic::after_reward(&reward, me.health, food);
        if !game.native_set_player_status(uuid, Some(health), Some(new_food), None) {
            log::info!("[BalancedMcHardcoreMain] The wake-up reward of {uuid:032x} was not applied");
            continue;
        }
        if reward.effects {
            for id in logic::WAKE_EFFECTS {
                // Ambient false, particles false; the icon shows (SleepHandler.java:50-51).
                game.native_add_effect(
                    uuid,
                    &horizon_client::mod_api::NativeEffect {
                        id: id.to_owned(),
                        amplifier: logic::WAKE_EFFECT_AMPLIFIER,
                        duration: logic::WAKE_EFFECT_TICKS,
                        ambient: false,
                        particles: false,
                    },
                );
            }
        }
    }
}

// C14: force `reduced_debug_info` in survival. C13: give the wake-up rewards.
pub(super) fn on_server_tick(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLifecycleContext,
    phase: horizon_client::mod_api::ServerTickPhase,
) {
    let horizon_client::mod_api::ServerTickPhase::ServerTickEnd { .. } = phase else {
        return;
    };
    // A list in memory belongs to the world where the player died (the owner of a single-player
    // world has the same uuid in every world).
    server().kept.new_world(context.epoch);
    hide_coordinates(game, context);
    wake_rewards(game);
}

// C13: the wake-up rewards. The hook fires with the player out of reach of the helpers (a guest
// waking from a packet or from the morning skip: `wake_guest_from`), so the player is only queued.
// The rule is read now, as Java reads it at the wake-up (SleepHandler.java:25-30).
pub(super) fn on_player_bed_leave(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    pos: [i32; 3],
) {
    let _ = (name, pos);
    if crate::game::bmh_settings::rule(game, Rule::FoodAndHealthRegenOnWakeUp) {
        server().woke.insert(player);
    }
}

// ---------------------------------------------------------------------------------------------
// E11 and E12: the carried stacks
// ---------------------------------------------------------------------------------------------

/// E11 (EternalItemHandler.java:24-57): this stack is a copy when an earlier slot holds the same
/// Eternal Item. The copy drops in front of the player and the stack loses one. The slot is
/// changed only when the item entity exists, so a failed drop loses nothing.
fn drop_copy(game: &mut crate::game::Game<'_>, slot: i32, item: &str) {
    let Some(me) = game.native_acting_player() else {
        return;
    };
    let rule_on = crate::game::bmh_settings::rule(game, Rule::OnlyCarryOneEternalItemType);
    if !logic::rule_applies(rule_on, me.game_mode) {
        return;
    }
    let earlier = items_in(game, logic::earlier_slots(slot));
    if !logic::is_copy(item, earlier.iter().map(|id| &**id)) {
        return;
    }
    let Some(stack) = game.native_carried_stack(slot).map(|stack| stack.clone()) else {
        return;
    };
    let (_, left) = logic::copy_drop(stack.count);
    if !drop_stack(game, &stack, false) {
        return;
    }
    if let Some(stack) = game.native_carried_stack(slot) {
        stack.count = left;
    }
}

/// E12 (EternalBundleItem.java:177-183): moves what the Eternal Bundle in `slot` must refuse (an
/// Eternal Item, an Ender Chest) out of it. The player gets it back in the inventory, else on the
/// ground. When both fail the stack goes back inside, so it is not lost.
fn refuse_in_bundle(game: &mut crate::game::Game<'_>, slot: i32) {
    let refused: Vec<ItemStack> = {
        let Some(bundle) = game.native_carried_stack(slot).filter(|stack| stack.count > 0) else {
            return;
        };
        let any = bundle
            .data()
            .is_some_and(|data| data.bundle_contents.iter().any(|s| logic::bundle_refuses(&s.item)));
        if !any {
            return;
        }
        let data = bundle.data_mut();
        let (refused, kept): (Vec<ItemStack>, Vec<ItemStack>) =
            std::mem::take(&mut data.bundle_contents)
                .into_iter()
                .partition(|s| logic::bundle_refuses(&s.item));
        data.bundle_contents = kept;
        data.bundle_selected = Default::default();
        bundle.tidy();
        refused
    };
    let Some(uuid) = game.native_acting_player().map(|me| me.uuid) else {
        put_back_in_bundle(game, slot, refused);
        return;
    };
    let mut stuck = Vec::new();
    for stack in refused {
        let Some(left) = game.native_give_stack(uuid, stack) else {
            continue;
        };
        if !drop_stack(game, &left, false) {
            stuck.push(left);
        }
    }
    put_back_in_bundle(game, slot, stuck);
}

/// Puts stacks back into the Eternal Bundle in `slot` (a refusal that could not be carried out).
fn put_back_in_bundle(game: &mut crate::game::Game<'_>, slot: i32, stacks: Vec<ItemStack>) {
    if stacks.is_empty() {
        return;
    }
    if let Some(bundle) = game.native_carried_stack(slot) {
        bundle.data_mut().bundle_contents.extend(stacks);
    }
}

// E11: drop a second Eternal Item of a type. E12: move a refused stack out of the Eternal Bundle.
// Eternal can set the count to 0 first; the slot empties at the end of the hook, so the bundle
// test reads the stack again.
pub(super) fn on_inventory_tick(game: &mut crate::game::Game<'_>, slot: i32, selected: bool) {
    let _ = selected;
    let Some(item) = game
        .native_carried_stack(slot)
        .filter(|stack| stack.count > 0)
        .map(|stack| stack.item.clone())
    else {
        return;
    };
    if !ids::is_eternal(&item) {
        return;
    }
    drop_copy(game, slot, &item);
    if logic::is_bundle(&item) {
        refuse_in_bundle(game, slot);
    }
}

// E11: refuse a second Eternal Item of a type (EternalItemHandler.java:60-77). The Totem of
// Eternity is refused always. The Totem area may refuse it too: the answer is the same.
pub(super) fn on_item_picked_up(
    game: &mut crate::game::Game<'_>,
    player: u128,
    item: &str,
    count: u32,
    canceled: &mut bool,
) {
    let _ = (player, count);
    if *canceled || !ids::is_eternal(item) {
        return;
    }
    let rule_on = crate::game::bmh_settings::rule(game, Rule::OnlyCarryOneEternalItemType);
    let mode = game.native_acting_player().map(|me| me.game_mode);
    let carried = items_in(game, logic::slot_order());
    if logic::blocks_pickup(item, rule_on, mode, carried.iter().map(|id| &**id)) {
        *canceled = true;
    }
}

// ---------------------------------------------------------------------------------------------
// E10 and E12: item entities and the use of the bundle
// ---------------------------------------------------------------------------------------------

// E10: an Eternal Item on the ground never despawns (G_ItemEntityMixin.java:45-53).
pub(super) fn on_item_expire(
    game: &mut crate::game::Game<'_>,
    entity: u32,
    stack: &horizon_client::items::ItemStack,
    extra_life: &mut i32,
) {
    let _ = (game, entity);
    *extra_life = logic::extra_life(&stack.item, *extra_life);
}

// E10: give an Eternal Item on the ground its health (G_ItemEntityMixin.java:26-40). The fire
// E12: the Eternal Bundle drops all its items at once (EternalBundleItem.java:141-158, 196-210).
// The hook claims the click, so the template's own use (one item at a time) does not start. Java
// answers FAIL for an empty bundle: the hook has no FAIL. The host's template use of a bundle
// ends in `start_using_item`, `swing_arm` and `Ok(true)` for every bundle, empty or not
// (HN game.rs:7321-7325), and a claim ends in `swing_arm` and `Ok(true)` (HN game.rs:7210-7213).
// So the other hand never gets its turn after a bundle, and the arm swings. The area picks the
// hand with a bundle that holds something, as Java's FAIL then off hand would. When no hand has
// one, it claims the click, so the template does not raise the hand for nothing.
pub(super) fn on_use_item(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    claimed: &mut bool,
) {
    if !logic::is_bundle(&stack.item) {
        return;
    }
    let Some(me) = game.native_acting_player() else {
        return;
    };
    // Minecraft tries the main hand first, and this hook claims its click.
    let main = i32::from(me.selected_slot);
    let loaded = logic::first_loaded(
        [main, -106].into_iter().map(|slot| {
            let loaded = game.native_carried_stack(slot).is_some_and(|held| {
                logic::is_bundle(&held.item)
                    && held.count > 0
                    && held.data().is_some_and(|data| !data.bundle_contents.is_empty())
            });
            (slot, loaded)
        }),
    );
    *claimed = true;
    let Some(slot) = loaded else {
        return;
    };
    let Some(held) = game.native_carried_stack(slot) else {
        return;
    };
    let spilled = horizon_client::bundle::destroyed(held);
    if spilled.is_empty() {
        return;
    }
    let total = spilled.len();
    let mut stuck = Vec::new();
    for dropped in spilled {
        if !drop_stack(game, &dropped, true) {
            stuck.push(dropped);
        }
    }
    let emptied = stuck.len() < total;
    put_back_in_bundle(game, slot, stuck);
    if !emptied {
        return;
    }
    game.award_stat("used", ids::ETERNAL_BUNDLE, 1);
    let pitch = logic::bundle_sound_pitch(random());
    game.native_play_sound_in_place(
        logic::BUNDLE_DROP_SOUND,
        me.position,
        logic::PLAYER_SOUND_SOURCE,
        logic::BUNDLE_SOUND_VOLUME,
        pitch,
    );
}

pub(super) fn on_tooltip(
    game: &crate::game::Game<'_>,
    stack: &ItemStack,
    lines: &mut Vec<(horizon_client::text::Text, u32)>,
) {
    use horizon_client::text::{self, Text};
    if !logic::is_bundle(&stack.item) {
        return;
    }
    let Some(contents) = horizon_client::bundle::Contents::of(game.items(), stack) else {
        return;
    };
    let used = (contents.weight_fraction() * 64.0) as u32;
    let fullness = Text::translate("item.minecraft.bundle.fullness", &[
        Text::literal(used.to_string()), Text::literal("64"),
    ]).with_color(text::GRAY);
    let heavy = Text::translate(bmh_logic::tools::KEY_HEAVY, &[]).with_color(bmh_logic::tools::GOLD);
    let at = lines.len().min(1);
    lines.splice(at..at, [(fullness, u32::MAX), (heavy, u32::MAX)]);
}

pub(super) fn on_item_bar(
    _game: &crate::game::Game<'_>,
    stack: &ItemStack,
    bar: &mut Option<(i32, u32)>,
) {
    if logic::is_bundle(&stack.item) && let Some((_, color)) = bar {
        *color = 0x6666FF;
    }
}
