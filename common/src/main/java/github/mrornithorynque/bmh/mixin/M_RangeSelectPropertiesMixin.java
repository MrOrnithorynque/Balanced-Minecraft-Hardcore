package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import com.mojang.serialization.MapCodec;

import github.mrornithorynque.bmh.client.AmuletCompass;

import net.minecraft.client.renderer.item.properties.numeric.RangeSelectItemModelProperties;
import net.minecraft.client.renderer.item.properties.numeric.RangeSelectItemModelProperty;
import net.minecraft.resources.Identifier;
import net.minecraft.util.ExtraCodecs;

/**
 * Registers the numeric item model property "bmh:north" (client/AmuletCompass), the needle of the Navigator's Amulet.
 * Vanilla fills this id map once, in ClientBootstrap, before any item model loads. NeoForge posts its own
 * registration event at the end of the same method, so one mixin covers the three loaders. Client list.
 */
@Mixin(RangeSelectItemModelProperties.class)
public abstract class M_RangeSelectPropertiesMixin {

    @Shadow
    @Final
    private static ExtraCodecs.LateBoundIdMapper<Identifier, MapCodec<? extends RangeSelectItemModelProperty>> ID_MAPPER;

    @Inject(method = "bootstrap", at = @At("TAIL"))
    private static void bmh$registerAmuletNeedle(CallbackInfo ci) {

        ID_MAPPER.put(AmuletCompass.ID, AmuletCompass.MAP_CODEC);
    }
}
