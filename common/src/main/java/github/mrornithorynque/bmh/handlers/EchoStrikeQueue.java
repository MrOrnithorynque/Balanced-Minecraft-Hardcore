package github.mrornithorynque.bmh.handlers;

import java.util.List;

/**
 * The echoes a player has started and that have not landed yet. {@code ServerPlayer} implements it
 * through {@code E_ServerPlayerMixin}, so the waiting echoes live and die with the player.
 */
public interface EchoStrikeQueue {

    /** Server thread only. */
    List<EchoStrikeHandler.PendingEcho> bmh$pendingEchoes();
}
