package github.mrornithorynque.bmh.neoforge;

import github.mrornithorynque.bmh.handlers.DisableDebugInfo;
import github.mrornithorynque.bmh.handlers.EternalItemHandler;
import github.mrornithorynque.bmh.handlers.JourneySignal;
import github.mrornithorynque.bmh.handlers.KeepItemAfterDeath;
import github.mrornithorynque.bmh.handlers.RespawnEffectHandler;
import github.mrornithorynque.bmh.handlers.RespawnPositionHandler;
import github.mrornithorynque.bmh.handlers.SleepHandler;

import net.minecraft.server.level.ServerPlayer;

import net.neoforged.bus.api.EventPriority;
import net.neoforged.bus.api.SubscribeEvent;
import net.neoforged.neoforge.event.entity.living.LivingDeathEvent;
import net.neoforged.neoforge.event.entity.player.EntityItemPickupEvent;
import net.neoforged.neoforge.event.entity.player.PlayerEvent;
import net.neoforged.neoforge.event.entity.player.PlayerWakeUpEvent;
import net.neoforged.neoforge.event.TickEvent.PlayerTickEvent;

/**
 * NeoForge side of the common handlers, same order as forge/ForgeEvents: the respawn distance is
 * measured before the Eternal Items leave the inventory, and the respawn effects play after the
 * kept items come back. mixin/F_PlayerListRespawnMixin puts the distant respawn position into
 * vanilla's respawn.
 */
public class NeoForgeEvents {

    @SubscribeEvent(priority = EventPriority.HIGHEST)
    public static void measureRespawnDistance(LivingDeathEvent event) {

        if (event.getEntity() instanceof ServerPlayer serverPlayer) {
            RespawnPositionHandler.onPlayerDeath(serverPlayer);
        }
    }

    @SubscribeEvent
    public static void keepEternalItems(LivingDeathEvent event) {

        if (event.getEntity() instanceof ServerPlayer serverPlayer) {
            KeepItemAfterDeath.onPlayerDeath(serverPlayer);
        }
    }

    // Last, and only when no listener cancelled the death: a revived player must not see the journey.
    @SubscribeEvent(priority = EventPriority.LOWEST)
    public static void announceJourney(LivingDeathEvent event) {

        if (event.getEntity() instanceof ServerPlayer serverPlayer) {
            JourneySignal.onPlayerDeath(serverPlayer);
        }
    }

    @SubscribeEvent
    public static void giveBackEternalItems(PlayerEvent.PlayerRespawnEvent event) {

        KeepItemAfterDeath.onPlayerRespawn(event.getEntity());
    }

    @SubscribeEvent(priority = EventPriority.LOWEST)
    public static void playRespawnEffects(PlayerEvent.PlayerRespawnEvent event) {

        RespawnEffectHandler.onPlayerRespawn(event.getEntity(), event.isEndConquered());
    }

    @SubscribeEvent
    public static void onPlayerWakeUp(PlayerWakeUpEvent event) {

        SleepHandler.onPlayerWakeUp(event.getEntity());
    }

    @SubscribeEvent
    public static void onItemPickup(EntityItemPickupEvent event) {

        if (EternalItemHandler.shouldBlockPickup(event.getEntity(), event.getItem().getItem())) {
            event.setCanceled(true);
        }
    }

    @SubscribeEvent
    public static void onPlayerTick(PlayerTickEvent event) {

        if (event.player instanceof ServerPlayer serverPlayer) {

            DisableDebugInfo.onPlayerTick(serverPlayer);

            if (event.phase == PlayerTickEvent.Phase.END) {
                EternalItemHandler.onPlayerTick(serverPlayer);
            }
        }
    }
}
