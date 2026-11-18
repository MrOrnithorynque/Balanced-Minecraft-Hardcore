// Area amulet, the part that lives in horizon-world: the hooks there have no `Game`.
// The module is `pub`, so the game side can reach it as `horizon_world::bmh_amulet_world`.
// Owner: the amulet agent. Log: docs/horizon/log/amulet.md.
//
// This crate cannot reach `bmh_logic` (the logic crate is a dependency of horizon-client only, see
// docs/horizon/HOOKS.md, "Hooks with no Game"), so the item id and the target are written here. The
// logic crate pins the same values (`bmh_logic::amulet::NEEDLE_TARGET`).

/// The Navigator's Amulet (`bmh_logic::ids::NAVIGATOR_AMULET`).
const AMULET: &str = "bmh:navigator_amulet";

/// The component that the `minecraft:compass` range property of the item model reads
/// (horizon-assets item_model.rs, `compass()`, target `lodestone`).
const TRACKER: &str = "minecraft:lodestone_tracker";

/// The block the needle aims at: far north of the origin of the Overworld (horizon/content/NOTES.md,
/// "The amulet needle"). Java aims at the block 1.0E6 north of the holder (AmuletCompass.java:64-66);
/// a fixed block differs by less than 2 degrees up to x = 1 000 000 (GAPS.md A1).
const TARGET_DIMENSION: &str = "minecraft:overworld";
const TARGET_POS: [i32; 3] = [0, 64, -30_000_000];

// A1: the needle of the amulet points north. The content item cannot carry a default
// `lodestone_tracker` (the content loader refuses the type, docs/horizon/log/content.md), so this hook
// sets it in the defaults table that every reader of the item uses. `tracked` is false: Horizon clears
// only a tracked target whose lodestone is gone (maps.rs, `tick_lodestone_compasses`). The answer
// depends on the item id alone, so it is the same on both sides and at each call.
pub(crate) fn on_modify_default_components(
    item: &str,
    components: &mut serde_json::Map<String, serde_json::Value>,
) {
    if item != AMULET {
        return;
    }
    let [x, y, z] = TARGET_POS;
    let tracker = serde_json::json!({
        "target": { "dimension": TARGET_DIMENSION, "pos": [x, y, z] },
        "tracked": false,
    });
    components.insert(TRACKER.to_owned(), tracker);
}
