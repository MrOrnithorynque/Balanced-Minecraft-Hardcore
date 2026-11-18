# Game checks 3 (wave 2 and 3, first part: totem, homecoming)

Tip when I started: 6cd1e3e (clean tree). Scenarios `t` (totem) and `u` (homecoming) in `horizon/tools/test-slice.sh`,
rows in the table of `docs/horizon/TESTING.md`. The sound is never tested (all runs are muted). No screenshot: every
check reads a file, a log line or a `horizon-ctl` reply.

## Steps

1. Read the brief, the two area logs, the helpers of test-slice.sh. Checked that no `horizon-client` runs.
2. Scenario `t`, first block (t1 to t5): give a Totem of Eternity, `/damage @s 100 minecraft:generic`. Result: the
   player died (health 0, screen `dead`, inventory empty), no `saved` line, no `The game refused the save` line.
3. To tell "the swap never ran" from "the hook never ran", I added a temporary log line (`TMPDBG`) at the top of
   `on_living_death` and in `on_player_tick` before `complete_save` (bmh_totem.rs), rebuilt, ran with `generic` and with
   `minecraft:fall` (`T_TYPE=minecraft:fall`). The binary held the string (`strings` found it). The log held no `TMPDBG`
   line at all in both runs: `living_death` is not called for the death of the single-player owner. The temporary
   lines are removed (`git checkout` of the file). Run logs of that time: scratch only.
4. Source read in the build stage copy (`target/stage/bmh/crates/horizon-client/src/game`): `native_living_death` has
   two callers only, `creatures.rs:1569` (mobs) and `lan_actions.rs:4309` (`guest_die`, a LAN guest). `guest_die` has
   no caller for the owner in a world that is not open to LAN.
5. Other totem checks written and run (t6 to t10). All pass except the save itself.
6. Scenario `u` (homecoming): one temporary log line in `on_server_tick` showed the record present until the player
   came home and gone after (`rec=true` then `rec=false`), so the homecoming works; my first checks read the title too
   late (it expires). Fixed the test (poll the log line `is home after`, read `messages` right after), and use
   `advancement revoke @s only bmh:journey/home_at_last` as the test of the award (`advancement test` does not exist
   here). The temporary line is removed (`git checkout`).
7. Full run at the end: see the last section.

## Verdict on the dead-flag fault (difference 12 of totem.md)

NOT SETTLED, and the question is moved. The game never reaches the dead flag for the owner: `living_death` does not
fire for a single-player owner at all, so the Totem of Eternity does not save him (t1 to t5 red). The dead flag may still
be real for a LAN guest (the only caller), but I could not test a guest. Evidence: `strings` finds `TMPDBG` in the
binary, the game log of the three runs has no `TMPDBG living_death` line while the player died by `/damage` and by a
fall; `lan_actions.rs:4309` and `creatures.rs:1569` are the only callers. Item 2 of "To check in the game" of
totem.md ("living_death fires for the single-player owner") is refuted.

## Checks

Totem (`t`): PASS t6 (Totem of Undying saves first: health 5.0, effects regeneration and absorption, no fire resistance,
Totem of Eternity kept), PASS t7 (second totem dropped, one kept), PASS t8 (anvil: Hollow Totem + Echo Heart gives
`bmh:totem_of_eternity`, `anvil_cost` 39), PASS t9 (E6: Eternal Sword damage 20000 penalty 40 + 2 gold blocks gives
damage 3500, `anvil_cost` 15, penalty 40 kept), PASS t10 (`/kill` kills a carrier).
FAIL t1, t2, t3, t4, t5 (the save on a lethal hit: the player dies, no Dormant Totem, no effects, no `saved` line, and
t5 has no live player to hit). Not tested: the extra totem in a chest, a hopper (needs a chest UI flow), the void (it
cannot save by design and the save is broken anyway), typing a name in the anvil, a LAN guest, tooltips.
Homecoming (`u`): PASS u1 (record `1 homes (0 amulet points), 11111 blocks`), u2 (title `Home`, subtitle `11111 blocks
since you left`), u3 (advancement `Home at last` held), u4 (rule false: no title, advancement still held, the
homecoming log line present). Not tested: the bell, a second player, `The long way home` (needs 30 000 blocks; the test
exile is 11 111), the amulet point title, a Nether exile, a restart with a record, the nights count, a Java record.

## Defects left

| Area | Severity | What happens | Expected | Evidence | Command |
|---|---|---|---|---|---|
| totem | major | The owner dies with a Totem of Eternity; `living_death` is never called for him | Alive at 1 health, Dormant Totem, three effects | t1 to t5 red; no `TMPDBG` line; callers only in `creatures.rs:1569`, `lan_actions.rs:4309` | `sh horizon/tools/test-slice.sh --only t --no-build` |
| totem | unknown | The dead flag of a canceled `living_death` on a LAN guest | Not reachable from the owner | not tested | none |

This needs a Horizon change (call `living_death` for the owner, and clear the flag on a cancel) or another hook. I did not fix it.

## Last full run

First full run: 91 passed, 11 failed. The extra reds (u2 to u4, m2, j2) came from my scenarios: they left the
difficulty Peaceful (the distance formula then differs) and u teleported to where the player stood, not to the
respawn point. Fixed: `difficulty normal` at the end of t and u, `spawnpoint @s` at the start of u.
Second full run (last line): `test-slice.sh: 95 passed, 7 failed, 0 skipped`. The seven reds: t1 to t5 (the totem
save, above), `n5` (immediate_respawn: 0.7 blocks, one WARN line, known from game-checks-2) and `m2` (16674.6 blocks
instead of 23333, also red in game-checks-2 before the p fix; not investigated this part).
No game was left running; the scratch folders are removed.
