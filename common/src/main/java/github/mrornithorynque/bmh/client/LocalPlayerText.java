package github.mrornithorynque.bmh.client;

import github.mrornithorynque.utilities.HexColor;
import github.mrornithorynque.utilities.TextDrawer;

import net.minecraft.client.Minecraft;
import net.minecraft.world.entity.player.Player;

/**
 * Client classes live here only: call it behind Services.PLATFORM.isPhysicalClient(), so that a
 * dedicated server never loads it (loading LocalPlayer there kills the server).
 */
public class LocalPlayerText {

    private static final int DURATION_MS = 5000;

    /**
     * TextDrawer renders on the client of this JVM, so the text only shows when the player is the
     * local one (singleplayer or LAN host). Compared by UUID: server code holds the ServerPlayer,
     * never the client's LocalPlayer instance.
     */
    public static void showToLocalPlayer(Player player, String message) {

        Player localPlayer = Minecraft.getInstance().player;

        if (localPlayer != null && localPlayer.getUUID().equals(player.getUUID())) {

            TextDrawer.getInstance().drawString(
                    message,
                    TextDrawer.ScreenPosition.CENTER,
                    HexColor.WHITE.getValue(),
                    DURATION_MS);
        }
    }
}
