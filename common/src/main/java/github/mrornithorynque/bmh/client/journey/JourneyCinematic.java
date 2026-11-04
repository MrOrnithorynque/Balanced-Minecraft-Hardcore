package github.mrornithorynque.bmh.client.journey;

import java.lang.ref.WeakReference;

import github.mrornithorynque.bmh.handlers.JourneySignal;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.screens.Overlay;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.network.chat.Component;
import net.minecraft.network.chat.contents.TranslatableContents;

/**
 * Starts the journey cinematic (JourneyOverlay) from the server signals of JourneySignal. The client mixins call it;
 * a dedicated server never loads it. Render thread only.
 */
public final class JourneyCinematic {

    // The dead player that the server armed a journey for, at its death. Weak references: a player who quits from the
    // death screen must not keep the old client world in memory.
    private static WeakReference<LocalPlayer> armedPlayer = new WeakReference<>(null);
    // The dead player whose respawn request waits for the fade to black.
    private static WeakReference<LocalPlayer> waitingPlayer = new WeakReference<>(null);
    private static boolean sendingRespawn;

    private JourneyCinematic() {
    }

    /** From F_GuiTitleMixin. Returns true for a journey signal: vanilla must not show it as a title. */
    public static boolean onTitle(Component title) {

        if (!(title.getContents() instanceof TranslatableContents contents)) {
            return false;
        }

        Minecraft minecraft = Minecraft.getInstance();

        switch (contents.getKey()) {
            case JourneySignal.DEPARTURE -> armedPlayer = new WeakReference<>(minecraft.player);
            case JourneySignal.ARRIVAL -> {
                Object[] args = contents.getArgs();
                Component message = args.length > 0 && args[0] instanceof Component component ? component : null;

                if (minecraft.getOverlay() instanceof JourneyOverlay overlay) {
                    overlay.arrive(message);
                } else if (minecraft.getOverlay() == null) {
                    // No fade from the death screen (a player who joined while dead): the cinematic starts in black.
                    JourneyOverlay overlay = new JourneyOverlay(minecraft, null);
                    overlay.arrive(message);
                    minecraft.setOverlay(overlay);
                }
            }
            default -> {
                return false;
            }
        }

        return true;
    }

    /** From F_LocalPlayerRespawnMixin. Returns true when the respawn waits for the fade to black. */
    public static boolean onRespawn(LocalPlayer player) {

        if (sendingRespawn) {
            return false;
        }

        // Minecraft calls respawn() again at each tick while a dead player has no death screen (immediate respawn).
        if (player == waitingPlayer.get()) {
            return true;
        }

        if (player != armedPlayer.get()) {
            return false;
        }

        armedPlayer = new WeakReference<>(null);
        Minecraft minecraft = Minecraft.getInstance();

        if (minecraft.getOverlay() != null) {
            return false;
        }

        waitingPlayer = new WeakReference<>(player);
        minecraft.setOverlay(new JourneyOverlay(minecraft, player));
        return true;
    }

    /**
     * From F_MinecraftOverlayMixin, before Minecraft changes its overlay. Another overlay (a resource reload) can take
     * the place of the journey during the fade: then the respawn request goes now, or the player stays dead. Nothing
     * can send it later: LocalPlayer removes itself 20 ticks after its death, and the level ticks no removed entity.
     */
    public static void onOverlayChange(Overlay next) {

        if (!(next instanceof JourneyOverlay) && Minecraft.getInstance().getOverlay() instanceof JourneyOverlay) {
            sendRespawn();
        }
    }

    // The overlay calls it once the screen is black.
    static void sendRespawn() {

        LocalPlayer player = waitingPlayer.get();
        waitingPlayer = new WeakReference<>(null);

        // After a disconnect there is no server to ask.
        if (player == null || player != Minecraft.getInstance().player) {
            return;
        }

        sendingRespawn = true;

        try {
            player.respawn();
        } finally {
            sendingRespawn = false;
        }
    }
}
