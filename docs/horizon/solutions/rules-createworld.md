# Workarounds for S1 (game rules) and S2 (Create World choice)

Date: 2026-11-19. Branch: `claude/gap-rules-createworld-1e413e` (cut from `horizon-26.3` at 5da909a). Nightly: fdc87aec. `NT/` = the Nightly's tree, `NT/crates/horizon-client/src/game/` = `G/`.

## Verdict

| Gap | Verdict |
|---|---|
| S1, six `bmh:*` rules | Closed for the player at the keyboard, open for three details. `/gamerule bmh:<rule> [true\|false]` typed in the chat now works for the owner and for LAN guests, with vanilla's answer texts and the `/gamerule` level (2). The value is kept in the world. Still different: no completion or colour inside `/gamerule`, no row in a Game Rules screen, and command blocks and functions do not reach it. |
| S2, Create World choice | Closed except for three details. The button sits in the Game tab (earlier fix rounds). This round adds the grey-out in Hardcore and debug worlds, and Re-Create shows the old world's state. Still different: not a child of the tab, not synced with the More tab list, no click sound. |

Everything below is proved in the game unless the text says "not run".

## S1. What was tried

### R1. A `gamerule` branch in the mod command tree. Refused.

`command_tree` drops every usage whose first word vanilla has: "`{usage}`: vanilla has /{name}" (`G/native_server.rs:146-148`). The doc of `native_register_commands` says the same: "A name vanilla has is left out" (`G/native_hooks.rs:4313`). A mod cannot add a branch to `/gamerule`.

### R2. A data pack or a content mod rule. No such thing.

The rule table is a `const` (`NT/crates/horizon-client-core/src/gamerules.rs:50` `RULES`, `:132` `available_rules`, `:138` `rule`). `GameRules.to_nbt` writes `minecraft:<name>` for those rules only (`:224`) and `from_nbt_with_features` reads only those (`:250`). The `/gamerule` tree is built from the same table (`NT/crates/horizon-command/src/vanilla.rs:298-315`). `NT/docs/mods.md` lists no game rule among the data of a content mod (grep for `game_rule` and `gamerule`: no hit).

### R3. The helpers of the level. Refused for a foreign name.

`native_level_game_rule` returns None for a rule outside the table (`G/native_level_world.rs:35`: `gamerules::rule(rule)?`). The hook `native_world_game_rule_change` (API 247) is told about the game's own rules after an operator uses the game rules screen. A BMH rule never reaches it.

### R4. The client hook `native_command` (API 35). Not used.

It runs a typed command before the server sees it (`G/native_hooks.rs:2902`), but only in the window's `Game`. A LAN guest's window would not help the host, and a dedicated server has no window. `with_native_commands` also keeps vanilla's command when a name is already in the tree (`G/chat.rs:41-52`), so it gives no completion for `gamerule`.

### R5. The server hook `native_player_command` (API 95). Works. This is what shipped.

`guest_command` runs the hook before it looks at the line (`G/lan_actions.rs:3534-3552`, "The mods hear every command first, and may stop it"). The owner of a single player world is one of the integrated server's players, so the owner's chat reaches the same code (check z1 below). The hook gets the player's UUID and name but not the permission level, and no helper that takes a UUID works inside it (`G/native_hooks.rs:4562-4565`).

Design (`bmh_settings.rs`, `on_player_command`, `on_server_tick`):

1. `player_command` sees `gamerule bmh:<name> [value]`. It sets `canceled` and queues the line `bmh rule <name> [value]` for that player. A vanilla rule (`gamerule keep_inventory true`) is left alone.
2. At the end of the same server tick, `server_tick` runs each queued line with `native_run_command_as(player, line)` (`G/native_server.rs:848-885`). That runs it as the player, with their level, and sends the answer to them as the answer of their own command.
3. `/bmh rule` already checks the level of `/gamerule` (2) and answers vanilla's texts `commands.gamerule.query` and `commands.gamerule.set` with the id `bmh:<name>`.

The cost is one tick of delay. The pure part (`gamerule_line_to_bmh`, `rule_name_matches`) has unit tests in `logic/src/settings.rs`.

### R6. Completion. Partly.

The chat box of the owner or a guest completes from the tree the server sent. The client asks the server only for nodes of the kind `ask_server` (`G/remote.rs:2404-2425`), and only mod commands have such nodes (`G/native_server.rs:56-62`). So `/gamerule bmh:` cannot complete. What changed: `/bmh rule <name:word?> <value:boolean>` now uses the asked-server kind, and the hook `tab_complete_v2` (`G/native_hooks.rs:7211`, pending: its doc says `API @API:tab_complete_v2@`) fills the rule names. `true` and `false` complete by the boolean parser. Proved in the game by check z9 and the screenshots below.

### R7. The saved `game_rules.dat`. Not usable.

Horizon writes the keys of its own table only. The value stays in `<world>/bmh/rules.txt` (settings log, "How the area works"). Check z8 shows that the value survives a quit to the title and a new open.

## S1. Evidence

`sh horizon/tools/test-slice.sh --no-build --only z,x` after `check.sh build`, 2026-11-19:

| Check | What it shows |
|---|---|
| z0 | Owner of a world with Allow Commands off (level 0): `/gamerule bmh:display_text_on_respawn false` is taken (log line "taken from a player") and refused with the unknown-command text. `rules.txt` is unchanged. |
| z1 | Owner of a world with Allow Commands on: the same line in the chat sets the rule. `rules.txt` says false. Answer: "Game rule bmh:display_text_on_respawn is now set to false". |
| z2 | `/gamerule bmh:display_text_on_respawn` answers "is currently set to false". |
| z3 | `... true` puts it back. |
| z4 | `/gamerule keep_inventory true` still runs in vanilla: "Game rule keep_inventory is now set to true". |
| z5 | `/gamerule bmh:nonsense true` answers "Unknown BMH rule: nonsense". |
| z6 | No "ran nothing for the line" warning in the log. |
| z8 | A rule set to false stays false after Save and Quit to Title and a new open of the world. |
| z9 | `/bmh rule rand` in the chat box shows a suggestion row, and Tab applies it. Screenshots `rules-createworld-shots/z9-completion.png` and `z9-completion-applied.png`. |
| x3 | A LAN guest at level 0 types `/gamerule bmh:random_death_spawn_point false`: the mod takes the line (log) and `/bmh rule` refuses it with the unknown-command text. `rules.txt` is unchanged. |

Not run: a dedicated server (same hook, no runner in `horizon/tools`), and a LAN guest at level 2 or higher.

## S1. What stays different

- No completion and no colour inside `/gamerule` (R6). Use `/bmh rule <Tab>`.
- No Game Rules screen row (the screen lists `RULES`, `G/game_rules_screen.rs:309`).
- Command blocks, functions and the server console do not hear `player_command`. They use `bmh rule ...`, which the console reaches through `perform_console_command`.
- The answer reaches the player one tick later.
- `execute if` and `/gamerule` queries from datapacks cannot read a BMH rule.

### Proposal if D8 is lifted (not built)

An injection that adds mod rules to the table, with these exact targets in the Nightly:

1. `horizon_client_core::gamerules::available_rules` (`gamerules.rs:132`) and `rule` (`:138`): chain a mod's `&'static Rule` list. `rule` strips `minecraft:` only, so it needs a `bmh:` branch.
2. `GameRules::to_nbt` (`:224`) and `from_nbt_with_features` (`:250`): write and read the key as `<namespace>:<name>`, not `minecraft:<name>`.
3. `horizon_command::vanilla` (`vanilla.rs:298-315`): the literal list gets the namespaced names, so the tree completes them.
4. `game_rules_screen.rs:58-61`: the lang keys `gamerule.minecraft.<name>` need the namespace.

Cleaner: the upstream ask of GAPS.md section 6, "a hook to register mod game rules".

## S2. What was tried

### R8. The state of the Create World screen. Readable, not writable.

`NativeScreenView` has `id`, `menu`, `title`, `rect`, `slots`, `mouse`, `partial_tick` (`NT/crates/horizon-client/src/mod_api.rs:297`). No tab and no widget list. But the mixin module sits inside `game.rs`, so it reads `Game.title`: `Mode::Create { options, .. }` holds `tab`, `hardcore`, `world_type`, `commands`, `rules` (`NT/crates/horizon-client/src/title/create_world.rs:20-48`). That is how the button gets its place (settings log, fix round 1) and now its two new behaviours.

### R9. A real widget in the tab's grid. Not possible without D8.

The grid and its rows are built in `title/create_world.rs` (`grid` at `:307`, rows near `:401` and `:532`). No hook reaches them. Proposal if D8 is lifted: an injection that appends a row at the end of the Game tab list near `:401`, and one more click arm near `:532`.

### R10. The choice stored in the creation rules. Not possible.

`options.rules` is a `GameRules`, so it cannot hold a BMH rule (R2). The More tab's Game Rules list therefore cannot show or sync the choice.

### R11. What was added this round (`bmh_settings.rs`, `track_screen`, `recreate_choice`, `create_locked`)

- Grey-out. In Hardcore (`options.hardcore`) and in a debug world (`world_type == Debug`) the button is drawn with the disabled sprite and a grey label, takes no click, and keeps its value. This is what `CreateWorldGameTabMixin.java:49-51` does.
- Re-Create. A Create screen with `options.commands` set is a Re-Create (a new world has it unset). The old world is the row that the world list had selected at the last screen hook. The button starts ON when any rule of that world's `bmh/rules.txt` is on (`CreateWorldGameTabMixin.java:58-60`, `anyMatch`). A world with no rules file starts ON.

## S2. Evidence

| Check | What it shows |
|---|---|
| o1 | Game Mode: Hardcore. A click on the button leaves the choice at its value. Screenshot `rules-createworld-shots/o1-hardcore-greyed.png`: the button is greyed like Difficulty and Allow Commands. |
| o2 | Back out of Hardcore (the game cycles to Creative): the button takes clicks again. Screenshot `o2-survival-active.png`. |
| o3 | Re-Create of a world made with the button OFF shows OFF (`o3-recreate-of-off-world.png`, name field `bmhrc`). Re-Create of a world whose rules are on shows ON (`o4-recreate-of-on-world.png`). |
| k0 to k6 | The older button checks still pass after the change of `track_screen`. |

Not run: the debug world case (the create screen offers no debug preset; the code reads `WorldPreset::Debug`), the tooltip at a new place, a real mouse at GUI scale 2 and 4.

## S2. What stays different

- Not a child of the tab's grid (R9). The place is computed from the game's own arithmetic.
- Not synced with the More tab's list (R10).
- No click sound and no narrator text. `native_play_local_sound` plays nothing on a server and needs `&mut Game`, while the click hook gets `&Game`. Not tried in the game.
- Re-Create copies the state, not the six values: an old world with mixed values becomes all ON or all OFF.

## Files

`horizon/native/mixins/bmh/bmh_settings.rs`, `logic/src/settings.rs` (5 new unit tests, 359 in total pass), `horizon-mixin.toml` (injections `hook_player_command` and `hook_tab_complete_v2`), `hook_player_command.rs`, `hook_tab_complete_v2.rs`, `hook_server_tick.rs`, `docs/horizon/HOOKS.md` (two rows), `horizon/tools/test-slice.sh` (scenarios z and o, check x3).
