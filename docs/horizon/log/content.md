# Content mod in the game, 2026-11-17

## Steps

1. Read the loader. `horizon-client/src/mod_content.rs`, `patch_components`: a component type is accepted only if a vanilla item has it
   in `items.json` or it is a Horizon custom component. No vanilla item has a default `minecraft:lodestone_tracker`. Other component
   keys of the nine items (`max_damage`, `enchantable`, `repairable`, `damage_resistant`, `attribute_modifiers`, `tool`,
   `enchantment_glint_override`) were not refused.
2. Removed the `components` block of `data/bmh/horizon/item/navigator_amulet.json`. Nothing else changed.
3. `sh horizon/tools/test-slice.sh --no-build --only a --keep`: a1 to a4 PASS. The game log has `Horizon: content from 1 mod(s) in ...`
   and no line with `content not loaded`. Searching the log for "bmh", "content", "warn" and "error" gives only the rule chat lines and
   `no sound: audio off` (audio is off on purpose). No second refusal.
4. Own run (a launch like the script's, scratch `.../scratch/content-run`, driven with horizon-ctl):
   - `give bmh:<id>` for the nine ids: each reply `{"given":1,"ok":true}`; `inventory` counts show the nine ids (`echo_heart` appears after 10 ticks).
   - Names: the give chat lines show `[The Eternal Pickaxe]`... `[Eternal Bundle]`, `[Navigator's Amulet]`, `[The Totem of Eternity]`, `[Dormant Totem]`,
     `[Hollow Totem]`, `[Echo Heart]` (shots/creative.png). The creative tab title reads "BMH Tab".
   - Screenshot `shots/creative.png` (looked at): hotbar with the nine items. Every texture shows; none is the missing-texture pattern.
     Sword (green gem), axe, pickaxe, bundle (dark bag with a gold clasp), amulet (compass needle, red), totem of eternity (purple, glint),
     dormant and hollow totems (dark), echo heart (purple heart). Two toasts "A hollow promise" and "Heart of an echo": advancements fired from the items.
   - Screenshot `shots/page2.png` (looked at): creative tab "BMH Tab" on page 2 with the nine items and the vanilla bundle.
   - Recipe: `recipe give @s bmh:echo_heart` and `bmh:navigator_amulet`: "Unlocked 1 recipe(s)". Crafted for real: crafting table, grid slots
     100-108 with 4 crying obsidian, 4 echo shards and a nether star, `click-slot 200 shift` gave `bmh:echo_heart` (inventory counts `{'bmh:echo_heart': 1}`).
   - Advancement: `advancement grant @s only bmh:root` answered "already have it" (given by the items), `revoke` answered "Revoked the advancement [Balanced Hardcore]".
     `grant bmh:journey/exile` gave "Granted the advancement [Not the end]", `revoke` revoked it.
   - Sounds: `playsound bmh:bell_sfx master @s` gave "Played sound bmh:bell_sfx to Player38078"; the same for `bmh:end`.
   - The game was quit through `horizon-ctl quit`; `pgrep -fl horizon-client` printed nothing after every run.
5. Full `sh horizon/tools/test-slice.sh --no-build`: 16 PASS, 2 FAIL (b0, b4). Lines:
   `PASS a1, a2, a3, a4, b0s, b1, b2, b3, c1, c2, d3, e1, e2, e3, f1, g1`; `FAIL b0` (9.2 blocks from the world spawn, known respawn defect);
   `FAIL b4` (below). SKIP none.

## Still wrong (native code, not touched)

- b4 fails. Log (second run, `--only b --keep`): `Sound command failed: playsound bmh:bell_sfx player @a[name=!Player53267] -4894 69 22813 1 1`
  and the same for `bmh:end`; and, on the second respawn only, `Advancement bmh:journey/exile not awarded to Player53267`.
  Cause: the sounds exist (step 4). The command targets every player except the arriving one, and the test world has one player, so the
  game answers that nobody hears it (own probe: `playsound ... @a[name=!Nobody] 0 70 0` answers "The sound is too far away to be heard"). The warning
  is the native code logging a command failure that no player is there to hear. The advancement is awarded the first time (g1 PASS, flag `done`
  in the player's advancements file); the warning on the second respawn is the grant answering "already have it" (own probe: second grant says so).
  So the b4 check cannot pass in a one-player world with the current native code. I did not weaken it.
- b0: respawn code, as before.
- The amulet needle spins at random (no tracker).
- Not tested: crafting the amulet, the bundle's recipe, lang `%%`, animation of the totem texture, a lodestone click on the amulet.

## For other areas

- Amulet area: in `native/mixins/bmh/hook_modify_default_components.rs`, add to `bmh:navigator_amulet` the component
  `minecraft:lodestone_tracker` with target dimension `minecraft:overworld`, pos [0, 64, -30000000], `tracked` false. The needle model
  (`assets/bmh/items/navigator_amulet.json`, vanilla `minecraft:compass`, target `lodestone`) then points north. Without it the needle spins.
- Respawn area: do not log a failed arrival `playsound` or an already-held advancement as a warning when no other player is near (or make b4 count them separately).
