# Items in general, the Eternal tools and their abilities, the data and assets

Short names used below. W = the BMH worktree. S = the scratchpad folder of the brief. All Horizon paths are under
S/src/horizon unless they start with S/horizon-docs or S/horizon-repo-docs. "Nightly" = S/horizon-repo-docs/native-hooks.nightly-2026-11-08.json.
Java paths are under W/common/src/main/java/github/mrornithorynque/bmh/ ("J/").
Main's hook level is `NATIVE_HOOKS_API: u32 = 415` (crates/horizon-client/src/game/native_hooks.rs:1086). The Nightly file has no API number,
so a hook that main grew after its first stamp (a new API number on an old hook name) is "name in Nightly, extension UNVERIFIED".

Reading note: S/horizon-docs/content-mods.txt (public site) is OLDER than the repo docs. It says an item "may change max_stack_size (1 to 99) and
rarity" (content-mods.txt:102) and "Content mods work in single player today" (content-mods.txt:126). The repo says otherwise (docs/mods.md:248-262
component patches; docs/mods.md:107-110 LAN and dedicated server). I follow the repo (main) and flag every place where it matters.

## Verdict table

| Feature | Verdict | Mechanism in one line |
|---|---|---|
| 1a Item ids, stack size, rarity, fire resistance (all 9 items) | HOOK (content mod data) | `data/bmh/horizon/item/<name>.json`, template + `components` patch |
| 1b Tool stats of sword, axe, pickaxe (33000 durability, damage, speed, enchantability, repair item) | HOOK (content mod data), numbers UNVERIFIED | netherite template + `components` (`max_damage`, `attribute_modifiers`, `tool`, `enchantable`, `repairable`) |
| 1c Item classes with code (tooltip lines, bar, `getDestroySpeed`, bundle clicks) | PARTIAL | class code does not exist; each behaviour maps to a hook (rows 3 to 7) |
| 1d Works on dedicated server | HOOK, same mods on both sides | docs/mods.md:107-110; content mods + native mod both need the same build on server and client |
| 1e Vanilla client joins | GAP | no registry sync for vanilla brand; modded item ids unknown to vanilla (BMH never supported it either) |
| 2 Recipes (shaped, smithing_transform), tags, lang, sounds.json, item models (select on display_context) | HOOK (content mod data) | vanilla data pack and resource pack formats in `data/` and `assets/` |
| 2b `range_dispatch` on `bmh:north` (amulet needle) | GAP as written, PARTIAL with a rewrite | foreign range property reads 0; use nested `minecraft:condition` + `item_model_property` hook |
| 3a Tools never break (keep last point) | PARTIAL | `item_damaged` (HOOK) on dedicated/LAN guests; PINNED for the single-player/LAN host |
| 3b Worn out: half damage | HOOK | `living_incoming_damage_v2` amount, or stack patch from `inventory_tick` |
| 3c Worn out: stone mining speed | PARTIAL | `break_speed` is a pending fragment (not in either catalog); alternative `inventory_tick` stack patch of `minecraft:tool` |
| 3d 7 percent death wear | HOOK | `loot_drops` API 405 (dying player's inventory), main-only extension |
| 3e Tooltip lines (ability, heavy, death wear, worn out) | HOOK (client) | `tooltip` hook, `lines: &mut Vec<(Text, u32)>` |
| 4a Echo Strike (sword) | PARTIAL | `living_incoming_damage_v2` + own queue on `server_tick`/`player_tick` + `hurt_entity` (pending fragment) |
| 4b Axe Smash | PARTIAL | `living_incoming_damage_v2` for the multiplier; fall distance must be tracked by the mod; `living_fall` to halve it on landing |
| 4c Echo Sense (pickaxe) | GAP for the glow as built; PARTIAL via client `world_primitives` | Horizon draws no outline for display entities; through-walls box outlines need a client half and a pending fragment |
| 5 Eternal Items on the ground | PARTIAL | `item_expire` (HOOK, never despawn); damage immunity only approximated (`Health` NBT) |
| 6 Items kept after death | HOOK, one main-only extension | `loot_drops` API 405 + `player_respawned` + `give_stack` |
| 6b One Eternal Item per type, no pickup of a second | HOOK | `item_picked_up` (cancel) + `inventory_tick` |
| 7 Eternal Bundle | PARTIAL | `minecraft:bundle` template is a bundle by component; refusing Eternal Items inside and "drop all at once" need hooks that are guest-only or pending |
| 8 Creative tab, two sprites of the sword | HOOK (content mod data) | `creative_tab/*.json`; `items/eternal_sword.json` select on `display_context` is decoded |

Counts over the 21 rows: HOOK 10 (1a, 1b, 1d, 2, 3b, 3d, 3e, 6, 6b, 8), PARTIAL 8 (1c, 3a, 3c, 4a, 4b, 5, 7, plus 2b with the rewrite), GAP 3 (1e, 4c glow, 2b as written). PINNED appears only as a sub-case of 3a (single player). Rows 3d and 6 depend on API 405, which is main-only or Nightly-unknown.

---

## 1. Item registration (the load-bearing question)

### Behaviour spec (what the Java mod does)

Registration: `init/ItemInit.java:19-97`, through `Services.PLATFORM.register(Registries.ITEM, name, ...)` (`ItemInit.java:101-108`).
Every item gets `Item.Properties.setId(key)` first (`ItemInit.java:107`). Namespace is the mod id `bmh`.

| Id | Class (J/items/) | Properties | Cite |
|---|---|---|---|
| `bmh:eternal_bundle` | EternalBundleItem | stacksTo 1, `BUNDLE_CONTENTS` = EMPTY, fireResistant, EPIC | ItemInit.java:19-25 |
| `bmh:eternal_axe` | EternalAxeItem (extends AxeItem) | tier ETERNAL, attack 17.0, speed -3.2, stacksTo 1, fireResistant, EPIC | ItemInit.java:27-35 |
| `bmh:eternal_pickaxe` | EternalPickaxeItem | tier ETERNAL, attack 7.0, speed -2.0, stacksTo 1, fireResistant, EPIC | ItemInit.java:37-45 |
| `bmh:eternal_sword` | EternalSwordItem | tier ETERNAL, attack 10.0, speed -1.4, stacksTo 1, fireResistant, EPIC | ItemInit.java:47-55 |
| `bmh:navigator_amulet` | NavigatorAmuletItem | stacksTo 1, COMMON | ItemInit.java:57-61 |
| `bmh:totem_of_eternity` | TotemOfEternityItem | stacksTo 1, fireResistant, EPIC | ItemInit.java:63-68 |
| `bmh:sleeping_totem` (Dormant Totem) | DormantTotemOfEternityItem | stacksTo 1, fireResistant, EPIC | ItemInit.java:74-79 |
| `bmh:dormant_totem` (Hollow Totem) | HintItem("tooltip.bmh.dormant_totem") | stacksTo 1, fireResistant, RARE | ItemInit.java:83-89 |
| `bmh:echo_heart` | HintItem("tooltip.bmh.echo_heart") | stacksTo 1, fireResistant, RARE | ItemInit.java:91-97 |

Tier `ETERNAL` (`init/BMHModTierInit.java:20-21`): `new ToolMaterial(BlockTags.INCORRECT_FOR_NETHERITE_TOOL, 33000, 15.0F, 0.0F, 80, ETERNAL_TOOL_MATERIALS)`
= incorrect-blocks tag of netherite (it mines what netherite mines), durability 33000, mining speed 15, attack damage bonus 0, enchantability 80, repair
tag `bmh:eternal_tool_materials` (= `minecraft:gold_block` only, `data/bmh/tags/item/eternal_tool_materials.json`). Sword and pickaxe use
`properties.sword(material, dmg, speed)` / `.pickaxe(...)` (`EternalSwordItem.java:21`, `EternalPickaxeItem.java:19`); the axe extends `AxeItem` for
stripping, scraping and un-waxing (`EternalAxeItem.java:16-21`). Sweep is tag based: `data/minecraft/tags/item/swords.json` adds `bmh:eternal_sword`
(and `axes.json`, `pickaxes.json` for their tags). Components added by code: only `BUNDLE_CONTENTS` on the bundle. `fireResistant()` is the
`damage_resistant` component.

Class-level behaviour that is not a property:
- Tool classes: tooltip lines (`appendHoverText`, e.g. `EternalSwordItem.java:24-32`), a bar that keeps 1 pixel (`getBarWidth` `:41-44`), `reduceDurability` (`:35-38`),
  `getDestroySpeed` capped at 4.0 when worn out (axe `EternalAxeItem.java:47-53`, pickaxe `EternalPickaxeItem.java:46-52`).
- HintItem: one gray tooltip line (`HintItem.java:25-31`).
- All Eternal Items implement `IEternalItem { void reduceDurability(ItemStack, int percentage); }` (`IEternalItem.java:5-8`). Implementers: sword, axe, pickaxe,
  bundle, Totem of Eternity, Dormant Totem. Hollow Totem and Echo Heart are NOT Eternal (`ItemInit.java:81-82`).
- Sounds: `init/SoundInit.java:14-15` registers sound events `bmh:end` and `bmh:bell_sfx` (`createVariableRangeEvent`).
- Creative tab: `init/CreativeTabInit.java:16-32`, title key `itemGroup.BMH_tab`, icon = Totem of Eternity, items in order: axe, bundle, pickaxe, sword,
  amulet, totem, dormant totem, hollow totem, echo heart, then vanilla `Items.BUNDLE`.

### Horizon mapping: every way a mod adds an item

There are two ways to declare an item and they share ONE declaration type.

**A. Content mod (`.hmod`, data only).**
- File `data/<id>/horizon/item/<name>.json`: "An item is a file data/<id>/horizon/item/<name>.json, copying a vanilla item" (S/horizon-docs/content-mods.txt:95).
  Id is `<mod id>:<name>`, so mod id `bmh` gives `bmh:eternal_sword` (content-mods.txt:56: "It is the namespace of its blocks and items (ruby:ruby_block)").
- The struct is `horizon_mods::content::Item` (crates/horizon-mods/src/content.rs:408-422): `template: String` (vanilla item), `max_stack_size: Option<u8>`,
  `rarity: Option<Rarity>`, `components: BTreeMap<String, serde_json::Value>`, `crafting_remainder: Option<String>`, `place_block: Option<String>`.
- docs/mods.md:248-262: "An item copies its template's data components, with `max_stack_size` and `rarity` changed, and any other 26.3 component set or removed in
  `components`, written as a data component patch is ... That is how a modded sword gets its own damage and durability, or a tool its mining speed", with an example
  setting `minecraft:max_damage` 900, `minecraft:attribute_modifiers` and `"!minecraft:repairable": {}`.
- Components must be real 26.3 component types: `DATA_COMPONENT_TYPES` (122 entries, crates/horizon-assets/src/item_model.rs:28-175) lists, among others,
  `minecraft:max_damage`, `minecraft:damage`, `minecraft:tool`, `minecraft:weapon`, `minecraft:attribute_modifiers`, `minecraft:enchantable`, `minecraft:repairable`,
  `minecraft:damage_resistant`, `minecraft:bundle_contents`, `minecraft:block_transformer`, `minecraft:rarity`, `minecraft:max_stack_size`, `minecraft:unbreakable`.
  docs/mods.md:333-335: "A component type no vanilla item has is refused".
- LIMITS that touch BMH: "Behaviour the game ties to a vanilla item's id doesn't come along yet: a copy of the trident doesn't throw" (docs/mods.md:334-337).
  An item can have no code of its own (content-mods.txt:154: "A content mod has no code").
- Creative tab: `data/<id>/horizon/creative_tab/*.json`, `{ "title": ..., "icon": "ruby:ruby", "items": [...] }` (docs/mods.md:317-332). The title may be a lang key
  (`"title": "itemGroup.ruby"`). Items "the game doesn't have are left out", so vanilla ids are accepted.
- Template and component facts for the Eternal tools. Axe stripping is data: `minecraft:block_transformer` is a component type in the list above and Horizon
  loads `data/minecraft/block_transformer` (crates/horizon-client/src/block_transform.rs:1-4 and :185). So a netherite_axe template should keep stripping.
  UNVERIFIED: that the template item carries that component in 26.3 (the prepared `items.json` is not in the repo). Sweep reads the item tag:
  `.get("minecraft:swords")` in crates/horizon-client/src/game/melee.rs:30-35, so `data/minecraft/tags/item/swords.json` works unchanged.
- A bundle is a bundle by component, not by id: `bundle: c.get("minecraft:bundle_contents").is_some()` (crates/horizon-items/src/items.rs:943) and
  `contents_of` in crates/horizon-items/src/bundle.rs:64-69. A `minecraft:bundle` template therefore behaves as vanilla 26.3 bundle (insert, remove, weight, fullness model).
- Loading time: "a content mod that adds blocks, items, ... loads only at start" (docs/api/content.md:44-46). No hot-load for items.

**B. Native mod `items` hook (code).**
- Catalog entry (S/src/horizon/native-hooks.json, hook `items`): `{"name":"items","version":1,"target":"...Game<'_>::native_items","neoforge":"RegisterEvent","mod_api":{"contract":"collect","out":"items"}}`. Same for `blocks`.
  Present in the Nightly 2026-11-08 file.
- docs/native-mods.md:1254 (API 145): "adds blocks and items a native mod declares in code, as NeoForge's `DeferredRegister.Blocks` and `.Items` drained by
  `RegisterEvent` ... `native_blocks(blocks)` and `native_items(items)`, asked once at start on every side, take `horizon_mods::content::Block` and `Item`, the declarations a
  content mod's files hold (a vanilla template and what differs ...). Ids are in the mod's namespace. They join the content mods' blocks and items, inside the content mod
  whose id is that namespace when it is on (its models, textures, loot and recipes), else on their own with the template's look".
- Meaning: the native hook builds the SAME `Item` struct (content.rs:408). It does NOT give an item its own Rust behaviour. Item behaviour in code is attached afterwards by
  the ordinary hooks (below), keyed on the item id. There is no `Item` subclass, no `use()` override for a registered item, no `getDestroySpeed`.
- Related native hooks: `component_types` (catalog, API 65, docs/native-mods.md:924: "Append namespaced component IDs; duplicates, invalid IDs and the `minecraft` namespace stop
  startup"; values are NBT) lets a mod add its own data component (for example `bmh:charge`); a content item can then set it (docs/mods.md:272-277). `modify_default_components`
  (catalog, API 79, docs/native-mods.md:996 and :5081-5099) edits the default components of ANY item, vanilla included, at start on both sides ("Client and server each apply their own
  mods, so both need the mod", :5097-5099).
- How Horizon's own mods register code items: UNVERIFIED. I searched S/src/horizon-mods/native/ names only through the docs; the docs say the native `items` hook is "the declarations a
  content mod's files hold", so no example registers an item with a code class.

**C. Assets of the item (models, textures, lang, sounds).**
- Content mod: `assets/<id>/items/*.json` (item model definitions), `models/`, `textures/`, `lang/en_us.json`, `sounds.json` + `sounds/*.ogg`, "as in a resource pack"
  (content-mods.txt:28-35, :110-111). The assets folder of a content mod is also the place for native-mod sounds: "Put sound events in `assets/<namespace>/sounds.json` and their files in
  `assets/<namespace>/sounds/*.ogg` at the archive's top level, not under `mixins/`" (docs/native-mods.md:~790, the "One .hmod for content and code" paragraph).
- Native mod `mixins/<id>/data/`: "A bundle's `data/` folder (any files) is staged as `native-data/<mod>/` ... compiles a file in with `horizon_client::native_data!`" (docs/native-mods.md:726-731).
  That is for the mod's own run-time bytes, NOT resource-pack assets. Do not put item models there.
- One `.hmod` can carry content and native code together: "A mod's content and its native code ship as one `.hmod`, and what is inside decides what happens ... Both may be in one file:
  `horizon-mod.toml`, `assets/` and `data/` are its content ... and the two manifests name the same mod id" (docs/native-mods.md, section "Data files", paragraph "One .hmod for content and code").

### Answers to the specific questions
- Can the ids be `bmh:eternal_sword` etc.? Yes, mod id `bmh`, file names `eternal_sword.json`, `sleeping_totem.json`, `dormant_totem.json` (keep the legacy ids so old worlds keep their items).
  A saved world stores ids as strings, so the same ids load.
- Own durability, attack damage, mining speed, enchantability, repair rules? Yes by component patch (content.rs:415 comment: "A 26.3 data component patch over the template's"):
  - durability 33000: `"minecraft:max_damage": 33000`
  - attack 10 / speed -1.4 (sword), 17 / -3.2 (axe), 7 / -2.0 (pickaxe): `"minecraft:attribute_modifiers"` with `minecraft:attack_damage` (`id: minecraft:base_attack_damage`, `operation: add_value`, `slot: mainhand`)
    and `minecraft:attack_speed`. The Java numbers are `attackDamage + material.attackDamageBonus (0)`, so the modifier amounts are 10.0, 17.0, 7.0; the attack speed modifier amount is the Java speed argument
    (-1.4, -3.2, -2.0) because vanilla's base is the 4.0 attribute base. UNVERIFIED: the exact 26.3 modifier ids (`minecraft:base_attack_speed`) and the exact `minecraft:tool` rule array
    that vanilla 26.1 `ToolMaterial` generates. Copy them from the netherite template and change the speed number.
  - mining speed 15: patch `minecraft:tool` (rules list, `speed` per rule). Copy the template's rules and set speed 15.0.
  - enchantability 80: `"minecraft:enchantable": {"value": 80}` (shape UNVERIFIED but the component type exists).
  - repair with gold block only: `"minecraft:repairable"` naming `#bmh:eternal_tool_materials`; the flat 15-level anvil cost and "no prior work penalty" (`B_AnvilMenuMixin`, `handlers/EternalWear.java:26-27`) need the
    `prepare_anvil` / `anvil_craft_pre` hooks (both in catalog and Nightly); that mixin belongs to another group, so it is not audited here.
  - fire resistance: `"minecraft:damage_resistant": {"types": "#minecraft:is_fire"}`. This is read by `Items::resists_fire` (crates/horizon-items/src/items.rs:693-712), which accepts only the
    `#minecraft:is_fire` string or the named source.
- Tier: 26.1 has no `Tier` class; a tier is components. The Java "incorrect blocks" tag is `INCORRECT_FOR_NETHERITE_TOOL` (BMHModTierInit.java:21); copy the netherite template's tool rules to keep the same set.

### Verdict (feature 1)
- 1a, 1b: HOOK. Pure content mod data. Use a content mod for ALL registration; it is the better path because the declaration, assets, creative tab, recipes and tags all live together and `cargo horizon-mod check` can validate them
  (docs/mods.md:277 notes the check cannot see native registrations).
- 1c: PARTIAL. Everything that was a Java method needs a hook (see features 3 to 7). The items carry no code.
- 1d dedicated server: docs/mods.md:107-110: "LAN and dedicated server players play with them when they run the same mods: a client whose mods add different content is refused when it joins, with the reason."
  The server also compares each `.hmod` by SHA-256 (`horizon:mod_list`, docs/mods.md:112-120; crates/horizon-protocol/src/mod_list.rs:1-12) and the registry hash (`horizon:block_registry`,
  crates/horizon-client/src/content_identity.rs:15 and :17-18 `MISMATCH`). `side = "both"` must be in the manifest or the check ignores the mod (docs/mods.md:118-120).
  The public page says the opposite (content-mods.txt:126: "Content mods work in single player today. Playing them together on a server is planned."): the repo is newer; treat the
  server claim as UNVERIFIED until a test boots it. docs/mods.md:402-403 says for entity types "Dedicated servers share that code, untested so far", which is the same level of confidence.
- 1e vanilla client: GAP. In the LAN join code a client whose brand is not "horizon" is refused only when `horizon_clients_only` is set or custom component IDs exist
  (crates/horizon-client/src/lan/conn.rs:1329-1335); otherwise it is let in and never receives the registry hash (conn.rs:1425-1430 sends the hash only `if brand == "horizon"`).
  Item stacks on the wire carry numeric item ids that come after vanilla's (docs/modpacks.md "Vanilla state ids do not move. Mod states are appended after vanilla's"; content_identity.rs:2-4 "item stacks ... carry their
  network ids"). A vanilla client cannot decode an id outside its item registry. UNVERIFIED by test; docs/mods.md:406 warns of the same for entities ("a vanilla client on a server with it may be disconnected").
  BMH never supported vanilla clients anyway (memory note `bmh-vanilla-client-closed.md`).
- Native mod needs the game rebuilt with it (docs/api/content.md:47-48), including on the server (docs/native-mods.md "Standalone server export", `horizon-launcher native-build FOLDER --server-output DIR ...`).

### RECOMMENDATION (feature 1)
Use BOTH, split by what each can do:
1. A content mod (`side = "both"`, capability `registries`) for everything declarative: the nine items, their `components` patches, the creative tab, recipes, tags (including additions to
   `minecraft:swords`, `axes`, `pickaxes`), lang files, `sounds.json` and the ogg files, item model definitions and textures. Evidence: content-mods.txt:28-35, :95-111; docs/mods.md:248-262, :317-332.
2. The native mod (same mod id `bmh`, same `.hmod` allowed: docs/native-mods.md "One .hmod for content and code") for all behaviour, through hooks keyed on the item ids. Use the `items` hook ONLY if you want the
   item declarations in Rust (for example to share constants); it adds nothing the JSON files cannot do (docs/native-mods.md:1254). Use `component_types` only if a stack needs a mod data component.
What neither can do: give an item a code class (no override of `use`, `getDestroySpeed`, `appendHoverText`, `overrideStackedOnOther`); tie behaviour to a vanilla item's id (the trident note, docs/mods.md:334-337).
Dedicated server: works only if server and every client run the same build of the mod (content mod hash + native build). Vanilla client: does not work.

---

## 2. Data carry-over

### Behaviour spec
Files in W/common/src/main/resources/:
- `data/bmh/recipe/`: `bundle.json` (shaped, result `minecraft:bundle`, rabbit hide + string), `dormant_totem.json` (shaped, result `bmh:dormant_totem`), `echo_heart.json` (shaped),
  `eternal_bundle.json` (shaped, `T` = `bmh:totem_of_eternity`), `navigator_amulet.json`, and three `minecraft:smithing_transform` recipes `eternal_{sword,axe,pickaxe}_smithing.json`
  (template `bmh:totem_of_eternity`, base `minecraft:netherite_*`, addition `minecraft:echo_shard`, result `bmh:eternal_*`; for example `eternal_sword_smithing.json`).
- `data/bmh/tags/block/echo_sense_ores.json` (`#minecraft:coal_ores` ... `minecraft:ancient_debris`), `data/bmh/tags/item/eternal_tool_materials.json` (`minecraft:gold_block`),
  `data/minecraft/tags/item/{swords,axes,pickaxes}.json` (add the bmh tool).
- `data/bmh/advancement/` (20 files, another group).
- `assets/bmh/items/*.json` (9 item model definitions), `assets/bmh/models/item/*.json` (incl. 32 `navigator_amulet_NN.json`), `assets/bmh/textures/item/*.png` + `totem_of_eternity.png.mcmeta`,
  `assets/bmh/lang/*.json` (10 languages), `assets/bmh/sounds.json` (`end`, `bell_sfx` with `subtitles` keys) and 11 ogg files (`end`, `bell_sfx`, 9 bundle files, which are unused copies of vanilla sounds).

### Horizon mapping
- Recipes: "Everything else uses vanilla 26.3's own formats ... Recipes in data/<id>/recipe/" (content-mods.txt:106-108). `minecraft:smithing_transform` is handled in crates/horizon-client/src/stations.rs:816
  (match arm) with a test at :2040-2055 ("smithing_transform_overlay_matches_optional_inputs_and_takes_template_result"). Ingredients of the same mod (`bmh:totem_of_eternity`) resolve because "Loaders that read only
  data/minecraft/ (recipes, loot, tags) read every namespace, as vanilla does" (docs/modpacks.md "Registries 0.2").
- Tags: "To add to vanilla's (...), put a tag file in data/minecraft/tags/ ... They add to the tag as a data pack does" (content-mods.txt:109). A tag entry naming an unknown id is dropped without failing the rest (docs/mods.md:519-521).
  Reading a block tag from Rust on the server: `native_server_block_in_tag(state, tag)` (crates/horizon-client/src/game/native_world_read.rs:67-73; docs/native-mods.md:918, API 63: "return `NotServer` on a client and
  `TagsUnavailable` before the world loads its tags"); API 28 `native_block_in_tag`/`native_item_in_tag` for tick hooks (docs/native-mods.md:288). These two are NOT in the helper catalog list of native-hooks.json
  (helpers list has no `*_in_tag`); UNVERIFIED whether the typed `Host` exposes them. Check before relying on `bmh:echo_sense_ores` as a tag.
- Lang: `assets/<id>/lang/en_us.json` (content-mods.txt:111). Java has `%%` in `tooltip.bmh.death_wear` ("Loses 7%% ...", W/common/src/main/resources/assets/bmh/lang/en_us.json) because Java's formatter eats `%`. UNVERIFIED how Horizon's lang
  loader treats `%%`: check the rendered tooltip.
- Sounds: `assets/<ns>/sounds.json` + ogg; `native_play_sound` doc: "Another namespace's event (a mod's, `mymod:machine_hum`) is sent by name ... players hear it if their resources define it (`assets/mymod/sounds.json`, a content mod's)" (crates/horizon-client/src/game/native_hooks.rs:10114-10121, API 61).
  So `bmh:end` and `bmh:bell_sfx` work through `sounds.json` without a registry entry. `SoundInit.java` has no Horizon counterpart.
- Item model definitions (crates/horizon-assets/src/item_model.rs):
  - `minecraft:select` with property `minecraft:display_context`: decoded (item_model.rs:735 `"minecraft:display_context" => Self::DisplayContext`; `thirdperson_lefthand` etc. at :187). The Eternal Sword's
    `assets/bmh/items/eternal_sword.json` therefore works unchanged (feature 8).
  - `minecraft:range_dispatch` with `minecraft:bundle/fullness`: decoded (item_model.rs:867). `eternal_bundle.json` works unchanged.
  - `minecraft:range_dispatch` with the custom property `bmh:north` (`assets/bmh/items/navigator_amulet.json`): GAP. `item_model.rs:507-530` calls `foreign::property_is_foreign(m)` and, for a foreign property, chooses the entry
    whose threshold is <= 0 ("A property Horizon cannot read is 0", item_model.rs:~523); `foreign.rs:1` says "Nodes and properties of other namespaces ... decode to the closest vanilla behaviour". The needle would sit on frame `navigator_amulet_00` forever.
    The select property list is also closed: `SelectProperty` has no mod-defined variant and `component_type` accepts only the fixed 122 vanilla names (item_model.rs:277-284), so a custom component cannot drive a `select`.
    Workable rewrite: replace the range dispatch by a tree of `minecraft:condition` nodes. A mod supplies its own namespaced condition property with `Hooks::item_model_property` (docs/native-mods.md:39-66:
    "A client mod supplies its own namespaced `minecraft:condition` property by implementing `Hooks::item_model_property` ... Vanilla properties do not enter the hook"), so 5 conditions (`bmh:needle_bit0` to `bmh:needle_bit4`, 32 leaves) can read the
    camera yaw (`native_camera`, native_hooks.rs:11405) and the stack's `patch` (StackView). Catalog: `item_model_property` v1, side client, in both catalogs. Doc caveat: docs/native-mods.md:66 says "Bundles need hooks 385 once this pending hook is stamped",
    yet the name is already in the Nightly file. UNVERIFIED which is true; test with a Nightly build.
- Pack format and parity: "Resource pack ... outside `format` it is marked too old or too new" (docs/api/content.md:26). The mod's `pack.mcmeta` is not needed for a content mod.

### Verdict
HOOK for recipes, tags, lang, sounds, select, bundle fullness. GAP for `bmh:north` as written; PARTIAL with the condition-tree rewrite (belongs to the amulet group).

### Parity risks
- Textures: `totem_of_eternity.png.mcmeta` (animation) is a resource-pack feature; UNVERIFIED that the item atlas animates it.
- Lang `%%` (above).

---

## 3. Tools never break, wear rules, death wear, tooltips

### Behaviour spec
- `handlers/EternalWear.java:19-84`. Constants: `WORN_DAMAGE_FACTOR = 0.5F` (`:21`), `WORN_MINING_SPEED = 4.0F` (`:24`), `GOLD_REPAIR_COST = 15` (`:27`).
- `isEternalTool(stack)`: item is EternalSwordItem, EternalAxeItem or EternalPickaxeItem (`:32-37`). Bundle and totems are NOT tools.
- `isWornOut(stack)`: tool and `getDamageValue() >= getMaxDamage() - 1` (`:39-42`).
- `capDamage(stack, amount)`: for tools returns `min(amount, maxDamage - 1 - damage)` (`:48-55`), so a tool at 0 points left gets its last point back.
- The cap is applied by `mixin/L_ItemStackWearMixin.java:20-27` on `ItemStack.hurtAndBreak` (both overloads) at HEAD with `@ModifyVariable` on the amount.
- `wear(stack, percentage)`: `damage = (int)(maxDamage * (percentage / 100.0))`; `setDamageValue(min(damage_now + damage, maxDamage - 1))` (`:58-67`). Percentage outside 0..100 throws.
- Death wear: `handlers/KeepItemAfterDeath.java:41-46`: in SURVIVAL only and only if game rule `RULE_DAMAGE_ETERNAL_ITEM_WHEN_DEAD` (`utilities/BMHGameRules.java:35`, default true), each Eternal item is
  `reduceDurability(stack, 7)`. For the bundle and totems `reduceDurability` is a no-op or has no durability.
- Worn out effects: half damage (`mixin/E_PlayerAttackMixin.java:73-74`, `damage *= 0.5` on the one `hurtOrSimulate` call); stone mining speed (`getDestroySpeed` min with 4.0, axe and pickaxe); no ability
  (`E_PlayerAttackMixin.java:88` returns when `bmh$worn`; `E_ServerPlayerGameModeMixin.java:54` ignores a worn pickaxe); the tooltip shows the red `tooltip.bmh.worn_out` line in place of the ability line
  (`items/EternalTooltips.java:30-37`).
- Tooltip lines (`EternalTooltips.java`): ability line DARK_AQUA (`tooltip.bmh.ability.eternal_{sword,axe,pickaxe}`), then `tooltip.bmh.heavy_with_eternity` GOLD, then `tooltip.bmh.death_wear` GRAY (tools only; the bundle shows `item.minecraft.bundle.fullness` GRAY then heavy,
  `EternalBundleItem.java:226-229`). Hint items: one GRAY line (`HintItem.java:30`).

### Horizon mapping
- `item_damaged`: catalog main and Nightly, v1, side server, `mod_api` contract `cancel`. Doc comment (crates/horizon-client/src/game/native_hooks.rs:9207-9217, API 162, docs/native-mods.md:1321): "`amount` is what is left after Unbreaking; change it (0 or
  less leaves the item whole) or set `cancel` for no wear at all. It is not heard for a creative player's tools ... nor for the wear of a mob's armour or of a tool a dispenser uses, nor the host of a LAN world." Signature:
  `native_item_damaged(player: u128, name: &str, item: &str, amount: &mut i32, cancel: &mut bool)`. Note it passes the item ID only, not the stack: to know the current damage the mod must read the held stack
  (`native_carried_stack(slot)` is only valid inside `inventory_tick`, native_hooks.rs:10648-10651) or `native_player_stacks(uuid)` (native_menus.rs:232, "None for a player not here, dead, or whose state is swapped in", and the hook swaps the
  player in, so this returns None). Open question 1.
  The call site is crates/horizon-client/src/game/enchant_effects.rs:119-138 (`fn wear`): it runs only `if loss > 0 && let (Some(uuid), Some(name)) = (self.lan_acting_uuid, self.lan_acting_name.clone())`. In single player (integrated server, the owner) those are not
  set, so the player's own tools are not heard. That is why "the host of a LAN world" is excluded.
- `player_destroy_item`: catalog both, server, "this hook only listens" (native_hooks.rs doc in the catalog `neoforge` field). It cannot prevent a break; useless for "never break".
- `living_incoming_damage_v2`: catalog both, server, cancellable. Docs/native-mods.md:513-526 ("Host-aware weapon damage (API next)"): "It currently fires for player-to-mob melee, sweep and spear hits (including LAN guests), after the
  victim's invulnerability check ... and before v1 ... It is not a replacement for v1 on projectiles, player victims, dragons, mob attacks or environmental damage." Signature (native_hooks.rs:9535-9547, API 384):
  `(entity: i32, kind: &str, damage_type: &str, causing: i32, source_position: Option<[f64;3]>, victim_position: [f64;3], victim_view: [f64;3], weapon: Option<&ItemStack>, amount: &mut f32, canceled: &mut bool)`, with a mutable `Host`.
  `amount` can be halved when `weapon` is a worn-out Eternal tool: HOOK. Does not fire against a player victim (Java `E_PlayerAttackMixin` also scales hits on players): parity gap in PvP.
- Stone mining speed: `native_break_speed(pos, state, original, speed: &mut f32)` (native_hooks.rs:2646-2669). It is a PENDING FRAGMENT: `tools/native-hook-fragments/break_speed.json` has `"name":"break_speed"`, `"side":"both"`; the name is in
  NEITHER the main catalog (S/src/horizon/native-hooks.json) NOR the Nightly (grep count 0). docs/native-mods.md:492-503 documents it: "It isn't asked for an unbreakable block (hardness -1) or in creative mode ... A mod that changes the speed alike on both sides
  keeps the crack in step with the server." Needs the item in hand: the signature has no stack, so the mod must read the held item (client side for prediction too). `Hooks::break_speed` is generated; the bundle must wait for the API stamp.
  Alternative that avoids the pending hook: when a tool becomes worn out, give THAT STACK a component patch of `minecraft:tool` with speed 4.0 and `minecraft:attribute_modifiers` at half damage, from `inventory_tick` (catalog both, API 39, native_hooks.rs:9348-9358:
  "`native_carried_stack` reads and changes the stack ... a count of 0 empties the slot ... This runs often, so check the item first"), and remove the patch after repair. Server side; the client then predicts mining from the synced stack. UNVERIFIED that a per-stack
  `minecraft:tool` patch is honoured by the dig speed code (the content patch is, per docs/mods.md:248-262).
- 7 percent death wear: see feature 6 (the stack is in hand at `loot_drops`; edit `stack.damage`, a public field used in enchant_effects.rs:119).
- Tooltip: hook `tooltip` (catalog both, v1, contract `collect`, out `lines`): `native_tooltip(&self, stack: &ItemStack, lines: &mut Vec<(Text, u32)>)` (native_hooks.rs:8075-8077). Doc (:8068-8074): "An item's tooltip, wherever the game shows one (slots, the creative
  tabs, the recipe book, chat's hover) ... Add, change or take out lines; `Text::legacy` reads `§` codes. Asked each frame the tooltip shows, never for one the stack hides. API 12." Colors are `rgba`; `u32::MAX` keeps the text's own colour. Client side only: the mod
  half on the client needs the lang keys (`Text` translatable, UNVERIFIED API for translatable `Text`; `Text::literal` and `Text::legacy` are the ones seen). The Java comment says tooltips are static text anyway (`EternalTooltips.java:11-13`).

### Verdict
- 3a: PARTIAL. HOOK on a dedicated server and for LAN guests via `item_damaged`, if the mod can read the damage of the item (open question 1). PINNED for single player and the LAN host: inject into `Game::wear` (crates/horizon-client/src/game/enchant_effects.rs:119, a `@ModifyVariable`-like
  head injection on `amount`/`loss` via horizon-mixins; the function is one place because every durability loss runs through it). The wear function is private to that file; the mixin pins one commit.
- 3b: HOOK (`living_incoming_damage_v2`, melee/sweep on mobs only).
- 3c: PARTIAL (pending hook, or the stack-patch workaround, which is HOOK-only but UNVERIFIED).
- 3d: HOOK with the API 405 extension (main-only, see feature 6).
- 3e: HOOK (client).

### Parity risks
- Bar width: Java keeps 1 pixel (`getBarWidth`); Horizon's bar comes from `minecraft:damage`/`max_damage`; a worn-out tool shows an empty bar. The `item_bar` hook exists (catalog both, listen) but "listen" only: UNVERIFIED that it can change the width.
- Repair flat 15 levels: anvil hooks (`prepare_anvil`) are guest-only ("Guest state is not swapped in" style notes); not audited here.
- Hits on players are not scaled in Horizon v2 (see above).

---

## 4. Abilities (all server side in Java)

### 4a. Echo Strike (sword)

#### Behaviour spec
Constants (`handlers/EchoStrikeHandler.java:44-49`): `MIN_ATTACK_STRENGTH = 0.9`, `DAMAGE_RATIO = 0.5`, `DELAY_TICKS = 5`, `RANGE = 5.0`, `MAX_PENDING_ECHOES = 8`.
1. `E_PlayerAttackMixin` (`:52-97`) at the head of `Player.attack` records the main-hand weapon, whether it is worn out, and `getAttackStrengthScale(0.5F)` (`:66`); it scales the damage argument; at the `causeFoodExhaustion` call (only the branch of a landed hit reaches it) it
   calls `EchoStrikeHandler.onSwordHit(attacker, target, bmh$hitDamage, bmh$attackStrength)` for a non-worn sword.
2. `onSwordHit` (`:63-83`): return if strength < 0.9, damage <= 0, or the target is not a `LivingEntity`. Queue (max 8, oldest dropped) a `PendingEcho(dimension, targetUuid, centre of target, damage*0.5, gameTime+5)` on the player (`E_ServerPlayerMixin.java:26-33`).
3. `E_ServerPlayerMixin.java:35-41` at `ServerPlayer.tick` HEAD calls `deliverDueEchoes` when the queue is not empty (`EchoStrikeHandler.java:86-112`): due echoes leave the queue first; an echo whose dimension differs from the attacker's is lost.
4. `deliver` (`:114-139`): origin = the target's current centre if it lives, else the stored origin. `nearestVictim` (`:141-159`): `level.getEntitiesOfClass(LivingEntity, AABB(origin).inflate(5), isValidVictim)`, nearest by squared distance <= 25.
5. `isValidVictim` (`:164-186`): must be `Enemy`; not the attacker, not the original target; alive, not invulnerable, attackable; not Player, ArmorStand, AbstractVillager; not an Ownable pet of the attacker; not the attacker's vehicle or passenger; not allied.
6. Hurt: `victim.hurtServer(level, damageSources().playerAttack(attacker), echo.damage())`. If false (invulnerability frames), nothing more. Else: `drawLine` of cyan dust (`DustParticleOptions` 0.1,0.9,1.0 size 1; step 0.25 block; `:188-205`) plus 6 `ELECTRIC_SPARK`
   at the end, `BmhAdvancements.award(attacker, "eternity/echo_strike")`, sound `AMETHYST_BLOCK_RESONATE` SoundSource.PLAYERS volume 0.6 pitch 1.6 at the victim. The echo never goes through `Player.attack`, so no echo of an echo.

#### Horizon mapping
- Landed hit with damage and weapon: `living_incoming_damage_v2` (catalog both; API 384; above). It fires "after the victim's invulnerability check" so it is a decent stand-in for "hit landed"; it gives `causing` (the player's network id), `kind` of the victim, `weapon`, final `amount`. It does NOT give the attack strength.
- Attack strength: no helper found. Searched native-mods.md and mod_api.rs for "attack strength" / `attack_strength`; the only items are `native_player_arm_swing` (guest swing, API 340, docs/native-mods.md:1681) and the hook `player_attack_cooldown_reset`
  (catalog both, server, native_hooks.rs:5467-5473, "A server player reset their attack strength by swinging (`handlePunch`) ... The player's state is not swapped in"). A mod can keep `last_reset_tick[player]` and compute the scale as the game does:
  `(ticks_since + 0.5) / delay` with `delay = 20 / attack_speed` (crates/horizon-client/src/game/melee.rs:6-14 `attack_strength_scale`). Approximation only: UNVERIFIED how `hurt_enemy`/v2 order against the reset.
- Queue and clock: `server_tick` (catalog both, listen, ServerTickEvent) or `player_tick` (catalog both, server, PlayerTickEvent.Post) plus a mod-owned `HashMap<u128, Vec<PendingEcho>>`; the game time from `level_game_time` / `entity.level(host).game_time(host)` (docs/native-mods.md "Entity and level handles").
  Lost on dimension change: compare `level.dimension` strings.
- Candidates near a point on the server: `native_entities_in(dimension, min, max)` is a pending fragment (tools/native-hook-fragments/entities_in.json; native_world_edit.rs:262-270, "API @API:entities_in@"; "It walks every entity of the level, so call it for a few boxes"). Not in either catalog.
  `client_entities` is client only (catalog helper). Another server query: `native_level_entities(dimension)` exists as a Rust fn (native_world_edit.rs, called by `native_entities_in`) but is not in the helper catalog: UNVERIFIED.
- Filtering: `entity_kind` (catalog helper both) gives `minecraft:zombie`; "is an Enemy" (hostile) needs a list of hostile kinds in the mod, or the entity tag `#minecraft:...`. Ownable pets, vehicle, allied: no helper found (searched `owner`, `tamed`, `ally`, `team` in native-mods.md).
  So the exclusions are partly UNVERIFIED and likely approximated by a hostile-kind list.
- Hurt: `native_hurt_entity(dimension, target: u32, amount: f32, damage_type: &str, attacker: Option<u32>) -> bool` (native_entities.rs:417-428; PENDING fragment tools/native-hook-fragments/hurt_entity.json, "API @API:hurt_entity@"). Doc: "Hurts the living entity `target` (a mob, or a player of the level) ... as `LivingEntity.hurtServer` and `/damage <target> <amount> <damage_type> by <attacker>` do:
  armour, effects, invulnerability time, knockback from the attacker, death and its message, and the damage hooks ... hear it ... Whether it was hurt: false for an invulnerable, dead or creative target". The `bool` matches the Java "invulnerability frames absorb the echo" test. Not in either catalog: next Nightly.
  Without it: `set_mob_health` (catalog helper both; `native_set_mob_health(dimension, id, health)` "false for ... a health that isn't above 0", native_mobs.rs:286) and `kill_mob` bypass armour, knockback and invulnerability frames and are NOT equivalent.
  Careful with recursion: the echo's `native_hurt_entity` call is heard by the damage hooks (doc: "the damage hooks ... hear it"), so the mod must mark its own echo (a re-entrancy flag) to avoid echoing an echo.
- Particles and sound: `native_send_particles` (native_hooks.rs:10141-10170 doc: "Queue particles in this authoritative level ... Block particle types require their block state; item, trail and vibration particles are not supported"); coloured dust: UNVERIFIED (no `DustParticleOptions` seen in the doc, "ParticleOptions" type).
  `native_send_particles_to(uuid, options, state, at, spread, speed, ...)` (native_hooks.rs:10352-10370) exists but is a pending fragment. Sound: `native_play_sound(sound, at, source, volume, pitch)` (native_hooks.rs:10119-10128, API 61): for a `minecraft:` event "must be registered".
- Advancement award: another group.

#### Verdict
PARTIAL. HOOK pieces: hit (v2), queue (server_tick), sound. PENDING-FRAGMENT pieces: `hurt_entity`, `entities_in`. APPROXIMATED: attack strength, "Enemy" filter, pets/allies, coloured dust.

#### Parity risks
Echo may appear on hits Java would not count (strength approximated); victims could differ (hostile list); the PvP case (Java's hit on a player target also feeds `onSwordHit` but the target must be `LivingEntity` and the victim `Enemy`, so players are never echo victims either way).

### 4b. Axe Smash

#### Behaviour spec
`handlers/AxeSmashHandler.java`. `isSmashing` (`:46-54`): `fallDistance > 1.5`, not on ground, not fall-flying, not in water, not on a climbable, not a passenger. `multiplier` (`:35-44`): `min(3.0, 1 + sqrt(2 * 0.08 * fallDistance))`, else 1.
`E_PlayerAttackMixin.java:75-77`: for a non-worn axe the damage argument is multiplied. On a landed hit (`:94-95`, when multiplier > 1): `onSmash` (`:57-81`): award `eternity/smash`; `attacker.fallDistance *= 0.5`; sounds `ANVIL_LAND` (SoundSource.PLAYERS, vol 0.6 pitch 0.7) and
`PLAYER_ATTACK_KNOCKBACK` (vol 1.0 pitch 0.6) at the target; particles `EXPLOSION` x1 at y+0.1, `CLOUD` x `round(10*multiplier)` (spread 0.5/0.05/0.5, speed 0.15) at y+0.1, `CRIT` x same count (spread 0.4, speed 0.5) at y+0.5 of the target.

#### Horizon mapping
- Multiplier: `living_incoming_damage_v2` `amount *= multiplier` (catalog both, API 384).
- Fall distance of a server player in flight: no helper. Searched native-mods.md for `fall_distance`: found only `native_block_fall_on` (docs/native-mods.md:970, API 73, "The landing hook also receives fall distance and mutable calculated damage"), `native_trample` (:2861) and `living_fall` (catalog both, v1, side both,
  `native_living_fall(entity, kind, distance: &mut f64, multiplier: &mut f32, cancel: &mut bool)`, crates/horizon-client/src/landing.rs:28-39, doc "A living entity is about to handle a fall (`LivingFallEvent`). Mods may change the distance and damage multiplier, or cancel fall handling. 357"). These fire only on LANDING.
  So the mod must track the airborne descent itself: `player_tick` + `server_players` position (`NativeServerPlayer.position`, `on_ground` is in the pending fragment `server_player_state`; mod_api.rs:700-701 shows `pub on_ground: bool` on main, "@API:server_player_state@" pending). Peak-y minus current-y approximates `fallDistance`.
- Isn't-in-water / climbing / gliding conditions: `gliding`, `sprinting` appear only in the pending `server_player_state` fragment; water/climbable need `level_block_state` at the player's feet. UNVERIFIED completeness.
- "Break the fall": `fallDistance *= 0.5`: at the next landing `living_fall` can set `*distance *= 0.5` for that player when a smash flag is set. HOOK. (A halving mid-air, before further falling, differs slightly from Java; the landing result is what players feel.)
- Sounds / particles: as above (`native_play_sound`, `native_send_particles` with `ParticleTypes.EXPLOSION`, `CLOUD`, `CRIT`: simple types, fine).

#### Verdict
PARTIAL: HOOK for multiplier, effects, fall softening; the airborne fall distance is mod-tracked (approximation); parity risk of when `fallDistance > 1.5` begins (Java resets on water, ladders, elytra and more).

### 4c. Echo Sense (pickaxe)

#### Behaviour spec
`mixin/E_ServerPlayerGameModeMixin.java:44-74`: at `destroyBlock` HEAD it pushes the block state if the main-hand item is an EternalPickaxeItem and not worn out (else AIR); at RETURN, if the return value is true (the break happened) and the state is in `bmh:echo_sense_ores`, and
`now >= nextSense` (cooldown `COOLDOWN_TICKS = 4`, `EchoSenseHandler.java:69`), it calls `onOreBroken`.
`EchoSenseHandler.java:106-149`: expiry = gameTime + 100. Same-ore predicate (`:171-182`): the first of the 8 vanilla ore families (coal, copper, iron, gold, redstone, lapis, diamond, emerald) that contains the broken state, else the same block (quartz ore, ancient debris; nether gold ore is in gold_ores).
Scan a sphere of radius 8 (`dx*dx+dy*dy+dz*dz <= 64`) around the broken block (`:185-217`), only loaded blocks, sorted nearest first. Existing displays (entities of `Display.BlockDisplay` in `AABB(pos).inflate(9)` with scoreboard tag `bmh_echo`) are re-timed (`setExpiry`); the rest get new displays while the
total stays <= 64 (`MAX_DISPLAYS`). A new display (`spawn`, `:219-239`): `Display.BlockDisplay` at the ore's block position, block state = the ore state, glow colour override 0x00F0FF, transformation translation 0.02 scale 0.96, `setGlowingTag(true)`, tags `bmh_echo` and `bmh_echo_until_<expiry>`, added with `addFreshEntity`.
If at least one new display: award `eternity/echo_sense` and play `AMETHYST_BLOCK_CHIME` (vol 0.8 pitch 0.7, PLAYERS) at the broken block. Cleanup: `E_DisplayMixin.java:16-20` at `Display.tick` HEAD calls `discardIfExpired`, which discards a tagged display when `gameTime >= expiry` or its block is no longer in the tag (`:152-169`).
`E_MinecraftServerMixin` logs the tag size at `loadLevel` TAIL (`EchoSenseHandler.java:89-99`). The glow is a vanilla entity visible to EVERY player (not per-player).

#### Horizon mapping
- Break: `destroy_block` (catalog both, server, cancellable, API 14, native_hooks.rs:8437-8449): fires "once the game allowed it ... where `ServerPlayerGameMode.destroyBlock` starts", i.e. BEFORE the break, with `pos` and `state`. That gives the broken state directly; "if the break really happened" is not told: a later cancel by another mod would still trigger a sense. Acceptable.
  It does not give the player's tool: use `native_acting_player()` + held stack (`native_held` returns `(item, count, plain)` where `plain` is false for a damaged tool, so a stack with damage cannot be matched with `plain` rules; UNVERIFIED how to get the item id only: the tuple's first field is the id).
- Scan blocks: `level_block_state` / `level_block` (catalog helper both; docs/native-mods.md "Entity and level handles": `level.block_state(host, pos)`, `is_loaded`), `read_columns` (catalog helper both) for bulk. Tag test: `native_server_block_in_tag` (see 2; catalog status UNVERIFIED).
- Spawn a display: `native_summon(dimension, "minecraft:block_display", pos, nbt)` (native_entities.rs:215-232; API 59; catalog helper `summon`, both). NBT of a block display: `block_state`, `transformation`, `glow_color_override`, `Glowing` (crates/horizon-client/src/game/persist/display.rs:40-70 saves `glow_color_override`, `transformation`, `block_state`;
  crates/horizon-client/src/entity_nbt.rs:85 handles `Glowing`). Tags: `native_entity_add_tag` / `native_entity_tags` (catalog helpers both, native_entity_data.rs:55-81). Remove: `native_remove_entity(dimension, id)` ("removes such an entity as `/kill` does", docs/native-mods.md "Entities").
- Expiry: the mod keeps its own list (id, expiry tick) and removes at `server_tick`; no need for `Display.tick` injection. Displays saved with a chunk would stay after a crash: do the cleanup at `entities_load` (catalog both) by tag, as Java's tick check did.
- GAP, the glow itself: Horizon does not draw an outline for display entities. Evidence: the outline pass is fed only for players (game/perspective.rs:823-839), dropped items (game/item_draw.rs:726-737), mobs (game/creatures/mob_geometry.rs:173, :311, :373, :619) and projectiles
  (mob_geometry.rs:376); `block_item_display_geometry` (game/text_displays.rs:264-300) never calls `mark_outline`; and `text_displays.rs:58-60` says the render-state ids cover "(not the view range, size or glow)". The client hook `entity_outline` (tools/native-hook-fragments/entity_outline.json: "Drawing only: the entity's `Glowing` flag is unchanged ... make any mob,
  player, dropped item or projectile glow in its own colour") is also pending and names no displays.
  Persisting `glow_color_override` (display.rs:64-65) does not draw anything.
- Alternative, needs a client half: `Hooks::world_primitives` (docs/native-mods.md "World primitives": `box_outline(min, max, color, width)` and `filled_box(...)`, each with `.through_walls()` "(no depth test)", "They blend over the world and write no depth"). Pending fragment (tools/native-hook-fragments/world_primitives.json, 14 lines; fn at native_hooks.rs:2892), not in the main catalog or the Nightly (grep of both: no `world_primitives`).
  The server sends the ore positions with `native_net_send_to_player(player, "bmh:echo", bytes)` (docs/native-mods.md "Mod messages": channel `horizon:mod_net`, 30 000 bytes, 32 messages a tick; "A vanilla client never gets them"). The client keeps the list and draws outlines through walls until expiry. Visible to ONE player (the miner) only, which differs from Java (everyone sees the glow).
- Sound: `native_play_sound("minecraft:block.amethyst_block.chime", ...)`.

#### Verdict
GAP for "block display with glow" (no outline); PARTIAL via client `world_primitives` + `native_net_send_to_player` (both pending or new, needs the mod on the client). Everything except the glow is HOOK.

#### Parity risks
Per-player instead of per-world visibility; the outline style (a box, not the entity outline silhouette); 64 display budget turns into a draw budget.

---

## 5. Eternal Items on the ground

### Behaviour spec
`mixin/G_ItemEntityMixin.java`: `hurtServer` and `hurtClient` return false at HEAD when the stack's item is an `IEternalItem` (`:26-40`); `tick` HEAD on the server calls `itemEntity.setUnlimitedLifetime()` (`:45-53`). The void and `/kill` still remove it.

### Horizon mapping
- Never despawn: `item_expire` (catalog both, v1, side server, listen; API 325). `native_item_expire(entity: u32, stack: &ItemStack, extra_life: &mut i32)` (native_hooks.rs:9243-9255): "A dropped item reaches its lifetime on the server (`ItemEntity.tick`, NeoForge's `ItemExpireEvent`). Every mod may set or add to `extra_life`, in ticks ... a positive total keeps the item that much longer ...
  Extensions are capped at 32766 ticks." Call site (game/item_entities.rs:149-157): `if item.age >= 6000 && item.position.y > -128.0 { ... if extra_life > 0 { item.age = 6000 - extra_life.min(32766); alive = true; }`. Setting `extra_life = 32766` for an Eternal stack resets the age each time it comes back to 6000: an unlimited lifetime in effect. The void rule (`y > -128`) is the same idea as Java's.
  `item_despawn` (catalog both, listen only, native_hooks.rs:6975-6985) cannot cancel; `item_spawn` (catalog both, listen, `native_item_spawn(entity, item, count, pos)` native_hooks.rs:6040-6042).
- Damage immunity: GAP in data terms. Item entity damage is hard-coded: lava (4 per tick unless `resists_fire`), fire, cactus (1 per tick unless the stack's `damage_resistant` names `minecraft:cactus` or one of five tags, entities.rs:483-520, items.rs:693-740) and explosions (everything but `minecraft:nether_star`, game/explosion.rs:728-735, `if &*item.stack.item != "minecraft:nether_star"`).
  `damage_resistant` on the item covers fire (and cactus when written with the right type) but not explosions. There is no hook for item-entity damage: `living_incoming_damage` is for "players, mobs and ender dragon" (docs/native-mods.md "Damage the health takes"). 
  Approximation: on `item_spawn` call `native_entity_merge_nbt(entity: EntityRef, nbt)` (API 386, native_entity_data.rs:36-42; true when changed; "only mobs, dropped items, armor stands and item frames") with `Health:32767s`. The item entity NBT has `Health` (entity_nbt.rs:2790-2793 writes `("Health", Tag::Short(e.health as i16))`, `Fire`, `Age`, `PickupDelay`), so the merge should load it; UNVERIFIED by test.
  The Java `Invulnerable` flag is not read by the item code (search of entities.rs, item_entities.rs: no `Invulnerable`), so `Age = -32768, Invulnerable` (from the brief) does not apply.
- `entity_merge_nbt`/`item_spawn` in both catalogs (helper `entity_merge_nbt` is in the main helper list and not flagged main-only).

### Verdict
PARTIAL: never despawning is HOOK; damage immunity is APPROXIMATE (large `Health`, plus `damage_resistant` for fire and cactus). Explosions will still hurt (each blast removes `amount` health), a long lava soak (4 per tick) will drain 32767 in about 8200 ticks.

### Parity risks
Items can in theory die in a very long lava pool or many explosions; Java never.

---

## 6. Items kept after death

### Behaviour spec
- `neoforge/.../NeoForgeEvents.java:30-38`: `RespawnPositionHandler.onPlayerDeath` at `LivingDeathEvent` HIGHEST (it counts Eternal items for the respawn distance first), `KeepItemAfterDeath.onPlayerDeath` at default priority (`:40-45`), `JourneySignal` LOWEST.
- `RespawnPositionHandler.calculateDistanceFromBed` (`handlers/RespawnPositionHandler.java:215-247`): loops the inventory container (41 slots), counts stacks whose item is `IEternalItem` (each STACK counts 1, a stack of 1), `easyLife = 1 + eternalItemCount + difficulty id`, capped by `MAX_EASY_LIFE`; `distance = min + (max - min) * easyLife / MAX_EASY_LIFE`. (Another group owns the distance; the point here is the ORDER: counting must happen before the items are removed.)
- `KeepItemAfterDeath.onPlayerDeath` (`handlers/KeepItemAfterDeath.java:32-56`): for every slot of the inventory (`getContainerSize()`), if the item is an `IEternalItem`: in SURVIVAL with `RULE_DAMAGE_ETERNAL_ITEM_WHEN_DEAD` call `reduceDurability(stack, 7)`; copy the stack into a list; remove the item from the inventory. The list is stored in a `static HashMap<UUID, List<ItemStack>>` (`:29`) which is NOT saved: a server restart between death and respawn loses the items.
- `onPlayerRespawn` (`:58-74`): `player.getInventory().add(stack)` for each saved stack (first free slot, not the old slot, armour not restored) and the entry is removed.
- `handlers/EternalItemHandler.java`: per tick (`onPlayerTick` `:24-55`): `dropExtraTotems`, then, if rule `RULE_ONLY_CARRY_ONE_ETERNAL_ITEM_TYPE` (default true, `BMHGameRules.java:33`) and SURVIVAL, a second stack of the same Eternal item is dropped (`player.drop(copy, false)`, shrink); `shouldBlockPickup` (`:60-77`) returns true when the player already carries the item (the totem always, the others per the rule): `ItemEntityPickupEvent.Pre.setCanPickup(FALSE)` (`NeoForgeEvents.java:73-79`).

### Horizon mapping
- Death: `loot_drops` (catalog both, v1, side server, `collect` out `drops`): native_hooks.rs:9180-9205 (API 75) and the `NativeLootDrops` doc (mod_api.rs:297-310, API 405): "A dying player's inventory is a call of its own too, after vanishing curse items are gone and before they are thrown (`entity` the player's id and `minecraft:player`, `player` them, `table` empty, `origin` where the items leave, `used` false);
  `drops` empty means the mod took the whole inventory. Not heard with keepInventory. API 405." docs/native-mods.md:1813: "A mod that takes every stack out of `drops` keeps the inventory." The mod removes the Eternal stacks from `drops: &mut Vec<ItemStack>`, edits `stack.damage` for the 7 percent, and keeps them in its own map keyed by uuid.
  API 405 is on main (hook level 415); whether the 2026-11-08 Nightly carries the extension is UNVERIFIED (the Nightly file lists no API numbers). Before 405 there is no hook that sees a dying player's inventory (`living_death` fires "before the loot, experience and death message have run", native_hooks.rs:6635-6650, API 392, but the helpers cannot take stacks from a dead player: `native_player_stacks` returns None for a dead player).
  `keep_inventory` rule: with it on, `loot_drops` is not heard and vanilla keeps everything including non-Eternal items (Java has the same bypass since the event is not cancelled; the Java keep list then also runs: duplicates are possible; ignore).
- Give back: `player_respawned` (catalog both, server, listen, "PlayerEvent.PlayerRespawnEvent") and/or `player_clone` (native_hooks.rs:8695-8707: "Heard before the new player is placed ... with no player's state swapped in"). Then `native_give_stack(uuid, stack) -> Option<ItemStack>` (native_server.rs:1316-1330, API 98): "Gives `stack` whole (its damage, name, enchantments... kept) ... onto equal stacks first, then into empty inventory slots, never the armour or an empty off hand. What did not fit comes back (nothing is dropped): all of it for a player who isn't here, is dead, or whose state is swapped out by another player's". This matches `Inventory.add` (first free slot, no armour). Do it in `player_respawned`, not `player_clone` (the new player is not placed yet). Catalog: helper `give_stack` both.
  If a leftover comes back, drop it with `native_summon("minecraft:item", ...)` or hold and retry next tick.
- Persistence: the Java map is lost on restart. On Horizon, `native_player_persistent_data(uuid)` / `native_set_player_persistent_data` (pending fragment `player_persistent_data`) could store it; or hold in memory like Java for parity.
- One per type: `item_picked_up` (catalog both, server, cancellable): `native_item_picked_up(player, item: &str, count: u32, canceled: &mut bool)` (native_hooks.rs:9229-9236, API 40: "`player` is about to take `count` of `item` from the ground ... Set `canceled` to leave it there: the stack isn't offered again for 10 ticks"). It gets the item id only, no components. Fine for "already carries one of this type": check with `native_count_items`/`native_player_stacks`. Per-tick duplicate drop: `inventory_tick` (catalog both) over slots `0..35,100..103,-106` using `native_carried_stack(slot)` (change/empty it) and `native_summon("minecraft:item", ...)` for the dropped copy. `player_tick` also exists.
- `native_count_items` and `native_carried_items` count only "plain" items (native_server.rs:1253-1262): a damaged tool is not plain, so use `native_carried_stack` inside `inventory_tick` instead.
- Respawn distance count: the BMH respawn code needs the Eternal-item count BEFORE `loot_drops` removes them. `living_death` (API 392, cancellable, fires before the loot) is the place to count (other group).

### Verdict
HOOK, with one main-only extension (API 405) for the whole death flow. Without 405: GAP (nothing sees the dying player's inventory).

### Parity risks
`loot_drops` of a dying player is documented "after vanishing curse items are gone"; Java ran before vanishing so a vanishing Eternal item stayed in Java's saved list. Horizon: it vanishes. Armour slots: Java returned armour pieces (none are Eternal) so irrelevant. Hand slot: `give_stack` puts the item in the first free slot (the same as Java's `Inventory.add`).

---

## 7. The Eternal Bundle

### Behaviour spec
`items/EternalBundleItem.java` (256 lines). Weight capacity 64 (`MAX_WEIGHT`, `:44`); bar colour RGB(102,102,255) (`:47`). Properties give it `BUNDLE_CONTENTS` = EMPTY (`ItemInit.java:23`).
- `overrideStackedOnOther` (`:59-94`, item clicked ONTO a slot): only if `stack.count == 1` and `ClickAction.SECONDARY` (right click). Empty slot: remove one stack from the bundle (`removeOne`) and `safeInsert` it into the slot; non-empty slot: `canInsert(slot item)` then `contents.tryTransfer(slot, player)`; sounds `BUNDLE_REMOVE_ONE` / `BUNDLE_INSERT` (vol 0.8, pitch 0.8 + rand*0.4).
- `overrideOtherStackedOnMe` (`:96-138`): the cursor stack onto the bundle, right click, `slot.allowModification`: empty cursor takes one out; otherwise `tryInsert` if `canInsert`.
- `use` (`:141-158`): `dropContents` all at once (`BUNDLE_CONTENTS` reset, `player.drop(stack, true)` for each on a ServerPlayer), `BUNDLE_DROP_CONTENTS` sound, `ITEM_USED` stat, SUCCESS; FAIL if empty. (The comment `:140` says vanilla 26.1 drops one at a time while used.)
- `canInsert` (`:177-183`): non-empty, `canFitInsideContainerItems`, NOT an `IEternalItem`, NOT an Ender Chest.
- `isBarVisible` when contents non-empty (`:160-163`), `getBarWidth = min(1 + mulAndTruncate(weight, 12), 13)` (`:165-168`).
- Tooltip image `BundleTooltip(contents)` (`:213-216`); text `item.minecraft.bundle.fullness` with `mulAndTruncate(weight, 64)` / 64 plus the heavy line (`:226-229`).
- `onDestroyed` spills contents (`:232-238`). The Eternal bundle itself drops on death like any Eternal Item (feature 6).
- Model: `assets/bmh/items/eternal_bundle.json` range_dispatch `minecraft:bundle/fullness` threshold 1e-07 -> `bmh:item/eternal_bundle_filled`.

### Horizon mapping
- Base: a content item with template `minecraft:bundle` and `rarity: epic`, `max_stack_size: 1`, `damage_resistant is_fire`. Bundle behaviour comes with the component: `bundle: c.get("minecraft:bundle_contents").is_some()` (crates/horizon-items/src/items.rs:943) and `BundleContents.Mutable` clones in crates/horizon-items/src/bundle.rs:1-4 ("The menus run `BundleItem`'s click overrides on it"). Insert, remove, weights, the fullness model, the bar and the tooltip image come from the template; the destroyed-in-fire spill is generic (entities.rs:529-535 `crate::bundle::destroyed`, "the component the stack holds is the dispatch ... which a mod's own item carries in the same way").
- Not insertable: Eternal Items and ender chests. No hook edits the bundle's insert rule. Closest: `inventory_click` (pending fragment `tools/native-hook-fragments/inventory_click.json`: `native_inventory_click(player, name, window, slot, button, mode, cancel)`, "a guest's click in their own inventory ... or a vanilla menu they have open, heard before it is applied ... Set `cancel` and nothing moves"; guests only, so NOT the single-player host) to cancel a click that would put an Eternal Item or ender chest into the bundle, plus
  `menu_clicked`? No: that is for the MOD's own menus. `player_bundle_item_select` (catalog both, listen only, native_hooks.rs:6893-6900, API 341) only hears the selection. Alternative: make the Eternal Items non-insertable by a component: vanilla's rule is `canFitInsideContainerItems`; check whether Horizon honours a component for it. UNVERIFIED (searched `can_fit_inside`, no hit). Items in the vanilla `#minecraft:bundle_blacklist`-style tag: UNVERIFIED (searched "bundle" tags in docs, no hit).
- Drop everything at once on use: `use_item` (catalog both, server, cancellable, `PlayerInteractEvent.RightClickItem`, contract `cancel` out `claimed`) can claim the right click on the bundle; then the mod must clear the bundle's contents and drop each stack: clear needs `native_set_inventory_slot` (pending fragment) or `inventory_tick` + `native_carried_stack`; drop with `native_summon("minecraft:item", ...)`.
  Not tested whether vanilla 26.3 drops one at a time; docs mention nothing. UNVERIFIED.
- Tooltip text "x/64": the vanilla fullness line already exists in the template; the extra heavy line via `tooltip` hook.
- Kept on death: as in feature 6.

### Verdict
PARTIAL. HOOK: bundle behaviour (component template), tooltip heavy line, model. PENDING FRAGMENT / GUEST-ONLY: refusing Eternal Items inside (`inventory_click`). UNVERIFIED: drop-all-at-once parity.

### Parity risks
A player could put Eternal Items (and an ender chest) into an Eternal Bundle in single player, which the Java mod forbids (the point: an Eternal item inside a bundle escapes the "heavy" count and duplicates rules).

---

## 8. Creative tab and the Eternal Sword's two sprites

### Behaviour spec
Tab: `init/CreativeTabInit.java:16-32` (title `itemGroup.BMH_tab`; the lang file also has `itemGroup.bmh_creative_tab`, `assets/bmh/lang/en_us.json` lines 7-8). Sword: `assets/bmh/items/eternal_sword.json` is a `minecraft:select` on `minecraft:display_context` with one case for `thirdperson_lefthand`,
`thirdperson_righthand`, `firstperson_lefthand`, `firstperson_righthand` -> model `bmh:item/eternal_sword_in_hand` (the 20x20 sprite, `assets/bmh/models/item/eternal_sword_in_hand.json`), fallback `bmh:item/eternal_sword` (16x16, parent `minecraft:item/handheld`).

### Horizon mapping
- Tab: content mod file `data/bmh/horizon/creative_tab/bmh.json` = `{ "title": "itemGroup.bmh.bmh", "icon": "bmh:totem_of_eternity", "items": ["bmh:eternal_axe","bmh:eternal_bundle","bmh:eternal_pickaxe","bmh:eternal_sword","bmh:navigator_amulet","bmh:totem_of_eternity","bmh:sleeping_totem","bmh:dormant_totem","bmh:echo_heart","minecraft:bundle"] }` (docs/mods.md:317-332). Lang key rule: "a lang file's `itemGroup.<mod id>.<file name>` (`creative_tab/gems.json` -> `itemGroup.ruby.gems`) translates any title" (docs/mods.md:329-331). Tabs "come after vanilla's tabs, ten to a page" (:325-327). Whether a vanilla id like `minecraft:bundle` is accepted in `items`: only "items the game doesn't have are left out" is stated (:318-319); UNVERIFIED for vanilla ids.
- Two sprites: `select` on `display_context` decoded: item_model.rs:735 (`"minecraft:display_context" => Self::DisplayContext`), context names at :187 (`ThirdPersonLeftHand => "thirdperson_lefthand"`). The same asset works unchanged.

### Verdict
HOOK (data). No Java hook-by-hook port needed (`H_ItemRendererSwordMixin` is already gone from `mc-26.1`, CLAUDE.md "no mixin, no EternalSwordModel").

### Parity risks
20x20 sprite scale in hand: Horizon's `handheld` parent and display transforms should match vanilla; UNVERIFIED visually. The creative tab appears in a paged strip after vanilla tabs, not as a tab in the vanilla order.

---

## Open questions

1. `item_damaged` passes only the item id (`item: &str`), not the stack. To cap damage at "maxDamage - 1 - damage" the mod needs the current damage. Inside the hook "The player's state is swapped in, so `native_acting_player`, `native_tell` and the item helpers reach them" (native_hooks.rs:9211-9214). I did not find a helper that returns the damaged stack from there
   (`native_held` returns `(item, count, plain)`; `native_carried_stack` is documented for `native_inventory_tick`). If none exists, the simplest cap is: set `amount = 0` for ANY wear on an Eternal tool and apply the real wear yourself from `inventory_tick`/`hurt_enemy` by patching `stack.damage` with a cap. That changes nothing visible. Needs a test.
2. Single-player coverage: which of these server hooks fire for the owner of a single-player world? `item_damaged` does not (enchant_effects.rs:128-130). `loot_drops`, `destroy_block`, `hurt_enemy`, `living_incoming_damage_v2`, `item_expire` say "every player of the world" or "(including LAN guests)", which I read as including the host, UNVERIFIED for each.
3. Is API 405 (dying player's inventory in `loot_drops`) in the Nightly the mod will pin? The Nightly JSON carries no API number. Compare `NATIVE_HOOKS_API` of the Nightly's native_hooks.rs (mod-code.txt:65: "The current Nightly's version is NATIVE_HOOKS_API in its native_hooks.rs").
4. Axe stripping and the sweep via a netherite template: whether the template carries `minecraft:block_transformer` and `minecraft:weapon` in 26.3 defaults (prepared `items.json` not in the repo).
5. How Horizon's own native mods register items with code (S/src/horizon-mods/native/): not opened in this audit. The docs say `items` takes only `horizon_mods::content::Item`.
6. Vanilla client with a content-mod server: the exact failure (disconnect on the first modded stack, or on join) is not tested.
7. `Host` exposure of `native_server_block_in_tag` (tag lookups) and `native_level_entities` (entity enumeration) in the typed API.
8. Dust particle colour (Echo Strike line) through `native_send_particles`.
9. `inventory_click` and `container_accepts` are guest/mod-container only; no hook found for the single-player host's bundle clicks.

## Step log

1. Ran the tree guard: `git -C W rev-parse --show-toplevel` printed W; branch `horizon-26.3`.
2. Read W/common/.../init/{ItemInit,BMHModTierInit,CreativeTabInit,SoundInit}.java and items/{EternalSword,EternalAxe,EternalPickaxe,Hint,IEternal,EternalTooltips}Item.java.
3. Read handlers/{EternalWear,KeepItemAfterDeath,EternalItemHandler}.java, mixin/{L_ItemStackWear,G_ItemEntity}Mixin.java, items/EternalBundleItem.java.
4. Read EchoStrikeHandler, EchoStrikeQueue, E_PlayerAttackMixin, AxeSmashHandler, EchoSenseHandler, E_ServerPlayerGameMode/E_Display/E_DisplayInvoker/E_BlockDisplayInvoker/E_MinecraftServer/E_ServerPlayer mixins.
5. Listed W/common/src/main/resources and read the item model, recipe, tag, sounds and lang samples.
6. Read S/horizon-docs/content-mods.txt, S/src/horizon/docs/api/content.md, docs/mods.md (item, creative tab, server sections), docs/modpacks.md "Registries 0.2".
7. Located the item declaration struct in crates/horizon-mods/src/content.rs:408 and the 122 component names in crates/horizon-assets/src/item_model.rs:28.
8. Read docs/native-mods.md for API 145, 65, 79, 5081 ("Items' default components"), 39-66 ("Conditional item model properties"), 492 ("How fast a player digs"), 726 ("Data files"), 2141 ("Standalone server export"), 3819 ("Entities"), 4580 ("Mod messages"), 4770 ("World primitives").
9. Loaded both catalogs (native-hooks.json main, Nightly 2026-11-08) with python3 -I and compared hook and helper names; found main-only names (`force_chunk*` only) and checked pending fragments under tools/native-hook-fragments/ for `break_speed`, `world_primitives`, `entities_in`, `hurt_entity`, `entity_outline`, `inventory_click`, `server_player_state`, `player_died_v2`, `set_inventory_slot`.
10. Read doc comments in native_hooks.rs for item_damaged, item_expire, item_despawn, item_spawn, loot_drops, hurt_enemy, player_died, player_clone, living_death, living_incoming_damage_v2, critical_hit, destroy_block, attack_entity, tooltip, native_play_sound, native_send_particles(_to), native_set_item_entity, native_summon, native_entity_merge_nbt, native_set_mob_health, native_kill_mob.
11. Read Horizon item entity code: game/item_entities.rs (expiry), entities.rs (environment_tick, hurt), game/explosion.rs (items), items.rs resists_*; melee.rs (sweep tag); enchant_effects.rs (wear); bundle.rs and items.rs:943 (bundle by component).
12. Checked display glow: game/glowing.rs, game/text_displays.rs, persist/display.rs, `mark_outline` callers; found no outline for block displays.
13. Checked item model decoding: item_model.rs lines 495-530, 620-740, foreign.rs (range_dispatch foreign property reads 0).
14. Checked join rules: lan/conn.rs:1325-1345 and 1395-1460, content_identity.rs, protocol/mod_list.rs, docs/mods.md:100-120 and :400-410, S/horizon-docs/multiplayer.txt.
15. Read the Java NeoForge event wiring (NeoForgeEvents.java:30-95) and RespawnPositionHandler.calculateDistanceFromBed to fix the order of counting and removing Eternal Items.
16. Wrote this report.
