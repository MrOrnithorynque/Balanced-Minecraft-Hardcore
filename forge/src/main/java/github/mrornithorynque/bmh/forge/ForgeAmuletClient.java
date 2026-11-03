package github.mrornithorynque.bmh.forge;

import github.mrornithorynque.bmh.client.AmuletCompass;
import github.mrornithorynque.bmh.init.ItemInit;

import net.minecraft.client.renderer.item.ItemProperties;

import net.minecraftforge.fml.event.lifecycle.FMLClientSetupEvent;

/** Mod bus, client only: the amulet's needle, an item property the model switches on. */
public class ForgeAmuletClient {

    public static void onClientSetup(FMLClientSetupEvent event) {

        event.enqueueWork(() -> ItemProperties.register(ItemInit.NAVIGATOR_AMULET.get(), AmuletCompass.ANGLE, AmuletCompass.angle()));
    }
}
