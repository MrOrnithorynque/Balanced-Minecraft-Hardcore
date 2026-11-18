// Area totem, the part that lives in horizon-world: the hooks there have no `Game`.
// The module is `pub`, so the game side can reach it as `horizon_world::bmh_totem_world`.
// Owner: the totem agent. Log: docs/horizon/log/totem.md.
//
// This crate cannot reach `bmh_logic` (the logic crate is a dependency of horizon-client only, see
// docs/horizon/HOOKS.md, "Hooks with no Game"), so the ids are written here. The logic crate pins
// the same effect id (`bmh_logic::totem::UNDYING_LOSES`).

/// The item whose defaults change.
const TOTEM_OF_UNDYING: &str = "minecraft:totem_of_undying";

/// The component that holds the effects of a death protection.
const DEATH_PROTECTION: &str = "minecraft:death_protection";

/// The effect that the Totem of Undying loses (A_LivingEntityTotemMixin.java:30).
const LOST_EFFECT: &str = "minecraft:fire_resistance";

// T5: the Totem of Undying gives no Fire Resistance. A death protection holds a list of
// `death_effects`; the entry of type `apply_effects` holds the effects. This hook takes the Fire
// Resistance out of every such list and leaves the rest (the clear, Regeneration, Absorption).
// The answer depends on the item id alone, so it is the same on both sides and at each call. Mobs
// that hold a totem read the same component, so they lose it too (GAPS T5).
pub(crate) fn on_modify_default_components(
    item: &str,
    components: &mut serde_json::Map<String, serde_json::Value>,
) {
    if item != TOTEM_OF_UNDYING {
        return;
    }
    let Some(entries) = components
        .get_mut(DEATH_PROTECTION)
        .and_then(|component| component.get_mut("death_effects"))
        .and_then(serde_json::Value::as_array_mut)
    else {
        return;
    };
    for entry in entries {
        let Some(effects) = entry.get_mut("effects").and_then(serde_json::Value::as_array_mut) else {
            continue;
        };
        effects.retain(|effect| {
            effect.get("id").and_then(serde_json::Value::as_str) != Some(LOST_EFFECT)
        });
    }
}
