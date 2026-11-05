package github.mrornithorynque.bmh.client.amulet;

import javax.annotation.Nullable;

import com.mojang.blaze3d.platform.InputConstants;

import github.mrornithorynque.bmh.handlers.AmuletPoints;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.components.EditBox;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.core.BlockPos;
import net.minecraft.network.chat.Component;

/**
 * The naming screen of a point of the Navigator's Amulet: a name field, Save, Remove (for a point that
 * exists) and Cancel. The server opens it (handlers/AmuletPointSignal) and decides everything: Save and
 * Remove send the command /bmh point (handlers/AmuletPointCommand) through a vanilla command packet,
 * because BMH has no network channel. Client only.
 */
public class PointScreen extends Screen {

    private static final int FIELD_WIDTH = 200;
    private static final int BUTTON_WIDTH = 64;
    private static final int GAP = 4;
    private static final int TEXT_COLOR = 0xFFFFFF;
    private static final int NOTE_COLOR = 0xA0A0A0;
    private static final int FULL_COLOR = 0xFF8080;

    private final BlockPos pos;
    @Nullable
    private final String existing;
    private final int used;
    private final int max;
    private final boolean bed;

    private EditBox name;
    private Button save;

    public PointScreen(BlockPos pos, @Nullable String existing, int used, int max, boolean bed) {

        super(Component.translatable(existing == null ? "bmh.point.screen.new" : "bmh.point.screen.edit"));
        this.pos = pos;
        this.existing = existing;
        this.used = used;
        this.max = max;
        this.bed = bed;
    }

    // A new point needs a free place; renaming a point does not.
    private boolean full() {

        return this.existing == null && this.used >= this.max;
    }

    @Override
    protected void init() {

        int top = this.height / 2 - 30;

        this.name = new EditBox(this.font, (this.width - FIELD_WIDTH) / 2, top, FIELD_WIDTH, 20, Component.translatable("bmh.point.screen.name"));
        this.name.setMaxLength(AmuletPoints.MAX_NAME_LENGTH);
        this.name.setValue(this.existing == null ? "" : this.existing);
        this.name.setResponder(value -> this.updateSave());
        this.addRenderableWidget(this.name);
        this.setInitialFocus(this.name);

        int buttons = this.existing == null ? 2 : 3;
        int x = (this.width - (buttons * BUTTON_WIDTH + (buttons - 1) * GAP)) / 2;
        int y = top + 30;

        this.save = this.addRenderableWidget(Button.builder(Component.translatable("bmh.point.screen.save"), button -> this.save())
                .bounds(x, y, BUTTON_WIDTH, 20).build());
        x += BUTTON_WIDTH + GAP;

        if (this.existing != null) {
            this.addRenderableWidget(Button.builder(Component.translatable("bmh.point.screen.remove"), button -> this.remove())
                    .bounds(x, y, BUTTON_WIDTH, 20).build());
            x += BUTTON_WIDTH + GAP;
        }

        this.addRenderableWidget(Button.builder(Component.translatable("gui.cancel"), button -> this.onClose())
                .bounds(x, y, BUTTON_WIDTH, 20).build());

        this.updateSave();
    }

    private void updateSave() {

        this.save.active = !this.full() && !AmuletPoints.cleanName(this.name.getValue()).isEmpty();
    }

    private void save() {

        String clean = AmuletPoints.cleanName(this.name.getValue());

        if (!this.full() && !clean.isEmpty()) {
            this.send("bmh point save " + this.pos.getX() + " " + this.pos.getY() + " " + this.pos.getZ() + " " + clean);
        }

        this.onClose();
    }

    private void remove() {

        this.send("bmh point remove " + this.pos.getX() + " " + this.pos.getY() + " " + this.pos.getZ());
        this.onClose();
    }

    private void send(String command) {

        Minecraft minecraft = Minecraft.getInstance();

        if (minecraft.player != null) {
            minecraft.player.connection.sendUnsignedCommand(command);
        }
    }

    @Override
    public boolean keyPressed(int keyCode, int scanCode, int modifiers) {

        if ((keyCode == InputConstants.KEY_RETURN || keyCode == InputConstants.KEY_NUMPADENTER) && this.save.active) {
            this.save();
            return true;
        }

        return super.keyPressed(keyCode, scanCode, modifiers);
    }

    @Override
    public void render(GuiGraphics graphics, int mouseX, int mouseY, float partialTick) {

        // A dim layer that works from 1.20 to 1.21.1: 1.20.1 draws no background in Screen.render, later versions do.
        graphics.fill(0, 0, this.width, this.height, 0x80000000);
        super.render(graphics, mouseX, mouseY, partialTick);

        int top = this.height / 2 - 30;
        graphics.drawCenteredString(this.font, this.title, this.width / 2, top - 30, TEXT_COLOR);
        graphics.drawCenteredString(this.font, Component.translatable(this.bed ? "bmh.point.screen.bed" : "bmh.point.screen.lodestone"),
                this.width / 2, top - 16, NOTE_COLOR);

        Component count = this.full()
                ? Component.translatable("bmh.point.full", this.max)
                : Component.translatable("bmh.point.screen.count", this.used + (this.existing == null ? 1 : 0), this.max);
        graphics.drawCenteredString(this.font, count, this.width / 2, top + 58, this.full() ? FULL_COLOR : NOTE_COLOR);
    }

    @Override
    public boolean isPauseScreen() {

        return false;
    }
}
