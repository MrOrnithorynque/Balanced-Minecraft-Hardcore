package github.mrornithorynque.bmh.selftest;

import java.util.Set;
import java.util.TreeSet;
import java.util.stream.Collectors;

/**
 * The bmh recipes are exactly the ones the mod means to ship. A stray recipe (the 2024 blast furnace totem, issue 5)
 * fails here. A new recipe must be added to EXPECTED.
 */
final class RecipeSetTest implements SelfTest.Test {

    private static final Set<String> EXPECTED = new TreeSet<>(Set.of(
            "bundle", "dormant_totem", "echo_heart", "eternal_axe_smithing", "eternal_bundle",
            "eternal_pickaxe_smithing", "eternal_sword_smithing", "navigator_amulet"));

    @Override
    public String name() {
        return "recipes";
    }

    @Override
    public void start(SelfTest.Ctx ctx) {
        Set<String> actual = ctx.server.getRecipeManager().getRecipeIds()
                .filter(id -> id.getNamespace().equals("bmh"))
                .map(id -> id.getPath())
                .collect(Collectors.toCollection(TreeSet::new));
        Set<String> extra = new TreeSet<>(actual);
        extra.removeAll(EXPECTED);
        Set<String> missing = new TreeSet<>(EXPECTED);
        missing.removeAll(actual);
        ctx.check("no unexpected recipe", extra.isEmpty(), "extra " + extra);
        ctx.check("every recipe loaded", missing.isEmpty(), "missing " + missing);
    }
}
