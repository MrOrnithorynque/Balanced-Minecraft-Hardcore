package github.mrornithorynque.bmh.client.sky;

import java.util.List;

/**
 * The look and timing of a reading in the sky: the chain is traced one segment at a time, each star brightens as the
 * line reaches it, the whole holds, then fades. Output is triangles in the star field's own frame, so whoever draws
 * them only needs the star field's transform.
 *
 * Plain Java on purpose (no Minecraft class): the offline sky previews rasterize these very triangles.
 */
public final class StarTrace {

    public static final double SEGMENT_SECONDS = 0.6;
    public static final double HOLD_SECONDS = 8.0;
    public static final double FADE_SECONDS = 2.0;
    private static final double STAR_FADE_IN_SECONDS = 0.3;

    /** Floats per vertex in {@link #triangles}: x, y, z, red, green, blue, alpha. */
    public static final int STRIDE = 7;

    // Amethyst white: the core of the line, and its violet edge. Sizes in sky units (stars are 100 out, 0.3 to 0.5
    // wide), so the line is a little wider than a star.
    private static final float[] CORE = { 0xf0 / 255.0F, 0xe4 / 255.0F, 0xff / 255.0F };
    private static final float[] EDGE = { 0xb5 / 255.0F, 0x8c / 255.0F, 0xff / 255.0F };
    private static final double CORE_HALF_WIDTH = 0.10;
    private static final double EDGE_HALF_WIDTH = 0.34;
    private static final double GLOW_HALF_WIDTH = 0.9;
    private static final float EDGE_ALPHA = 0.6F;
    private static final float GLOW_ALPHA = 0.14F;
    /** The line stops this far from each star, so the stars stand out as the chain's knots. */
    private static final double STAR_GAP = 0.6;
    private static final double HALO_RADIUS = 1.1;
    private static final double LAST_HALO_RADIUS = 1.6;
    private static final float HALO_ALPHA = 0.55F;
    private static final double SPARK_RADIUS = 0.55;
    private static final int HALO_SIDES = 12;

    private StarTrace() {
    }

    /** Seconds from the first segment to the end of the fade, for a chain of {@code stars} stars. */
    public static double duration(int stars) {
        return traceSeconds(stars) + HOLD_SECONDS + FADE_SECONDS;
    }

    private static double traceSeconds(int stars) {
        return Math.max(0, stars - 1) * SEGMENT_SECONDS;
    }

    /**
     * The triangles to draw {@code seconds} after the reading appeared, {@link #STRIDE} floats per vertex.
     * {@code skyAlpha} (0 to 1) dims everything, the way rain and dawn dim the stars.
     */
    public static float[] triangles(List<StarField.Star> stars, int[] chain, double seconds, float skyAlpha) {
        Buffer out = new Buffer();
        double traced = traceSeconds(chain.length);
        double fade = seconds <= traced + HOLD_SECONDS ? 1.0
                : 1.0 - (seconds - traced - HOLD_SECONDS) / FADE_SECONDS;
        float alpha = (float) Math.max(0.0, Math.min(1.0, fade)) * skyAlpha;
        if (alpha <= 0.0F || seconds < 0.0) {
            return out.toArray();
        }

        for (int i = 0; i < chain.length; i++) {
            StarField.Star star = stars.get(chain[i]);
            double lit = Math.max(0.0, Math.min(1.0, (seconds - i * SEGMENT_SECONDS) / STAR_FADE_IN_SECONDS));
            if (lit > 0.0) {
                float a = (float) lit * alpha;
                // The star itself, again, brighter: its own square, in the line's core colour.
                double[][] c = star.corners;
                out.triangle(c[0], c[1], c[2], CORE, a * 0.9F, CORE, a * 0.9F, CORE, a * 0.9F);
                out.triangle(c[0], c[2], c[3], CORE, a * 0.9F, CORE, a * 0.9F, CORE, a * 0.9F);
                halo(out, star.x, star.y, star.z, i == chain.length - 1 ? LAST_HALO_RADIUS : HALO_RADIUS,
                        CORE, HALO_ALPHA * a);
            }
            if (i + 1 < chain.length) {
                double progress = (seconds - i * SEGMENT_SECONDS) / SEGMENT_SECONDS;
                if (progress > 0.0) {
                    segment(out, star, stars.get(chain[i + 1]), Math.min(1.0, progress), alpha);
                }
            }
        }
        return out.toArray();
    }

    // A flat ribbon along the chord between two stars, facing the camera (which sits at the sphere's centre, so the
    // chord covers the same pixels as the arc). Across it: a faint violet glow, the violet edge, the white core.
    private static void segment(Buffer out, StarField.Star from, StarField.Star to, double progress, float alpha) {
        double dx = to.x - from.x, dy = to.y - from.y, dz = to.z - from.z;
        double length = Math.sqrt(dx * dx + dy * dy + dz * dz);
        if (length <= 2.0 * STAR_GAP) {
            return;
        }
        double ux = dx / length, uy = dy / length, uz = dz / length;
        double start = STAR_GAP;
        double end = STAR_GAP + (length - 2.0 * STAR_GAP) * progress;
        double[] a = { from.x + ux * start, from.y + uy * start, from.z + uz * start };
        double[] b = { from.x + ux * end, from.y + uy * end, from.z + uz * end };

        // Sideways: across the chord and across the line of sight.
        double mx = from.x + to.x, my = from.y + to.y, mz = from.z + to.z;
        double sx = uy * mz - uz * my, sy = uz * mx - ux * mz, sz = ux * my - uy * mx;
        double s = Math.sqrt(sx * sx + sy * sy + sz * sz);
        sx /= s;
        sy /= s;
        sz /= s;

        double[] offsets = { -GLOW_HALF_WIDTH, -EDGE_HALF_WIDTH, -CORE_HALF_WIDTH, 0.0, CORE_HALF_WIDTH,
                EDGE_HALF_WIDTH, GLOW_HALF_WIDTH };
        float[][] colours = { EDGE, EDGE, EDGE, CORE, EDGE, EDGE, EDGE };
        float[] alphas = { 0.0F, GLOW_ALPHA * alpha, EDGE_ALPHA * alpha, alpha, EDGE_ALPHA * alpha,
                GLOW_ALPHA * alpha, 0.0F };
        for (int k = 0; k + 1 < offsets.length; k++) {
            double[] a0 = offset(a, sx, sy, sz, offsets[k]);
            double[] a1 = offset(a, sx, sy, sz, offsets[k + 1]);
            double[] b0 = offset(b, sx, sy, sz, offsets[k]);
            double[] b1 = offset(b, sx, sy, sz, offsets[k + 1]);
            out.triangle(a0, b0, b1, colours[k], alphas[k], colours[k], alphas[k], colours[k + 1], alphas[k + 1]);
            out.triangle(a0, b1, a1, colours[k], alphas[k], colours[k + 1], alphas[k + 1], colours[k + 1], alphas[k + 1]);
        }

        if (progress < 1.0) {
            halo(out, b[0], b[1], b[2], SPARK_RADIUS, CORE, 0.8F * alpha);
        }
    }

    // A soft disc around a point of the sphere: bright centre, transparent violet rim, tangent to the sphere. The sky
    // is drawn with back faces culled: every triangle here winds counter-clockwise as seen from the camera at the
    // sphere's centre (the ribbons and vanilla's own star quads do too).
    private static void halo(Buffer out, double x, double y, double z, double radius, float[] colour, float alpha) {
        double r = Math.sqrt(x * x + y * y + z * z);
        double nx = x / r, ny = y / r, nz = z / r;
        // Any vector not parallel to the normal gives the tangent plane's first axis.
        double px = Math.abs(ny) < 0.9 ? 0.0 : 1.0, py = Math.abs(ny) < 0.9 ? 1.0 : 0.0;
        double ex = py * nz, ey = -px * nz, ez = px * ny - py * nx;
        double e = Math.sqrt(ex * ex + ey * ey + ez * ez);
        ex /= e;
        ey /= e;
        ez /= e;
        double fx = ny * ez - nz * ey, fy = nz * ex - nx * ez, fz = nx * ey - ny * ex;
        double[] centre = { x, y, z };
        double[] previous = null;
        for (int k = 0; k <= HALO_SIDES; k++) {
            double angle = 2.0 * Math.PI * k / HALO_SIDES;
            double cos = Math.cos(angle) * radius, sin = Math.sin(angle) * radius;
            double[] rim = { x + ex * cos + fx * sin, y + ey * cos + fy * sin, z + ez * cos + fz * sin };
            if (previous != null) {
                out.triangle(centre, rim, previous, colour, alpha, EDGE, 0.0F, EDGE, 0.0F);
            }
            previous = rim;
        }
    }

    private static double[] offset(double[] p, double sx, double sy, double sz, double by) {
        return new double[] { p[0] + sx * by, p[1] + sy * by, p[2] + sz * by };
    }

    private static final class Buffer {

        private float[] data = new float[1024];
        private int size;

        void triangle(double[] p0, double[] p1, double[] p2, float[] c0, float a0, float[] c1, float a1, float[] c2,
                float a2) {
            vertex(p0, c0, a0);
            vertex(p1, c1, a1);
            vertex(p2, c2, a2);
        }

        private void vertex(double[] p, float[] c, float a) {
            if (size + STRIDE > data.length) {
                data = java.util.Arrays.copyOf(data, data.length * 2);
            }
            data[size++] = (float) p[0];
            data[size++] = (float) p[1];
            data[size++] = (float) p[2];
            data[size++] = c[0];
            data[size++] = c[1];
            data[size++] = c[2];
            data[size++] = a;
        }

        float[] toArray() {
            return java.util.Arrays.copyOf(data, size);
        }
    }
}
