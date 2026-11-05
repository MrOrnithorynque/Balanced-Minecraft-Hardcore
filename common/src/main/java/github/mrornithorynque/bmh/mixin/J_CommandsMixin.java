package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import github.mrornithorynque.bmh.handlers.AmuletPointCommand;

import net.minecraft.commands.CommandBuildContext;
import net.minecraft.commands.Commands;

/**
 * Registers BMH's commands (AmuletPointCommand) when the server builds its command tree. The three loaders
 * fire their own command events from the same constructor; one common mixin serves them all. The
 * constructor is the same from 1.20 to 1.21.1.
 */
@Mixin(Commands.class)
public abstract class J_CommandsMixin {

    @Inject(method = "<init>", at = @At("TAIL"))
    private void bmh$registerCommands(Commands.CommandSelection selection, CommandBuildContext context, CallbackInfo ci) {

        AmuletPointCommand.register(((Commands) (Object) this).getDispatcher());
    }
}
