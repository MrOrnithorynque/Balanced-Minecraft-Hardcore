package github.mrornithorynque.bmh.utilities;

/**
 * The defaults and ranges of the mod config. The config of each loader and the config screen
 * (client/config/ConfigScreen) read them here.
 */
public class BMHConfigLimits {

    public static final int MIN_DISTANCE_FROM_BED = 5000;
    public static final int MAX_DISTANCE_FROM_BED = 200000;

    public static final int DEFAULT_MIN_DISTANCE_FROM_BED = 5000;
    public static final int DEFAULT_MAX_DISTANCE_FROM_BED = 60000;

    public static final int MIN_AMULET_POINTS = 1;
    public static final int MAX_AMULET_POINTS = 16;
    public static final int DEFAULT_AMULET_POINTS = 5;
}
