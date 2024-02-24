package github.mrornithorynque.bmh.items;

import github.mrornithorynque.utilities.HexColor;
import github.mrornithorynque.utilities.TextDrawer;

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

            TextDrawer.getInstance().drawString(
                    "Current coordinates: " +
                            player.blockPosition().getX() +
                            ", " +
                            player.blockPosition().getY() +
                            ", " +
                            player.blockPosition().getZ() +
                            ".",
                    TextDrawer.ScreenPosition.CENTER,
                    HexColor.WHITE.getValue(),
                    5000);
        }

        return InteractionResult.sidedSuccess(player.level().isClientSide);
    }
}
