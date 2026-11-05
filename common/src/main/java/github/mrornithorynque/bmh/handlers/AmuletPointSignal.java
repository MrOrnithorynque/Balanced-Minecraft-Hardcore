package github.mrornithorynque.bmh.handlers;

import javax.annotation.Nullable;

import net.minecraft.core.BlockPos;
import net.minecraft.network.chat.Component;
import net.minecraft.network.protocol.game.ClientboundSetTitleTextPacket;
import net.minecraft.server.level.ServerPlayer;

/**
 * Tells a player's client to open the naming screen of a point (client/amulet/PointScreen). BMH has no
 * network channel: the signal is a vanilla title packet with the key bmh.point.open, as JourneySignal
 * does, and mixin.F_GuiTitleMixin takes it out before vanilla shows it. Every argument is a string:
 * x, y, z, the point's name (empty for a new point), the points the player uses, the cap, and "bed"
 * or "lodestone".
 */
public final class AmuletPointSignal {

    public static final String OPEN = "bmh.point.open";

    private AmuletPointSignal() {
    }

    public static void open(ServerPlayer player, BlockPos pos, @Nullable String name, int used, int max, boolean bed) {

        // A fake player of another mod can have no connection.
        if (player.connection == null) {
            return;
        }

        Component title = Component.translatableWithFallback(OPEN, "",
                String.valueOf(pos.getX()), String.valueOf(pos.getY()), String.valueOf(pos.getZ()),
                name == null ? "" : name, String.valueOf(used), String.valueOf(max), bed ? "bed" : "lodestone");

        player.connection.send(new ClientboundSetTitleTextPacket(title));
    }
}
