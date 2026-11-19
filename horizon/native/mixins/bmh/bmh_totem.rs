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
// - A hopper or dropper keeps a totem when the destination already holds one.

use bmh_logic::eternal as eternal_logic;
use bmh_logic::ids;
use bmh_logic::totem as logic;
use horizon_client::items::ItemStack;
use horizon_client::lan::menu::{Slots, VEHICLE_KEY_Y};
use horizon_client::mod_api::{NativeEffect, NativeServerPlayer};
use horizon_client::particles::{ParticleOptions, ParticleType};
use horizon_client::text::Text;
use horizon_world::nbt::Tag;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};

/// The game mode number of a spectator (`NativeServerPlayer.game_mode`: 3 is spectator).
/// TotemOfEternityHandler.java:62 skips a spectator.
const SPECTATOR: i32 = 3;

/// The name of the player entity in `living_death`.
const PLAYER_KIND: &str = "minecraft:player";

/// The container that a player has open: where it is, and which slots held a totem when he opened it.
struct OpenMenu {
    dimension: String,
    slots: Slots,
    clicked: bool,
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
    player: NativeServerPlayer,
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
}

static SERVER: Mutex<Server> = Mutex::new(Server {
    book: logic::SaveBook::new(),
    open: BTreeMap::new(),
    awards: Vec::new(),
    hand: Vec::new(),
    epoch: None,
});

/// The server state. A lock that a panic left behind still holds valid data.
fn server() -> MutexGuard<'static, Server> {
    SERVER.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
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

/// Returns the item id only if this stack can protect the player.
fn hand_protector<'a>(game: &crate::game::Game<'_>, stack: Option<&'a ItemStack>) -> Option<&'a str> {
    let stack = stack.filter(|stack| stack.count > 0)?;
    let view = horizon_client::item_render::stack_view(stack);
    game.pack.item_renderer.models.components(&view)
        .contains_key("minecraft:death_protection")
        .then_some(stack.item.as_ref())
}

// T2, route A (docs/horizon/solutions/totem-save.md): Horizon's own hand check saves the player
// with the `death_protection` of the Totem of Eternity (content item). This hook runs before the
// game spends the hand. The player is swapped in, at 0 health. The slot is not written here: the
// use-up reads the hands after this hook.
pub(super) fn on_player_totem(game: &mut crate::game::Game<'_>, player: u128, _name: &str) {
    let Some(me) = game.native_acting_player() else {
        return;
    };
    if me.uuid != player {
        return;
    }
    let main_slot = i32::from(me.selected_slot);
    let main = hand_protector(game, game.inventory.get(usize::from(me.selected_slot)).and_then(Option::as_ref));
    let off = hand_protector(game, game.offhand.as_ref());
    let work = match logic::hand_save(main, off, me.selected_slot) {
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
    server().hand.push(HandSaveRecord { player: me, work });
}

/// Returns true once the Dormant Totem reaches its slot, the inventory or the ground.
fn place_dormant(game: &mut crate::game::Game<'_>, player: &NativeServerPlayer, slot: usize) -> bool {
    let dormant = ItemStack::new(ids::SLEEPING_TOTEM, 1);
    let kind = crate::game::NativeInventoryKind::Player;
    if game.native_set_inventory_slot(player.uuid, kind, slot, None, Some(dormant.clone())).is_ok() {
        return true;
    }
    match game.native_give_stack(player.uuid, dormant) {
        None => true,
        Some(left) => crate::game::bmh_eternal::drop_at_player(game, player, &left, false),
    }
}

/// Puts the Totem of Eternity back from the off hand into the main hand `slot` after a swap save.
/// It moves only when the off hand still holds it and the main slot is empty.
fn swap_back(game: &mut crate::game::Game<'_>, uuid: u128, name: &str, slot: usize, totem: ItemStack) {
    let kind = crate::game::NativeInventoryKind::Player;
    let Ok(inventory) = game.native_inventory(uuid, kind) else { return };
    if !matches!(inventory.slots.get(slot), Some(None)) {
        return;
    }
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
        if crate::game::bmh_eternal::drop_stack(game, &stack, false) {
            remove_totem(game, slot, i32::from(me.selected_slot));
        }
    }
}

/// Visit the live slots in the same order as the open menu.
fn visit_slots(
    game: &mut crate::game::Game<'_>,
    source: Slots,
    mut visit: impl FnMut(usize, &mut Option<ItemStack>),
) {
    let mut index = 0;
    let mut visit_part = |slots: &mut [Option<ItemStack>]| {
        for slot in slots {
            visit(index, slot);
            index += 1;
        }
    };
    match source {
        Slots::Ender => visit_part(&mut game.ender_items),
        Slots::Block(key) if key.1 == VEHICLE_KEY_Y => {
            if let Some(entity) = game.world.block_entities.get_mut(&key) {
                visit_part(entity.slots_mut());
            } else if let Some(cart) = game.carts.iter_mut().find(|c| c.id as i32 == key.2) {
                visit_part(&mut cart.items);
            } else if let Some(boat) = game.boats.iter_mut().find(|b| b.id as i32 == key.2) {
                visit_part(&mut boat.items);
            }
        }
        Slots::Block(key) => {
            if let Some(entity) = game.world.block_entities.get_mut(&key) {
                visit_part(entity.slots_mut());
            }
        }
        Slots::Double(first, second) => {
            for key in [first, second] {
                if let Some(entity) = game.world.block_entities.get_mut(&key) {
                    visit_part(entity.slots_mut());
                }
            }
        }
        Slots::Native => {}
    }
}

fn menu_totems(game: &mut crate::game::Game<'_>, source: Slots) -> Vec<usize> {
    let mut found = Vec::new();
    visit_slots(game, source, |index, slot| {
        if slot.as_ref().is_some_and(|stack| stack.count > 0 && logic::is_totem(&stack.item)) {
            found.push(index);
        }
    });
    found
}

/// The menu event omits the vehicle's key. Read it from the open menu after the host returns.
fn remember_vehicle_menus(game: &crate::game::Game<'_>) {
    let Some(host) = game.lan.as_ref() else { return };
    let mut state = server();
    for guest in &host.hub.guests {
        let Some(menu) = guest.menu.as_ref() else { continue };
        if !matches!(menu.slots, Slots::Block(key) if key.1 == VEHICLE_KEY_Y) {
            continue;
        }
        if state.open.get(&guest.uuid).is_some_and(|old| old.slots == menu.slots && old.dimension == guest.dimension) {
            continue;
        }
        let before = menu.sent.iter().take(menu.size).enumerate().filter_map(|(slot, entry)| {
            entry.as_ref().filter(|item| item.count > 0 && logic::is_totem(&item.item)).map(|_| slot)
        }).collect();
        state.open.insert(guest.uuid, OpenMenu { dimension: guest.dimension.clone(), slots: menu.slots, clicked: false, before });
    }
}

/// Only the player who clicked can receive a returned stack from a shared chest.
pub(super) fn note_menu_click(player: u128, window: i32) {
    if window > 0 && let Some(menu) = server().open.get_mut(&player) {
        menu.clicked = true;
    }
}

/// Return each extra totem to the player who clicked. Preserve its stack data.
fn scan_open_container(game: &mut crate::game::Game<'_>, uuid: u128) {
    let menu = server().open.get_mut(&uuid).and_then(|menu| {
        (menu.dimension == game.native_dimension() && std::mem::take(&mut menu.clicked))
            .then(|| (menu.slots, menu.before.clone()))
    });
    let Some((source, before)) = menu else { return };
    if !game.native_acting_player().is_some_and(|me| me.uuid == uuid) {
        return;
    }
    let mut now = menu_totems(game, source);
    let extras = logic::container_extras(&before, &now);
    if !extras.is_empty() {
        let mut taken = Vec::new();
        visit_slots(game, source, |index, slot| {
            if extras.contains(&index) && let Some(stack) = slot.take() {
                taken.push((index, stack));
            }
        });
        for (index, stack) in taken {
            if let Some(left) = game.native_give_stack(uuid, stack) && !crate::game::bmh_eternal::drop_stack(game, &left, false) {
                let mut restore = Some(left);
                visit_slots(game, source, |slot_index, slot| {
                    if slot_index == index { *slot = restore.take(); }
                });
                continue;
            }
            now.retain(|slot| *slot != index);
            log::info!("[BalancedMcHardcoreMain] Returned a second Totem of Eternity from an open container");
        }
        match source {
            Slots::Block(key) => game.container_changed(key),
            Slots::Double(first, second) => {
                game.container_changed(first);
                game.container_changed(second);
            }
            _ => {}
        }
    }
    for (player, menu) in &mut server().open {
        if menu.dimension == game.native_dimension() && menu.slots == source && (*player == uuid || source != Slots::Ender) {
            menu.before.clone_from(&now);
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

// T3: the advancement of a save. A state tied to an old world is forgotten (the owner of a
// single-player world has the same uuid in every world).
pub(super) fn on_server_tick(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLifecycleContext,
    phase: horizon_client::mod_api::ServerTickPhase,
) {
    if let horizon_client::mod_api::ServerTickPhase::ServerTickStart { .. } = phase {
        let mut s = server();
        if s.epoch != Some(context.epoch) {
            s.book.clear();
            s.open.clear();
            s.awards.clear();
            s.hand.clear();
            s.epoch = Some(context.epoch);
        }
        return;
    }
    let horizon_client::mod_api::ServerTickPhase::ServerTickEnd { .. } = phase else {
        return;
    };
    let (mut names, hand) = {
        let mut s = server();
        (std::mem::take(&mut s.awards), std::mem::take(&mut s.hand))
    };
    // T2, route A: no player is swapped in here, so `native_set_inventory_slot` reaches each one.
    for record in hand {
        match record.work {
            HandWork::Dormant { slot } => {
                if !place_dormant(game, &record.player, slot) {
                    server().hand.push(record);
                    continue;
                }
                log::info!(
                    "[BalancedMcHardcoreMain] The Totem of Eternity saved {} and is now dormant",
                    record.player.name
                );
                names.push(record.player.name);
            }
            HandWork::SwapBack { slot, totem } => {
                swap_back(game, record.player.uuid, &record.player.name, slot, totem);
            }
        }
    }
    remember_vehicle_menus(game);
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

// T12: note the container that the player opened, and which slots held a totem.
pub(super) fn on_player_container_open(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    dimension: &str,
    menu: &str,
    pos: Option<[i32; 3]>,
) {
    let _ = name;
    if !menu.starts_with("minecraft:generic_9x") && !matches!(menu, "minecraft:hopper" | "minecraft:shulker_box" | "minecraft:generic_3x3") {
        return;
    }
    let Some([x, y, z]) = pos else { return };
    let key = (x, y, z);
    let source = if game.res.facts().get(game.world.block(x, y, z)).name.as_ref() == "minecraft:ender_chest" {
        Slots::Ender
    } else if let Some((other, right)) = game.chest_partner(key) {
        if right { Slots::Double(key, other) } else { Slots::Double(other, key) }
    } else if game.world.block_entities.contains_key(&key) {
        Slots::Block(key)
    } else {
        return;
    };
    let before = menu_totems(game, source);
    server().open.insert(player, OpenMenu { dimension: dimension.to_owned(), slots: source, clicked: false, before });
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
