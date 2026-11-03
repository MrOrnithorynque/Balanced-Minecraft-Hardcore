package github.mrornithorynque.bmh.client.sky;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

/**
 * Vanilla's star field, rebuilt star for star. {@code LevelRenderer.drawStars} (1.20 to 1.20.4, identical bytecode)
 * makes 1500 tries from seed 10842, keeps the points inside the unit ball and draws a small square 100 blocks out in
 * each kept direction. This class runs the same random stream, the same filter and the same double arithmetic, so its
 * star i is the sky's star i.
 *
 * Plain Java on purpose (no Minecraft class): the offline sky previews compile and run it as is.
 */
public final class StarField {

    /** Distance of the stars from the camera, in the sky's own units. */
    public static final double RADIUS = 100.0;

    /** One star, in the star field's own frame (before the sky's rotation). */
    public static final class Star {

        /** Index among the kept stars, in drawing order. */
        public final int index;
        /** Centre of the drawn square, at {@link #RADIUS} from the camera. */
        public final double x, y, z;
        /** Half the side of the square: 0.15 to 0.25. Bigger stars read better in the chain. */
        public final double size;
        /** The four corners vanilla draws, as {x, y, z}. */
        public final double[][] corners;

        Star(int index, double size, double[][] corners) {
            this.index = index;
            this.size = size;
            this.corners = corners;
            double cx = 0.0, cy = 0.0, cz = 0.0;
            for (double[] corner : corners) {
                cx += corner[0] / 4.0;
                cy += corner[1] / 4.0;
                cz += corner[2] / 4.0;
            }
            this.x = cx;
            this.y = cy;
            this.z = cz;
        }
    }

    private static List<Star> stars;

    private StarField() {
    }

    /** Every star vanilla draws, in drawing order. Built once. */
    public static synchronized List<Star> stars() {
        if (stars == null) {
            stars = Collections.unmodifiableList(build());
        }
        return stars;
    }

    // Mirrors LevelRenderer.drawStars line for line, variable names included.
    private static List<Star> build() {
        Random randomsource = new Random(10842L);
        List<Star> out = new ArrayList<>();

        for (int i = 0; i < 1500; ++i) {
            double d0 = (double) (randomsource.nextFloat() * 2.0F - 1.0F);
            double d1 = (double) (randomsource.nextFloat() * 2.0F - 1.0F);
            double d2 = (double) (randomsource.nextFloat() * 2.0F - 1.0F);
            double d3 = (double) (0.15F + randomsource.nextFloat() * 0.1F);
            double d4 = d0 * d0 + d1 * d1 + d2 * d2;
            if (d4 < 1.0 && d4 > 0.01) {
                d4 = 1.0 / Math.sqrt(d4);
                d0 *= d4;
                d1 *= d4;
                d2 *= d4;
                double d5 = d0 * 100.0;
                double d6 = d1 * 100.0;
                double d7 = d2 * 100.0;
                double d8 = Math.atan2(d0, d2);
                double d9 = Math.sin(d8);
                double d10 = Math.cos(d8);
                double d11 = Math.atan2(Math.sqrt(d0 * d0 + d2 * d2), d1);
                double d12 = Math.sin(d11);
                double d13 = Math.cos(d11);
                double d14 = randomsource.nextDouble() * Math.PI * 2.0;
                double d15 = Math.sin(d14);
                double d16 = Math.cos(d14);
                double[][] corners = new double[4][];

                for (int j = 0; j < 4; ++j) {
                    double d18 = (double) ((j & 2) - 1) * d3;
                    double d19 = (double) ((j + 1 & 2) - 1) * d3;
                    double d21 = d18 * d16 - d19 * d15;
                    double d22 = d19 * d16 + d18 * d15;
                    double d23 = d21 * d12 + 0.0 * d13;
                    double d24 = 0.0 * d12 - d21 * d13;
                    double d25 = d24 * d9 - d22 * d10;
                    double d26 = d22 * d9 + d24 * d10;
                    corners[j] = new double[] { d5 + d25, d6 + d23, d7 + d26 };
                }

                out.add(new Star(out.size(), d3, corners));
            }
        }

        return out;
    }

    /** Minecraft's LegacyRandomSource (the generator behind RandomSource.create(seed)), the two calls drawStars makes. */
    static final class Random {

        private long seed;

        Random(long seed) {
            this.seed = (seed ^ 25214903917L) & 281474976710655L;
        }

        int next(int bits) {
            seed = seed * 25214903917L + 11L & 281474976710655L;
            return (int) (seed >> 48 - bits);
        }

        float nextFloat() {
            return (float) next(24) * 5.9604645E-8F;
        }

        double nextDouble() {
            int i = next(26);
            int j = next(27);
            long k = ((long) i << 27) + (long) j;
            return (double) k * 1.110223E-16F;
        }
    }
}
