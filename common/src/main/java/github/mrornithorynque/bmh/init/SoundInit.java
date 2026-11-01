package github.mrornithorynque.bmh.init;

import java.util.function.Supplier;

import javax.annotation.Nonnull;

import github.mrornithorynque.bmh.BalancedMcHardcore;
import github.mrornithorynque.bmh.platform.Services;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.sounds.SoundEvent;

public class SoundInit {

    public static final Supplier<SoundEvent> RESPAWN_SOUND = registerSoundEvents("end");
    public static final Supplier<SoundEvent> BELL_SOUND = registerSoundEvents("bell_sfx");

    private static Supplier<SoundEvent> registerSoundEvents(@Nonnull String name) {

        return Services.PLATFORM.register(
                Registries.SOUND_EVENT,
                name,
                () -> SoundEvent.createVariableRangeEvent(ResourceLocation.fromNamespaceAndPath(BalancedMcHardcore.MODID, name)));
    }

    public static void init() {
        /* This method is empty but ensures static fields are initialized. */ }
}
