use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

use crate::logic::{ids, respawn, totem};
use mod_api::{
    ActiveEffect, Flow, Identifier, ItemStack, LivingDeathEvent,
    LivingUseTotemEvent, ServerHost, StatType,
};

use crate::respawn::player_by_entity;

#[derive(Default)]
struct Pending {
    saves: HashSet<u128>,
    hand: HashMap<i32, bool>,
    undying: HashSet<u128>,
}

static PENDING: OnceLock<Mutex<Pending>> = OnceLock::new();

fn pending() -> std::sync::MutexGuard<'static, Pending> {
    PENDING.get_or_init(Mutex::default).lock().unwrap_or_else(|error| error.into_inner())
}

pub(super) fn reset() {
    *pending() = Pending::default();
}


pub(super) fn on_death(host: &mut dyn ServerHost, event: &LivingDeathEvent<'_>) -> Flow {
    if event.kind != "minecraft:player" || bypasses_protection(host, event.source.damage_type) {
        return Flow::Continue;
    }
    let Some(player) = player_by_entity(host, event.entity) else {
        return Flow::Continue;
    };
    let Some(living) = host.living(player.entity()) else {
        return Flow::Continue;
    };
    let (Some(awake), Some(dormant)) = (
        Identifier::parse_namespaced(ids::TOTEM_OF_ETERNITY),
        Identifier::parse_namespaced(ids::SLEEPING_TOTEM),
    ) else {
        return Flow::Continue;
    };
    let (Some(awake), Some(dormant)) = (host.item_key(&awake), host.item_key(&dormant)) else {
        return Flow::Continue;
    };
    let Some(slot) = player.inventory_slots(host).and_then(|slots| {
        slots
            .into_iter()
            .find_map(|(slot, stack)| (stack.item() == awake).then_some(slot))
    }) else {
        return Flow::Continue;
    };

    // The engine attempts a held protector before it calls the death hook.
    // Restore health before writing a living player's inventory.
    if !living.set_health(host, totem::SAVE_HEALTH) {
        return Flow::Continue;
    }
    if let Err(error) = player.set_inventory_slot(host, slot, Some(ItemStack::of(dormant))) {
        let _ = living.set_health(host, 0.0);
        eprintln!("[BMH] Could not replace the inventory totem: {error:?}");
        return Flow::Continue;
    }

    living.remove_all_effects(host);
    for effect in totem::SAVE_EFFECTS {
        if let Some(id) = host.effect_key(effect.id) {
            living.add_effect(
                host,
                &ActiveEffect::new(
                    id,
                    effect.duration,
                    i32::from(effect.amplifier),
                    false,
                    true,
                    true,
                ),
            );
        }
    }
    player.award_stat(host, StatType::Used, ids::TOTEM_OF_ETERNITY, 1);
    player.entity().game_event(host, "minecraft:item_interact_finish");
    player.entity().broadcast_event(host, 35);
    if let Some(name) = player.name(host) {
        eprintln!("[BMH] The Totem of Eternity saved {name} and is now dormant");
    }
    pending().saves.insert(player.uuid());
    Flow::Cancel
}

pub(super) fn inspect_hand(event: &LivingUseTotemEvent<'_>) {
    if event.kind == "minecraft:player" {
        pending().hand.insert(event.entity, event.item == "minecraft:totem_of_undying");
    }
}

pub(super) fn used_hand_item(host: &dyn ServerHost, uuid: u128) {
    let Some(player) = host.player(uuid) else {
        return;
    };
    let mut pending = pending();
    if pending.hand.remove(&(player.entity().id() as i32)) == Some(true) {
        pending.undying.insert(uuid);
    }
}

pub(super) fn end_tick(host: &mut dyn ServerHost) {
    let (saves, undying) = {
        let mut pending = pending();
        pending.hand.clear();
        (std::mem::take(&mut pending.saves), std::mem::take(&mut pending.undying))
    };
    for uuid in saves {
        if let Some(command) = host.player(uuid)
            .and_then(|player| player.name(host))
            .and_then(|name| respawn::award_command(&name, totem::SAVED_ADVANCEMENT))
        {
            host.commands().run_command(&command);
        }
    }
    if let Some(fire_resistance) = host.effect_key("minecraft:fire_resistance") {
        for uuid in undying {
            if let Some(living) = host.player(uuid).and_then(|player| host.living(player.entity())) {
                living.remove_effect(host, fire_resistance);
            }
        }
    }

}

fn bypasses_protection(host: &dyn ServerHost, damage_type: &str) -> bool {
    respawn::BYPASSES_INVULNERABILITY.contains(&damage_type)
        || host
            .registry_entry("minecraft:damage_type", damage_type)
            .and_then(|entry| {
                host.registry_entry_in_tag(&entry, "minecraft:bypasses_invulnerability")
            })
            .unwrap_or(false)
}
