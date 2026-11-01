package github.mrornithorynque.bmh.fabric;

import java.io.IOException;
import java.io.Reader;
import java.io.Writer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Properties;

import org.slf4j.Logger;
import com.mojang.logging.LogUtils;

import net.fabricmc.loader.api.FabricLoader;

/**
 * config/bmh.properties: the same two settings as Forge's config/bmh-common.toml, with the same
 * defaults and range. A missing file is written with the defaults; an out of range value falls
 * back to its default, as Forge does.
 */
public class FabricConfig {

    private static final Logger LOGGER = LogUtils.getLogger();

    private static final int MIN_DISTANCE_FROM_BED = 5000;
    private static final int MAX_DISTANCE_FROM_BED = 200000;

    private static FabricConfig instance;

    public final int minDistanceFromBed;
    public final int maxDistanceFromBed;

    private FabricConfig(Properties properties) {

        this.minDistanceFromBed = readInRange(properties, "minDistanceFromBed", 5000);
        this.maxDistanceFromBed = readInRange(properties, "maxDistanceFromBed", 60000);
    }

    public static FabricConfig get() {

        if (instance == null) {
            instance = load(FabricLoader.getInstance().getConfigDir().resolve("bmh.properties"));
        }

        return instance;
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
            properties.setProperty("minDistanceFromBed", "5000");
            properties.setProperty("maxDistanceFromBed", "60000");

            try (Writer writer = Files.newBufferedWriter(file)) {
                properties.store(writer, "Distance from bed to respawn, " + MIN_DISTANCE_FROM_BED + " to " + MAX_DISTANCE_FROM_BED);
            } catch (IOException e) {
                LOGGER.warn("[BalancedMcHardcoreMain] Could not write " + file, e);
            }
        }

        return new FabricConfig(properties);
    }

    private static int readInRange(Properties properties, String key, int defaultValue) {

        try {
            int value = Integer.parseInt(properties.getProperty(key, String.valueOf(defaultValue)).trim());

            if (value >= MIN_DISTANCE_FROM_BED && value <= MAX_DISTANCE_FROM_BED) {
                return value;
            }
        } catch (NumberFormatException e) {
            // falls through to the default
        }

        LOGGER.warn("[BalancedMcHardcoreMain] " + key + " must be a number from " + MIN_DISTANCE_FROM_BED + " to "
                + MAX_DISTANCE_FROM_BED + ", using " + defaultValue);

        return defaultValue;
    }
}
