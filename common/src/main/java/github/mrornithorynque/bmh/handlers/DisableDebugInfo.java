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

    // reducedDebugInfo is a world game rule, so forcing it on is saved with the world. We remember
    // that we did, and what it was before, to give it back when the rule is off or nobody is in
    // survival. A weak reference keeps a world we left from being held in memory. After a reload we
    // no longer know we forced it, so a world saved while forced keeps it until the player resets it.
    private static WeakReference<MinecraftServer> forcedServer = new WeakReference<>(null);
    private static boolean forcing = false;
    private static boolean valueBeforeForcing = false;

    // Called on every server player tick.
    public static void onPlayerTick(ServerPlayer serverPlayer) {

        MinecraftServer minecraftServer = serverPlayer.getServer();

        if (minecraftServer == null)
            return;

        // The state belongs to another world (singleplayer: left one, joined another).
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
            // set() tells every client, so only call it on a change.
            if (!ruleReduceDebugInfo.get())
                ruleReduceDebugInfo.set(true, minecraftServer);
        } else if (forcing) {
            forcing = false;
            ruleReduceDebugInfo.set(valueBeforeForcing, minecraftServer);
        }
    }
}
