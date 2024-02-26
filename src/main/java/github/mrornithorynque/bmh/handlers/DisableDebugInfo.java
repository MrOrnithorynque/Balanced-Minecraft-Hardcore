
package github.mrornithorynque.bmh.handlers;

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

    @SubscribeEvent
    public static void onPlayerTick(PlayerTickEvent event) {

        Player player = event.player;

        if (player instanceof ServerPlayer) {

            ServerPlayer serverPlayer = (ServerPlayer) player;

            if (serverPlayer.gameMode.getGameModeForPlayer() != GameType.SURVIVAL) {
                return;
            }

            MinecraftServer minecraftServer = serverPlayer.getServer();

            if (minecraftServer != null) {

                GameRules gameRules = minecraftServer.getGameRules();
                GameRules.BooleanValue rule = gameRules.getRule(GameRules.RULE_REDUCEDDEBUGINFO);
                rule.set(true, minecraftServer);
            }
        }
    }
}