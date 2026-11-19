// Area sky (owner: the sky agent). Log: docs/horizon/log/sky.md.
// The star reading of the Navigator's Amulet, drawn in the night sky, and the line name that the spyglass shows.
// Java reference: client/sky/StarField.java, StarChain.java, StarTrace.java, StarReadingRenderer.java and
// mixin/K_GuiSpyglassMixin.java. Horizon reference for the sky turn and the star brightness:
// horizon-client-render/src/environment.rs and the 26.3 data file data/minecraft/timeline/day.json.
//
// Every value here is plain data. The game module (bmh_sky.rs) reads the clock, the weather and the camera,
// and draws what this file answers.

use crate::contract::AmuletReading;
use std::sync::OnceLock;

// ---------------------------------------------------------------------------------------------------------
// The star field (StarField.java)
// ---------------------------------------------------------------------------------------------------------

/// The seed of vanilla's star field (StarField.java:67, `SkyRenderer.buildStars`).
pub const FIELD_SEED: i64 = 10842;
/// The tries of the star field (StarField.java:70).
pub const FIELD_TRIES: usize = 1500;

/// One star of the field, in the field's own frame (before the sky turns it). StarField.java:23-48.
#[derive(Debug, Clone, PartialEq)]
pub struct Star {
    /// The centre of the drawn quad, the mean of the four corners, about 100 from the camera.
    pub x: f64,
    pub y: f64,
    pub z: f64,
    /// Vanilla's size parameter: 0.15 to 0.25.
    pub size: f64,
    /// The four corners that vanilla draws.
    pub corners: [[f64; 3]; 4],
}

/// Minecraft's `SingleThreadedRandomSource`, the two calls that `buildStars` makes (StarField.java:98-121).
struct LegacyRandom {
    seed: i64,
}

impl LegacyRandom {
    const MULTIPLIER: i64 = 25_214_903_917;
    const MASK: i64 = 281_474_976_710_655;

    fn new(seed: i64) -> Self {
        Self { seed: (seed ^ Self::MULTIPLIER) & Self::MASK }
    }

    fn next(&mut self, bits: u32) -> i32 {
        self.seed = self.seed.wrapping_mul(Self::MULTIPLIER).wrapping_add(11) & Self::MASK;
        // Java `(int) (seed >> 48 - bits)`: the cast keeps the low 32 bits.
        (self.seed >> (48 - bits)) as i32
    }

    fn next_float(&mut self) -> f32 {
        self.next(24) as f32 * 5.960_464_5e-8_f32
    }

    fn next_double(&mut self) -> f64 {
        let i = i64::from(self.next(26));
        let j = i64::from(self.next(27));
        // Java multiplies by the float literal 1.110223E-16F, widened to a double.
        ((i << 27) + j) as f64 * f64::from(1.110_223e-16_f32)
    }
}

/// JOML `Math.invsqrt`: one over the square root, in floats.
fn invsqrt(r: f32) -> f32 {
    1.0 / (f64::from(r).sqrt() as f32)
}

/// JOML `Math.cosFromSin` (no fast maths): the cosine from the sine, with the sign of the angle's half turn.
fn cos_from_sin(sin: f32, angle: f32) -> f32 {
    let pi = std::f64::consts::PI;
    let (half_pi, two_pi, pi_f) = ((pi * 0.5) as f32, (pi * 2.0) as f32, pi as f32);
    let cos = f64::from(1.0_f32 - sin * sin).sqrt() as f32;
    let a = angle + half_pi;
    // Java `(int) (a / PI2_f)` cuts toward zero.
    let mut b = a - ((a / two_pi) as i32) as f32 * two_pi;
    if b < 0.0 {
        b += two_pi;
    }
    if b >= pi_f { -cos } else { cos }
}

/// Builds the star field as `SkyRenderer.buildStars` (26.1.2) does, with JOML's float maths
/// (StarField.java:66-91).
fn build() -> Vec<Star> {
    let mut random = LegacyRandom::new(FIELD_SEED);
    let mut out = Vec::new();
    for _ in 0..FIELD_TRIES {
        let x = random.next_float() * 2.0 - 1.0;
        let y = random.next_float() * 2.0 - 1.0;
        let z = random.next_float() * 2.0 - 1.0;
        let size = 0.15_f32 + random.next_float() * 0.1_f32;
        let length_sq = x * x + y * y + z * z;
        if length_sq <= 0.010_000_001_f32 || length_sq >= 1.0 {
            continue;
        }
        // `new Vector3f(x, y, z).normalize(100.0F)`.
        let scale = invsqrt(x * x + (y * y + z * z)) * 100.0;
        let centre = [x * scale, y * scale, z * scale];
        let z_rot = (random.next_double() * f64::from(std::f32::consts::PI) * 2.0) as f32;
        // `new Matrix3f().rotateTowards(-centre, up)`: the columns left, up and the direction.
        let dir = [-centre[0], -centre[1], -centre[2]];
        let inv_dir = invsqrt(dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]);
        let n = [dir[0] * inv_dir, dir[1] * inv_dir, dir[2] * inv_dir];
        // left = up x direction, with up = (0, 1, 0).
        let left = [n[2], 0.0_f32, -n[0]];
        let inv_left = invsqrt(left[0] * left[0] + left[1] * left[1] + left[2] * left[2]);
        let left = [left[0] * inv_left, left[1] * inv_left, left[2] * inv_left];
        let up = [
            n[1] * left[2] - n[2] * left[1],
            n[2] * left[0] - n[0] * left[2],
            n[0] * left[1] - n[1] * left[0],
        ];
        // `.rotateZ(-zRot)`: the first two columns turn about the third.
        let angle = -z_rot;
        let sin = f64::from(angle).sin() as f32;
        let cos = cos_from_sin(sin, angle);
        let col0 = [left[0] * cos + up[0] * sin, left[1] * cos + up[1] * sin, left[2] * cos + up[2] * sin];
        let col1 = [up[0] * cos - left[0] * sin, up[1] * cos - left[1] * sin, up[2] * cos - left[2] * sin];
        // `new Vector3f(a, b, 0).mul(rotation).add(centre)`.
        let corner = |a: f32, b: f32| -> [f64; 3] {
            [
                f64::from(col0[0] * a + col1[0] * b + centre[0]),
                f64::from(col0[1] * a + col1[1] * b + centre[1]),
                f64::from(col0[2] * a + col1[2] * b + centre[2]),
            ]
        };
        let corners = [corner(size, -size), corner(size, size), corner(-size, size), corner(-size, -size)];
        // StarField.java:38-46: the centre is the mean of the corners, added in order.
        let mut c = [0.0_f64; 3];
        for corner in &corners {
            for (sum, value) in c.iter_mut().zip(corner) {
                *sum += value / 4.0;
            }
        }
        out.push(Star { x: c[0], y: c[1], z: c[2], size: f64::from(size), corners });
    }
    out
}

/// Every star that vanilla draws, in drawing order. Built once (StarField.java:56-61).
pub fn field() -> &'static [Star] {
    static FIELD: OnceLock<Vec<Star>> = OnceLock::new();
    FIELD.get_or_init(build)
}

// ---------------------------------------------------------------------------------------------------------
// The chain (StarChain.java)
// ---------------------------------------------------------------------------------------------------------

/// Search passes, in degrees: max off-plane, lowest top star, highest bottom star, max gap, max off-plane of the
/// last star (StarChain.java:16-22).
const PASSES: [[f64; 5]; 4] = [
    [10.0, 50.0, 18.0, 20.0, 4.0],
    [12.0, 45.0, 20.0, 22.0, 6.0],
    [14.0, 40.0, 24.0, 26.0, 10.0],
    [18.0, 35.0, 28.0, 30.0, 18.0],
];
/// StarChain.java:23.
const MIN_STARS: usize = 5;
/// StarChain.java:24.
const MAX_STARS: usize = 7;
/// StarChain.java:26.
const MIN_ELEVATION: f64 = 5.0;
/// StarChain.java:28.
const MAX_ELEVATION: f64 = 72.0;
/// StarChain.java:29.
const MIN_GAP: f64 = 6.0;
/// StarChain.java:30.
const PREFERRED_GAP: f64 = 11.0;
/// StarChain.java:31.
const OFF_PLANE_WEIGHT: f64 = 2.0;
/// StarChain.java:33.
const BEND_WEIGHT: f64 = 3.0;
/// StarChain.java:34.
const LAST_STAR_WEIGHT: f64 = 4.0;

/// Where a star of the field stands in the world at this time of day, as a unit vector (x east, y up, z south).
/// The sky turns by `Ry(-90) * Rx(timeOfDay * 360)` (StarChain.java:44-53; Horizon game.rs:12232-12235).
pub fn to_world(x: f64, y: f64, z: f64, time_of_day: f64) -> [f64; 3] {
    let angle = (time_of_day * 360.0).to_radians();
    let (sin, cos) = angle.sin_cos();
    let wx = -(y * sin + z * cos);
    let wy = y * cos - z * sin;
    let wz = x;
    let length = (wx * wx + wy * wy + wz * wz).sqrt();
    [wx / length, wy / length, wz / length]
}

/// The horizontal unit vector {x, z} toward an azimuth: degrees clockwise from north, 0 north (-Z), 90 east (+X)
/// (StarChain.java:59-62, AmuletReading.java:63-74).
pub fn toward(azimuth: f64) -> [f64; 2] {
    let a = azimuth.to_radians();
    [a.sin(), -a.cos()]
}

/// The sun and the moon turn with the stars. A star on the bright disc of either is lost in it: the disc plus 3
/// blocks, half sizes 13 and 8 at 100 (StarChain.java:102-106).
pub fn behind_sun_or_moon(star: &Star) -> bool {
    let y = star.y.abs();
    let half = (if star.y > 0.0 { 13.0 } else { 8.0 }) / 100.0;
    star.x.abs() <= half * y && star.z.abs() <= half * y
}

fn dot(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// The chain for a reading toward `azimuth` at `time_of_day` (0 to 1, 0.5 at midnight): indexes into `stars`,
/// highest star first. Empty when the sky has no usable star in that direction (StarChain.java:69-93).
pub fn pick(stars: &[Star], azimuth: f64, time_of_day: f64) -> Vec<usize> {
    let [hx, hz] = toward(azimuth);
    // The normal of the vertical plane through the azimuth.
    let (nx, nz) = (-hz, hx);
    let mut dirs = Vec::with_capacity(stars.len());
    let mut elevation = Vec::with_capacity(stars.len());
    let mut off_plane = Vec::with_capacity(stars.len());
    for star in stars {
        let w = to_world(star.x, star.y, star.z, time_of_day);
        elevation.push(if behind_sun_or_moon(star) {
            f64::NAN
        } else {
            w[1].atan2(w[0] * hx + w[2] * hz).to_degrees()
        });
        off_plane.push((w[0] * nx + w[2] * nz).asin().to_degrees());
        dirs.push(w);
    }
    for pass in PASSES {
        let chain = search(stars, &dirs, &elevation, &off_plane, pass);
        if !chain.is_empty() {
            return chain;
        }
    }
    Vec::new()
}

/// One pass of the chain search: dynamic programming over the candidates sorted from high to low
/// (StarChain.java:111-224). `best[c][j][i]` is the cheapest chain of c + 2 stars that starts high enough and
/// ends on candidates j then i.
fn search(stars: &[Star], dirs: &[[f64; 3]], elevation: &[f64], off_plane: &[f64], pass: [f64; 5]) -> Vec<usize> {
    let [max_off, min_top, max_bottom, max_gap, max_last_off] = pass;
    let mut candidates: Vec<usize> = (0..stars.len())
        .filter(|&i| {
            let e = elevation[i];
            e >= MIN_ELEVATION && e <= MAX_ELEVATION && off_plane[i].abs() <= max_off
        })
        .collect();
    // Highest first, with the insertion sort of the Java code (equal elevations keep their order).
    for i in 1..candidates.len() {
        let c = candidates[i];
        let mut j = i;
        while j > 0 && elevation[candidates[j - 1]] < elevation[c] {
            candidates[j] = candidates[j - 1];
            j -= 1;
        }
        candidates[j] = c;
    }
    let count = candidates.len();
    let mut star_cost = vec![0.0; count];
    let mut off = vec![0.0; count];
    let mut elev = vec![0.0; count];
    let mut gap_cost = vec![vec![0.0; count]; count];
    for i in 0..count {
        let star = candidates[i];
        off[i] = off_plane[star];
        elev[i] = elevation[star];
        let o = off[i] / max_off;
        star_cost[i] = (0.25 - stars[star].size) / 0.1 + OFF_PLANE_WEIGHT * o * o;
        for j in 0..i {
            let gap = dot(&dirs[candidates[j]], &dirs[star]).min(1.0).acos().to_degrees();
            let uneven = (gap - PREFERRED_GAP) / PREFERRED_GAP;
            let link = gap >= MIN_GAP && gap <= max_gap && elev[j] - elev[i] >= MIN_GAP * 0.5;
            gap_cost[j][i] = if link { uneven * uneven } else { f64::INFINITY };
        }
    }

    let layers = MAX_STARS - 1;
    let mut best = vec![vec![vec![f64::INFINITY; count]; count]; layers];
    let mut previous = vec![vec![vec![0_usize; count]; count]; layers];
    for j in 0..count {
        if elev[j] >= min_top {
            for i in j + 1..count {
                best[0][j][i] = star_cost[j] + gap_cost[j][i] + star_cost[i];
            }
        }
    }
    for c in 1..layers {
        for j in 0..count {
            for i in j + 1..count {
                let so_far = best[c - 1][j][i];
                if so_far == f64::INFINITY {
                    continue;
                }
                let (ux, uy) = (off[i] - off[j], elev[i] - elev[j]);
                let u = (ux * ux + uy * uy).sqrt();
                for m in i + 1..count {
                    if gap_cost[i][m] == f64::INFINITY {
                        continue;
                    }
                    // The bend at star i, in the (off-plane, elevation) chart.
                    let (vx, vy) = (off[m] - off[i], elev[m] - elev[i]);
                    let bend = 1.0 - (ux * vx + uy * vy) / (u * (vx * vx + vy * vy).sqrt());
                    let cost = so_far + gap_cost[i][m] + BEND_WEIGHT * bend + star_cost[m];
                    if cost < best[c][i][m] {
                        best[c][i][m] = cost;
                        previous[c][i][m] = j;
                    }
                }
            }
        }
    }

    let mut found: Option<(usize, usize, usize)> = None;
    let mut best_score = f64::INFINITY;
    for c in MIN_STARS - 2..layers {
        for j in 0..count {
            for i in j + 1..count {
                if best[c][j][i] == f64::INFINITY || elev[i] > max_bottom || off[i].abs() > max_last_off {
                    continue;
                }
                let o = off[i] / max_off;
                let score = (best[c][j][i] + LAST_STAR_WEIGHT * o * o) / (c + 2) as f64;
                if score < best_score {
                    best_score = score;
                    found = Some((c, j, i));
                }
            }
        }
    }
    let Some((best_c, best_j, best_i)) = found else {
        return Vec::new();
    };
    let mut chain = vec![0; best_c + 2];
    let (mut j, mut i) = (best_j, best_i);
    for c in (0..=best_c).rev() {
        chain[c + 1] = candidates[i];
        chain[c] = candidates[j];
        let above = previous[c][j][i];
        i = j;
        j = above;
    }
    chain
}

// ---------------------------------------------------------------------------------------------------------
// The trace (StarTrace.java)
// ---------------------------------------------------------------------------------------------------------

/// StarTrace.java:14.
pub const SEGMENT_SECONDS: f64 = 0.6;
/// StarTrace.java:15.
pub const HOLD_SECONDS: f64 = 30.0;
/// StarTrace.java:16.
pub const FADE_SECONDS: f64 = 2.0;
/// StarTrace.java:17.
const STAR_FADE_IN_SECONDS: f64 = 0.3;
/// Floats per vertex in `triangles`: x, y, z, red, green, blue, alpha (StarTrace.java:20).
pub const STRIDE: usize = 7;

/// The core of the line, amethyst white (StarTrace.java:24).
const CORE: [f32; 3] = [0xf0 as f32 / 255.0, 0xe4 as f32 / 255.0, 0xff as f32 / 255.0];
/// The violet edge of the line (StarTrace.java:25).
const EDGE: [f32; 3] = [0xb5 as f32 / 255.0, 0x8c as f32 / 255.0, 0xff as f32 / 255.0];
/// The edge of the line toward the world spawn: half way to the core (StarTrace.java:27).
const SPAWN_EDGE: [f32; 3] =
    [(EDGE[0] + CORE[0]) / 2.0, (EDGE[1] + CORE[1]) / 2.0, (EDGE[2] + CORE[2]) / 2.0];
/// StarTrace.java:28.
const CORE_HALF_WIDTH: f64 = 0.10;
/// StarTrace.java:29.
const EDGE_HALF_WIDTH: f64 = 0.34;
/// StarTrace.java:30.
const GLOW_HALF_WIDTH: f64 = 0.9;
/// StarTrace.java:31.
const EDGE_ALPHA: f32 = 0.6;
/// StarTrace.java:32.
const GLOW_ALPHA: f32 = 0.14;
/// The line stops this far from each star (StarTrace.java:34).
const STAR_GAP: f64 = 0.6;
/// StarTrace.java:35.
const HALO_RADIUS: f64 = 1.1;
/// StarTrace.java:36.
const LAST_HALO_RADIUS: f64 = 1.6;
/// StarTrace.java:37.
const HALO_ALPHA: f32 = 0.55;
/// StarTrace.java:38.
const SPARK_RADIUS: f64 = 0.55;
/// StarTrace.java:39.
const HALO_SIDES: usize = 12;

/// Seconds from the first segment to the end of the fade, for a chain of `stars` stars (StarTrace.java:45-51).
pub fn duration(stars: usize) -> f64 {
    trace_seconds(stars) + HOLD_SECONDS + FADE_SECONDS
}

fn trace_seconds(stars: usize) -> f64 {
    stars.saturating_sub(1) as f64 * SEGMENT_SECONDS
}

/// Adds one vertex to `out` (StarTrace.java:179-190).
fn vertex(out: &mut Vec<f32>, p: &[f64; 3], c: &[f32; 3], a: f32) {
    out.extend_from_slice(&[p[0] as f32, p[1] as f32, p[2] as f32, c[0], c[1], c[2], a]);
}

#[allow(clippy::too_many_arguments)]
fn triangle(
    out: &mut Vec<f32>,
    p: [&[f64; 3]; 3],
    c0: &[f32; 3],
    a0: f32,
    c1: &[f32; 3],
    a1: f32,
    c2: &[f32; 3],
    a2: f32,
) {
    vertex(out, p[0], c0, a0);
    vertex(out, p[1], c1, a1);
    vertex(out, p[2], c2, a2);
}

/// Adds the triangles of one chain, `seconds` after the reading appeared, to `out` (STRIDE floats a vertex, in
/// the star field's frame). `sky_alpha` (0 to 1) dims everything; `lighter` gives the line toward the world spawn
/// its lighter edge (StarTrace.java:62-93). Indexes that are not in `stars` add nothing.
pub fn triangles(out: &mut Vec<f32>, stars: &[Star], chain: &[usize], seconds: f64, sky_alpha: f32, lighter: bool) {
    let edge = if lighter { &SPAWN_EDGE } else { &EDGE };
    let traced = trace_seconds(chain.len());
    let fade =
        if seconds <= traced + HOLD_SECONDS { 1.0 } else { 1.0 - (seconds - traced - HOLD_SECONDS) / FADE_SECONDS };
    let alpha = fade.clamp(0.0, 1.0) as f32 * sky_alpha;
    if alpha <= 0.0 || seconds < 0.0 || alpha.is_nan() {
        return;
    }
    for (i, &index) in chain.iter().enumerate() {
        let Some(star) = stars.get(index) else {
            continue;
        };
        let lit = ((seconds - i as f64 * SEGMENT_SECONDS) / STAR_FADE_IN_SECONDS).clamp(0.0, 1.0);
        if lit > 0.0 {
            let a = lit as f32 * alpha;
            // The star itself again, brighter: its own square, in the core colour.
            let [c0, c1, c2, c3] = &star.corners;
            triangle(out, [c0, c1, c2], &CORE, a * 0.9, &CORE, a * 0.9, &CORE, a * 0.9);
            triangle(out, [c0, c2, c3], &CORE, a * 0.9, &CORE, a * 0.9, &CORE, a * 0.9);
            let radius = if i + 1 == chain.len() { LAST_HALO_RADIUS } else { HALO_RADIUS };
            halo(out, [star.x, star.y, star.z], radius, &CORE, HALO_ALPHA * a);
        }
        if let Some(next) = chain.get(i + 1).and_then(|&n| stars.get(n)) {
            let progress = (seconds - i as f64 * SEGMENT_SECONDS) / SEGMENT_SECONDS;
            if progress > 0.0 {
                segment(out, star, next, progress.min(1.0), alpha, edge);
            }
        }
    }
}

/// A flat ribbon along the chord between two stars, facing the camera at the sphere's centre: a faint violet
/// glow, the violet edge, the white core (StarTrace.java:97-134).
fn segment(out: &mut Vec<f32>, from: &Star, to: &Star, progress: f64, alpha: f32, edge: &[f32; 3]) {
    let (dx, dy, dz) = (to.x - from.x, to.y - from.y, to.z - from.z);
    let length = (dx * dx + dy * dy + dz * dz).sqrt();
    if length <= 2.0 * STAR_GAP {
        return;
    }
    let (ux, uy, uz) = (dx / length, dy / length, dz / length);
    let start = STAR_GAP;
    let end = STAR_GAP + (length - 2.0 * STAR_GAP) * progress;
    let a = [from.x + ux * start, from.y + uy * start, from.z + uz * start];
    let b = [from.x + ux * end, from.y + uy * end, from.z + uz * end];

    // Sideways: across the chord and across the line of sight.
    let (mx, my, mz) = (from.x + to.x, from.y + to.y, from.z + to.z);
    let (sx, sy, sz) = (uy * mz - uz * my, uz * mx - ux * mz, ux * my - uy * mx);
    let s = (sx * sx + sy * sy + sz * sz).sqrt();
    let side = [sx / s, sy / s, sz / s];

    let offsets = [
        -GLOW_HALF_WIDTH,
        -EDGE_HALF_WIDTH,
        -CORE_HALF_WIDTH,
        0.0,
        CORE_HALF_WIDTH,
        EDGE_HALF_WIDTH,
        GLOW_HALF_WIDTH,
    ];
    let colours = [edge, edge, edge, &CORE, edge, edge, edge];
    let alphas =
        [0.0, GLOW_ALPHA * alpha, EDGE_ALPHA * alpha, alpha, EDGE_ALPHA * alpha, GLOW_ALPHA * alpha, 0.0];
    let shift = |p: &[f64; 3], by: f64| [p[0] + side[0] * by, p[1] + side[1] * by, p[2] + side[2] * by];
    for k in 0..offsets.len() - 1 {
        let a0 = shift(&a, offsets[k]);
        let a1 = shift(&a, offsets[k + 1]);
        let b0 = shift(&b, offsets[k]);
        let b1 = shift(&b, offsets[k + 1]);
        let (ck, cn) = (colours[k], colours[k + 1]);
        let (ak, an) = (alphas[k], alphas[k + 1]);
        triangle(out, [&a0, &b0, &b1], ck, ak, ck, ak, cn, an);
        triangle(out, [&a0, &b1, &a1], ck, ak, cn, an, cn, an);
    }

    if progress < 1.0 {
        halo(out, b, SPARK_RADIUS, &CORE, 0.8 * alpha);
    }
}

/// A soft disc around a point of the sphere, tangent to it: bright centre, transparent violet rim. Every triangle
/// winds counter-clockwise as seen from the camera at the sphere's centre (StarTrace.java:139-161).
fn halo(out: &mut Vec<f32>, centre: [f64; 3], radius: f64, colour: &[f32; 3], alpha: f32) {
    let [x, y, z] = centre;
    let r = (x * x + y * y + z * z).sqrt();
    let (nx, ny, nz) = (x / r, y / r, z / r);
    // Any vector that is not parallel to the normal gives the first axis of the tangent plane.
    let (px, py) = if ny.abs() < 0.9 { (0.0, 1.0) } else { (1.0, 0.0) };
    let (ex, ey, ez) = (py * nz, -px * nz, px * ny - py * nx);
    let e = (ex * ex + ey * ey + ez * ez).sqrt();
    let (ex, ey, ez) = (ex / e, ey / e, ez / e);
    let (fx, fy, fz) = (ny * ez - nz * ey, nz * ex - nx * ez, nx * ey - ny * ex);
    let mut previous: Option<[f64; 3]> = None;
    for k in 0..=HALO_SIDES {
        let angle = 2.0 * std::f64::consts::PI * k as f64 / HALO_SIDES as f64;
        let (cos, sin) = (angle.cos() * radius, angle.sin() * radius);
        let rim = [x + ex * cos + fx * sin, y + ey * cos + fy * sin, z + ez * cos + fz * sin];
        if let Some(previous) = &previous {
            triangle(out, [&centre, &rim, previous], colour, alpha, &EDGE, 0.0, &EDGE, 0.0);
        }
        previous = Some(rim);
    }
}

// ---------------------------------------------------------------------------------------------------------
// The sky turn and the star brightness (Horizon environment.rs, vanilla 26.3 timeline day.json)
// ---------------------------------------------------------------------------------------------------------

/// The length of the day timeline (day.json `period_ticks`).
const DAY_PERIOD: i64 = 24000;
/// The ease of the sun, moon and star angle tracks (day.json, `cubic_bezier`).
const ANGLE_EASE: [f32; 4] = [0.362, 0.241, 0.638, 0.759];
/// `minecraft:visual/star_angle` (day.json): 360 then 0 at tick 6000.
const STAR_ANGLE_KEYS: [(i64, f32); 2] = [(6000, 360.0), (6000, 0.0)];
/// `minecraft:visual/star_brightness` (day.json), linear.
const STAR_BRIGHTNESS_KEYS: [(i64, f32); 12] = [
    (92, 0.037),
    (627, 0.0),
    (11373, 0.0),
    (11732, 0.016),
    (11959, 0.044),
    (12399, 0.143),
    (12729, 0.258),
    (13228, 0.5),
    (22772, 0.5),
    (23032, 0.364),
    (23356, 0.225),
    (23758, 0.101),
];
/// Vanilla's star brightness at full night: a reading there is at full strength
/// (StarReadingRenderer.java:74).
const FULL_NIGHT_STARS: f32 = 0.5;

/// The ease of a timeline track (Horizon environment.rs:8-62, `KeyframeTrackSampler`).
#[derive(Clone, Copy)]
enum Ease {
    Linear,
    Bezier([f32; 4]),
}

impl Ease {
    /// environment.rs:14-61, in floats, the same steps.
    fn apply(self, x: f32) -> f32 {
        let Self::Bezier([x1, y1, x2, y2]) = self else {
            return x;
        };
        let curve = |c1: f32, c2: f32| (3.0 * c1 - 3.0 * c2 + 1.0, -6.0 * c1 + 3.0 * c2, 3.0 * c1);
        let (ax, bx, cx) = curve(x1, x2);
        let (ay, by, cy) = curve(y1, y2);
        let sample = |(a, b, c): (f32, f32, f32), t: f32| ((a * t + b) * t + c) * t;
        let gradient = |t: f32| (3.0 * ax * t + 2.0 * bx) * t + cx;
        let mut t = x;
        let mut solved = false;
        for _ in 0..4 {
            let error = sample((ax, bx, cx), t) - x;
            if error.abs() < 1e-5 {
                solved = true;
                break;
            }
            let g = gradient(t);
            if g < 1e-5 {
                break;
            }
            t -= (error / g).clamp(-0.25, 0.25);
        }
        if !solved {
            let (mut t0, mut t1) = (0.0_f32, 1.0_f32);
            while t0 < t1 {
                let error = sample((ax, bx, cx), t) - x;
                if error.abs() < 1e-5 {
                    break;
                }
                if error < 0.0 {
                    t0 = t;
                } else {
                    t1 = t;
                }
                let next = (t0 + t1) / 2.0;
                if next == t {
                    break;
                }
                t = next;
            }
        }
        sample((ay, by, cy), t)
    }
}

/// One value of a track at `ticks` (Horizon environment.rs:86-118, `Track::sample`).
fn sample(keys: &[(i64, f32)], ease: Ease, ticks: i64) -> f32 {
    let (Some(&first), Some(&last)) = (keys.first(), keys.last()) else {
        return 0.0;
    };
    if keys.len() == 1 {
        return first.1;
    }
    let t = ticks.rem_euclid(DAY_PERIOD);
    // (from value, from tick, to value, to tick): the wrap before the first key, each pair, the wrap after.
    let mut segments = vec![(last.1, last.0 - DAY_PERIOD, first.1, first.0)];
    segments.extend(keys.windows(2).filter_map(|pair| match pair {
        [a, b] => Some((a.1, a.0, b.1, b.0)),
        _ => None,
    }));
    let wrap = (last.1, last.0, first.1, first.0 + DAY_PERIOD);
    segments.push(wrap);
    let (from, from_t, to, to_t) = segments.iter().find(|s| t < s.3).copied().unwrap_or(wrap);
    if t <= from_t {
        return from;
    }
    if t >= to_t {
        return to;
    }
    let alpha = ease.apply((t - from_t) as f32 / (to_t - from_t) as f32);
    from + (to - from) * alpha
}

/// The star angle in degrees at the day time `day_time` (`minecraft:visual/star_angle`, environment.rs:288).
/// 0 at noon, 180 at midnight.
pub fn star_angle(day_time: i64) -> f32 {
    sample(&STAR_ANGLE_KEYS, Ease::Bezier(ANGLE_EASE), day_time)
}

/// The time of day that the chain takes: the star angle over 360 (StarReadingRenderer.java:262-266).
pub fn time_of_day(day_time: i64) -> f64 {
    f64::from(star_angle(day_time)) / 360.0
}

/// The star brightness at `day_time` with the rain and thunder levels (environment.rs:289, then `apply_weather`
/// at environment.rs:316-384: the rain layer counts the rain that the thunder does not cover).
pub fn star_brightness(day_time: i64, rain: f32, thunder: f32) -> f32 {
    let mut brightness = sample(&STAR_BRIGHTNESS_KEYS, Ease::Linear, day_time);
    for level in [rain - thunder, thunder] {
        if level > 0.0 {
            brightness += (0.0 - brightness) * level;
        }
    }
    brightness
}

/// The strength of a reading: `min(1, starBrightness / 0.5)` (StarReadingRenderer.java:142).
pub fn sky_alpha(star_brightness: f32) -> f32 {
    (star_brightness / FULL_NIGHT_STARS).min(1.0)
}

// ---------------------------------------------------------------------------------------------------------
// The reading on the client (StarReadingRenderer.java)
// ---------------------------------------------------------------------------------------------------------

/// A reading this old in game ticks when the client first sees it shows nothing (StarReadingRenderer.java:72).
pub const FRESH_TICKS: i64 = 40;
/// Through a spyglass, the last star of a line within this angle of the view's centre shows the line's name
/// (StarReadingRenderer.java:69).
pub const AIM_DEGREES: f64 = 0.8;

/// A log line of a started chain (StarReadingRenderer.java:276-278).
pub fn chain_log(azimuth: f32, stars: usize, last: &[f64; 3]) -> String {
    let last_azimuth = (last[0].atan2(-last[2]).to_degrees() + 360.0) % 360.0;
    format!(
        "[BalancedMcHardcoreMain] Star reading toward {}: {} stars, the last at azimuth {}, altitude {}",
        java_fixed1(f64::from(azimuth)),
        stars,
        java_fixed1(last_azimuth),
        java_fixed1(last[1].asin().to_degrees())
    )
}

/// Java `String.format("%.1f", value)`: the shortest decimal of the value, rounded half up to one digit after
/// the point (Java's Formatter rounds the digits of `Double.toString`).
fn java_fixed1(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_owned();
    }
    if value.is_infinite() {
        return if value > 0.0 { "Infinity" } else { "-Infinity" }.to_owned();
    }
    let sign = if value.is_sign_negative() { "-" } else { "" };
    // Rust prints the shortest decimal that reads back as the same value, with no exponent.
    let digits = format!("{}", value.abs());
    let (whole, fraction) = digits.split_once('.').unwrap_or((digits.as_str(), ""));
    let mut tenths = fraction.chars();
    let first = tenths.next().and_then(|c| c.to_digit(10)).unwrap_or(0);
    let second = tenths.next().and_then(|c| c.to_digit(10)).unwrap_or(0);
    let Ok(whole) = whole.parse::<u128>() else {
        return format!("{value:.1}");
    };
    let scaled = whole.saturating_mul(10).saturating_add(u128::from(first)) + u128::from(second >= 5);
    format!("{sign}{}.{}", scaled / 10, scaled % 10)
}

/// Java `Math.round` of a double: the floor of the value plus one half.
fn java_round(value: f64) -> i64 {
    (value + 0.5).floor() as i64
}

/// The self-check line of the azimuth convention, logged once (StarReadingRenderer.java:94-101).
pub fn self_check_log() -> String {
    let north = toward(0.0);
    let east = toward(90.0);
    format!(
        "[BalancedMcHardcoreMain] Sky azimuth self-check: from (0, 0), azimuth 0 aims at ({}, {}), azimuth 90 at \
         ({}, {}); expected (0, -100) and (100, 0)",
        java_round(north[0] * 100.0),
        java_round(north[1] * 100.0),
        java_round(east[0] * 100.0),
        java_round(east[1] * 100.0)
    )
}

/// The reading that shows: one chain per line, all started at the same moment (StarReadingRenderer.java:105).
#[derive(Debug, Clone, PartialEq)]
struct Shown {
    chains: Vec<Vec<usize>>,
    names: Vec<String>,
    /// The index in `chains` of the line toward the world spawn.
    spawn_chain: Option<usize>,
    /// The client clock at the start, in milliseconds.
    start_ms: i64,
}

/// What the sky shows on the client, and the reading that waits to start.
#[derive(Debug, Default)]
pub struct Sky {
    /// The game time of the last reading shown (Java `lastShownTime`). None before the first one.
    last_shown_time: Option<i64>,
    /// The newest reading that the server sent and that did not start yet.
    pending: Option<AmuletReading>,
    shown: Option<Shown>,
    /// The world and dimension of the last frame (Java `lastLevel`).
    level: Option<String>,
}

/// One frame of the sky: what the game module asks for and what it draws.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyView {
    /// The client clock, in milliseconds (the time base of the animation).
    pub now_ms: i64,
    /// The client's game time (`native_clock().game_time`).
    pub game_time: i64,
    /// The client's day time (`native_clock().day_time`).
    pub day_time: i64,
    /// The star brightness of the frame (`star_brightness`).
    pub star_brightness: f32,
    /// Horizon draws its stars this frame: the Overworld, a star brightness above 0.
    pub stars_drawn: bool,
}

/// What to draw this frame.
#[derive(Debug, Clone, PartialEq)]
pub struct SkyFrame {
    /// The triangles in the star field's frame, STRIDE floats a vertex.
    pub vertices: Vec<f32>,
    /// The star angle in degrees: the renderer turns the field by it.
    pub star_angle: f32,
}

impl Sky {
    /// An empty sky, for a `static`.
    pub const fn new() -> Self {
        Self { last_shown_time: None, pending: None, shown: None, level: None }
    }

    /// A new reading came from the server. With two readings before a start, the newest wins
    /// (StarReadingRenderer.java:169-171).
    pub fn hear(&mut self, reading: AmuletReading) {
        if self.pending.as_ref().is_none_or(|p| reading.time > p.time) {
            self.pending = Some(reading);
        }
    }

    /// The world or the dimension of this frame. Another one drops the shown reading and the last shown time
    /// (StarReadingRenderer.java:124-130). None outside a world.
    ///
    /// Java keeps the reading that waits: it is on the amulet stack, and it starts in the new level if it is still
    /// fresh. So a waiting reading stays here too, but not out of a world: Java has no player and no stack then.
    pub fn level(&mut self, level: Option<String>) {
        if self.level != level {
            let pending = if level.is_some() { self.pending.take() } else { None };
            *self = Self { level, pending, ..Self::default() };
        }
    }

    /// Starts a fresh reading if one waits, then gives the triangles of this frame. Java runs both only when
    /// vanilla draws the stars (D_LevelRendererSkyMixin.java:21-28). The second value is the log lines of the
    /// chains that started (StarReadingRenderer.java:268-281).
    pub fn frame(&mut self, view: &SkyView) -> (Option<SkyFrame>, Vec<String>) {
        let mut logs = Vec::new();
        if !view.stars_drawn {
            self.drop_stale(view.game_time);
            return (None, logs);
        }
        self.start(view, &mut logs);
        let Some(shown) = &self.shown else {
            return (None, logs);
        };
        let seconds = (view.now_ms - shown.start_ms) as f64 / 1000.0;
        let alpha = sky_alpha(view.star_brightness);
        let stars = field();
        let mut vertices = Vec::new();
        let mut any = false;
        for (c, chain) in shown.chains.iter().enumerate() {
            if seconds <= duration(chain.len()) {
                any = true;
                triangles(&mut vertices, stars, chain, seconds, alpha, shown.spawn_chain == Some(c));
            }
        }
        if !any {
            self.shown = None;
            return (None, logs);
        }
        let frame = (!vertices.is_empty()).then(|| SkyFrame { vertices, star_angle: star_angle(view.day_time) });
        (frame, logs)
    }

    /// A waiting reading that can no longer be fresh never starts (StarReadingRenderer.java:185).
    fn drop_stale(&mut self, game_time: i64) {
        if self.pending.as_ref().is_some_and(|p| game_time.saturating_sub(p.time) >= FRESH_TICKS) {
            self.pending = None;
        }
    }

    /// StarReadingRenderer.java:172-209: a reading starts when it is newer than the last one shown and within
    /// FRESH_TICKS of the client's game time.
    fn start(&mut self, view: &SkyView, logs: &mut Vec<String>) {
        self.drop_stale(view.game_time);
        let Some(reading) = self.pending.take() else {
            return;
        };
        let newer = self.last_shown_time.is_none_or(|last| reading.time > last);
        if !newer {
            return;
        }
        if view.game_time.saturating_sub(reading.time).saturating_abs() >= FRESH_TICKS {
            // Ahead of the client by FRESH_TICKS or more: it can still become fresh.
            self.pending = Some(reading);
            return;
        }
        self.last_shown_time = Some(reading.time);
        let time_of_day = time_of_day(view.day_time);
        let stars = field();
        let mut chains = Vec::new();
        let mut names = Vec::new();
        let mut spawn_chain = None;
        for (i, &azimuth) in reading.azimuths.iter().enumerate() {
            let chain = pick(stars, f64::from(azimuth), time_of_day);
            let Some(last) = chain.last().and_then(|&l| stars.get(l)) else {
                continue;
            };
            logs.push(chain_log(azimuth, chain.len(), &to_world(last.x, last.y, last.z, time_of_day)));
            if reading.spawn == Some(i) {
                spawn_chain = Some(chains.len());
            }
            chains.push(chain);
            names.push(reading.names.get(i).cloned().unwrap_or_default());
        }
        self.shown =
            (!chains.is_empty()).then_some(Shown { chains, names, spawn_chain, start_ms: view.now_ms });
    }

    /// The name of the line that the view's centre aims at, or None: the last star of a named line still in the
    /// sky within AIM_DEGREES of `look` (a unit vector), the nearest first (StarReadingRenderer.java:216-242).
    pub fn aimed_name(&self, now_ms: i64, day_time: i64, look: [f64; 3]) -> Option<&str> {
        let shown = self.shown.as_ref()?;
        let seconds = (now_ms - shown.start_ms) as f64 / 1000.0;
        let time_of_day = time_of_day(day_time);
        let stars = field();
        let mut best = AIM_DEGREES.to_radians().cos();
        let mut name = None;
        for (chain, line) in shown.chains.iter().zip(&shown.names) {
            if line.is_empty() || seconds > duration(chain.len()) {
                continue;
            }
            let Some(star) = chain.last().and_then(|&l| stars.get(l)) else {
                continue;
            };
            let sky = to_world(star.x, star.y, star.z, time_of_day);
            let d = dot(&sky, &look);
            if d > best {
                best = d;
                name = Some(line.as_str());
            }
        }
        name
    }
}

/// The view direction of a camera: Minecraft's yaw and pitch in degrees (`Camera.forwardVector`).
pub fn look(yaw: f32, pitch: f32) -> [f64; 3] {
    let (yaw, pitch) = (f64::from(yaw).to_radians(), f64::from(pitch).to_radians());
    [-yaw.sin() * pitch.cos(), -pitch.sin(), yaw.cos() * pitch.cos()]
}

// ---------------------------------------------------------------------------------------------------------
// The spyglass name (K_GuiSpyglassMixin.java)
// ---------------------------------------------------------------------------------------------------------

/// Amethyst, without its alpha (K_GuiSpyglassMixin.java:25).
pub const NAME_RGB: u32 = 0xB5_8CFF;
/// The name fades in over this time (K_GuiSpyglassMixin.java:27).
pub const FADE_IN_MILLIS: i64 = 2000;
/// Below this alpha the name is not drawn (K_GuiSpyglassMixin.java:29).
pub const MIN_ALPHA: i64 = 4;
/// The name sits this far under the centre of the view, in GUI pixels (K_GuiSpyglassMixin.java:57).
pub const NAME_BELOW_CENTRE: i32 = 16;
/// The fade of the aimed name (K_GuiSpyglassMixin.java:31-59).
#[derive(Debug, Default)]
pub struct NameFade {
    name: Option<String>,
    since_ms: i64,
}

/// A name to draw: its text, where its left end starts and the top, and its ARGB colour.
#[derive(Debug, Clone, PartialEq)]
pub struct NameDraw {
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub argb: u32,
}

impl NameFade {
    /// No name yet, for a `static`.
    pub const fn new() -> Self {
        Self { name: None, since_ms: 0 }
    }

    /// The name under the view this frame. A new name starts its fade again; no name hides it at once. Gives the
    /// alpha (0 to 255) to draw it with, or None below MIN_ALPHA (K_GuiSpyglassMixin.java:44-59).
    pub fn update(&mut self, name: Option<&str>, now_ms: i64) -> Option<i64> {
        if name.is_none() || name != self.name.as_deref() {
            self.name = name.map(str::to_owned);
            self.since_ms = now_ms;
        }
        name?;
        let alpha = 255 * (now_ms - self.since_ms).clamp(0, FADE_IN_MILLIS) / FADE_IN_MILLIS;
        (alpha >= MIN_ALPHA).then_some(alpha)
    }
}

/// Where the name draws: centred at half the GUI width, at half the GUI height plus 16. `width` is the text
/// width in GUI pixels (Java `centeredText`: `x - font.width(text) / 2`, whole numbers).
pub fn name_draw(text: &str, width: i32, gui: [i32; 2], alpha: i64) -> NameDraw {
    let argb = (alpha.clamp(0, 255) as u32) << 24 | NAME_RGB;
    NameDraw { text: text.to_owned(), x: gui[0] / 2 - width / 2, y: gui[1] / 2 + NAME_BELOW_CENTRE, argb }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reading(azimuths: &[f32], names: &[&str], spawn: Option<usize>, time: i64) -> AmuletReading {
        AmuletReading {
            azimuths: azimuths.to_vec(),
            names: names.iter().map(|n| (*n).to_owned()).collect(),
            spawn,
            time,
        }
    }

    fn night(now_ms: i64, game_time: i64) -> SkyView {
        SkyView { now_ms, game_time, day_time: 18000, star_brightness: 0.5, stars_drawn: true }
    }

    #[test]
    fn the_field_lies_on_the_sphere() {
        let stars = field();
        assert!((700..1500).contains(&stars.len()), "{}", stars.len());
        for star in stars {
            let r = (star.x * star.x + star.y * star.y + star.z * star.z).sqrt();
            assert!((99.9..100.1).contains(&r), "{r}");
            assert!((0.15..=0.25).contains(&star.size));
        }
    }

    // Values printed by the real Java classes (StarField, StarChain, StarTrace of common/src, JOML 1.10.5, Java 21),
    // with a small driver that is not kept: docs/horizon/log/sky.md, step 5.
    #[test]
    fn the_field_is_the_java_field() {
        let stars = field();
        assert_eq!(stars.len(), 780);
        let java = [
            (0, [-53.246864318847656, 69.92573547363281, 47.69865798950195], 0.1501842588186264),
            (1, [-96.8908920288086, -18.4666748046875, 16.466272354125977], 0.24897167086601257),
            (2, [-98.7992935180664, 9.65153694152832, 12.0643310546875], 0.19748930633068085),
            (500, [-70.81916809082031, 41.60963439941406, -57.037574768066406], 0.15372544527053833),
            (779, [-19.18501091003418, 92.13239288330078, -33.81653594970703], 0.19358836114406586),
        ];
        for (i, centre, size) in java {
            let s = &stars[i];
            assert!((s.x - centre[0]).abs() < 1e-9, "{i}: {}", s.x);
            assert!((s.y - centre[1]).abs() < 1e-9, "{i}: {}", s.y);
            assert!((s.z - centre[2]).abs() < 1e-9, "{i}: {}", s.z);
            assert!((s.size - size).abs() < 1e-12, "{i}: {}", s.size);
        }
        // Corners 0 and 2 of star 0 and star 779.
        let c = &stars[0].corners;
        assert!((c[0][0] + 53.22471237182617).abs() < 1e-5 && (c[0][1] - 70.05602264404297).abs() < 1e-5);
        assert!((c[2][2] - 47.86492919921875).abs() < 1e-5);
        let c = &stars[779].corners;
        assert!((c[0][0] + 18.9603328704834).abs() < 1e-5 && (c[2][1] - 92.14289855957031).abs() < 1e-5);
    }

    #[test]
    fn the_chains_are_the_java_chains() {
        let java: [(f64, f64, &[usize]); 8] = [
            (0.0, 0.5, &[332, 33, 724, 474, 659]),
            (90.0, 0.5, &[376, 705, 78, 413, 550]),
            (180.0, 0.5, &[163, 197, 627, 31, 476]),
            (270.0, 0.5, &[419, 684, 470, 248, 773, 311]),
            (33.3, 0.42, &[195, 776, 288, 481, 301, 667]),
            (250.0, 0.61, &[445, 771, 536, 151, 401, 712]),
            (123.4, 0.55, &[348, 619, 491, 719, 640]),
            (359.9, 0.47, &[12, 449, 477, 777, 659]),
        ];
        for (azimuth, time_of_day, chain) in java {
            assert_eq!(pick(field(), azimuth, time_of_day), chain, "{azimuth} at {time_of_day}");
        }
    }

    #[test]
    fn the_trace_is_the_java_trace() {
        let stars = field();
        let chain = pick(stars, 90.0, 0.5);
        // (seconds, lighter, float count, sum of value times (index % 7 + 1), alpha of the first vertex).
        let java = [
            (0.15, false, 798, 5400.93417, 0.359999985),
            (0.15, true, 798, 5440.69888, 0.359999985),
            (1.0, false, 1344, 17653.9066, 0.719999969),
            (1.0, true, 1344, 17733.4360, 0.719999969),
            (20.0, false, 2478, 57009.3271, 0.719999969),
            (20.0, true, 2478, 57168.3859, 0.719999969),
            (33.0, false, 2478, 56808.5335, 0.504000008),
            (33.0, true, 2478, 56967.5923, 0.504000008),
        ];
        for (seconds, lighter, len, sum, alpha) in java {
            let mut v = Vec::new();
            triangles(&mut v, stars, &chain, seconds, 0.8, lighter);
            assert_eq!(v.len(), len, "{seconds} {lighter}");
            let ours: f64 = v.iter().enumerate().map(|(i, x)| f64::from(*x) * ((i % 7) + 1) as f64).sum();
            assert!((ours - sum).abs() < 1e-2, "{seconds} {lighter}: {ours}");
            assert!((f64::from(v[6]) - alpha).abs() < 1e-7);
            // The first vertex: corner 0 of star 376, the core colour.
            assert!((v[0] - 1.756_141_19).abs() < 1e-5 && (v[1] + 79.037_185_7).abs() < 1e-4);
            assert!((v[3] - 0.941_176_474).abs() < 1e-7 && (v[4] - 0.894_117_653).abs() < 1e-7);
        }
    }

    #[test]
    fn star_quads_wind_counter_clockwise_from_the_centre() {
        // The sky pass culls back faces: a triangle shows when it winds counter-clockwise as seen from the
        // camera at the centre, that is, when its normal points to the centre.
        for star in field() {
            let [c0, c1, c2, c3] = &star.corners;
            assert!(faces_centre(c0, c1, c2) && faces_centre(c0, c2, c3));
        }
    }

    fn faces_centre(p0: &[f64; 3], p1: &[f64; 3], p2: &[f64; 3]) -> bool {
        let u = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let v = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
        dot(&n, p0) < 0.0
    }

    fn point(v: &[f32], i: usize) -> [f64; 3] {
        let at = i * STRIDE;
        [f64::from(v[at]), f64::from(v[at + 1]), f64::from(v[at + 2])]
    }

    #[test]
    fn every_trace_triangle_winds_counter_clockwise_from_the_centre() {
        let stars = field();
        for azimuth in [0.0, 47.5, 90.0, 133.0, 180.0, 226.0, 270.0, 315.0] {
            for time_of_day in [0.3, 0.5, 0.7] {
                let chain = pick(stars, azimuth, time_of_day);
                assert!(!chain.is_empty());
                for seconds in [0.1, 1.0, 2.2, 10.0, 33.0] {
                    for lighter in [false, true] {
                        let mut v = Vec::new();
                        triangles(&mut v, stars, &chain, seconds, 1.0, lighter);
                        assert_eq!(v.len() % (3 * STRIDE), 0);
                        for t in 0..v.len() / (3 * STRIDE) {
                            let (p0, p1, p2) = (point(&v, 3 * t), point(&v, 3 * t + 1), point(&v, 3 * t + 2));
                            assert!(faces_centre(&p0, &p1, &p2), "azimuth {azimuth}, triangle {t}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn to_world_is_the_sky_turn_of_horizon() {
        // Ry(-90) * Rx(a) applied to (x, y, z) is (-(y sin a + z cos a), y cos a - z sin a, x).
        let w = to_world(0.0, 100.0, 0.0, 0.0);
        assert!((w[0] - 0.0).abs() < 1e-12 && (w[1] - 1.0).abs() < 1e-12 && w[2].abs() < 1e-12);
        // A quarter turn (time of day 0.25): the field's +y goes to the world's -x (west).
        let w = to_world(0.0, 100.0, 0.0, 0.25);
        assert!((w[0] + 1.0).abs() < 1e-12 && w[1].abs() < 1e-12);
        // The field's +x is always the world's +z (south).
        let w = to_world(50.0, 0.0, 0.0, 0.37);
        assert!((w[2] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn azimuths_follow_the_amulet_convention() {
        let north = toward(0.0);
        assert!(north[0].abs() < 1e-12 && (north[1] + 1.0).abs() < 1e-12);
        let east = toward(90.0);
        assert!((east[0] - 1.0).abs() < 1e-12 && east[1].abs() < 1e-12);
        assert_eq!(
            self_check_log(),
            "[BalancedMcHardcoreMain] Sky azimuth self-check: from (0, 0), azimuth 0 aims at (0, -100), azimuth 90 \
             at (100, 0); expected (0, -100) and (100, 0)"
        );
    }

    #[test]
    fn a_chain_runs_down_toward_its_azimuth() {
        let stars = field();
        for azimuth in [0.0_f64, 90.0, 180.0, 270.0, 33.0] {
            let time_of_day = 0.5;
            let chain = pick(stars, azimuth, time_of_day);
            assert!((MIN_STARS..=MAX_STARS).contains(&chain.len()), "{azimuth}: {chain:?}");
            let [hx, hz] = toward(azimuth);
            let elevations: Vec<f64> = chain
                .iter()
                .map(|&i| {
                    let s = &stars[i];
                    let w = to_world(s.x, s.y, s.z, time_of_day);
                    w[1].atan2(w[0] * hx + w[2] * hz).to_degrees()
                })
                .collect();
            for pair in elevations.windows(2) {
                assert!(pair[0] - pair[1] >= MIN_GAP * 0.5, "{azimuth}: {elevations:?}");
            }
            let first = elevations.first().copied().unwrap_or(0.0);
            let last = elevations.last().copied().unwrap_or(90.0);
            assert!(first <= MAX_ELEVATION && last >= MIN_ELEVATION, "{azimuth}: {elevations:?}");
            // The last star is near the plane of the azimuth (the widest pass allows 18 degrees).
            let s = &stars[chain[chain.len() - 1]];
            let w = to_world(s.x, s.y, s.z, time_of_day);
            let off = (w[0] * -hz + w[2] * hx).asin().to_degrees();
            assert!(off.abs() <= 18.0, "{azimuth}: {off}");
        }
    }

    #[test]
    fn a_bad_azimuth_gives_no_chain() {
        assert!(pick(field(), f64::NAN, 0.5).is_empty());
        assert!(pick(field(), f64::INFINITY, 0.5).is_empty());
    }

    #[test]
    fn the_sun_and_moon_discs_hide_their_stars() {
        let on_sun = Star { x: 5.0, y: 99.0, z: -5.0, size: 0.2, corners: [[0.0; 3]; 4] };
        let off_sun = Star { x: 14.0, y: 99.0, z: 0.0, ..on_sun.clone() };
        let on_moon = Star { x: 7.0, y: -99.0, z: 7.0, ..on_sun.clone() };
        let off_moon = Star { x: 9.0, y: -99.0, z: 0.0, ..on_sun.clone() };
        assert!(behind_sun_or_moon(&on_sun));
        assert!(!behind_sun_or_moon(&off_sun));
        assert!(behind_sun_or_moon(&on_moon));
        assert!(!behind_sun_or_moon(&off_moon));
    }

    #[test]
    fn the_trace_times_are_the_java_times() {
        // 5 stars: 4 segments of 0.6 s, a hold of 30 s and a fade of 2 s.
        assert!((duration(5) - 34.4).abs() < 1e-9);
        assert!((duration(7) - 35.6).abs() < 1e-9);
        assert!((duration(0) - 32.0).abs() < 1e-9);
        let stars = field();
        let chain = pick(stars, 0.0, 0.5);
        let mut v = Vec::new();
        // Before the start and after the end: nothing.
        triangles(&mut v, stars, &chain, -0.1, 1.0, false);
        assert!(v.is_empty());
        triangles(&mut v, stars, &chain, duration(chain.len()) + 0.01, 1.0, false);
        assert!(v.is_empty());
        // No sky alpha: nothing.
        triangles(&mut v, stars, &chain, 5.0, 0.0, false);
        assert!(v.is_empty());
    }

    /// Counts what one frame draws: the first star at 0.15 s is lit half (lit = 0.5), with no segment yet.
    #[test]
    fn the_first_frame_is_one_lit_star() {
        let stars = field();
        let chain = pick(stars, 90.0, 0.5);
        let mut v = Vec::new();
        triangles(&mut v, stars, &chain, 0.15, 1.0, false);
        // 2 triangles for the square, 12 for the halo, no segment (progress 0.25 draws a ribbon: see below).
        // At 0.15 s the first segment has progress 0.25, so it draws 12 ribbon triangles and a 12 triangle spark.
        assert_eq!(v.len(), (2 + 12 + 12 + 12) * 3 * STRIDE);
        // The square's alpha: lit 0.5 times 0.9.
        assert!((v[6] - 0.45).abs() < 1e-6);
        // The colour is the core colour.
        assert!((v[3] - 240.0 / 255.0).abs() < 1e-6);
    }

    #[test]
    fn the_spawn_line_has_the_lighter_edge() {
        let stars = field();
        let chain = pick(stars, 200.0, 0.5);
        let (mut plain, mut lighter) = (Vec::new(), Vec::new());
        triangles(&mut plain, stars, &chain, 20.0, 1.0, false);
        triangles(&mut lighter, stars, &chain, 20.0, 1.0, true);
        assert_eq!(plain.len(), lighter.len());
        assert!(plain.chunks(STRIDE).any(|v| (v[3] - 0xb5 as f32 / 255.0).abs() < 1e-6));
        let half = (0xb5 as f32 + 0xf0 as f32) / 2.0 / 255.0;
        assert!(lighter.chunks(STRIDE).any(|v| (v[3] - half).abs() < 1e-6));
    }

    #[test]
    fn the_star_angle_is_the_26_3_timeline() {
        // Noon 0, midnight 180 (the bezier is symmetric about its middle), sunset and sunrise near 90 and 270.
        assert!(star_angle(6000).abs() < 1e-3);
        assert!((star_angle(18000) - 180.0).abs() < 1e-3);
        assert!((star_angle(18000 + 24000 * 3) - 180.0).abs() < 1e-3);
        assert!((star_angle(-6000) - 180.0).abs() < 1e-3);
        // The ease makes noon and midnight slower than linear: 3000 ticks after noon is less than 45 degrees.
        let a = star_angle(9000);
        assert!(a > 0.0 && a < 45.0, "{a}");
        // Symmetry: 3000 ticks before midnight and after it.
        assert!((star_angle(15000) + star_angle(21000) - 360.0).abs() < 1e-2);
        assert!((time_of_day(18000) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn the_star_brightness_is_the_26_3_timeline() {
        assert!((star_brightness(18000, 0.0, 0.0) - 0.5).abs() < 1e-6);
        assert_eq!(star_brightness(6000, 0.0, 0.0), 0.0);
        assert!((star_brightness(92, 0.0, 0.0) - 0.037).abs() < 1e-6);
        // Half way from 12399 (0.143) to 12729 (0.258).
        assert!((star_brightness(12564, 0.0, 0.0) - 0.2005).abs() < 1e-4);
        // Across the wrap: half way from 23758 (0.101) to 24092 (0.037).
        assert!((star_brightness(23925, 0.0, 0.0) - 0.069).abs() < 1e-3);
        // Rain halves it at level 0.5; thunder takes the rest of the rain.
        assert!((star_brightness(18000, 0.5, 0.0) - 0.25).abs() < 1e-6);
        assert!((star_brightness(18000, 1.0, 0.5) - 0.125).abs() < 1e-6);
        assert_eq!(star_brightness(18000, 1.0, 1.0), 0.0);
        assert_eq!(sky_alpha(0.5), 1.0);
        assert!((sky_alpha(0.25) - 0.5).abs() < 1e-6);
    }

    /// Runs one frame and says whether a reading started in it (it logs one line per chain).
    fn started(sky: &mut Sky, view: &SkyView) -> bool {
        !sky.frame(view).1.is_empty()
    }

    /// Runs one frame and says whether it draws.
    fn drawn(sky: &mut Sky, view: &SkyView) -> bool {
        sky.frame(view).0.is_some()
    }

    #[test]
    fn a_fresh_reading_starts_once() {
        let mut sky = Sky::default();
        sky.level(Some("world|minecraft:overworld".into()));
        sky.hear(reading(&[0.0, 90.0], &["Home", ""], Some(1), 1000));
        let (frame, logs) = sky.frame(&night(5000, 1001));
        // The start frame is at 0 seconds: nothing is lit yet (Java draws nothing then either).
        assert!(frame.is_none());
        assert_eq!(logs.len(), 2);
        assert!(logs[0].starts_with("[BalancedMcHardcoreMain] Star reading toward 0.0: "), "{}", logs[0]);
        assert!(drawn(&mut sky, &night(5100, 1003)));
        // The same reading again does not start again: it is not newer.
        sky.hear(reading(&[0.0], &["Home"], None, 1000));
        assert!(!started(&mut sky, &night(5200, 1004)));
        // It plays to the end, then stops.
        assert!(drawn(&mut sky, &night(5000 + 33_000, 1700)));
        let end = 5000 + (duration(MAX_STARS) * 1000.0) as i64 + 10;
        assert!(!drawn(&mut sky, &night(end, 2000)));
        assert!(!drawn(&mut sky, &night(5000, 2001)));
    }

    #[test]
    fn an_old_reading_never_starts() {
        let mut sky = Sky::default();
        sky.hear(reading(&[0.0], &["Home"], None, 1000));
        assert!(!started(&mut sky, &night(0, 1040)));
        // 39 ticks old still starts.
        sky.hear(reading(&[0.0], &["Home"], None, 1001));
        assert!(started(&mut sky, &night(0, 1040)));
    }

    #[test]
    fn a_reading_ahead_of_the_client_waits() {
        let mut sky = Sky::default();
        sky.hear(reading(&[0.0], &["Home"], None, 1100));
        assert!(!started(&mut sky, &night(0, 1000)));
        assert!(started(&mut sky, &night(10, 1061)));
    }

    #[test]
    fn the_newest_of_two_readings_wins() {
        let mut sky = Sky::default();
        sky.hear(reading(&[0.0], &["Old"], None, 1000));
        sky.hear(reading(&[90.0], &["New"], None, 1001));
        sky.hear(reading(&[180.0], &["Older"], None, 999));
        let (_, logs) = sky.frame(&night(0, 1001));
        assert_eq!(logs.len(), 1);
        assert!(logs[0].contains("toward 90.0"), "{}", logs[0]);
    }

    #[test]
    fn no_start_while_the_stars_do_not_show() {
        let mut sky = Sky::default();
        sky.hear(reading(&[0.0], &["Home"], None, 1000));
        assert!(!started(&mut sky, &SkyView { stars_drawn: false, ..night(0, 1001) }));
        // The rain stops within the 40 ticks: it starts then.
        assert!(started(&mut sky, &night(100, 1030)));
        // A shown reading draws nothing while the stars do not show.
        assert!(!drawn(&mut sky, &SkyView { stars_drawn: false, ..night(1000, 1031) }));
        assert!(drawn(&mut sky, &night(1100, 1032)));
        // Not after the 40 ticks.
        let mut sky = Sky::default();
        sky.hear(reading(&[0.0], &["Home"], None, 1000));
        assert!(!started(&mut sky, &SkyView { stars_drawn: false, ..night(0, 1045) }));
        assert!(!started(&mut sky, &night(100, 1046)));
    }

    #[test]
    fn another_level_drops_the_reading() {
        let mut sky = Sky::default();
        sky.level(Some("a|minecraft:overworld".into()));
        sky.hear(reading(&[0.0], &["Home"], None, 1000));
        assert!(started(&mut sky, &night(0, 1001)));
        assert!(drawn(&mut sky, &night(500, 1002)));
        sky.level(Some("a|minecraft:the_nether".into()));
        assert!(!drawn(&mut sky, &night(1000, 1003)));
        // The last shown time is gone too: the same time can show again in the new level.
        sky.hear(reading(&[0.0], &["Home"], None, 1000));
        assert!(started(&mut sky, &night(1100, 1004)));
        // The same level keeps it.
        sky.level(Some("a|minecraft:the_nether".into()));
        assert!(drawn(&mut sky, &night(1600, 1005)));
        // Out of a world: gone.
        sky.level(None);
        assert!(!drawn(&mut sky, &night(1700, 1006)));
    }

    #[test]
    fn another_level_keeps_the_waiting_reading() {
        // Java keeps the reading on the stack: it starts in the new level while it is fresh.
        let mut sky = Sky::default();
        sky.level(Some("a|minecraft:overworld".into()));
        sky.hear(reading(&[0.0], &["Home"], None, 1000));
        sky.level(Some("a|minecraft:the_nether".into()));
        sky.level(Some("a|minecraft:overworld".into()));
        assert!(started(&mut sky, &night(0, 1010)));
        // Out of a world, Java has no stack: the waiting reading goes.
        let mut sky = Sky::default();
        sky.level(Some("a|minecraft:overworld".into()));
        sky.hear(reading(&[0.0], &["Home"], None, 1000));
        sky.level(None);
        sky.level(Some("a|minecraft:overworld".into()));
        assert!(!started(&mut sky, &night(0, 1010)));
    }

    #[test]
    fn the_frame_fades_with_the_sky_alpha() {
        let mut sky = Sky::default();
        sky.hear(reading(&[0.0], &["Home"], None, 1000));
        assert!(started(&mut sky, &night(0, 1001)));
        let (full, _) = sky.frame(&night(1000, 1001));
        let (half, _) = sky.frame(&SkyView { star_brightness: 0.25, ..night(1000, 1001) });
        let (Some(full), Some(half)) = (full, half) else {
            panic!("no frame");
        };
        assert_eq!(full.vertices.len(), half.vertices.len());
        assert!((full.vertices[6] - 0.9).abs() < 1e-6);
        assert!((full.vertices[6] - 2.0 * half.vertices[6]).abs() < 1e-6);
        assert!((full.star_angle - 180.0).abs() < 1e-3);
    }

    #[test]
    fn the_spyglass_aims_at_the_last_named_star() {
        let mut sky = Sky::default();
        sky.hear(reading(&[0.0, 90.0, 180.0], &["Home", "Mine", ""], Some(2), 1000));
        assert!(started(&mut sky, &night(0, 1001)));
        let stars = field();
        let tod = time_of_day(18000);
        let last_of = |azimuth: f64| {
            let chain = pick(stars, azimuth, tod);
            let s = &stars[chain[chain.len() - 1]];
            to_world(s.x, s.y, s.z, tod)
        };
        assert_eq!(sky.aimed_name(100, 18000, last_of(0.0)), Some("Home"));
        assert_eq!(sky.aimed_name(100, 18000, last_of(90.0)), Some("Mine"));
        // The spawn line has no name.
        assert_eq!(sky.aimed_name(100, 18000, last_of(180.0)), None);
        // 1 degree off is too far (AIM_DEGREES 0.8).
        let v = last_of(0.0);
        let (sin, cos) = 1_f64.to_radians().sin_cos();
        let turned = [v[0] * cos + v[2] * sin, v[1], -v[0] * sin + v[2] * cos];
        assert_eq!(sky.aimed_name(100, 18000, turned), None);
        // After the end of the line: no name.
        assert_eq!(sky.aimed_name(40_000, 18000, last_of(0.0)), None);
    }

    #[test]
    fn the_look_vector_is_minecrafts() {
        let south = look(0.0, 0.0);
        assert!((south[2] - 1.0).abs() < 1e-12);
        let west = look(90.0, 0.0);
        assert!((west[0] + 1.0).abs() < 1e-12);
        let up = look(0.0, -90.0);
        assert!((up[1] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn the_name_fades_in_over_two_seconds() {
        let mut fade = NameFade::default();
        assert_eq!(fade.update(None, 0), None);
        // A new name starts at alpha 0: not drawn.
        assert_eq!(fade.update(Some("Home"), 100), None);
        // 255 * 31 / 2000 = 3 (below 4), 255 * 32 / 2000 = 4.
        assert_eq!(fade.update(Some("Home"), 131), None);
        assert_eq!(fade.update(Some("Home"), 132), Some(4));
        assert_eq!(fade.update(Some("Home"), 1100), Some(127));
        assert_eq!(fade.update(Some("Home"), 2100), Some(255));
        assert_eq!(fade.update(Some("Home"), 9000), Some(255));
        // Another name starts again.
        assert_eq!(fade.update(Some("Mine"), 9100), None);
        assert_eq!(fade.update(Some("Mine"), 10100), Some(127));
        // Away from the star: gone at once, and the next start is from 0.
        assert_eq!(fade.update(None, 10200), None);
        assert_eq!(fade.update(Some("Mine"), 10300), None);
    }

    #[test]
    fn the_name_sits_under_the_centre() {
        let draw = name_draw("Home", 25, [427, 240], 255);
        assert_eq!((draw.x, draw.y), (213 - 12, 136));
        assert_eq!(draw.argb, 0xFFB5_8CFF);
        assert_eq!(name_draw("Home", 24, [428, 241], 4).argb, 0x04B5_8CFF);
    }

    #[test]
    fn the_chain_log_line_is_the_java_line() {
        assert_eq!(
            chain_log(90.0, 6, &[1.0, 0.0, 0.0]),
            "[BalancedMcHardcoreMain] Star reading toward 90.0: 6 stars, the last at azimuth 90.0, altitude 0.0"
        );
        let up = 30_f64.to_radians();
        // Java rounds half up: 12.25 gives 12.3.
        assert_eq!(
            chain_log(12.25, 5, &[0.0, up.sin(), -up.cos()]),
            "[BalancedMcHardcoreMain] Star reading toward 12.3: 5 stars, the last at azimuth 0.0, altitude 30.0"
        );
    }

    #[test]
    fn fixed_one_rounds_as_java() {
        // Java formats the shortest decimal: 0.15 gives 0.2 (C would give 0.1), 0.05 gives 0.1.
        assert_eq!(java_fixed1(0.15), "0.2");
        assert_eq!(java_fixed1(0.05), "0.1");
        assert_eq!(java_fixed1(0.04), "0.0");
        assert_eq!(java_fixed1(-0.04), "-0.0");
        assert_eq!(java_fixed1(359.96), "360.0");
        assert_eq!(java_fixed1(12.0), "12.0");
        assert_eq!(java_fixed1(f64::from(0.1_f32)), "0.1");
        assert_eq!(java_fixed1(-7.25), "-7.3");
        assert_eq!(java_fixed1(f64::NAN), "NaN");
    }
}
