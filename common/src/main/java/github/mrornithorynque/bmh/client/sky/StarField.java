package github.mrornithorynque.bmh.client.sky;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

import org.joml.Matrix3f;
import org.joml.Vector3f;

/**
 * Vanilla's star field, rebuilt star for star. {@code SkyRenderer.buildStars} (26.1.2) makes 1500 tries from seed
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

    // Mirrors SkyRenderer.buildStars (26.1.2) line for line, variable names included. Each corner is the star's
    // centre plus a rotated offset, so the quad is a square on the star's direction. 1.21.1 chained the corners
    // (vector3f.add moved vector3f itself), which put the quad half a size beside the direction.
    private static List<Star> build() {
        Random random = new Random(10842L);
        List<Star> out = new ArrayList<>();

        for (int i = 0; i < 1500; i++) {
            float x = random.nextFloat() * 2.0F - 1.0F;
            float y = random.nextFloat() * 2.0F - 1.0F;
            float z = random.nextFloat() * 2.0F - 1.0F;
            float starSize = 0.15F + random.nextFloat() * 0.1F;
            float lengthSq = x * x + y * y + z * z; // Mth.lengthSquared
            if (!(lengthSq <= 0.010000001F) && !(lengthSq >= 1.0F)) {
                Vector3f starCenter = new Vector3f(x, y, z).normalize(100.0F);
                float zRot = (float) (random.nextDouble() * (float) Math.PI * 2.0);
                Matrix3f rotation = new Matrix3f().rotateTowards(new Vector3f(starCenter).negate(), new Vector3f(0.0F, 1.0F, 0.0F))
                        .rotateZ(-zRot);
                double[][] corners = new double[4][];
                corners[0] = corner(new Vector3f(starSize, -starSize, 0.0F).mul(rotation).add(starCenter));
                corners[1] = corner(new Vector3f(starSize, starSize, 0.0F).mul(rotation).add(starCenter));
                corners[2] = corner(new Vector3f(-starSize, starSize, 0.0F).mul(rotation).add(starCenter));
                corners[3] = corner(new Vector3f(-starSize, -starSize, 0.0F).mul(rotation).add(starCenter));
                out.add(new Star(out.size(), starSize, corners));
            }
        }

        return out;
    }

    private static double[] corner(Vector3f v) {
        return new double[] { v.x(), v.y(), v.z() };
    }

    /** Minecraft's SingleThreadedRandomSource (RandomSource.createThreadLocalInstance(seed)), the two calls buildStars makes. */
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
