package github.mrornithorynque.bmh.client.config;

import java.util.List;

import org.jspecify.annotations.Nullable;

import github.mrornithorynque.bmh.platform.Services;
import github.mrornithorynque.bmh.utilities.BMHConfigLimits;

import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.components.EditBox;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.input.KeyEvent;
import net.minecraft.network.chat.Component;
import net.minecraft.util.FormattedCharSequence;
import net.minecraft.util.StringUtil;

/**
 * The config screen of the mods list (Forge and NeoForge) and of Mod Menu (Fabric): the three values of
 * the mod config file, with their ranges. Done writes the file through the loader. It changes the local
 * file only: a dedicated server keeps its own file. Client only.
 */
public class ConfigScreen extends Screen {

    private static final int TOP = 36;
    private static final int ROW_HEIGHT = 36;
    private static final int FIELD_WIDTH = 120;
    private static final int BUTTON_WIDTH = 98;
    private static final int GAP = 4;
    // Opaque: since 26.1 text with alpha 0 is not drawn (1.21.1's font made it opaque). FIELD_COLOR is
    // EditBox.DEFAULT_TEXT_COLOR.
    private static final int TEXT_COLOR = 0xFFFFFFFF;
    private static final int NOTE_COLOR = 0xFFA0A0A0;
    private static final int ERROR_COLOR = 0xFFFF8080;
    private static final int FIELD_COLOR = 0xFFE0E0E0;

    private final Screen parent;

    // They keep their text when the window changes size: init() runs again then.
    private EditBox minDistance;
    private EditBox maxDistance;
    private EditBox amuletPoints;
    private Button done;
    @Nullable
    private Component error;

    public ConfigScreen(Screen parent) {

        super(Component.translatable("bmh.config.title"));
        this.parent = parent;
    }

    @Override
    protected void init() {

        this.minDistance = this.field(this.minDistance, 0, Services.PLATFORM.minDistanceFromBed());
        this.maxDistance = this.field(this.maxDistance, 1, Services.PLATFORM.maxDistanceFromBed());
        this.amuletPoints = this.field(this.amuletPoints, 2, Services.PLATFORM.maxAmuletPoints());

        int y = this.height - 28;

        this.done = this.addRenderableWidget(Button.builder(Component.translatable("gui.done"), button -> this.save())
                .bounds(this.width / 2 - BUTTON_WIDTH - GAP / 2, y, BUTTON_WIDTH, 20).build());
        this.addRenderableWidget(Button.builder(Component.translatable("gui.cancel"), button -> this.onClose())
                .bounds(this.width / 2 + GAP / 2, y, BUTTON_WIDTH, 20).build());

        // The responders call validate(): set them only when every field exists.
        this.minDistance.setResponder(value -> this.validate());
        this.maxDistance.setResponder(value -> this.validate());
        this.amuletPoints.setResponder(value -> this.validate());
        this.setInitialFocus(this.minDistance);
        this.validate();
    }

    private EditBox field(@Nullable EditBox old, int row, int configValue) {

        // Digits only. 26.1 removed EditBox.setFilter: the box refuses any typed or pasted text that is not all digits,
        // as the 1.21.1 filter did.
        EditBox box = new EditBox(this.font, (this.width - FIELD_WIDTH) / 2, TOP + row * ROW_HEIGHT + 11, FIELD_WIDTH, 20, this.label(row)) {

            @Override
            public void insertText(String input) {

                if (StringUtil.filterText(input).chars().allMatch(Character::isDigit)) {
                    super.insertText(input);
                }
            }
        };
        box.setMaxLength(6);
        box.setValue(old != null ? old.getValue() : String.valueOf(configValue));

        return this.addRenderableWidget(box);
    }

    private Component label(int row) {

        return switch (row) {
            case 0 -> Component.translatable("bmh.config.minDistanceFromBed",
                    BMHConfigLimits.MIN_DISTANCE_FROM_BED, BMHConfigLimits.MAX_DISTANCE_FROM_BED);
            case 1 -> Component.translatable("bmh.config.maxDistanceFromBed",
                    BMHConfigLimits.MIN_DISTANCE_FROM_BED, BMHConfigLimits.MAX_DISTANCE_FROM_BED);
            default -> Component.translatable("bmh.config.maxAmuletPoints",
                    BMHConfigLimits.MIN_AMULET_POINTS, BMHConfigLimits.MAX_AMULET_POINTS);
        };
    }

    private void validate() {

        Integer min = this.read(this.minDistance, BMHConfigLimits.MIN_DISTANCE_FROM_BED, BMHConfigLimits.MAX_DISTANCE_FROM_BED);
        Integer max = this.read(this.maxDistance, BMHConfigLimits.MIN_DISTANCE_FROM_BED, BMHConfigLimits.MAX_DISTANCE_FROM_BED);
        Integer points = this.read(this.amuletPoints, BMHConfigLimits.MIN_AMULET_POINTS, BMHConfigLimits.MAX_AMULET_POINTS);

        if (min == null || max == null || points == null) {
            this.error = Component.translatable("bmh.config.outOfRange");
        } else if (min > max) {
            this.error = Component.translatable("bmh.config.minAboveMax");
        } else {
            this.error = null;
        }

        this.done.active = this.error == null;
    }

    // The value of a field, or null when it is empty or out of range. A bad value shows in red.
    @Nullable
    private Integer read(EditBox box, int min, int max) {

        Integer value = null;

        try {
            int parsed = Integer.parseInt(box.getValue());

            if (parsed >= min && parsed <= max) {
                value = parsed;
            }
        } catch (NumberFormatException e) {
            // empty: value stays null
        }

        box.setTextColor(value != null ? FIELD_COLOR : ERROR_COLOR);

        return value;
    }

    private void save() {

        if (this.error == null) {
            Services.PLATFORM.saveConfig(Integer.parseInt(this.minDistance.getValue()), Integer.parseInt(this.maxDistance.getValue()),
                    Integer.parseInt(this.amuletPoints.getValue()));
        }

        this.onClose();
    }

    @Override
    public void onClose() {

        this.minecraft.setScreen(this.parent);
    }

    @Override
    public boolean keyPressed(KeyEvent event) {

        // Enter or the keypad Enter.
        if (event.isConfirmation() && this.done.active) {
            this.save();
            return true;
        }

        return super.keyPressed(event);
    }

    // 1.21.1 drew this dim layer first, then the vanilla background. Since 26.1 the background has its own step.
    @Override
    public void extractBackground(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float partialTick) {

        // 1.20 and 1.20.1 draw no background in Screen.render, later versions do. Without a world, nothing is behind the screen.
        graphics.fill(0, 0, this.width, this.height, this.minecraft.level == null ? 0xFF202020 : 0x80000000);
        super.extractBackground(graphics, mouseX, mouseY, partialTick);
    }

    @Override
    public void extractRenderState(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float partialTick) {

        super.extractRenderState(graphics, mouseX, mouseY, partialTick);

        graphics.centeredText(this.font, this.title, this.width / 2, 15, TEXT_COLOR);

        for (int row = 0; row < 3; row++) {
            graphics.centeredText(this.font, this.label(row), this.width / 2, TOP + row * ROW_HEIGHT, NOTE_COLOR);
        }

        int y = TOP + 3 * ROW_HEIGHT + 2;

        if (this.error != null) {
            graphics.centeredText(this.font, this.error, this.width / 2, y, ERROR_COLOR);
        }

        List<FormattedCharSequence> note = this.font.split(Component.translatable("bmh.config.note"), Math.min(this.width - 40, 300));

        for (int i = 0; i < note.size(); i++) {
            graphics.centeredText(this.font, note.get(i), this.width / 2, y + 14 + i * 10, NOTE_COLOR);
        }
    }
}
