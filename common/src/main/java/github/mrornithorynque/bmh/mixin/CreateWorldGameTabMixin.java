package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import github.mrornithorynque.bmh.utilities.BMHGameRules;

import net.minecraft.client.gui.components.CycleButton;
import net.minecraft.client.gui.components.Tooltip;
import net.minecraft.client.gui.components.tabs.GridLayoutTab;
import net.minecraft.client.gui.screens.worldselection.CreateWorldScreen;
import net.minecraft.client.gui.screens.worldselection.WorldCreationUiState;
import net.minecraft.network.chat.Component;
import net.minecraft.world.level.gamerules.GameRule;
import net.minecraft.world.level.gamerules.GameRules;

/**
 * Adds a "Balanced Hardcore: ON/OFF" button to the Game tab of the Create New World screen,
 * under Allow Cheats. It flips every BMH game rule of the world being created.
 */
@Mixin(targets = "net.minecraft.client.gui.screens.worldselection.CreateWorldScreen$GameTab")
public abstract class CreateWorldGameTabMixin extends GridLayoutTab {

    private static final Component BALANCED_HARDCORE = Component.translatable("bmh.createWorld.balancedHardcore");
    private static final Component BALANCED_HARDCORE_INFO = Component.translatable("bmh.createWorld.balancedHardcore.info");

    // Never called: Mixin only needs it to compile against GridLayoutTab.
    private CreateWorldGameTabMixin(Component title) {
        super(title);
    }

    @Inject(method = "<init>", at = @At("TAIL"))
    private void bmh$addBalancedHardcoreToggle(CreateWorldScreen screen, CallbackInfo ci) {

        WorldCreationUiState uiState = screen.getUiState();

        CycleButton<Boolean> toggle = CycleButton.onOffBuilder(isBalancedHardcoreEnabled(uiState.getGameRules()))
                .withTooltip(value -> Tooltip.create(BALANCED_HARDCORE_INFO))
                .create(0, 0, 210, 20, BALANCED_HARDCORE,
                        (button, enabled) -> setBalancedHardcoreEnabled(uiState, enabled));

        // Keeps the button in sync when the rules are edited from the More tab, and greys it out
        // in Hardcore, where nobody respawns and BMH has nothing to do.
        uiState.addListener(state -> {
            toggle.setValue(isBalancedHardcoreEnabled(state.getGameRules()));
            toggle.active = !state.isHardcore() && !state.isDebug();
        });

        // The tab is a one column grid: the next free row is the number of children.
        int[] rowCount = { 0 };
        this.layout.visitChildren(child -> rowCount[0]++);
        this.layout.addChild(toggle, rowCount[0], 0);
    }

    private static boolean isBalancedHardcoreEnabled(GameRules gameRules) {

        return BMHGameRules.ALL_RULES.stream().anyMatch(gameRules::get);
    }

    private static void setBalancedHardcoreEnabled(WorldCreationUiState uiState, boolean enabled) {

        GameRules gameRules = uiState.getGameRules().copy(uiState.getSettings().dataConfiguration().enabledFeatures());

        for (GameRule<Boolean> rule : BMHGameRules.ALL_RULES) {
            gameRules.set(rule, enabled, null);
        }

        uiState.setGameRules(gameRules);
    }
}
