package github.mrornithorynque.bmh.forge;

import github.mrornithorynque.bmh.handlers.DisableDebugInfo;
import github.mrornithorynque.bmh.handlers.EternalItemHandler;
import github.mrornithorynque.bmh.handlers.JourneySignal;
import github.mrornithorynque.bmh.handlers.KeepItemAfterDeath;
import github.mrornithorynque.bmh.handlers.RespawnEffectHandler;
import github.mrornithorynque.bmh.handlers.RespawnPositionHandler;
import github.mrornithorynque.bmh.handlers.SleepHandler;

import net.minecraft.server.level.ServerPlayer;

import net.minecraftforge.event.TickEvent.PlayerTickEvent;
import net.minecraftforge.event.entity.living.LivingDeathEvent;
import net.minecraftforge.event.entity.player.EntityItemPickupEvent;
import net.minecraftforge.event.entity.player.PlayerEvent;
import net.minecraftforge.event.entity.player.PlayerWakeUpEvent;
import net.minecraftforge.eventbus.api.listener.Priority;

/**
 * Forge side of the common handlers. The priorities give the order the handlers rely on: the
 * respawn distance is measured before the Eternal Items leave the inventory, and the respawn
 * effects play after the kept items come back. The distant respawn position is not an event:
 * mixin/F_PlayerListRespawnMixin puts it into vanilla's respawn.
 */
public class ForgeEvents {

    // Forge 26 has one bus per event (EventBus 7). A listener of a cancellable event that returns false cancels nothing.
    public static void register() {

        LivingDeathEvent.BUS.addListener(Priority.HIGHEST, ForgeEvents::measureRespawnDistance);
        LivingDeathEvent.BUS.addListener(ForgeEvents::keepEternalItems);
        LivingDeathEvent.BUS.addListener(Priority.LOWEST, ForgeEvents::announceJourney);
        PlayerEvent.PlayerRespawnEvent.BUS.addListener(ForgeEvents::giveBackEternalItems);
        PlayerEvent.PlayerRespawnEvent.BUS.addListener(Priority.LOWEST, ForgeEvents::playRespawnEffects);
        PlayerWakeUpEvent.BUS.addListener(ForgeEvents::onPlayerWakeUp);
        EntityItemPickupEvent.BUS.addListener(ForgeEvents::onItemPickup);
        PlayerTickEvent.Post.BUS.addListener(ForgeEvents::onPlayerTick);
    }

    private static boolean measureRespawnDistance(LivingDeathEvent event) {

        if (event.getEntity() instanceof ServerPlayer serverPlayer) {
            RespawnPositionHandler.onPlayerDeath(serverPlayer);
        }
        return false;
    }

    private static boolean keepEternalItems(LivingDeathEvent event) {

        if (event.getEntity() instanceof ServerPlayer serverPlayer) {
            KeepItemAfterDeath.onPlayerDeath(serverPlayer);
        }
        return false;
    }

    // Last, and only when no listener cancelled the death: a revived player must not see the journey.
    private static boolean announceJourney(LivingDeathEvent event) {

        if (event.getEntity() instanceof ServerPlayer serverPlayer) {
            JourneySignal.onPlayerDeath(serverPlayer);
        }
        return false;
    }

    private static void giveBackEternalItems(PlayerEvent.PlayerRespawnEvent event) {

        KeepItemAfterDeath.onPlayerRespawn(event.getEntity());
    }

    private static void playRespawnEffects(PlayerEvent.PlayerRespawnEvent event) {

        RespawnEffectHandler.onPlayerRespawn(event.getEntity(), event.isEndConquered());
    }

    private static void onPlayerWakeUp(PlayerWakeUpEvent event) {

        SleepHandler.onPlayerWakeUp(event.getEntity());
    }

    // Returns true to cancel the pickup.
    private static boolean onItemPickup(EntityItemPickupEvent event) {

        return EternalItemHandler.shouldBlockPickup(event.getEntity(), event.getItem().getItem());
    }

    // The end of the player tick, as on Fabric and NeoForge.
    private static void onPlayerTick(PlayerTickEvent.Post event) {

        if (event.player() instanceof ServerPlayer serverPlayer) {
            DisableDebugInfo.onPlayerTick(serverPlayer);
            EternalItemHandler.onPlayerTick(serverPlayer);
        }
    }
}
