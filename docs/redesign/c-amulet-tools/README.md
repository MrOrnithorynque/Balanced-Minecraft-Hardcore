# Part C checks without a game client

What was used to check the Navigator's Amulet on a dedicated server of each loader (log: `../log-c-amulet.md`, step 10). Nothing
here ships in the mod.

- `boot_server.sh <worktree> <JAVA_HOME> <forge|fabric|neoforge> <mc version> <log dir> [during-script]`: takes a Gradle slot and
  the server lock of the redesign run (`/private/tmp/bmh-gradle.N`, `/private/tmp/bmh-runserver.lock`), writes a fresh flat world and
  `server.properties` (port 25613, RCON 25713, password `bmh`, offline mode), boots `:<loader>:runServer`, runs the during-script
  once the log says `Done`, stops the server and frees the locks. The port and the lock paths are the ones of this run: change
  them to reuse it elsewhere.
- `during.sh`: runs `amulet_scenario.js` against the live server.
- `amulet_scenario.js`: a mineflayer bot (a vanilla client: it sees nothing of the modded items) that sends "use item" and "use
  item on block" packets, reads the action bar and checks the item and the player with RCON `data get entity`. 28 checks: refusal
  by day, a reading at night, cooldown, charges 3 to 0 and the empty message, refill at the next night, rain and roof refusals,
  Home on a lodestone and on a bed, the azimuth from the four sides of Home, Home across death and relog, the Nether refusal.
  Setup: `npm install --ignore-scripts --prefix <scratch dir> mineflayer@4.39.0`, then
  `BOT_MODULES=<scratch dir>/node_modules node amulet_scenario.js <port> <mc version>` (needs `python3` for `rcon.py`).
- `rcon.py <port> <command...>`: a tiny RCON client.
- `needle_sim.py <navigator_amulet.json>`: replays vanilla's compass angle for every yaw and resolves the model's overrides, then
  checks the needle frame points at north on screen (worst error must stay under half a frame, 5.625 degrees).

What these cannot show: anything the client draws (needle, bar, tooltip, sounds, arm swing). Those are in `../checklist-c-amulet.md`.
