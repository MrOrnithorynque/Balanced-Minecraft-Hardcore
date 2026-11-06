package github.mrornithorynque.bmh.platform;

import java.util.List;
import java.util.function.Supplier;

import net.minecraft.core.Registry;
import net.minecraft.resources.ResourceKey;
import net.minecraft.world.item.CreativeModeTab;
import net.minecraft.world.level.ItemLike;

/**
 * Everything common code needs from the mod loader. One implementation per loader, found by
 * {@link Services} through META-INF/services.
 */
public interface IPlatformHelper {

    /**
     * Registers an object under the bmh namespace. The returned supplier is only valid once the
     * loader has run its registration (Forge registers later, Fabric right away).
     */
    <T, I extends T> Supplier<I> register(ResourceKey<? extends Registry<T>> registry, String name, Supplier<I> factory);

    /**
     * A creative tab builder that already lists these items. Vanilla 26.1 keeps CreativeModeTab.Output protected, and
     * each loader makes it public: so only the loader code can fill the tab.
     */
    CreativeModeTab.Builder creativeTabBuilder(List<Supplier<? extends ItemLike>> items);

    /** True on the game client (singleplayer and LAN host included), false on a dedicated server. */
    boolean isPhysicalClient();

    int minDistanceFromBed();

    int maxDistanceFromBed();

    /** The most points a player can keep on the Navigator's Amulet. */
    int maxAmuletPoints();

    /** Writes the mod config file. The values are in range: the config screen checks them first. */
    void saveConfig(int minDistanceFromBed, int maxDistanceFromBed, int maxAmuletPoints);
}
