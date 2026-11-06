package github.mrornithorynque.bmh.client;

import com.mojang.serialization.Codec;
import com.mojang.serialization.MapCodec;

import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.client.renderer.item.properties.numeric.NeedleDirectionHelper;
import net.minecraft.client.renderer.item.properties.numeric.RangeSelectItemModelProperty;
import net.minecraft.resources.Identifier;
import net.minecraft.util.Mth;
import net.minecraft.util.RandomSource;
import net.minecraft.world.entity.ItemOwner;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.Level;
import net.minecraft.world.phys.Vec3;

/**
 * The needle of the Navigator's Amulet, the numeric item model property "bmh:north". It is vanilla's compass
 * (CompassAngleState, its wobble included) aimed at a point far due north of whoever holds it. In the other
 * dimensions there is no target and the needle spins, as vanilla's does. The item model
 * (assets/bmh/items/navigator_amulet.json) reads it with a range_dispatch of scale 32.
 * mixin/M_RangeSelectPropertiesMixin registers it on every loader. Client class: a dedicated server never loads it.
 */
public class AmuletCompass extends NeedleDirectionHelper implements RangeSelectItemModelProperty {

    public static final Identifier ID = Identifier.fromNamespaceAndPath("bmh", "north");

    // Each model that reads the property gets its own instance, with its own wobble, as vanilla's compass does.
    public static final MapCodec<AmuletCompass> MAP_CODEC = Codec.BOOL.optionalFieldOf("wobble", true)
            .xmap(AmuletCompass::new, AmuletCompass::wobble);

    // Far enough for the angle to stay exact however far from the target's x the holder stands.
    private static final double NORTH_DISTANCE = 1.0E6;

    private final NeedleDirectionHelper.Wobbler wobbler;
    private final NeedleDirectionHelper.Wobbler noTargetWobbler;
    private final RandomSource random = RandomSource.create();

    public AmuletCompass(boolean wobble) {

        super(wobble);
        this.wobbler = this.newWobbler(0.8F);
        this.noTargetWobbler = this.newWobbler(0.8F);
    }

    @Override
    public MapCodec<AmuletCompass> type() {

        return MAP_CODEC;
    }

    // The same steps as vanilla's CompassAngleState.calculate, with the target due north of the owner.
    @Override
    protected float calculate(ItemStack itemStack, ClientLevel level, int seed, ItemOwner owner) {

        long gameTime = level.getGameTime();

        if (level.dimension() != Level.OVERWORLD) {
            return this.spin(seed, gameTime);
        }

        Vec3 position = owner.position();
        // The target is the center of the block NORTH_DISTANCE north of the owner, as BlockPos.containing gave in 1.21.1.
        double targetX = Math.floor(position.x()) + 0.5;
        double targetZ = Math.floor(position.z() - NORTH_DISTANCE) + 0.5;
        float angleToTarget = (float) (Math.atan2(targetZ - position.z(), targetX - position.x()) / (float) (Math.PI * 2));
        float ownerYRotation = Mth.positiveModulo(owner.getVisualRotationYInDegrees() / 360.0F, 1.0F);
        float targetRotation;

        if (owner.asLivingEntity() instanceof Player player && player.isLocalPlayer() && player.level().tickRateManager().runsNormally()) {

            if (this.wobbler.shouldUpdate(gameTime)) {
                this.wobbler.update(gameTime, 0.5F - (ownerYRotation - 0.25F));
            }

            targetRotation = angleToTarget + this.wobbler.rotation();
        } else {
            targetRotation = 0.5F - (ownerYRotation - 0.25F - angleToTarget);
        }

        return Mth.positiveModulo(targetRotation, 1.0F);
    }

    private float spin(int seed, long gameTime) {

        if (this.noTargetWobbler.shouldUpdate(gameTime)) {
            this.noTargetWobbler.update(gameTime, this.random.nextFloat());
        }

        float targetRotation = this.noTargetWobbler.rotation() + seed * 1327217883 / 2.1474836E9F;

        return Mth.positiveModulo(targetRotation, 1.0F);
    }
}
