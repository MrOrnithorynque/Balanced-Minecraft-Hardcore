// Area journey (owner: the journey agent): the journey cinematic of a distant respawn, on the client.
// It holds the timeline (fade to black, night sky, star chain, message, pixel dissolve), what each frame
// draws, the sounds and the music fade, the death screen model that finds the Respawn click to hold, and
// the client state machine of the journey signals. Everything is pure: the game module `bmh_journey`
// reads the clock and the screen, asks this file, and draws or plays what it answers.
// Java reference: client/journey/JourneyOverlay.java (26.1), JourneyCinematic.java, JourneyMusic.java,
// mixin/F_LocalPlayerRespawnMixin.java, F_MinecraftOverlayMixin.java, F_GuiTitleMixin.java,
// handlers/JourneySignal.java.

use crate::wire::JourneySignal;

// The timeline, in milliseconds. FADE_OUT counts from the start, the others from the moment the
// screen is black.

/// The fade to black on the death screen (JourneyOverlay.java:46).
pub const FADE_OUT: i64 = 600;
/// Black only, before the sky comes in (JourneyOverlay.java:47).
pub const BLACK: i64 = 250;
/// The sky comes in over this time (JourneyOverlay.java:48).
pub const SKY_IN: i64 = 1200;
/// Home lights up (JourneyOverlay.java:49).
pub const HOME_AT: i64 = 700;
/// The line starts (JourneyOverlay.java:50).
pub const TRACE_AT: i64 = 1500;
/// The time of one segment of the line (JourneyOverlay.java:51).
pub const SEGMENT: i64 = 210;
/// The message may start (JourneyOverlay.java:52).
pub const MESSAGE_AT: i64 = 1700;
/// The message fades in (JourneyOverlay.java:53).
pub const MESSAGE_IN: i64 = 400;
/// The message stays (JourneyOverlay.java:54).
pub const MESSAGE_HOLD: i64 = 1600;
/// The message fades out (JourneyOverlay.java:55).
pub const MESSAGE_OUT: i64 = 400;
/// The last star shines alone for a moment before the sky breaks up (JourneyOverlay.java:57).
pub const LAST_STAR_BEAT: i64 = 400;
/// A star of the chain lights up over this time (JourneyOverlay.java:58).
pub const STAR_LIGHT_UP: i64 = 150;
/// The wait for the arrival signal after the respawn (JourneyOverlay.java:59).
pub const ARRIVAL_WAIT: i64 = 1000;
/// The player's chunk must be there this long (JourneyOverlay.java:60).
pub const TERRAIN_GRACE: i64 = 500;
/// The pixel dissolve (JourneyOverlay.java:61).
pub const DISSOLVE: i64 = 1000;
/// The journey stops waiting for the terrain this long after the respawn (JourneyOverlay.java:63).
pub const GIVE_UP: i64 = 30_000;
/// The journey stops waiting for the respawn this long after its start (JourneyOverlay.java:65).
pub const SERVER_WAIT: i64 = 150_000;

/// The art grid is twice as fine as the GUI, in whole screen pixels (JourneyOverlay.java:68).
pub const ART_RESOLUTION: f64 = 2.0;
/// The blocks of the dissolve across the screen height (JourneyOverlay.java:70).
pub const BLOCKS_HIGH: i32 = 72;

/// The top of the night gradient (JourneyOverlay.java:72).
pub const SKY_TOP: u32 = 0x0B_1433;
/// The bottom of the night gradient (JourneyOverlay.java:73).
pub const SKY_BOTTOM: u32 = 0x03_040B;
/// The colour of the message (JourneyOverlay.java:74).
pub const MESSAGE_COLOR: u32 = 0xFF_F1D0;
/// The colour of the Milky Way glow (JourneyOverlay.java:75).
pub const DUST_COLOR: u32 = 0x9A_A8D8;
/// The colours of the stars (JourneyOverlay.java:76).
pub const STAR_COLORS: [u32; 4] = [0xFF_FFFF, 0xCF_E0FF, 0xFF_F2CF, 0xBF_D4FF];
/// The core of the line, amethyst white (JourneyOverlay.java:78).
pub const LINE_CORE: u32 = 0xF0_E4FF;
/// The violet edge of the line (JourneyOverlay.java:79).
pub const LINE_EDGE: u32 = 0xB5_8CFF;
/// The 4x4 ordered dither of the dissolve edge (JourneyOverlay.java:82).
pub const BAYER: [i32; 16] = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];

/// The sky repeats every TILE screen heights across (JourneyOverlay.java:85).
pub const TILE: f64 = 3.0;
/// The stars of the two layers (JourneyOverlay.java:86).
pub const STAR_COUNT: usize = 1100;
/// The faint stars of the Milky Way band (JourneyOverlay.java:87).
pub const BAND_STAR_COUNT: usize = 360;
/// The blobs of the Milky Way glow (JourneyOverlay.java:88).
pub const DUST_COUNT: usize = 420;
/// How fast each star layer moves with the view (JourneyOverlay.java:90).
pub const LAYER_SPEED: [f64; 2] = [0.3, 0.6];
/// How fast the band moves with the view (JourneyOverlay.java:91).
pub const BAND_SPEED: f64 = 0.3;
/// The segments of the chain (JourneyOverlay.java:92).
pub const SEGMENTS: usize = 11;
/// Where Home is at the start, in screen widths (JourneyOverlay.java:94).
pub const HOME_SCREEN_X: f64 = 0.3;
/// Where the view keeps the head of the line, in screen widths (JourneyOverlay.java:95).
pub const HEAD_SCREEN_X: f64 = 0.55;
/// The line stops this many pixels from each star (JourneyOverlay.java:97).
pub const STAR_GAP: i32 = 8;

/// The line height of the font, which centres the message (JourneyOverlay.java:689, `font.lineHeight`).
pub const LINE_HEIGHT: i32 = 9;

/// The message key with the player name (JourneySignal.java:19, `MESSAGE`).
pub const MESSAGE_KEY: &str = "bmh.journey.message";
/// The text a language without the key shows (JourneySignal.java:40).
pub const MESSAGE_FALLBACK: &str = "A new journey begins for you, %s.";

/// The chime of each star (JourneyOverlay.java:407, `SoundEvents.AMETHYST_BLOCK_CHIME`).
pub const CHIME_SOUND: &str = "minecraft:block.amethyst_block.chime";
/// The long resonance of the last star (JourneyOverlay.java:405, `SoundEvents.AMETHYST_BLOCK_RESONATE`).
pub const RESONATE_SOUND: &str = "minecraft:block.amethyst_block.resonate";
/// The sky shatters (JourneyOverlay.java:302, `SoundEvents.AMETHYST_CLUSTER_BREAK`).
pub const BREAK_SOUND: &str = "minecraft:block.amethyst_cluster.break";

/// A sound event that no resource defines. The music hook asks for it once, as a track that replaces
/// the current one: that stops the game music at the end of the fade (JourneyMusic.java:45-48,
/// `stopPlaying`). Nothing plays for it.
pub const MUSIC_CUT: &str = "bmh:journey_music_cut";
/// The delays of the cut track, in ticks. A new song waits at least this long after the cut.
pub const MUSIC_CUT_DELAY: i32 = 24_000;

/// The death screen buttons are inactive this long after the death (vanilla `DeathScreen`, 20 ticks;
/// Horizon screens.rs:700-706, `died_at`).
pub const DEATH_DELAY: i64 = 1000;
/// The buttons of the quit confirmation are inactive this long (vanilla `ConfirmScreen.setDelay(20)`;
/// Horizon screens.rs:708-711).
pub const CONFIRM_DELAY: i64 = 1000;
/// The key of the confirmation title (Horizon screens.rs:178-184, `death_confirm_texts`).
pub const CONFIRM_TITLE_KEY: &str = "deathScreen.quit.confirm";

/// The GLFW index of the left mouse button: only it clicks the death screen (Horizon game.rs,
/// `Screen::Dead if pressed && button == MouseButton::Left`).
pub const LEFT_BUTTON: i32 = 0;
/// The GLFW bit of the shift key in `modifiers`.
pub const SHIFT: u32 = 1;

// Rounding and colours, as Java computes them.

/// Java `Math.round(float)`: the nearest integer, a half rounds up.
fn round_f(x: f32) -> i32 {
    let floor = x.floor();
    let rounded = if x - floor >= 0.5 { floor + 1.0 } else { floor };
    rounded as i32
}

/// Java `Math.round(double)`: the nearest integer, a half rounds up.
fn round_d(x: f64) -> i64 {
    let floor = x.floor();
    let rounded = if x - floor >= 0.5 { floor + 1.0 } else { floor };
    rounded as i64
}

/// `Mth.clamp(float)`.
fn clamp_f(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// `Mth.clamp(double)`.
fn clamp_d(value: f64, min: f64, max: f64) -> f64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// The smoothstep curve of the timeline: slow at the start and at the end (JourneyOverlay.java:714-718).
pub fn ease(t: f32) -> f32 {
    let t = clamp_f(t, 0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A colour with an alpha from 0 to 1 (JourneyOverlay.java:720-723).
pub fn argb(alpha: f32, rgb: u32) -> u32 {
    let byte = round_f(clamp_f(alpha, 0.0, 1.0) * 255.0) as u32;
    (byte << 24) | (rgb & 0xFF_FFFF)
}

/// The colour between two colours, channel by channel (JourneyOverlay.java:725-731).
pub fn lerp_color(from: u32, to: u32, t: f32) -> u32 {
    let channel = |shift: u32| {
        let a = ((from >> shift) & 0xFF) as f32;
        let b = ((to >> shift) & 0xFF) as f32;
        (round_f(a + t * (b - a)) as u32 & 0xFF) << shift
    };
    channel(16) | channel(8) | channel(0)
}

/// `java.util.Random`, so that the sky has the same distributions as the Java overlay. Only the calls
/// that the overlay makes are here.
#[derive(Debug, Clone)]
pub struct JavaRandom {
    seed: i64,
    next_gaussian: Option<f64>,
}

impl JavaRandom {
    const MULTIPLIER: i64 = 0x5_DEEC_E66D;
    const ADDEND: i64 = 0xB;
    const MASK: i64 = (1 << 48) - 1;

    /// `new Random(seed)`.
    pub fn new(seed: i64) -> Self {
        Self {
            seed: (seed ^ Self::MULTIPLIER) & Self::MASK,
            next_gaussian: None,
        }
    }

    /// `Random.next(bits)`.
    fn next(&mut self, bits: u32) -> i32 {
        self.seed = self.seed.wrapping_mul(Self::MULTIPLIER).wrapping_add(Self::ADDEND) & Self::MASK;
        (self.seed >> (48 - bits)) as i32
    }

    /// `Random.nextInt(bound)` for a bound above 0.
    pub fn next_int(&mut self, bound: i32) -> i32 {
        let mut r = self.next(31);
        let m = bound.wrapping_sub(1);
        if bound & m == 0 {
            return ((i64::from(bound) * i64::from(r)) >> 31) as i32;
        }
        let mut u = r;
        loop {
            r = u % bound;
            if u.wrapping_sub(r).wrapping_add(m) >= 0 {
                return r;
            }
            u = self.next(31);
        }
    }

    /// `Random.nextDouble()`.
    pub fn next_double(&mut self) -> f64 {
        let high = i64::from(self.next(26)) << 27;
        let low = i64::from(self.next(27));
        (high + low) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// `Random.nextFloat()`.
    pub fn next_float(&mut self) -> f32 {
        self.next(24) as f32 / (1 << 24) as f32
    }

    /// `Random.nextBoolean()`.
    pub fn next_boolean(&mut self) -> bool {
        self.next(1) != 0
    }

    /// `Random.nextGaussian()`, the polar method with its second value kept.
    pub fn next_gaussian(&mut self) -> f64 {
        if let Some(value) = self.next_gaussian.take() {
            return value;
        }
        loop {
            let v1 = 2.0 * self.next_double() - 1.0;
            let v2 = 2.0 * self.next_double() - 1.0;
            let s = v1 * v1 + v2 * v2;
            if s < 1.0 && s != 0.0 {
                let multiplier = (-2.0 * s.ln() / s).sqrt();
                self.next_gaussian = Some(v2 * multiplier);
                return v1 * multiplier;
            }
        }
    }
}

/// The art grid of one frame: one art pixel is `pixel` screen pixels, about half a GUI pixel
/// (JourneyOverlay.java:201-206).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Grid {
    /// Screen pixels per art pixel.
    pub pixel: i32,
    /// The width of the window in art pixels.
    pub width: i32,
    /// The height of the window in art pixels.
    pub height: i32,
}

/// The GUI scale of a window: its width in screen pixels over its width in GUI pixels. Horizon's GUI
/// width is the screen width over the scale, rounded up (`HudBatch::new`), so the quotient rounds back
/// to the scale. At least 1.
pub fn gui_scale(window_width: u32, gui_width: f32) -> f64 {
    if gui_width <= 0.0 {
        return 1.0;
    }
    (f64::from(window_width) / f64::from(gui_width)).round().max(1.0)
}

/// The art grid for a GUI scale and a window size in screen pixels (JourneyOverlay.java:202-206).
pub fn grid(gui_scale: f64, window: [u32; 2]) -> Grid {
    let pixel = i64::from((round_d(gui_scale / ART_RESOLUTION) as i32).max(1));
    let cells = |size: u32| i32::try_from((i64::from(size) + pixel - 1) / pixel).unwrap_or(i32::MAX);
    Grid {
        pixel: pixel as i32,
        width: cells(window[0]),
        height: cells(window[1]),
    }
}

/// One star of the night sky, at its place before the view moves (JourneyOverlay.java:869-901).
#[derive(Debug, Clone)]
struct Star {
    // In units of the screen height.
    u: f64,
    v: f64,
    // 0 for the far layer, 1 for the near one (a band star is on layer 0).
    layer: usize,
    brightness: f32,
    // 0 for a dot, 1 for a plus, 2 for a plus with long arms.
    arms: i32,
    twinkle: f32,
    color: u32,
    phase: f64,
    speed: f64,
    delay: i64,
}

impl Star {
    /// `layer` is 0 or 1 for the two layers, -1 for the band (JourneyOverlay.java:885-900).
    fn new(random: &mut JavaRandom, u: f64, v: f64, layer: i32) -> Self {
        let kind = random.next_float();
        let arms = if layer == 1 {
            if kind < 0.12 {
                2
            } else if kind < 0.45 {
                1
            } else {
                0
            }
        } else if layer == 0 && kind < 0.06 {
            1
        } else {
            0
        };
        let brightness = if arms > 0 {
            0.8 + random.next_float() * 0.2
        } else {
            (if layer < 0 { 0.15 } else { 0.2 }) + random.next_float() * 0.5
        };
        let index = random.next_int(STAR_COLORS.len() as i32);
        let color = usize::try_from(index)
            .ok()
            .and_then(|i| STAR_COLORS.get(i))
            .copied()
            .unwrap_or(STAR_COLORS[0]);
        let phase = random.next_double() * std::f64::consts::PI * 2.0;
        let speed = 0.0015 + random.next_double() * 0.0025;
        let delay = (random.next_double() * SKY_IN as f64 * 0.8) as i64;
        Self {
            u,
            v,
            layer: usize::from(layer == 1),
            brightness,
            arms,
            twinkle: if arms > 0 { 0.45 } else { 0.25 },
            color,
            phase,
            speed,
            delay,
        }
    }
}

/// One blob of the Milky Way glow (JourneyOverlay.java:904-919).
#[derive(Debug, Clone)]
struct Dust {
    u: f64,
    v: f64,
    // In units of a 240th of the screen height.
    size: i32,
    alpha: f32,
}

impl Dust {
    fn new(random: &mut JavaRandom) -> Self {
        let u = -0.1 + random.next_double() * (TILE + 0.1);
        let v = band_v(u) + random.next_gaussian() * 0.045;
        let size = 2 + random.next_int(5);
        let alpha = 0.02 + random.next_float() * 0.035;
        Self { u, v, size, alpha }
    }
}

/// The band of the Milky Way crosses the sky from high on the left to low on the right
/// (JourneyOverlay.java:709-712).
fn band_v(u: f64) -> f64 {
    0.1 + 0.24 * u / TILE
}

/// The sky of one journey: the stars, the band, the glow and the chain from Home to the new place, in
/// units of the screen height (JourneyOverlay.java:134-176).
#[derive(Debug, Clone)]
struct Sky {
    stars: Vec<Star>,
    band_stars: Vec<Star>,
    dust: Vec<Dust>,
    chain_u: [f64; SEGMENTS + 1],
    chain_v: [f64; SEGMENTS + 1],
    chain_length: [f64; SEGMENTS + 1],
}

impl Sky {
    /// The random calls come in the Java order: the stars, the band, the glow, then the chain.
    fn new(random: &mut JavaRandom) -> Self {
        let mut stars = Vec::with_capacity(STAR_COUNT);
        for i in 0..STAR_COUNT {
            let u = random.next_double() * TILE;
            let v = random.next_double();
            stars.push(Star::new(random, u, v, if i % 3 == 0 { 1 } else { 0 }));
        }
        let mut band_stars = Vec::with_capacity(BAND_STAR_COUNT);
        for _ in 0..BAND_STAR_COUNT {
            let along = -0.1 + random.next_double() * (TILE + 0.1);
            let v = band_v(along) + random.next_gaussian() * 0.06;
            band_stars.push(Star::new(random, along, v, -1));
        }
        let dust = (0..DUST_COUNT).map(|_| Dust::new(random)).collect();
        // The line zigzags: each segment turns up or down by 18 to 62 degrees, and it usually turns the
        // other way than the segment before (JourneyOverlay.java:152-176).
        let mut chain_u = [0.0; SEGMENTS + 1];
        let mut chain_v = [0.0; SEGMENTS + 1];
        let mut chain_length = [0.0; SEGMENTS + 1];
        chain_v[0] = 0.4 + random.next_double() * 0.15;
        let mut turn: i32 = if random.next_boolean() { 1 } else { -1 };
        for i in 1..=SEGMENTS {
            if random.next_float() < 0.7 {
                turn = -turn;
            }
            let length = 0.14 + random.next_double() * 0.16;
            let angle = (f64::from(turn) * (18.0 + random.next_double() * 44.0)).to_radians();
            let mut v = chain_v[i - 1] + length * angle.sin();
            // At the top or the bottom limit, the segment bounces back and the zigzag goes on.
            if !(0.22..=0.66).contains(&v) {
                v = if v < 0.22 { 0.44 - v } else { 1.32 - v };
                turn = if v < 0.44 { 1 } else { -1 };
            }
            chain_u[i] = chain_u[i - 1] + length * angle.cos();
            chain_v[i] = v;
            chain_length[i] = chain_length[i - 1] + (chain_u[i] - chain_u[i - 1]).hypot(chain_v[i] - chain_v[i - 1]);
        }
        Self {
            stars,
            band_stars,
            dust,
            chain_u,
            chain_v,
            chain_length,
        }
    }

    /// The full length of the chain.
    fn total(&self) -> f64 {
        self.chain_length[SEGMENTS]
    }

    /// How much of the chain the line has traced at `time` after the black, slow at the start and at the
    /// end (JourneyOverlay.java:423-427).
    fn traced_length(&self, time: i64) -> f64 {
        let progress = clamp_d((time - TRACE_AT) as f64 / (SEGMENTS as i64 * SEGMENT) as f64, 0.0, 1.0);
        self.total() * f64::from(ease(progress as f32))
    }

    /// How far the view has travelled, in pixels: it follows the head once the head passes it
    /// (JourneyOverlay.java:430-433).
    fn camera(&self, traced: f64, width: i32, height: i32) -> f64 {
        (HOME_SCREEN_X * f64::from(width) + self.u_at(traced) * f64::from(height) - HEAD_SCREEN_X * f64::from(width))
            .max(0.0)
    }

    /// The horizontal place of a point at this length along the chain (JourneyOverlay.java:436-446).
    fn u_at(&self, length: f64) -> f64 {
        for i in 0..SEGMENTS {
            if length <= self.chain_length[i + 1] {
                let part = (length - self.chain_length[i]) / (self.chain_length[i + 1] - self.chain_length[i]);
                return self.chain_u[i] + part * (self.chain_u[i + 1] - self.chain_u[i]);
            }
        }
        self.chain_u[SEGMENTS]
    }

    /// Star `i` of the chain on the screen, in art pixels (JourneyOverlay.java:448-452).
    fn chain_point(&self, i: usize, camera: f64, width: i32, height: i32) -> [i32; 2] {
        let i = i.min(SEGMENTS);
        [
            round_d(HOME_SCREEN_X * f64::from(width) + self.chain_u[i] * f64::from(height) - camera) as i32,
            round_d(self.chain_v[i] * f64::from(height)) as i32,
        ]
    }
}

/// One rectangle of the art, in art pixels: `x0 <= x1`, `y0 <= y1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fill {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
    pub argb: u32,
}

/// What a frame draws on, with the dissolve state for the blocks (JourneyOverlay.java:734-763).
struct Canvas {
    fills: Vec<Fill>,
    width: i32,
    height: i32,
    reveal: f32,
    cell: i32,
    dissolve: [i32; 2],
}

impl Canvas {
    fn new(width: i32, height: i32, reveal: f32, dissolve: [i32; 2]) -> Self {
        Self {
            fills: Vec::new(),
            width,
            height,
            reveal,
            cell: (height / BLOCKS_HIGH).max(4),
            dissolve,
        }
    }

    /// `ArtLayer.fill`: the corners are sorted.
    fn fill(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, argb: u32) {
        self.fills.push(Fill {
            x0: x0.min(x1),
            y0: y0.min(y1),
            x1: x0.max(x1),
            y1: y0.max(y1),
            argb,
        });
    }

    /// The block at (x, y) is revealed when the dissolve passes its threshold: near the last star first,
    /// with a dither on the edge (JourneyOverlay.java:695-706).
    fn revealed_block(&self, x: i32, y: i32) -> bool {
        let cell = self.cell;
        let dx = f64::from(x) + f64::from(cell) / 2.0 - f64::from(self.dissolve[0]);
        let dy = f64::from(y) + f64::from(cell) / 2.0 - f64::from(self.dissolve[1]);
        let far_x = f64::from(self.dissolve[0].max(self.width - self.dissolve[0]));
        let far_y = f64::from(self.dissolve[1].max(self.height - self.dissolve[1]));
        let radial = ((dx * dx + dy * dy).sqrt() / (far_x * far_x + far_y * far_y).sqrt()).min(1.0);
        let index = (((y / cell) & 3) * 4 + ((x / cell) & 3)) as usize;
        let dither = f64::from(BAYER.get(index).copied().unwrap_or(0)) / 16.0;
        f64::from(self.reveal) > 0.6 * radial + 0.4 * dither
    }

    /// True while the block of this pixel still shows the sky.
    fn covered(&self, x: i32, y: i32) -> bool {
        self.reveal <= 0.0 || !self.revealed_block(x - x.rem_euclid(self.cell), y - y.rem_euclid(self.cell))
    }

    fn pixel(&mut self, x: i32, y: i32, argb: u32) {
        if x >= 0 && y >= 0 && x < self.width && y < self.height && self.covered(x, y) {
            self.fill(x, y, x + 1, y + 1, argb);
        }
    }
}

/// The black, then the night gradient. During the dissolve, only the blocks that are not revealed yet
/// (JourneyOverlay.java:455-472). Horizon has no gradient helper (GAPS.md C15): the gradient is one fill
/// per run of art rows that share a colour, each row with the colour at its middle.
fn draw_sky(canvas: &mut Canvas, sky: f32) {
    let (width, height) = (canvas.width, canvas.height);
    if canvas.reveal <= 0.0 {
        canvas.fill(0, 0, width, height, 0xFF00_0000);
        let alpha = argb(sky, 0) & 0xFF00_0000;
        let mut band: Option<(i32, u32)> = None;
        for y in 0..height {
            let t = (y as f32 + 0.5) / height as f32;
            let color = alpha | lerp_color(SKY_TOP, SKY_BOTTOM, t);
            match band {
                Some((_, current)) if current == color => {}
                Some((start, current)) => {
                    canvas.fill(0, start, width, y, current);
                    band = Some((y, color));
                }
                None => band = Some((y, color)),
            }
        }
        if let Some((start, current)) = band {
            canvas.fill(0, start, width, height, current);
        }
        return;
    }
    let cell = canvas.cell;
    let mut y = 0;
    while y < height {
        let row = lerp_color(SKY_TOP, SKY_BOTTOM, y as f32 / height as f32);
        let color = 0xFF00_0000 | lerp_color(0x00_0000, row, sky);
        let mut x = 0;
        while x < width {
            if !canvas.revealed_block(x, y) {
                canvas.fill(x, y, (x + cell).min(width), (y + cell).min(height), color);
            }
            x += cell;
        }
        y += cell;
    }
}

/// The faint glow of the Milky Way: wide, dim blocks on the farthest layer (JourneyOverlay.java:475-497).
fn draw_dust(canvas: &mut Canvas, sky_ref: &Sky, sky: f32, camera: f64) {
    // The glow fades out as the sky breaks up: it has no blocks of its own to hide in.
    let alpha = sky * (1.0 - clamp_f(canvas.reveal * 5.0, 0.0, 1.0));
    if alpha <= 0.0 {
        return;
    }
    let height = canvas.height;
    let unit = (height / 240).max(1);
    for blob in &sky_ref.dust {
        let x = round_d(blob.u * f64::from(height) - BAND_SPEED * camera) as i32;
        let y = round_d(blob.v * f64::from(height)) as i32;
        let size = blob.size * unit;
        if x + size < 0 || x - size > canvas.width {
            continue;
        }
        canvas.fill(x - size / 2, y - size / 2, x + size / 2, y + size / 2, argb(alpha * blob.alpha, DUST_COLOR));
    }
}

/// The two star layers and the band (JourneyOverlay.java:499-515).
fn draw_stars(canvas: &mut Canvas, sky_ref: &Sky, now: i64, time: i64, sky: f32, camera: f64) {
    if sky <= 0.0 {
        return;
    }
    let height = f64::from(canvas.height);
    let tile = round_d(TILE * height).max(1);
    for star in &sky_ref.stars {
        let speed = LAYER_SPEED.get(star.layer).copied().unwrap_or(LAYER_SPEED[0]);
        let x = round_d(star.u * height - speed * camera).rem_euclid(tile) as i32;
        draw_star(canvas, star, x, now, time);
    }
    for star in &sky_ref.band_stars {
        let x = round_d(star.u * height - BAND_SPEED * camera) as i32;
        draw_star(canvas, star, x, now, time);
    }
}

/// One star: a pixel, and a plus for the bright ones (JourneyOverlay.java:517-539).
fn draw_star(canvas: &mut Canvas, star: &Star, x: i32, now: i64, time: i64) {
    let y = round_d(star.v * f64::from(canvas.height)) as i32;
    // Each star lights up on its own, while the sky comes in.
    let appear = clamp_f((time - BLACK - star.delay) as f32 / 300.0, 0.0, 1.0);
    if appear <= 0.0
        || x < 2
        || y < 2
        || x >= canvas.width - 2
        || y >= canvas.height - 2
        || !canvas.covered(x, y)
    {
        return;
    }
    let twinkle = 1.0 - star.twinkle * (0.5 + 0.5 * (star.phase + now as f64 * star.speed).sin() as f32);
    let alpha = star.brightness * twinkle * appear;
    canvas.pixel(x, y, argb(alpha, star.color));
    for arm in 1..=star.arms {
        let color = argb(alpha * if arm == 1 { 0.5 } else { 0.22 }, star.color);
        canvas.pixel(x - arm, y, color);
        canvas.pixel(x + arm, y, color);
        canvas.pixel(x, y - arm, color);
        canvas.pixel(x, y + arm, color);
    }
}

/// Home lights up first. Then the line goes from star to star, and each star brightens as the line
/// reaches it (JourneyOverlay.java:542-586).
fn draw_chain(canvas: &mut Canvas, sky_ref: &Sky, now: i64, time: i64, sky: f32, traced: f64, camera: f64) {
    if time < HOME_AT {
        return;
    }
    let (width, height) = (canvas.width, canvas.height);
    // Not dimmed by the sky: the chain must stand out while the sky is still coming in.
    let appear = clamp_f((time - HOME_AT) as f32 / STAR_LIGHT_UP as f32, 0.0, 1.0);
    let lengths = &sky_ref.chain_length;
    for i in 0..SEGMENTS {
        if traced <= lengths[i] {
            break;
        }
        let from = sky_ref.chain_point(i, camera, width, height);
        let to = sky_ref.chain_point(i + 1, camera, width, height);
        let part = ((traced - lengths[i]) / (lengths[i + 1] - lengths[i])).min(1.0);
        draw_segment(canvas, from, to, part, appear, part < 1.0);
    }
    let trace_end = TRACE_AT + SEGMENTS as i64 * SEGMENT;
    for i in 0..=SEGMENTS {
        let [x, y] = sky_ref.chain_point(i, camera, width, height);
        // 0 until the line reaches the star, then 1 within STAR_LIGHT_UP. Home is lit from the start. The
        // last star counts from the end of the trace.
        let reached = if i == SEGMENTS {
            (time - trace_end) as f64
        } else {
            (traced - lengths[i]) / sky_ref.total() * SEGMENTS as f64 * SEGMENT as f64
        };
        let lit = if i == 0 {
            appear
        } else {
            clamp_f((reached / STAR_LIGHT_UP as f64) as f32, 0.0, 1.0) * appear
        };
        if i == 0 {
            // Home pulses: a bright star in a double ring.
            let pulse = 0.5 + 0.5 * (now as f64 * 0.008).sin() as f32;
            draw_halo(canvas, x, y, 10 + round_f(pulse * 2.0), lit * (0.75 + 0.25 * pulse), LINE_EDGE);
            draw_halo(canvas, x, y, 16 + round_f(pulse * 3.0), lit * 0.3, LINE_EDGE);
            draw_knot(canvas, x, y, lit, 3);
        } else if lit > 0.0 {
            let last = i == SEGMENTS;
            draw_halo(canvas, x, y, if last { 12 } else { 7 }, lit, LINE_EDGE);
            let (radius, factor, color) = if last { (18, 0.35, LINE_EDGE) } else { (4, 0.5, LINE_CORE) };
            draw_halo(canvas, x, y, radius, lit * factor, color);
            draw_knot(canvas, x, y, lit, if last { 3 } else { 2 });
        } else {
            // A star the line has not reached yet: already a little brighter than the others.
            draw_knot(canvas, x, y, 0.5 * appear * sky, 1);
        }
    }
}

/// A star of the chain: a bright core with arms, and faint corners for the bigger ones
/// (JourneyOverlay.java:589-608).
fn draw_knot(canvas: &mut Canvas, x: i32, y: i32, alpha: f32, arms: i32) {
    canvas.pixel(x, y, argb(alpha, 0xFF_FFFF));
    for arm in 1..=arms {
        let color = argb(alpha * if arm == 1 { 0.8 } else { 0.4 }, LINE_CORE);
        canvas.pixel(x - arm, y, color);
        canvas.pixel(x + arm, y, color);
        canvas.pixel(x, y - arm, color);
        canvas.pixel(x, y + arm, color);
    }
    if arms > 1 {
        let corner = argb(alpha * 0.35, LINE_CORE);
        canvas.pixel(x - 1, y - 1, corner);
        canvas.pixel(x + 1, y - 1, corner);
        canvas.pixel(x - 1, y + 1, corner);
        canvas.pixel(x + 1, y + 1, corner);
    }
}

/// A pixel line with a violet edge and a faint glow, stopped short of both stars, and a spark at its
/// moving head (JourneyOverlay.java:611-643).
fn draw_segment(canvas: &mut Canvas, from: [i32; 2], to: [i32; 2], part: f64, alpha: f32, head: bool) {
    let dx = to[0] - from[0];
    let dy = to[1] - from[1];
    let steps = dx.abs().max(dy.abs());
    let flat = dx.abs() >= dy.abs();
    let last = round_d(f64::from(steps) * part) as i32;
    let mut spark = None;
    let mut k = STAR_GAP;
    while k <= last.min(steps - STAR_GAP) {
        let x = from[0] + round_f(dx as f32 * k as f32 / steps as f32);
        let y = from[1] + round_f(dy as f32 * k as f32 / steps as f32);
        spark = Some([x, y]);
        // The core, the violet edge in two steps, then a faint glow.
        for side in -3..=3 {
            let (px, py) = if flat { (x, y + side) } else { (x + side, y) };
            let color = match side.abs() {
                0 => argb(alpha, LINE_CORE),
                1 => argb(alpha * 0.6, LINE_EDGE),
                2 => argb(alpha * 0.3, LINE_EDGE),
                _ => argb(alpha * 0.12, LINE_EDGE),
            };
            canvas.pixel(px, py, color);
        }
        k += 1;
    }
    if let (true, Some([x, y])) = (head, spark) {
        for sx in -1..=1 {
            for sy in -1..=1 {
                let factor = if sx == 0 || sy == 0 { 1.0 } else { 0.5 };
                canvas.pixel(x + sx, y + sy, argb(alpha * factor, 0xFF_FFFF));
            }
        }
        draw_halo(canvas, x, y, 4, alpha * 0.8, LINE_EDGE);
    }
}

/// A ring of pixels around a star, with enough points to have no holes (JourneyOverlay.java:646-655).
fn draw_halo(canvas: &mut Canvas, x: i32, y: i32, radius: i32, alpha: f32, color: u32) {
    let radius_d = f64::from(radius);
    let points = ((radius_d * 2.0 * std::f64::consts::PI).ceil() as i32).max(8);
    let argb = argb(alpha * 0.55, color);
    for k in 0..points {
        let angle = std::f64::consts::PI * 2.0 * f64::from(k) / f64::from(points);
        let px = x + round_d(angle.cos() * radius_d) as i32;
        let py = y + round_d(angle.sin() * radius_d) as i32;
        canvas.pixel(px, py, argb);
    }
}

/// The message of a frame: the player name for `MESSAGE_KEY`, and the alpha of the text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub name: String,
    pub alpha: u8,
}

impl Message {
    /// The colour of the text: the alpha over `MESSAGE_COLOR` (JourneyOverlay.java:689).
    pub fn color(&self) -> u32 {
        (u32::from(self.alpha) << 24) | MESSAGE_COLOR
    }
}

/// Where the message goes, in GUI pixels (JourneyOverlay.java:681-690): a pose that moves to
/// `translate` and scales by `scale`, then the text at `at`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MessageLayout {
    pub translate: [f32; 2],
    pub scale: f32,
    pub at: [i32; 2],
}

/// The layout of a message `text_width` GUI pixels wide on a GUI `gui_width` by `gui_height`: twice the
/// size when it fits with 16 pixels on each side, under the chain at 80 % of the height.
pub fn message_layout(text_width: i32, gui_width: i32, gui_height: i32) -> MessageLayout {
    let scale = if text_width.saturating_mul(2) <= gui_width - 32 { 2.0 } else { 1.0 };
    MessageLayout {
        translate: [gui_width as f32 / 2.0, gui_height as f32 * 0.8],
        scale,
        at: [-text_width / 2, -LINE_HEIGHT / 2],
    }
}

/// What one frame draws (JourneyOverlay.java:198-248).
#[derive(Debug, Clone, PartialEq)]
pub struct Picture {
    /// The art grid the fills use.
    pub grid: Grid,
    /// During the fade: one fill over the whole GUI in this colour.
    pub fade: Option<u32>,
    /// The black and the night sky, in art pixels.
    pub sky: Vec<Fill>,
    /// The glow, the stars and the chain, in art pixels, drawn after `sky`.
    pub art: Vec<Fill>,
    /// The message, drawn last at GUI size.
    pub message: Option<Message>,
}

/// A sound to play at the player, for this player only.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sound {
    pub id: &'static str,
    pub volume: f32,
    pub pitch: f32,
}

/// What the client sees in one frame, as the game module reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct View {
    /// The clock, in milliseconds.
    pub now: i64,
    /// A world is joined, the same one as at the last frame.
    pub in_world: bool,
    /// The death screen is open.
    pub death_screen: bool,
    /// Another screen is up, which the game does not name: the respawn's loading screen, or a prompt,
    /// book, sign or pack reload over the death screen (Horizon `NativeScreenKind::Other`). The client
    /// cannot tell them apart, so such a frame changes nothing in the death screen state.
    pub covered: bool,
    /// A loading screen covers the world (Java `LevelLoadingScreen`).
    pub loading: bool,
    /// The chunk of the player is loaded on the client.
    pub chunk: bool,
    /// The art grid of the window.
    pub grid: Grid,
    /// Seeds the sky of a journey that this frame starts.
    pub seed: i64,
}

/// What the game module must do after a frame.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Commands {
    /// Send `RespawnRequest::Held` to the server.
    pub send_held: bool,
    /// Send the respawn request (`ServerboundClientCommandPacket` PERFORM_RESPAWN).
    pub send_respawn: bool,
    /// Keep the input held: open the mod's empty screen when no screen is open
    /// (JourneyOverlay.java:374-379, `HoldScreen`).
    pub hold: bool,
    /// Let the input go: close the mod's screen (JourneyOverlay.java:381-386).
    pub release: bool,
    /// The sounds to start, in order.
    pub sounds: Vec<Sound>,
}

/// What the music hook does in one client tick (JourneyMusic.java).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MusicAction {
    /// Leave the music as the game chose it.
    Keep,
    /// Multiply the volume by this part of 1.
    Scale(f32),
    /// Stop the track that plays now: ask for `MUSIC_CUT`, which replaces it.
    Cut,
    /// No music until the journey ends.
    Quiet,
}

/// One journey cinematic (Java `JourneyOverlay`).
#[derive(Debug, Clone)]
pub struct Journey {
    sky: Sky,
    start: i64,
    dark_start: i64,
    /// Java `deadPlayer != null`: the journey fades from the death screen and holds the respawn.
    faded: bool,
    /// The server said that the landing is ready (GAPS.md C4).
    ready: bool,
    respawn_sent: bool,
    respawned_at: Option<i64>,
    arrived: bool,
    /// Java `message`: the player name of the message, None when the rule hides it.
    name: Option<String>,
    message_start: Option<i64>,
    chunk_seen_at: Option<i64>,
    dissolve_start: Option<i64>,
    dissolve: [i32; 2],
    ambient_played: bool,
    /// The next star of the chain to chime, from Home (0) to the last one (SEGMENTS).
    next_chime: usize,
    music_cut: bool,
    /// The journey started in black for an exile that the server finishes later (an early respawn):
    /// the reveal waits for `Arrival`, or for ARRIVAL_WAIT after `Ready` when no arrival comes.
    awaits_arrival: bool,
    /// When this journey first saw `ready` and `arrived`.
    ready_at: Option<i64>,
    arrived_at: Option<i64>,
    /// The clock and the grid of the last update, which the picture uses.
    now: i64,
    grid: Grid,
}

impl Journey {
    /// A journey that starts in black at `now` and waits for the server's `Arrival`: the respawn went
    /// without the hold, and the server moves the player to the landing later or at once.
    pub fn waiting(now: i64, ready: bool, seed: i64) -> Self {
        Self {
            awaits_arrival: true,
            ..Self::new(now, false, ready, seed)
        }
    }

    /// A journey that starts at `now`. With `faded`, it fades from the death screen and the respawn
    /// waits for the black; without, it starts in black after the respawn (JourneyOverlay.java:126-177).
    pub fn new(now: i64, faded: bool, ready: bool, seed: i64) -> Self {
        let mut random = JavaRandom::new(seed);
        Self {
            sky: Sky::new(&mut random),
            start: now,
            dark_start: if faded { now + FADE_OUT } else { now },
            faded,
            ready,
            respawn_sent: !faded,
            respawned_at: None,
            arrived: false,
            name: None,
            message_start: None,
            chunk_seen_at: None,
            dissolve_start: None,
            dissolve: [0, 0],
            ambient_played: false,
            next_chime: 0,
            music_cut: false,
            awaits_arrival: false,
            ready_at: None,
            arrived_at: None,
            now,
            grid: Grid::default(),
        }
    }

    /// The server signal after the respawn. The name is None when the rule hides the message. Too late
    /// for a message once the sky breaks up (JourneyOverlay.java:180-188).
    pub fn arrive(&mut self, name: Option<String>) {
        self.arrived = true;
        if self.dissolve_start.is_none() {
            self.name = name;
        }
    }

    /// The server said that the landing is ready: the held respawn may go.
    pub fn set_ready(&mut self) {
        self.ready = true;
    }

    /// A new death and respawn came while this journey runs: it waits for the new arrival.
    fn wait_again(&mut self) {
        self.awaits_arrival = true;
        self.ready = false;
        self.ready_at = None;
        self.arrived = false;
        self.arrived_at = None;
    }

    /// Moves the timeline on (JourneyOverlay.java:251-312). `done` in the answer says that the journey
    /// is over and draws nothing more.
    pub fn update(&mut self, view: &View) -> (Commands, bool) {
        let now = view.now;
        self.now = now;
        self.grid = view.grid;
        if self.ready && self.ready_at.is_none() {
            self.ready_at = Some(now);
        }
        if self.arrived && self.arrived_at.is_none() {
            self.arrived_at = Some(now);
            if self.awaits_arrival {
                // The player moves to the landing now: the terrain test starts again there.
                self.chunk_seen_at = None;
            }
        }
        let mut commands = Commands::default();
        // The respawn waits for the black and, on Horizon, for the server's Ready (GAPS.md C4).
        if !self.respawn_sent && now >= self.dark_start && self.ready {
            self.respawn_sent = true;
            commands.send_respawn = true;
        }
        let respawned = self.respawn_sent && !view.death_screen;
        if respawned {
            if self.respawned_at.is_none() {
                self.respawned_at = Some(now);
            }
            commands.hold = self.dissolve_start.is_none();
            if !self.ambient_played && now >= self.dark_start + BLACK {
                self.ambient_played = true;
                // JourneyOverlay.java:282-285, `SoundInit.RESPAWN_SOUND` at 0.3.
                commands.sounds.push(Sound {
                    id: crate::respawn::RESPAWN_SOUND,
                    volume: 0.3,
                    pitch: 1.0,
                });
            }
        }
        if self.name.is_some() && self.message_start.is_none() && now >= self.dark_start + MESSAGE_AT {
            self.message_start = Some(now);
        }
        if let Some(sound) = self.chime(now - self.dark_start) {
            commands.sounds.push(sound);
        }
        if self.dissolve_start.is_none() && (self.ready_to_reveal(now, respawned, view) || self.gave_up(now)) {
            // The pixels break up from the last star of the chain.
            let (width, height) = (view.grid.width, view.grid.height);
            let camera = self.sky.camera(self.sky.total(), width, height);
            self.dissolve = self.sky.chain_point(SEGMENTS, camera, width, height);
            self.dissolve_start = Some(now);
            commands.release = true;
            // A journey that gave up before the server said Ready sends the respawn now, so the player
            // is not left dead under nothing (a difference from Java, which sent it at the black).
            if !self.respawn_sent {
                self.respawn_sent = true;
                commands.send_respawn = true;
            }
            // The sky shatters (JourneyOverlay.java:302).
            commands.sounds.push(Sound {
                id: BREAK_SOUND,
                volume: 0.6,
                pitch: 0.8,
            });
        }
        if self.dissolve_start.is_some_and(|start| now - start >= DISSOLVE) {
            // JourneyOverlay.java:305-308, `SoundInit.BELL_SOUND` at 1.0.
            commands.sounds.push(Sound {
                id: crate::respawn::BELL_SOUND,
                volume: 1.0,
                pitch: 1.0,
            });
            return (commands, true);
        }
        (commands, false)
    }

    /// The server may hold the respawn for a long time: the wait for the terrain counts from the
    /// respawn, not from the start (JourneyOverlay.java:315-318).
    fn gave_up(&self, now: i64) -> bool {
        if self.awaits_arrival {
            // The move to the landing is the respawn of an early exile: the search may take as long as a
            // held respawn, and the terrain wait counts from the arrival (or from Ready when none comes).
            return match self.arrived_at.or(self.ready_at) {
                Some(at) => now - at > GIVE_UP,
                None => now - self.start > SERVER_WAIT,
            };
        }
        match self.respawned_at {
            Some(at) => now - at > GIVE_UP,
            None => now - self.start > SERVER_WAIT,
        }
    }

    /// JourneyOverlay.java:320-347, in the Java order of the tests.
    fn ready_to_reveal(&mut self, now: i64, respawned: bool, view: &View) -> bool {
        if !respawned {
            return false;
        }
        // First, so that the grace for the terrain runs while the line is traced.
        let terrain = self.terrain_ready(now, view);
        if now < self.dark_start + TRACE_AT + SEGMENTS as i64 * SEGMENT + LAST_STAR_BEAT {
            return false;
        }
        // The arrival signal comes right after the respawn. Without it, go on without a message.
        if !self.arrived && self.respawned_at.is_some_and(|at| now - at < ARRIVAL_WAIT) {
            return false;
        }
        // An early exile: the server sends Arrival when it moves the player. Ready comes with it; Ready
        // with no Arrival in ARRIVAL_WAIT means that the player keeps the vanilla point.
        if self.awaits_arrival && !self.arrived && self.ready_at.is_none_or(|at| now - at < ARRIVAL_WAIT) {
            return false;
        }
        if self.message_start.is_some_and(|start| now < start + MESSAGE_IN + MESSAGE_HOLD + MESSAGE_OUT) {
            return false;
        }
        if self.name.is_some() && self.message_start.is_none() {
            return false;
        }
        terrain
    }

    /// No loading screen, and the player's chunk there for TERRAIN_GRACE (JourneyOverlay.java:352-371).
    fn terrain_ready(&mut self, now: i64, view: &View) -> bool {
        if view.loading {
            return false;
        }
        if !view.chunk {
            self.chunk_seen_at = None;
            return false;
        }
        let seen = *self.chunk_seen_at.get_or_insert(now);
        // The chunk arrives before its sections are built.
        now - seen >= TERRAIN_GRACE
    }

    /// A chime for Home, a higher one for each star the line reaches, and a long resonance for the last
    /// star. One star at most per frame (JourneyOverlay.java:390-412).
    fn chime(&mut self, time: i64) -> Option<Sound> {
        if self.next_chime > SEGMENTS || time < HOME_AT {
            return None;
        }
        let reached = self.next_chime == 0
            || if self.next_chime == SEGMENTS {
                time >= TRACE_AT + SEGMENTS as i64 * SEGMENT
            } else {
                self.sky.traced_length(time) >= self.sky.chain_length[self.next_chime]
            };
        if !reached {
            return None;
        }
        let sound = if self.next_chime == SEGMENTS {
            Sound {
                id: RESONATE_SOUND,
                volume: 1.0,
                pitch: 1.0,
            }
        } else {
            Sound {
                id: CHIME_SOUND,
                volume: if self.next_chime == 0 { 1.0 } else { 0.7 },
                pitch: 0.7 + 0.8 * self.next_chime as f32 / SEGMENTS as f32,
            }
        };
        self.next_chime += 1;
        Some(sound)
    }

    /// What the music hook does now (JourneyMusic.java:22-48). Only a journey that fades from the death
    /// screen fades the music (JourneyOverlay.java:264-266).
    pub fn music(&mut self, now: i64) -> MusicAction {
        if !self.faded {
            return MusicAction::Keep;
        }
        let part = 1.0 - ease((now - self.start) as f32 / FADE_OUT as f32);
        if part > 0.0 {
            MusicAction::Scale(part)
        } else if !self.music_cut {
            self.music_cut = true;
            MusicAction::Cut
        } else {
            MusicAction::Quiet
        }
    }

    /// The picture of the last update (JourneyOverlay.java:198-248).
    pub fn picture(&self) -> Picture {
        let now = self.now;
        let grid = self.grid;
        let mut picture = Picture {
            grid,
            fade: None,
            sky: Vec::new(),
            art: Vec::new(),
            message: None,
        };
        if now < self.dark_start {
            picture.fade = Some(argb(ease((now - self.start) as f32 / FADE_OUT as f32), 0x00_0000));
            return picture;
        }
        let time = now - self.dark_start;
        let sky = clamp_f((time - BLACK) as f32 / SKY_IN as f32, 0.0, 1.0);
        let reveal = self
            .dissolve_start
            .map_or(0.0, |start| ease((now - start) as f32 / DISSOLVE as f32));
        if grid.width > 0 && grid.height > 0 {
            let (sky_fills, art) = self.art(now, time, grid.width, grid.height, sky, reveal);
            picture.sky = sky_fills;
            picture.art = art;
        }
        picture.message = self.message(now);
        picture
    }

    /// The fills of the sky, then of the glow, the stars and the chain, in the Java order.
    fn art(&self, now: i64, time: i64, width: i32, height: i32, sky: f32, reveal: f32) -> (Vec<Fill>, Vec<Fill>) {
        let traced = self.sky.traced_length(time);
        let camera = self.sky.camera(traced, width, height);
        let mut canvas = Canvas::new(width, height, reveal, self.dissolve);
        draw_sky(&mut canvas, sky);
        let sky_fills = std::mem::take(&mut canvas.fills);
        draw_dust(&mut canvas, &self.sky, sky, camera);
        draw_stars(&mut canvas, &self.sky, now, time, sky, camera);
        draw_chain(&mut canvas, &self.sky, now, time, sky, traced, camera);
        (sky_fills, canvas.fills)
    }

    /// The message and its alpha, skipped below an alpha byte of 4 (JourneyOverlay.java:657-679).
    fn message(&self, now: i64) -> Option<Message> {
        let name = self.name.as_ref()?;
        let start = self.message_start?;
        let time = now - start;
        let alpha = if time < MESSAGE_IN {
            time as f32 / MESSAGE_IN as f32
        } else if time < MESSAGE_IN + MESSAGE_HOLD {
            1.0
        } else {
            1.0 - (time - MESSAGE_IN - MESSAGE_HOLD) as f32 / MESSAGE_OUT as f32
        };
        let byte = round_f(clamp_f(alpha, 0.0, 1.0) * 255.0);
        if byte < 4 {
            return None;
        }
        Some(Message {
            name: name.clone(),
            alpha: u8::try_from(byte).unwrap_or(u8::MAX),
        })
    }
}

/// The answer of the death screen model to a click or a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Press {
    /// The game handles it as usual.
    Pass,
    /// The press does nothing: the button is not active yet. The mod cancels it so that the game does
    /// not act on it either.
    Swallow,
    /// The press would respawn the player (Java: it calls `LocalPlayer.respawn`).
    Respawn,
}

/// A key of the death screen, from vanilla's key name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeathKey {
    Tab,
    Up,
    Down,
    Left,
    Right,
    /// Enter, the keypad's Enter or Space: the focused button.
    Activate,
    Escape,
    Other,
}

impl DeathKey {
    fn of(name: &str) -> Self {
        match name {
            "key.keyboard.tab" => Self::Tab,
            "key.keyboard.up" => Self::Up,
            "key.keyboard.down" => Self::Down,
            "key.keyboard.left" => Self::Left,
            "key.keyboard.right" => Self::Right,
            "key.keyboard.enter" | "key.keyboard.keypad.enter" | "key.keyboard.space" => Self::Activate,
            "key.keyboard.escape" => Self::Escape,
            _ => Self::Other,
        }
    }
}

/// A model of Horizon's death screen input (screens.rs:699-830): which button has the focus, and
/// whether the quit confirmation is open. It tells which click or key respawns, because Horizon gives
/// the mod no hook on the respawn itself (GAPS.md C4, substitute). Not hardcore: a hardcore world sends
/// no journey.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeathScreen {
    opened: i64,
    focus: Option<u8>,
    confirm: Option<i64>,
}

/// The Respawn and Title Screen buttons, 200 by 20 GUI pixels (Horizon screens.rs:185-198, vanilla
/// `DeathScreen.init`).
fn death_buttons(size: [f32; 2]) -> [[f32; 2]; 2] {
    let x = (size[0].ceil() / 2.0).floor() - 100.0;
    let y = (size[1].ceil() / 4.0).floor();
    [[x, y + 72.0], [x, y + 96.0]]
}

/// The yes and no buttons of the quit confirmation, 150 by 20, 4 apart, under a title `title_width`
/// wide with no message (Horizon confirm_screen.rs:120-150, `Choice::new` with two buttons).
fn confirm_buttons(size: [f32; 2], title_width: f32) -> [[f32; 2]; 2] {
    let row = 150.0 * 2.0 + 4.0;
    let column = title_width.max(0.0).max(row);
    let total = 9.0 + 8.0 + 8.0 + 36.0;
    let left = ((size[0] - column) / 2.0).trunc();
    let top = ((size[1] - total) / 2.0).trunc();
    let x = left + ((column - row) / 2.0).trunc();
    let y = top + 9.0 + 8.0 + 8.0 + 16.0;
    [[x, y], [x + 154.0, y]]
}

/// `ui::hovered`: the cursor is on the box.
fn on(cursor: [f32; 2], corner: [f32; 2], width: f32) -> bool {
    cursor[0] >= corner[0] && cursor[0] < corner[0] + width && cursor[1] >= corner[1] && cursor[1] < corner[1] + 20.0
}

/// Tab moves the focus between the two buttons (Horizon screens.rs:787-799).
fn tab(focus: Option<u8>, reverse: bool) -> Option<u8> {
    match focus {
        Some(0) => Some(1),
        Some(1) => Some(0),
        _ => Some(if reverse { 1 } else { 0 }),
    }
}

impl DeathScreen {
    /// The death screen opened at `now`.
    pub fn new(now: i64) -> Self {
        Self {
            opened: now,
            focus: None,
            confirm: None,
        }
    }

    /// A left click at `cursor` on a GUI of `size`. `title_width` is the width of the confirmation
    /// title (`CONFIRM_TITLE_KEY`) in GUI pixels.
    pub fn click(&mut self, now: i64, cursor: [f32; 2], size: [f32; 2], title_width: f32) -> Press {
        if let Some(opened) = self.confirm {
            let [yes, no] = confirm_buttons(size, title_width);
            let answer = if on(cursor, yes, 150.0) {
                Some(true)
            } else if on(cursor, no, 150.0) {
                Some(false)
            } else {
                None
            };
            return match answer {
                Some(_) if now - opened < CONFIRM_DELAY => Press::Swallow,
                // Yes leaves the world: the game does it.
                Some(true) => Press::Pass,
                Some(false) => Press::Respawn,
                None => Press::Pass,
            };
        }
        let [respawn, title] = death_buttons(size);
        let hit = if on(cursor, respawn, 200.0) {
            Some(0)
        } else if on(cursor, title, 200.0) {
            Some(1)
        } else {
            None
        };
        match hit {
            Some(_) if now - self.opened < DEATH_DELAY => Press::Swallow,
            Some(0) => {
                self.focus = Some(0);
                Press::Respawn
            }
            Some(_) => {
                // Title Screen asks first (Horizon screens.rs:736-741).
                self.focus = Some(1);
                self.confirm = Some(now);
                Press::Pass
            }
            None => Press::Pass,
        }
    }

    /// A key press, by vanilla's key name, with GLFW's modifier bits (Horizon screens.rs:745-830).
    pub fn key(&mut self, now: i64, name: &str, modifiers: u32) -> Press {
        let key = DeathKey::of(name);
        let reverse = modifiers & SHIFT != 0;
        if let Some(opened) = self.confirm {
            let acts = matches!(
                key,
                DeathKey::Escape | DeathKey::Tab | DeathKey::Left | DeathKey::Right | DeathKey::Activate
            );
            if acts && now - opened < CONFIRM_DELAY {
                return Press::Swallow;
            }
            return match key {
                // Escape answers no, and no respawns.
                DeathKey::Escape => Press::Respawn,
                DeathKey::Tab => {
                    self.focus = tab(self.focus, reverse);
                    Press::Pass
                }
                DeathKey::Left => {
                    self.focus = Some(0);
                    Press::Pass
                }
                DeathKey::Right => {
                    self.focus = Some(1);
                    Press::Pass
                }
                DeathKey::Activate if self.focus == Some(1) => Press::Respawn,
                _ => Press::Pass,
            };
        }
        let acts = matches!(key, DeathKey::Tab | DeathKey::Up | DeathKey::Down | DeathKey::Activate);
        if acts && now - self.opened < DEATH_DELAY {
            return Press::Swallow;
        }
        match key {
            DeathKey::Tab => {
                self.focus = tab(self.focus, reverse);
                Press::Pass
            }
            DeathKey::Up => {
                self.focus = Some(0);
                Press::Pass
            }
            DeathKey::Down => {
                self.focus = Some(1);
                Press::Pass
            }
            DeathKey::Activate => match self.focus {
                Some(0) => Press::Respawn,
                Some(_) => {
                    // Title Screen asks first, with the focus on yes.
                    self.confirm = Some(now);
                    self.focus = Some(0);
                    Press::Pass
                }
                None => Press::Pass,
            },
            _ => Press::Pass,
        }
    }
}

/// The client state of the journey (Java `JourneyCinematic` and the overlay it starts).
#[derive(Debug, Clone)]
pub struct Client {
    /// The server sent `Departure` while the death screen was up: the next Respawn press starts a
    /// journey. The arm belongs to this death (Java: a WeakReference to the dead LocalPlayer,
    /// JourneyCinematic.java:39 and :73): the press uses it, and the first frame that shows the world
    /// again turns it into a journey in black. So it never reaches a later death.
    armed: bool,
    /// The server sent `Ready` since the last `Departure`.
    ready: bool,
    journey: Option<Journey>,
    death: Option<DeathScreen>,
    send_held: bool,
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    /// No journey, nothing armed.
    pub const fn new() -> Self {
        Self {
            armed: false,
            ready: false,
            journey: None,
            death: None,
            send_held: false,
        }
    }

    /// The world was left or changed: everything ends (JourneyOverlay.java:257-261). True when a
    /// journey ran, so the module closes the hold screen.
    pub fn leave_world(&mut self) -> bool {
        let release = self.journey.is_some();
        *self = Self::new();
        release
    }

    /// A journey signal of the server (JourneyCinematic.java:30-59). `playing` is true when no screen
    /// covers the world now (the game's screen kind is Playing or Chat). `seed` seeds the sky of a
    /// journey that starts now.
    pub fn signal(&mut self, now: i64, signal: JourneySignal, playing: bool, seed: i64) {
        match signal {
            JourneySignal::Departure => {
                self.ready = false;
                if !playing {
                    // The death screen, or a screen that may cover it (Other): the next press is held,
                    // or the first frame of the world starts the journey in black.
                    self.armed = true;
                } else if let Some(journey) = &mut self.journey {
                    // A death and its respawn while a journey runs: that journey waits for the new arrival.
                    journey.wait_again();
                } else {
                    // The respawn went before Departure came (the immediate respawn rule, or a press
                    // before it): no death screen to fade from. Black at once, so the vanilla point does
                    // not show while the server finishes the exile (review journey-semantics major 1).
                    self.journey = Some(Journey::waiting(now, false, seed));
                }
            }
            JourneySignal::Ready => {
                self.ready = true;
                if let Some(journey) = &mut self.journey {
                    journey.set_ready();
                }
            }
            JourneySignal::Arrival { name } => {
                self.armed = false;
                self.ready = false;
                match &mut self.journey {
                    Some(journey) => journey.arrive(name),
                    None => {
                        // No fade from the death screen (the respawn went without the mod's hold): the
                        // cinematic starts in black.
                        let mut journey = Journey::new(now, false, true, seed);
                        journey.arrive(name);
                        self.journey = Some(journey);
                    }
                }
            }
        }
    }

    /// One frame. Ends the journey when the world is left (JourneyOverlay.java:257-261).
    pub fn frame(&mut self, view: &View) -> Commands {
        if !view.in_world {
            return Commands {
                release: self.leave_world(),
                ..Commands::default()
            };
        }
        if view.death_screen {
            if self.death.is_none() {
                self.death = Some(DeathScreen::new(view.now));
            }
        } else if !view.covered {
            // The world shows: the death screen is closed, and the player is alive again.
            self.death = None;
            if self.armed && self.journey.is_none() {
                // The death screen closed without the hold (a gamepad click, `horizon-ctl respawn`): the
                // cinematic starts in black now, before this frame is drawn, so the arrival place never
                // shows (review journey-semantics minor, the arrival frame).
                self.journey = Some(Journey::waiting(view.now, self.ready, view.seed));
            }
            self.armed = false;
        }
        let send_held = std::mem::take(&mut self.send_held);
        let mut commands = match &mut self.journey {
            Some(journey) => {
                let (commands, done) = journey.update(view);
                if done {
                    self.journey = None;
                }
                commands
            }
            None => Commands::default(),
        };
        commands.send_held = send_held;
        commands
    }

    /// True while a journey runs: it takes every click and key of the death screen and of the hold
    /// screen, as the Java overlay did.
    pub fn holds_input(&self) -> bool {
        self.journey.is_some()
    }

    /// A press that the death screen model says respawns starts the journey and is held
    /// (JourneyCinematic.java:62-87). Returns true when the press must be cancelled.
    fn press(&mut self, press: Press, now: i64, seed: i64) -> bool {
        if !self.armed {
            // No Departure yet. The buttons stay inactive for DEATH_DELAY, as vanilla's death screen
            // does and Horizon's remote one does not: `Departure` comes one server tick after the death
            // packets, so a fast press is not lost for the hold (review journey-semantics minor). After
            // that, no journey for this death: the game keeps its own behaviour.
            return press == Press::Swallow;
        }
        match press {
            Press::Pass => false,
            Press::Swallow => true,
            Press::Respawn => {
                self.armed = false;
                self.journey = Some(Journey::new(now, true, self.ready, seed));
                self.send_held = true;
                true
            }
        }
    }

    /// A mouse press on the death screen. Returns true when the press must be cancelled.
    pub fn death_click(
        &mut self,
        now: i64,
        button: i32,
        cursor: [f32; 2],
        size: [f32; 2],
        title_width: f32,
        seed: i64,
    ) -> bool {
        if self.journey.is_some() {
            return true;
        }
        if button != LEFT_BUTTON {
            return false;
        }
        let press = self
            .death
            .get_or_insert_with(|| DeathScreen::new(now))
            .click(now, cursor, size, title_width);
        self.press(press, now, seed)
    }

    /// A key press on the death screen. Returns true when the press must be cancelled.
    pub fn death_key(&mut self, now: i64, name: &str, modifiers: u32, seed: i64) -> bool {
        if self.journey.is_some() {
            return true;
        }
        let press = self
            .death
            .get_or_insert_with(|| DeathScreen::new(now))
            .key(now, name, modifiers);
        self.press(press, now, seed)
    }

    /// The picture of the running journey, if one runs.
    pub fn picture(&self) -> Option<Picture> {
        self.journey.as_ref().map(Journey::picture)
    }

    /// What the music hook does now.
    pub fn music(&mut self, now: i64) -> MusicAction {
        self.journey.as_mut().map_or(MusicAction::Keep, |journey| journey.music(now))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Reference values: the Java code of JourneyOverlay copied into a plain Java program (the Minecraft
    // types taken out) and run on OpenJDK 21: docs/horizon/log/journey-ref/JourneyRef.java. Its output,
    // out.txt next to it, holds the values below.

    fn hash(fills: &[Fill]) -> i64 {
        let mut h: i64 = 17;
        for f in fills {
            for v in [f.x0, f.y0, f.x1, f.y1, f.argb as i32] {
                h = h.wrapping_mul(31).wrapping_add(i64::from(v));
            }
        }
        h
    }

    #[test]
    fn java_random_matches_java() {
        let mut r = JavaRandom::new(42);
        assert_eq!(r.next(32), -1_170_105_035);
        assert_eq!(r.next_double(), 0.054_665_262_747_160_77);
        assert_eq!(r.next_float(), 0.047_939_3);
        assert!(!r.next_boolean());
        assert_eq!(r.next_int(7), 4);
        assert_eq!(r.next_int(4), 1);
        let g1 = r.next_gaussian();
        let g2 = r.next_gaussian();
        assert!((g1 - 0.266_973_605_724_219_6).abs() < 1e-15, "{g1}");
        assert!((g2 + 0.525_276_745_424_168_8).abs() < 1e-15, "{g2}");
    }

    #[test]
    fn colours_and_ease_match_java() {
        assert_eq!(ease(0.25), 0.156_25);
        assert_eq!(ease(0.5), 0.5);
        assert_eq!(ease(-1.0), 0.0);
        assert_eq!(ease(2.0), 1.0);
        assert_eq!(argb(0.5, MESSAGE_COLOR), 0x80FF_F1D0);
        assert_eq!(argb(1.0, 0), 0xFF00_0000);
        assert_eq!(argb(-1.0, 0x12_3456), 0x0012_3456);
        assert_eq!(lerp_color(SKY_TOP, SKY_BOTTOM, 0.5), 0x07_0C1F);
        assert_eq!(lerp_color(0, SKY_TOP, 0.37), 0x04_0713);
    }

    #[test]
    fn java_rounding_rounds_halves_up() {
        assert_eq!(round_f(2.5), 3);
        assert_eq!(round_f(-2.5), -2);
        assert_eq!(round_f(-2.6), -3);
        assert_eq!(round_d(-0.5), 0);
        assert_eq!(round_d(1.499_999), 1);
        assert_eq!(round_d(f64::NAN), 0);
    }

    #[test]
    fn the_chain_matches_java() {
        let sky = Sky::new(&mut JavaRandom::new(12345));
        let java: [(f64, f64, f64); 12] = [
            (0.0, 0.474_549_656_103_836_7, 0.0),
            (0.237_318_592_326_178_13, 0.568_550_686_089_212_8, 0.255_257_336_627_158_1),
            (0.462_749_712_315_626_1, 0.466_166_996_421_129_13, 0.502_848_955_568_000_6),
            (0.599_490_830_354_401_8, 0.392_556_661_576_881_3, 0.658_144_205_844_294_1),
            (0.822_164_350_377_803_1, 0.514_224_854_456_503_1, 0.911_889_443_589_172_7),
            (0.973_561_808_083_444_6, 0.654_050_797_405_527, 1.117_977_979_226_93),
            (1.116_806_484_642_936, 0.516_998_456_666_920_5, 1.316_226_261_601_745),
            (1.294_710_137_329_438_7, 0.594_722_327_501_501_3, 1.510_367_214_872_069_7),
            (1.486_107_913_390_644_7, 0.440_318_699_759_824_9, 1.756_280_998_424_284_9),
            (1.674_860_558_150_066, 0.637_491_040_006_316_5, 2.029_236_109_499_265),
            (1.864_552_953_915_152_2, 0.521_777_703_339_731_3, 2.251_435_977_396_643_5),
            (2.121_276_047_673_632, 0.615_211_308_787_201_3, 2.524_632_949_370_783),
        ];
        for (i, (u, v, len)) in java.iter().enumerate() {
            assert!((sky.chain_u[i] - u).abs() < 1e-12, "u {i}");
            assert!((sky.chain_v[i] - v).abs() < 1e-12, "v {i}");
            assert!((sky.chain_length[i] - len).abs() < 1e-12, "len {i}");
        }
        let star = &sky.stars[0];
        assert!((star.u - 1.085_409_321_481_415_6).abs() < 1e-15);
        assert!((star.v - 0.932_993_485_288_541).abs() < 1e-15);
        assert_eq!((star.layer, star.arms, star.color, star.delay), (1, 0, 0xCF_E0FF, 948));
        assert_eq!(star.brightness, 0.218_836_47);
        assert!((star.phase - 0.783_313_023_942_002).abs() < 1e-15);
        assert!((star.speed - 0.001_588_339_870_762_652).abs() < 1e-18);
        let band = &sky.band_stars[0];
        assert!((band.u - 1.515_790_332_678_981_2).abs() < 1e-15);
        assert!((band.v - 0.181_454_181_913_414_34).abs() < 1e-12);
        assert_eq!((band.layer, band.arms, band.delay), (0, 0, 225));
        assert_eq!(band.brightness, 0.470_367_34);
        let dust = &sky.dust[DUST_COUNT - 1];
        assert!((dust.u - 0.978_799_882_375_591_8).abs() < 1e-15);
        assert!((dust.v - 0.112_530_490_935_450_91).abs() < 1e-12);
        assert_eq!((dust.size, dust.alpha), (3, 0.048_655_286));
    }

    #[test]
    fn the_view_and_the_last_star_match_java() {
        let sky = Sky::new(&mut JavaRandom::new(12345));
        assert!((sky.traced_length(2000) - 0.303_638_617_803_021_6).abs() < 1e-12);
        let camera = sky.camera(sky.total(), 960, 540);
        assert!((camera - 905.489_065_743_761_3).abs() < 1e-9);
        assert_eq!(sky.chain_point(SEGMENTS, camera, 960, 540), [528, 332]);
        // Before the trace and after it.
        assert_eq!(sky.traced_length(TRACE_AT), 0.0);
        assert_eq!(sky.traced_length(TRACE_AT + SEGMENTS as i64 * SEGMENT), sky.total());
        assert_eq!(sky.camera(0.0, 960, 540), 0.0);
    }

    fn frame(journey: &Journey, now: i64, time: i64, width: i32, height: i32, reveal: f32) -> (Vec<Fill>, Vec<Fill>) {
        let sky = clamp_f((time - BLACK) as f32 / SKY_IN as f32, 0.0, 1.0);
        journey.art(now, time, width, height, sky, reveal)
    }

    #[test]
    fn a_frame_of_the_trace_draws_the_java_pixels() {
        let journey = Journey::new(0, true, true, 12345);
        let (sky, art) = frame(&journey, 1_000_000, 2600, 960, 540, 0.0);
        // Java: 5369 fills, the black and one gradient first.
        assert_eq!(art.len(), 5369 - 2);
        assert_eq!(hash(&art), -3_727_128_888_027_030_015);
        // The black, then bands from the top colour to the bottom colour at the full sky.
        assert_eq!(sky.first(), Some(&Fill { x0: 0, y0: 0, x1: 960, y1: 540, argb: 0xFF00_0000 }));
        assert_eq!(sky.get(1).map(|f| f.argb), Some(0xFF0B_1433));
        assert_eq!(sky.last().map(|f| (f.argb, f.y1)), Some((0xFF03_040B, 540)));
        // A new band starts when one channel steps: red 8 steps, green 16, blue 40.
        assert!(sky.len() <= 2 + 8 + 16 + 40);
        // The bands cover every row once, in order.
        let mut row = 0;
        for band in &sky[1..] {
            assert_eq!((band.x0, band.x1, band.y0), (0, 960, row));
            row = band.y1;
        }
        assert_eq!(row, 540);
    }

    #[test]
    fn a_frame_of_the_dissolve_draws_the_java_pixels() {
        let mut journey = Journey::new(0, true, true, 12345);
        journey.dissolve = [528, 332];
        let (sky, art) = frame(&journey, 1_004_000, 6600, 960, 540, ease(0.3));
        let all: Vec<Fill> = sky.into_iter().chain(art).collect();
        assert_eq!(all.len(), 14_740);
        assert_eq!(hash(&all), 7_913_159_391_653_663_012);
    }

    #[test]
    fn a_small_window_early_in_the_sky_draws_the_java_pixels() {
        let journey = Journey::new(0, true, true, 12345);
        let (_, art) = frame(&journey, 1_000_123, 900, 427, 241, 0.0);
        assert_eq!(art.len(), 1437 - 2);
        assert_eq!(hash(&art), 6_809_253_188_077_824_399);
    }

    #[test]
    fn the_grid_follows_the_gui_scale() {
        // Scale 2: one art pixel is one screen pixel. Scale 3 and 4: two. Scale 5: three.
        assert_eq!(grid(2.0, [1920, 1080]), Grid { pixel: 1, width: 1920, height: 1080 });
        assert_eq!(grid(3.0, [1921, 1081]), Grid { pixel: 2, width: 961, height: 541 });
        assert_eq!(grid(4.0, [1709, 1009]), Grid { pixel: 2, width: 855, height: 505 });
        assert_eq!(grid(5.0, [1600, 1200]), Grid { pixel: 3, width: 534, height: 400 });
        assert_eq!(grid(1.0, [0, 0]), Grid { pixel: 1, width: 0, height: 0 });
        assert_eq!(gui_scale(1709, 428.0), 4.0);
        assert_eq!(gui_scale(1920, 960.0), 2.0);
        assert_eq!(gui_scale(1920, 0.0), 1.0);
    }

    #[test]
    fn the_message_layout_matches_java() {
        // Fits twice: 100 * 2 <= 427 - 32.
        let layout = message_layout(100, 427, 240);
        assert_eq!(layout.scale, 2.0);
        assert_eq!(layout.translate, [213.5, 192.0]);
        assert_eq!(layout.at, [-50, -4]);
        // Too wide for twice the size.
        assert_eq!(message_layout(198, 427, 240).scale, 1.0);
        assert_eq!(message_layout(197, 427, 240).scale, 2.0);
        // Java int division: -101 / 2 is -50.
        assert_eq!(message_layout(101, 427, 240).at, [-50, -4]);
    }

    fn view(now: i64) -> View {
        View {
            now,
            in_world: true,
            death_screen: true,
            loading: false,
            chunk: true,
            grid: Grid { pixel: 1, width: 960, height: 540 },
            covered: false,
            seed: 1,
        }
    }

    #[test]
    fn the_fade_draws_black_over_600_ms() {
        let mut journey = Journey::new(1000, true, false, 1);
        journey.update(&view(1000));
        assert_eq!(journey.picture().fade, Some(0x0000_0000));
        journey.update(&view(1300));
        assert_eq!(journey.picture().fade, Some(argb(0.5, 0)));
        journey.update(&view(1599));
        assert!(journey.picture().fade.is_some());
        journey.update(&view(1600));
        let picture = journey.picture();
        assert_eq!(picture.fade, None);
        // Black only during BLACK: the sky has no alpha yet, and no star shows.
        assert_eq!(picture.sky.get(1).map(|f| f.argb >> 24), Some(0));
        assert!(picture.art.iter().all(|f| f.argb >> 24 == 0 || f.argb & 0xFF_FFFF == DUST_COLOR));
    }

    #[test]
    fn the_respawn_waits_for_the_black_and_for_ready() {
        let mut journey = Journey::new(0, true, false, 1);
        assert!(!journey.update(&view(599)).0.send_respawn);
        // Black, but the server has not said Ready.
        assert!(!journey.update(&view(600)).0.send_respawn);
        journey.set_ready();
        assert!(journey.update(&view(700)).0.send_respawn);
        // Once only.
        assert!(!journey.update(&view(800)).0.send_respawn);
        // Ready before the black: the respawn goes at the black.
        let mut early = Journey::new(0, true, true, 1);
        assert!(!early.update(&view(599)).0.send_respawn);
        assert!(early.update(&view(600)).0.send_respawn);
    }

    #[test]
    fn the_chimes_follow_the_line() {
        let mut journey = Journey::new(0, true, true, 12345);
        // Nothing before Home.
        assert_eq!(journey.chime(HOME_AT - 1), None);
        let home = journey.chime(HOME_AT);
        assert_eq!(home, Some(Sound { id: CHIME_SOUND, volume: 1.0, pitch: 0.7 }));
        // Star 1 waits for the line.
        assert_eq!(journey.chime(TRACE_AT), None);
        let mut sounds = Vec::new();
        for time in (TRACE_AT..TRACE_AT + 3000).step_by(16) {
            if let Some(sound) = journey.chime(time) {
                sounds.push((time, sound));
            }
        }
        assert_eq!(sounds.len(), SEGMENTS);
        assert_eq!(sounds[0].1, Sound { id: CHIME_SOUND, volume: 0.7, pitch: 0.7 + 0.8 / 11.0 });
        assert_eq!(sounds[9].1.pitch, 0.7 + 0.8 * 10.0 / 11.0);
        let (time, last) = sounds[SEGMENTS - 1];
        assert_eq!(last, Sound { id: RESONATE_SOUND, volume: 1.0, pitch: 1.0 });
        assert!(time >= TRACE_AT + SEGMENTS as i64 * SEGMENT);
        assert_eq!(journey.chime(10_000), None);
    }

    /// Runs a faded journey with the given world from `from` to `to` in 16 ms frames and gives every
    /// command and the time it ended.
    fn run(journey: &mut Journey, from: i64, to: i64, world: impl Fn(i64) -> View) -> (Vec<(i64, Commands)>, Option<i64>) {
        let mut out = Vec::new();
        let mut now = from;
        while now <= to {
            let (commands, done) = journey.update(&world(now));
            if commands != Commands::default() {
                out.push((now, commands));
            }
            if done {
                return (out, Some(now));
            }
            now += 16;
        }
        (out, None)
    }

    #[test]
    fn a_whole_journey_ends_with_the_bell() {
        let mut journey = Journey::new(0, true, true, 12345);
        // The server answers the respawn at 900 ms; the arrival comes with the name.
        let world = |now: i64| View {
            death_screen: now < 900,
            ..view(now)
        };
        let (first, _) = run(&mut journey, 0, 3000, world);
        // A late arrival: the message starts at once and holds the reveal past the last star.
        journey.arrive(Some("Steve".to_owned()));
        let (rest, end) = run(&mut journey, 3001, 20_000, world);
        let all: Vec<(i64, Commands)> = first.into_iter().chain(rest).collect();
        let sent: Vec<i64> = all.iter().filter(|(_, c)| c.send_respawn).map(|(t, _)| *t).collect();
        assert_eq!(sent, [608]);
        let ids: Vec<&str> = all.iter().flat_map(|(_, c)| c.sounds.iter().map(|s| s.id)).collect();
        assert_eq!(ids.first(), Some(&crate::respawn::RESPAWN_SOUND));
        assert_eq!(ids.iter().filter(|id| **id == CHIME_SOUND).count(), SEGMENTS);
        assert_eq!(ids[ids.len() - 3..], [RESONATE_SOUND, BREAK_SOUND, crate::respawn::BELL_SOUND]);
        // The message starts at 3001 ms and lasts 2400 ms: the dissolve waits for it.
        let release = all.iter().find(|(_, c)| c.release).map(|(t, _)| *t);
        let message_end = 3001 + MESSAGE_IN + MESSAGE_HOLD + MESSAGE_OUT;
        assert!(release.is_some_and(|t| (message_end..message_end + 16).contains(&t)), "{release:?}");
        assert!(end.is_some_and(|t| t - release.unwrap_or(0) >= DISSOLVE && t - release.unwrap_or(0) < DISSOLVE + 16));
        // The input is held from the respawn until the dissolve.
        let hold: Vec<i64> = all.iter().filter(|(_, c)| c.hold).map(|(t, _)| *t).collect();
        assert_eq!(hold.first(), Some(&912));
    }

    #[test]
    fn no_message_reveals_after_the_last_star() {
        let mut journey = Journey::new(0, true, true, 12345);
        journey.arrive(None);
        let world = |now: i64| View {
            death_screen: now < 650,
            ..view(now)
        };
        let (all, _) = run(&mut journey, 0, 20_000, world);
        let release = all.iter().find(|(_, c)| c.release).map(|(t, _)| *t);
        let beat = FADE_OUT + TRACE_AT + SEGMENTS as i64 * SEGMENT + LAST_STAR_BEAT;
        assert!(release.is_some_and(|t| (beat..beat + 16).contains(&t)), "{release:?}");
        assert!(journey.message(beat).is_none());
    }

    #[test]
    fn the_terrain_holds_the_reveal() {
        let mut journey = Journey::new(0, true, true, 12345);
        journey.arrive(None);
        // Loading until 8 s, then the chunk; the reveal comes 500 ms after the chunk.
        let world = |now: i64| View {
            death_screen: now < 650,
            loading: now < 8000,
            ..view(now)
        };
        let (all, _) = run(&mut journey, 0, 20_000, world);
        let release = all.iter().find(|(_, c)| c.release).map(|(t, _)| *t);
        assert!(release.is_some_and(|t| (8000 + TERRAIN_GRACE..8000 + TERRAIN_GRACE + 16).contains(&t)), "{release:?}");
        // A chunk that goes away starts the grace again.
        let mut again = Journey::new(0, true, true, 1);
        again.respawn_sent = true;
        let mut v = View { death_screen: false, ..view(5000) };
        assert!(!again.terrain_ready(5000, &v));
        assert!(again.terrain_ready(5500, &v));
        v.chunk = false;
        assert!(!again.terrain_ready(5600, &v));
        v.chunk = true;
        assert!(!again.terrain_ready(5700, &v));
        assert!(again.terrain_ready(6200, &v));
    }

    #[test]
    fn the_arrival_waits_one_second_at_most() {
        let mut journey = Journey::new(0, true, true, 12345);
        // The respawn is answered late, after the last star: then the reveal waits ARRIVAL_WAIT.
        let world = |now: i64| View {
            death_screen: now < 5000,
            ..view(now)
        };
        let (all, _) = run(&mut journey, 0, 20_000, world);
        let release = all.iter().find(|(_, c)| c.release).map(|(t, _)| *t);
        assert!(release.is_some_and(|t| (5008 + ARRIVAL_WAIT..5008 + ARRIVAL_WAIT + 16).contains(&t)), "{release:?}");
    }

    #[test]
    fn the_journey_gives_up_and_always_ends() {
        // The terrain never comes: 30 s after the respawn.
        let mut journey = Journey::new(0, true, true, 1);
        journey.arrive(None);
        let world = |now: i64| View {
            death_screen: now < 650,
            loading: true,
            ..view(now)
        };
        let (all, end) = run(&mut journey, 0, 40_000, world);
        let release = all.iter().find(|(_, c)| c.release).map(|(t, _)| *t);
        assert!(release.is_some_and(|t| t > 656 + GIVE_UP && t <= 656 + GIVE_UP + 16), "{release:?}");
        assert!(end.is_some());
        // The server never says Ready: 150 s after the start, and the respawn goes then.
        let mut stuck = Journey::new(0, true, false, 1);
        let (all, end) = run(&mut stuck, 0, 160_000, view);
        let sent: Vec<i64> = all.iter().filter(|(_, c)| c.send_respawn).map(|(t, _)| *t).collect();
        assert_eq!(sent.len(), 1);
        assert!(sent[0] > SERVER_WAIT && sent[0] <= SERVER_WAIT + 16);
        assert!(end.is_some());
    }

    #[test]
    fn a_late_arrival_has_no_message() {
        let mut journey = Journey::new(0, true, true, 1);
        journey.dissolve_start = Some(10);
        journey.arrive(Some("Alex".to_owned()));
        assert!(journey.arrived);
        assert_eq!(journey.name, None);
    }

    #[test]
    fn the_message_fades_in_holds_and_fades_out() {
        let mut journey = Journey::new(0, false, true, 1);
        journey.arrive(Some("Steve".to_owned()));
        journey.message_start = Some(0);
        let alpha = |now| journey.message(now).map(|m| m.alpha);
        // Under an alpha byte of 4: skipped.
        assert_eq!(alpha(0), None);
        assert_eq!(alpha(5), None);
        assert_eq!(alpha(200), Some(128));
        assert_eq!(alpha(400), Some(255));
        assert_eq!(alpha(1999), Some(255));
        assert_eq!(alpha(2200), Some(128));
        assert_eq!(alpha(2399), None);
        assert_eq!(alpha(2400), None);
        let message = Message { name: "Steve".to_owned(), alpha: 0x80 };
        assert_eq!(message.color(), 0x80FF_F1D0);
    }

    #[test]
    fn the_music_fades_then_stops_once() {
        let mut journey = Journey::new(0, true, true, 1);
        assert_eq!(journey.music(0), MusicAction::Scale(1.0));
        assert_eq!(journey.music(300), MusicAction::Scale(0.5));
        assert_eq!(journey.music(600), MusicAction::Cut);
        assert_eq!(journey.music(650), MusicAction::Quiet);
        // A journey that starts in black leaves the music alone.
        let mut black = Journey::new(0, false, true, 1);
        assert_eq!(black.music(0), MusicAction::Keep);
        assert_eq!(black.music(900), MusicAction::Keep);
    }

    const GUI: [f32; 2] = [427.0, 241.0];
    // Respawn at (113, 60 + 72), Title Screen at (113, 60 + 96).
    const RESPAWN: [f32; 2] = [120.0, 140.0];
    const TITLE: [f32; 2] = [120.0, 160.0];

    #[test]
    fn the_death_buttons_sit_where_horizon_draws_them() {
        // Horizon's own test: a 427x253 GUI puts Respawn at (113, 135).
        assert_eq!(death_buttons([427.0, 253.0]), [[113.0, 135.0], [113.0, 159.0]]);
        assert_eq!(death_buttons([320.0, 334.0]), [[60.0, 155.0], [60.0, 179.0]]);
        // The confirmation of a 427x241 GUI: a 304 wide row, 41 under the top of a 61 high column.
        assert_eq!(confirm_buttons(GUI, 150.0), [[61.0, 131.0], [215.0, 131.0]]);
        // A title wider than the row moves nothing: the row stays centred.
        assert_eq!(confirm_buttons(GUI, 350.0), [[61.0, 131.0], [215.0, 131.0]]);
    }

    #[test]
    fn a_respawn_click_after_the_delay_respawns() {
        let mut screen = DeathScreen::new(0);
        // The first second: the buttons are inactive.
        assert_eq!(screen.click(999, RESPAWN, GUI, 150.0), Press::Swallow);
        assert_eq!(screen.click(500, TITLE, GUI, 150.0), Press::Swallow);
        assert_eq!(screen.click(500, [1.0, 1.0], GUI, 150.0), Press::Pass);
        assert_eq!(screen.click(1000, RESPAWN, GUI, 150.0), Press::Respawn);
        // The edges of the button: the right and bottom edges are outside.
        let mut screen = DeathScreen::new(0);
        assert_eq!(screen.click(2000, [313.0, 140.0], GUI, 150.0), Press::Pass);
        assert_eq!(screen.click(2000, [113.0, 132.0], GUI, 150.0), Press::Respawn);
    }

    #[test]
    fn title_screen_asks_and_no_respawns() {
        let mut screen = DeathScreen::new(0);
        assert_eq!(screen.click(2000, TITLE, GUI, 150.0), Press::Pass);
        let [yes, no] = confirm_buttons(GUI, 150.0);
        // The confirmation waits one second too.
        assert_eq!(screen.click(2500, no, GUI, 150.0), Press::Swallow);
        // The old Respawn place is no button of the confirmation.
        assert_eq!(screen.click(3000, [300.0, 200.0], GUI, 150.0), Press::Pass);
        assert_eq!(screen.click(3000, no, GUI, 150.0), Press::Respawn);
        assert_eq!(screen.click(3000, yes, GUI, 150.0), Press::Pass);
    }

    #[test]
    fn the_keys_of_the_death_screen() {
        let mut screen = DeathScreen::new(0);
        assert_eq!(screen.key(500, "key.keyboard.enter", 0), Press::Swallow);
        // No focus yet: Enter does nothing.
        assert_eq!(screen.key(1500, "key.keyboard.enter", 0), Press::Pass);
        assert_eq!(screen.key(1500, "key.keyboard.tab", 0), Press::Pass);
        assert_eq!(screen.key(1500, "key.keyboard.space", 0), Press::Respawn);
        // Shift+Tab from no focus goes to Title Screen; Enter then asks.
        let mut screen = DeathScreen::new(0);
        assert_eq!(screen.key(1500, "key.keyboard.tab", SHIFT), Press::Pass);
        assert_eq!(screen.key(1500, "key.keyboard.keypad.enter", 0), Press::Pass);
        // The confirmation opened with the focus on yes. Escape answers no.
        assert_eq!(screen.key(2000, "key.keyboard.escape", 0), Press::Swallow);
        assert_eq!(screen.key(2600, "key.keyboard.enter", 0), Press::Pass);
        let mut screen = DeathScreen::new(0);
        screen.key(1500, "key.keyboard.down", 0);
        screen.key(1500, "key.keyboard.enter", 0);
        assert_eq!(screen.key(2600, "key.keyboard.right", 0), Press::Pass);
        assert_eq!(screen.key(2600, "key.keyboard.enter", 0), Press::Respawn);
        let mut screen = DeathScreen::new(0);
        screen.key(1500, "key.keyboard.down", 0);
        screen.key(1500, "key.keyboard.enter", 0);
        assert_eq!(screen.key(2600, "key.keyboard.escape", 0), Press::Respawn);
        // Up focuses Respawn; another key does nothing.
        let mut screen = DeathScreen::new(0);
        assert_eq!(screen.key(1500, "key.keyboard.a", 0), Press::Pass);
        screen.key(1500, "key.keyboard.up", 0);
        assert_eq!(screen.key(1500, "key.keyboard.enter", 0), Press::Respawn);
    }

    fn client_view(now: i64, death_screen: bool) -> View {
        View {
            death_screen,
            ..view(now)
        }
    }

    /// A frame where a screen that the game does not name covers the world (`NativeScreenKind::Other`).
    fn covered_view(now: i64) -> View {
        View {
            death_screen: false,
            covered: true,
            ..view(now)
        }
    }

    #[test]
    fn the_client_holds_the_click_after_a_departure() {
        let mut client = Client::new();
        client.frame(&client_view(0, true));
        // Not armed: the click is the game's own.
        assert!(!client.death_click(1500, LEFT_BUTTON, RESPAWN, GUI, 150.0, 1));
        // Not armed, but in the first second: the button is not active yet, as in vanilla.
        assert!(client.death_click(500, LEFT_BUTTON, RESPAWN, GUI, 150.0, 1));
        client.signal(1600, JourneySignal::Departure, false, 1);
        client.signal(1600, JourneySignal::Ready, false, 1);
        // The right button does nothing on the death screen.
        assert!(!client.death_click(2000, 1, RESPAWN, GUI, 150.0, 1));
        assert!(client.death_click(2000, LEFT_BUTTON, RESPAWN, GUI, 150.0, 1));
        assert!(client.holds_input());
        // Every press goes to the journey now.
        assert!(client.death_key(2010, "key.keyboard.a", 0, 1));
        let commands = client.frame(&client_view(2016, true));
        assert!(commands.send_held);
        assert!(!commands.send_respawn);
        let commands = client.frame(&client_view(2600, true));
        assert!(commands.send_respawn);
        assert!(!client.frame(&client_view(2616, true)).send_held);
    }

    #[test]
    fn a_press_before_the_departure_waits_for_the_delay() {
        let mut client = Client::new();
        client.frame(&client_view(0, true));
        // A fast click and Space before Departure: cancelled, the button is not active yet.
        assert!(client.death_click(300, LEFT_BUTTON, RESPAWN, GUI, 150.0, 1));
        assert!(client.death_key(300, "key.keyboard.tab", 0, 1));
        assert!(!client.holds_input());
        // Departure comes in that second: the next press is held.
        client.signal(400, JourneySignal::Departure, false, 1);
        assert!(client.death_click(1000, LEFT_BUTTON, RESPAWN, GUI, 150.0, 1));
        assert!(client.holds_input());
        // A click on no button passes as before.
        let mut quiet = Client::new();
        quiet.frame(&client_view(0, true));
        assert!(!quiet.death_click(300, LEFT_BUTTON, [1.0, 1.0], GUI, 150.0, 1));
    }

    #[test]
    fn a_death_screen_closed_without_the_hold_starts_black_in_that_frame() {
        let mut client = Client::new();
        client.frame(&client_view(0, true));
        client.signal(10, JourneySignal::Departure, false, 1);
        client.frame(&client_view(20, true));
        // `horizon-ctl respawn`: the respawn's loading screen first (Other), which changes nothing.
        client.frame(&covered_view(30));
        assert!(!client.holds_input());
        // The first frame that shows the world: black before it is drawn, never the arrival place.
        let commands = client.frame(&client_view(40, false));
        assert!(client.holds_input());
        assert!(!commands.send_respawn);
        // The whole window is opaque black: the first fill of the sky.
        let picture = client.picture();
        assert!(picture.is_some_and(|p| p.fade.is_none() && p.sky.first().is_some_and(|f| f.argb == 0xFF00_0000)));
        assert_eq!(client.music(40), MusicAction::Keep);
        // The arrival gives the name to that journey.
        client.signal(50, JourneySignal::Arrival { name: Some("Steve".to_owned()) }, true, 1);
        assert!(client.holds_input());
        assert!(!client.armed);
    }

    #[test]
    fn a_screen_over_the_death_screen_keeps_the_arm_and_the_confirmation() {
        let mut client = Client::new();
        client.frame(&client_view(0, true));
        client.signal(10, JourneySignal::Departure, false, 1);
        // Title Screen opens the quit confirmation.
        assert!(!client.death_click(2000, LEFT_BUTTON, TITLE, GUI, 150.0, 1));
        // A prompt or a pack reload covers the death screen, then goes.
        client.frame(&covered_view(2100));
        client.frame(&covered_view(2200));
        assert!(!client.holds_input());
        client.frame(&client_view(2300, true));
        // The confirmation is still known: its No respawns, and the press is held.
        let [_, no] = confirm_buttons(GUI, 150.0);
        assert!(client.death_click(3100, LEFT_BUTTON, no, GUI, 150.0, 1));
        assert!(client.holds_input());
    }

    #[test]
    fn a_departure_under_a_covering_screen_stays_armed() {
        let mut client = Client::new();
        // A book over the death screen when Departure comes: the game says Other.
        client.signal(0, JourneySignal::Departure, false, 1);
        client.frame(&covered_view(5));
        client.frame(&client_view(10, true));
        assert!(client.death_click(1100, LEFT_BUTTON, RESPAWN, GUI, 150.0, 1));
    }

    #[test]
    fn an_immediate_respawn_starts_black_and_waits_for_the_arrival() {
        let mut client = Client::new();
        // `immediate_respawn`: no death screen, Departure comes over the world.
        client.signal(0, JourneySignal::Departure, true, 1);
        assert!(client.holds_input());
        assert!(client.picture().is_some_and(|p| p.fade.is_none()));
        // The search takes 6 s: no reveal and no respawn request meanwhile.
        for now in (16..6000).step_by(16) {
            let commands = client.frame(&client_view(now, false));
            assert!(!commands.send_respawn && !commands.release, "{now}");
        }
        // Ready and Arrival come together, when the server moves the player.
        client.signal(6000, JourneySignal::Ready, false, 1);
        client.signal(6000, JourneySignal::Arrival { name: Some("Steve".to_owned()) }, false, 1);
        let mut release = None;
        let mut now = 6016;
        while now < 20_000 && client.holds_input() {
            if client.frame(&client_view(now, false)).release && release.is_none() {
                release = Some(now);
            }
            now += 16;
        }
        // The message runs from the arrival (2400 ms), then the dissolve.
        let message_end = 6016 + MESSAGE_IN + MESSAGE_HOLD + MESSAGE_OUT;
        assert!(release.is_some_and(|t| (message_end..message_end + 32).contains(&t)), "{release:?}");
        assert!(!client.holds_input());
    }

    #[test]
    fn a_waiting_journey_ends_without_an_arrival() {
        // Ready with no Arrival: the server kept the vanilla point. The reveal comes ARRIVAL_WAIT later.
        let mut journey = Journey::waiting(0, false, 1);
        let world = |now: i64| View { death_screen: false, ..view(now) };
        let (first, _) = run(&mut journey, 0, 8000, world);
        assert!(first.iter().all(|(_, c)| !c.release));
        journey.set_ready();
        let (rest, end) = run(&mut journey, 8001, 20_000, world);
        let release = rest.iter().find(|(_, c)| c.release).map(|(t, _)| *t);
        assert!(release.is_some_and(|t| (8001 + ARRIVAL_WAIT..8001 + ARRIVAL_WAIT + 16).contains(&t)), "{release:?}");
        assert!(end.is_some());
        // Nothing comes: the journey gives up after SERVER_WAIT, as a held respawn does.
        let mut lost = Journey::waiting(0, false, 1);
        let (all, end) = run(&mut lost, 0, 160_000, world);
        let release = all.iter().find(|(_, c)| c.release).map(|(t, _)| *t);
        assert!(release.is_some_and(|t| t > SERVER_WAIT && t <= SERVER_WAIT + 16), "{release:?}");
        assert!(end.is_some());
        assert!(all.iter().all(|(_, c)| !c.send_respawn));
    }

    #[test]
    fn the_arrival_of_a_waiting_journey_starts_the_terrain_test_again() {
        let mut journey = Journey::waiting(0, false, 1);
        let world = |now: i64| View { death_screen: false, ..view(now) };
        // The chunk of the vanilla point is there long before the arrival.
        let (first, _) = run(&mut journey, 0, 5000, world);
        assert!(first.iter().all(|(_, c)| !c.release));
        // Ready and Arrival come together when the server moves the player.
        journey.set_ready();
        journey.arrive(None);
        // At the landing, the chunk comes 2 s after the arrival.
        let landing = |now: i64| View { death_screen: false, chunk: now >= 7000, ..view(now) };
        let (all, _) = run(&mut journey, 5008, 20_000, landing);
        let release = all.iter().find(|(_, c)| c.release).map(|(t, _)| *t);
        // Frames every 16 ms from 5008: the chunk is first seen at 7008.
        assert!(release.is_some_and(|t| (7008 + TERRAIN_GRACE..7008 + TERRAIN_GRACE + 16).contains(&t)), "{release:?}");
    }

    #[test]
    fn the_arm_belongs_to_one_death() {
        let mut client = Client::new();
        // Death 1 with the immediate rule: Departure over the world, a journey in black, its arrival.
        client.signal(0, JourneySignal::Departure, true, 1);
        client.signal(2000, JourneySignal::Ready, true, 1);
        client.signal(2000, JourneySignal::Arrival { name: None }, true, 1);
        let mut now = 16;
        while client.holds_input() && now < 30_000 {
            client.frame(&client_view(now, false));
            now += 16;
        }
        assert!(!client.holds_input());
        // Death 2 with no Departure (the rule off, a creative death): the press is the game's own.
        client.frame(&client_view(40_000, true));
        assert!(!client.death_click(41_500, LEFT_BUTTON, RESPAWN, GUI, 150.0, 1));
        assert!(!client.holds_input());
        // An arm whose death screen closed without the hold becomes that journey, not a later hold.
        let mut client = Client::new();
        client.frame(&client_view(0, true));
        client.signal(10, JourneySignal::Departure, false, 1);
        client.frame(&client_view(20, false));
        assert!(!client.armed);
    }

    #[test]
    fn leaving_the_world_ends_everything() {
        let mut client = Client::new();
        client.signal(0, JourneySignal::Arrival { name: None }, true, 1);
        assert!(client.holds_input());
        let commands = client.frame(&View {
            in_world: false,
            ..client_view(10, false)
        });
        assert!(commands.release);
        assert!(!client.holds_input());
        assert!(client.picture().is_none());
        // A Ready of the old world does not leak into the next death.
        client.signal(20, JourneySignal::Ready, false, 1);
        client.frame(&View {
            in_world: false,
            ..client_view(30, false)
        });
        client.signal(40, JourneySignal::Departure, false, 1);
        assert!(!client.ready);
        // The module resets for a new world before a signal of its first frame: the signal stays.
        assert!(!client.leave_world());
        client.signal(50, JourneySignal::Departure, false, 1);
        client.frame(&client_view(60, true));
        assert!(client.death_click(1100, LEFT_BUTTON, RESPAWN, GUI, 150.0, 1));
        assert!(client.leave_world());
    }

    #[test]
    fn a_departure_resets_ready_and_ready_reaches_a_running_journey() {
        let mut client = Client::new();
        client.signal(0, JourneySignal::Ready, false, 1);
        client.signal(0, JourneySignal::Departure, false, 1);
        assert!(!client.ready);
        client.frame(&client_view(0, true));
        assert!(client.death_click(1500, LEFT_BUTTON, RESPAWN, GUI, 150.0, 1));
        assert!(!client.frame(&client_view(2200, true)).send_respawn);
        client.signal(2300, JourneySignal::Ready, false, 1);
        assert!(client.frame(&client_view(2316, true)).send_respawn);
    }
}
