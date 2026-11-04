package github.mrornithorynque.bmh.client.journey;

import github.mrornithorynque.bmh.mixin.F_MusicManagerAccessor;
import github.mrornithorynque.bmh.mixin.F_SoundInstanceVolumeAccessor;

import net.minecraft.client.Minecraft;
import net.minecraft.client.resources.sounds.SoundInstance;
import net.minecraft.sounds.SoundSource;

/**
 * Fades the game music out while the journey fades to black. Vanilla reads the volume of a playing sound only when it
 * starts, and again when a volume slider moves: so each step sets the volume of the sound and then asks the sound
 * manager to apply it. Render thread only.
 */
final class JourneyMusic {

    private SoundInstance music;
    private float startVolume;
    private boolean stopped;

    /** Sets the music to the given part (1 is full volume, 0 is silence) of the volume it had when the fade began. */
    void fade(float part) {

        if (this.stopped) {
            return;
        }

        Minecraft minecraft = Minecraft.getInstance();
        SoundInstance current = ((F_MusicManagerAccessor) minecraft.getMusicManager()).bmh$getCurrentMusic();

        if (current == null) {
            return;
        }

        // The music manager can start another track during the fade: that track fades from its own volume.
        if (current != this.music) {
            this.music = current;
            this.startVolume = ((F_SoundInstanceVolumeAccessor) current).bmh$getVolume();
        }

        ((F_SoundInstanceVolumeAccessor) current).bmh$setVolume(this.startVolume * part);
        minecraft.getSoundManager().updateSourceVolume(SoundSource.MUSIC, 1.0F);

        if (part <= 0.0F) {
            minecraft.getMusicManager().stopPlaying();
            this.stopped = true;
        }
    }
}
