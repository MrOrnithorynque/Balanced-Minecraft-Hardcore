package github.mrornithorynque.bmh.mixin;

import org.jetbrains.annotations.NotNull;
import org.objectweb.asm.Opcodes;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import com.mojang.authlib.minecraft.client.MinecraftClient;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphics;

import net.minecraftforge.client.gui.overlay.ForgeGui;

/**
 * Forge InGame GUI Mixin.
 */
@Mixin(ForgeGui.class)
public abstract class ForgeGuiMixin {

    @SuppressWarnings("checkstyle:MethodName")
    @Shadow
    public abstract MinecraftClient getMinecraft();

    @Inject(remap = false, method = "renderHUDText", at = @At(value = "INVOKE", opcode = Opcodes.PUTFIELD,
    target ="Lnet/minecraftforge/client/gui/overlay/ForgeGui$OverlayAccess;update()V"), cancellable = true)
    public void customDebugMenu(final int width, final int height, final GuiGraphics guiGraphics, final @NotNull CallbackInfo ci) {
    }
}