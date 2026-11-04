package github.mrornithorynque.bmh.client;

import net.minecraft.client.Minecraft;
import net.minecraft.client.Screenshot;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.components.events.GuiEventListener;
import net.minecraft.client.gui.screens.DeathScreen;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.client.server.IntegratedServer;
import net.minecraft.commands.CommandSourceStack;
import net.minecraft.network.chat.Component;

/**
 * THROWAWAY verification hook for the journey respawn, never committed. Only with BMH_JOURNEY_SMOKE=1 in a
 * singleplayer runClient: sets BMH's rules, kills the player, presses the death screen's Respawn button, logs every
 * position packet and saves a frame every 3 ticks until the cinematic ends, then quits. Lines start with [JOURNEY-SMOKE].
 */
public final class JourneySmoke {

    public static final boolean ENABLED = "1".equals(System.getenv("BMH_JOURNEY_SMOKE"));
    private static final String TIME = System.getenv().getOrDefault("BMH_JOURNEY_TIME", "1000");
    // With doImmediateRespawn there is no death screen: the frames start at the kill.
    private static final boolean IMMEDIATE = "1".equals(System.getenv("BMH_JOURNEY_IMMEDIATE"));
    // A resource reload during the fade puts a LoadingOverlay in the place of the journey.
    private static final boolean RELOAD = "1".equals(System.getenv("BMH_JOURNEY_RELOAD"));
    private static int ticks = -1;
    private static int pressTick = -1;
    private static int endTick = -1;
    private static int frame = 0;
    private static LocalPlayer deadPlayer;
    private static long pressMillis;

    private JourneySmoke() {
    }

    public static void tick(Minecraft mc) {
        if (!ENABLED || mc.level == null || mc.player == null || mc.getSingleplayerServer() == null) {
            return;
        }
        IntegratedServer server = mc.getSingleplayerServer();
        ticks++;
        if (ticks == 0) {
            log("world joined at " + pos(mc.player) + " worldSpawn=" + mc.level.getSharedSpawnPos().toShortString());
            // Every BMH sound the client plays, with its time after the Respawn press.
            mc.getSoundManager().addListener((sound, events, range) -> {
                if ("bmh".equals(sound.getLocation().getNamespace()) || sound.getLocation().getPath().contains("amethyst")) {
                    log("sound " + sound.getLocation() + " t+" + (System.currentTimeMillis() - pressMillis) + "ms");
                }
            });
        } else if (ticks == 40) {
            run(server, "difficulty peaceful", "gamerule doDaylightCycle false", "gamerule doWeatherCycle false",
                    "weather clear", "time set " + TIME, "gamerule doImmediateRespawn " + IMMEDIATE,
                    "gamerule randomDeathSpawnPoint true", "gamerule displayTextOnRespawn true", "gamemode survival @a");
        } else if (ticks == 100) {
            deadPlayer = mc.player;
            log("killing the player at " + pos(mc.player) + " immediate=" + IMMEDIATE);
            run(server, "kill @a");
            if (IMMEDIATE) {
                pressTick = ticks;
                pressMillis = System.currentTimeMillis();
            }
        } else if (ticks == 140 && !IMMEDIATE) {
            log("death screen open=" + (mc.screen instanceof DeathScreen) + " screen=" + name(mc.screen));
            shot(mc, "journey-0-death.png");
        } else if (ticks == 150 && !IMMEDIATE) {
            pressRespawn(mc);
            pressTick = ticks;
            pressMillis = System.currentTimeMillis();
        }

        if (RELOAD && pressTick >= 0 && ticks == pressTick + 4) {
            log("reloading resources, overlay=" + name(mc.getOverlay()));
            mc.reloadResourcePacks();
        }

        if (pressTick >= 0 && endTick < 0 && ticks > pressTick) {
            int since = ticks - pressTick;
            if (since % 3 == 0) {
                shot(mc, String.format("journey-1-%03d.png", frame++));
            }
            if (since % 5 == 0 || since < 12) {
                log("t+" + (System.currentTimeMillis() - pressMillis) + "ms overlay=" + name(mc.getOverlay()) + " screen="
                        + name(mc.screen) + " respawned=" + (mc.player != deadPlayer) + " pos=" + pos(mc.player)
                        + " tickCount=" + mc.player.tickCount + " paused=" + mc.isPaused() + " waiting=" + waiting());
            }
            if (since > 10 && mc.getOverlay() == null && mc.player != deadPlayer) {
                endTick = ticks;
                log("overlay gone at t+" + (System.currentTimeMillis() - pressMillis) + "ms pos=" + pos(mc.player));
            }
            if (since > 900) {
                endTick = ticks;
                log("TIMEOUT waiting for the overlay to end");
            }
        }

        if (endTick >= 0) {
            if (ticks == endTick + 1 || ticks == endTick + 20) {
                shot(mc, "journey-2-after-" + (ticks - endTick) + ".png");
            }
            if (ticks == endTick + 39) {
                log("final respawned=" + (mc.player != deadPlayer) + " alive=" + mc.player.isAlive() + " screen="
                        + name(mc.screen) + " overlay=" + name(mc.getOverlay()) + " pos=" + pos(mc.player));
            }
            if (ticks == endTick + 40) {
                log("stopping");
                mc.stop();
            }
        }
        if (ticks > 3000) {
            log("stopping (global timeout)");
            mc.stop();
        }
    }

    /** Called by the packet mixin after each position packet and respawn packet, on the render thread. */
    public static void packet(Minecraft mc, String what) {
        if (ENABLED && pressTick >= 0 && mc.player != null) {
            log(what + " t+" + (System.currentTimeMillis() - pressMillis) + "ms pos=" + pos(mc.player));
        }
    }

    // JourneyCinematic.waitingPlayer, read by name: dev runs use the source names.
    private static String waiting() {
        try {
            java.lang.reflect.Field f = Class.forName("github.mrornithorynque.bmh.client.journey.JourneyCinematic").getDeclaredField("waitingPlayer");
            f.setAccessible(true);
            Object ref = f.get(null);
            return String.valueOf(ref instanceof java.lang.ref.Reference<?> r ? r.get() != null : ref);
        } catch (ReflectiveOperationException e) {
            return "?" + e.getClass().getSimpleName();
        }
    }

    private static void pressRespawn(Minecraft mc) {
        if (!(mc.screen instanceof DeathScreen deathScreen)) {
            log("no death screen to press (screen=" + name(mc.screen) + ")");
            return;
        }
        for (GuiEventListener child : deathScreen.children()) {
            if (child instanceof Button button && button.getMessage().equals(Component.translatable("deathScreen.respawn"))) {
                log("pressing Respawn (active=" + button.active + ")");
                button.onPress();
                log("after press: overlay=" + name(mc.getOverlay()) + " button active=" + button.active);
                return;
            }
        }
        log("no Respawn button found");
    }

    private static void run(IntegratedServer server, String... commands) {
        server.execute(() -> {
            CommandSourceStack source = server.createCommandSourceStack().withSuppressedOutput();
            for (String command : commands) {
                server.getCommands().performPrefixedCommand(source, command);
            }
        });
    }

    private static void shot(Minecraft mc, String name) {
        mc.tell(() -> Screenshot.grab(mc.gameDirectory, name, mc.getMainRenderTarget(),
                message -> log("screenshot " + name)));
    }

    private static String pos(LocalPlayer player) {
        return Math.round(player.getX()) + "," + Math.round(player.getY()) + "," + Math.round(player.getZ());
    }

    private static String name(Object o) {
        return o == null ? "-" : o.getClass().getSimpleName();
    }

    private static void log(String message) {
        System.out.println("[JOURNEY-SMOKE] " + message);
    }
}
