package github.mrornithorynque.bmh.client.amulet;

import java.util.ArrayList;
import java.util.List;

import github.mrornithorynque.bmh.handlers.AmuletPointSignal;

import net.minecraft.client.Minecraft;
import net.minecraft.core.BlockPos;
import net.minecraft.network.chat.Component;
import net.minecraft.network.chat.contents.TranslatableContents;

/**
 * Reads the server's bmh.point.open title (handlers/AmuletPointSignal) and opens the naming screen.
 * mixin.F_GuiTitleMixin calls it for every title. Client only.
 */
public final class PointSignal {

    private PointSignal() {
    }

    /** True when the title was the signal: it is used here, and vanilla must not show it. */
    public static boolean onTitle(Component title) {

        if (!(title.getContents() instanceof TranslatableContents contents)) {
            return false;
        }

        if (AmuletPointSignal.LIST.equals(contents.getKey())) {
            onList(contents.getArgs());
            return true;
        }

        if (!AmuletPointSignal.OPEN.equals(contents.getKey())) {
            return false;
        }

        Object[] args = contents.getArgs();
        Minecraft minecraft = Minecraft.getInstance();

        // A held click can send the signal again while the screen is open: keep the screen the player types in.
        if (args.length < 7 || minecraft.screen instanceof PointScreen) {
            return true;
        }

        try {
            BlockPos pos = new BlockPos(Integer.parseInt(text(args[0])), Integer.parseInt(text(args[1])), Integer.parseInt(text(args[2])));
            String name = text(args[3]);
            int used = Integer.parseInt(text(args[4]));
            int max = Integer.parseInt(text(args[5]));
            minecraft.setScreen(new PointScreen(pos, name.isEmpty() ? null : name, used, max, "bed".equals(text(args[6]))));
        } catch (NumberFormatException e) {
            // A malformed signal opens nothing.
        }

        return true;
    }

    // The list: the cap, the count, then name, x, y, z of each point. An open list screen takes the new list in place
    // (the server sends it again after each removal); a naming screen stays as it is.
    private static void onList(Object[] args) {

        Minecraft minecraft = Minecraft.getInstance();

        if (minecraft.screen instanceof PointScreen) {
            return;
        }

        try {
            int max = Integer.parseInt(text(args[0]));
            int count = Integer.parseInt(text(args[1]));
            List<PointListScreen.Entry> points = new ArrayList<>();

            for (int i = 0; i < count; i++) {
                int at = 2 + i * 4;
                points.add(new PointListScreen.Entry(text(args[at]),
                        new BlockPos(Integer.parseInt(text(args[at + 1])), Integer.parseInt(text(args[at + 2])), Integer.parseInt(text(args[at + 3])))));
            }

            if (minecraft.screen instanceof PointListScreen screen) {
                screen.update(points);
            } else {
                minecraft.setScreen(new PointListScreen(points, max));
            }
        } catch (NumberFormatException | ArrayIndexOutOfBoundsException e) {
            // A malformed signal opens nothing.
        }
    }

    // A string argument can come back as a plain component after the packet's JSON round trip.
    private static String text(Object arg) {

        return arg instanceof Component component ? component.getString() : String.valueOf(arg);
    }
}
