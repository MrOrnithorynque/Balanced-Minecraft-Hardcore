package github.mrornithorynque.bmh.init;

import github.mrornithorynque.bmh.BalancedMcHardcoreMain;

import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.BlockBehaviour;
import net.minecraft.world.level.block.state.properties.NoteBlockInstrument;
import net.minecraft.world.level.material.MapColor;

import net.minecraftforge.registries.DeferredRegister;
import net.minecraftforge.registries.ForgeRegistries;
import net.minecraftforge.registries.RegistryObject;

public class BlockInit {

    public static final DeferredRegister<Block> BLOCKS = DeferredRegister.create(ForgeRegistries.BLOCKS,
            BalancedMcHardcoreMain.MODID);

    public static final RegistryObject<Block> TEST_BLOCK = BLOCKS.register("test_block",
            () -> new Block(BlockBehaviour.Properties.of()
                    .mapColor(MapColor.COLOR_BLACK)
                    .strength(5.0f, 17.0f)
                    .instrument(NoteBlockInstrument.BANJO)
                    .requiresCorrectToolForDrops()
                    .lightLevel(state -> 10)));
}
