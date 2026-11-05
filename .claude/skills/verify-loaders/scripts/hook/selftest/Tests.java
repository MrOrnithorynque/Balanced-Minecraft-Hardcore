package github.mrornithorynque.bmh.selftest;

import java.util.List;

/** THROWAWAY (see selftest.sh): every feature test, in the order SelfTest runs them. GateTest first, always. */
final class Tests {

    static final List<SelfTest.Test> ALL = List.of(
            new GateTest(),
            new RecipeSetTest(),
            new OneEternalPerTypeTest(),
            new DeathKeepTest(),
            new RespawnTest(),
            new SleepTest(),
            new CoordinatesTest(),
            new AxeSmashTest(),
            new MenuTest());

    private Tests() {
    }
}
