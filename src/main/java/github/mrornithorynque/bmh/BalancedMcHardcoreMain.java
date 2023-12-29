package github.mrornithorynque.bmh;

import github.mrornithorynque.bmh.init.ItemInit;
import github.mrornithorynque.bmh.handlers.PlayerDeathHandler;
import github.mrornithorynque.bmh.handlers.RespawnMessageOverlay;
import github.mrornithorynque.bmh.init.BlockInit;

import com.mojang.logging.LogUtils;

import net.minecraftforge.common.MinecraftForge;
import net.minecraftforge.eventbus.api.IEventBus;
import net.minecraftforge.fml.common.Mod;
import net.minecraftforge.fml.event.lifecycle.FMLCommonSetupEvent;
import net.minecraftforge.fml.javafmlmod.FMLJavaModLoadingContext;
import org.slf4j.Logger;

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

        // Register the setup method for mod loading
        modEventBus.addListener(this::setup);
    }

    private void commonSetup(final FMLCommonSetupEvent event) {

        // Some common setup code
        LOGGER.info("HELLO FROM COMMON SETUP");
    }

    private void setup(final FMLCommonSetupEvent event) {

        MinecraftForge.EVENT_BUS.register(new PlayerDeathHandler());
        MinecraftForge.EVENT_BUS.register(new RespawnMessageOverlay());
    }
}
