package github.mrornithorynque.bmh.init;

import javax.annotation.Nonnull;

import github.mrornithorynque.bmh.BalancedMcHardcoreMain;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.tags.TagKey;
import net.minecraft.tags.BlockTags;
import net.minecraft.world.level.block.Block;

public class TagInit {

    public static final TagKey<Block> TOTEM_OF_ETERNITY = tag("totem_of_eternity");

    private static TagKey<Block> tag(@Nonnull String name) {

        return BlockTags.create(new ResourceLocation(BalancedMcHardcoreMain.MODID, name));
    }
}
