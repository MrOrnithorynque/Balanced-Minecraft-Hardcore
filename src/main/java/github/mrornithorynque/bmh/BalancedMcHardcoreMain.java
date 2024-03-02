package github.mrornithorynque.bmh;

import org.slf4j.Logger;
import com.mojang.logging.LogUtils;

import github.mrornithorynque.bmh.init.*;
import github.mrornithorynque.bmh.handlers.*;
import github.mrornithorynque.utilities.TextDrawer;
import github.mrornithorynque.bmh.utilities.BMHGameRules;

import net.minecraftforge.common.MinecraftForge;
import net.minecraftforge.eventbus.api.IEventBus;
import net.minecraftforge.fml.common.Mod;
import net.minecraftforge.fml.event.lifecycle.FMLCommonSetupEvent;
import net.minecraftforge.fml.javafmlmod.FMLJavaModLoadingContext;

@Mod(BalancedMcHardcoreMain.MODID)
public class BalancedMcHardcoreMain {

    public static final String MODID = "bmh";
    private static final Logger LOGGER = LogUtils.getLogger();

    public BalancedMcHardcoreMain() {

        LOGGER.info("[BalancedMcHardcoreMain] Loading...");

        IEventBus modEventBus = FMLJavaModLoadingContext.get().getModEventBus();

        modEventBus.addListener(this::commonSetup);

        ItemInit.ITEMS.register(modEventBus);
        BlockInit.BLOCKS.register(modEventBus);
        SoundInit.SOUND_EVENTS.register(modEventBus);
        CreativeTabInit.CREATIVE_MODE_TABS.register(modEventBus);

        BMHGameRules.init();

        modEventBus.addListener(this::setup);

        LOGGER.info("[BalancedMcHardcoreMain] Setup complete.");
    }

    private void commonSetup(final FMLCommonSetupEvent event) {
    }

    private void setup(final FMLCommonSetupEvent event) {

        MinecraftForge.EVENT_BUS.register(new RespawnEffectHandler());
        MinecraftForge.EVENT_BUS.register(new SleepHandler());
        MinecraftForge.EVENT_BUS.register(new EternalItemHandler());
        MinecraftForge.EVENT_BUS.register(new DisableDebugInfo());
        MinecraftForge.EVENT_BUS.register(new RespawnPositionHandler());
        MinecraftForge.EVENT_BUS.register(TextDrawer.getInstance());
    }
}
