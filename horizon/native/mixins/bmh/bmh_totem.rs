// Area totem: the edge between the game and `bmh_logic::totem`.
// It holds the save from death (T2, T3, T4), the tooltip lines of the four totem items (T1), the
// rule "one Totem of Eternity per player and per container" (T11, T12, T13) and the anvil (T6
// awakening, E6 gold block repair). T5 (the Totem of Undying without Fire Resistance) is in
// `bmh_totem_world.rs`. Every decision is in `bmh_logic::totem`; this file reads the game and
// carries out the answers.
// Owner: the totem agent. Log: docs/horizon/log/totem.md.
//
// The flow of a save, on the server (GAPS T2, T3; verify-2 row 1a):
// - `living_death` runs with no player swapped in: no helper can read a stack there. It asks the
//   book of saves (`logic::SaveBook`), which knows from the last player tick whether the player
//   carries a Totem of Eternity. If so, the hook cancels the death (Horizon keeps the player at
//   1 health) and the book remembers the wait.
// - The next `player_tick` has the player swapped in. It turns the first Totem of Eternity into a
//   Dormant Totem, sets the health to 1, replaces the effects with the three of the save, plays the
//   sound and sends the particles (T4).
// - The advancement `eternity/saved` is awarded at the end of the next server tick, because
//   `run_command` works only where no player is swapped in.
//
// One Totem of Eternity per player (T11) and per container (T12, T13):
// - `player_tick` drops every Totem of Eternity after the first, whatever the game rule says.
// - A container block is checked after the fact, while the player has its menu open and when he
//   closes it. The Java mod refuses the placement (a slot rule); no hook can do that.
// - A hopper move into a container that holds a totem is canceled. A hopper never picks up a
//   Totem of Eternity from the ground: that hook has no `Game`, so it cannot read the hopper.

use bmh_logic::eternal as eternal_logic;
use bmh_logic::ids;
use bmh_logic::totem as logic;
use horizon_client::containers::BlockEntity;
use horizon_client::items::ItemStack;
use horizon_client::lan::menu::{Slots, VEHICLE_KEY_Y};
use horizon_client::mod_api::{NativeEffect, NativeServerPlayer};
use horizon_client::particles::{ParticleOptions, ParticleType};
use horizon_client::text::Text;
use horizon_core::geometry::Direction;
use horizon_world::nbt::{Compound, Tag};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::Instant;

/// The game mode number of a spectator (`NativeServerPlayer.game_mode`: 3 is spectator).
/// TotemOfEternityHandler.java:62 skips a spectator.
const SPECTATOR: i32 = 3;

/// The name of the player entity in `living_death`.
const PLAYER_KIND: &str = "minecraft:player";

/// The container that a player has open: where it is, and which slots held a totem when he opened it.
struct OpenMenu {
    dimension: String,
    pos: [i32; 3],
    before: Vec<usize>,
}

/// What a hand save leaves to do at the end of the server tick (route A, GAPS T2).
enum HandWork {
    /// The game spent the Totem of Eternity of `slot` (`NativeInventoryKind::Player` numbering): put
    /// the Dormant Totem there.
    Dormant { slot: usize },
    /// The hook swapped the hands so the Totem of Undying saved first: put `totem` back from the off
    /// hand into the main hand `slot`.
    SwapBack { slot: usize, totem: ItemStack },
}

/// One hand save that waits for the end of the server tick.
struct HandSaveRecord {
    uuid: u128,
    name: String,
    work: HandWork,
}

/// What the area keeps between hooks (all on the server).
struct Server {
    /// Who carries a totem, and which saves wait for their swap.
    book: logic::SaveBook,
    /// The container each player has open.
    open: BTreeMap<u128, OpenMenu>,
    /// The names of the players who were saved and wait for the advancement.
    awards: Vec<String>,
    /// The hand saves of this tick, finished at its end.
    hand: Vec<HandSaveRecord>,
    /// The world that the state above belongs to.
    epoch: Option<horizon_client::mod_api::NativeWorldEpoch>,
    /// The scan of containers (T13, T14): the totems each held at the last scan, by level and
    /// position. A double chest is under both halves.
    scan_blocks: BTreeMap<(String, [i32; 3]), usize>,
    /// The same for chest minecarts, hopper minecarts and chest boats, by entity id.
    scan_vehicles: BTreeMap<u32, usize>,
    /// False until the first scan has set the counts: that scan takes nothing out.
    scan_ready: bool,
    /// Server ticks since the last scan.
    scan_ticks: u32,
    rng: bmh_logic::respawn::Rng,
    seeded: bool,
}

static SERVER: Mutex<Server> = Mutex::new(Server {
    book: logic::SaveBook::new(),
    open: BTreeMap::new(),
    awards: Vec::new(),
    hand: Vec::new(),
    epoch: None,
    scan_blocks: BTreeMap::new(),
    scan_vehicles: BTreeMap::new(),
    scan_ready: false,
    scan_ticks: 0,
    rng: bmh_logic::respawn::Rng::new(0x54_4F54_454D),
    seeded: false,
});

/// The hook without a `Game` (`hopper_pickup_item`) acts only inside a server tick that the gated
/// `server_tick` hook opened (HOOKS.md, "Hooks with no Game"). This holds the start time of that
/// tick in milliseconds after `CLOCK_BASE`, plus 1, and 0 when no such tick is open. So a world with
/// the mod off never gets the rule, also after a world with the mod on in the same process.
static ACTIVE_TICK: AtomicU64 = AtomicU64::new(0);

/// The start of the clock of `ACTIVE_TICK`.
static CLOCK_BASE: OnceLock<Instant> = OnceLock::new();

/// A tick mark older than this does not count. A player can turn the mod off between the start and
/// the end of a tick: the end then does not clear the mark, and this limit ends it.
const ACTIVE_TICK_MAX_AGE_MS: u64 = 60_000;

/// The milliseconds since `CLOCK_BASE`, plus 1 (never 0).
fn clock_mark() -> u64 {
    let base = *CLOCK_BASE.get_or_init(Instant::now);
    u64::try_from(base.elapsed().as_millis()).unwrap_or(u64::MAX - 1).saturating_add(1)
}

/// True inside a server tick that the gated `server_tick` hook opened.
fn active() -> bool {
    let mark = ACTIVE_TICK.load(Ordering::Relaxed);
    mark != 0 && clock_mark().saturating_sub(mark) < ACTIVE_TICK_MAX_AGE_MS
}

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
// Helpers for the stacks of the acting player
// ---------------------------------------------------------------------------------------------

/// The non-empty stacks the acting player carries, with their slot, in the order of
/// `eternal::slot_order` (the 36 main slots, the armour, the off hand).
fn carried_items(game: &mut crate::game::Game<'_>) -> Vec<(i32, Arc<str>)> {
    eternal_logic::slot_order()
        .into_iter()
        .filter_map(|slot| {
            game.native_carried_stack(slot)
                .filter(|stack| stack.count > 0)
                .map(|stack| (slot, stack.item.clone()))
        })
        .collect()
}

/// The slots of the acting player that hold a Totem of Eternity.
fn totem_slots(game: &mut crate::game::Game<'_>) -> Vec<i32> {
    let carried = carried_items(game);
    logic::totem_slots(carried.iter().map(|(slot, item)| (*slot, &**item)))
}

/// Drops `stack` from the acting player as vanilla `Player.drop(stack, false)` does: 0.3 below the
/// eyes, a pickup delay of 40 ticks, thrown along the view with a small spread. An Eternal Item gets
/// the largest health in its data, so a blast does not kill it (E10). True when the item entity
/// exists. The stack is not lost on a failure: the caller keeps it.
/// (The eternal area has the same function. It is private there and the area files are separate.)
fn drop_stack(game: &mut crate::game::Game<'_>, stack: &ItemStack) -> bool {
    let Some(me) = game.native_acting_player() else {
        return false;
    };
    let at = eternal_logic::drop_position(me.position, f64::from(me.eye_height));
    let velocity = eternal_logic::drop_velocity(me.yaw, me.pitch, false, &mut random);
    let mut nbt = Compound::new();
    nbt.insert("Item".into(), horizon_client::save::item_to_nbt(stack));
    nbt.insert("PickupDelay".into(), Tag::Short(eternal_logic::DROP_PICKUP_DELAY));
    nbt.insert(
        "Motion".into(),
        Tag::List(velocity.iter().map(|v| Tag::Double(v.to_bits())).collect()),
    );
    if let Some(health) = eternal_logic::item_health(&stack.item) {
        nbt.insert("Health".into(), Tag::Short(health));
    }
    game.native_summon(&me.dimension, "minecraft:item", at, nbt).is_some()
}

/// Takes the Totem of Eternity out of `slot` of the acting player, and out of no other slot
/// (TotemOfEternityHandler.java:80 empties exactly that slot). `take_items` takes from the main hand
/// first, so it empties `slot` only when `slot` is the selected hotbar slot and holds a plain stack
/// (no damage, no data). Any other stack gets a count of 0: the game empties such a slot at the next
/// inventory tick. False when nothing changed.
fn remove_totem(game: &mut crate::game::Game<'_>, slot: i32, selected: i32) -> bool {
    let plain = game
        .native_carried_stack(slot)
        .is_some_and(|stack| stack.damage == 0 && stack.extra.is_none());
    if plain && slot == selected && game.native_take_items(ids::TOTEM_OF_ETERNITY, 1) == 1 {
        return true;
    }
    match game.native_carried_stack(slot) {
        Some(stack) => {
            stack.count = 0;
            true
        }
        None => false,
    }
}

// ---------------------------------------------------------------------------------------------
// T2, T3, T4: the save from death
// ---------------------------------------------------------------------------------------------

// T2: a Totem of Eternity in the inventory cancels the death and records the entity for the swap
// (A_LivingEntityTotemMixin.java:40-46). The Totem of Undying in a hand saved the player before this
// hook: Horizon never calls `living_death` for that death. The hook has no player swapped in.
pub(super) fn on_living_death(
    game: &mut crate::game::Game<'_>,
    entity: u32,
    kind: &str,
    damage_type: &str,
    attacker: u32,
    cancel: &mut bool,
) {
    let _ = (game, attacker);
    if *cancel || kind != PLAYER_KIND {
        return;
    }
    if server().book.on_death(entity, damage_type) {
        *cancel = true;
    }
}

/// The sound and the particles of a save (GAPS T4: no item overlay without an injection).
fn announce_save(game: &mut crate::game::Game<'_>, me: &NativeServerPlayer) {
    game.native_play_sound_in_place(
        logic::SAVE_SOUND,
        me.position,
        eternal_logic::PLAYER_SOUND_SOURCE,
        logic::SAVE_SOUND_VOLUME,
        logic::SAVE_SOUND_PITCH,
    );
    let at = [me.position[0], me.position[1] + logic::SAVE_PARTICLE_HEIGHT, me.position[2]];
    game.native_send_particles_in_place(
        &ParticleOptions::simple(ParticleType::TotemOfUndying),
        None,
        at,
        logic::SAVE_PARTICLE_SPREAD,
        logic::SAVE_PARTICLE_SPEED,
        logic::SAVE_PARTICLES,
    );
}

/// The swap of a save (TotemOfEternityHandler.java:27-52), in the tick after the death hook. The
/// first Totem of Eternity becomes a Dormant Totem in the same slot, the health is 1, the effects
/// are replaced by the three of the save, and the advancement waits for the end of the tick.
fn complete_save(game: &mut crate::game::Game<'_>, me: &NativeServerPlayer) {
    let carried = carried_items(game);
    let first = logic::first_totem_slot(carried.iter().map(|(slot, item)| (*slot, &**item)));
    let Some(slot) = first else {
        // The totem left the inventory between the death and this tick (a drop, a swap). The save
        // did not spend a totem, so it does not stand: the death comes back.
        log::warn!(
            "[BalancedMcHardcoreMain] {} had no Totem of Eternity left to spend: the death stands",
            me.name
        );
        if !game.native_set_player_status(me.uuid, Some(0.0), None, None) {
            log::warn!(
                "[BalancedMcHardcoreMain] The game refused to set the health of {} to 0",
                me.name
            );
        }
        return;
    };
    if let Some(stack) = game.native_carried_stack(slot) {
        *stack = ItemStack::new(ids::SLEEPING_TOTEM, 1);
    }
    // The game refuses both helpers for a body that it still marks dead after the canceled death
    // (log, fix round 1). The warnings make such a refused save visible.
    if !game.native_set_player_status(me.uuid, Some(logic::SAVE_HEALTH), None, None) {
        log::warn!(
            "[BalancedMcHardcoreMain] The game refused the save health of {}",
            me.name
        );
    }
    let effects: Vec<NativeEffect> = logic::SAVE_EFFECTS
        .iter()
        .map(|effect| NativeEffect {
            id: effect.id.to_owned(),
            amplifier: effect.amplifier,
            duration: effect.duration,
            ambient: false,
            particles: true,
        })
        .collect();
    if !game.native_set_effects(me.uuid, &effects) {
        log::warn!(
            "[BalancedMcHardcoreMain] The game refused the save effects of {}",
            me.name
        );
    }
    announce_save(game, me);
    server().awards.push(me.name.clone());
    log::info!(
        "[BalancedMcHardcoreMain] The Totem of Eternity saved {} and is now dormant",
        me.name
    );
}

/// The item id of a non-empty stack in `slot` of the acting player.
fn carried_item(game: &mut crate::game::Game<'_>, slot: i32) -> Option<Arc<str>> {
    game.native_carried_stack(slot)
        .filter(|stack| stack.count > 0)
        .map(|stack| stack.item.clone())
}

// T2, route A (docs/horizon/solutions/totem-save.md): Horizon's own hand check saves the player
// with the `death_protection` of the Totem of Eternity (content item). This hook runs before the
// game spends the hand. The player is swapped in, at 0 health. The slot is not written here: the
// use-up reads the hands after this hook.
pub(super) fn on_player_totem(game: &mut crate::game::Game<'_>, player: u128, name: &str) {
    let Some(me) = game.native_acting_player() else {
        return;
    };
    if me.uuid != player {
        return;
    }
    let main_slot = i32::from(me.selected_slot);
    let main = carried_item(game, main_slot);
    let off = carried_item(game, logic::CARRIED_OFF_HAND);
    let work = match logic::hand_save(main.as_deref(), off.as_deref(), me.selected_slot) {
        logic::HandSave::Nothing => return,
        logic::HandSave::Swap => {
            let totem = game.native_carried_stack(main_slot).map(|stack| stack.clone());
            let undying = game.native_carried_stack(logic::CARRIED_OFF_HAND).map(|stack| stack.clone());
            let (Some(totem), Some(undying)) = (totem, undying) else {
                return;
            };
            if let Some(stack) = game.native_carried_stack(main_slot) {
                *stack = undying;
            }
            if let Some(stack) = game.native_carried_stack(logic::CARRIED_OFF_HAND) {
                *stack = totem.clone();
            }
            HandWork::SwapBack { slot: usize::from(me.selected_slot), totem }
        }
        logic::HandSave::Eternity { inventory, .. } => HandWork::Dormant { slot: inventory },
    };
    server().hand.push(HandSaveRecord { uuid: player, name: name.to_owned(), work });
}

/// Puts the Dormant Totem in `slot` of `uuid`, which the game emptied. Another item there: the
/// inventory gets it. No room: a warning (the save already happened).
fn place_dormant(game: &mut crate::game::Game<'_>, uuid: u128, name: &str, slot: usize) {
    let dormant = ItemStack::new(ids::SLEEPING_TOTEM, 1);
    let kind = crate::game::NativeInventoryKind::Player;
    if game.native_set_inventory_slot(uuid, kind, slot, None, Some(dormant.clone())).is_ok() {
        return;
    }
    if game.native_give_stack(uuid, dormant).is_some() {
        log::warn!("[BalancedMcHardcoreMain] No room for the Dormant Totem of {name}");
    }
}

/// Puts the Totem of Eternity back from the off hand into the main hand `slot` after a swap save.
/// It moves only when the off hand still holds it and the main slot is empty.
fn swap_back(game: &mut crate::game::Game<'_>, uuid: u128, name: &str, slot: usize, totem: ItemStack) {
    let kind = crate::game::NativeInventoryKind::Player;
    if game.native_set_inventory_slot(uuid, kind, logic::INVENTORY_OFF_HAND, Some(&totem), None).is_err() {
        return;
    }
    if game.native_set_inventory_slot(uuid, kind, slot, None, Some(totem.clone())).is_ok() {
        return;
    }
    let back = game.native_set_inventory_slot(uuid, kind, logic::INVENTORY_OFF_HAND, None, Some(totem.clone()));
    if back.is_err() && game.native_give_stack(uuid, totem).is_some() {
        log::warn!("[BalancedMcHardcoreMain] No room for the Totem of Eternity of {name}");
    }
}

// ---------------------------------------------------------------------------------------------
// T11, T12: one Totem of Eternity per player and per container
// ---------------------------------------------------------------------------------------------

/// T11 (TotemOfEternityHandler.java:56-76, `dropExtraTotems`): every Totem of Eternity after the
/// first drops in front of the player and leaves his slot. Not for a spectator. The slot changes
/// only when the item entity exists, and the drop comes first, so a failed drop loses nothing.
fn drop_extra_totems(game: &mut crate::game::Game<'_>, me: &NativeServerPlayer) {
    if me.game_mode == SPECTATOR {
        return;
    }
    let slots = totem_slots(game);
    for &slot in logic::extra_slots(&slots) {
        let Some(stack) = game
            .native_carried_stack(slot)
            .filter(|stack| stack.count > 0 && logic::is_totem(&stack.item))
            .map(|stack| stack.clone())
        else {
            continue;
        };
        if drop_stack(game, &stack) {
            remove_totem(game, slot, i32::from(me.selected_slot));
        }
    }
}

/// The slots of the container at `pos` that hold a Totem of Eternity, in slot order. None where no
/// container is (another level, a chunk that is not loaded, a block without slots). A double
/// chest answers with both halves. Read through the top side, which reaches every slot of a chest,
/// a barrel, a shulker box and a hopper.
fn container_totems(game: &crate::game::Game<'_>, dimension: &str, pos: [i32; 3]) -> Option<Vec<usize>> {
    let slots = game.native_container_slots(dimension, pos, Direction::Up)?;
    let mut found: Vec<usize> = slots
        .iter()
        .filter(|slot| {
            slot.stack.as_ref().is_some_and(|stack| stack.count > 0 && logic::is_totem(&stack.item))
        })
        .map(|slot| slot.slot)
        .collect();
    found.sort_unstable();
    Some(found)
}

/// T12: takes the second Totem of Eternity out of the container that `uuid` has open, and gives it
/// to him. The inventory rule of T11 then drops it if he already carries one. Runs in his turn
/// (`player_tick`, `player_container_close`): the helpers reach him there.
fn scan_open_container(game: &mut crate::game::Game<'_>, uuid: u128) {
    let open = server().open.get(&uuid).map(|menu| (menu.dimension.clone(), menu.pos, menu.before.clone()));
    let Some((dimension, pos, before)) = open else {
        return;
    };
    if !game.native_acting_player().is_some_and(|me| me.uuid == uuid) {
        return;
    }
    let Some(now) = container_totems(game, &dimension, pos) else {
        return;
    };
    for slot in logic::container_extras(&before, &now) {
        let Some(stack) = game.native_extract_item(&dimension, pos, Direction::Up, slot, 1) else {
            continue;
        };
        let Some(left) = game.native_give_stack(uuid, stack) else {
            continue;
        };
        if !drop_stack(game, &left) {
            log::warn!(
                "[BalancedMcHardcoreMain] A second Totem of Eternity left a container and could not be given or dropped"
            );
        }
    }
}

// T2, T11, T12: the swap of a save, the extra totems of the inventory and of the open container.
// Then the note for the next death: the entity of the player and whether he carries a totem.
pub(super) fn on_player_tick(game: &mut crate::game::Game<'_>) {
    let Some(me) = game.native_acting_player() else {
        return;
    };
    if server().book.take_pending(me.uuid) {
        complete_save(game, &me);
    }
    scan_open_container(game, me.uuid);
    drop_extra_totems(game, &me);
    let has_totem = !totem_slots(game).is_empty();
    server().book.note(me.uuid, me.entity_id, has_totem);
}

// State: remove this player from the area's per-player statics.
pub(super) fn on_player_logged_out(game: &mut crate::game::Game<'_>, player: u128, name: &str) {
    let _ = (game, name);
    let mut s = server();
    s.book.forget(player);
    s.open.remove(&player);
}

// T13: the start of a tick opens the window of the hopper rule, and the end closes it.
// T3: the advancement of a save. A state tied to an old world is forgotten (the owner of a
// single-player world has the same uuid in every world).
pub(super) fn on_server_tick(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLifecycleContext,
    phase: horizon_client::mod_api::ServerTickPhase,
) {
    if let horizon_client::mod_api::ServerTickPhase::ServerTickStart { .. } = phase {
        ACTIVE_TICK.store(clock_mark(), Ordering::Relaxed);
        return;
    }
    let horizon_client::mod_api::ServerTickPhase::ServerTickEnd { .. } = phase else {
        return;
    };
    ACTIVE_TICK.store(0, Ordering::Relaxed);
    let (mut names, hand) = {
        let mut s = server();
        if s.epoch != Some(context.epoch) {
            s.book.clear();
            s.open.clear();
            s.awards.clear();
            s.hand.clear();
            s.scan_blocks.clear();
            s.scan_vehicles.clear();
            s.scan_ready = false;
            s.scan_ticks = 0;
            s.epoch = Some(context.epoch);
        }
        (std::mem::take(&mut s.awards), std::mem::take(&mut s.hand))
    };
    // T2, route A: no player is swapped in here, so `native_set_inventory_slot` reaches each one.
    for record in hand {
        match record.work {
            HandWork::Dormant { slot } => {
                place_dormant(game, record.uuid, &record.name, slot);
                log::info!(
                    "[BalancedMcHardcoreMain] The Totem of Eternity saved {} and is now dormant",
                    record.name
                );
                names.push(record.name);
            }
            HandWork::SwapBack { slot, totem } => {
                swap_back(game, record.uuid, &record.name, slot, totem);
            }
        }
    }
    scan_all(game);
    for name in names {
        // A repeat of the advancement answers "unable to grant": nothing to retry.
        crate::game::bmh::award(game, &name, logic::SAVED_ADVANCEMENT);
    }
}

// T11: refuse a second Totem of Eternity (EternalItemHandler.java:60-77). The eternal area refuses
// it too: both only set `canceled`, with the same answer.
pub(super) fn on_item_picked_up(
    game: &mut crate::game::Game<'_>,
    player: u128,
    item: &str,
    count: u32,
    canceled: &mut bool,
) {
    let _ = (player, count);
    if *canceled || !logic::is_totem(item) {
        return;
    }
    let carried = carried_items(game);
    if logic::refuses_pickup(item, carried.iter().map(|(_, id)| &**id)) {
        *canceled = true;
    }
}

// T1: the tooltip lines of the four totem items (TotemOfEternityItem.java:32-38,
// DormantTotemOfEternityItem.java:116-123, HintItem.java:25-30). They go under the name, as the
// amulet's lines do.
pub(super) fn on_tooltip(
    game: &crate::game::Game<'_>,
    stack: &ItemStack,
    lines: &mut Vec<(Text, u32)>,
) {
    let _ = game;
    let added: Vec<(Text, u32)> = logic::tooltip(&stack.item)
        .into_iter()
        .map(|line| {
            let arguments: Vec<Text> =
                line.number.map(|number| Text::literal(number.to_string())).into_iter().collect();
            (Text::translate(line.key, &arguments).with_color(line.color), u32::MAX)
        })
        .collect();
    if added.is_empty() {
        return;
    }
    let at = lines.len().min(1);
    lines.splice(at..at, added);
}

// T13, T14: refuse a hopper move of a Totem of Eternity into a container that holds one
// (I_ContainerTotemMixin.java:27-44: `canPlaceItem` is false when another totem is in the
// container; a double chest counts both halves). A chest minecart or boat is reported at the block it
// is in. `native_container_slots` does not read it, so the entity's items are read from the game's
// list of vehicles (`vehicle_holds_totem_at`).
pub(super) fn on_inventory_move_item(
    game: &crate::game::Game<'_>,
    source: [i32; 3],
    destination: [i32; 3],
    item: &str,
    cancel: &mut bool,
) {
    let _ = source;
    if *cancel || !logic::is_totem(item) {
        return;
    }
    let dimension = game.native_dimension();
    match container_totems(game, dimension, destination) {
        Some(found) => {
            if !found.is_empty() {
                *cancel = true;
            }
        }
        // No block container there: a chest minecart or a chest boat is reported at the block it is
        // in (T14). Its items are read from the entity.
        None => {
            if vehicle_holds_totem_at(game, destination) {
                *cancel = true;
            }
        }
    }
}

// T13: a hopper picks up a Totem of Eternity from the ground only when it holds none
// (I_ContainerTotemMixin.java:27-44: `canPlaceItem`). The hook has no `Game`, but it binds the
// level, which holds the world's block entities: the hopper's slots are read there. The hook acts
// only inside a server tick that the gated `server_tick` hook opened (`active`).
pub(super) fn on_hopper_pickup_item(
    level: &crate::game::GameLevel<'_>,
    dimension: crate::game::LevelDimension,
    pos: horizon_core::position::BlockPos,
    item: &str,
    count: u32,
    cancel: &mut bool,
) {
    let _ = (dimension, count);
    if !active() || !logic::is_totem(item) {
        return;
    }
    let holds = level
        .world
        .block_entities
        .get(&(pos.x, pos.y, pos.z))
        .is_some_and(|entity| count_totems(entity.slots()) > 0);
    if holds {
        *cancel = true;
    }
}

// T12: note the container that the player opened, and which slots held a totem.
pub(super) fn on_player_container_open(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    dimension: &str,
    menu: &str,
    pos: Option<[i32; 3]>,
) {
    let _ = (name, menu);
    let before = pos.and_then(|pos| container_totems(game, dimension, pos).map(|found| (pos, found)));
    let mut s = server();
    match before {
        Some((pos, before)) => {
            s.open.insert(player, OpenMenu { dimension: dimension.to_owned(), pos, before });
        }
        None => {
            s.open.remove(&player);
        }
    }
}

// T12: scan the closed container once more, and forget it. The menu's items are where it left them.
pub(super) fn on_player_container_close(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    dimension: &str,
    menu: &str,
    pos: Option<[i32; 3]>,
) {
    let _ = (name, dimension, menu, pos);
    scan_open_container(game, player);
    server().open.remove(&player);
}

// ---------------------------------------------------------------------------------------------
// T12, T14: the slot rule of the menus, before the click lands
// ---------------------------------------------------------------------------------------------

/// True for a stack of a Totem of Eternity.
fn holds_totem(stack: &Option<ItemStack>) -> bool {
    stack.as_ref().is_some_and(|stack| stack.count > 0 && logic::is_totem(&stack.item))
}

/// The totems in a list of slots.
fn count_totems(slots: &[Option<ItemStack>]) -> usize {
    slots.iter().filter(|stack| holds_totem(stack)).count()
}

/// The guest `player` and the slots of the menu `window` that he has open, as a list that says
/// which slot holds a totem, in the menu's order (which is the window's order). The container
/// can be a block, a double chest, the ender chest, or the items of a chest minecart or a chest
/// boat (`Slots::read` reads the key under which the game keeps them while a menu shows them).
/// None for the inventory window, a menu of a mod and a menu that is not this window.
fn menu_totems<'a>(
    game: &'a crate::game::Game<'_>,
    player: u128,
    window: i32,
) -> Option<(&'a horizon_client::lan::guest::Guest, Vec<bool>)> {
    if window == 0 {
        return None;
    }
    let host = game.lan.as_ref()?;
    let guest = host.hub.guests.iter().find(|guest| guest.placed && guest.uuid == player)?;
    let menu = guest.menu.as_ref().filter(|menu| menu.window == window && menu.mod_type.is_none())?;
    if menu.slots == Slots::Native {
        return None;
    }
    let items = menu.slots.read(&game.world, guest)?;
    Some((guest, items.iter().map(holds_totem).collect()))
}

// T12, T14: a click in a container menu that would put a second Totem of Eternity in the container
// is canceled before it lands. This is the job of `Slot.mayPlace` in the Java mod
// (I_SlotTotemMixin.java:21-28). The game answers a canceled click as one the server refused:
// the slots and the cursor of the client go back to the server's.
// GAME RESULT (2026-11-19): in the game this hook finds no menu. The game takes the LAN host out of
// `game.lan` while it handles a click (`lan_inventory_click_hooks(&mut self, host, ...)`), so `game.lan`
// is None here and the cursor, the inventory and the open menu cannot be read. The code is kept for
// the injection proposal in docs/horizon/solutions/totem-containers.md; today it returns at once.
pub(super) fn on_inventory_click(
    game: &crate::game::Game<'_>,
    player: u128,
    name: &str,
    window: i32,
    slot: i32,
    button: i32,
    mode: i32,
    cancel: &mut bool,
) {
    let _ = name;
    if *cancel {
        return;
    }
    let refused = {
        let Some((guest, container)) = menu_totems(game, player, window) else {
            return;
        };
        if !container.iter().any(|totem| *totem) {
            return;
        }
        let size = container.len();
        // The window after the container: the 27 main slots, then the 9 of the hotbar.
        let player_stack = |window_slot: usize| -> Option<&Option<ItemStack>> {
            let index = window_slot.checked_sub(size)?;
            if index < 27 {
                guest.inventory.get(9 + index)
            } else if index < 36 {
                guest.inventory.get(index - 27)
            } else {
                None
            }
        };
        let key_stack = mode == 2
            && match button {
                40 => holds_totem(&guest.offhand),
                0..=8 => usize::try_from(button).ok().and_then(|n| guest.inventory.get(n)).is_some_and(holds_totem),
                _ => false,
            };
        let click = logic::Click {
            slot,
            mode,
            cursor: holds_totem(&guest.carried),
            slot_stack: usize::try_from(slot).ok().and_then(player_stack).is_some_and(holds_totem),
            key_stack,
        };
        logic::click_refused(&container, &click)
    };
    if refused {
        *cancel = true;
        log::info!(
            "[BalancedMcHardcoreMain] A click that would put a second Totem of Eternity in a container was refused"
        );
    }
}

// T12, T14: the same for a drag over the slots of a container menu.
pub(super) fn on_inventory_drag(
    game: &crate::game::Game<'_>,
    player: u128,
    name: &str,
    window: i32,
    kind: i32,
    slots: &[i32],
    cancel: &mut bool,
) {
    let _ = (name, kind);
    if *cancel {
        return;
    }
    let Some((guest, container)) = menu_totems(game, player, window) else {
        return;
    };
    if logic::drag_refused(&container, slots, holds_totem(&guest.carried)) {
        *cancel = true;
        log::info!(
            "[BalancedMcHardcoreMain] A drag that would put a second Totem of Eternity in a container was refused"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// T13, T14: chest minecarts, chest boats and the paths with no hook
// ---------------------------------------------------------------------------------------------

/// The block that holds a position of the world.
fn block_of(position: &impl VehiclePosition) -> [i32; 3] {
    let [x, y, z] = position.xyz();
    [x.floor() as i32, y.floor() as i32, z.floor() as i32]
}

/// The position of a vehicle as three numbers (the game's own vector type).
trait VehiclePosition {
    fn xyz(&self) -> [f64; 3];
}

impl VehiclePosition for horizon_core::geometry::Vec3 {
    fn xyz(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
}

/// True when the vehicle `id` holds a totem. While a menu shows the vehicle, the game keeps the live
/// items under a key in the world's block entities and the vehicle's own list may be old: that copy
/// counts then.
fn vehicle_has_totem(game: &crate::game::Game<'_>, id: u32, own: &[Option<ItemStack>]) -> bool {
    let key = (i32::MAX - 15, VEHICLE_KEY_Y, id as i32);
    let slots = game.world.block_entities.get(&key).map_or(own, |entity| entity.slots());
    count_totems(slots) > 0
}

/// T14: true when a chest minecart, hopper minecart or chest boat in the block `pos` holds a totem.
fn vehicle_holds_totem_at(game: &crate::game::Game<'_>, pos: [i32; 3]) -> bool {
    game.carts.iter().any(|cart| block_of(&cart.position) == pos && vehicle_has_totem(game, cart.id, &cart.items))
        || game.boats.iter().any(|boat| block_of(&boat.position) == pos && vehicle_has_totem(game, boat.id, &boat.items))
}

/// Takes `count` totems out of `items`, the ones in the highest slots first.
fn take_totems(items: &mut [Option<ItemStack>], count: usize) -> Vec<ItemStack> {
    let mut taken = Vec::new();
    for slot in items.iter_mut().rev() {
        if taken.len() >= count {
            break;
        }
        if holds_totem(slot) {
            taken.extend(slot.take());
        }
    }
    taken
}

/// Drops `stack` at `at` as an item entity with a pickup delay, a small hop and (for an Eternal Item)
/// the health that a blast does not kill. False when the game made no entity.
fn drop_at(game: &mut crate::game::Game<'_>, dimension: &str, at: [f64; 3], stack: &ItemStack) -> bool {
    let mut nbt = Compound::new();
    nbt.insert("Item".into(), horizon_client::save::item_to_nbt(stack));
    nbt.insert("PickupDelay".into(), Tag::Short(eternal_logic::DROP_PICKUP_DELAY));
    nbt.insert(
        "Motion".into(),
        Tag::List(vec![Tag::Double(0.0f64.to_bits()), Tag::Double(0.2f64.to_bits()), Tag::Double(0.0f64.to_bits())]),
    );
    if let Some(health) = eternal_logic::item_health(&stack.item) {
        nbt.insert("Health".into(), Tag::Short(health));
    }
    game.native_summon(dimension, "minecraft:item", at, nbt).is_some()
}

/// The scan of T13 and T14, once a second: any container that grew to two totems since the last scan
/// gives the new ones back to the world. It catches what has no hook: a dropper, a hopper minecart
/// that picks up an item, a command. The first scan of a world only counts, and a container that is
/// new to the scan (a chunk that just loaded) is only counted, so a pair that was already there is
/// left alone, as in Java.
fn scan_all(game: &mut crate::game::Game<'_>) {
    let run = {
        let mut s = server();
        s.scan_ticks += 1;
        if s.scan_ticks >= SCAN_EVERY {
            s.scan_ticks = 0;
            true
        } else {
            false
        }
    };
    if !run {
        return;
    }
    let (ready, blocks, vehicles) = {
        let s = server();
        (s.scan_ready, s.scan_blocks.clone(), s.scan_vehicles.clone())
    };
    let dimension = game.native_dimension();
    let (blocks_now, vehicles_now) = (
        scan_blocks(game, dimension, ready, &blocks),
        scan_vehicles(game, dimension, ready, &vehicles),
    );
    let mut s = server();
    s.scan_blocks = blocks_now;
    s.scan_vehicles = vehicles_now;
    s.scan_ready = true;
}

/// Server ticks between two scans.
const SCAN_EVERY: u32 = 20;

fn scan_blocks(
    game: &mut crate::game::Game<'_>,
    dimension: &'static str,
    ready: bool,
    previous: &BTreeMap<(String, [i32; 3]), usize>,
) -> BTreeMap<(String, [i32; 3]), usize> {
    let candidates: Vec<[i32; 3]> = game
        .world
        .block_entities
        .iter()
        .filter(|(pos, entity)| {
            pos.1 != VEHICLE_KEY_Y
                && !matches!(
                    entity,
                    BlockEntity::Beehive(_) | BlockEntity::Campfire(_) | BlockEntity::Brushable(_) | BlockEntity::Mod(_)
                )
                && count_totems(entity.slots()) > 0
        })
        .map(|(pos, _)| [pos.0, pos.1, pos.2])
        .collect();
    let mut now = BTreeMap::new();
    for pos in candidates {
        let Some(found) = container_totems(game, dimension, pos) else {
            continue;
        };
        let key = (dimension.to_owned(), pos);
        // The other half of a double chest knows the count of the whole.
        let known = previous.get(&key).copied().or_else(|| {
            [[1, 0], [-1, 0], [0, 1], [0, -1]]
                .iter()
                .find_map(|[dx, dz]| previous.get(&(dimension.to_owned(), [pos[0] + dx, pos[1], pos[2] + dz])).copied())
        });
        let mut count = found.len();
        let eject = if ready { logic::scan_ejects(known, count) } else { 0 };
        for slot in found.iter().rev().take(eject) {
            let Some(stack) = game.native_extract_item(dimension, pos, Direction::Up, *slot, 1) else {
                continue;
            };
            count = count.saturating_sub(1);
            let at = [f64::from(pos[0]) + 0.5, f64::from(pos[1]) + 1.0, f64::from(pos[2]) + 0.5];
            if !drop_at(game, dimension, at, &stack) {
                log::warn!("[BalancedMcHardcoreMain] A second Totem of Eternity left a container and could not be dropped");
            }
        }
        if eject > 0 {
            log::info!("[BalancedMcHardcoreMain] The scan took a second Totem of Eternity out of a container");
        }
        now.insert(key, count);
    }
    now
}

fn scan_vehicles(
    game: &mut crate::game::Game<'_>,
    dimension: &'static str,
    ready: bool,
    previous: &BTreeMap<u32, usize>,
) -> BTreeMap<u32, usize> {
    // (id, is a boat, position) of the vehicles with a totem that no menu shows now.
    let mut found: Vec<(u32, bool, [f64; 3], usize)> = Vec::new();
    for cart in game.carts.iter() {
        let count = count_totems(&cart.items);
        if count > 0 && !game.world.block_entities.contains_key(&(i32::MAX - 15, VEHICLE_KEY_Y, cart.id as i32)) {
            found.push((cart.id, false, cart.position.xyz(), count));
        }
    }
    for boat in game.boats.iter() {
        let count = count_totems(&boat.items);
        if count > 0 && !game.world.block_entities.contains_key(&(i32::MAX - 15, VEHICLE_KEY_Y, boat.id as i32)) {
            found.push((boat.id, true, boat.position.xyz(), count));
        }
    }
    let mut now = BTreeMap::new();
    for (id, boat, at, count) in found {
        let eject = if ready { logic::scan_ejects(previous.get(&id).copied(), count) } else { 0 };
        let taken = if eject == 0 {
            Vec::new()
        } else if boat {
            game.boats.iter_mut().find(|b| b.id == id).map(|b| take_totems(&mut b.items, eject)).unwrap_or_default()
        } else {
            game.carts.iter_mut().find(|c| c.id == id).map(|c| take_totems(&mut c.items, eject)).unwrap_or_default()
        };
        for stack in &taken {
            if !drop_at(game, dimension, [at[0], at[1] + 1.0, at[2]], stack) {
                log::warn!("[BalancedMcHardcoreMain] A second Totem of Eternity left a vehicle and could not be dropped");
            }
        }
        if !taken.is_empty() {
            log::info!("[BalancedMcHardcoreMain] The scan took a second Totem of Eternity out of a vehicle");
        }
        now.insert(id, count - taken.len());
    }
    now
}

// ---------------------------------------------------------------------------------------------
// T6, E6: the anvil
// ---------------------------------------------------------------------------------------------

// T6: a Hollow Totem on the left and an Echo Heart on the right give a Totem of Eternity for 39
// levels, and the take uses one Echo Heart (B_AnvilMenuMixin.java:68-79). The name field is
// ignored: the result is a fresh stack.
// E6: a gold block repair of an Eternal tool costs 15 levels and keeps the prior work penalty of
// the tool, on the result as on the tool (B_AnvilMenuMixin.java:81-114). Java puts the penalty aside
// before vanilla's run, so vanilla never finds the repair too expensive. Here vanilla's run keeps the
// penalty. When it gives a result, that result stands: only its penalty and the cost change. When it
// says "Too Expensive" (no result and a cost of 40 or more), the repair itself was valid, and the hook
// builds the result that vanilla gives with no penalty (`gold_repair_result`). For any other refusal
// (nothing to repair), there is no result, as in Java.
// The hook runs on both sides with the same inputs: the answer depends on the stacks only.
pub(super) fn on_anvil_update(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    left: &ItemStack,
    right: Option<&ItemStack>,
    item_name: Option<&str>,
    vanilla_output: Option<&ItemStack>,
    vanilla_xp_cost: i32,
    vanilla_material_cost: i32,
    output: &mut Option<ItemStack>,
    xp_cost: &mut i32,
    material_cost: &mut i32,
    canceled: &mut bool,
) {
    let _ = (game, player, name, vanilla_material_cost);
    if *canceled {
        return;
    }
    match logic::anvil_case(&left.item, right.map(|stack| &*stack.item)) {
        Some(logic::Anvil::Awaken) => {
            *output = Some(ItemStack::new(ids::TOTEM_OF_ETERNITY, 1));
            *xp_cost = logic::AWAKEN_COST;
            *material_cost = logic::AWAKEN_MATERIAL_COST;
        }
        Some(logic::Anvil::GoldRepair) => {
            if output.is_none()
                && vanilla_output.is_none()
                && vanilla_xp_cost >= logic::TOO_EXPENSIVE
                && let Some((result, used)) =
                    right.and_then(|blocks| gold_repair_result(left, blocks, item_name))
            {
                *output = Some(result);
                *material_cost = i32::try_from(used).unwrap_or(i32::MAX);
            }
            let Some(result) = output.as_mut() else {
                return;
            };
            let penalty = left.repair_cost();
            if result.repair_cost() != penalty {
                result.data_mut().repair_cost = penalty;
            }
            *xp_cost = logic::GOLD_REPAIR_COST;
        }
        None => {}
    }
}

/// E6: the result of vanilla `AnvilMenu.createResult` for a gold block repair of `left` with the gold
/// blocks `blocks` and the rename box text `item_name`, as Java gets it with the prior work penalty
/// put aside (B_AnvilMenuMixin.java:82-90): the repaired damage and the rename. The caller sets the
/// penalty and the cost. Also the gold blocks it uses. None when there is nothing to repair, or when
/// Java's run is still too expensive.
fn gold_repair_result(left: &ItemStack, blocks: &ItemStack, item_name: Option<&str>) -> Option<(ItemStack, u32)> {
    let repair = logic::gold_repair(left.damage, blocks.count)?;
    let custom = left.custom_name();
    let hover = custom.clone().or_else(|| left.item_name());
    let rename = logic::anvil_rename(item_name, hover.as_deref(), custom.is_some());
    if logic::gold_repair_refused(blocks.repair_cost(), repair.blocks, rename != logic::Rename::Keep) {
        return None;
    }
    let mut result = left.clone();
    result.damage = repair.damage;
    match rename {
        logic::Rename::Keep => {}
        logic::Rename::Set => {
            if let Some(text) = item_name {
                result.data_mut().custom_name = Some(Tag::String(text.into()));
            }
        }
        logic::Rename::Clear => {
            result.data_mut().custom_name = None;
        }
    }
    result.tidy();
    Some((result, repair.blocks))
}
