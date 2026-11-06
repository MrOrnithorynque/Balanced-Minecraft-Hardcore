package github.mrornithorynque.bmh.handlers;

import java.lang.ref.WeakReference;

import github.mrornithorynque.bmh.utilities.BMHGameRules;

import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.level.gamerules.GameRules;
import net.minecraft.world.level.GameType;

import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

public class DisableDebugInfo {

    private static final Logger LOGGER = LogUtils.getLogger();

    // The game rule reduced_debug_info is part of the world, so the world saves it when we force it on.
    // We remember the value from before. We restore it when our rule is off or no player is in survival.
    // After a reload we forget this, so the player must reset the game rule by hand.
    // The weak reference lets the game free a world that the player left.
    private static WeakReference<MinecraftServer> forcedServer = new WeakReference<>(null);
    private static boolean forcing = false;
    private static boolean valueBeforeForcing = false;

    // Called on every server player tick.
    public static void onPlayerTick(ServerPlayer serverPlayer) {

        MinecraftServer minecraftServer = serverPlayer.level().getServer();

        if (minecraftServer == null)
            return;

        // This state is from another world: the player left one and joined another.
        if (forcing && forcedServer.get() != minecraftServer)
            forcing = false;

        GameRules gameRules = minecraftServer.getGameRules();
        boolean reducedDebugInfo = gameRules.get(GameRules.REDUCED_DEBUG_INFO);

        boolean shouldForce = gameRules.get(BMHGameRules.RULE_REMOVE_COORDINATES_IN_SURVIVAL)
                && minecraftServer.getPlayerList().getPlayers().stream()
                        .anyMatch(player -> player.gameMode.getGameModeForPlayer() == GameType.SURVIVAL);

        if (shouldForce) {
            if (!forcing) {
                forcing = true;
                forcedServer = new WeakReference<>(minecraftServer);
                valueBeforeForcing = reducedDebugInfo;
            }
            // set() sends a packet to every client, so call it only when the value changes.
            if (!reducedDebugInfo)
                gameRules.set(GameRules.REDUCED_DEBUG_INFO, true, minecraftServer);
        } else if (forcing) {
            forcing = false;
            gameRules.set(GameRules.REDUCED_DEBUG_INFO, valueBeforeForcing, minecraftServer);
        }
    }
}
