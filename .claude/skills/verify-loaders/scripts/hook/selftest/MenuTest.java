package github.mrornithorynque.bmh.selftest;

import github.mrornithorynque.bmh.init.ItemInit;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.inventory.AnvilMenu;
import net.minecraft.world.inventory.ClickType;
import net.minecraft.world.inventory.ContainerLevelAccess;
import net.minecraft.world.inventory.SmithingMenu;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Items;
import net.minecraft.world.level.block.Blocks;

/**
 * The menus, with the result taken by a shift-click like a player. The smithing table makes an Eternal Sword from a
 * netherite sword and keeps its Sharpness V. A gold block repairs an Eternal Pickaxe at the anvil and uses the left
 * item up, so no copy stays behind.
 */
final class MenuTest implements SelfTest.Test {

    @Override
    public String name() {
        return "menus";
    }

    @Override
    public void start(SelfTest.Ctx ctx) {
        smithing(ctx);
        anvil(ctx);
    }

    private static void smithing(SelfTest.Ctx ctx) {
        ServerPlayer player = ctx.player("bmh-smith", 0, 0);
        ctx.level.setBlockAndUpdate(ctx.origin.offset(0, 1, 2), Blocks.SMITHING_TABLE.defaultBlockState());
        SmithingMenu menu = new SmithingMenu(7, player.getInventory(), ContainerLevelAccess.create(ctx.level, ctx.origin.offset(0, 1, 2)));
        ItemStack base = new ItemStack(Items.NETHERITE_SWORD);
        MockPlayers.enchantSharpness(ctx.level, base, 5);
        menu.getSlot(0).set(new ItemStack(ItemInit.TOTEM_OF_ETERNITY.get()));
        menu.getSlot(1).set(base);
        menu.getSlot(2).set(new ItemStack(Items.ECHO_SHARD));
        ItemStack result = menu.getSlot(3).getItem().copy();
        ctx.check("smithing: the result is the Eternal Sword", result.is(ItemInit.ETERNAL_SWORD.get()), "result " + result);
        ctx.check("smithing: the result keeps Sharpness V", MockPlayers.sharpness(ctx.level, result) == 5,
                "sharpness " + MockPlayers.sharpness(ctx.level, result));
        menu.clicked(3, 0, ClickType.QUICK_MOVE, player);
        ctx.check("smithing: taking it uses the three inputs up", !menu.getSlot(0).hasItem() && !menu.getSlot(1).hasItem()
                && !menu.getSlot(2).hasItem() && player.getInventory().countItem(ItemInit.ETERNAL_SWORD.get()) == 1,
                "template " + menu.getSlot(0).getItem() + ", base " + menu.getSlot(1).getItem() + ", addition "
                        + menu.getSlot(2).getItem() + ", swords " + player.getInventory().countItem(ItemInit.ETERNAL_SWORD.get()));
        menu.removed(player);
    }

    private static void anvil(SelfTest.Ctx ctx) {
        ServerPlayer player = ctx.player("bmh-anvil", 8, 0);
        player.giveExperienceLevels(30);
        ctx.level.setBlockAndUpdate(ctx.origin.offset(8, 1, 2), Blocks.ANVIL.defaultBlockState());
        AnvilMenu menu = new AnvilMenu(8, player.getInventory(), ContainerLevelAccess.create(ctx.level, ctx.origin.offset(8, 1, 2)));
        ItemStack pickaxe = new ItemStack(ItemInit.ETERNAL_PICKAXE.get());
        int damage = pickaxe.getMaxDamage() / 2;
        pickaxe.setDamageValue(damage);
        menu.getSlot(0).set(pickaxe);
        menu.getSlot(1).set(new ItemStack(Items.GOLD_BLOCK, 10));
        ItemStack result = menu.getSlot(2).getItem().copy();
        ctx.check("anvil: a gold block repairs the Eternal Pickaxe", result.is(ItemInit.ETERNAL_PICKAXE.get()) && result.getDamageValue() < damage,
                "damage " + damage + " -> " + result.getDamageValue() + ", cost " + menu.getCost());
        menu.clicked(2, 0, ClickType.QUICK_MOVE, player);
        int gold = menu.getSlot(1).getItem().getCount();
        ctx.check("anvil: taking it empties the left slot, one pickaxe in all", !menu.getSlot(0).hasItem()
                && player.getInventory().countItem(ItemInit.ETERNAL_PICKAXE.get()) == 1 && gold < 10,
                "left " + menu.getSlot(0).getItem() + ", pickaxes " + player.getInventory().countItem(ItemInit.ETERNAL_PICKAXE.get())
                        + ", gold left " + gold);
        menu.removed(player);
    }
}
