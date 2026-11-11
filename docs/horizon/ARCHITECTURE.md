# BMH on Horizon: architecture

This file is the contract of the Horizon port. Every agent that writes code for it reads it first.
The evidence for each choice is in `docs/horizon/audit/` (five audits, five verifications, audit 6) and in
`docs/horizon/GAPS.md`.

## What Horizon is, in five lines

- Horizon is Minecraft: Java Edition 26.3 written again in Rust. Java mods do not run in it.
- A content mod (`.hmod`) is data and assets only: items, recipes, tags, lang, models, advancements.
- A native mod (`.hnmod`) is Rust source. A tool (horizon-mixins) adds it to the game's source before the game
  compiles. The game calls the mod at hook points (`native_<hook>` functions of `Game`).
- One process holds two `Game`s in single player: the window's game (client) and the integrated server's game, each
  on its own thread. The owner of a single-player world is a guest of that server (audit 6, V0). So a server hook
  fires for the single-player owner, for a LAN guest and for a dedicated server's player alike.
- The game runs only with files prepared from the player's own Minecraft 26.3. Without them we can compile and run
  unit tests, and nothing else.

## Decisions

| # | Decision | Reason |
|---|---|---|
| D1 | Build on the Nightly of 2026-11-08, commit `fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca` (tag `nightly-source/2026-11-08-fdc87ae`, hook API 411). The mod pins that commit. | The distant respawn needs `native_player_respawn_position`, and other features need other hooks, that have no API number yet ("pending fragments"). The Nightly's source already holds every one the port needs (checked: the twelve fragment files are in the tag, and a probe bundle on the respawn hook compiles against it). A Nightly's commit is kept for good and is the game players run. Read signatures in the Nightly's tree, not in main: line numbers and a few names differ. |
| D2 | Mixin helpers on hook points, plus one crate of pure logic. No typed `[api]` crate. | `native.py` builds and packages only this shape, and the registry accepts only this shape today. |
| D3 | An injection names its hook by `target = "<path>::native_<hook>"`, never by `hook = "<name>"`. | A pending hook is in no catalog, so `hook =` fails with E_HOOK_UNKNOWN. A target path works for stamped and pending hooks. |
| D4 | The mod id is `bmh` for the content part and the native part. Item ids are the Java ids (`bmh:eternal_sword`, `bmh:dormant_totem`, `bmh:sleeping_totem` and so on). | Old worlds keep their items. A combined `.hmod` needs one id. |
| D5 | The six BMH game rules become world settings of the mod: `/bmh rule <name> [true|false]`, saved with the world. They are not in `/gamerule`. | Horizon's game rule table is a `const` (audit 1 row 4, verify-5 G4). This is a real difference from the Java mod. |
| D6 | The server and the client talk through mod messages (`net_send_to_player`, `net_from_server`, `net_send_to_server`, `net_from_client`), never through title packets. | Horizon has a channel. The Java trick existed because BMH had none. |
| D7 | The server decides. The client draws and asks. A client request (a point name, a respawn that waited) is checked on the server as the Java command was. | Same trust rule as the Java mod. |
| D8 | No injection into the game's own functions (tier X) in the first version, except where a feature has no other way and the list in GAPS.md says so. | Each such injection ties the mod to one commit for good. A hook point becomes stable once Horizon stamps it. |

## Folders

```
horizon/
  content/                       the content mod (id bmh): horizon-mod.toml, data/, assets/, NOTES.md
  native/
    Cargo.toml                   workspace of the logic crate (unit tests: cargo test)
    compatibility.json           game, hooks number, Horizon commit, horizon-mixins commit
    mixins/bmh/
      horizon-mixin.toml         the manifest: one [[module]] per module file, one [[injection]] per hook
      logic/                     [[crate]] id "logic", package `bmh_logic`: pure Rust, no game type, unit tests
        Cargo.toml
        src/lib.rs               `pub mod` lines only
        src/<area>.rs            one file per area (below)
      bmh.rs                     module `bmh`: ids, shared state root, small helpers every module uses
      bmh_<area>.rs              one module per area: the edge between the game and bmh_logic
      hook_<hook>.rs             one helper per hook point: it calls the modules, in a fixed order
  tools/                         check.sh (compile into the game), test scripts for horizon-ctl
docs/horizon/                    audit/, GAPS.md, ARCHITECTURE.md, HOOKS.md, LOG.md
```

The Java sources stay on this branch while the port is in work: they are the behaviour reference. They go away
when the port is complete.

## Areas, and who owns which file

One area = one owner. An agent edits only the files of its area. The hook helpers (`hook_*.rs`), the manifest,
`bmh.rs`, `logic/src/lib.rs` and `logic/Cargo.toml` belong to the integrator: an agent that needs a change there
makes it in its own worktree, keeps it as small as possible, and names it in its report.

| Area | Module file | Logic file | What it holds | Java reference |
|---|---|---|---|---|
| respawn | `bmh_respawn.rs` | `respawn.rs` | distance at death, landing search, position inside the respawn, exile record, 200 ticks immunity, arrival sounds, journey signals (server side), journey advancements | `handlers/RespawnPositionHandler`, `RespawnSearch`, `RespawnEffectHandler`, `JourneySignal`, `mixin/F_*` (server ones) |
| homecoming | `bmh_homecoming.rs` | `homecoming.rs` | the record on the player, the check once a second, title, bell, two advancements | `handlers/HomecomingHandler`, `mixin/C_ServerPlayerMixin`, `E_ServerPlayerMixin` |
| eternal | `bmh_eternal.rs` | `eternal.rs` | Eternal Items kept after death, one of each type, items on the ground, the Eternal Bundle, sleep bonus, hidden coordinates | `handlers/KeepItemAfterDeath`, `EternalItemHandler`, `SleepHandler`, `DisableDebugInfo`, `items/EternalBundleItem`, `mixin/G_ItemEntityMixin` |
| totem | `bmh_totem.rs` | `totem.rs` | the save from death, its effects, the Dormant Totem, one Totem of Eternity per inventory and per container, the Totem of Undying without Fire Resistance, the anvil (awakening, gold block repair) | `handlers/TotemOfEternityHandler`, `TotemAwakeningHandler`, `mixin/A_*`, `B_AnvilMenuMixin`, `I_*` |
| ritual | `bmh_ritual.rs` | `moon.rs` | the moon ritual on the server and its look on the client (pose, particles) | `handlers/MoonRitualHandler`, `items/DormantTotemOfEternityItem`, `client/Ritual*` |
| tools | `bmh_tools.rs` | `tools.rs` | wear rules, worn out state, 7 % death wear, Echo Strike, Axe Smash, Echo Sense (server and client glow), tooltips | `handlers/EternalWear`, `EchoStrikeHandler`, `AxeSmashHandler`, `EchoSenseHandler`, `items/Eternal*`, `mixin/E_*`, `L_*` |
| amulet | `bmh_amulet.rs` | `amulet.rs` | use and use-on, readings and charges, named points on the player, `/bmh point`, the naming and list screens | `items/NavigatorAmuletItem`, `handlers/Amulet*`, `client/amulet/*` |
| sky | `bmh_sky.rs` | `stars.rs` | the star reading drawn in the sky, the spyglass name | `client/sky/*`, `mixin/D_*`, `K_*` |
| journey | `bmh_journey.rs` | `journey.rs` | the cinematic on the client: timeline, drawing, sounds, the Respawn hold | `client/journey/*`, `mixin/F_GuiTitleMixin`, `F_LocalPlayerRespawnMixin`, `F_MinecraftOverlayMixin` |
| settings | `bmh_settings.rs` | `settings.rs` | config file, the six world rules and `/bmh rule`, the Options > Mods row and page, the Create World button, migration of old world data | `utilities/BMHGameRules`, `BMHConfigLimits`, `neoforge/NeoForgeConfig`, `client/config/ConfigScreen`, `mixin/CreateWorldGameTabMixin`, `N_*` |

`bmh_logic` also has `wire.rs` (integrator): the channel names and the byte layout of every mod message, with
round-trip tests. Both sides use it, so the two sides cannot drift.

## How a hook reaches an area

`hook_<hook>.rs` holds one function. It calls, in the order HOOKS.md gives, one function per area that uses the
hook: `crate::game::bmh_<area>::on_<hook>(...)`. The skeleton has every such function with an empty body, so the
whole bundle compiles from the first day and each area fills its own bodies.

```rust
// hook_player_tick.rs
fn bmh_hook_player_tick(game: &mut crate::game::Game<'_>, player: u128, name: &str) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_respawn::on_player_tick(game, player, name);
    crate::game::bmh_homecoming::on_player_tick(game, player, name);
    crate::game::bmh_totem::on_player_tick(game, player, name);
}
```

Order matters where the Java mod used event priorities (death: the distance is stored before the Eternal Items
leave the inventory, the journey signal goes last and only if nobody cancelled). HOOKS.md records the order and its
reason next to the Java priority it replaces.

## State

- A `static` in a module is one value for the whole process. In single player the server thread and the window
  thread both see it. So server state and client state live in different statics, named `SERVER_*` and
  `CLIENT_*`, each behind a `Mutex` (or an atomic). Never a `thread_local!`: it would give two silent copies.
- The server never reads `CLIENT_*` and the client never reads `SERVER_*`. What the client must know comes in a mod
  message. This keeps single player, LAN and a dedicated server on the same path.
- A hook can fire inside another hook. Take a lock, copy what you need, release it, then call the game.
  Never call a `game.native_*` helper while a lock is held.
- State that must survive a restart goes on the player or the level (attachments, saved with the world), not in a
  static. Per-player maps in statics are keyed by the player's uuid (`u128`) and cleaned in `player_logged_out`.

## Logic crate rules

- `bmh_logic` knows nothing of the game: no `Game`, no Horizon type. It takes plain values (numbers, `[f64; 3]`,
  `&str` ids, small structs of its own) and returns plain values or commands for the module to carry out.
- Where the logic must ask the world something (is this column land, what is the ground at x z), it takes a trait
  or a closure, so a unit test gives a fake world.
- Every formula, table, constant, state machine and text key of the Java mod lives here, with the Java `file:line`
  it comes from in a comment, and a unit test that pins the Java value.
- No dependency outside Rust's standard library, unless the crate and version are in Horizon's `Cargo.lock`.
- Randomness comes in as a parameter (a seed or an `FnMut() -> f64`), so tests are exact.

## Module and helper rules (the tool refuses or the build fails otherwise)

- A helper file holds exactly one function. Its parameters are the manifest's bindings, same names, same order.
  Bind only what the function uses: an unused parameter is a build error (`RUSTFLAGS="-D warnings"`).
- A module function that takes `Game` is `pub(super)`, not `pub` (`Game` is private to `game.rs`).
- Module and helper files have no inner attribute (`#![...]`), no `//!` comment, no `mod other;` and no
  `include!`. Use ordinary comments and inline modules.
- Warnings are errors: no unused import, variable, function or constant. A function nobody calls yet gets a real
  caller or does not exist.
- The logic crate is reached by its package name: `bmh_logic::respawn::distance(...)`.
- A panic in the mod stops the game. No `unwrap()` or `expect()` on anything the world, a file or a message
  supplies. `Option` and `Result` all the way; on a failure, do what vanilla would do.
- Check `game.native_on(bmh::ID)` first in every hook helper: a mod switched off must do nothing.
- Comments and messages use Simplified Technical English. Never an em-dash or an en-dash.

## Build and test

- `sh horizon/tools/check.sh` compiles the bundle into Horizon with Horizon's own tool (`native.py check`), under a
  lock so two runs never share the stage folder. It prints only errors. A first run takes minutes, a later one
  about 30 seconds.
- `cargo test` in `horizon/native` runs the logic crate's unit tests, with no game.
- `sh horizon/tools/content-check.sh` builds and checks the content mod with `cargo horizon-mod`.
- In-game tests drive a real game through `horizon-ctl` (launch a world, run commands, read state, events and
  screenshots). They need prepared game files. Until the owner has Minecraft 26.3 installed, they are written and
  not run, and every report says so.

## What is not proven by a compile

A hook that compiles can still fire at another moment than we think, or for another player, or not at all. Each
area keeps a list "to check in the game" in `docs/horizon/LOG.md`. Nothing on that list is called done.
