// Totem inventory and container limits, anvil recipes, tooltips and advancements.
// Player death protection runs in bmh_player_protection.rs.
// One Totem of Eternity per player (T11) and per container (T12, T13):
// - `player_tick` drops every Totem of Eternity after the first, whatever the game rule says.
// - The menu refuses a second totem before it changes a slot.
// - A hopper or dropper keeps a totem when the destination already holds one.

use bmh_logic::eternal as eternal_logic;
use bmh_logic::ids;
use bmh_logic::totem as logic;
use horizon_client::items::ItemStack;
use horizon_client::mod_api::NativeServerPlayer;
use horizon_client::text::Text;
use horizon_world::nbt::Tag;
use std::sync::{Arc, Mutex, MutexGuard};

/// The game mode number of a spectator (`NativeServerPlayer.game_mode`: 3 is spectator).
/// TotemOfEternityHandler.java:62 skips a spectator.
const SPECTATOR: i32 = 3;

/// What the area keeps between hooks (all on the server).
struct Server {
    /// The names of the players who were saved and wait for the advancement.
    awards: Vec<String>,
    /// The world that the state above belongs to.
    epoch: Option<horizon_client::mod_api::NativeWorldEpoch>,
}

static SERVER: Mutex<Server> = Mutex::new(Server {
    awards: Vec::new(),
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
// The server can grant the advancement after the player's turn.
pub(in crate::game) fn record_save(name: &str) {
    server().awards.push(name.to_owned());
    log::info!("[BalancedMcHardcoreMain] The Totem of Eternity saved {name} and is now dormant");
}

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

pub(super) fn on_player_tick(game: &mut crate::game::Game<'_>) {
    let Some(me) = game.native_acting_player() else {
        return;
    };
    drop_extra_totems(game, &me);
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
            s.awards.clear();
            s.epoch = Some(context.epoch);
        }
        return;
    }
    let horizon_client::mod_api::ServerTickPhase::ServerTickEnd { .. } = phase else {
        return;
    };
    let names = std::mem::take(&mut server().awards);
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
