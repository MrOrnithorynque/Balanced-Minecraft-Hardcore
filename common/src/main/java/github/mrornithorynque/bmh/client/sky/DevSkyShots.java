package github.mrornithorynque.bmh.client.sky;

import net.minecraft.client.CloudStatus;
import net.minecraft.client.Minecraft;
import net.minecraft.client.Screenshot;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.client.server.IntegratedServer;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.level.GameType;

/**
 * DEVELOPMENT TRIGGER, removed before part D is reported done (brief, part D, item 4). Only with BMH_DEV_SKYSHOTS=1
 * in the environment of a singleplayer runClient: fakes star readings toward 0, 90, 180 and 270 at midnight and at
 * dayTime 22000, faces the camera that way, saves a screenshot of each in run/screenshots, then quits the game.
 */
final class DevSkyShots {

    static final boolean ENABLED = "1".equals(System.getenv("BMH_DEV_SKYSHOTS"));

    private static final int[][] STEPS = {
            { 0, 18000 }, { 90, 18000 }, { 180, 18000 }, { 270, 18000 },
            { 0, 22000 }, { 90, 22000 }, { 180, 22000 }, { 270, 22000 },
    };
    private static final double SETTLE_TICKS = 200;
    private static final double TIME_TICKS = 60;
    private static final double SHOT_TICKS = 60;

    private static double started = -1;
    private static int step = -1;
    private static double stepStart;
    private static boolean fired;

    private DevSkyShots() {
    }

    static void frame(ClientLevel level, LocalPlayer player, double now, float partialTick) {
        Minecraft minecraft = Minecraft.getInstance();
        IntegratedServer server = minecraft.getSingleplayerServer();
        if (server == null || step >= STEPS.length) {
            return;
        }
        if (started < 0) {
            started = now;
            minecraft.options.hideGui = true;
            minecraft.options.cloudStatus().set(CloudStatus.OFF);
            server.execute(() -> {
                for (ServerPlayer p : server.getPlayerList().getPlayers()) {
                    p.setGameMode(GameType.SPECTATOR);
                    p.teleportTo(p.getX(), 140.0, p.getZ());
                }
            });
            return;
        }
        if (now - started < SETTLE_TICKS) {
            return;
        }
        if (step < 0 || now - stepStart > TIME_TICKS + SHOT_TICKS + 20) {
            step++;
            stepStart = now;
            fired = false;
            if (step >= STEPS.length) {
                minecraft.tell(minecraft::stop);
                return;
            }
            long dayTime = STEPS[step][1];
            server.execute(() -> server.overworld().setDayTime(dayTime));
        }
        int azimuth = STEPS[step][0];
        float yaw = azimuth + 180.0F;
        player.setYRot(yaw);
        player.yRotO = yaw;
        player.setXRot(-32.0F);
        player.xRotO = -32.0F;
        if (!fired && now - stepStart >= TIME_TICKS) {
            fired = true;
            System.out.println("[BalancedMcHardcoreMain] dev sky shot " + azimuth + " at dayTime " + level.getDayTime());
            StarReadingRenderer.show(level, azimuth, now, partialTick);
        }
        if (fired && now - stepStart >= TIME_TICKS + SHOT_TICKS) {
            String name = String.format("bmh-sky-az%03d-%d.png", azimuth, STEPS[step][1]);
            stepStart = -1e9; // next frame moves on
            minecraft.tell(() -> Screenshot.grab(minecraft.gameDirectory, name, minecraft.getMainRenderTarget(),
                    message -> System.out.println("[BalancedMcHardcoreMain] " + message.getString())));
        }
    }
}
