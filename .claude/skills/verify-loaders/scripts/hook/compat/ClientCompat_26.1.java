package github.mrornithorynque.bmh.client;

import java.util.Map;
import java.util.function.Consumer;

import net.minecraft.SharedConstants;
import net.minecraft.client.Minecraft;
import net.minecraft.client.Screenshot;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.input.MouseButtonEvent;
import net.minecraft.client.input.MouseButtonInfo;
import net.minecraft.client.renderer.item.ItemStackRenderState;
import net.minecraft.client.resources.model.sprite.Material;
import net.minecraft.client.resources.sounds.SoundInstance;
import net.minecraft.util.RandomSource;
import net.minecraft.world.item.ItemDisplayContext;
import net.minecraft.world.item.ItemStack;

/**
 * THROWAWAY verification hook (never committed, see branch.sh and death_client.sh): the calls of SmokeClient and
 * JourneySmoke that differ between Minecraft versions. This copy serves 26.1 (built on 26.1.2). The 1.21.1 copy has the
 * same methods with the same signatures. Both hooks call these and nothing else version specific.
 */
final class ClientCompat {

    // 26.1 made the game rules a registry: the old names of the hooks map to the new ids (see also rule_names.js).
    private static final Map<String, String> RULES = Map.of(
            "doDaylightCycle", "minecraft:advance_time",
            "doWeatherCycle", "minecraft:advance_weather",
            "doMobSpawning", "minecraft:spawn_mobs",
            "doImmediateRespawn", "minecraft:immediate_respawn",
            "randomDeathSpawnPoint", "bmh:random_death_spawn_point",
            "displayTextOnRespawn", "bmh:display_text_on_respawn");

    private ClientCompat() {
    }

    /** The name of the running Minecraft version, e.g. 26.1.2. */
    static String versionName() {
        return SharedConstants.getCurrentVersion().name();
    }

    /** A give command for an Eternal Bundle with one diamond in it (the bundle_contents component). */
    static String filledBundleCommand() {
        return "give @a bmh:eternal_bundle[minecraft:bundle_contents=[{id:\"minecraft:diamond\",count:1}]]";
    }

    /** A gamerule command. The name is the 1.21.1 one; 26.1 renamed the rules and BMH's moved to its own namespace. */
    static String gamerule(String name, Object value) {
        String id = RULES.get(name);
        if (id == null) throw new IllegalArgumentException("no 26.1 id for the game rule " + name);
        return "gamerule " + id + " " + value;
    }

    /** The time of day of the client level. */
    static long dayTime(Minecraft mc) {
        return mc.level.getOverworldClockTime();
    }

    /** The world spawn point of the client level, as "x, y, z". */
    static String worldSpawn(Minecraft mc) {
        return mc.level.getRespawnData().pos().toShortString();
    }

    /** True while the item of this stack is on cooldown for the local player. */
    static boolean onCooldown(Minecraft mc, ItemStack stack) {
        return mc.player.getCooldowns().isOnCooldown(stack);
    }

    /** Selects a hotbar slot of the local player. */
    static void selectSlot(Minecraft mc, int slot) {
        mc.player.getInventory().setSelectedSlot(slot);
    }

    /** The registry name of a sound, as "namespace:path". */
    static String soundName(SoundInstance sound) {
        return sound.getIdentifier().toString();
    }

    /** Presses a button like a left click on it. */
    static void press(Button button) {
        button.onPress(new MouseButtonEvent(0, 0, new MouseButtonInfo(0, 0)));
    }

    /**
     * Saves a screenshot of the main render target as <game dir>/screenshots/<name>, on the render thread. Like the
     * removed Minecraft.tell, it queues the task for the start of the next frame instead of running it at once.
     */
    static void screenshot(Minecraft mc, String name, Consumer<String> done) {
        mc.schedule(mc.wrapRunnable(
                () -> Screenshot.grab(mc.gameDirectory, name, mc.getMainRenderTarget(), 1, message -> done.accept(message.getString()))));
    }

    /**
     * The log line about the Eternal Sword's two models. 26.1 chooses the model in the item definition (a select on the
     * display context), so this resolves the stack for three contexts and compares the texture of the first layer:
     * NONE gives the fallback (the inventory model), a first person hand gives the in-hand model, the GUI gives the
     * fallback again.
     */
    static String swordModelLine(Minecraft mc, ItemStack sword) {
        String base = texture(mc, sword, ItemDisplayContext.NONE);
        String inHand = texture(mc, sword, ItemDisplayContext.FIRST_PERSON_RIGHT_HAND);
        String gui = texture(mc, sword, ItemDisplayContext.GUI);
        return "sword textures: inventory " + base + ", hand " + inHand + ", GUI " + gui
                + "; sword in-hand model differs from the inventory one: " + !inHand.equals(base)
                + ", inventory model kept in the GUI: " + gui.equals(base);
    }

    private static String texture(Minecraft mc, ItemStack stack, ItemDisplayContext context) {
        ItemStackRenderState state = new ItemStackRenderState();
        mc.getItemModelResolver().updateForTopItem(state, stack, context, mc.level, mc.player, 0);
        Material.Baked material = state.pickParticleMaterial(RandomSource.create(0));
        return material == null ? "none" : material.sprite().contents().name().toString();
    }
}
