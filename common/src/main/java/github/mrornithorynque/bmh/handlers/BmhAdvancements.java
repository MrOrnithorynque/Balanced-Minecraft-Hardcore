package github.mrornithorynque.bmh.handlers;

import org.jspecify.annotations.Nullable;

import github.mrornithorynque.bmh.BalancedMcHardcore;

import net.minecraft.advancements.AdvancementHolder;
import net.minecraft.resources.Identifier;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerPlayer;

/**
 * The BMH advancement tab. Some advancements use the vanilla trigger {@code minecraft:inventory_changed} and need no
 * code. The others use {@code minecraft:impossible} in the JSON and are awarded from the server code through this
 * class, so no handler sees the advancement API, which differs per Minecraft version. The data is in
 * {@code data/bmh/advancements/} (the folder is {@code advancement} from 1.21).
 */
public class BmhAdvancements {

    /** The exile distance, in Overworld blocks, that earns "Far, far away" and "The long way home". */
    public static final int FAR_AWAY_BLOCKS = 30000;

    // The only criterion of an advancement that the server code awards.
    private static final String CRITERION = "done";

    /**
     * Awards the advancement {@code bmh:<path>} to the player. Does nothing when the advancement is missing (a data
     * pack can remove it) or already done. The path is relative, such as "journey/exile". True when this call
     * completed it.
     */
    public static boolean award(ServerPlayer player, String path) {

        AdvancementHolder advancement = find(player.level().getServer(), path);

        return advancement != null && player.getAdvancements().award(advancement, CRITERION);
    }

    /** True when the server knows the advancement {@code bmh:<path>}. */
    public static boolean exists(MinecraftServer server, String path) {

        return find(server, path) != null;
    }

    /** True when the player has completed the advancement {@code bmh:<path>}. False when it does not exist. */
    public static boolean isDone(ServerPlayer player, String path) {

        AdvancementHolder advancement = find(player.level().getServer(), path);

        return advancement != null && player.getAdvancements().getOrStartProgress(advancement).isDone();
    }

    @Nullable
    private static AdvancementHolder find(MinecraftServer server, String path) {

        return server.getAdvancements().get(Identifier.fromNamespaceAndPath(BalancedMcHardcore.MODID, path));
    }
}
