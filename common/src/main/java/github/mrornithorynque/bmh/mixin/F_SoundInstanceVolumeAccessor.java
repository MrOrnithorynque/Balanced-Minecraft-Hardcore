package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

import net.minecraft.client.resources.sounds.AbstractSoundInstance;

/** AbstractSoundInstance.volume is protected on every Minecraft version. */
@Mixin(AbstractSoundInstance.class)
public interface F_SoundInstanceVolumeAccessor {

    @Accessor("volume")
    float bmh$getVolume();

    @Accessor("volume")
    void bmh$setVolume(float volume);
}
