package github.mrornithorynque.bmh.client;

import java.util.function.Consumer;

import net.minecraft.SharedConstants;
import net.minecraft.client.Minecraft;
import net.minecraft.client.Screenshot;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.resources.model.BakedModel;
import net.minecraft.client.resources.sounds.SoundInstance;
import net.minecraft.world.item.ItemDisplayContext;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.phys.Vec3;

/**
 * THROWAWAY verification hook (never committed, see branch.sh and death_client.sh): the calls of SmokeClient and
 * JourneySmoke that differ between Minecraft versions. This copy serves 1.20 to 1.21.1. The 26.1 copy has the same
 * methods with the same signatures. Both hooks call these and nothing else version specific.
 */
final class ClientCompat {

    private ClientCompat() {
    }

    /** The name of the running Minecraft version, e.g. 1.21.1. */
    static String versionName() {
        return SharedConstants.getCurrentVersion().getName();
    }

    /** A give command for an Eternal Bundle with one diamond in it: NBT before 1.20.5, the bundle_contents component after. */
    static String filledBundleCommand() {
        return versionName().startsWith("1.21")
                ? "give @a bmh:eternal_bundle[minecraft:bundle_contents=[{id:\"minecraft:diamond\",count:1}]]"
                : "give @a bmh:eternal_bundle{Items:[{id:\"minecraft:diamond\",Count:1b}]}";
    }

    /** A gamerule command. The name is the 1.21.1 one; 26.1 renamed the rules and BMH's moved to its own namespace. */
    static String gamerule(String name, Object value) {
        return "gamerule " + name + " " + value;
    }

    /** The time of day of the client level. */
    static long dayTime(Minecraft mc) {
        return mc.level.getDayTime();
    }

    /**
     * The moon direction, the same math as MoonRitualHandler.moonDirection up to 1.21.1: the sky turns by timeOfDay * 360
     * degrees and the moon sits at y = -100.
     */
    static Vec3 moonDirection(Minecraft mc) {
        double angle = mc.level.getTimeOfDay(1.0F) * Math.PI * 2.0;
        return new Vec3(Math.sin(angle), -Math.cos(angle), 0.0);
    }

    /** The world spawn point of the client level, as "x, y, z". */
    static String worldSpawn(Minecraft mc) {
        return mc.level.getSharedSpawnPos().toShortString();
    }

    /** True while the item of this stack is on cooldown for the local player. */
    static boolean onCooldown(Minecraft mc, ItemStack stack) {
        return mc.player.getCooldowns().isOnCooldown(stack.getItem());
    }

    /** Selects a hotbar slot of the local player. */
    static void selectSlot(Minecraft mc, int slot) {
        mc.player.getInventory().selected = slot;
    }

    /** The registry name of a sound, as "namespace:path". */
    static String soundName(SoundInstance sound) {
        return sound.getLocation().toString();
    }

    /** Presses a button like a click on it. */
    static void press(Button button) {
        button.onPress();
    }

    /** Saves a screenshot of the main render target as <game dir>/screenshots/<name>, on the render thread. */
    static void screenshot(Minecraft mc, String name, Consumer<String> done) {
        mc.tell(() -> Screenshot.grab(mc.gameDirectory, name, mc.getMainRenderTarget(), message -> done.accept(message.getString())));
    }

    /**
     * The log line about the Eternal Sword's two models: the one in the hand differs from the inventory one, and the
     * inventory one stays in the GUI. EternalSwordModel swaps the model of ItemRenderer.render by the display context.
     */
    static String swordModelLine(Minecraft mc, ItemStack sword) {
        BakedModel base = mc.getItemRenderer().getModel(sword, mc.level, mc.player, 0);
        return "sword in-hand model differs from the inventory one: "
                + (EternalSwordModel.inHandModel(base, sword, ItemDisplayContext.FIRST_PERSON_RIGHT_HAND) != base)
                + ", inventory model kept in the GUI: "
                + (EternalSwordModel.inHandModel(base, sword, ItemDisplayContext.GUI) == base);
    }
}
