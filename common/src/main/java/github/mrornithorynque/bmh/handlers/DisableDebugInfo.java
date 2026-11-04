package github.mrornithorynque.bmh.handlers;

import java.lang.ref.WeakReference;

import github.mrornithorynque.bmh.utilities.BMHGameRules;

import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.level.GameRules;
import net.minecraft.world.level.GameType;

import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

public class DisableDebugInfo {

    private static final Logger LOGGER = LogUtils.getLogger();

    // The game rule reducedDebugInfo is part of the world, so the world saves it when we force it on.
    // We remember the value from before. We restore it when our rule is off or no player is in survival.
    // After a reload we forget this, so the player must reset the game rule by hand.
    // The weak reference lets the game free a world that the player left.
    private static WeakReference<MinecraftServer> forcedServer = new WeakReference<>(null);
    private static boolean forcing = false;
    private static boolean valueBeforeForcing = false;

    // Called on every server player tick.
    public static void onPlayerTick(ServerPlayer serverPlayer) {

        MinecraftServer minecraftServer = serverPlayer.getServer();

        if (minecraftServer == null)
            return;

        // This state is from another world: the player left one and joined another.
        if (forcing && forcedServer.get() != minecraftServer)
            forcing = false;

        GameRules gameRules = minecraftServer.getGameRules();
        GameRules.BooleanValue ruleReduceDebugInfo = gameRules.getRule(GameRules.RULE_REDUCEDDEBUGINFO);

        boolean shouldForce = gameRules.getRule(BMHGameRules.RULE_REMOVE_COORDINATES_IN_SURVIVAL).get()
                && minecraftServer.getPlayerList().getPlayers().stream()
                        .anyMatch(player -> player.gameMode.getGameModeForPlayer() == GameType.SURVIVAL);

        if (shouldForce) {
            if (!forcing) {
                forcing = true;
                forcedServer = new WeakReference<>(minecraftServer);
                valueBeforeForcing = ruleReduceDebugInfo.get();
            }
            // set() sends a packet to every client, so call it only when the value changes.
            if (!ruleReduceDebugInfo.get())
                ruleReduceDebugInfo.set(true, minecraftServer);
        } else if (forcing) {
            forcing = false;
            ruleReduceDebugInfo.set(valueBeforeForcing, minecraftServer);
        }
    }
}
