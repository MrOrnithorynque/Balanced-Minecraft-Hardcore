package github.mrornithorynque.bmh.selftest;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.UUID;

import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

import net.minecraft.core.BlockPos;
import net.minecraft.network.chat.Component;
import net.minecraft.network.chat.contents.TranslatableContents;
import net.minecraft.network.protocol.Packet;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.level.GameRules;

/**
 * THROWAWAY verification hook (never committed, see selftest.sh). Runs the feature checks of Tests.ALL on a dedicated
 * server, one after the other, then halts the server. Each check prints one line:
 * [BMH-SELFTEST] PASS|FAIL <test> / <check> | <detail>. The last line is [BMH-SELFTEST] END <passed>/<total>.
 * A test gets its own mock players (MockPlayers, a copy per Minecraft version) and its own place in the world.
 */
public final class SelfTest {

    private static final Logger LOG = LogUtils.getLogger();
    private static final boolean ON = "1".equals(System.getenv("BMH_SELFTEST"));
    private static final int START_TICK = 60;
    private static final int TEST_TIMEOUT = 1200;
    private static final int WARMUP_TICKS = 20;

    /** A feature test: start() sets the scene, then the test schedules its checks with later(). */
    public interface Test {
        String name();

        void start(Ctx ctx);
    }

    /** What a test sees: the server, a place of its own, mock players, checks and delayed steps. */
    public static final class Ctx {
        public final MinecraftServer server;
        public final ServerLevel level;
        public final BlockPos origin;
        private final String test;
        private final List<UUID> players = new ArrayList<>();
        private final List<long[]> dueTicks = new ArrayList<>();
        private final List<Runnable> dueSteps = new ArrayList<>();
        private final Map<GameRules.Key<GameRules.BooleanValue>, Boolean> savedRules = new LinkedHashMap<>();

        Ctx(MinecraftServer server, String test, BlockPos origin) {
            this.server = server;
            this.level = server.overworld();
            this.test = test;
            this.origin = origin;
        }

        /** A survival mock player, standing at origin + (dx, 1, dz), ticked by SelfTest like a connected one. */
        public ServerPlayer player(String name, int dx, int dz) {
            ServerPlayer player = MockPlayers.place(server, level, name);
            player.teleportTo(origin.getX() + dx + 0.5, origin.getY() + 1, origin.getZ() + dz + 0.5);
            player.setGameMode(net.minecraft.world.level.GameType.SURVIVAL);
            players.add(player.getUUID());
            return player;
        }

        /** The current object of a mock player: a respawn replaces the ServerPlayer. Null once it left. */
        public ServerPlayer live(ServerPlayer player) {
            return server.getPlayerList().getPlayer(player.getUUID());
        }

        /** Sets a boolean game rule for this test. SelfTest puts the old value back when the test ends. */
        public void rule(GameRules.Key<GameRules.BooleanValue> key, boolean value) {
            GameRules.BooleanValue rule = server.getGameRules().getRule(key);
            savedRules.putIfAbsent(key, rule.get());
            rule.set(value, server);
        }

        /** ServerPlayer.spawnInvulnerableTime, private on every version: read by reflection (Mojang names in dev). */
        public int spawnInvulnerableTime(ServerPlayer player) {
            try {
                java.lang.reflect.Field field = ServerPlayer.class.getDeclaredField("spawnInvulnerableTime");
                field.setAccessible(true);
                return field.getInt(player);
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
        }

        /** The title packets this player got since the last call, as "key" or "key(n args)", oldest first. */
        public List<String> titles(ServerPlayer player) {
            List<String> titles = new ArrayList<>();
            for (Packet<?> packet : MockPlayers.sent(player)) {
                Component text = MockPlayers.titleText(packet);
                if (text == null) continue;
                if (text.getContents() instanceof TranslatableContents translatable) {
                    int args = translatable.getArgs().length;
                    titles.add(translatable.getKey() + (args == 0 ? "" : "(" + args + " args)"));
                } else {
                    titles.add(text.getString());
                }
            }
            return titles;
        }

        public void check(String check, boolean ok, String detail) {
            record(test, check, ok, detail);
        }

        /** Runs step after the given number of server ticks. */
        public void later(int ticks, Runnable step) {
            dueTicks.add(new long[] { server.getTickCount() + ticks });
            dueSteps.add(step);
        }

        boolean done() {
            return dueSteps.isEmpty();
        }

        void tick() {
            // A connected player's doTick comes from its connection, which the server does not tick for a mock one.
            for (UUID id : players) {
                ServerPlayer player = server.getPlayerList().getPlayer(id);
                if (player != null) player.doTick();
            }
            for (int i = 0; i < dueSteps.size(); i++) {
                if (dueTicks.get(i)[0] <= server.getTickCount()) {
                    Runnable step = dueSteps.remove(i);
                    dueTicks.remove(i);
                    i--;
                    step.run();
                }
            }
        }

        // The test place: 5 x 5 chunks around the origin, forced like /forceload, so their entities load and tick.
        // Without it only the origin chunk had its entities loaded: an item 16 blocks away was invisible to the scans.
        void forceChunks(boolean forced) {
            int cx = origin.getX() >> 4;
            int cz = origin.getZ() >> 4;
            for (int x = cx - 2; x <= cx + 2; x++) {
                for (int z = cz - 2; z <= cz + 2; z++) {
                    level.setChunkForced(x, z, forced);
                }
            }
        }

        void close() {
            forceChunks(false);
            savedRules.forEach((key, value) -> server.getGameRules().getRule(key).set(value, server));
            for (UUID id : players) {
                ServerPlayer player = server.getPlayerList().getPlayer(id);
                if (player != null) server.getPlayerList().remove(player);
            }
        }
    }

    private static int index = -1;
    private static Ctx current;
    private static long started;
    private static int passed;
    private static int total;

    private SelfTest() {
    }

    public static void tick(MinecraftServer server) {
        if (!ON || index >= Tests.ALL.size() || server.getTickCount() < START_TICK) return;
        try {
            if (current != null) {
                current.tick();
                if (current.done()) {
                    next(server);
                } else if (server.getTickCount() - started > TEST_TIMEOUT) {
                    record(current.test, "finishes in time", false, "still waiting after " + TEST_TIMEOUT + " ticks");
                    next(server);
                }
            } else {
                next(server);
            }
        } catch (Throwable t) {
            LOG.error("[BMH-SELFTEST] FAIL {} / no exception | {}", current == null ? "?" : current.test, t.toString(), t);
            total++;
            next(server);
        }
    }

    private static void next(MinecraftServer server) {
        if (current != null) current.close();
        index++;
        if (index >= Tests.ALL.size()) {
            LOG.info("[BMH-SELFTEST] END {}/{}", passed, total);
            server.halt(false);
            return;
        }
        Test test = Tests.ALL.get(index);
        BlockPos spawn = server.overworld().getSharedSpawnPos();
        current = new Ctx(server, test.name(), spawn.offset(64 * (index + 1), 0, 0));
        started = server.getTickCount();
        current.forceChunks(true);
        // A forced chunk loads its entities a few ticks later: an item added before that is not seen by the scans.
        Ctx ctx = current;
        ctx.later(WARMUP_TICKS, () -> {
            try {
                test.start(ctx);
            } catch (Throwable t) {
                LOG.error("[BMH-SELFTEST] FAIL {} / no exception | {}", test.name(), t.toString(), t);
                total++;
                ctx.dueSteps.clear();
            }
        });
    }

    static void record(String test, String check, boolean ok, String detail) {
        total++;
        if (ok) passed++;
        LOG.info("[BMH-SELFTEST] {} {} / {} | {}", ok ? "PASS" : "FAIL", test, check, detail);
    }
}
