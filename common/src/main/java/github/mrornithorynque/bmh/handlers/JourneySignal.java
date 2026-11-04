package github.mrornithorynque.bmh.handlers;

import net.minecraft.network.chat.Component;
import net.minecraft.network.protocol.game.ClientboundSetTitleTextPacket;
import net.minecraft.server.level.ServerPlayer;

/**
 * Tells a player's client about a distant respawn, so that it plays the journey cinematic (client/journey). BMH has no
 * network channel of its own: a vanilla title packet carries the signal, and the client mixin F_GuiTitleMixin takes
 * the BMH keys out before vanilla shows them. A client without BMH shows the fallback text: nothing, or the message.
 */
public final class JourneySignal {

    /** Sent at death: the next respawn is a distant one. */
    public static final String DEPARTURE = "bmh.journey.departure";
    /** Sent after the respawn. Its argument, when there is one, is the message to show. */
    public static final String ARRIVAL = "bmh.journey.arrival";
    /** The message of the cinematic, with the player name as argument. */
    public static final String MESSAGE = "bmh.journey.message";

    private JourneySignal() {
    }

    /**
     * Called last on death, once no other mod can cancel it: the client then fades to black on the death screen before
     * it asks for the respawn. A hardcore world makes a spectator of the player instead of a new journey.
     */
    public static void onPlayerDeath(ServerPlayer serverPlayer) {

        if (RespawnPositionHandler.isDistantRespawn(serverPlayer) && !serverPlayer.server.isHardcore()) {
            send(serverPlayer, Component.translatableWithFallback(DEPARTURE, ""));
        }
    }

    public static void sendArrival(ServerPlayer serverPlayer, boolean withMessage) {

        if (withMessage) {
            Component message = Component.translatableWithFallback(MESSAGE, "A new journey begins for you, %s.",
                    serverPlayer.getName());
            send(serverPlayer, Component.translatableWithFallback(ARRIVAL, "%s", message));
        } else {
            send(serverPlayer, Component.translatableWithFallback(ARRIVAL, ""));
        }
    }

    // A fake player of another mod can have no connection.
    private static void send(ServerPlayer serverPlayer, Component title) {

        if (serverPlayer.connection != null) {
            serverPlayer.connection.send(new ClientboundSetTitleTextPacket(title));
        }
    }
}
