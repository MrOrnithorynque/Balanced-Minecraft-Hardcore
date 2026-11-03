import github.mrornithorynque.bmh.client.sky.StarChain;
import github.mrornithorynque.bmh.client.sky.StarField;
import github.mrornithorynque.bmh.client.sky.StarTrace;

import java.awt.BasicStroke;
import java.awt.Color;
import java.awt.Font;
import java.awt.Graphics2D;
import java.awt.RenderingHints;
import java.awt.image.BufferedImage;
import java.io.File;
import java.io.IOException;
import java.io.PrintWriter;
import java.util.List;
import java.util.Locale;
import java.util.function.Function;
import javax.imageio.ImageIO;

/**
 * Offline previews of the amulet's sky reading. Runs the mod's own star code (StarField, StarChain, StarTrace from
 * common/src, compiled next to this file) and rasterizes vanilla's stars plus the reading's triangles with the game's
 * additive blend, as a player would see them and as an all-sky map. Also sweeps every azimuth over the whole night
 * and writes the chain statistics to stats.txt.
 *
 * Run from the repository root (JDK 17+), with the three plain-Java classes of client/sky:
 *   S=common/src/main/java/github/mrornithorynque/bmh/client/sky
 *   javac -d /tmp/sky docs/redesign/sky-previews/SkyPreview.java $S/StarField.java $S/StarChain.java $S/StarTrace.java
 *   java -Djava.awt.headless=true -cp /tmp/sky SkyPreview docs/redesign/sky-previews
 */
public class SkyPreview {

    static final int VIEW_W = 1280, VIEW_H = 720, MAP = 720, HEADER = 44;
    static final double FOV = 70.0, PITCH = 32.0;

    public static void main(String[] args) throws IOException {
        File dir = new File(args.length > 0 ? args[0] : ".");
        List<StarField.Star> stars = StarField.stars();

        try (PrintWriter stats = new PrintWriter(new File(dir, "stats.txt"), "UTF-8")) {
            sweep(stars, stats);
        }
        if (args.length > 1 && args[1].equals("stats")) {
            return;
        }

        int[] azimuths = { 0, 90, 180, 270 };
        long[][] times = { { 18000, 0 }, { 22000, 1 } };
        for (long[] time : times) {
            for (int azimuth : azimuths) {
                String name = String.format(Locale.ROOT, "az%03d-%s.png", azimuth, time[1] == 0 ? "midnight" : "late-night");
                ImageIO.write(preview(stars, azimuth, time[0], 2.5), "png", new File(dir, name));
                System.out.println("wrote " + name);
            }
        }
        double[] frames = { 0.35, 0.9, 1.6, 9.0, 10.2 };
        for (double t : frames) {
            String name = String.format(Locale.ROOT, "trace-az135-midnight-%04.1fs.png", t);
            ImageIO.write(preview(stars, 135, 18000, t), "png", new File(dir, name));
            System.out.println("wrote " + name);
        }
    }

    // Vanilla DimensionType.timeOfDay and ClientLevel.getStarBrightness (Math.cos for Mth.cos's table).
    static double timeOfDay(long dayTime) {
        double d0 = frac(dayTime / 24000.0 - 0.25);
        double d1 = 0.5 - Math.cos(d0 * Math.PI) / 2.0;
        return (float) (d0 * 2.0 + d1) / 3.0F;
    }

    static float starBrightness(double timeOfDay) {
        float f1 = 1.0F - ((float) Math.cos(timeOfDay * Math.PI * 2) * 2.0F + 0.25F);
        f1 = Math.max(0.0F, Math.min(1.0F, f1));
        return f1 * f1 * 0.5F;
    }

    static double frac(double v) {
        return v - Math.floor(v);
    }

    // Every azimuth, the whole night (dusk 12500 to dawn 23500): how many stars, how far the last one sits from the
    // true direction, how even the gaps are.
    static void sweep(List<StarField.Star> stars, PrintWriter out) {
        int cases = 0, empty = 0;
        int[] lengths = new int[8];
        int[] errorBuckets = new int[4]; // last star off the true azimuth: < 2, 2-5, 5-10, >= 10 degrees
        int[] turnBuckets = new int[3];  // sharpest bend of the chain: < 30, 30-60, >= 60 degrees
        double worstLastAzimuth = 0, sumLastAzimuth = 0, worstOff = 0;
        double minGap = 180, maxGap = 0, minTop = 90, maxBottom = 0;
        String worstCase = "";
        long started = System.nanoTime();
        for (long dayTime = 12500; dayTime <= 23500; dayTime += 250) {
            double t = timeOfDay(dayTime);
            for (int azimuth = 0; azimuth < 360; azimuth++) {
                int[] chain = StarChain.pick(stars, azimuth, t);
                cases++;
                if (chain.length == 0) {
                    empty++;
                    continue;
                }
                lengths[chain.length]++;
                double[] last = world(stars.get(chain[chain.length - 1]), t);
                double error = Math.abs(angleDiff(azimuthOf(last), azimuth));
                sumLastAzimuth += error;
                errorBuckets[error < 2 ? 0 : error < 5 ? 1 : error < 10 ? 2 : 3]++;
                if (error > worstLastAzimuth) {
                    worstLastAzimuth = error;
                    worstCase = "azimuth " + azimuth + " dayTime " + dayTime;
                }
                double sharpest = 0;
                for (int i = 1; i + 1 < chain.length; i++) {
                    double[] p = world(stars.get(chain[i - 1]), t), q = world(stars.get(chain[i]), t), r = world(stars.get(chain[i + 1]), t);
                    double[] u = { q[0] - p[0], q[1] - p[1], q[2] - p[2] }, v = { r[0] - q[0], r[1] - q[1], r[2] - q[2] };
                    sharpest = Math.max(sharpest, Math.toDegrees(Math.acos(Math.max(-1, Math.min(1, dot(u, v) / Math.sqrt(dot(u, u) * dot(v, v)))))));
                }
                turnBuckets[sharpest < 30 ? 0 : sharpest < 60 ? 1 : 2]++;
                for (int i = 0; i < chain.length; i++) {
                    double[] w = world(stars.get(chain[i]), t);
                    double a = Math.toRadians(azimuth);
                    worstOff = Math.max(worstOff, Math.abs(Math.toDegrees(Math.asin(w[0] * Math.cos(a) + w[2] * Math.sin(a)))));
                    if (i > 0) {
                        double gap = Math.toDegrees(Math.acos(Math.min(1, dot(w, world(stars.get(chain[i - 1]), t)))));
                        minGap = Math.min(minGap, gap);
                        maxGap = Math.max(maxGap, gap);
                    }
                }
                minTop = Math.min(minTop, altitude(world(stars.get(chain[0]), t)));
                maxBottom = Math.max(maxBottom, altitude(last));
            }
        }
        double ms = (System.nanoTime() - started) / 1e6 / cases;
        out.printf(Locale.ROOT, "Sweep: azimuth 0-359 step 1, dayTime 12500-23500 step 250: %d readings%n", cases);
        out.printf(Locale.ROOT, "No chain found: %d%n", empty);
        for (int k = 1; k < lengths.length; k++) {
            if (lengths[k] > 0) {
                out.printf(Locale.ROOT, "Chains of %d stars: %d (%.1f%%)%n", k, lengths[k], 100.0 * lengths[k] / cases);
            }
        }
        out.printf(Locale.ROOT, "Last star vs true azimuth: mean %.2f deg, worst %.2f deg (%s)%n",
                sumLastAzimuth / Math.max(1, cases - empty), worstLastAzimuth, worstCase);
        out.printf(Locale.ROOT, "  under 2 deg %d, 2-5 deg %d, 5-10 deg %d, 10 deg or more %d%n",
                errorBuckets[0], errorBuckets[1], errorBuckets[2], errorBuckets[3]);
        out.printf(Locale.ROOT, "Sharpest bend per chain: under 30 deg %d, 30-60 deg %d, 60 deg or more %d%n",
                turnBuckets[0], turnBuckets[1], turnBuckets[2]);
        out.printf(Locale.ROOT, "Worst off-plane star: %.2f deg%n", worstOff);
        out.printf(Locale.ROOT, "Gaps between stars: %.1f to %.1f deg%n", minGap, maxGap);
        out.printf(Locale.ROOT, "Top star altitude >= %.1f deg, bottom star altitude <= %.1f deg%n", minTop, maxBottom);
        out.printf(Locale.ROOT, "Pick time: %.3f ms per reading (this JVM, cold start included)%n", ms);
        out.flush();
    }

    static BufferedImage preview(List<StarField.Star> stars, int azimuth, long dayTime, double seconds) {
        double t = timeOfDay(dayTime);
        float brightness = starBrightness(t);
        int[] chain = StarChain.pick(stars, azimuth, t);
        float skyAlpha = Math.min(1.0F, brightness / 0.5F);
        float[] tris = StarTrace.triangles(stars, chain, seconds, skyAlpha);

        // Player view: looking toward the azimuth, PITCH up, vertical FOV like the default 70.
        double a = Math.toRadians(azimuth), p = Math.toRadians(PITCH);
        double[] fwd = { Math.sin(a) * Math.cos(p), Math.sin(p), -Math.cos(a) * Math.cos(p) };
        double[] right = { Math.cos(a), 0, Math.sin(a) };
        double[] up = cross(right, fwd);
        double focal = (VIEW_H / 2.0) / Math.tan(Math.toRadians(FOV / 2));
        Function<double[], double[]> view = w -> {
            double z = dot(w, fwd);
            if (z <= 0.05) {
                return null;
            }
            return new double[] { VIEW_W / 2.0 + dot(w, right) / z * focal, VIEW_H / 2.0 - dot(w, up) / z * focal };
        };
        // All-sky map: zenith in the centre, horizon on the rim, north up, east right (a map, not a look up).
        double radius = MAP / 2.0 - 30;
        Function<double[], double[]> map = w -> {
            double alt = altitude(w);
            if (alt < -2) {
                return null;
            }
            double r = (90 - alt) / 90 * radius, az = Math.toRadians(azimuthOf(w));
            return new double[] { MAP / 2.0 + r * Math.sin(az), MAP / 2.0 - r * Math.cos(az) };
        };

        Raster viewR = new Raster(VIEW_W, VIEW_H), mapR = new Raster(MAP, MAP);
        viewR.sky(view, fwd, up, focal);
        mapR.disc(radius);
        for (Raster r : new Raster[] { viewR, mapR }) {
            Function<double[], double[]> proj = r == viewR ? view : map;
            // The moon (40 blocks wide, 100 out), dim grey, only to show where it is.
            double[][] moon = { { -20, -100, 20 }, { 20, -100, 20 }, { 20, -100, -20 }, { -20, -100, -20 } };
            float[] grey = { 0.35F, 0.35F, 0.33F };
            r.quad(proj, toWorld(moon, t), grey, 1.0F);
            // Vanilla's stars: shader colour (b, b, b, b), blend SRC_ALPHA + ONE.
            float[] white = { brightness, brightness, brightness };
            for (StarField.Star star : stars) {
                r.quad(proj, toWorld(star.corners, t), white, brightness);
            }
            for (int i = 0; i + 3 * StarTrace.STRIDE <= tris.length; i += 3 * StarTrace.STRIDE) {
                r.triangle(proj, tris, i, t);
            }
        }

        BufferedImage img = new BufferedImage(VIEW_W + MAP, VIEW_H + HEADER, BufferedImage.TYPE_INT_RGB);
        viewR.copyTo(img, 0, HEADER);
        mapR.copyTo(img, VIEW_W, HEADER);
        Graphics2D g = img.createGraphics();
        g.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);
        g.setRenderingHint(RenderingHints.KEY_TEXT_ANTIALIASING, RenderingHints.VALUE_TEXT_ANTIALIAS_ON);
        annotate(g, view, map, radius, azimuth, dayTime, t, seconds, stars, chain, brightness);
        g.dispose();
        return img;
    }

    static void annotate(Graphics2D g, Function<double[], double[]> view, Function<double[], double[]> map, double radius,
            int azimuth, long dayTime, double t, double seconds, List<StarField.Star> stars, int[] chain, float brightness) {
        g.setColor(new Color(18, 18, 22));
        g.fillRect(0, 0, VIEW_W + MAP, HEADER);
        g.setFont(new Font("SansSerif", Font.PLAIN, 15));
        g.setColor(new Color(225, 225, 230));
        String lastInfo = "";
        if (chain.length > 0) {
            double[] last = world(stars.get(chain[chain.length - 1]), t);
            lastInfo = String.format(Locale.ROOT, "  |  last star: azimuth %.1f, altitude %.1f", azimuthOf(last), altitude(last));
        }
        g.drawString(String.format(Locale.ROOT,
                "Reading toward azimuth %d  |  dayTime %d (sky angle %.3f, star brightness %.2f)  |  %d stars, %.2f s after the reading%s",
                azimuth, dayTime, t, brightness, chain.length, seconds, lastInfo), 14, 28);

        // Player view: horizon, compass ticks along it, the true direction.
        g.translate(0, HEADER);
        g.setClip(0, 0, VIEW_W, VIEW_H);
        g.setFont(new Font("SansSerif", Font.PLAIN, 13));
        for (int az = 0; az < 360; az += 5) {
            double r = Math.toRadians(az);
            double[] s = view.apply(new double[] { Math.sin(r), 0, -Math.cos(r) });
            if (s == null) {
                continue;
            }
            boolean major = az % 45 == 0;
            g.setColor(new Color(90, 90, 100));
            g.drawLine((int) s[0], (int) s[1], (int) s[0], (int) s[1] + (major ? 10 : 4));
            if (major) {
                g.drawString(compass(az), (int) s[0] - 6, (int) s[1] + 26);
            }
        }
        double[] h = view.apply(new double[] { Math.sin(Math.toRadians(azimuth)), 0, -Math.cos(Math.toRadians(azimuth)) });
        if (h != null) {
            g.setColor(new Color(255, 120, 90));
            g.fillPolygon(new int[] { (int) h[0] - 7, (int) h[0] + 7, (int) h[0] }, new int[] { (int) h[1] + 40, (int) h[1] + 40, (int) h[1] + 30 }, 3);
            g.drawString("true direction", (int) h[0] - 40, (int) h[1] + 56);
        }
        g.setColor(new Color(150, 150, 160));
        g.drawString(String.format(Locale.ROOT, "Player view: facing azimuth %d, %.0f deg up, vertical FOV %.0f", azimuth, PITCH, FOV), 12, VIEW_H - 12);
        g.setClip(null);

        // Map: altitude rings, compass, the azimuth's half-plane.
        g.translate(VIEW_W, 0);
        g.setColor(new Color(60, 60, 70));
        g.setStroke(new BasicStroke(1f));
        for (int alt = 0; alt < 90; alt += 30) {
            int r = (int) ((90 - alt) / 90.0 * radius);
            g.drawOval(MAP / 2 - r, MAP / 2 - r, 2 * r, 2 * r);
        }
        g.setColor(new Color(150, 150, 160));
        for (int az = 0; az < 360; az += 90) {
            double r = Math.toRadians(az);
            g.drawString(compass(az), (int) (MAP / 2 + (radius + 14) * Math.sin(r)) - 5, (int) (MAP / 2 - (radius + 14) * Math.cos(r)) + 5);
        }
        double r = Math.toRadians(azimuth);
        g.setColor(new Color(255, 120, 90, 110));
        g.setStroke(new BasicStroke(1f, BasicStroke.CAP_BUTT, BasicStroke.JOIN_MITER, 10f, new float[] { 4f, 6f }, 0f));
        g.drawLine(MAP / 2, MAP / 2, (int) (MAP / 2 + radius * Math.sin(r)), (int) (MAP / 2 - radius * Math.cos(r)));
        g.setColor(new Color(150, 150, 160));
        g.drawString("All-sky map: zenith centre, rings every 30 deg, north up, east right", 12, MAP - 12);
        g.translate(-VIEW_W, -HEADER);
    }

    static String compass(int az) {
        String[] names = { "N", "NE", "E", "SE", "S", "SW", "W", "NW" };
        return names[(az / 45) % 8];
    }

    static double[] world(StarField.Star s, double t) {
        return StarChain.toWorld(s.x, s.y, s.z, t);
    }

    static double[][] toWorld(double[][] points, double t) {
        double[][] out = new double[points.length][];
        for (int i = 0; i < points.length; i++) {
            out[i] = StarChain.toWorld(points[i][0], points[i][1], points[i][2], t);
        }
        return out;
    }

    static double altitude(double[] w) {
        return Math.toDegrees(Math.asin(Math.max(-1, Math.min(1, w[1]))));
    }

    static double azimuthOf(double[] w) {
        double az = Math.toDegrees(Math.atan2(w[0], -w[2]));
        return az < 0 ? az + 360 : az;
    }

    static double angleDiff(double a, double b) {
        double d = (a - b) % 360;
        return d > 180 ? d - 360 : d < -180 ? d + 360 : d;
    }

    static double dot(double[] a, double[] b) {
        return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    }

    static double[] cross(double[] a, double[] b) {
        return new double[] { a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0] };
    }

    /** A float RGB canvas with GL's SRC_ALPHA + ONE blend. */
    static final class Raster {

        final int w, h;
        final float[] rgb;

        Raster(int w, int h) {
            this.w = w;
            this.h = h;
            this.rgb = new float[w * h * 3];
        }

        // Near-black night sky over a black ground (vanilla's midnight sky is close to black).
        void sky(Function<double[], double[]> view, double[] fwd, double[] up, double focal) {
            for (int y = 0; y < h; y++) {
                // The view ray through this row, centre column: its altitude decides sky or ground.
                double dy = (h / 2.0 - y) / focal;
                double[] ray = { fwd[0] + up[0] * dy, fwd[1] + up[1] * dy, fwd[2] + up[2] * dy };
                double alt = Math.toDegrees(Math.atan2(ray[1], Math.hypot(ray[0], ray[2])));
                float k = (float) Math.max(0, Math.min(1, 1 - alt / 60));
                float[] c = alt < 0 ? new float[] { 0.012F, 0.014F, 0.018F }
                        : new float[] { 0.008F + 0.03F * k, 0.010F + 0.035F * k, 0.025F + 0.06F * k };
                for (int x = 0; x < w; x++) {
                    System.arraycopy(c, 0, rgb, (y * w + x) * 3, 3);
                }
            }
        }

        void disc(double radius) {
            for (int y = 0; y < h; y++) {
                for (int x = 0; x < w; x++) {
                    double d = Math.hypot(x - w / 2.0, y - h / 2.0) / radius;
                    float[] c = d > 1 ? new float[] { 0.07F, 0.07F, 0.08F } : new float[] { 0.01F + 0.03F * (float) d, 0.012F + 0.035F * (float) d, 0.03F + 0.06F * (float) d };
                    System.arraycopy(c, 0, rgb, (y * w + x) * 3, 3);
                }
            }
        }

        void quad(Function<double[], double[]> proj, double[][] corners, float[] c, float a) {
            double[] p0 = proj.apply(corners[0]), p1 = proj.apply(corners[1]), p2 = proj.apply(corners[2]), p3 = proj.apply(corners[3]);
            if (p0 == null || p1 == null || p2 == null || p3 == null) {
                return;
            }
            fill(p0, p1, p2, c, a, c, a, c, a);
            fill(p0, p2, p3, c, a, c, a, c, a);
        }

        void triangle(Function<double[], double[]> proj, float[] v, int i, double t) {
            double[][] s = new double[3][];
            float[][] c = new float[3][];
            float[] a = new float[3];
            for (int k = 0; k < 3; k++) {
                int o = i + k * StarTrace.STRIDE;
                s[k] = proj.apply(StarChain.toWorld(v[o], v[o + 1], v[o + 2], t));
                if (s[k] == null) {
                    return;
                }
                c[k] = new float[] { v[o + 3], v[o + 4], v[o + 5] };
                a[k] = v[o + 6];
            }
            fill(s[0], s[1], s[2], c[0], a[0], c[1], a[1], c[2], a[2]);
        }

        // Barycentric fill, 4x4 supersampled for coverage, colours interpolated per vertex like GL's.
        void fill(double[] p0, double[] p1, double[] p2, float[] c0, float a0, float[] c1, float a1, float[] c2, float a2) {
            double area = (p1[0] - p0[0]) * (p2[1] - p0[1]) - (p2[0] - p0[0]) * (p1[1] - p0[1]);
            if (Math.abs(area) < 1e-12) {
                return;
            }
            int minX = (int) Math.max(0, Math.floor(Math.min(p0[0], Math.min(p1[0], p2[0]))));
            int maxX = (int) Math.min(w - 1, Math.ceil(Math.max(p0[0], Math.max(p1[0], p2[0]))));
            int minY = (int) Math.max(0, Math.floor(Math.min(p0[1], Math.min(p1[1], p2[1]))));
            int maxY = (int) Math.min(h - 1, Math.ceil(Math.max(p0[1], Math.max(p1[1], p2[1]))));
            for (int y = minY; y <= maxY; y++) {
                for (int x = minX; x <= maxX; x++) {
                    float r = 0, gr = 0, b = 0;
                    for (int sy = 0; sy < 4; sy++) {
                        for (int sx = 0; sx < 4; sx++) {
                            double px = x + (sx + 0.5) / 4, py = y + (sy + 0.5) / 4;
                            double w0 = ((p1[0] - px) * (p2[1] - py) - (p2[0] - px) * (p1[1] - py)) / area;
                            double w1 = ((p2[0] - px) * (p0[1] - py) - (p0[0] - px) * (p2[1] - py)) / area;
                            double w2 = 1 - w0 - w1;
                            if (w0 < 0 || w1 < 0 || w2 < 0) {
                                continue;
                            }
                            float a = (float) (w0 * a0 + w1 * a1 + w2 * a2);
                            r += a * (float) (w0 * c0[0] + w1 * c1[0] + w2 * c2[0]);
                            gr += a * (float) (w0 * c0[1] + w1 * c1[1] + w2 * c2[1]);
                            b += a * (float) (w0 * c0[2] + w1 * c1[2] + w2 * c2[2]);
                        }
                    }
                    int o = (y * w + x) * 3;
                    rgb[o] += r / 16;
                    rgb[o + 1] += gr / 16;
                    rgb[o + 2] += b / 16;
                }
            }
        }

        void copyTo(BufferedImage img, int ox, int oy) {
            for (int y = 0; y < h; y++) {
                for (int x = 0; x < w; x++) {
                    int o = (y * w + x) * 3;
                    int r = (int) (Math.min(1, rgb[o]) * 255 + 0.5);
                    int g = (int) (Math.min(1, rgb[o + 1]) * 255 + 0.5);
                    int b = (int) (Math.min(1, rgb[o + 2]) * 255 + 0.5);
                    img.setRGB(ox + x, oy + y, (r << 16) | (g << 8) | b);
                }
            }
        }
    }
}
