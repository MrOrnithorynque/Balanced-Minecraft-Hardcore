package github.mrornithorynque.bmh.client;

import net.minecraft.SharedConstants;
import net.minecraft.client.Minecraft;
import net.minecraft.client.Screenshot;
import net.minecraft.client.server.IntegratedServer;
import net.minecraft.commands.CommandSourceStack;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.item.ItemStack;

/**
 * THROWAWAY verification hook. This file ships with the verify-loaders skill (scripts/hook/); its copy in common/src is
 * never committed: branch.sh copies it into its verify worktree and removes it at the end.
 * Only with BMH_SMOKE=1 in a singleplayer runClient: gives every BMH item, takes a real star reading at midnight (client
 * use, server item logic, client sky renderer), holds the Eternal Sword, lets the Totem of Eternity save the player,
 * saves four screenshots and quits the game. Every step logs a
 * "[BMH-SMOKE]" line.
 */
public final class SmokeClient {

    private static final boolean ENABLED = "1".equals(System.getenv("BMH_SMOKE"));
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
            // a bundle with one diamond in it: NBT before 1.20.5, the bundle_contents component after
            String version = SharedConstants.getCurrentVersion().getName();
            String filledBundle = version.startsWith("1.21")
                    ? "give @a bmh:eternal_bundle[minecraft:bundle_contents=[{id:\"minecraft:diamond\",count:1}]]"
                    : "give @a bmh:eternal_bundle{Items:[{id:\"minecraft:diamond\",Count:1b}]}";
            run(server, "difficulty peaceful", "gamerule doDaylightCycle false", "gamerule doWeatherCycle false",
                    "gamerule doMobSpawning false", "weather clear", "time set 18000", "clear @a",
                    "execute as @a at @s run setworldspawn ~ ~ ~-2000",
                    "execute as @a at @s run setblock ~ 199 ~ minecraft:glass",
                    "execute as @a at @s run tp @s ~ 200 ~ 180 -30",
                    "give @a bmh:navigator_amulet", "give @a bmh:eternal_sword", "give @a bmh:eternal_axe",
                    "give @a bmh:eternal_pickaxe", filledBundle, "give @a bmh:totem_of_eternity",
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
                    + " dayTime=" + mc.level.getDayTime() + " raining=" + mc.level.isRaining());
        } else if (ticks == 140) {
            log("use amulet: " + mc.gameMode.useItem(mc.player, InteractionHand.MAIN_HAND));
        } else if (ticks == 150) {
            ItemStack held = mc.player.getMainHandItem();
            log("after use: cooldown=" + mc.player.getCooldowns().isOnCooldown(held.getItem()) + " data=" + data(held));
        } else if (ticks == 185) {
            shot(mc, "bmh-smoke-1-hud.png");
        } else if (ticks == 195) {
            mc.options.hideGui = true;
        } else if (ticks == 205) {
            shot(mc, "bmh-smoke-2-sky.png");
        } else if (ticks == 215) {
            mc.options.hideGui = false;
            // The Eternal Sword (hotbar slot 1) in the hand: its 20x20 model there, its 16x16 one in the hotbar.
            mc.player.getInventory().selected = 1;
            run(server, "gamemode survival @a", "effect clear @a");
        } else if (ticks == 235) {
            shot(mc, "bmh-smoke-3-sword.png");
        } else if (ticks == 240) {
            // Nothing in the hands saves the player, so the Totem of Eternity (slot 5) does, with its own animation.
            run(server, "damage @a 1000 minecraft:generic");
        } else if (ticks == 252) {
            shot(mc, "bmh-smoke-4-totem.png");
        } else if (ticks == 270) {
            String slot5 = String.valueOf(BuiltInRegistries.ITEM.getKey(mc.player.getInventory().getItem(5).getItem()));
            log("after the totem: health=" + mc.player.getHealth() + " slot5=" + slot5);
        } else if (ticks == 280) {
            log("stopping");
            mc.stop();
        }
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
                message -> log("screenshot " + name + ": " + message.getString())));
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
