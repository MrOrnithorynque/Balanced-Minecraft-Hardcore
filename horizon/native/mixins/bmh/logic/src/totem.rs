// Area totem (owner: the totem agent): the pure logic of the Totem of Eternity.
// It holds the save from death and its effects (T2, T3, T4), the tooltip lines of the four totem items
// (T1), the rule "one Totem of Eternity per player, per container" (T11, T12, T13), the rows of the
// anvil (T6 awakening, E6 gold block repair) and the Totem of Undying without Fire Resistance (T5).
// Java reference: handlers/TotemOfEternityHandler, handlers/TotemAwakeningHandler, mixin/A_*,
// mixin/B_AnvilMenuMixin, mixin/I_*, items/TotemOfEternityItem, items/HintItem.
// The module `bmh_totem.rs` reads the game and carries out the answers of this file.

use crate::ids;
use crate::respawn::BYPASSES_INVULNERABILITY;

// ---------------------------------------------------------------------------------------------
// T2, T3, T4: the save from death
// ---------------------------------------------------------------------------------------------

/// The advancement that a save awards (TotemOfEternityHandler.java:50, `BmhAdvancements.award`).
pub const SAVED_ADVANCEMENT: &str = "eternity/saved";

/// The health that a save leaves (TotemOfEternityHandler.java:42, `player.setHealth(1.0F)`).
pub const SAVE_HEALTH: f32 = 1.0;

/// One effect of a save.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveEffect {
    pub id: &'static str,
    /// 0 for level I, as in `MobEffectInstance`.
    pub amplifier: u8,
    pub duration: i32,
}

/// The effects of a save, in the order Java adds them (TotemOfEternityHandler.java:44-46). The
/// three-argument `MobEffectInstance` constructor sets `ambient` to false and shows particles and
/// icon. `removeAllEffects` runs before (TotemOfEternityHandler.java:43); the module replaces the
/// whole list. Fire Resistance stays: only the Totem of Undying loses it (A_LivingEntityTotemMixin.java:30).
pub const SAVE_EFFECTS: [SaveEffect; 3] = [
    SaveEffect { id: "minecraft:regeneration", amplifier: 1, duration: 900 },
    SaveEffect { id: "minecraft:absorption", amplifier: 1, duration: 100 },
    SaveEffect { id: "minecraft:fire_resistance", amplifier: 0, duration: 800 },
];

/// The sound of the save. Vanilla plays it with entity event 35 (`ClientPacketListener.handleEntityEvent`,
/// `SoundEvents.TOTEM_USE`, volume 1.0, pitch 1.0). GAPS T4 ships the sound and the particles only.
pub const SAVE_SOUND: &str = "minecraft:item.totem.use";

/// The sound volume and pitch of the save (vanilla `playLocalSound(..., 1.0F, 1.0F, false)`).
pub const SAVE_SOUND_VOLUME: f32 = 1.0;
pub const SAVE_SOUND_PITCH: f32 = 1.0;

/// How many `totem_of_undying` particles the save sends. Vanilla runs a 30 tick emitter of 16
/// particles a tick; the mod sends one burst (GAPS T4, a chosen number, not a Java value).
pub const SAVE_PARTICLES: i32 = 100;

/// The half spread of the burst around the middle of the player, and its speed (chosen numbers).
pub const SAVE_PARTICLE_SPREAD: [f64; 3] = [0.3, 0.9, 0.3];
pub const SAVE_PARTICLE_SPEED: f64 = 0.5;

/// The burst starts about the middle of the body: the feet plus this height (a player is 1.8 high).
pub const SAVE_PARTICLE_HEIGHT: f64 = 0.9;

/// True when the Totem of Eternity can save a player from this damage. Damage that bypasses
/// invulnerability (the void, `/kill`) still kills, as with vanilla's totem
/// (A_LivingEntityTotemMixin.java:41, `!source.is(DamageTypeTags.BYPASSES_INVULNERABILITY)`). The
/// game gives only the damage type, so the list of the tag is written in `respawn.rs`.
pub fn can_save_from(damage_type: &str) -> bool {
    !BYPASSES_INVULNERABILITY.contains(&damage_type)
}

/// True for the id of a Totem of Eternity (`instanceof TotemOfEternityItem`).
pub fn is_totem(item: &str) -> bool {
    item == ids::TOTEM_OF_ETERNITY
}

/// What the module knows of a player from his last tick: which entity he is, and whether he
/// carried a Totem of Eternity. A death hook has only an entity id and cannot read a stack, so the
/// answer comes from this note (GAPS T2, verify-2 row 1a).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Carrier {
    uuid: u128,
    entity: u32,
    has_totem: bool,
}

/// The book of saves: who carries a totem, and which saves wait for their swap.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SaveBook {
    carriers: Vec<Carrier>,
    pending: Vec<u128>,
}

impl SaveBook {
    pub const fn new() -> SaveBook {
        SaveBook { carriers: Vec::new(), pending: Vec::new() }
    }

    /// Notes what a player tick saw: the entity id of the player and whether he carries a Totem of
    /// Eternity. An entity id of 0 or less is not known and never matches a death.
    pub fn note(&mut self, uuid: u128, entity: i32, has_totem: bool) {
        let entity = u32::try_from(entity).unwrap_or(0);
        match self.carriers.iter_mut().find(|c| c.uuid == uuid) {
            Some(carrier) => {
                carrier.entity = entity;
                carrier.has_totem = has_totem;
            }
            None => self.carriers.push(Carrier { uuid, entity, has_totem }),
        }
    }

    /// A player is about to die from `damage_type`. True when the totem saves him: the module cancels
    /// the death and swaps the totem at his next tick. A second death before the swap is a real
    /// death: the first save spent the totem (TotemOfEternityHandler.java:30-41 swaps at once).
    pub fn on_death(&mut self, entity: u32, damage_type: &str) -> bool {
        let Some(carrier) = self.carriers.iter_mut().find(|c| c.entity == entity && entity != 0) else {
            return false;
        };
        let uuid = carrier.uuid;
        if self.pending.contains(&uuid) {
            self.pending.retain(|p| *p != uuid);
            return false;
        }
        if !carrier.has_totem || !can_save_from(damage_type) {
            return false;
        }
        carrier.has_totem = false;
        self.pending.push(uuid);
        true
    }

    /// True once when a save of this player waits for its swap. The swap runs in his next tick.
    pub fn take_pending(&mut self, uuid: u128) -> bool {
        let waiting = self.pending.contains(&uuid);
        self.pending.retain(|p| *p != uuid);
        waiting
    }

    /// Forgets a player (he left the world).
    pub fn forget(&mut self, uuid: u128) {
        self.carriers.retain(|c| c.uuid != uuid);
        self.pending.retain(|p| *p != uuid);
    }

    /// Forgets everyone (the world changed).
    pub fn clear(&mut self) {
        self.carriers.clear();
        self.pending.clear();
    }
}

// ---------------------------------------------------------------------------------------------
// T11, T12, T13: one Totem of Eternity per player and per container
// ---------------------------------------------------------------------------------------------

/// The first slot that holds a Totem of Eternity, in the order of `slots` (TotemOfEternityHandler.java:27-39,
/// the first stack in container order saves the player). `slots` pairs a slot number with its item id.
pub fn first_totem_slot<'a>(slots: impl IntoIterator<Item = (i32, &'a str)>) -> Option<i32> {
    slots.into_iter().find(|(_, item)| is_totem(item)).map(|(slot, _)| slot)
}

/// The slots that hold a Totem of Eternity, in the order of `slots`.
pub fn totem_slots<'a>(slots: impl IntoIterator<Item = (i32, &'a str)>) -> Vec<i32> {
    slots.into_iter().filter(|(_, item)| is_totem(item)).map(|(slot, _)| slot).collect()
}

/// The slots whose totem must leave: every one after the first (TotemOfEternityHandler.java:60-76,
/// `dropExtraTotems`: the first stays, the others drop). `totem_slots` is in container order.
pub fn extra_slots(totem_slots: &[i32]) -> &[i32] {
    totem_slots.get(1..).unwrap_or(&[])
}

/// True when a player must not pick up this item: a Totem of Eternity while he carries one
/// (EternalItemHandler.java:60-77, `shouldBlockPickup`; the totem is refused whatever the game
/// rule says and whatever the game mode is).
pub fn refuses_pickup<'a>(item: &str, carried: impl IntoIterator<Item = &'a str>) -> bool {
    is_totem(item) && carried.into_iter().any(is_totem)
}

/// The slots of a container whose totem must leave (I_SlotTotemMixin.java:23-29 and
/// TotemOfEternityHandler.holdsAnotherTotem, done after the fact). `before` are the slots that held a
/// totem when the player opened the container; `now` are the slots that hold one. The Java mod
/// refuses a totem that would join another one. A totem in a slot of `before` was already there. A
/// totem in another slot is new: it leaves when another totem stays. If no totem was there before,
/// the first new one stays. A pair that was already in the container is not touched. `now` is in
/// slot order. The answer is in slot order too.
pub fn container_extras(before: &[usize], now: &[usize]) -> Vec<usize> {
    let mut stays = now.iter().any(|slot| before.contains(slot));
    let mut leave = Vec::new();
    for slot in now.iter().filter(|slot| !before.contains(slot)) {
        if stays {
            leave.push(*slot);
        } else {
            stays = true;
        }
    }
    leave
}

// ---------------------------------------------------------------------------------------------
// T12, T14: the slot rule of the menus, before the click lands
// ---------------------------------------------------------------------------------------------

/// What one click of a container menu asks of the rule (`ClickType` ids: 0 pickup, 1 quick move,
/// 2 swap). `slot` is the window slot: the container's slots first, then the player's 36.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Click {
    pub slot: i32,
    pub mode: i32,
    /// The cursor holds a Totem of Eternity.
    pub cursor: bool,
    /// The clicked slot, when it is a player slot, holds one (a quick move takes it from there).
    pub slot_stack: bool,
    /// The hotbar slot of a number key (or the off hand) holds one (a swap puts it in the slot).
    pub key_stack: bool,
}

/// True when `container` (true = the slot holds a totem) holds a totem in a slot other than `slot`.
/// A swap inside one slot moves the same stack, so that slot does not count
/// (TotemOfEternityHandler.java:90-106, `i != slot`).
pub fn holds_another(container: &[bool], slot: usize) -> bool {
    container.iter().enumerate().any(|(i, totem)| *totem && i != slot)
}

/// True when vanilla's click would put a second Totem of Eternity in the container, so that
/// `Slot.mayPlace` of the Java mod (I_SlotTotemMixin.java:21-28) would refuse it. The hook cancels
/// the click then: nothing moves, as when `mayPlace` is false.
/// - Pickup (left or right button): the cursor totem goes into a container slot.
/// - Quick move: a player slot with a totem goes to the container.
/// - Swap (number key, off hand key): the hotbar totem goes into the container slot.
/// Other modes (clone, throw, pick up all) place nothing.
pub fn click_refused(container: &[bool], click: &Click) -> bool {
    let Ok(slot) = usize::try_from(click.slot) else {
        return false;
    };
    let inside = slot < container.len();
    match click.mode {
        0 => inside && click.cursor && holds_another(container, slot),
        1 => !inside && click.slot_stack && container.iter().any(|totem| *totem),
        2 => inside && click.key_stack && holds_another(container, slot),
        _ => false,
    }
}

/// True when a drag (quick craft) of a cursor totem would reach a container slot that refuses it.
/// Vanilla skips a refused slot and fills the others; here the whole drag stops, which is the
/// same result for a stack of one except for a drag over the container and the inventory together.
pub fn drag_refused(container: &[bool], slots: &[i32], cursor: bool) -> bool {
    cursor
        && slots
            .iter()
            .any(|slot| usize::try_from(*slot).is_ok_and(|slot| slot < container.len() && holds_another(container, slot)))
}

/// How many totems the periodic scan takes out of a container (T13 droppers, T14 vehicles, any
/// path that has no hook). `previous` is the count of the last scan, None when the container was
/// not known then (a chunk that just loaded: a pair that was already there is left alone, as in
/// Java). A container that grows past one loses the new ones.
pub fn scan_ejects(previous: Option<usize>, now: usize) -> usize {
    match previous {
        Some(previous) if now >= 2 && now > previous => now - previous.max(1),
        _ => 0,
    }
}

// ---------------------------------------------------------------------------------------------
// T1: the tooltip lines
// ---------------------------------------------------------------------------------------------

/// `ChatFormatting.GRAY` (TotemOfEternityItem.java:36, HintItem.java:28).
pub const GRAY: u32 = 0xAA_AAAA;
/// `ChatFormatting.GOLD` (EternalTooltips.java:24).
pub const GOLD: u32 = 0xFF_AA00;

/// Lang keys of the totem items (the keys of the Java mod).
pub const KEY_ETERNITY: &str = "tooltip.bmh.eternity";
pub const KEY_SAVE: &str = "tooltip.bmh.totem_of_eternity_save";
pub const KEY_HEAVY: &str = "tooltip.bmh.heavy_with_eternity";
pub const KEY_SLEEPING: &str = "tooltip.bmh.sleeping_totem";
pub const KEY_HOLLOW: &str = "tooltip.bmh.dormant_totem";
pub const KEY_ECHO_HEART: &str = "tooltip.bmh.echo_heart";

/// The levels that the moon ritual drains (MoonRitualHandler.java:36, `RITUAL_LEVELS`). The tooltip of
/// the Dormant Totem names them.
pub const RITUAL_LEVELS: u32 = 30;

/// One line of a tooltip: the lang key, the color, and the number that fills `%s` when there is one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line {
    pub key: &'static str,
    pub color: u32,
    pub number: Option<u32>,
}

const fn line(key: &'static str, color: u32) -> Line {
    Line { key, color, number: None }
}

/// The lines that an item adds under its name, in the order of the Java `appendHoverText`.
/// Empty for any other item.
/// - Totem of Eternity (TotemOfEternityItem.java:34-38): eternity, save, heavy.
/// - Dormant Totem, id `sleeping_totem` (DormantTotemOfEternityItem.java:116-123): eternity, the
///   ritual with its levels, heavy.
/// - Hollow Totem, id `dormant_totem`, and Echo Heart (HintItem.java:25-30): one gray hint.
pub fn tooltip(item: &str) -> Vec<Line> {
    match item {
        ids::TOTEM_OF_ETERNITY => vec![line(KEY_ETERNITY, GRAY), line(KEY_SAVE, GRAY), line(KEY_HEAVY, GOLD)],
        ids::SLEEPING_TOTEM => vec![
            line(KEY_ETERNITY, GRAY),
            Line { key: KEY_SLEEPING, color: GRAY, number: Some(RITUAL_LEVELS) },
            line(KEY_HEAVY, GOLD),
        ],
        ids::DORMANT_TOTEM => vec![line(KEY_HOLLOW, GRAY)],
        ids::ECHO_HEART => vec![line(KEY_ECHO_HEART, GRAY)],
        _ => Vec::new(),
    }
}

// ---------------------------------------------------------------------------------------------
// T6, E6: the anvil
// ---------------------------------------------------------------------------------------------

/// The cost of the awakening: the most a survival anvil allows, 40 and above is "Too Expensive"
/// (TotemAwakeningHandler.java:12, `COST`).
pub const AWAKEN_COST: i32 = 39;

/// How many Echo Hearts the take uses (B_AnvilMenuMixin.java:76, `repairItemCountCost = 1`).
pub const AWAKEN_MATERIAL_COST: i32 = 1;

/// A gold block repair of an Eternal tool always costs this many levels (EternalWear.java:27,
/// `GOLD_REPAIR_COST`).
pub const GOLD_REPAIR_COST: i32 = 15;

/// The item that repairs an Eternal tool (EternalWear.java:71, `Items.GOLD_BLOCK`).
pub const GOLD_BLOCK: &str = "minecraft:gold_block";

/// The durability of an Eternal tool (BMHModTierInit.java:21, `new ToolMaterial(..., 33000, ...)`).
pub const ETERNAL_TOOL_MAX_DAMAGE: u32 = 33_000;

/// Vanilla `AnvilMenu.createResult`: in survival, a cost of 40 or more gives no result ("Too
/// Expensive"). For a gold block repair, only the prior work penalty can push the cost that high.
pub const TOO_EXPENSIVE: i32 = 40;

/// A gold block repair: the damage that is left, and how many gold blocks it uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GoldRepair {
    pub damage: u32,
    pub blocks: u32,
}

/// The repair loop of vanilla `AnvilMenu.createResult`: each gold block removes a quarter of the
/// durability, up to the damage. Java runs it with the prior work penalty put aside
/// (B_AnvilMenuMixin.java:82-90), so the loop alone gives the result. None when there is nothing to
/// repair or no block (vanilla: no result).
pub fn gold_repair(damage: u32, blocks: u32) -> Option<GoldRepair> {
    let quarter = ETERNAL_TOOL_MAX_DAMAGE / 4;
    let mut damage = damage;
    let mut repair = damage.min(quarter);
    if repair == 0 {
        return None;
    }
    let mut used = 0;
    while repair > 0 && used < blocks {
        damage = damage.saturating_sub(repair);
        used += 1;
        repair = damage.min(quarter);
    }
    (used > 0).then_some(GoldRepair { damage, blocks: used })
}

/// True when Java's vanilla run still says "Too Expensive" with the tool's penalty put aside: the
/// cost is then the penalty of the gold blocks, plus one level a block used, plus one level for a
/// rename (vanilla `AnvilMenu.createResult`, `tax + price`). Plain gold blocks have no penalty, so
/// this needs blocks that a command gave a penalty.
pub fn gold_repair_refused(blocks_penalty: u32, blocks: u32, renamed: bool) -> bool {
    let cost = u64::from(blocks_penalty) + u64::from(blocks) + u64::from(renamed);
    cost >= TOO_EXPENSIVE as u64
}

/// What the rename box of an anvil does to the name of the result (vanilla `AnvilMenu.createResult`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rename {
    /// The result keeps the name of the input.
    Keep,
    /// The result gets the text of the box as its custom name.
    Set,
    /// The result loses its custom name.
    Clear,
}

/// The rename of an anvil. `text` is the text of the box. `hover` is the name the input shows when the
/// game knows it: its custom name, else its `item_name` component. None means the default name of the
/// item. A vanilla client sends an empty text when the box holds that default name
/// (`AnvilScreen.onNameChanged`), so a text that is not blank is then a new name.
pub fn anvil_rename(text: Option<&str>, hover: Option<&str>, has_custom_name: bool) -> Rename {
    match text {
        Some(text) if !text.trim().is_empty() => {
            if hover == Some(text) {
                Rename::Keep
            } else {
                Rename::Set
            }
        }
        _ if has_custom_name => Rename::Clear,
        _ => Rename::Keep,
    }
}

/// What an anvil does with two stacks, when the mod has a say.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anvil {
    /// A Hollow Totem and an Echo Heart give a Totem of Eternity.
    Awaken,
    /// An Eternal tool and a gold block: a repair for a flat cost that keeps the prior work penalty.
    GoldRepair,
}

/// True for an Eternal sword, axe or pickaxe (EternalWear.java:32-37, `isEternalTool`).
pub fn is_eternal_tool(item: &str) -> bool {
    matches!(item, ids::ETERNAL_SWORD | ids::ETERNAL_AXE | ids::ETERNAL_PICKAXE)
}

/// The case of the anvil for the item in the left slot and the item in the right slot
/// (TotemAwakeningHandler.java:17-22 and EternalWear.java:69-72). Only the Hollow Totem wakes at
/// the anvil: the Dormant Totem wakes in the moon ritual.
pub fn anvil_case(left: &str, right: Option<&str>) -> Option<Anvil> {
    let right = right?;
    if left == ids::DORMANT_TOTEM && right == ids::ECHO_HEART {
        return Some(Anvil::Awaken);
    }
    if is_eternal_tool(left) && right == GOLD_BLOCK {
        return Some(Anvil::GoldRepair);
    }
    None
}

// ---------------------------------------------------------------------------------------------
// T2: the save from a hand (route A, docs/horizon/solutions/totem-save.md)
// ---------------------------------------------------------------------------------------------

/// The Totem of Undying. Java lets vanilla's hand check run first (A_LivingEntityTotemMixin.java:46),
/// so a Totem of Undying in a hand saves before the Totem of Eternity.
pub const TOTEM_OF_UNDYING: &str = "minecraft:totem_of_undying";

/// The off hand in the numbering of `native_carried_stack`.
pub const CARRIED_OFF_HAND: i32 = -106;

/// The off hand in the numbering of `native_set_inventory_slot` (`NativeInventoryKind::Player`).
pub const INVENTORY_OFF_HAND: usize = 40;

/// What the mod does when Horizon's own hand check saves a player. The game spends the main hand
/// when it has `death_protection`, else the off hand (`Game::apply_totem`, game/totem.rs:86-93).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandSave {
    /// No Totem of Eternity is spent: nothing to do.
    Nothing,
    /// The main hand holds a Totem of Eternity and the off hand a Totem of Undying. The mod swaps
    /// the two hands before the use-up, so the Totem of Undying saves first, as in Java. After the
    /// tick, the Totem of Eternity goes back to the main hand.
    Swap,
    /// The game spends the Totem of Eternity of this hand. After the tick, the Dormant Totem goes in
    /// the emptied slot. `carried` is the slot for `native_carried_stack`, `inventory` the slot for
    /// `native_set_inventory_slot`.
    Eternity { carried: i32, inventory: usize },
}

/// The answer for the two hands of a player that the game saves. `main` and `off` are the item ids
/// of the hands (None when empty). `selected` is the selected hotbar slot (0 to 8). Only the Totem
/// of Undying and the Totem of Eternity carry `death_protection` in this mod.
pub fn hand_save(main: Option<&str>, off: Option<&str>, selected: u8) -> HandSave {
    match (main, off) {
        (Some(TOTEM_OF_UNDYING), _) => HandSave::Nothing,
        (Some(m), Some(TOTEM_OF_UNDYING)) if is_totem(m) => HandSave::Swap,
        (Some(m), _) if is_totem(m) => HandSave::Eternity {
            carried: i32::from(selected),
            inventory: usize::from(selected),
        },
        (_, Some(o)) if is_totem(o) => HandSave::Eternity {
            carried: CARRIED_OFF_HAND,
            inventory: INVENTORY_OFF_HAND,
        },
        _ => HandSave::Nothing,
    }
}

// ---------------------------------------------------------------------------------------------
// T5: the Totem of Undying without Fire Resistance
// ---------------------------------------------------------------------------------------------

/// The effect that the Totem of Undying loses (A_LivingEntityTotemMixin.java:30,
/// `player.removeEffect(MobEffects.FIRE_RESISTANCE)`). `bmh_totem_world.rs` writes the same id: its
/// crate cannot reach this one, and the test below pins the value.
pub const UNDYING_LOSES: &str = "minecraft:fire_resistance";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hand_save_undying_in_the_main_hand_saves_first() {
        let t = Some(ids::TOTEM_OF_ETERNITY);
        assert_eq!(hand_save(Some(TOTEM_OF_UNDYING), t, 3), HandSave::Nothing);
        assert_eq!(hand_save(Some(TOTEM_OF_UNDYING), None, 3), HandSave::Nothing);
    }

    #[test]
    fn hand_save_swaps_when_the_undying_is_in_the_off_hand() {
        assert_eq!(
            hand_save(Some(ids::TOTEM_OF_ETERNITY), Some(TOTEM_OF_UNDYING), 2),
            HandSave::Swap
        );
    }

    #[test]
    fn hand_save_spends_the_eternity_of_the_main_hand() {
        assert_eq!(
            hand_save(Some(ids::TOTEM_OF_ETERNITY), None, 4),
            HandSave::Eternity { carried: 4, inventory: 4 }
        );
        assert_eq!(
            hand_save(Some(ids::TOTEM_OF_ETERNITY), Some("minecraft:shield"), 8),
            HandSave::Eternity { carried: 8, inventory: 8 }
        );
    }

    #[test]
    fn hand_save_spends_the_eternity_of_the_off_hand() {
        let off = HandSave::Eternity { carried: -106, inventory: 40 };
        assert_eq!(hand_save(None, Some(ids::TOTEM_OF_ETERNITY), 0), off);
        assert_eq!(hand_save(Some("minecraft:stone_sword"), Some(ids::TOTEM_OF_ETERNITY), 0), off);
    }

    #[test]
    fn hand_save_leaves_other_hands_alone() {
        assert_eq!(hand_save(None, Some(TOTEM_OF_UNDYING), 0), HandSave::Nothing);
        assert_eq!(hand_save(Some(ids::SLEEPING_TOTEM), None, 0), HandSave::Nothing);
        assert_eq!(hand_save(None, None, 0), HandSave::Nothing);
    }

    #[test]
    fn the_save_effects_are_the_java_ones() {
        // TotemOfEternityHandler.java:44-46.
        let seen: Vec<(&str, u8, i32)> = SAVE_EFFECTS.iter().map(|e| (e.id, e.amplifier, e.duration)).collect();
        assert_eq!(
            seen,
            [
                ("minecraft:regeneration", 1, 900),
                ("minecraft:absorption", 1, 100),
                ("minecraft:fire_resistance", 0, 800),
            ]
        );
        assert_eq!(SAVE_EFFECTS[2].id, UNDYING_LOSES);
        assert_eq!(SAVE_HEALTH, 1.0);
        assert_eq!(SAVED_ADVANCEMENT, "eternity/saved");
    }

    #[test]
    fn damage_that_bypasses_invulnerability_is_not_saved() {
        assert!(!can_save_from("minecraft:out_of_world"));
        assert!(!can_save_from("minecraft:generic_kill"));
        assert!(can_save_from("minecraft:generic"));
        assert!(can_save_from("minecraft:player_attack"));
        assert!(can_save_from("minecraft:fall"));
    }

    #[test]
    fn a_player_with_a_totem_is_saved_once() {
        let mut book = SaveBook::new();
        book.note(7, 42, true);
        assert!(book.on_death(42, "minecraft:generic"));
        // The swap has not run: a second death is a real death and clears the wait.
        assert!(!book.on_death(42, "minecraft:generic"));
        assert!(!book.take_pending(7));
    }

    #[test]
    fn the_swap_runs_once() {
        let mut book = SaveBook::new();
        book.note(7, 42, true);
        assert!(book.on_death(42, "minecraft:fall"));
        assert!(book.take_pending(7));
        assert!(!book.take_pending(7));
        // The next tick sees the Dormant Totem: no more save.
        book.note(7, 42, false);
        assert!(!book.on_death(42, "minecraft:fall"));
    }

    #[test]
    fn no_totem_no_save() {
        let mut book = SaveBook::new();
        book.note(7, 42, false);
        assert!(!book.on_death(42, "minecraft:generic"));
        assert!(!book.take_pending(7));
    }

    #[test]
    fn an_unknown_entity_or_zero_is_never_saved() {
        let mut book = SaveBook::new();
        book.note(7, 42, true);
        book.note(8, 0, true);
        book.note(9, -3, true);
        assert!(!book.on_death(43, "minecraft:generic"));
        assert!(!book.on_death(0, "minecraft:generic"));
    }

    #[test]
    fn the_void_and_kill_still_kill() {
        let mut book = SaveBook::new();
        book.note(7, 42, true);
        assert!(!book.on_death(42, "minecraft:out_of_world"));
        assert!(!book.on_death(42, "minecraft:generic_kill"));
        // The totem is not spent.
        assert!(book.on_death(42, "minecraft:generic"));
    }

    #[test]
    fn a_new_entity_id_replaces_the_old_one() {
        let mut book = SaveBook::new();
        book.note(7, 42, true);
        book.note(7, 55, true);
        assert!(!book.on_death(42, "minecraft:generic"));
        assert!(book.on_death(55, "minecraft:generic"));
    }

    #[test]
    fn forgetting_a_player_drops_his_wait() {
        let mut book = SaveBook::new();
        book.note(7, 42, true);
        assert!(book.on_death(42, "minecraft:generic"));
        book.forget(7);
        assert!(!book.take_pending(7));
        assert!(!book.on_death(42, "minecraft:generic"));
        book.note(7, 42, true);
        book.clear();
        assert!(!book.on_death(42, "minecraft:generic"));
    }

    #[test]
    fn two_players_are_kept_apart() {
        let mut book = SaveBook::new();
        book.note(1, 10, true);
        book.note(2, 20, true);
        assert!(book.on_death(20, "minecraft:generic"));
        assert!(!book.take_pending(1));
        assert!(book.take_pending(2));
    }

    #[test]
    fn the_first_totem_in_container_order_saves() {
        let slots = [(0, "minecraft:stick"), (3, ids::TOTEM_OF_ETERNITY), (9, ids::TOTEM_OF_ETERNITY)];
        assert_eq!(first_totem_slot(slots), Some(3));
        assert_eq!(first_totem_slot([(0, ids::SLEEPING_TOTEM), (1, ids::DORMANT_TOTEM)]), None);
        assert_eq!(first_totem_slot([]), None);
    }

    #[test]
    fn only_the_first_totem_stays_in_an_inventory() {
        let slots = [(0, "minecraft:stick"), (3, ids::TOTEM_OF_ETERNITY), (9, ids::TOTEM_OF_ETERNITY), (-106, ids::TOTEM_OF_ETERNITY)];
        let found = totem_slots(slots);
        assert_eq!(found, [3, 9, -106]);
        assert_eq!(extra_slots(&found), [9, -106]);
        assert!(extra_slots(&[]).is_empty());
        assert!(extra_slots(&[4]).is_empty());
    }

    #[test]
    fn a_second_totem_is_never_picked_up() {
        assert!(refuses_pickup(ids::TOTEM_OF_ETERNITY, ["minecraft:stick", ids::TOTEM_OF_ETERNITY]));
        assert!(!refuses_pickup(ids::TOTEM_OF_ETERNITY, ["minecraft:stick"]));
        // The Dormant Totem is another type, and another item is not this rule's.
        assert!(!refuses_pickup(ids::TOTEM_OF_ETERNITY, [ids::SLEEPING_TOTEM]));
        assert!(!refuses_pickup(ids::SLEEPING_TOTEM, [ids::SLEEPING_TOTEM]));
        assert!(!refuses_pickup("minecraft:stick", [ids::TOTEM_OF_ETERNITY]));
    }

    #[test]
    fn a_new_totem_leaves_a_container_that_has_one() {
        // One totem at slot 4 when opened, a second placed at slot 1: the new one leaves.
        assert_eq!(container_extras(&[4], &[1, 4]), [1]);
        // Nothing before, two now: the first stays.
        assert_eq!(container_extras(&[], &[2, 8]), [8]);
        // One totem only, new or moved: it stays.
        assert_eq!(container_extras(&[], &[5]), Vec::<usize>::new());
        assert_eq!(container_extras(&[4], &[6]), Vec::<usize>::new());
        // A pair that was there before is not touched.
        assert_eq!(container_extras(&[1, 2], &[1, 2]), Vec::<usize>::new());
        // A pair that was there and a third one: the third leaves.
        assert_eq!(container_extras(&[1, 2], &[1, 2, 3]), [3]);
        // Three new ones with none before: the first stays.
        assert_eq!(container_extras(&[], &[0, 1, 2]), [1, 2]);
        // Nothing now.
        assert!(container_extras(&[3], &[]).is_empty());
    }

    #[test]
    fn the_totem_lines_follow_the_java_items() {
        let eternity = tooltip(ids::TOTEM_OF_ETERNITY);
        let keys: Vec<&str> = eternity.iter().map(|l| l.key).collect();
        assert_eq!(keys, ["tooltip.bmh.eternity", "tooltip.bmh.totem_of_eternity_save", "tooltip.bmh.heavy_with_eternity"]);
        let colors: Vec<u32> = eternity.iter().map(|l| l.color).collect();
        assert_eq!(colors, [0xAAAAAA, 0xAAAAAA, 0xFFAA00]);

        let dormant = tooltip(ids::SLEEPING_TOTEM);
        let keys: Vec<&str> = dormant.iter().map(|l| l.key).collect();
        assert_eq!(keys, ["tooltip.bmh.eternity", "tooltip.bmh.sleeping_totem", "tooltip.bmh.heavy_with_eternity"]);
        assert_eq!(dormant[1].number, Some(30));
        assert_eq!(dormant[0].number, None);

        assert_eq!(tooltip(ids::DORMANT_TOTEM), [line("tooltip.bmh.dormant_totem", GRAY)]);
        assert_eq!(tooltip(ids::ECHO_HEART), [line("tooltip.bmh.echo_heart", GRAY)]);
        assert!(tooltip("minecraft:stick").is_empty());
        assert!(tooltip(ids::NAVIGATOR_AMULET).is_empty());
    }

    #[test]
    fn the_anvil_cases() {
        assert_eq!(anvil_case(ids::DORMANT_TOTEM, Some(ids::ECHO_HEART)), Some(Anvil::Awaken));
        // The Dormant Totem (id sleeping_totem) does not wake at the anvil.
        assert_eq!(anvil_case(ids::SLEEPING_TOTEM, Some(ids::ECHO_HEART)), None);
        // The order matters: the totem is on the left.
        assert_eq!(anvil_case(ids::ECHO_HEART, Some(ids::DORMANT_TOTEM)), None);
        assert_eq!(anvil_case(ids::DORMANT_TOTEM, None), None);
        assert_eq!(anvil_case(ids::ETERNAL_SWORD, Some(GOLD_BLOCK)), Some(Anvil::GoldRepair));
        assert_eq!(anvil_case(ids::ETERNAL_AXE, Some(GOLD_BLOCK)), Some(Anvil::GoldRepair));
        assert_eq!(anvil_case(ids::ETERNAL_PICKAXE, Some(GOLD_BLOCK)), Some(Anvil::GoldRepair));
        // Not the bundle, not a vanilla tool, not another material.
        assert_eq!(anvil_case(ids::ETERNAL_BUNDLE, Some(GOLD_BLOCK)), None);
        assert_eq!(anvil_case("minecraft:golden_sword", Some(GOLD_BLOCK)), None);
        assert_eq!(anvil_case(ids::ETERNAL_SWORD, Some("minecraft:gold_ingot")), None);
        assert_eq!(anvil_case(ids::ETERNAL_SWORD, None), None);
    }

    #[test]
    fn the_anvil_numbers_are_the_java_ones() {
        // TotemAwakeningHandler.java:12, B_AnvilMenuMixin.java:76, EternalWear.java:27.
        assert_eq!(AWAKEN_COST, 39);
        assert_eq!(AWAKEN_MATERIAL_COST, 1);
        assert_eq!(GOLD_REPAIR_COST, 15);
        // An awakening must stay under the "Too Expensive" line of a survival anvil.
        assert!(AWAKEN_COST < 40);
    }

    #[test]
    fn a_gold_block_repairs_a_quarter() {
        // 33000 / 4 = 8250 a block (vanilla `getMaxDamage() / 4`).
        assert_eq!(gold_repair(10_000, 1), Some(GoldRepair { damage: 1_750, blocks: 1 }));
        // The second block repairs what is left, and the third is not used.
        assert_eq!(gold_repair(10_000, 3), Some(GoldRepair { damage: 0, blocks: 2 }));
        // A worn out tool (32999 damage) needs four blocks.
        assert_eq!(gold_repair(32_999, 64), Some(GoldRepair { damage: 0, blocks: 4 }));
        assert_eq!(gold_repair(32_999, 2), Some(GoldRepair { damage: 16_499, blocks: 2 }));
        // A small damage takes one block.
        assert_eq!(gold_repair(1, 5), Some(GoldRepair { damage: 0, blocks: 1 }));
    }

    #[test]
    fn no_repair_without_damage_or_blocks() {
        assert_eq!(gold_repair(0, 5), None);
        assert_eq!(gold_repair(500, 0), None);
        // Damage above the durability: the loop still ends.
        assert_eq!(gold_repair(u32::MAX, 1), Some(GoldRepair { damage: u32::MAX - 8_250, blocks: 1 }));
    }

    #[test]
    fn the_rename_box_follows_vanilla() {
        // No text or a blank text: a custom name goes, else nothing changes.
        assert_eq!(anvil_rename(None, Some("Old"), true), Rename::Clear);
        assert_eq!(anvil_rename(Some("  "), Some("Old"), true), Rename::Clear);
        assert_eq!(anvil_rename(Some(""), None, false), Rename::Keep);
        assert_eq!(anvil_rename(None, None, false), Rename::Keep);
        // The same text as the shown name changes nothing.
        assert_eq!(anvil_rename(Some("Old"), Some("Old"), true), Rename::Keep);
        // Another text is the new name.
        assert_eq!(anvil_rename(Some("New"), Some("Old"), true), Rename::Set);
        assert_eq!(anvil_rename(Some("New"), None, false), Rename::Set);
    }

    #[test]
    fn plain_gold_blocks_are_never_too_expensive() {
        assert!(!gold_repair_refused(0, 4, true));
        assert!(!gold_repair_refused(35, 4, false));
        assert!(gold_repair_refused(35, 4, true));
        assert!(gold_repair_refused(u32::MAX, 1, false));
    }

    #[test]
    fn too_expensive_is_the_vanilla_line() {
        assert_eq!(TOO_EXPENSIVE, 40);
        assert_eq!(ETERNAL_TOOL_MAX_DAMAGE, 33_000);
    }

    #[test]
    fn the_sound_is_the_vanilla_totem_sound() {
        assert_eq!(SAVE_SOUND, "minecraft:item.totem.use");
        assert_eq!((SAVE_SOUND_VOLUME, SAVE_SOUND_PITCH), (1.0, 1.0));
        assert!(SAVE_PARTICLES > 0);
    }

    #[test]
    fn a_menu_click_that_places_a_second_totem_is_refused() {
        // I_SlotTotemMixin.java:21-28: mayPlace is false when another totem is in the container.
        let chest = [false, true, false, false];
        let click = |slot, mode, cursor, slot_stack, key_stack| Click { slot, mode, cursor, slot_stack, key_stack };
        // Pickup with a totem on the cursor, onto another slot: refused. Onto the totem's own slot: a swap.
        assert!(click_refused(&chest, &click(0, 0, true, false, false)));
        assert!(!click_refused(&chest, &click(1, 0, true, false, false)));
        // No totem on the cursor: free. A player slot (4 and up) with the cursor totem: free.
        assert!(!click_refused(&chest, &click(0, 0, false, false, false)));
        assert!(!click_refused(&chest, &click(6, 0, true, false, false)));
        // Quick move of a player totem into a chest that holds one: refused. From the chest: free.
        assert!(click_refused(&chest, &click(5, 1, false, true, false)));
        assert!(!click_refused(&chest, &click(1, 1, false, false, false)));
        assert!(!click_refused(&[false; 4], &click(5, 1, false, true, false)));
        // A number key: the hotbar totem into another slot is refused, into the totem's slot it is not.
        assert!(click_refused(&chest, &click(2, 2, false, false, true)));
        assert!(!click_refused(&chest, &click(1, 2, false, false, true)));
        // Clone, throw, pick up all and the outside click place nothing.
        for mode in [3, 4, 6] {
            assert!(!click_refused(&chest, &click(0, mode, true, true, true)));
        }
        assert!(!click_refused(&chest, &click(-999, 0, true, true, true)));
    }

    #[test]
    fn a_drag_of_a_totem_over_a_full_container_slot_is_refused() {
        let chest = [true, false, false];
        assert!(drag_refused(&chest, &[1, 2], true));
        assert!(!drag_refused(&chest, &[0], true));
        assert!(!drag_refused(&chest, &[1], false));
        assert!(!drag_refused(&chest, &[3, 4], true));
        assert!(!drag_refused(&[false; 3], &[0, 1], true));
    }

    #[test]
    fn the_scan_takes_only_the_new_totems() {
        assert_eq!(scan_ejects(Some(1), 2), 1);
        assert_eq!(scan_ejects(Some(0), 2), 1);
        assert_eq!(scan_ejects(Some(1), 1), 0);
        assert_eq!(scan_ejects(Some(2), 2), 0);
        assert_eq!(scan_ejects(Some(2), 3), 1);
        assert_eq!(scan_ejects(None, 5), 0);
    }
}
