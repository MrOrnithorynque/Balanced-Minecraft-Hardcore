package github.mrornithorynque.bmh.client.sky;

import java.util.List;

/**
 * Picks the stars of an amulet reading: 5 to 7 real stars in a chain from high in the sky down to the horizon, in the
 * vertical plane of the reading's azimuth, as the sky stands at the given time of day. The last star, the lowest,
 * marks the direction.
 *
 * Plain Java on purpose (no Minecraft class): the offline sky previews compile and run it as is.
 */
public final class StarChain {

    // Search limits, in degrees. The first pass is the brief's ("about 10 degrees" off the plane); the later passes
    // only run where the sky has a hole along the plane, and each one gives a little more room.
    private static final double[][] PASSES = {
            // max off-plane, lowest top star, highest bottom star, max gap, max off-plane of the last star
            { 10.0, 50.0, 18.0, 20.0, 4.0 },
            { 12.0, 45.0, 20.0, 22.0, 6.0 },
            { 14.0, 40.0, 24.0, 26.0, 10.0 },
            { 18.0, 35.0, 28.0, 30.0, 18.0 },
    };
    private static final int MIN_STARS = 5;
    private static final int MAX_STARS = 7;
    /** Stars this close to the horizon hide behind the first hill. */
    private static final double MIN_ELEVATION = 5.0;
    /** The chain starts no higher, so that one look up toward home takes it all in (default FOV 70). */
    private static final double MAX_ELEVATION = 72.0;
    private static final double MIN_GAP = 6.0;
    private static final double PREFERRED_GAP = 11.0;
    private static final double OFF_PLANE_WEIGHT = 2.0;
    /** Per bend, times (1 - cos): 0.4 for a 30-degree bend, 1.5 for 60, 3 for a right angle. */
    private static final double BEND_WEIGHT = 3.0;
    private static final double LAST_STAR_WEIGHT = 4.0;

    private StarChain() {
    }

    /**
     * Where a star of the star field stands in the world at this time of day, as a unit vector (x east, y up, z
     * south). Vanilla draws the stars after YP(-90) then XP(timeOfDay * 360) on the camera's rotation.
     */
    public static double[] toWorld(double x, double y, double z, double timeOfDay) {
        double angle = Math.toRadians(timeOfDay * 360.0);
        double sin = Math.sin(angle);
        double cos = Math.cos(angle);
        double wx = -(y * sin + z * cos);
        double wy = y * cos - z * sin;
        double wz = x;
        double length = Math.sqrt(wx * wx + wy * wy + wz * wz);
        return new double[] { wx / length, wy / length, wz / length };
    }

    /**
     * The horizontal unit vector {x, z} toward an azimuth, in the amulet's convention (part C writes it): degrees
     * clockwise from north, 0 north (-Z), 90 east (+X).
     */
    public static double[] toward(double azimuth) {
        double a = Math.toRadians(azimuth);
        return new double[] { Math.sin(a), -Math.cos(a) };
    }

    /**
     * The chain for a reading toward {@code azimuth} (degrees clockwise from north, 0 north, 90 east) at
     * {@code timeOfDay} (Minecraft's 0 to 1 sky angle, 0.5 at midnight): indexes into {@code stars}, highest star
     * first. Empty if the sky has no usable star in that direction (never seen on the vanilla field, kept for safety).
     */
    public static int[] pick(List<StarField.Star> stars, double azimuth, double timeOfDay) {
        double[] toward = toward(azimuth);
        double hx = toward[0], hz = toward[1]; // toward the azimuth, on the horizon
        double nx = -hz, nz = hx;               // normal of the vertical plane through it

        int n = stars.size();
        double[][] dirs = new double[n][];
        double[] elevation = new double[n];
        double[] offPlane = new double[n];
        for (int i = 0; i < n; i++) {
            StarField.Star star = stars.get(i);
            double[] w = toWorld(star.x, star.y, star.z, timeOfDay);
            dirs[i] = w;
            elevation[i] = behindSunOrMoon(star) ? Double.NaN : Math.toDegrees(Math.atan2(w[1], w[0] * hx + w[2] * hz));
            offPlane[i] = Math.toDegrees(Math.asin(w[0] * nx + w[2] * nz));
        }

        for (double[] pass : PASSES) {
            int[] chain = search(stars, dirs, elevation, offPlane, pass[0], pass[1], pass[2], pass[3], pass[4]);
            if (chain.length > 0) {
                return chain;
            }
        }
        return new int[0];
    }

    /**
     * The sun and the moon turn with the stars: in the star field's frame the sun is a 60-block square at y = 100 and
     * the moon a 40-block square at y = -100. Vanilla adds the stars over them, so a star there always sits on a
     * bright disc, where a line would wash out. Same stars at every time of day; 3 blocks of margin.
     */
    static boolean behindSunOrMoon(StarField.Star star) {
        double y = Math.abs(star.y);
        double half = (star.y > 0.0 ? 30.0 + 3.0 : 20.0 + 3.0) / 100.0;
        return Math.abs(star.x) <= half * y && Math.abs(star.z) <= half * y;
    }

    // Dynamic programming over the candidates sorted from high to low: best[c][j][i] is the cheapest chain of c + 2
    // stars that starts high enough and ends on candidates j then i. Costs: small stars, distance from the plane (more
    // for the last star, the one that marks the direction), uneven gaps and bends.
    private static int[] search(List<StarField.Star> stars, double[][] dirs, double[] elevation, double[] offPlane,
            double maxOff, double minTop, double maxBottom, double maxGap, double maxLastOff) {

        int[] candidates = new int[stars.size()];
        int count = 0;
        for (int i = 0; i < stars.size(); i++) {
            double e = elevation[i];
            if (e >= MIN_ELEVATION && e <= MAX_ELEVATION && Math.abs(offPlane[i]) <= maxOff) {
                candidates[count++] = i;
            }
        }
        // Highest first (insertion sort: a few dozen candidates).
        for (int i = 1; i < count; i++) {
            int c = candidates[i];
            int j = i - 1;
            while (j >= 0 && elevation[candidates[j]] < elevation[c]) {
                candidates[j + 1] = candidates[j];
                j--;
            }
            candidates[j + 1] = c;
        }

        double[] starCost = new double[count];
        double[] off = new double[count], elev = new double[count];
        double[][] gapCost = new double[count][count];
        for (int i = 0; i < count; i++) {
            int star = candidates[i];
            off[i] = offPlane[star];
            elev[i] = elevation[star];
            double o = off[i] / maxOff;
            starCost[i] = (0.25 - stars.get(star).size) / 0.1 + OFF_PLANE_WEIGHT * o * o;
            for (int j = 0; j < i; j++) {
                double gap = Math.toDegrees(Math.acos(Math.min(1.0, dot(dirs[candidates[j]], dirs[star]))));
                double uneven = (gap - PREFERRED_GAP) / PREFERRED_GAP;
                boolean link = gap >= MIN_GAP && gap <= maxGap && elev[j] - elev[i] >= MIN_GAP * 0.5;
                gapCost[j][i] = link ? uneven * uneven : Double.POSITIVE_INFINITY;
            }
        }

        double[][][] best = new double[MAX_STARS - 1][count][count];
        int[][][] previous = new int[MAX_STARS - 1][count][count];
        for (double[][] layer : best) {
            for (double[] row : layer) {
                java.util.Arrays.fill(row, Double.POSITIVE_INFINITY);
            }
        }
        for (int j = 0; j < count; j++) {
            if (elev[j] >= minTop) {
                for (int i = j + 1; i < count; i++) {
                    best[0][j][i] = starCost[j] + gapCost[j][i] + starCost[i];
                }
            }
        }
        for (int c = 1; c < MAX_STARS - 1; c++) {
            for (int j = 0; j < count; j++) {
                for (int i = j + 1; i < count; i++) {
                    double sofar = best[c - 1][j][i];
                    if (sofar == Double.POSITIVE_INFINITY) {
                        continue;
                    }
                    double ux = off[i] - off[j], uy = elev[i] - elev[j];
                    double u = Math.sqrt(ux * ux + uy * uy);
                    for (int m = i + 1; m < count; m++) {
                        if (gapCost[i][m] == Double.POSITIVE_INFINITY) {
                            continue;
                        }
                        // The bend at star i, in the (off-plane, elevation) chart, which is close to true angles
                        // near the plane.
                        double vx = off[m] - off[i], vy = elev[m] - elev[i];
                        double bend = 1.0 - (ux * vx + uy * vy) / (u * Math.sqrt(vx * vx + vy * vy));
                        double cost = sofar + gapCost[i][m] + BEND_WEIGHT * bend + starCost[m];
                        if (cost < best[c][i][m]) {
                            best[c][i][m] = cost;
                            previous[c][i][m] = j;
                        }
                    }
                }
            }
        }

        int bestC = -1, bestJ = -1, bestI = -1;
        double bestScore = Double.POSITIVE_INFINITY;
        for (int c = MIN_STARS - 2; c < MAX_STARS - 1; c++) {
            for (int j = 0; j < count; j++) {
                for (int i = j + 1; i < count; i++) {
                    if (best[c][j][i] == Double.POSITIVE_INFINITY || elev[i] > maxBottom
                            || Math.abs(off[i]) > maxLastOff) {
                        continue;
                    }
                    double o = off[i] / maxOff;
                    double score = (best[c][j][i] + LAST_STAR_WEIGHT * o * o) / (c + 2);
                    if (score < bestScore) {
                        bestScore = score;
                        bestC = c;
                        bestJ = j;
                        bestI = i;
                    }
                }
            }
        }
        if (bestC < 0) {
            return new int[0];
        }

        int[] chain = new int[bestC + 2];
        for (int c = bestC, j = bestJ, i = bestI; c >= 0; c--) {
            chain[c + 1] = candidates[i];
            chain[c] = candidates[j];
            int above = previous[c][j][i];
            i = j;
            j = above;
        }
        return chain;
    }

    private static double dot(double[] a, double[] b) {
        return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    }
}
