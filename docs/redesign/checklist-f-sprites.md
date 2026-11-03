# Checklist: part F, sprites

What to look at in game (`runClient`, any loader, any version: the four textures are the same files everywhere).
Review sheets of what you should see: `docs/redesign/sprites/review-sheet.png`, `before-after.png`, `family.png`.
Look at each item at GUI scale 2, 3 and 4 (Options, Video Settings, GUI Scale).

1. `/give @p bmh:eternal_bundle`. In the inventory: your dark bag and tan cord exactly as before, plus a small gold
   ring hanging under the knot of the cord, bottom centre. From GUI scale 2 up, the ring shows a dark hole.
2. Put any item into that bundle (pick the bundle up and right-click it onto an item). The bag turns into your full,
   tied-neck bag with the same gold ring, now hanging under the end of the cord, in the same place on the slot. If the
   bundle keeps looking empty while it holds something, that is the `filled` property (part A), not the texture.
3. Hold the bundle in first person, in third person (F5), and drop it (Q): the ring is a gold bump on the item, no
   stray or see-through pixels around it.
4. `/give @p bmh:dormant_totem` and `/give @p bmh:totem_of_eternity`, side by side. The dormant one never sparkles
   and never moves: eyes closed (two dark lines), forehead gem dull grey-violet, the rest of the body identical to the
   awakened totem's first frame. The awakened one keeps its animation, its green and cyan eyes and its violet gem.
5. `/give @p bmh:echo_heart`. A violet heart jewel, its left lobe lit with a small white glint, and a cyan four-point
   star glowing in a dark diamond-shaped window at its centre. Check it in the hotbar at night or underground (dark
   background): the whole heart, point included, should stay readable. Put one in an item frame too.
6. Any item showing a magenta and black checkerboard means its model points at a missing texture: the models are
   part B's (`models/item/dormant_totem.json`, `echo_heart.json` must use `bmh:item/dormant_totem`,
   `bmh:item/echo_heart`).
