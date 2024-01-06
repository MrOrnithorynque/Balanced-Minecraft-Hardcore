package github.mrornithorynque.utilities;

public enum HexColor {

    BLACK(0x000000),
    WHITE(0xFFFFFF),
    RED(0xFF0000),
    GREEN(0x00FF00),
    BLUE(0x0000FF),
    YELLOW(0xFFFF00),
    CYAN(0x00FFFF),
    MAGENTA(0xFF00FF);

    private final int hexValue;

    HexColor(int hexValue) {

        this.hexValue = hexValue;
    }

    public int getValue() {

        return hexValue;
    }

    @Override
    public String toString() {

        return String.format("#%06X", hexValue);
    }
}
