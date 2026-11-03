# Checklist: part B (Totem of Eternal Life, recipes, tooltips)

What only eyes in the game can confirm. The server side (anvil result and cost, smithing data, recipes, repair,
tooltip lines) was already exercised headlessly: see `log-b-totem.md`. Give yourself items with the commands below
(creative or `/gamemode survival` + `/xp set @s 39 levels` where a step needs levels).

Textures of the Dormant Totem and the Echo Heart are placeholders (the Totem of Eternal Life's first frame, twice) until
part F's sprites are merged: at first both new items look like the totem.

The creative tab's title used to show the raw key `itemGroup.BMH_tab`; the integration fixed it (the lang file now has that key).

## 1. The two new items

1. `/give @p bmh:dormant_totem` and `/give @p bmh:echo_heart`.
   - Names "Dormant Totem of Eternal Life" and "Echo Heart", aqua (rare). Each has one gray tooltip line that tells where
     the two go in an anvil (Dormant Totem left, Echo Heart right).
   - Both show in the BMH creative tab, after the Totem of Eternal Life.
2. Drop either one in lava: it does not burn (fire resistant).
3. Keep both in your inventory and die in survival: they drop where you died like any item (they are not Eternal
   Items), while the Totem of Eternal Life, the tools and the bundle come back with you.

## 2. Recipes (crafting table, survival so the recipes are really used)

4. Dormant Totem: netherite ingot, amethyst block, netherite ingot / diamond block, totem of undying, emerald block /
   (empty), netherite ingot, (empty) gives the Dormant Totem (the old Totem of Eternal Life recipe).
5. Echo Heart: crying obsidian in the four corners, echo shards in the four edges, a nether star in the middle.
6. The old crafting recipes are gone: a netherite sword + netherite ingot + Totem of Eternal Life (old sword pattern) gives
   nothing; the dragon egg pickaxe recipe gives nothing.
7. Eternal Bundle: string, gold ingot, string / leather, Totem of Eternal Life, leather / leather, netherite scrap, leather.
   A vanilla bundle recipe still works (the `bundle.json` rabbit hide one).

## 3. The anvil

8. `/xp set @s 38 levels`. Dormant Totem in the left slot, Echo Heart in the right slot: the result is a Totem of
   Eternal Life, the cost reads 39 and is red / not takeable at 38 levels.
9. `/xp set @s 39 levels`: take it. Both inputs are gone, you are back to 0 levels, the anvil may take its usual damage.
10. Type a name in the name field before taking it: the result is not renamed.
11. Echo Heart on the left, Dormant Totem on the right: no result.
12. The awakened totem glows (enchantment glint) and its tooltip reads: `Eternity` (gray), then
    `Heavy with eternity: you wake farther while you carry it` (gold).
13. Try to put the awakened totem in a grindstone: both slots refuse it (a grindstone only takes damageable or enchanted
    items), so the glint cannot be stripped. It is not an enchantment.

## 4. Eternal tools at the smithing table

14. 1.20.4 (NBT):
    `/give @p minecraft:netherite_sword{Enchantments:[{id:"minecraft:sharpness",lvl:5},{id:"minecraft:mending",lvl:1}],Damage:100,display:{Name:'{"text":"Test blade"}'}}`.
    1.21.1 (components):
    `/give @p minecraft:netherite_sword[enchantments={levels:{"minecraft:sharpness":5,"minecraft:mending":1}},damage=100,custom_name='"Test blade"']`.
    Same for `netherite_axe` and `netherite_pickaxe`.
15. Smithing table: Totem of Eternal Life in the template slot, the netherite tool in the base slot, an echo shard in the
    addition slot. The result is the Eternal Sword / Axe / Pickaxe with Sharpness V, Mending, the name "Test blade" and
    the same wear (damage 100 out of 10000). Taking it consumes the totem, the tool and the shard.
16. A diamond tool, or a diamond instead of the echo shard, gives nothing. The Dormant Totem is not accepted as template.
17. Tooltips of the three tools: a dark aqua ability line (added when the parts were merged), `Heavy with eternity: ...`
    (gold), `Each death: -7% durability` (gray), then the enchantments.

## 5. Repair and bundle

18. `/give @p bmh:eternal_sword{Damage:5000}` (1.21.1: `bmh:eternal_sword[damage=5000]`). Anvil with a gold block: the
    sword is repaired by 2500 (25%). With a Totem of Eternal Life: no result.
19. Eternal Bundle tooltip: the contents preview right under the name (as in vanilla), then the fullness line (gray), then
    the gold `Heavy with eternity` line. Part A fixes the filled texture separately.
20. Die with all of it: items return, tools lose 7% durability, as before.

## 6. Open points for Louis (decisions or work that part B did not take)

- Settled by the coordinator: the awakened totem keeps Louis's name "The Totem of Eternal Life" and the new item is "Dormant Totem of
  Eternal Life" (lang, changelog and this checklist say so). The README still says "Totem of Eternal Life" in its Totem section.
- Settled by the coordinator: each tool has a dark aqua ability line above the gold one (Echo Strike, Smash, Echo Sense), added
  when the parts were merged.
- README: the crafting-grid pictures for the totem, the three tools and the bundle (`resources/*_crafting_grid.png`) show
  recipes that no longer exist, and the Totem section still says it is "only used to craft other eternal items". It needs a
  rewrite and new pictures at release time.
