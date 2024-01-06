package github.mrornithorynque.bmh.utilities;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

import org.slf4j.Logger;

import net.minecraft.client.Minecraft;

import net.minecraftforge.client.event.RenderGuiOverlayEvent;
import net.minecraftforge.eventbus.api.SubscribeEvent;
import net.minecraftforge.fml.common.Mod.EventBusSubscriber;
import net.minecraftforge.api.distmarker.Dist;
import com.mojang.blaze3d.platform.Window;

import com.mojang.logging.LogUtils;

@EventBusSubscriber(value = Dist.CLIENT)
public class TextDrawer {

    private static TextDrawer instance;
    private Minecraft mc;
    private static final Logger LOGGER = LogUtils.getLogger();

    public enum ScreenPosition {

        TOP_LEFT,
        TOP_CENTER,
        TOP_RIGHT,
        CENTER_LEFT,
        CENTER,
        CENTER_RIGHT,
        BOTTOM_LEFT,
        BOTTOM_CENTER,
        BOTTOM_RIGHT,
        UNDEFINED
    }

    private class Text {

        String   message;
        int      x;
        int      y;
        int      color;
        long     messageEndTime;
        int      timeFadeIn;
        int      timeFadeOut;
        ScreenPosition position;

        Text() {

            this.message        = "";
            this.x              = 0;
            this.y              = 0;
            this.color          = 0xFFFFFF;
            this.messageEndTime = 0;
            this.timeFadeIn     = 0;
            this.timeFadeOut    = 0;
            this.position       = ScreenPosition.UNDEFINED;
        }

        String log() {

            return "message: " + message
                 + ", x: "     + x
                 + ", y: "     + y
                 + ", color: " + color
                 + ", messageEndTime: " + messageEndTime;
        }
    }

    private List<Text> textList;

    private TextDrawer() {

        this.mc = Minecraft.getInstance();
        this.textList = new ArrayList<>();
    }

    public static TextDrawer getInstance() {

        if (instance == null) {

            instance = new TextDrawer();
        }

        return instance;
    }

    @SubscribeEvent
    public void onRenderGuiOverlay(RenderGuiOverlayEvent.Post event) {

        Iterator<Text> iterator = textList.iterator();

        while (iterator.hasNext()) {

            Text text = iterator.next();

            if (System.currentTimeMillis() < text.messageEndTime) {

                drawText(text, event);
            } else {

                iterator.remove();
            }
        }
    }

    public void drawString(String message, int x, int y, int color, int delay) {

        Text text           = new Text();
        text.message        = message;
        text.x              = x;
        text.y              = y;
        text.color          = color;
        text.messageEndTime = System.currentTimeMillis() + delay;

        textList.add(text);
    }

    public void drawString(String message, ScreenPosition position, int color, int delay) {

        Text text           = new Text();
        text.message        = message;
        text.position       = position;
        text.color          = color;
        text.messageEndTime = System.currentTimeMillis() + delay;

        textList.add(text);
    }

    public void drawStringFadeInOut(String message, int x, int y, int color, int timeFadeIn, int timeFadeOut, int delay) {

        Text text           = new Text();
        text.message        = message;
        text.x              = x;
        text.y              = y;
        text.color          = color;
        text.messageEndTime = System.currentTimeMillis() + delay;
        text.timeFadeIn     = timeFadeIn;
        text.timeFadeOut    = timeFadeOut;

        textList.add(text);
    }

    public void drawStringFadeInOut(String message, ScreenPosition position, int color, int timeFadeIn, int timeFadeOut, int delay) {

        Text text           = new Text();
        text.message        = message;
        text.position       = position;
        text.color          = color;
        text.messageEndTime = System.currentTimeMillis() + delay;
        text.timeFadeIn     = timeFadeIn;
        text.timeFadeOut    = timeFadeOut;

        textList.add(text);
    }

    private void drawText(Text text, RenderGuiOverlayEvent.Post event) {

        long currentTime = System.currentTimeMillis();
        long elapsedTime = currentTime - (text.messageEndTime - text.timeFadeIn - text.timeFadeOut);

        int alpha = 255; // Default alpha value (opaque)

        if (elapsedTime < text.timeFadeIn) {
            alpha = (int) (255 * (elapsedTime / (float) text.timeFadeIn));
        } else if (currentTime > text.messageEndTime - text.timeFadeOut) {
            alpha = (int) (255 * ((text.messageEndTime - currentTime) / (float) text.timeFadeOut));
        }

        alpha = Math.max(0, Math.min(255, alpha));

        int finalColor = (text.color & 0x00FFFFFF) | (alpha << 24);

        int width  = event.getWindow().getGuiScaledWidth();
        int height = event.getWindow().getGuiScaledHeight();

        int x = text.x;
        int y = text.y;

        switch (text.position) {

            case TOP_LEFT:
                x = 0;
                y = 0;
                break;
            case TOP_CENTER:
                x = (width - mc.font.width(text.message)) / 2;
                y = 0;
                break;
            case TOP_RIGHT:
                x = width - mc.font.width(text.message);
                y = 0;
                break;
            case CENTER_LEFT:
                x = 0;
                y = height / 2;
                break;
            case CENTER:
                x = (width - mc.font.width(text.message)) / 2;
                y = height / 2;
                break;
            case CENTER_RIGHT:
                x = width  - mc.font.width(text.message);
                y = height / 2;
                break;
            case BOTTOM_LEFT:
                x = 0;
                y = height - mc.font.lineHeight;
                break;
            case BOTTOM_CENTER:
                x = (width - mc.font.width(text.message)) / 2;
                y = height - mc.font.lineHeight;
                break;
            case BOTTOM_RIGHT:
                x = width  - mc.font.width(text.message);
                y = height - mc.font.lineHeight;
                break;
            default: break;
        }

        event.getGuiGraphics().drawString(mc.font, text.message, x, y, finalColor);
    }
}
