package github.mrornithorynque.bmh.items;

import java.util.List;

import github.mrornithorynque.bmh.client.RitualGlow;
import github.mrornithorynque.bmh.client.RitualParticles;
import github.mrornithorynque.bmh.handlers.MoonRitualHandler;
import github.mrornithorynque.bmh.init.ItemInit;
import github.mrornithorynque.bmh.platform.Services;

import net.minecraft.ChatFormatting;
import net.minecraft.network.chat.Component;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.InteractionResultHolder;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.TooltipFlag;
import net.minecraft.world.item.UseAnim;
import net.minecraft.world.level.Level;

/**
 * The Totem of Eternity after it saved a player. It cannot save anyone, but it keeps the eternity: it stays with the
 * player through death, and it makes the respawn farther like every Eternal Item. The moon ritual wakes it again
 * (handlers/MoonRitualHandler): hold use at night and look at the moon. An Echo Heart does not wake it.
 */
public class DormantTotemOfEternityItem extends Item implements IEternalItem {

    public DormantTotemOfEternityItem(Item.Properties properties) {
        super(properties);
    }

    @Override
    public void reduceDurability(ItemStack stack, int percentage) {
        // The totem has no durability.
    }

    /**
     * Only the server starts the ritual. The client follows the server's "using" flag (LocalPlayer.onSyncedDataUpdated):
     * a client that starts on its own guess shows a ritual that the server refused, for example when the look turns to
     * the moon in the same tick as the click. The click is always consumed: with a PASS, vanilla also uses the item in
     * the other hand in the same click. The server tells the player why the ritual does not start.
     */
    @Override
    public InteractionResultHolder<ItemStack> use(Level level, Player player, InteractionHand hand) {

        ItemStack stack = player.getItemInHand(hand);

        // A held use key sends the click again until the server's flag arrives: it must not start the ritual again.
        if (!(player instanceof ServerPlayer serverPlayer) || player.isUsingItem()) {
            return InteractionResultHolder.consume(stack);
        }

        String reason = MoonRitualHandler.blockedReason(serverPlayer);

        if (reason != null) {
            serverPlayer.displayClientMessage(Component.translatable(reason), true);
            return InteractionResultHolder.consume(stack);
        }

        MoonRitualHandler.start(serverPlayer);
        player.startUsingItem(hand);
        return InteractionResultHolder.consume(stack);
    }

    // No glint while it sleeps. It shimmers while a moon ritual charges it, on the client that draws it.
    @Override
    public boolean isFoil(ItemStack stack) {
        return Services.PLATFORM.isPhysicalClient() && RitualGlow.isCharging(stack);
    }

    @Override
    public int getUseDuration(ItemStack stack, LivingEntity entity) {
        return MoonRitualHandler.RITUAL_TICKS;
    }

    // Seen by others and in third person, the bow pose raises both arms along the look, so toward the moon. In first
    // person, client/RitualHandRenderer draws the totem raised in front of the eyes in place of the bow pose.
    @Override
    public UseAnim getUseAnimation(ItemStack stack) {
        return UseAnim.BOW;
    }

    @Override
    public void onUseTick(Level level, LivingEntity entity, ItemStack stack, int remaining) {

        if (entity instanceof ServerPlayer serverPlayer) {

            if (!MoonRitualHandler.tick(serverPlayer, remaining)) {
                serverPlayer.stopUsingItem();
            }

            return;
        }

        // The moonlight that drifts toward the totem, on every client that sees the ritual.
        if (level.isClientSide && entity instanceof Player player && Services.PLATFORM.isPhysicalClient()) {
            RitualParticles.spawn(level, player);
        }
    }

    /** The use ran to its end: the ritual succeeds. The client puts the new totem in the hand too. */
    @Override
    public ItemStack finishUsingItem(ItemStack stack, Level level, LivingEntity entity) {

        if (entity instanceof ServerPlayer serverPlayer) {
            return MoonRitualHandler.succeed(serverPlayer, this);
        }

        return new ItemStack(ItemInit.TOTEM_OF_ETERNITY.get());
    }

    /** The player let go of the use key before the end: the ritual fails. */
    @Override
    public void releaseUsing(ItemStack stack, Level level, LivingEntity entity, int remaining) {

        if (entity instanceof ServerPlayer serverPlayer) {
            MoonRitualHandler.release(serverPlayer);
        }
    }

    @Override
    public void appendHoverText(ItemStack stack, Item.TooltipContext context, List<Component> lines, TooltipFlag flag) {

        super.appendHoverText(stack, context, lines, flag);
        lines.add(Component.translatable("tooltip.bmh.eternity").withStyle(ChatFormatting.GRAY));
        lines.add(Component.translatable("tooltip.bmh.sleeping_totem", MoonRitualHandler.RITUAL_LEVELS)
                .withStyle(ChatFormatting.GRAY));
        EternalTooltips.addHeavy(lines);
    }
}
