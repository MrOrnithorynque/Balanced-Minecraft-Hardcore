package github.mrornithorynque.bmh.init;

import github.mrornithorynque.bmh.BalancedMcHardcore;

import net.minecraft.core.registries.Registries;
import net.minecraft.resources.Identifier;
import net.minecraft.tags.BlockTags;
import net.minecraft.tags.TagKey;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ToolMaterial;

public class BMHModTierInit {

    // The items that repair an Eternal tool at an anvil: the gold block only (data/bmh/tags/item/eternal_tool_materials.json).
    public static final TagKey<Item> ETERNAL_TOOL_MATERIALS = TagKey.create(Registries.ITEM,
            Identifier.fromNamespaceAndPath(BalancedMcHardcore.MODID, "eternal_tool_materials"));

    // 33000 uses, mining speed 15, no attack damage bonus, enchantability 80. It mines everything netherite mines
    // (the old level 5, one above netherite, meant no more).
    public static final ToolMaterial ETERNAL = new ToolMaterial(
            BlockTags.INCORRECT_FOR_NETHERITE_TOOL, 33000, 15.0F, 0.0F, 80, ETERNAL_TOOL_MATERIALS);
}
