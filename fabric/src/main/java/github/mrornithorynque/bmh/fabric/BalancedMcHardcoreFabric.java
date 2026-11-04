package github.mrornithorynque.bmh.fabric;

import org.slf4j.Logger;
import com.mojang.logging.LogUtils;

import github.mrornithorynque.bmh.BalancedMcHardcore;
import github.mrornithorynque.bmh.handlers.DisableDebugInfo;
import github.mrornithorynque.bmh.handlers.EternalItemHandler;
import github.mrornithorynque.bmh.handlers.KeepItemAfterDeath;
import github.mrornithorynque.bmh.handlers.RespawnEffectHandler;

import net.fabricmc.api.ModInitializer;
import net.fabricmc.fabric.api.entity.event.v1.ServerPlayerEvents;
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerTickEvents;
import net.minecraft.server.level.ServerPlayer;

/**
 * Fabric side of the common handlers. Death, wake-up and item pickup go through the mixins in
 * {@code fabric.mixin}: Fabric API's own events for those fire at different moments than Forge's.
 */
public class BalancedMcHardcoreFabric implements ModInitializer {

    private static final Logger LOGGER = LogUtils.getLogger();

    @Override
    public void onInitialize() {

        LOGGER.info("[BalancedMcHardcoreMain] Loading...");

        BalancedMcHardcore.init();
        // Writes config/bmh.properties on first launch, as Forge does with its config.
        FabricConfig.get();

        ServerTickEvents.END_SERVER_TICK.register(server -> {
            for (ServerPlayer serverPlayer : server.getPlayerList().getPlayers()) {
                DisableDebugInfo.onPlayerTick(serverPlayer);
                EternalItemHandler.onPlayerTick(serverPlayer);
            }
        });

        // Same order as the Forge priorities: give the items back, then the effects. The position is
        // already set: mixin/F_PlayerListRespawnMixin puts it into vanilla's respawn.
        // "alive" is true when the player comes back from the End, Forge's isEndConquered.
        ServerPlayerEvents.AFTER_RESPAWN.register((oldPlayer, newPlayer, alive) -> {
            KeepItemAfterDeath.onPlayerRespawn(newPlayer);
            RespawnEffectHandler.onPlayerRespawn(newPlayer, alive);
        });

        LOGGER.info("[BalancedMcHardcoreMain] Setup complete.");
    }
}
