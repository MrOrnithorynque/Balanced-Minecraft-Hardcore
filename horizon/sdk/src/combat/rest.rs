use crate::logic::{contract::Rule, eternal, respawn};
use mod_api::{ActiveEffect, GameMode, GameRuleValue, LevelRef, ServerHost};
use std::collections::BTreeSet;
use std::sync::Mutex;

struct RestState {
    world: Option<LevelRef>,
    waking: BTreeSet<u128>,
    debug: eternal::DebugForce,
}

impl RestState {
    const fn new() -> Self {
        Self {
            world: None,
            waking: BTreeSet::new(),
            debug: eternal::DebugForce::new(),
        }
    }
}

static STATE: Mutex<RestState> = Mutex::new(RestState::new());

pub(in crate::combat) fn clear() {
    *STATE.lock().unwrap_or_else(|e| e.into_inner()) = RestState::new();
}

pub(crate) fn on_bed_leave(host: &dyn ServerHost, player: u128) {
    if crate::settings::rule(host, Rule::FoodAndHealthRegenOnWakeUp) {
        STATE
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .waking
            .insert(player);
    }
}

pub(in crate::combat) fn tick(host: &mut dyn ServerHost) {
    let waking = std::mem::take(&mut STATE.lock().unwrap_or_else(|e| e.into_inner()).waking);
    for uuid in waking {
        let Some(player) = host.player(uuid) else {
            continue;
        };
        let (Some(living), Some(food)) = (host.living(player.entity()), player.food(host)) else {
            continue;
        };
        let Some(health) = living.health(host) else {
            continue;
        };
        let Some(reward) = eternal::sleep_reward(food.level, health) else {
            continue;
        };
        let (_, (food_level, _, exhaustion)) = eternal::after_reward(
            &reward,
            health,
            (food.level, food.saturation, food.exhaustion),
        );
        living.heal(host, reward.heal);
        player.set_food_level(host, food_level);
        player.set_exhaustion(host, exhaustion);
        if reward.effects {
            for id in eternal::WAKE_EFFECTS {
                if let Some(effect) = host.effect_key(id) {
                    living.add_effect(
                        host,
                        &ActiveEffect::new(
                            effect,
                            eternal::WAKE_EFFECT_TICKS,
                            i32::from(eternal::WAKE_EFFECT_AMPLIFIER),
                            false,
                            false,
                            true,
                        ),
                    );
                }
            }
        }
    }
    hide_coordinates(host);
}

fn hide_coordinates(host: &mut dyn ServerHost) {
    let Some(level) = host.level(respawn::OVERWORLD) else {
        return;
    };
    let Ok(Some(GameRuleValue::Bool(current))) = level.game_rule(host, eternal::REDUCED_DEBUG_INFO)
    else {
        return;
    };
    let rule = crate::settings::rule(host, Rule::RemoveCoordinatesInSurvival);
    let survival = host
        .players()
        .server_players()
        .iter()
        .any(|player| player.game_mode(host) == Some(GameMode::Survival));
    let change = {
        let mut state = STATE.lock().unwrap_or_else(|e| e.into_inner());
        if state.world != Some(level) {
            state.debug.forget();
            state.world = Some(level);
        }
        state.debug.step(rule, survival, current)
    };
    if let eternal::DebugAction::Set(value) = change {
        level.set_game_rule(
            host,
            eternal::REDUCED_DEBUG_INFO,
            GameRuleValue::Bool(value),
        );
    }
}
