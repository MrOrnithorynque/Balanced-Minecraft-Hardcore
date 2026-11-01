
package github.mrornithorynque.bmh.handlers;

import github.mrornithorynque.bmh.utilities.BMHGameRules;

import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.level.GameRules;
import net.minecraft.world.level.GameType;

import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

public class DisableDebugInfo {

    private static final Logger LOGGER = LogUtils.getLogger();

    private static boolean reduceDebugInfoSnapshot = false;

    // Called on every server player tick.
    public static void onPlayerTick(ServerPlayer serverPlayer) {

        MinecraftServer minecraftServer = serverPlayer.getServer();

        if (minecraftServer == null)
            return;

        GameRules gameRules = minecraftServer.getGameRules();
        GameRules.BooleanValue ruleReduceDebugInfo = gameRules.getRule(GameRules.RULE_REDUCEDDEBUGINFO);

        if (!gameRules.getRule(BMHGameRules.RULE_REMOVE_COORDINATES_IN_SURVIVAL).get())
            return;

        if (serverPlayer.gameMode.getGameModeForPlayer() != GameType.SURVIVAL) {
            reduceDebugInfoSnapshot = ruleReduceDebugInfo.get();
            ruleReduceDebugInfo.set(reduceDebugInfoSnapshot, minecraftServer);
            return;
        }

        ruleReduceDebugInfo.set(true, minecraftServer);
    }
}