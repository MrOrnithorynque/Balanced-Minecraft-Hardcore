// Area tools, the part that lives in horizon-client-lib: the hooks there have no `Game`.
// The module is `pub`, so the game side can reach it as `horizon_client_lib::bmh_tools_lib`.
// Owner: the tools agent. The hook helpers (hook_*.rs) call the functions below.
//
// This crate cannot reach `bmh_logic` (docs/horizon/HOOKS.md, "Hooks with no Game"), so the one rule
// that lives here is written out: a landing loses the distance that the smashes of the fall took
// (AxeSmashHandler.java:67, `attacker.fallDistance *= 0.5`). The unit tests of the rule are
// `bmh_logic::tools::a_smash_halves_the_fall`.

use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};

/// The entity type of a player.
const PLAYER: &str = "minecraft:player";

/// The blocks of fall that the smashes removed, by the entity id of the player. The game side sets an
/// entry at a smash and clears it when the player is on the ground again.
static SOFTENING: Mutex<BTreeMap<u32, f64>> = Mutex::new(BTreeMap::new());

/// The lock of the table. A panic in another hook leaves the value usable.
fn table() -> MutexGuard<'static, BTreeMap<u32, f64>> {
    SOFTENING.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// E8: a smash removed `removed` blocks of the fall of the player with this entity id.
pub fn add_softening(entity: u32, removed: f64) {
    if removed.is_finite() && removed > 0.0 {
        *table().entry(entity).or_insert(0.0) += removed;
    }
}

/// E8: the fall of this player ended, or the player left.
pub fn clear_softening(entity: u32) {
    table().remove(&entity);
}

// E8: take the removed distance off the landing of a player who smashed.
pub(crate) fn on_living_fall(
    entity: u32,
    kind: &str,
    distance: &mut f64,
    multiplier: &mut f32,
    cancel: &mut bool,
) {
    let _ = (multiplier, cancel);
    if kind != PLAYER {
        return;
    }
    let removed = table().get(&entity).copied();
    if let Some(removed) = removed {
        *distance = (*distance - removed).max(0.0);
    }
}
