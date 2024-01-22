package github.mrornithorynque.bmh.init;

import github.mrornithorynque.bmh.BalancedMcHardcoreMain;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.tags.TagKey;
import net.minecraft.tags.BlockTags;
import net.minecraft.world.level.block.Block;


public class TagInit {

    public static final TagKey<Block> LIFE_TOTEM = tag("life_totem");

    private static TagKey<Block> tag(String name) {

        return BlockTags.create(new ResourceLocation(BalancedMcHardcoreMain.MODID, name));
    }
}
