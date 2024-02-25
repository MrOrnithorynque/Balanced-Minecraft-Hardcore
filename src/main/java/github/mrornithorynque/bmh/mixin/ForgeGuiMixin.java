package github.mrornithorynque.bmh.mixin;

// import net.minecraft.client.MinecraftClient;
// import net.minecraft.client.gui.DrawContext;
import net.minecraftforge.client.gui.overlay.ForgeGui;
import org.jetbrains.annotations.NotNull;
import org.objectweb.asm.Opcodes;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;


@Mixin(ForgeGui.class)
public abstract class ForgeGuiMixin {

//   @SuppressWarnings("checkstyle:MethodName")
//   @Shadow
//   public abstract MinecraftClient getMinecraft();

//   @Inject(remap = false, method = "renderHUDText", at = @At(value = "INVOKE", opcode = Opcodes.PUTFIELD, target =
//   "Lnet/minecraftforge/client/gui/overlay/ForgeGui$OverlayAccess;update()V"), cancellable = true)
//   public void customDebugMenu(final int width, final int height, final DrawContext guiGraphics, final @NotNull CallbackInfo ci) {
//     this.getMinecraft().getDebugHud().render(guiGraphics);

//     this.getMinecraft().getProfiler().pop();

//     ci.cancel();
//   }
}