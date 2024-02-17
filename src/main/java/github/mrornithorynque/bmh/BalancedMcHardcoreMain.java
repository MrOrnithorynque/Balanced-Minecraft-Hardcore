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

        IEventBus modEventBus = FMLJavaModLoadingContext.get().getModEventBus();

        // Register the commonSetup method for mod loading
        modEventBus.addListener(this::commonSetup);

        // Register the ItemLoader to the mod event bus
        ItemInit.ITEMS.register(modEventBus);
        BlockInit.BLOCKS.register(modEventBus);
        SoundInit.SOUND_EVENT.register(modEventBus);
        CreativeTabInit.CREATIVE_MODE_TABS.register(modEventBus);
        BMHGameRules.init();

        // Register the setup method for mod loading
        modEventBus.addListener(this::setup);
    }

    private void commonSetup(final FMLCommonSetupEvent event) {
    }

    private void setup(final FMLCommonSetupEvent event) {

        MinecraftForge.EVENT_BUS.register(new RespawnMessageOverlay());
        MinecraftForge.EVENT_BUS.register(new SleepHandler());
        MinecraftForge.EVENT_BUS.register(new EternalItemHandler());
        MinecraftForge.EVENT_BUS.register(new SetFarRespawnPosition());
        MinecraftForge.EVENT_BUS.register(TextDrawer.getInstance());

    }
}
