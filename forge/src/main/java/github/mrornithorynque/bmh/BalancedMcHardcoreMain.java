package github.mrornithorynque.bmh;

import org.slf4j.Logger;
import com.mojang.logging.LogUtils;

import github.mrornithorynque.bmh.forge.ForgeClientEvents;
import github.mrornithorynque.bmh.forge.ForgeEvents;
import github.mrornithorynque.bmh.forge.ForgePlatformHelper;

import net.minecraftforge.common.MinecraftForge;
import net.minecraftforge.eventbus.api.IEventBus;
import net.minecraftforge.fml.common.Mod;
import net.minecraftforge.fml.config.ModConfig;
import net.minecraftforge.fml.event.lifecycle.FMLCommonSetupEvent;
import net.minecraftforge.fml.javafmlmod.FMLJavaModLoadingContext;
import net.minecraftforge.fml.loading.FMLEnvironment;

@Mod(BalancedMcHardcore.MODID)
public class BalancedMcHardcoreMain {

    public static final String MODID = BalancedMcHardcore.MODID;
    private static final Logger LOGGER = LogUtils.getLogger();

    // Forge passes the loading context to the constructor (the static get() is on its way out).
    public BalancedMcHardcoreMain(FMLJavaModLoadingContext context) {

        LOGGER.info("[BalancedMcHardcoreMain] Loading...");

        IEventBus modEventBus = context.getModEventBus();

        context.registerConfig(ModConfig.Type.COMMON, Config.SPEC);

        BalancedMcHardcore.init();
        ForgePlatformHelper.registerAll(modEventBus);

        modEventBus.addListener(this::setup);

        // The HUD text and the config screen need the client classes: never touch them on a dedicated server.
        if (FMLEnvironment.dist.isClient()) {
            modEventBus.register(ForgeClientEvents.class);
            ForgeClientEvents.registerConfigScreen(context);
        }

        LOGGER.info("[BalancedMcHardcoreMain] Setup complete.");
    }

    private void setup(final FMLCommonSetupEvent event) {

        MinecraftForge.EVENT_BUS.register(ForgeEvents.class);
    }
}
