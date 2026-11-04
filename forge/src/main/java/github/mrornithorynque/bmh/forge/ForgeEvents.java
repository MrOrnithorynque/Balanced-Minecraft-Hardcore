package github.mrornithorynque.bmh.forge;

import github.mrornithorynque.bmh.handlers.DisableDebugInfo;
import github.mrornithorynque.bmh.handlers.EternalItemHandler;
import github.mrornithorynque.bmh.handlers.JourneySignal;
import github.mrornithorynque.bmh.handlers.KeepItemAfterDeath;
import github.mrornithorynque.bmh.handlers.RespawnEffectHandler;
import github.mrornithorynque.bmh.handlers.RespawnPositionHandler;
import github.mrornithorynque.bmh.handlers.SleepHandler;
import github.mrornithorynque.bmh.items.TotemOfEternityItem;

import net.minecraft.server.level.ServerPlayer;

import net.minecraftforge.event.TickEvent.PlayerTickEvent;
import net.minecraftforge.event.entity.living.LivingDeathEvent;
import net.minecraftforge.event.entity.player.EntityItemPickupEvent;
import net.minecraftforge.event.entity.player.PlayerEvent;
import net.minecraftforge.event.entity.player.PlayerWakeUpEvent;
import net.minecraftforge.eventbus.api.EventPriority;
import net.minecraftforge.eventbus.api.SubscribeEvent;

/**
 * Forge side of the common handlers. The priorities give the order the handlers rely on: the
 * respawn distance is measured before the Eternal Items leave the inventory, and the respawn
 * effects play after the kept items come back. The distant respawn position is not an event:
 * mixin/F_PlayerListRespawnMixin puts it into vanilla's respawn.
 */
public class ForgeEvents {

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

        if (TotemOfEternityItem.shouldBlockPickup(event.getEntity(), event.getItem().getItem())) {
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
