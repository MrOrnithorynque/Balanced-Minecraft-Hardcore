use bmh_logic::{ids, respawn, totem};
use mod_api::{ActiveEffect, Flow, Identifier, ItemStack, LivingDeathEvent, ServerHost, StatType};

pub(super) fn on_death(host: &mut dyn ServerHost, event: &LivingDeathEvent<'_>) -> Flow {
    if event.kind != "minecraft:player" || bypasses_protection(host, event.source.damage_type) {
        return Flow::Continue;
    }
    let Some(player) = host
        .players()
        .server_players()
        .into_iter()
        .find(|player| player.entity().id() == event.entity)
    else {
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
        host.commands().run_command(&format!(
            "advancement grant {name} only bmh:{}",
            totem::SAVED_ADVANCEMENT
        ));
        eprintln!("[BMH] The Totem of Eternity saved {name} and is now dormant");
    }
    Flow::Cancel
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
