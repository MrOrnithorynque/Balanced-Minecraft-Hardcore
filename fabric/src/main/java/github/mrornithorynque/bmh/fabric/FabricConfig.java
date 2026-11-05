package github.mrornithorynque.bmh.fabric;

import java.io.IOException;
import java.io.Reader;
import java.io.Writer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Properties;

import org.slf4j.Logger;
import com.mojang.logging.LogUtils;

import github.mrornithorynque.bmh.utilities.BMHConfigLimits;

import net.fabricmc.loader.api.FabricLoader;

/**
 * config/bmh.properties: the same settings as Forge's config/bmh-common.toml, with the same
 * defaults and ranges. A missing file is written with the defaults, and a missing key takes its
 * default; an out of range value falls back to its default, as Forge does.
 */
public class FabricConfig {

    private static final Logger LOGGER = LogUtils.getLogger();

    private static FabricConfig instance;

    public final int minDistanceFromBed;
    public final int maxDistanceFromBed;
    public final int maxAmuletPoints;

    private FabricConfig(Properties properties) {

        this.minDistanceFromBed = readInRange(properties, "minDistanceFromBed", BMHConfigLimits.DEFAULT_MIN_DISTANCE_FROM_BED,
                BMHConfigLimits.MIN_DISTANCE_FROM_BED, BMHConfigLimits.MAX_DISTANCE_FROM_BED);
        this.maxDistanceFromBed = readInRange(properties, "maxDistanceFromBed", BMHConfigLimits.DEFAULT_MAX_DISTANCE_FROM_BED,
                BMHConfigLimits.MIN_DISTANCE_FROM_BED, BMHConfigLimits.MAX_DISTANCE_FROM_BED);
        this.maxAmuletPoints = readInRange(properties, "maxAmuletPoints", BMHConfigLimits.DEFAULT_AMULET_POINTS,
                BMHConfigLimits.MIN_AMULET_POINTS, BMHConfigLimits.MAX_AMULET_POINTS);
    }

    public static FabricConfig get() {

        if (instance == null) {
            instance = load(file());
        }

        return instance;
    }

    /** The config screen calls it on the client thread: writes the file, then the new values apply. */
    public static void save(int minDistanceFromBed, int maxDistanceFromBed, int maxAmuletPoints) {

        Properties properties = properties(minDistanceFromBed, maxDistanceFromBed, maxAmuletPoints);
        store(file(), properties);
        instance = new FabricConfig(properties);
    }

    private static Path file() {

        return FabricLoader.getInstance().getConfigDir().resolve("bmh.properties");
    }

    private static FabricConfig load(Path file) {

        Properties properties = new Properties();

        if (Files.exists(file)) {
            try (Reader reader = Files.newBufferedReader(file)) {
                properties.load(reader);
            } catch (IOException e) {
                LOGGER.warn("[BalancedMcHardcoreMain] Could not read " + file + ", using defaults", e);
            }
        } else {
            properties = properties(BMHConfigLimits.DEFAULT_MIN_DISTANCE_FROM_BED, BMHConfigLimits.DEFAULT_MAX_DISTANCE_FROM_BED,
                    BMHConfigLimits.DEFAULT_AMULET_POINTS);
            store(file, properties);
        }

        return new FabricConfig(properties);
    }

    private static Properties properties(int minDistanceFromBed, int maxDistanceFromBed, int maxAmuletPoints) {

        Properties properties = new Properties();
        properties.setProperty("minDistanceFromBed", String.valueOf(minDistanceFromBed));
        properties.setProperty("maxDistanceFromBed", String.valueOf(maxDistanceFromBed));
        properties.setProperty("maxAmuletPoints", String.valueOf(maxAmuletPoints));

        return properties;
    }

    private static void store(Path file, Properties properties) {

        try (Writer writer = Files.newBufferedWriter(file)) {
            properties.store(writer, "Distance from bed to respawn, " + BMHConfigLimits.MIN_DISTANCE_FROM_BED + " to "
                    + BMHConfigLimits.MAX_DISTANCE_FROM_BED + ". Most points on the Navigator's Amulet, "
                    + BMHConfigLimits.MIN_AMULET_POINTS + " to " + BMHConfigLimits.MAX_AMULET_POINTS + ".");
        } catch (IOException e) {
            LOGGER.warn("[BalancedMcHardcoreMain] Could not write " + file, e);
        }
    }

    private static int readInRange(Properties properties, String key, int defaultValue, int min, int max) {

        try {
            int value = Integer.parseInt(properties.getProperty(key, String.valueOf(defaultValue)).trim());

            if (value >= min && value <= max) {
                return value;
            }
        } catch (NumberFormatException e) {
            // falls through to the default
        }

        LOGGER.warn("[BalancedMcHardcoreMain] " + key + " must be a number from " + min + " to " + max + ", using " + defaultValue);

        return defaultValue;
    }
}
