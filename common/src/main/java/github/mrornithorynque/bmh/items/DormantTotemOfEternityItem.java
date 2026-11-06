package github.mrornithorynque.bmh.items;

import java.util.List;

import javax.annotation.Nullable;

import github.mrornithorynque.bmh.handlers.MoonRitualHandler;
import github.mrornithorynque.bmh.init.ItemInit;

import net.minecraft.ChatFormatting;
import net.minecraft.core.particles.ParticleTypes;
import net.minecraft.network.chat.Component;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.util.RandomSource;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.InteractionResultHolder;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.TooltipFlag;
import net.minecraft.world.item.UseAnim;
import net.minecraft.world.level.Level;
import net.minecraft.world.phys.Vec3;

/**
 * The Totem of Eternity after it saved a player. It cannot save anyone, but it keeps the eternity: it stays with the
 * player through death, and it makes the respawn farther like every Eternal Item. The moon ritual wakes it again
 * (handlers/MoonRitualHandler): hold use at night and look at the moon. An Echo Heart does not wake it.
 */
public class DormantTotemOfEternityItem extends Item implements IEternalItem {

    // The moonlight particles start this far from the eyes, toward the moon.
    private static final double PARTICLE_START = 4.0;

    public DormantTotemOfEternityItem(Item.Properties properties) {
        super(properties);
    }

    @Override
    public void reduceDurability(ItemStack stack, int percentage) {
        // The totem has no durability.
    }

    /**
     * Both sides run the same test. The click is always consumed: with a PASS, vanilla also uses the item in the
     * other hand in the same click. The server tells the player why the ritual does not start.
     */
    @Override
    public InteractionResultHolder<ItemStack> use(Level level, Player player, InteractionHand hand) {

        ItemStack stack = player.getItemInHand(hand);
        String reason = MoonRitualHandler.blockedReason(player);

        if (reason != null) {

            if (player instanceof ServerPlayer serverPlayer) {
                serverPlayer.displayClientMessage(Component.translatable(reason), true);
            }

            return InteractionResultHolder.consume(stack);
        }

        if (player instanceof ServerPlayer serverPlayer) {
            MoonRitualHandler.start(serverPlayer);
        }

        player.startUsingItem(hand);
        return InteractionResultHolder.consume(stack);
    }

    @Override
    public int getUseDuration(ItemStack stack) {
        return MoonRitualHandler.RITUAL_TICKS;
    }

    // The arm raises the totem toward the sky.
    @Override
    public UseAnim getUseAnimation(ItemStack stack) {
        return UseAnim.SPEAR;
    }

    @Override
    public void onUseTick(Level level, LivingEntity entity, ItemStack stack, int remaining) {

        if (entity instanceof ServerPlayer serverPlayer) {

            if (!MoonRitualHandler.tick(serverPlayer, remaining)) {
                serverPlayer.stopUsingItem();
            }

            return;
        }

        if (level.isClientSide && entity instanceof Player player) {
            spawnMoonlight(level, player);
        }
    }

    /**
     * End rod particles drift from the moon toward the totem. They are full bright and white like moonlight, so they
     * read at night, and their friction slows them down as they come near the totem.
     */
    private static void spawnMoonlight(Level level, Player player) {

        RandomSource random = player.getRandom();
        Vec3 hand = MoonRitualHandler.handPosition(player);
        Vec3 moon = MoonRitualHandler.moonDirection(level);
        Vec3 start = player.getEyePosition().add(moon.scale(PARTICLE_START))
                .add(random.nextGaussian() * 0.6, random.nextGaussian() * 0.6, random.nextGaussian() * 0.6);
        // A particle with friction 0.91 goes about 11 times its first speed in all.
        Vec3 speed = hand.subtract(start).scale(1.0 / 11.0);

        level.addParticle(ParticleTypes.END_ROD, start.x, start.y, start.z, speed.x, speed.y, speed.z);
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
    public void appendHoverText(ItemStack stack, @Nullable Level level, List<Component> lines, TooltipFlag flag) {

        super.appendHoverText(stack, level, lines, flag);
        lines.add(Component.translatable("tooltip.bmh.eternity").withStyle(ChatFormatting.GRAY));
        lines.add(Component.translatable("tooltip.bmh.sleeping_totem", MoonRitualHandler.RITUAL_LEVELS)
                .withStyle(ChatFormatting.GRAY));
        EternalTooltips.addHeavy(lines);
    }
}
