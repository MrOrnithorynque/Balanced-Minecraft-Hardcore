package github.mrornithorynque.bmh.neoforge;

import org.slf4j.Logger;
import com.mojang.logging.LogUtils;

import github.mrornithorynque.bmh.BalancedMcHardcore;

import net.neoforged.bus.api.IEventBus;
import net.neoforged.fml.ModContainer;
import net.neoforged.fml.common.Mod;
import net.neoforged.fml.config.ModConfig;
import net.neoforged.fml.loading.FMLEnvironment;
import net.neoforged.neoforge.common.NeoForge;

@Mod(BalancedMcHardcore.MODID)
public class BalancedMcHardcoreNeoForge {

    private static final Logger LOGGER = LogUtils.getLogger();

    public BalancedMcHardcoreNeoForge(IEventBus modEventBus, ModContainer container) {

        LOGGER.info("[BalancedMcHardcoreMain] Loading...");

        container.registerConfig(ModConfig.Type.COMMON, NeoForgeConfig.SPEC);

        BalancedMcHardcore.init();
        NeoForgePlatformHelper.registerAll(modEventBus);

        NeoForge.EVENT_BUS.register(NeoForgeEvents.class);

        // The HUD text needs the client classes: never touch them on a dedicated server.
        if (FMLEnvironment.dist.isClient()) {
            modEventBus.register(NeoForgeClientEvents.class);
            modEventBus.addListener(NeoForgeAmuletClient::onClientSetup);
        }

        LOGGER.info("[BalancedMcHardcoreMain] Setup complete.");
    }
}
