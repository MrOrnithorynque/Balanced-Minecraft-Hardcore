import java.util.ArrayList;
import java.util.List;
import java.util.Random;

// Reference values for the Horizon port of JourneyOverlay. The code below is a copy of
// common/src/main/java/github/mrornithorynque/bmh/client/journey/JourneyOverlay.java (26.1), with the
// Minecraft types taken out (Mth.clamp, Mth.lerp written inline). Run: java JourneyRef.java
public class JourneyRef {

    static final long FADE_OUT = 600;
    static final long BLACK = 250;
    static final long SKY_IN = 1200;
    static final long HOME_AT = 700;
    static final long TRACE_AT = 1500;
    static final long SEGMENT = 210;
    static final long STAR_LIGHT_UP = 150;
    static final int BLOCKS_HIGH = 72;
    static final int SKY_TOP = 0x0B1433;
    static final int SKY_BOTTOM = 0x03040B;
    static final int DUST_COLOR = 0x9AA8D8;
    static final int[] STAR_COLORS = { 0xFFFFFF, 0xCFE0FF, 0xFFF2CF, 0xBFD4FF };
    static final int LINE_CORE = 0xF0E4FF;
    static final int LINE_EDGE = 0xB58CFF;
    static final int[] BAYER = { 0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5 };
    static final double TILE = 3.0;
    static final int STAR_COUNT = 1100;
    static final int BAND_STAR_COUNT = 360;
    static final int DUST_COUNT = 420;
    static final double[] LAYER_SPEED = { 0.3, 0.6 };
    static final double BAND_SPEED = 0.3;
    static final int SEGMENTS = 11;
    static final double HOME_SCREEN_X = 0.3;
    static final double HEAD_SCREEN_X = 0.55;
    static final int STAR_GAP = 8;

    static float clampF(float v, float lo, float hi) { return v < lo ? lo : Math.min(v, hi); }
    static double clampD(double v, double lo, double hi) { return v < lo ? lo : Math.min(v, hi); }
    static float lerpF(float d, float a, float b) { return a + d * (b - a); }
    static double lerpD(double d, double a, double b) { return a + d * (b - a); }

    Star[] stars;
    Star[] bandStars;
    Dust[] dust;
    double[] chainU = new double[SEGMENTS + 1];
    double[] chainV = new double[SEGMENTS + 1];
    double[] chainLength = new double[SEGMENTS + 1];
    int dissolveX;
    int dissolveY;
    List<int[]> fills = new ArrayList<>();

    JourneyRef(Random random) {
        this.stars = new Star[STAR_COUNT];
        this.bandStars = new Star[BAND_STAR_COUNT];
        this.dust = new Dust[DUST_COUNT];
        for (int i = 0; i < STAR_COUNT; i++) {
            this.stars[i] = new Star(random, random.nextDouble() * TILE, random.nextDouble(), i % 3 == 0 ? 1 : 0);
        }
        for (int i = 0; i < BAND_STAR_COUNT; i++) {
            double along = -0.1 + random.nextDouble() * (TILE + 0.1);
            this.bandStars[i] = new Star(random, along, bandV(along) + random.nextGaussian() * 0.06, -1);
        }
        for (int i = 0; i < DUST_COUNT; i++) {
            this.dust[i] = new Dust(random);
        }
        this.chainV[0] = 0.4 + random.nextDouble() * 0.15;
        int turn = random.nextBoolean() ? 1 : -1;
        for (int i = 1; i <= SEGMENTS; i++) {
            if (random.nextFloat() < 0.7F) {
                turn = -turn;
            }
            double length = 0.14 + random.nextDouble() * 0.16;
            double angle = Math.toRadians(turn * (18 + random.nextDouble() * 44));
            double v = this.chainV[i - 1] + length * Math.sin(angle);
            if (v < 0.22 || v > 0.66) {
                v = v < 0.22 ? 0.44 - v : 1.32 - v;
                turn = v < 0.44 ? 1 : -1;
            }
            this.chainU[i] = this.chainU[i - 1] + length * Math.cos(angle);
            this.chainV[i] = v;
            this.chainLength[i] = this.chainLength[i - 1]
                    + Math.hypot(this.chainU[i] - this.chainU[i - 1], this.chainV[i] - this.chainV[i - 1]);
        }
    }

    double tracedLength(long time) {
        double progress = clampD((double) (time - TRACE_AT) / (SEGMENTS * SEGMENT), 0.0, 1.0);
        return this.chainLength[SEGMENTS] * ease((float) progress);
    }

    double camera(double traced, int width, int height) {
        return Math.max(0.0, HOME_SCREEN_X * width + uAt(traced) * height - HEAD_SCREEN_X * width);
    }

    double uAt(double length) {
        for (int i = 0; i < SEGMENTS; i++) {
            if (length <= this.chainLength[i + 1]) {
                double part = (length - this.chainLength[i]) / (this.chainLength[i + 1] - this.chainLength[i]);
                return lerpD(part, this.chainU[i], this.chainU[i + 1]);
            }
        }
        return this.chainU[SEGMENTS];
    }

    int[] chainPoint(int i, double camera, int width, int height) {
        return new int[] { (int) Math.round(HOME_SCREEN_X * width + this.chainU[i] * height - camera),
                (int) Math.round(this.chainV[i] * height) };
    }

    // The art of one frame, as extractRenderState draws it after the fade (without the message).
    void drawArt(long now, long time, int width, int height, float reveal) {
        float sky = clampF((float) (time - BLACK) / SKY_IN, 0.0F, 1.0F);
        double traced = tracedLength(time);
        double camera = camera(traced, width, height);
        Frame frame = new Frame(width, height, reveal);
        drawSky(frame, sky);
        drawDust(frame, sky, camera);
        drawStars(frame, now, time, sky, camera);
        drawChain(frame, now, time, sky, traced, camera);
    }

    void fill(int x0, int y0, int x1, int y1, int color) {
        this.fills.add(new int[] { Math.min(x0, x1), Math.min(y0, y1), Math.max(x0, x1), Math.max(y0, y1), color });
    }

    void drawSky(Frame frame, float sky) {
        if (frame.reveal <= 0.0F) {
            fill(0, 0, frame.width, frame.height, 0xFF000000);
            // The gradient: recorded as one rectangle with both colours.
            this.fills.add(new int[] { 0, 0, frame.width, frame.height, argb(sky, SKY_TOP), argb(sky, SKY_BOTTOM) });
            return;
        }
        for (int y = 0; y < frame.height; y += frame.cell) {
            int color = 0xFF000000 | lerpColor(0x000000, lerpColor(SKY_TOP, SKY_BOTTOM, (float) y / frame.height), sky);
            for (int x = 0; x < frame.width; x += frame.cell) {
                if (!revealedBlock(frame, x, y)) {
                    fill(x, y, Math.min(x + frame.cell, frame.width), Math.min(y + frame.cell, frame.height), color);
                }
            }
        }
    }

    void drawDust(Frame frame, float sky, double camera) {
        float alpha = sky * (1.0F - clampF(frame.reveal * 5.0F, 0.0F, 1.0F));
        if (alpha <= 0.0F) {
            return;
        }
        int unit = Math.max(1, frame.height / 240);
        for (Dust blob : this.dust) {
            int x = (int) Math.round(blob.u * frame.height - BAND_SPEED * camera);
            int y = (int) Math.round(blob.v * frame.height);
            int size = blob.size * unit;
            if (x + size < 0 || x - size > frame.width) {
                continue;
            }
            fill(x - size / 2, y - size / 2, x + size / 2, y + size / 2, argb(alpha * blob.alpha, DUST_COLOR));
        }
    }

    void drawStars(Frame frame, long now, long time, float sky, double camera) {
        if (sky <= 0.0F) {
            return;
        }
        long tile = Math.round(TILE * frame.height);
        for (Star star : this.stars) {
            int x = (int) Math.floorMod(Math.round(star.u * frame.height - LAYER_SPEED[star.layer] * camera), tile);
            drawStar(frame, star, x, now, time);
        }
        for (Star star : this.bandStars) {
            drawStar(frame, star, (int) Math.round(star.u * frame.height - BAND_SPEED * camera), now, time);
        }
    }

    void drawStar(Frame frame, Star star, int x, long now, long time) {
        int y = (int) Math.round(star.v * frame.height);
        float appear = clampF((time - BLACK - star.delay) / 300.0F, 0.0F, 1.0F);
        if (appear <= 0.0F || x < 2 || y < 2 || x >= frame.width - 2 || y >= frame.height - 2 || !frame.covered(x, y)) {
            return;
        }
        float twinkle = 1.0F - star.twinkle * (0.5F + 0.5F * (float) Math.sin(star.phase + now * star.speed));
        float alpha = star.brightness * twinkle * appear;
        frame.pixel(x, y, argb(alpha, star.color));
        for (int arm = 1; arm <= star.arms; arm++) {
            int armColor = argb(alpha * (arm == 1 ? 0.5F : 0.22F), star.color);
            frame.pixel(x - arm, y, armColor);
            frame.pixel(x + arm, y, armColor);
            frame.pixel(x, y - arm, armColor);
            frame.pixel(x, y + arm, armColor);
        }
    }

    void drawChain(Frame frame, long now, long time, float sky, double traced, double camera) {
        if (time < HOME_AT) {
            return;
        }
        float appear = clampF((float) (time - HOME_AT) / STAR_LIGHT_UP, 0.0F, 1.0F);
        for (int i = 0; i < SEGMENTS; i++) {
            if (traced <= this.chainLength[i]) {
                break;
            }
            int[] from = chainPoint(i, camera, frame.width, frame.height);
            int[] to = chainPoint(i + 1, camera, frame.width, frame.height);
            double part = Math.min(1.0, (traced - this.chainLength[i]) / (this.chainLength[i + 1] - this.chainLength[i]));
            drawSegment(frame, from, to, part, appear, part < 1.0);
        }
        for (int i = 0; i <= SEGMENTS; i++) {
            int[] at = chainPoint(i, camera, frame.width, frame.height);
            double reached = i == SEGMENTS ? time - TRACE_AT - SEGMENTS * SEGMENT
                    : (traced - this.chainLength[i]) / this.chainLength[SEGMENTS] * SEGMENTS * SEGMENT;
            float lit = i == 0 ? appear : clampF((float) (reached / STAR_LIGHT_UP), 0.0F, 1.0F) * appear;
            if (i == 0) {
                float pulse = 0.5F + 0.5F * (float) Math.sin(now * 0.008);
                drawHalo(frame, at[0], at[1], 10 + Math.round(pulse * 2), lit * (0.75F + 0.25F * pulse), LINE_EDGE);
                drawHalo(frame, at[0], at[1], 16 + Math.round(pulse * 3), lit * 0.3F, LINE_EDGE);
                drawKnot(frame, at[0], at[1], lit, 3);
            } else if (lit > 0.0F) {
                boolean last = i == SEGMENTS;
                drawHalo(frame, at[0], at[1], last ? 12 : 7, lit, LINE_EDGE);
                drawHalo(frame, at[0], at[1], last ? 18 : 4, lit * (last ? 0.35F : 0.5F), last ? LINE_EDGE : LINE_CORE);
                drawKnot(frame, at[0], at[1], lit, last ? 3 : 2);
            } else {
                drawKnot(frame, at[0], at[1], 0.5F * appear * sky, 1);
            }
        }
    }

    static void drawKnot(Frame frame, int x, int y, float alpha, int arms) {
        frame.pixel(x, y, argb(alpha, 0xFFFFFF));
        for (int arm = 1; arm <= arms; arm++) {
            int color = argb(alpha * (arm == 1 ? 0.8F : 0.4F), LINE_CORE);
            frame.pixel(x - arm, y, color);
            frame.pixel(x + arm, y, color);
            frame.pixel(x, y - arm, color);
            frame.pixel(x, y + arm, color);
        }
        if (arms > 1) {
            int corner = argb(alpha * 0.35F, LINE_CORE);
            frame.pixel(x - 1, y - 1, corner);
            frame.pixel(x + 1, y - 1, corner);
            frame.pixel(x - 1, y + 1, corner);
            frame.pixel(x + 1, y + 1, corner);
        }
    }

    void drawSegment(Frame frame, int[] from, int[] to, double part, float alpha, boolean head) {
        int dx = to[0] - from[0];
        int dy = to[1] - from[1];
        int steps = Math.max(Math.abs(dx), Math.abs(dy));
        boolean flat = Math.abs(dx) >= Math.abs(dy);
        int last = (int) Math.round(steps * part);
        int[] spark = null;
        for (int k = STAR_GAP; k <= Math.min(last, steps - STAR_GAP); k++) {
            int x = from[0] + Math.round((float) dx * k / steps);
            int y = from[1] + Math.round((float) dy * k / steps);
            spark = new int[] { x, y };
            for (int side = -3; side <= 3; side++) {
                int px = flat ? x : x + side;
                int py = flat ? y + side : y;
                int abs = Math.abs(side);
                frame.pixel(px, py, abs == 0 ? argb(alpha, LINE_CORE) : abs == 1 ? argb(alpha * 0.6F, LINE_EDGE)
                        : abs == 2 ? argb(alpha * 0.3F, LINE_EDGE) : argb(alpha * 0.12F, LINE_EDGE));
            }
        }
        if (head && spark != null) {
            for (int sx = -1; sx <= 1; sx++) {
                for (int sy = -1; sy <= 1; sy++) {
                    frame.pixel(spark[0] + sx, spark[1] + sy, argb(alpha * (sx == 0 || sy == 0 ? 1.0F : 0.5F), 0xFFFFFF));
                }
            }
            drawHalo(frame, spark[0], spark[1], 4, alpha * 0.8F, LINE_EDGE);
        }
    }

    static void drawHalo(Frame frame, int x, int y, int radius, float alpha, int color) {
        int points = Math.max(8, (int) Math.ceil(radius * 2 * Math.PI));
        for (int k = 0; k < points; k++) {
            double angle = Math.PI * 2 * k / points;
            frame.pixel(x + (int) Math.round(Math.cos(angle) * radius), y + (int) Math.round(Math.sin(angle) * radius),
                    argb(alpha * 0.55F, color));
        }
    }

    boolean revealedBlock(Frame frame, int x, int y) {
        int cell = frame.cell;
        double dx = x + cell / 2.0 - this.dissolveX;
        double dy = y + cell / 2.0 - this.dissolveY;
        double farX = Math.max(this.dissolveX, frame.width - this.dissolveX);
        double farY = Math.max(this.dissolveY, frame.height - this.dissolveY);
        double radial = Math.min(1.0, Math.sqrt(dx * dx + dy * dy) / Math.sqrt(farX * farX + farY * farY));
        double dither = BAYER[((y / cell) & 3) * 4 + ((x / cell) & 3)] / 16.0;
        return frame.reveal > 0.6 * radial + 0.4 * dither;
    }

    static double bandV(double u) {
        return 0.1 + 0.24 * u / TILE;
    }

    static float ease(float t) {
        t = clampF(t, 0.0F, 1.0F);
        return t * t * (3.0F - 2.0F * t);
    }

    static int argb(float alpha, int rgb) {
        return (Math.round(clampF(alpha, 0.0F, 1.0F) * 255) << 24) | (rgb & 0xFFFFFF);
    }

    static int lerpColor(int from, int to, float t) {
        int r = Math.round(lerpF(t, (from >> 16) & 0xFF, (to >> 16) & 0xFF));
        int g = Math.round(lerpF(t, (from >> 8) & 0xFF, (to >> 8) & 0xFF));
        int b = Math.round(lerpF(t, from & 0xFF, to & 0xFF));
        return (r << 16) | (g << 8) | b;
    }

    final class Frame {
        final int width;
        final int height;
        final float reveal;
        final int cell;

        Frame(int width, int height, float reveal) {
            this.width = width;
            this.height = height;
            this.reveal = reveal;
            this.cell = Math.max(4, height / BLOCKS_HIGH);
        }

        boolean covered(int x, int y) {
            return this.reveal <= 0.0F || !revealedBlock(this, x - Math.floorMod(x, this.cell), y - Math.floorMod(y, this.cell));
        }

        void pixel(int x, int y, int color) {
            if (x >= 0 && y >= 0 && x < this.width && y < this.height && covered(x, y)) {
                fill(x, y, x + 1, y + 1, color);
            }
        }
    }

    static final class Star {
        final double u;
        final double v;
        final int layer;
        final float brightness;
        final int arms;
        final float twinkle;
        final int color;
        final double phase;
        final double speed;
        final long delay;

        Star(Random random, double u, double v, int layer) {
            this.u = u;
            this.v = v;
            this.layer = Math.max(0, layer);
            float kind = random.nextFloat();
            this.arms = layer == 1 ? (kind < 0.12F ? 2 : kind < 0.45F ? 1 : 0) : layer == 0 && kind < 0.06F ? 1 : 0;
            this.brightness = this.arms > 0 ? 0.8F + random.nextFloat() * 0.2F
                    : (layer < 0 ? 0.15F : 0.2F) + random.nextFloat() * 0.5F;
            this.twinkle = this.arms > 0 ? 0.45F : 0.25F;
            this.color = STAR_COLORS[random.nextInt(STAR_COLORS.length)];
            this.phase = random.nextDouble() * Math.PI * 2;
            this.speed = 0.0015 + random.nextDouble() * 0.0025;
            this.delay = (long) (random.nextDouble() * SKY_IN * 0.8);
        }
    }

    static final class Dust {
        final double u;
        final double v;
        final int size;
        final float alpha;

        Dust(Random random) {
            this.u = -0.1 + random.nextDouble() * (TILE + 0.1);
            this.v = bandV(this.u) + random.nextGaussian() * 0.045;
            this.size = 2 + random.nextInt(5);
            this.alpha = 0.02F + random.nextFloat() * 0.035F;
        }
    }

    static long hash(List<int[]> fills, int from) {
        long h = 17;
        for (int i = from; i < fills.size(); i++) {
            for (int v : fills.get(i)) {
                h = h * 31 + v;
            }
        }
        return h;
    }

    public static void main(String[] args) {
        Random r = new Random(42);
        System.out.println("random42 nextInt=" + r.nextInt() + " nextDouble=" + r.nextDouble() + " nextFloat="
                + r.nextFloat() + " nextBoolean=" + r.nextBoolean() + " nextInt7=" + r.nextInt(7) + " nextInt4="
                + r.nextInt(4) + " nextGaussian=" + r.nextGaussian() + " nextGaussian2=" + r.nextGaussian());
        System.out.println("ease(0.25)=" + ease(0.25F) + " ease(0.5)=" + ease(0.5F) + " ease(-1)=" + ease(-1F)
                + " argb(0.5,MSG)=" + Integer.toHexString(argb(0.5F, 0xFFF1D0)) + " lerpColor(.5)="
                + Integer.toHexString(lerpColor(SKY_TOP, SKY_BOTTOM, 0.5F)) + " lerpColor0(.37)="
                + Integer.toHexString(lerpColor(0, SKY_TOP, 0.37F)));

        JourneyRef j = new JourneyRef(new Random(12345));
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i <= SEGMENTS; i++) {
            sb.append(String.format("i=%d u=%s v=%s len=%s%n", i, Double.toString(j.chainU[i]),
                    Double.toString(j.chainV[i]), Double.toString(j.chainLength[i])));
        }
        System.out.print(sb);
        Star s = j.stars[0];
        System.out.println("star0 u=" + s.u + " v=" + s.v + " layer=" + s.layer + " arms=" + s.arms + " bright="
                + s.brightness + " color=" + Integer.toHexString(s.color) + " phase=" + s.phase + " speed=" + s.speed
                + " delay=" + s.delay);
        Star b = j.bandStars[0];
        System.out.println("band0 u=" + b.u + " v=" + b.v + " arms=" + b.arms + " bright=" + b.brightness + " delay="
                + b.delay);
        Dust d = j.dust[419];
        System.out.println("dust419 u=" + d.u + " v=" + d.v + " size=" + d.size + " alpha=" + d.alpha);

        int width = 960;
        int height = 540;
        double camEnd = j.camera(j.chainLength[SEGMENTS], width, height);
        int[] last = j.chainPoint(SEGMENTS, camEnd, width, height);
        System.out.println("traced(2000)=" + j.tracedLength(2000) + " camera(end)=" + camEnd + " last=" + last[0] + ","
                + last[1]);

        // A frame in the trace, before the dissolve.
        long now = 1_000_000;
        long time = 2600;
        j.drawArt(now, time, width, height, 0.0F);
        System.out.println("frameA count=" + j.fills.size() + " hash(from 2)=" + hash(j.fills, 2) + " gradient="
                + Integer.toHexString(j.fills.get(1)[4]) + "," + Integer.toHexString(j.fills.get(1)[5]));
        // A frame in the dissolve.
        j.fills.clear();
        j.dissolveX = last[0];
        j.dissolveY = last[1];
        j.drawArt(now + 4000, 6600, width, height, ease(0.3F));
        System.out.println("frameB count=" + j.fills.size() + " hash=" + hash(j.fills, 0));
        // A small window early in the sky fade-in.
        j.fills.clear();
        j.drawArt(now + 123, 900, 427, 241, 0.0F);
        System.out.println("frameC count=" + j.fills.size() + " hash(from 2)=" + hash(j.fills, 2));
    }
}
