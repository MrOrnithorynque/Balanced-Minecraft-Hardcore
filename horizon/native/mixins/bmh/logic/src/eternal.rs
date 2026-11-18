// Area eternal (owner: the eternal agent): the rules for Eternal Items as things a player owns.
// It holds the Eternal Items kept through death, the rule "one of each type", the items on the
// ground, the Eternal Bundle, the rewards of a night of sleep and the hidden coordinates.
// Java reference: handlers/KeepItemAfterDeath, EternalItemHandler, SleepHandler, DisableDebugInfo,
// items/EternalBundleItem, mixin/G_ItemEntityMixin. Log: docs/horizon/log/eternal.md.
// This file knows no game type. The module `bmh_eternal.rs` reads the game and carries out the answers.

use crate::ids::{ETERNAL_BUNDLE, TOTEM_OF_ETERNITY, is_eternal};

/// Game mode 0 is survival (`NativeServerPlayer.game_mode`). Adventure is not survival:
/// Java tests `GameType.SURVIVAL` only (EternalItemHandler.java:82, DisableDebugInfo.java:45).
pub const SURVIVAL: i32 = 0;

// ---------------------------------------------------------------------------------------------
// C11: Eternal Items kept through death (KeepItemAfterDeath.java)
// ---------------------------------------------------------------------------------------------

/// The key of the kept stacks in the compound that the mod keeps on the player. The stacks wait
/// there too, so a restart of the game while the player is dead does not lose them (Java kept
/// them in memory only: KeepItemAfterDeath.java:21, lost at a restart).
pub const KEPT_KEY: &str = "bmh_eternal_kept";

/// Takes every Eternal Item out of the dropped stacks and gives them back in their order
/// (KeepItemAfterDeath.java:34-50: `isItemToKeep` is `instanceof IEternalItem`, the slots go in
/// order). `view` gives the item id and the count of a stack. A stack with a count of 0 does not
/// drop, so it is not kept. The other stacks stay in `drops` in their order. The answer does not
/// depend on what the other areas did to `drops` before: Respawn counts first and Tools wears
/// the tools first (docs/horizon/HOOKS.md, `loot_drops`), and this function only partitions.
pub fn take_eternal<T>(
    drops: &mut Vec<T>,
    view: impl for<'a> Fn(&'a T) -> (&'a str, u32),
) -> Vec<T> {
    let mut kept = Vec::new();
    let mut rest = Vec::with_capacity(drops.len());
    for stack in drops.drain(..) {
        let eternal = {
            let (item, count) = view(&stack);
            count > 0 && is_eternal(item)
        };
        if eternal {
            kept.push(stack);
        } else {
            rest.push(stack);
        }
    }
    *drops = rest;
    kept
}

/// The stacks that wait for a player after a death: the ones that already wait, then the new ones.
/// Java replaces the list when the new one is not empty (KeepItemAfterDeath.java:53-54) and keeps
/// it when the new one is empty. Waiting stacks exist at a second death only when the first
/// respawn could not give them. Java lost them there; the port keeps them, so no path loses a
/// stack. Horizon kills a player who logs in dead once more: that death must not erase them.
pub fn keep_after_death<T>(mut waiting: Vec<T>, new: Vec<T>) -> Vec<T> {
    waiting.extend(new);
    waiting
}

/// The lists that wait in memory for a player, because the game refused to save them on the
/// player. They belong to one world: the single-player owner has the same uuid in every world, so
/// a list must not outlive the world where the player died. `E` is the world epoch of the host.
/// A stored empty list means "nothing waits", and it hides an older list in the saved data that
/// could not be cleared.
pub struct Fallback<E, T> {
    epoch: Option<E>,
    lists: std::collections::BTreeMap<u128, Vec<T>>,
}

impl<E: PartialEq + Copy, T> Fallback<E, T> {
    pub const fn new() -> Self {
        Fallback { epoch: None, lists: std::collections::BTreeMap::new() }
    }

    /// Forgets every list when `epoch` is not the world of the lists.
    pub fn new_world(&mut self, epoch: E) {
        if self.epoch != Some(epoch) {
            self.lists.clear();
            self.epoch = Some(epoch);
        }
    }

    pub fn get(&self, player: u128) -> Option<&Vec<T>> {
        self.lists.get(&player)
    }

    pub fn put(&mut self, player: u128, list: Vec<T>) {
        self.lists.insert(player, list);
    }

    pub fn clear(&mut self, player: u128) {
        self.lists.remove(&player);
    }
}

/// The slot of the first hand, in the given order, whose bundle holds something. Each pair is a
/// slot and whether the Eternal Bundle in it is not empty. Java answers FAIL for an empty bundle,
/// and Minecraft then tries the other hand (EternalBundleItem.java:155).
pub fn first_loaded(hands: impl IntoIterator<Item = (i32, bool)>) -> Option<i32> {
    hands.into_iter().find(|(_, loaded)| *loaded).map(|(slot, _)| slot)
}

// ---------------------------------------------------------------------------------------------
// E11: one Eternal Item of each type (EternalItemHandler.java)
// ---------------------------------------------------------------------------------------------

/// The rule "one of each type" applies to a player in survival when the world rule is on
/// (EternalItemHandler.java:79-83, `ruleApplies`).
pub fn rule_applies(rule_on: bool, game_mode: i32) -> bool {
    rule_on && game_mode == SURVIVAL
}

/// The slots of a player in the order of `native_inventory_tick`: the 36 main slots, the armour
/// from the head (103) to the feet (100), then the off hand (-106)
/// (`native_inventory_ticks`, horizon-client game/native_hooks.rs). Java scans the container
/// in the order main slots, armour feet to head, off hand (EternalItemHandler.java:33-36). The
/// armour order is the only difference, and an Eternal Item does not go in an armour slot.
pub fn slot_order() -> Vec<i32> {
    (0..36).chain([103, 102, 101, 100]).chain([-106]).collect()
}

/// The slots that come before `slot` in the order of `slot_order`. The first stack of a type
/// stays. Each later one is a copy (EternalItemHandler.java:38-55).
pub fn earlier_slots(slot: i32) -> Vec<i32> {
    slot_order().into_iter().take_while(|s| *s != slot).collect()
}

/// True when `item` is already in an earlier slot: this stack is a copy to drop. The type of the
/// rule is the item itself (EternalItemHandler.java:90, `item.getItem() == eternalItem.getItem()`).
pub fn is_copy<'a>(item: &str, earlier: impl IntoIterator<Item = &'a str>) -> bool {
    earlier.into_iter().any(|id| id == item)
}

/// What Java does with a copy: it drops a copy of the whole stack and takes one off the original
/// (EternalItemHandler.java:43-44, `drop(newItem.copy(), false)` then `shrink(1)`). Gives the count that
/// drops and the count that stays. Every Eternal Item has a stack size of 1, so the stack that
/// stays is empty.
pub fn copy_drop(count: u32) -> (u32, u32) {
    (count, count.saturating_sub(1))
}

/// True when the player must not pick up `item` from the ground (EternalItemHandler.java:60-77,
/// `shouldBlockPickup`). Only an Eternal Item can be refused. The Totem of Eternity is refused
/// always. The other Eternal Items follow the rule of the tick: with `game_mode` None (a player
/// that the area cannot name) or a rule that does not apply, they are not refused. `carried` are
/// the ids of the stacks in the 41 slots of the player.
pub fn blocks_pickup<'a>(
    item: &str,
    rule_on: bool,
    game_mode: Option<i32>,
    carried: impl IntoIterator<Item = &'a str>,
) -> bool {
    if !is_eternal(item) {
        return false;
    }
    if item != TOTEM_OF_ETERNITY && !game_mode.is_some_and(|mode| rule_applies(rule_on, mode)) {
        return false;
    }
    carried.into_iter().any(|id| id == item)
}

// ---------------------------------------------------------------------------------------------
// Dropping a stack from a player (vanilla `Player.drop`, used by Java in E11 and E12)
// ---------------------------------------------------------------------------------------------

/// The pickup delay of a stack that a player drops, in ticks (vanilla `createItemStackToDrop`,
/// and `drop_inventory` in horizon-client game.rs: 40).
pub const DROP_PICKUP_DELAY: i16 = 40;

/// A dropped stack leaves 0.3 below the eyes (vanilla `Player.drop`).
pub const DROP_BELOW_EYES: f64 = 0.3;

/// Where a player's dropped stack starts: the feet, the eye height less 0.3.
pub fn drop_position(feet: [f64; 3], eye_height: f64) -> [f64; 3] {
    [feet[0], feet[1] + eye_height - DROP_BELOW_EYES, feet[2]]
}

/// The motion of a dropped stack (vanilla `Player.drop`, as horizon-client's `drop_motion`
/// computes it). `randomly` scatters it in any direction (the bundle: EternalBundleItem.java:207,
/// `drop(itemStack, true)`). Else it goes along the view (the copy of E11: EternalItemHandler.java:43,
/// `drop(copy, false)`). `yaw` and `pitch` are in degrees. `rand` gives values in [0, 1), in the
/// order of vanilla's draws.
pub fn drop_velocity(yaw: f32, pitch: f32, randomly: bool, rand: &mut impl FnMut() -> f64) -> [f64; 3] {
    let tau = std::f64::consts::TAU;
    if randomly {
        let power = rand() * 0.5;
        let direction = rand() * tau;
        return [-direction.sin() * power, 0.2, direction.cos() * power];
    }
    let (pitch, yaw) = (f64::from(pitch).to_radians(), f64::from(yaw).to_radians());
    let direction = rand() * tau;
    let power = 0.02 * rand();
    let spread = (rand() - rand()) * 0.1;
    [
        -yaw.sin() * pitch.cos() * 0.3 + direction.cos() * power,
        -pitch.sin() * 0.3 + 0.1 + spread,
        yaw.cos() * pitch.cos() * 0.3 + direction.sin() * power,
    ]
}

// ---------------------------------------------------------------------------------------------
// E10: Eternal Items on the ground (G_ItemEntityMixin.java)
// ---------------------------------------------------------------------------------------------

/// The longest extension that `item_expire` takes, in ticks (native_hooks.rs, `native_item_expire`:
/// "Extensions are capped at 32766 ticks"). An item that gets it goes back to the age
/// `6000 - 32766` and asks again when it reaches 6000: an unlimited lifetime in effect
/// (G_ItemEntityMixin.java:45-53, `setUnlimitedLifetime`).
pub const EXTRA_LIFE_TICKS: i32 = 32766;

/// The extra life that an item on the ground gets. Another mod may have added to `current`.
pub fn extra_life(item: &str, current: i32) -> i32 {
    if is_eternal(item) { current.max(EXTRA_LIFE_TICKS) } else { current }
}

/// The health of an Eternal Item on the ground, in the `Health` short of its entity data. Java
/// ignores all damage (G_ItemEntityMixin.java:26-40, `hurtServer` and `hurtClient` return false).
/// Horizon has no hook for the damage of an item entity: explosions take health away, so the
/// health is the largest short (GAPS.md E10: about 6500 blasts).
pub const ITEM_HEALTH: i16 = i16::MAX;

/// The health to give to a new item on the ground. None for an item that is not Eternal.
pub fn item_health(item: &str) -> Option<i16> {
    is_eternal(item).then_some(ITEM_HEALTH)
}

// ---------------------------------------------------------------------------------------------
// E12: the Eternal Bundle (EternalBundleItem.java)
// ---------------------------------------------------------------------------------------------

/// The Ender Chest, which a bundle refuses (EternalBundleItem.java:181).
pub const ENDER_CHEST: &str = "minecraft:ender_chest";

/// True when the Eternal Bundle refuses `item` (EternalBundleItem.java:177-183, `canInsert`: the
/// stack is not an Eternal Item and not an Ender Chest). The other tests of `canInsert` (not
/// empty, `canFitInsideContainerItems`) stay with the template bundle of the game.
pub fn bundle_refuses(item: &str) -> bool {
    is_eternal(item) || item == ENDER_CHEST
}

/// True when `item` is the Eternal Bundle.
pub fn is_bundle(item: &str) -> bool {
    item == ETERNAL_BUNDLE
}

/// The sound of a bundle that drops its contents (EternalBundleItem.java:253, `BUNDLE_DROP_CONTENTS`).
pub const BUNDLE_DROP_SOUND: &str = "minecraft:item.bundle.drop_contents";

/// The volume of the bundle sounds (EternalBundleItem.java:253, 0.8).
pub const BUNDLE_SOUND_VOLUME: f32 = 0.8;

/// The pitch of the bundle sounds: `0.8 + random * 0.4` (EternalBundleItem.java:254).
pub fn bundle_sound_pitch(random: f64) -> f32 {
    (0.8 + random * 0.4) as f32
}

/// The sound source of a player's sounds (`SoundSource.PLAYERS`): the index in horizon-client's
/// `SOUND_SOURCES` ("player", the eighth name).
pub const PLAYER_SOUND_SOURCE: usize = 7;

// ---------------------------------------------------------------------------------------------
// C13: the rewards of a night of sleep (SleepHandler.java)
// ---------------------------------------------------------------------------------------------

/// The exhaustion of a reward (SleepHandler.java:40, `amountOfExhaustion`).
pub const WAKE_EXHAUSTION: f32 = 4.0;

/// The most exhaustion a player holds (vanilla `FoodData.addExhaustion`: 40).
pub const EXHAUSTION_CAP: f32 = 40.0;

/// The most food a player holds.
pub const FOOD_MAX: i32 = 20;

/// The health at which the two effects come (SleepHandler.java:48, `playerHealth >= 20f`).
pub const FULL_HEALTH: f32 = 20.0;

/// The length of Haste II and Resistance II (SleepHandler.java:50-51, 3600 ticks).
pub const WAKE_EFFECT_TICKS: i32 = 3600;

/// The amplifier of both effects (SleepHandler.java:50-51, amplifier 1: level II).
pub const WAKE_EFFECT_AMPLIFIER: u8 = 1;

/// The two effects of a full-health wake-up (SleepHandler.java:50-51). The flags are
/// `ambient` false, particles false, icon shown.
pub const WAKE_EFFECTS: [&str; 2] = ["minecraft:haste", "minecraft:resistance"];

/// What a wake-up gives, by the food level and the health at that moment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SleepReward {
    /// The health points that come back (`player.heal`).
    pub heal: f32,
    /// The exhaustion that is added (`foodData.addExhaustion`).
    pub exhaustion: f32,
    /// The food points that go (`foodData.setFoodLevel(foodLevel - cost)`).
    pub food_cost: i32,
    /// Haste II and Resistance II for 3600 ticks.
    pub effects: bool,
}

/// The reward of a wake-up (SleepHandler.java:39-77). The food level decides, in this order:
/// 15 or more, 10 or more, 5 or more, more than 1. At 1 or 0 nothing happens. `health` is the
/// health before the heal: the two effects need a full 20 (line 48).
pub fn sleep_reward(food: i32, health: f32) -> Option<SleepReward> {
    if food >= 15 {
        Some(SleepReward {
            heal: 6.0,
            exhaustion: WAKE_EXHAUSTION,
            food_cost: 4,
            effects: health >= FULL_HEALTH,
        })
    } else if food >= 10 {
        Some(SleepReward { heal: 4.0, exhaustion: WAKE_EXHAUSTION, food_cost: 2, effects: false })
    } else if food >= 5 {
        Some(SleepReward { heal: 3.0, exhaustion: WAKE_EXHAUSTION, food_cost: 1, effects: false })
    } else if food > 1 {
        Some(SleepReward {
            heal: 1.0,
            exhaustion: WAKE_EXHAUSTION / 4.0,
            food_cost: 1,
            effects: false,
        })
    } else {
        None
    }
}

/// The health and the food of a player after a reward: `(health, (level, saturation,
/// exhaustion))`. The health is not capped here: the game caps it at the maximum
/// (`native_set_player_status`). The exhaustion stops at 40 and the level at 0 (vanilla
/// `FoodData.addExhaustion` and `setFoodLevel`). The saturation does not change.
pub fn after_reward(reward: &SleepReward, health: f32, food: (i32, f32, f32)) -> (f32, (i32, f32, f32)) {
    let (level, saturation, exhaustion) = food;
    (
        health + reward.heal,
        (
            (level - reward.food_cost).clamp(0, FOOD_MAX),
            saturation,
            (exhaustion + reward.exhaustion).min(EXHAUSTION_CAP),
        ),
    )
}

// ---------------------------------------------------------------------------------------------
// C14: hide the coordinates in survival (DisableDebugInfo.java)
// ---------------------------------------------------------------------------------------------

/// The game rule that Horizon's debug screen reads to hide the coordinates
/// (`reduced_debug_info`, horizon-client-core gamerules.rs:109).
pub const REDUCED_DEBUG_INFO: &str = "reduced_debug_info";

/// The command that sets the rule (the console runs it with `native_run_command`).
pub fn rule_command(value: bool) -> String {
    format!("gamerule {REDUCED_DEBUG_INFO} {value}")
}

/// What the area must do with the rule after a tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugAction {
    /// Nothing: the rule has the value it needs.
    Keep,
    /// Set the rule to this value.
    Set(bool),
}

/// The state of the forcing (DisableDebugInfo.java:25-26): whether the area forces the rule, and
/// the value that the rule had before. The area forgets it when the world changes
/// (DisableDebugInfo.java:37-38).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DebugForce {
    forcing: bool,
    before: bool,
}

impl DebugForce {
    /// A state that forces nothing.
    pub const fn new() -> DebugForce {
        DebugForce { forcing: false, before: false }
    }

    /// The state of the area forgets the world that it forced (a new world: the old value is
    /// not this world's).
    pub fn forget(&mut self) {
        *self = DebugForce::new();
    }

    /// One tick (DisableDebugInfo.java:43-58). `rule_on` is the world rule
    /// `remove_coordinates_in_survival`. `any_survival` is true when a player is in survival.
    /// `current` is the value of `reduced_debug_info` now.
    /// - The rule is on and a player is in survival: remember the value before (once), then set
    ///   the rule to true when it is not.
    /// - Else, when the area forced the rule: stop forcing and put the value before back. Java
    ///   sets it without a test. A set to the same value changes nothing, so this skips it.
    pub fn step(&mut self, rule_on: bool, any_survival: bool, current: bool) -> DebugAction {
        if rule_on && any_survival {
            if !self.forcing {
                self.forcing = true;
                self.before = current;
            }
            if current { DebugAction::Keep } else { DebugAction::Set(true) }
        } else if self.forcing {
            self.forcing = false;
            if self.before == current { DebugAction::Keep } else { DebugAction::Set(self.before) }
        } else {
            DebugAction::Keep
        }
    }
}

/// The value of the rule for `step`: what the game says, else the last value that the area set,
/// else false (a new world has the rule off). GAPS.md C14: without the read helper the
/// restore sets false.
pub fn current_rule(read: Option<bool>, last_set: Option<bool>) -> bool {
    read.or(last_set).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{
        DORMANT_TOTEM, ECHO_HEART, ETERNAL_AXE, ETERNAL_PICKAXE, ETERNAL_SWORD, NAVIGATOR_AMULET,
        SLEEPING_TOTEM,
    };

    fn view<'a>(stack: &'a (&'static str, u32)) -> (&'a str, u32) {
        (stack.0, stack.1)
    }

    // ---- C11 ----

    #[test]
    fn eternal_stacks_leave_the_drops_in_order() {
        let mut drops = vec![
            ("minecraft:dirt", 12),
            (ETERNAL_SWORD, 1),
            (NAVIGATOR_AMULET, 1),
            (TOTEM_OF_ETERNITY, 1),
            ("minecraft:stone", 3),
            (ETERNAL_BUNDLE, 1),
        ];
        let kept = take_eternal(&mut drops, view);
        assert_eq!(kept, vec![(ETERNAL_SWORD, 1), (TOTEM_OF_ETERNITY, 1), (ETERNAL_BUNDLE, 1)]);
        assert_eq!(
            drops,
            vec![("minecraft:dirt", 12), (NAVIGATOR_AMULET, 1), ("minecraft:stone", 3)]
        );
    }

    #[test]
    fn the_six_eternal_items_are_kept_and_the_three_others_drop() {
        let mut drops = vec![
            (ETERNAL_SWORD, 1),
            (ETERNAL_AXE, 1),
            (ETERNAL_PICKAXE, 1),
            (ETERNAL_BUNDLE, 1),
            (TOTEM_OF_ETERNITY, 1),
            (SLEEPING_TOTEM, 1),
            (DORMANT_TOTEM, 1),
            (ECHO_HEART, 1),
            (NAVIGATOR_AMULET, 1),
        ];
        let kept = take_eternal(&mut drops, view);
        assert_eq!(kept.len(), 6);
        assert_eq!(drops, vec![(DORMANT_TOTEM, 1), (ECHO_HEART, 1), (NAVIGATOR_AMULET, 1)]);
    }

    #[test]
    fn an_empty_stack_is_not_kept() {
        let mut drops = vec![(ETERNAL_SWORD, 0), ("minecraft:dirt", 0)];
        let kept = take_eternal(&mut drops, view);
        assert!(kept.is_empty());
        assert_eq!(drops, vec![(ETERNAL_SWORD, 0), ("minecraft:dirt", 0)]);
    }

    #[test]
    fn no_drops_and_no_eternal_stacks_give_nothing() {
        let mut none: Vec<(&'static str, u32)> = Vec::new();
        assert!(take_eternal(&mut none, view).is_empty());
        let mut plain = vec![("minecraft:dirt", 1)];
        assert!(take_eternal(&mut plain, view).is_empty());
        assert_eq!(plain.len(), 1);
    }

    #[test]
    fn the_split_does_not_depend_on_what_other_areas_did_to_the_drops() {
        // Respawn only counts. Tools may change a stack (the wear is in its data, not in its id).
        // Both orders of an extra stack in front leave the same kept list.
        let mut with_extra = vec![("minecraft:dirt", 1), (ETERNAL_AXE, 1)];
        let mut without = vec![(ETERNAL_AXE, 1)];
        assert_eq!(take_eternal(&mut with_extra, view), take_eternal(&mut without, view));
    }

    #[test]
    fn a_death_with_nothing_to_keep_does_not_erase_the_waiting_list() {
        // KeepItemAfterDeath.java:53-54.
        let waiting = vec![ETERNAL_SWORD];
        assert_eq!(keep_after_death(waiting.clone(), Vec::new()), waiting);
        // Stacks that no respawn could give wait with the new ones: nothing is lost.
        assert_eq!(keep_after_death(waiting, vec![ETERNAL_AXE]), vec![ETERNAL_SWORD, ETERNAL_AXE]);
        assert!(keep_after_death(Vec::<&str>::new(), Vec::new()).is_empty());
    }

    #[test]
    fn a_list_in_memory_does_not_outlive_its_world() {
        let player = 7_u128;
        let mut memory: Fallback<u32, &str> = Fallback::new();
        memory.new_world(1);
        memory.put(player, vec![ETERNAL_SWORD]);
        // The same world, tick after tick: the list stays.
        memory.new_world(1);
        assert_eq!(memory.get(player), Some(&vec![ETERNAL_SWORD]));
        // Another world with the same player uuid: nothing waits.
        memory.new_world(2);
        assert_eq!(memory.get(player), None);
        // Back in the first world is a new epoch again: still nothing.
        memory.new_world(1);
        assert_eq!(memory.get(player), None);
    }

    #[test]
    fn an_empty_list_in_memory_hides_nothing_and_clear_removes_it() {
        let mut memory: Fallback<u32, &str> = Fallback::new();
        memory.new_world(1);
        memory.put(1, Vec::new());
        assert_eq!(memory.get(1), Some(&Vec::new()));
        memory.clear(1);
        assert_eq!(memory.get(1), None);
        // Players do not share a list.
        memory.put(2, vec![ETERNAL_AXE]);
        assert_eq!(memory.get(1), None);
    }

    #[test]
    fn the_bundle_in_use_is_the_first_hand_that_holds_something() {
        // An empty main hand bundle lets the off hand bundle act (Java: FAIL, then the off hand).
        assert_eq!(first_loaded([(3, false), (-106, true)]), Some(-106));
        assert_eq!(first_loaded([(3, true), (-106, true)]), Some(3));
        assert_eq!(first_loaded([(3, false), (-106, false)]), None);
        assert_eq!(first_loaded([(3, false)]), None);
    }

    // ---- E11 ----

    #[test]
    fn the_rule_needs_survival_and_the_world_rule() {
        assert!(rule_applies(true, 0));
        assert!(!rule_applies(false, 0));
        assert!(!rule_applies(true, 1));
        assert!(!rule_applies(true, 2));
        assert!(!rule_applies(true, 3));
    }

    #[test]
    fn the_slots_go_in_the_order_of_the_inventory_tick() {
        let order = slot_order();
        assert_eq!(order.len(), 41);
        assert_eq!(order[0], 0);
        assert_eq!(order[35], 35);
        assert_eq!(&order[36..40], &[103, 102, 101, 100]);
        assert_eq!(order[40], -106);
        assert!(earlier_slots(0).is_empty());
        assert_eq!(earlier_slots(3), vec![0, 1, 2]);
        assert_eq!(earlier_slots(-106).len(), 40);
        assert_eq!(earlier_slots(101).last(), Some(&102));
    }

    #[test]
    fn the_first_stack_of_a_type_stays_and_the_later_ones_are_copies() {
        assert!(!is_copy(ETERNAL_SWORD, []));
        assert!(!is_copy(ETERNAL_SWORD, [ETERNAL_AXE, SLEEPING_TOTEM]));
        assert!(is_copy(ETERNAL_SWORD, [ETERNAL_AXE, ETERNAL_SWORD]));
    }

    #[test]
    fn the_two_totems_are_two_types() {
        assert!(!is_copy(TOTEM_OF_ETERNITY, [SLEEPING_TOTEM]));
        assert!(!is_copy(SLEEPING_TOTEM, [TOTEM_OF_ETERNITY]));
    }

    #[test]
    fn a_copy_drops_whole_and_loses_one() {
        assert_eq!(copy_drop(1), (1, 0));
        assert_eq!(copy_drop(3), (3, 2));
        assert_eq!(copy_drop(0), (0, 0));
    }

    #[test]
    fn a_pickup_is_refused_when_the_type_is_carried() {
        let carried = [ETERNAL_SWORD, "minecraft:dirt"];
        assert!(blocks_pickup(ETERNAL_SWORD, true, Some(0), carried));
        assert!(!blocks_pickup(ETERNAL_AXE, true, Some(0), carried));
        assert!(!blocks_pickup(ETERNAL_SWORD, true, Some(0), []));
        // Not an Eternal Item.
        assert!(!blocks_pickup("minecraft:dirt", true, Some(0), ["minecraft:dirt"]));
        assert!(!blocks_pickup(NAVIGATOR_AMULET, true, Some(0), [NAVIGATOR_AMULET]));
    }

    #[test]
    fn a_pickup_follows_the_rule_except_for_the_totem_of_eternity() {
        let sword = [ETERNAL_SWORD];
        assert!(!blocks_pickup(ETERNAL_SWORD, false, Some(0), sword));
        assert!(!blocks_pickup(ETERNAL_SWORD, true, Some(1), sword));
        assert!(!blocks_pickup(ETERNAL_SWORD, true, None, sword));
        let totem = [TOTEM_OF_ETERNITY];
        assert!(blocks_pickup(TOTEM_OF_ETERNITY, false, Some(1), totem));
        assert!(blocks_pickup(TOTEM_OF_ETERNITY, false, None, totem));
        // The Dormant Totem is not the Totem of Eternity: it follows the rule.
        assert!(!blocks_pickup(SLEEPING_TOTEM, false, Some(0), [SLEEPING_TOTEM]));
        assert!(blocks_pickup(SLEEPING_TOTEM, true, Some(0), [SLEEPING_TOTEM]));
    }

    // ---- drops from a player ----

    fn constant(value: f64) -> impl FnMut() -> f64 {
        move || value
    }

    #[test]
    fn a_dropped_stack_starts_below_the_eyes() {
        let at = drop_position([10.0, 64.0, -3.0], 1.62);
        assert_eq!(at[0], 10.0);
        assert!((at[1] - 65.32).abs() < 1e-9);
        assert_eq!(at[2], -3.0);
        assert_eq!(DROP_PICKUP_DELAY, 40);
    }

    #[test]
    fn a_stack_dropped_along_the_view_goes_forward() {
        // Yaw 0 looks to +z. The draws are 0.5: direction pi, power 0.01, spread 0.
        let v = drop_velocity(0.0, 0.0, false, &mut constant(0.5));
        assert!((v[0] - -0.01).abs() < 1e-9, "{v:?}");
        assert!((v[1] - 0.1).abs() < 1e-9, "{v:?}");
        assert!((v[2] - 0.3).abs() < 1e-9, "{v:?}");
        // Yaw 90 looks to -x.
        let v = drop_velocity(90.0, 0.0, false, &mut constant(0.5));
        assert!((v[0] - -0.31).abs() < 1e-9, "{v:?}");
        assert!(v[2].abs() < 1e-9, "{v:?}");
        // Looking straight up (pitch -90) throws the stack up.
        let v = drop_velocity(0.0, -90.0, false, &mut constant(0.5));
        assert!((v[1] - 0.4).abs() < 1e-9, "{v:?}");
    }

    #[test]
    fn a_stack_dropped_randomly_goes_in_any_direction() {
        let v = drop_velocity(0.0, 0.0, true, &mut constant(0.5));
        assert!(v[0].abs() < 1e-9, "{v:?}");
        assert_eq!(v[1], 0.2);
        assert!((v[2] - -0.25).abs() < 1e-9, "{v:?}");
        let v = drop_velocity(0.0, 0.0, true, &mut constant(0.0));
        assert_eq!(v, [0.0, 0.2, 0.0]);
    }

    // ---- E10 ----

    #[test]
    fn an_eternal_item_on_the_ground_gets_the_longest_extra_life() {
        assert_eq!(EXTRA_LIFE_TICKS, 32766);
        assert_eq!(extra_life(ETERNAL_SWORD, 0), 32766);
        assert_eq!(extra_life(SLEEPING_TOTEM, 100), 32766);
        // Another mod may have added more than the cap: nothing is taken away.
        assert_eq!(extra_life(ETERNAL_AXE, 40000), 40000);
        assert_eq!(extra_life("minecraft:dirt", 0), 0);
        assert_eq!(extra_life(NAVIGATOR_AMULET, 0), 0);
    }

    #[test]
    fn an_eternal_item_on_the_ground_gets_the_largest_health_short() {
        assert_eq!(item_health(ETERNAL_BUNDLE), Some(32767));
        assert_eq!(item_health(DORMANT_TOTEM), None);
        assert_eq!(item_health("minecraft:dirt"), None);
    }

    // ---- E12 ----

    #[test]
    fn the_bundle_refuses_eternal_items_and_ender_chests() {
        for id in crate::ids::ETERNAL_ITEMS {
            assert!(bundle_refuses(id), "{id}");
        }
        assert!(bundle_refuses("minecraft:ender_chest"));
        assert!(!bundle_refuses("minecraft:chest"));
        assert!(!bundle_refuses(NAVIGATOR_AMULET));
        assert!(!bundle_refuses(ECHO_HEART));
        assert!(!bundle_refuses("minecraft:dirt"));
    }

    #[test]
    fn only_the_eternal_bundle_is_the_bundle() {
        assert!(is_bundle(ETERNAL_BUNDLE));
        assert!(!is_bundle("minecraft:bundle"));
    }

    #[test]
    fn the_bundle_sound_has_the_java_volume_and_pitch() {
        assert_eq!(BUNDLE_SOUND_VOLUME, 0.8);
        assert!((bundle_sound_pitch(0.0) - 0.8).abs() < 1e-6);
        assert!((bundle_sound_pitch(0.5) - 1.0).abs() < 1e-6);
        assert!((bundle_sound_pitch(1.0) - 1.2).abs() < 1e-6);
        assert_eq!(BUNDLE_DROP_SOUND, "minecraft:item.bundle.drop_contents");
        assert_eq!(PLAYER_SOUND_SOURCE, 7);
    }

    // ---- C13 ----

    #[test]
    fn a_wake_up_at_food_15_or_more_heals_6_and_costs_4() {
        let r = sleep_reward(15, 12.0).unwrap();
        assert_eq!(r, SleepReward { heal: 6.0, exhaustion: 4.0, food_cost: 4, effects: false });
        let r = sleep_reward(20, 20.0).unwrap();
        assert_eq!(r, SleepReward { heal: 6.0, exhaustion: 4.0, food_cost: 4, effects: true });
        // The effects need a health of 20 or more before the heal.
        assert!(!sleep_reward(20, 19.5).unwrap().effects);
        assert!(sleep_reward(15, 24.0).unwrap().effects);
    }

    #[test]
    fn a_wake_up_at_lower_food_gives_less() {
        assert_eq!(
            sleep_reward(14, 20.0).unwrap(),
            SleepReward { heal: 4.0, exhaustion: 4.0, food_cost: 2, effects: false }
        );
        assert_eq!(sleep_reward(10, 20.0).unwrap().heal, 4.0);
        assert_eq!(
            sleep_reward(9, 20.0).unwrap(),
            SleepReward { heal: 3.0, exhaustion: 4.0, food_cost: 1, effects: false }
        );
        assert_eq!(sleep_reward(5, 20.0).unwrap().heal, 3.0);
        assert_eq!(
            sleep_reward(4, 20.0).unwrap(),
            SleepReward { heal: 1.0, exhaustion: 1.0, food_cost: 1, effects: false }
        );
        assert_eq!(sleep_reward(2, 20.0).unwrap().heal, 1.0);
    }

    #[test]
    fn a_wake_up_at_food_1_or_0_gives_nothing() {
        assert_eq!(sleep_reward(1, 5.0), None);
        assert_eq!(sleep_reward(0, 5.0), None);
        assert_eq!(sleep_reward(-3, 5.0), None);
    }

    #[test]
    fn the_effects_are_haste_ii_and_resistance_ii_for_3600_ticks() {
        assert_eq!(WAKE_EFFECTS, ["minecraft:haste", "minecraft:resistance"]);
        assert_eq!(WAKE_EFFECT_TICKS, 3600);
        assert_eq!(WAKE_EFFECT_AMPLIFIER, 1);
    }

    #[test]
    fn a_reward_changes_the_health_and_the_food() {
        let r = sleep_reward(18, 10.0).unwrap();
        let (health, (level, saturation, exhaustion)) = after_reward(&r, 10.0, (18, 3.5, 1.0));
        assert_eq!(health, 16.0);
        assert_eq!((level, saturation, exhaustion), (14, 3.5, 5.0));
        // The exhaustion stops at 40 and the food at 0.
        let r = sleep_reward(2, 10.0).unwrap();
        let (_, food) = after_reward(&r, 10.0, (2, 0.0, 39.9));
        assert_eq!(food, (1, 0.0, 40.0));
        let r = SleepReward { heal: 0.0, exhaustion: 0.0, food_cost: 4, effects: false };
        assert_eq!(after_reward(&r, 1.0, (3, 0.0, 0.0)).1.0, 0);
    }

    // ---- C14 ----

    #[test]
    fn the_rule_command_names_the_vanilla_rule() {
        assert_eq!(rule_command(true), "gamerule reduced_debug_info true");
        assert_eq!(rule_command(false), "gamerule reduced_debug_info false");
    }

    #[test]
    fn survival_forces_the_rule_once_and_a_later_tick_changes_nothing() {
        let mut f = DebugForce::new();
        assert_eq!(f.step(true, true, false), DebugAction::Set(true));
        // The game applied it: the next ticks keep it.
        assert_eq!(f.step(true, true, true), DebugAction::Keep);
        assert_eq!(f.step(true, true, true), DebugAction::Keep);
    }

    #[test]
    fn an_operator_who_turns_the_rule_off_is_forced_again() {
        let mut f = DebugForce::new();
        f.step(true, true, false);
        assert_eq!(f.step(true, true, false), DebugAction::Set(true));
    }

    #[test]
    fn the_value_before_comes_back_when_nobody_is_in_survival() {
        let mut f = DebugForce::new();
        assert_eq!(f.step(true, true, false), DebugAction::Set(true));
        assert_eq!(f.step(true, false, true), DebugAction::Set(false));
        // Restored once: nothing more.
        assert_eq!(f.step(true, false, false), DebugAction::Keep);
    }

    #[test]
    fn the_value_before_comes_back_when_the_world_rule_goes_off() {
        let mut f = DebugForce::new();
        f.step(true, true, false);
        assert_eq!(f.step(false, true, true), DebugAction::Set(false));
    }

    #[test]
    fn an_operator_value_of_true_stays_true() {
        // DisableDebugInfo.java:48-51: the value before is remembered once.
        let mut f = DebugForce::new();
        assert_eq!(f.step(true, true, true), DebugAction::Keep);
        assert_eq!(f.step(true, false, true), DebugAction::Keep);
    }

    #[test]
    fn nothing_is_forced_when_the_rule_is_off_or_nobody_is_in_survival() {
        let mut f = DebugForce::new();
        assert_eq!(f.step(false, true, false), DebugAction::Keep);
        assert_eq!(f.step(true, false, false), DebugAction::Keep);
        assert_eq!(f.step(false, false, true), DebugAction::Keep);
    }

    #[test]
    fn a_new_world_forgets_the_old_value() {
        let mut f = DebugForce::new();
        f.step(true, true, true);
        f.forget();
        assert_eq!(f, DebugForce::new());
        // The old "before" (true) is gone: the new world remembers its own.
        assert_eq!(f.step(true, true, false), DebugAction::Set(true));
        assert_eq!(f.step(true, false, true), DebugAction::Set(false));
    }

    #[test]
    fn the_current_value_is_read_else_assumed() {
        assert!(current_rule(Some(true), Some(false)));
        assert!(!current_rule(Some(false), Some(true)));
        assert!(current_rule(None, Some(true)));
        assert!(!current_rule(None, Some(false)));
        assert!(!current_rule(None, None));
    }
}
