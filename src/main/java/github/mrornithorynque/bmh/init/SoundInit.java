package github.mrornithorynque.bmh.init;

import github.mrornithorynque.bmh.BalancedMcHardcoreMain;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.sounds.SoundEvent;
import net.minecraftforge.registries.DeferredRegister;
import net.minecraftforge.registries.ForgeRegistries;
import net.minecraftforge.registries.RegistryObject;

public class SoundInit {

    public static final DeferredRegister<SoundEvent> SOUND_EVENT =
        DeferredRegister.create(ForgeRegistries.SOUND_EVENTS, BalancedMcHardcoreMain.MODID);

    public static final RegistryObject<SoundEvent> TEST_SOUND =
        registerSoundEvents("test_sound");

    private static RegistryObject<SoundEvent> registerSoundEvents(String string) {

        return SOUND_EVENT.register(
            string,
            () -> SoundEvent.createVariableRangeEvent(new ResourceLocation(BalancedMcHardcoreMain.MODID, string)));
    }
}
