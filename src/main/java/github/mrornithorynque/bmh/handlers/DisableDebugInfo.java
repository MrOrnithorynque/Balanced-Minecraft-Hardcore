
package github.mrornithorynque.bmh.handlers;

import github.mrornithorynque.bmh.utilities.BMHGameRules;

import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.level.GameRules;
import net.minecraft.world.level.GameType;
import net.minecraftforge.event.TickEvent.PlayerTickEvent;
import net.minecraftforge.eventbus.api.SubscribeEvent;
import net.minecraftforge.fml.common.Mod;

import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

@Mod.EventBusSubscriber
public class DisableDebugInfo {

    private static final Logger LOGGER = LogUtils.getLogger();

    private static boolean reduceDebugInfoSnapshot = false;

    @SubscribeEvent
    public static void onPlayerTick(PlayerTickEvent event) {

        Player player = event.player;

        if (player instanceof ServerPlayer) {

            ServerPlayer serverPlayer = (ServerPlayer) player;
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
}