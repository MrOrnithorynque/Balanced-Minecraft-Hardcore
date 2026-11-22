use crate::logic::{contract::Rule, eternal, ids, tools};
use mod_api::{
    AttachmentCodec, AttachmentType, Compound, GameMode, GameRuleValue, Holder, ItemPickedUpEvent,
    ItemStack, LivingDropsEvent, PlayerRef, ServerHost, Tag, TriState,
};
use std::collections::{BTreeMap, HashSet};
use std::sync::Mutex;

struct DeathInventory {
    items: Vec<ItemStack>,
    wear_kept_inventory: bool,
}

static PENDING: Mutex<BTreeMap<u128, DeathInventory>> = Mutex::new(BTreeMap::new());

static KEPT: AttachmentType<Compound> = AttachmentType {
    namespace: "bmh",
    name: "eternal_kept",
    default: Compound::new,
    codec: Some(AttachmentCodec {
        encode: |value| Ok(value.clone()),
        decode: |value| Ok(value.clone()),
    }),
    copy_on_death: true,
    sync: false,
};
const KEPT_ITEMS: &str = "items";
const WEAR_INVENTORY: &str = "wear_inventory";

pub(in crate::combat) fn clear() {
    PENDING.lock().unwrap_or_else(|e| e.into_inner()).clear();
}

pub(crate) fn record_death(host: &dyn ServerHost, player: PlayerRef) {
    let Some(level) = player.level(host) else {
        return;
    };
    let keep_inventory = matches!(
        level.game_rule(host, "minecraft:keep_inventory"),
        Ok(Some(GameRuleValue::Bool(true))),
    );
    let wear = crate::settings::rule(host, Rule::DamageEternalItemWhenDead)
        && player.game_mode(host) == Some(GameMode::Survival);
    let items = if keep_inventory {
        Vec::new()
    } else {
        player
            .inventory_slots(host)
            .unwrap_or_default()
            .into_iter()
            .map(|(_, stack)| stack)
            .filter(|stack| ids::is_eternal(&stack.item().id(host)))
            .map(|stack| if wear { death_wear(host, stack) } else { stack })
            .collect()
    };
    PENDING.lock().unwrap_or_else(|e| e.into_inner()).insert(
        player.uuid(),
        DeathInventory {
            items,
            wear_kept_inventory: keep_inventory && wear,
        },
    );
}

pub(crate) fn on_living_drops(host: &mut dyn ServerHost, event: &mut LivingDropsEvent<'_>) {
    if event.kind != "minecraft:player" || event.cancelled {
        return;
    }
    let Some(uuid) = event.player else {
        return;
    };
    let Some(DeathInventory {
        mut items,
        wear_kept_inventory,
    }) = PENDING
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .remove(&uuid)
    else {
        return;
    };
    let Some(player) = host.player(uuid) else {
        return;
    };

    // Read the complete inventory before vanilla removes items with Curse of Vanishing.
    // Keep additional Eternal Items if an earlier drop listener added any.
    let mut in_inventory = BTreeMap::<_, usize>::new();
    for stack in &items {
        *in_inventory.entry(stack.item().id(host)).or_default() += 1;
    }
    for drop in event.drops.iter() {
        let id = drop.stack.item().id(host);
        if ids::is_eternal(&id) {
            let count = in_inventory.entry(id).or_default();
            if *count == 0 {
                items.push(drop.stack.clone());
            } else {
                *count -= 1;
            }
        }
    }

    let holder = Holder::Player(uuid);
    let Ok(previous) = KEPT.get_existing(host.attachments(), holder) else {
        return;
    };
    let mut kept = previous.unwrap_or_default();
    let Some(mut nbt) = items
        .iter()
        .map(|stack| stack.to_nbt(host).map(Tag::Compound))
        .collect::<Option<Vec<_>>>()
    else {
        return;
    };
    if let Some(legacy) =
        player
            .persistent_data(host)
            .and_then(|data| match data.get(&eternal::KEPT_KEY.into()) {
                Some(Tag::List(stacks)) => Some(stacks.clone()),
                _ => None,
            })
    {
        nbt.extend(legacy);
    }
    if let Some(Tag::List(waiting)) = kept.get_mut(&KEPT_ITEMS.into()) {
        waiting.append(&mut nbt);
    } else if !nbt.is_empty() {
        kept.insert(KEPT_ITEMS.into(), Tag::List(nbt));
    }
    if wear_kept_inventory {
        kept.insert(WEAR_INVENTORY.into(), Tag::Byte(1));
    }
    if kept.is_empty() {
        return;
    }
    if let Err(error) = KEPT.set(host.attachments(), holder, kept) {
        eprintln!("[BMH] Could not keep Eternal Items through death: {error:?}");
        return;
    }
    event
        .drops
        .retain(|drop| !ids::is_eternal(&drop.stack.item().id(host)));
}

pub(crate) fn on_respawn(host: &mut dyn ServerHost, uuid: u128) {
    let Some(player) = host.player(uuid) else {
        return;
    };
    let holder = Holder::Player(uuid);
    let Ok(Some(mut kept)) = KEPT.remove(host.attachments(), holder) else {
        return;
    };
    if kept.remove(&WEAR_INVENTORY.into()).is_some() {
        wear_current_inventory(host, player);
    }
    if let Some(Tag::List(stacks)) = kept.remove(&KEPT_ITEMS.into()) {
        let mut remaining = Vec::new();
        for item in stacks {
            let decoded = match &item {
                Tag::Compound(stack) => ItemStack::from_nbt(host, stack),
                _ => None,
            };
            if decoded.is_none_or(|stack| player.give_stack(host, stack).is_err()) {
                remaining.push(item);
            }
        }
        if !remaining.is_empty() {
            kept.insert(KEPT_ITEMS.into(), Tag::List(remaining));
        }
    }
    if !kept.is_empty()
        && let Err(error) = KEPT.set(host.attachments(), holder, kept)
    {
        eprintln!("[BMH] Could not save Eternal Items awaiting return: {error:?}");
    }
}

fn wear_current_inventory(host: &mut dyn ServerHost, player: PlayerRef) {
    for (slot, stack) in player.inventory_slots(host).unwrap_or_default() {
        let wear = death_wear(host, stack.clone());
        if wear != stack {
            let _ = player.set_inventory_slot(host, slot, Some(wear));
        }
    }
}

fn death_wear(host: &dyn ServerHost, mut stack: ItemStack) -> ItemStack {
    if tools::tool_of(&stack.item().id(host)).is_some() {
        stack.set_damage(tools::death_wear(stack.damage(), super::max_damage(&stack)));
    }
    stack
}

pub(crate) fn on_pickup(host: &dyn ServerHost, event: &mut ItemPickedUpEvent<'_>) {
    let (Some(player), Some(stack)) = (host.player(event.player), event.item.stack(host)) else {
        return;
    };
    let id = stack.item().id(host);
    if !is_limited(&id, survival_limit(host, player)) {
        return;
    }
    if player
        .inventory_slots(host)
        .unwrap_or_default()
        .iter()
        .any(|(_, item)| item.item() == stack.item())
    {
        *event.can_pickup = TriState::False;
    }
}

pub(crate) fn enforce_inventory(host: &mut dyn ServerHost, player: PlayerRef) {
    let Some(mut slots) = player.inventory_slots(host) else {
        return;
    };
    slots.sort_unstable_by_key(|(slot, _)| *slot);
    let mut seen = HashSet::new();
    let survival_limit = survival_limit(host, player);
    for (slot, mut stack) in slots {
        let id = stack.item().id(host);
        if is_limited(&id, survival_limit) && !seen.insert(stack.item()) {
            drop_duplicate(host, player, slot, stack);
        } else if id == ids::ETERNAL_BUNDLE {
            super::bundle::return_refused(host, player, slot, &stack);
        } else if tools::tool_of(&id).is_some() {
            let damage = tools::restore_last_point(stack.damage(), super::max_damage(&stack));
            if damage != stack.damage() {
                stack.set_damage(damage);
                let _ = player.set_inventory_slot(host, slot, Some(stack));
            }
        }
    }
}

fn survival_limit(host: &dyn ServerHost, player: PlayerRef) -> bool {
    player.game_mode(host) == Some(GameMode::Survival)
        && crate::settings::rule(host, Rule::OnlyCarryOneEternalItemType)
}

fn is_limited(id: &str, survival: bool) -> bool {
    id == ids::TOTEM_OF_ETERNITY || (survival && ids::is_eternal(id))
}

fn drop_duplicate(host: &mut dyn ServerHost, player: PlayerRef, slot: u32, stack: ItemStack) {
    let entity = player.entity();
    let (Some(level), Some(position), Some(eye_height)) = (
        player.level(host),
        entity.position(host),
        entity.eye_height(host),
    ) else {
        return;
    };
    if player.set_inventory_slot(host, slot, None).is_err() {
        return;
    }
    let [x, y, z] = eternal::drop_position(position, eye_height);
    if !matches!(
        level.spawn_item(
            host,
            x,
            y,
            z,
            stack.clone(),
            u32::from(eternal::DROP_PICKUP_DELAY as u16),
        ),
        Ok(Some(_))
    ) {
        let _ = player.set_inventory_slot(host, slot, Some(stack));
    }
}
