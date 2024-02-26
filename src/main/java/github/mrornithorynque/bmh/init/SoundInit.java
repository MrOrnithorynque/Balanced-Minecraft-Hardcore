package github.mrornithorynque.bmh.init;

import javax.annotation.Nonnull;

import github.mrornithorynque.bmh.BalancedMcHardcoreMain;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.sounds.SoundEvent;
import net.minecraftforge.common.util.ForgeSoundType;
import net.minecraftforge.eventbus.api.IEventBus;
import net.minecraftforge.registries.DeferredRegister;
import net.minecraftforge.registries.ForgeRegistries;
import net.minecraftforge.registries.RegistryObject;

public class SoundInit {

    public static final DeferredRegister<SoundEvent> SOUND_EVENTS = DeferredRegister.create(ForgeRegistries.SOUND_EVENTS,
        BalancedMcHardcoreMain.MODID);

    public static final RegistryObject<SoundEvent> RESPAWN_SOUND = registerSoundEvents("end");
    public static final RegistryObject<SoundEvent> BELL_SOUND = registerSoundEvents("bell_sfx");

    private static RegistryObject<SoundEvent> registerSoundEvents(@Nonnull String name) {

        return SOUND_EVENTS.register(
            name,
            () -> SoundEvent.createVariableRangeEvent(new ResourceLocation(BalancedMcHardcoreMain.MODID, name)));
    }
}
