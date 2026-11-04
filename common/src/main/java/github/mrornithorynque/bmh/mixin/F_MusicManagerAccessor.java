package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

import net.minecraft.client.resources.sounds.SoundInstance;
import net.minecraft.client.sounds.MusicManager;

/** MusicManager.currentMusic is private on every Minecraft version. It is the game music that plays now, or null. */
@Mixin(MusicManager.class)
public interface F_MusicManagerAccessor {

    @Accessor("currentMusic")
    SoundInstance bmh$getCurrentMusic();
}
