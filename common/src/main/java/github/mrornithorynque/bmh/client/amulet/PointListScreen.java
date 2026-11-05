package github.mrornithorynque.bmh.client.amulet;

import java.util.ArrayList;
import java.util.List;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.core.BlockPos;
import net.minecraft.network.chat.Component;

/**
 * The list of the points of the Navigator's Amulet, with a Remove button on each. It is the way to remove a point
 * whose bed or lodestone is gone: the naming screen needs the block. The server opens it (handlers/AmuletPointSignal)
 * and sends the list again after each removal, so the screen always shows what the server keeps. Remove sends the
 * command /bmh point delete (handlers/AmuletPointCommand). Client only.
 */
public class PointListScreen extends Screen {

    public record Entry(String name, BlockPos pos) {
    }

    private static final int PER_PAGE = 5;
    private static final int PANEL_WIDTH = 280;
    private static final int ROW_HEIGHT = 24;
    private static final int NAME_WIDTH = 110;
    private static final int COORDS_X = 114;
    private static final int BUTTON_X = 216;
    private static final int BUTTON_WIDTH = 64;
    private static final int TEXT_COLOR = 0xFFFFFF;
    private static final int NOTE_COLOR = 0xA0A0A0;

    private List<Entry> points;
    private final int max;
    private int page;

    public PointListScreen(List<Entry> points, int max) {

        super(Component.translatable("bmh.point.list.title"));
        this.points = List.copyOf(points);
        this.max = max;
    }

    /** The server sent the list again: show it, on the same page if it still exists. */
    public void update(List<Entry> points) {

        this.points = List.copyOf(points);
        this.page = Math.min(this.page, this.lastPage());
        this.rebuildWidgets();
    }

    private int lastPage() {

        return Math.max(0, (this.points.size() - 1) / PER_PAGE);
    }

    private int left() {

        return (this.width - PANEL_WIDTH) / 2;
    }

    private int top() {

        return this.height / 2 - 70;
    }

    @Override
    protected void init() {

        int left = this.left();
        int top = this.top();
        int first = this.page * PER_PAGE;

        for (int row = 0; row < PER_PAGE && first + row < this.points.size(); row++) {

            BlockPos pos = this.points.get(first + row).pos();

            this.addRenderableWidget(Button.builder(Component.translatable("bmh.point.screen.remove"), button -> this.remove(pos))
                    .bounds(left + BUTTON_X, top + row * ROW_HEIGHT, BUTTON_WIDTH, 20).build());
        }

        int y = top + PER_PAGE * ROW_HEIGHT + 24;

        Button previous = this.addRenderableWidget(Button.builder(Component.literal("<"), button -> this.turn(-1))
                .bounds(left, y, 20, 20).build());
        Button next = this.addRenderableWidget(Button.builder(Component.literal(">"), button -> this.turn(1))
                .bounds(left + PANEL_WIDTH - 20, y, 20, 20).build());
        previous.active = this.page > 0;
        next.active = this.page < this.lastPage();

        this.addRenderableWidget(Button.builder(Component.translatable("gui.done"), button -> this.onClose())
                .bounds(this.width / 2 - 40, y, 80, 20).build());
    }

    private void turn(int by) {

        this.page = Math.max(0, Math.min(this.lastPage(), this.page + by));
        this.rebuildWidgets();
    }

    private void remove(BlockPos pos) {

        Minecraft minecraft = Minecraft.getInstance();

        if (minecraft.player != null) {
            minecraft.player.connection.sendUnsignedCommand("bmh point delete " + pos.getX() + " " + pos.getY() + " " + pos.getZ());
        }
    }

    @Override
    public void render(GuiGraphics graphics, int mouseX, int mouseY, float partialTick) {

        // A dim layer that works from 1.20 to 1.21.1: 1.20.1 draws no background in Screen.render, later versions do.
        graphics.fill(0, 0, this.width, this.height, 0x80000000);
        super.render(graphics, mouseX, mouseY, partialTick);

        int left = this.left();
        int top = this.top();

        graphics.drawCenteredString(this.font, this.title, this.width / 2, top - 28, TEXT_COLOR);
        graphics.drawCenteredString(this.font, Component.translatable("bmh.point.list.count", this.points.size(), this.max),
                this.width / 2, top - 14, NOTE_COLOR);

        if (this.points.isEmpty()) {
            graphics.drawCenteredString(this.font, Component.translatable("bmh.point.list.empty"), this.width / 2, top + 30, NOTE_COLOR);
        }

        int first = this.page * PER_PAGE;

        for (int row = 0; row < PER_PAGE && first + row < this.points.size(); row++) {

            Entry entry = this.points.get(first + row);
            int y = top + row * ROW_HEIGHT + 6;

            graphics.drawString(this.font, this.font.plainSubstrByWidth(entry.name(), NAME_WIDTH), left, y, TEXT_COLOR);
            graphics.drawString(this.font, entry.pos().getX() + " " + entry.pos().getY() + " " + entry.pos().getZ(),
                    left + COORDS_X, y, NOTE_COLOR);
        }

        graphics.drawCenteredString(this.font, Component.translatable("bmh.point.list.spawn"), this.width / 2,
                top + PER_PAGE * ROW_HEIGHT + 6, NOTE_COLOR);
    }

    @Override
    public boolean isPauseScreen() {

        return false;
    }
}
