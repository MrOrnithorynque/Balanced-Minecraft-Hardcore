package github.mrornithorynque.bmh.client;

import net.minecraft.client.renderer.item.ClampedItemPropertyFunction;
import net.minecraft.client.renderer.item.CompassItemPropertyFunction;
import net.minecraft.core.BlockPos;
import net.minecraft.core.GlobalPos;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.world.level.Level;

/**
 * The needle of the Navigator's Amulet: vanilla's compass machinery (its wobble included) aimed at a
 * point far due north of whoever holds it. Client class: ClientItemProperties registers it (every loader's
 * client setup calls that), a dedicated server never loads it.
 */
public class AmuletCompass {

    /** The item property the model's overrides switch on, the same name as vanilla's compass. */
    public static final ResourceLocation ANGLE = new ResourceLocation("angle");

    // Far enough for the angle to stay exact however far from the target's x the holder stands.
    private static final double NORTH_DISTANCE = 1.0E6;

    /** Overworld only: in the other dimensions there is no target and the needle spins, as vanilla's does. */
    public static ClampedItemPropertyFunction angle() {

        return new CompassItemPropertyFunction((level, stack, entity) -> level.dimension() == Level.OVERWORLD
                ? GlobalPos.of(Level.OVERWORLD, BlockPos.containing(entity.getX(), entity.getY(), entity.getZ() - NORTH_DISTANCE))
                : null);
    }
}
