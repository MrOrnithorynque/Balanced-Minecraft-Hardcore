package github.mrornithorynque.bmh.item;

import net.minecraft.world.entity.player.Player;
import net.minecraft.world.InteractionResult;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.context.UseOnContext;

public class NavigatorAmuletItem extends Item {

    public NavigatorAmuletItem(Item.Properties properties) {

        super(properties);
    }

    @Override
    public InteractionResult useOn(UseOnContext context) {

        Player player = context.getPlayer();
        ItemStack itemStack = context.getItemInHand();

        if (!player.level().isClientSide) {
            itemStack.hurtAndBreak(1, player, (p) -> p.broadcastBreakEvent(context.getHand()));

            // Your custom logic when the amulet is used, if any
        }

        return InteractionResult.sidedSuccess(player.level().isClientSide);
    }
}
