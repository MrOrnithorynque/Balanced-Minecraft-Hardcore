package github.mrornithorynque.bmh.client.sky;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

import org.joml.Quaternionf;
import org.joml.Vector3f;

/**
 * Vanilla's star field, rebuilt star for star. {@code LevelRenderer.drawStars} (1.21.1) makes 1500 tries from seed
 * 10842, keeps the points inside the unit ball and draws a small quad 100 blocks out in each kept direction. This class
 * runs the same random stream, the same float filter and the same JOML calls, so its star i is the sky's star i.
 *
 * No Minecraft class (JOML only): the offline sky previews can compile and run it as is.
 */
public final class StarField {

    /** Distance of the stars from the camera, in the sky's own units. */
    public static final double RADIUS = 100.0;

    /** One star, in the star field's own frame (before the sky's rotation). */
    public static final class Star {

        /** Index among the kept stars, in drawing order. */
        public final int index;
        /** Centre of the drawn quad, about {@link #RADIUS} from the camera. */
        public final double x, y, z;
        /** Vanilla's size parameter for the quad: 0.15 to 0.25. Bigger stars read better in the chain. */
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

    // Mirrors LevelRenderer.drawStars line for line, variable names included. Vanilla's vector3f.add(...) moves
    // vector3f itself, so each corner starts from the previous one: the quad is a diamond half a size beside the star's
    // direction, not a square on it. Same here.
    private static List<Star> build() {
        Random randomsource = new Random(10842L);
        List<Star> out = new ArrayList<>();

        for (int j = 0; j < 1500; j++) {
            float f1 = randomsource.nextFloat() * 2.0F - 1.0F;
            float f2 = randomsource.nextFloat() * 2.0F - 1.0F;
            float f3 = randomsource.nextFloat() * 2.0F - 1.0F;
            float f4 = 0.15F + randomsource.nextFloat() * 0.1F;
            float f5 = f1 * f1 + f2 * f2 + f3 * f3; // Mth.lengthSquared
            if (!(f5 <= 0.010000001F) && !(f5 >= 1.0F)) {
                Vector3f vector3f = new Vector3f(f1, f2, f3).normalize(100.0F);
                float f6 = (float) (randomsource.nextDouble() * (float) Math.PI * 2.0);
                Quaternionf quaternionf = new Quaternionf().rotateTo(new Vector3f(0.0F, 0.0F, -1.0F), vector3f).rotateZ(f6);
                double[][] corners = new double[4][];
                corners[0] = corner(vector3f.add(new Vector3f(f4, -f4, 0.0F).rotate(quaternionf)));
                corners[1] = corner(vector3f.add(new Vector3f(f4, f4, 0.0F).rotate(quaternionf)));
                corners[2] = corner(vector3f.add(new Vector3f(-f4, f4, 0.0F).rotate(quaternionf)));
                corners[3] = corner(vector3f.add(new Vector3f(-f4, -f4, 0.0F).rotate(quaternionf)));
                out.add(new Star(out.size(), f4, corners));
            }
        }

        return out;
    }

    private static double[] corner(Vector3f v) {
        return new double[] { v.x(), v.y(), v.z() };
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
