package github.mrornithorynque.bmh;

import org.slf4j.Logger;
import com.mojang.logging.LogUtils;

import github.mrornithorynque.bmh.forge.ForgeAmuletClient;
import github.mrornithorynque.bmh.forge.ForgeClientEvents;
import github.mrornithorynque.bmh.forge.ForgeClientSetup;
import github.mrornithorynque.bmh.forge.ForgeEvents;
import github.mrornithorynque.bmh.forge.ForgePlatformHelper;

import net.minecraftforge.common.MinecraftForge;
import net.minecraftforge.eventbus.api.IEventBus;
import net.minecraftforge.fml.ModLoadingContext;
import net.minecraftforge.fml.common.Mod;
import net.minecraftforge.fml.config.ModConfig;
import net.minecraftforge.fml.event.lifecycle.FMLCommonSetupEvent;
import net.minecraftforge.fml.javafmlmod.FMLJavaModLoadingContext;
import net.minecraftforge.fml.loading.FMLEnvironment;

@Mod(BalancedMcHardcore.MODID)
public class BalancedMcHardcoreMain {

    public static final String MODID = BalancedMcHardcore.MODID;
    private static final Logger LOGGER = LogUtils.getLogger();

    public BalancedMcHardcoreMain() {

        LOGGER.info("[BalancedMcHardcoreMain] Loading...");

        IEventBus modEventBus = FMLJavaModLoadingContext.get().getModEventBus();

        ModLoadingContext.get().registerConfig(ModConfig.Type.COMMON, Config.SPEC);

        BalancedMcHardcore.init();
        ForgePlatformHelper.registerAll(modEventBus);

        // Client setup (item properties) is a mod bus event, and needs the client classes: never on a dedicated server.
        if (FMLEnvironment.dist.isClient()) {
            modEventBus.addListener(ForgeAmuletClient::onClientSetup);
        }

        modEventBus.addListener(this::setup);

        // Item properties need the client classes: never touch them on a dedicated server.
        if (FMLEnvironment.dist.isClient()) {
            modEventBus.addListener(ForgeClientSetup::onClientSetup);
        }

        LOGGER.info("[BalancedMcHardcoreMain] Setup complete.");
    }

    private void setup(final FMLCommonSetupEvent event) {

        MinecraftForge.EVENT_BUS.register(ForgeEvents.class);

        // The HUD text needs the client classes: never touch them on a dedicated server.
        if (FMLEnvironment.dist.isClient()) {
            MinecraftForge.EVENT_BUS.register(ForgeClientEvents.class);
        }
    }
}
