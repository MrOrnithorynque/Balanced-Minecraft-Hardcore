package github.mrornithorynque.bmh.mixin;

import java.nio.file.Path;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import github.mrornithorynque.bmh.utilities.BMHGameRules;

import net.minecraft.nbt.CompoundTag;
import net.minecraft.util.datafix.DataFixTypes;
import net.minecraft.world.level.storage.SavedDataStorage;

/**
 * A world from before 26.1 keeps its BMH game rules under their old names, which 26.1 cannot read: the game rules file
 * logs an error and the BMH rules go back to their defaults. This gives them their new ids, with their values, as soon
 * as the file is read and fixed, before the game parses it.
 */
@Mixin(SavedDataStorage.class)
public abstract class N_SavedDataGameRulesMixin {

    @Inject(method = "readTagFromDisk", at = @At("RETURN"))
    private void bmh$migrateGameRules(Path dataFile, DataFixTypes type, int newVersion, CallbackInfoReturnable<CompoundTag> cir) {

        BMHGameRules.migrateLegacyNames(cir.getReturnValue());
    }
}
