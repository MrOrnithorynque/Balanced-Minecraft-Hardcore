use crate::logic::ids;
use mod_api::{Host, Identifier, MenuContainer, MenuPart, MenuSlotRulesEvent, Registrar};

const STORAGE_MENUS: &[&str] = &[
    "minecraft:generic_9x1",
    "minecraft:generic_9x2",
    "minecraft:generic_9x3",
    "minecraft:generic_9x4",
    "minecraft:generic_9x5",
    "minecraft:generic_9x6",
    "minecraft:generic_3x3",
    "minecraft:shulker_box",
    "minecraft:hopper",
    "minecraft:furnace",
    "minecraft:blast_furnace",
    "minecraft:smoker",
    "minecraft:brewing_stand",
    "minecraft:anvil",
    "minecraft:smithing",
    "minecraft:grindstone",
    "minecraft:enchantment",
    "minecraft:merchant",
    "minecraft:beacon",
    "horizon:mount_inventory",
];

struct TotemLimit;

#[mod_api::horizon_part]
impl MenuPart for TotemLimit {
    fn menu_slot_rules(&self, host: &dyn Host, event: &mut MenuSlotRulesEvent<'_>) {
        if !event.may_place || event.container == MenuContainer::Player {
            return;
        }
        let Some(awake) =
            Identifier::parse_namespaced(ids::TOTEM_OF_ETERNITY).and_then(|id| host.item_key(&id))
        else {
            return;
        };
        if event.stack.item() != awake {
            return;
        }
        if let Some(slots) = event.context.slots.get(event.container as usize) {
            event.may_place = !slots.iter().enumerate().any(|(index, stack)| {
                index != event.slot && stack.as_ref().is_some_and(|stack| stack.item() == awake)
            });
        }
    }
}

pub(super) fn register(registrar: &mut dyn Registrar) {
    for menu in STORAGE_MENUS {
        registrar.menu_part(menu, TotemLimit);
    }
}
