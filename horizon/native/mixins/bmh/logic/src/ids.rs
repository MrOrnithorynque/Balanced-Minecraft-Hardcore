// Ids that every side and every area shares (integrator).

/// The mod id of the content part and the native part (decision D4).
pub const MOD_ID: &str = "bmh";

/// The mod version. It is `mod_version` in the repo's gradle.properties.
pub const VERSION: &str = "1.9.0";

/// The Eternal Sword.
pub const ETERNAL_SWORD: &str = "bmh:eternal_sword";
/// The Eternal Axe.
pub const ETERNAL_AXE: &str = "bmh:eternal_axe";
/// The Eternal Pickaxe.
pub const ETERNAL_PICKAXE: &str = "bmh:eternal_pickaxe";
/// The Eternal Bundle.
pub const ETERNAL_BUNDLE: &str = "bmh:eternal_bundle";
/// The Navigator's Amulet.
pub const NAVIGATOR_AMULET: &str = "bmh:navigator_amulet";
/// The Totem of Eternity.
pub const TOTEM_OF_ETERNITY: &str = "bmh:totem_of_eternity";
/// The Dormant Totem (after a save). The Java id is kept for old worlds.
pub const SLEEPING_TOTEM: &str = "bmh:sleeping_totem";
/// The Hollow Totem (crafted). The Java id is kept for old worlds.
pub const DORMANT_TOTEM: &str = "bmh:dormant_totem";
/// The Echo Heart.
pub const ECHO_HEART: &str = "bmh:echo_heart";

/// Every item of the mod.
pub const ALL_ITEMS: [&str; 9] = [
    ETERNAL_SWORD,
    ETERNAL_AXE,
    ETERNAL_PICKAXE,
    ETERNAL_BUNDLE,
    NAVIGATOR_AMULET,
    TOTEM_OF_ETERNITY,
    SLEEPING_TOTEM,
    DORMANT_TOTEM,
    ECHO_HEART,
];

/// The Eternal Items: the items that implement IEternalItem in the Java mod
/// (EternalSwordItem.java:18, EternalAxeItem.java:17, EternalPickaxeItem.java:16,
/// EternalBundleItem.java:42, TotemOfEternityItem.java:12,
/// DormantTotemOfEternityItem.java:30). The Navigator's Amulet, the Hollow
/// Totem and the Echo Heart are not Eternal Items.
pub const ETERNAL_ITEMS: [&str; 6] = [
    ETERNAL_SWORD,
    ETERNAL_AXE,
    ETERNAL_PICKAXE,
    ETERNAL_BUNDLE,
    TOTEM_OF_ETERNITY,
    SLEEPING_TOTEM,
];

/// True when `item` is an Eternal Item (Java: `instanceof IEternalItem`,
/// RespawnPositionHandler.java:230). Every area uses this test.
/// The "one of each type" rule compares the item itself
/// (EternalItemHandler.java:90, `item.getItem() == eternalItem.getItem()`),
/// so the type of an Eternal Item is its id: two stacks are of one type when
/// their ids are equal. The Totem of Eternity and the Dormant Totem are two types.
pub fn is_eternal(item: &str) -> bool {
    ETERNAL_ITEMS.contains(&item)
}

/// The channel of an area's mod messages: "bmh:<area>".
pub fn channel(area: &str) -> String {
    format!("{MOD_ID}:{area}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_item_is_in_the_mod_namespace() {
        for id in ALL_ITEMS {
            assert!(id.starts_with("bmh:"), "{id}");
        }
    }

    #[test]
    fn the_eternal_items_are_the_java_ieternalitem_set() {
        let eternal: Vec<&str> = ALL_ITEMS.into_iter().filter(|id| is_eternal(id)).collect();
        assert_eq!(eternal, ETERNAL_ITEMS);
        assert!(!is_eternal(NAVIGATOR_AMULET));
        assert!(!is_eternal(DORMANT_TOTEM));
        assert!(!is_eternal(ECHO_HEART));
        assert!(!is_eternal("minecraft:totem_of_undying"));
        assert!(!is_eternal(""));
    }

    #[test]
    fn a_channel_has_the_mod_id() {
        assert_eq!(channel("journey"), "bmh:journey");
    }
}
