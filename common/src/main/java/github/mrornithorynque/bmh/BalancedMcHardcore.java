package github.mrornithorynque.bmh;

import github.mrornithorynque.bmh.init.CreativeTabInit;
import github.mrornithorynque.bmh.init.ItemInit;
import github.mrornithorynque.bmh.init.SoundInit;
import github.mrornithorynque.bmh.utilities.BMHGameRules;

public class BalancedMcHardcore {

    public static final String MODID = "bmh";

    /** Called once by each loader while the mod is constructed: loads every registry and game rule class. */
    public static void init() {

        ItemInit.init();
        SoundInit.init();
        CreativeTabInit.init();
        BMHGameRules.init();
    }
}
