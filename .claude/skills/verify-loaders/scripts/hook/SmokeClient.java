package github.mrornithorynque.bmh.client;

import net.minecraft.client.CameraType;
import net.minecraft.client.Minecraft;
import net.minecraft.client.server.IntegratedServer;
import net.minecraft.commands.CommandSourceStack;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.phys.Vec3;

/**
 * THROWAWAY verification hook. This file ships with the verify-loaders skill (scripts/hook/); its copy in common/src is
 * never committed: branch.sh copies it into its verify worktree and removes it at the end.
 * Only with BMH_SMOKE=1 in a singleplayer runClient: gives every BMH item, takes a real star reading at midnight (client
 * use, server item logic, client sky renderer), holds the Eternal Sword, lets the Totem of Eternity save the player,
 * wakes the Dormant Totem with the moon ritual, fails a second ritual, saves nine screenshots and quits the game. Every step logs a "[BMH-SMOKE]" line.
 * What differs between Minecraft versions is in ClientCompat, which branch.sh copies from hook/compat/ (one file per
 * version) next to this one.
 */
public final class SmokeClient {

    private static final boolean ENABLED = "1".equals(System.getenv("BMH_SMOKE"));
    // MoonRitualHandler.RITUAL_TICKS. The hook does not import the class: branch.sh also runs it on tags from before it.
    private static final int RITUAL_TICKS = 300;
    private static final int RITUAL_START = 300;
    private static final int RITUAL_END = RITUAL_START + RITUAL_TICKS + 20;
    private static final int FAIL_START = RITUAL_END + 30;
    private static int ticks = -1;

    private SmokeClient() {
    }

    public static void tick(Minecraft mc) {
        if (!ENABLED || mc.level == null || mc.player == null || mc.getSingleplayerServer() == null) {
            return;
        }
        IntegratedServer server = mc.getSingleplayerServer();
        ticks++;
        if (ticks == 0) {
            log("world joined");
        } else if (ticks == 60) {
            String version = ClientCompat.versionName();
            run(server, "difficulty peaceful", ClientCompat.gamerule("doDaylightCycle", false),
                    ClientCompat.gamerule("doWeatherCycle", false), ClientCompat.gamerule("doMobSpawning", false),
                    "weather clear", "time set 18000", "clear @a",
                    "execute as @a at @s run setworldspawn ~ ~ ~-2000",
                    "execute as @a at @s run setblock ~ 199 ~ minecraft:glass",
                    "execute as @a at @s run tp @s ~ 200 ~ 180 -30",
                    "give @a bmh:navigator_amulet", "give @a bmh:eternal_sword", "give @a bmh:eternal_axe",
                    "give @a bmh:eternal_pickaxe", ClientCompat.filledBundleCommand(), "give @a bmh:totem_of_eternity",
                    "give @a bmh:dormant_totem", "give @a bmh:echo_heart", "give @a minecraft:totem_of_undying");
            log("setup sent (" + version + ")");
        } else if (ticks == 120) {
            StringBuilder bar = new StringBuilder();
            for (int i = 0; i < 9; i++) {
                ItemStack stack = mc.player.getInventory().getItem(i);
                bar.append(i).append('=').append(stack.isEmpty() ? "-" : BuiltInRegistries.ITEM.getKey(stack.getItem())).append(' ');
            }
            log("hotbar " + bar.toString().trim());
            log("position y=" + Math.round(mc.player.getY()) + " yaw=" + Math.round(mc.player.getYRot())
                    + " dayTime=" + ClientCompat.dayTime(mc) + " raining=" + mc.level.isRaining());
        } else if (ticks == 140) {
            log("use amulet: " + mc.gameMode.useItem(mc.player, InteractionHand.MAIN_HAND));
        } else if (ticks == 150) {
            ItemStack held = mc.player.getMainHandItem();
            log("after use: cooldown=" + ClientCompat.onCooldown(mc, held) + " data=" + data(held));
        } else if (ticks == 185) {
            shot(mc, "bmh-smoke-1-hud.png");
        } else if (ticks == 195) {
            mc.options.hideGui = true;
        } else if (ticks == 205) {
            shot(mc, "bmh-smoke-2-sky.png");
        } else if (ticks == 215) {
            mc.options.hideGui = false;
            // The Eternal Sword (hotbar slot 1) in the hand: its 20x20 model there, its 16x16 one in the hotbar.
            ClientCompat.selectSlot(mc, 1);
            run(server, "gamemode survival @a", "effect clear @a");
        } else if (ticks == 235) {
            shot(mc, "bmh-smoke-3-sword.png");
            ItemStack sword = mc.player.getInventory().getItem(1);
            log(ClientCompat.swordModelLine(mc, sword));
        } else if (ticks == 240) {
            // Nothing in the hands saves the player, so the Totem of Eternity (slot 5) does, with its own animation.
            // The command's feedback goes to the log.
            server.execute(() -> {
                ServerPlayer target = server.getPlayerList().getPlayers().get(0);
                log("before the damage: server health=" + target.getHealth() + " mode=" + target.gameMode.getGameModeForPlayer()
                        + " invulnerable=" + target.getAbilities().invulnerable + " slot5="
                        + BuiltInRegistries.ITEM.getKey(target.getInventory().getItem(5).getItem()));
                server.getCommands().performPrefixedCommand(server.createCommandSourceStack(), "damage @a[limit=1] 1000 minecraft:generic");
            });
        } else if (ticks == 252) {
            shot(mc, "bmh-smoke-4-totem.png");
        } else if (ticks == 270) {
            String slot5 = String.valueOf(BuiltInRegistries.ITEM.getKey(mc.player.getInventory().getItem(5).getItem()));
            log("after the totem: health=" + mc.player.getHealth() + " slot5=" + slot5);
        } else if (ticks == 280) {
            // The moon ritual with the Dormant Totem the save left in slot 5. At 15000 the moon is low in the east, so a
            // wrong sign in the moon direction puts the crosshair on empty sky in the screenshots.
            ClientCompat.selectSlot(mc, 5);
            run(server, "time set 15000", "xp set @a 0 levels", "xp add @a 40 levels", "effect clear @a");
        } else if (ticks >= RITUAL_START && ticks < RITUAL_END) {
            aimAtMoon(mc);
            mc.options.keyUse.setDown(true);
            if (ticks == RITUAL_START) {
                Vec3 moon = moonDirection(mc);
                log("ritual start: dayTime=" + ClientCompat.dayTime(mc) + " moon=" + String.format("%.2f %.2f %.2f", moon.x, moon.y, moon.z)
                        + " yaw=" + Math.round(mc.player.getYRot()) + " pitch=" + Math.round(mc.player.getXRot()) + " levels=" + mc.player.experienceLevel);
            } else if (ticks == RITUAL_START + 150) {
                log("ritual mid: using=" + mc.player.isUsingItem() + " levels=" + mc.player.experienceLevel);
                shot(mc, "bmh-smoke-5-ritual.png");
            } else if (ticks == RITUAL_START + 156) {
                // A queued screenshot sees a camera change made in the same tick: each change gets its own tick.
                mc.options.setCameraType(CameraType.THIRD_PERSON_BACK);
            } else if (ticks == RITUAL_START + 164) {
                shot(mc, "bmh-smoke-6-ritual-back.png");
            } else if (ticks == RITUAL_START + 168) {
                mc.options.setCameraType(CameraType.THIRD_PERSON_FRONT);
            } else if (ticks == RITUAL_START + 176) {
                shot(mc, "bmh-smoke-7-ritual-front.png");
            } else if (ticks == RITUAL_START + 180) {
                mc.options.setCameraType(CameraType.FIRST_PERSON);
            } else if (ticks == RITUAL_START + RITUAL_TICKS + 6) {
                shot(mc, "bmh-smoke-8-woken.png");
            }
        } else if (ticks == RITUAL_END) {
            mc.options.keyUse.setDown(false);
        } else if (ticks == RITUAL_END + 10) {
            log("after the ritual: slot5=" + BuiltInRegistries.ITEM.getKey(mc.player.getInventory().getItem(5).getItem())
                    + " levels=" + mc.player.experienceLevel);
            // A second ritual that fails: the player looks away after 40 ticks.
            run(server, "item replace entity @a hotbar.5 with bmh:sleeping_totem", "xp set @a 0 levels", "xp add @a 40 levels");
        } else if (ticks >= FAIL_START && ticks < FAIL_START + 46) {
            if (ticks < FAIL_START + 40) {
                aimAtMoon(mc);
            } else {
                mc.player.setXRot(0.0F);
                mc.player.xRotO = 0.0F;
            }
            mc.options.keyUse.setDown(true);
            if (ticks == FAIL_START + 44) {
                shot(mc, "bmh-smoke-9-failed.png");
            }
        } else if (ticks == FAIL_START + 46) {
            mc.options.keyUse.setDown(false);
        } else if (ticks == FAIL_START + 56) {
            log("after the failure: slot5=" + BuiltInRegistries.ITEM.getKey(mc.player.getInventory().getItem(5).getItem())
                    + " levels=" + mc.player.experienceLevel);
        } else if (ticks == FAIL_START + 66) {
            log("stopping");
            mc.stop();
        }
    }

    // The moon direction (ClientCompat: the sky math differs per version). The screenshots, not this code, prove it:
    // the moon must be under the crosshair, and the server log says how far the look was from its own moon direction.
    private static Vec3 moonDirection(Minecraft mc) {
        return ClientCompat.moonDirection(mc);
    }

    // The yaw and pitch that look along the moon direction.
    private static void aimAtMoon(Minecraft mc) {
        Vec3 moon = moonDirection(mc);
        float pitch = (float) -Math.toDegrees(Math.asin(moon.y));
        float yaw = (float) Math.toDegrees(Math.atan2(-moon.x, moon.z));
        mc.player.setYRot(yaw);
        mc.player.yRotO = yaw;
        mc.player.setXRot(pitch);
        mc.player.xRotO = pitch;
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
        ClientCompat.screenshot(mc, name, message -> log("screenshot " + name + ": " + message));
    }

    /** The stack's data, 1.20.x (NBT tag) or 1.21.1 (components), read by name: dev runs use the official names. */
    private static String data(ItemStack stack) {
        for (String method : new String[] { "getTag", "getComponents" }) {
            try {
                return String.valueOf(stack.getClass().getMethod(method).invoke(stack));
            } catch (ReflectiveOperationException ignored) {
                // the other version's method
            }
        }
        return "?";
    }

    private static void log(String message) {
        System.out.println("[BMH-SMOKE] " + message);
    }
}
