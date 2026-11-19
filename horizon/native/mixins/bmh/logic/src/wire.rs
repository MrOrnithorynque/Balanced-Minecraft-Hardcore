// The byte layout of mod messages (integrator).
// Every mod message lives in this file (docs/horizon/ARCHITECTURE.md): its
// channel, its tag bytes, its encode and decode, and a round-trip test. The
// channel of an area is "bmh:<area>" (`crate::ids::channel`).
// The messages between two areas (journey and respawn) are below now. An area
// that needs a message of its own (the Echo Sense glow, the amulet screens)
// adds it here as an integrator change, in a section with the area's name,
// and names the change in its report (docs/horizon/HOOKS.md, "Contracts").
// Both sides use the same code, so the two sides cannot drift.
// Layout: integers and floats are little endian. A bool is one byte, 0 or 1.
// A string is a u32 byte length, then that many UTF-8 bytes.

/// The largest string that a message can hold, in bytes.
pub const MAX_STR: usize = 4096;

/// Writes the fields of one message, in order.
#[derive(Debug, Default, Clone)]
pub struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    /// An empty message.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds one byte.
    pub fn u8(&mut self, value: u8) -> &mut Self {
        self.bytes.push(value);
        self
    }

    /// Adds an unsigned 32-bit integer.
    pub fn u32(&mut self, value: u32) -> &mut Self {
        self.bytes.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Adds a signed 32-bit integer.
    pub fn i32(&mut self, value: i32) -> &mut Self {
        self.bytes.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Adds a signed 64-bit integer.
    pub fn i64(&mut self, value: i64) -> &mut Self {
        self.bytes.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Adds a 32-bit float.
    pub fn f32(&mut self, value: f32) -> &mut Self {
        self.bytes.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Adds a 64-bit float.
    pub fn f64(&mut self, value: f64) -> &mut Self {
        self.bytes.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Adds a bool as one byte.
    pub fn bool(&mut self, value: bool) -> &mut Self {
        self.u8(u8::from(value))
    }

    /// Adds a string. A string longer than `MAX_STR` bytes is cut at the
    /// last character boundary that fits, so the reader always accepts it.
    pub fn str(&mut self, value: &str) -> &mut Self {
        let mut end = value.len().min(MAX_STR);
        while !value.is_char_boundary(end) {
            end -= 1;
        }
        let cut = &value[..end];
        // `cut` is at most MAX_STR bytes, so the length fits in a u32.
        self.u32(cut.len() as u32);
        self.bytes.extend_from_slice(cut.as_bytes());
        self
    }

    /// The bytes written so far.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// The message, ready to send.
    pub fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

/// Reads the fields of one message, in order. Each read gives None when the
/// input is too short or not valid. A reader never panics.
#[derive(Debug, Clone)]
pub struct Reader<'a> {
    bytes: &'a [u8],
}

impl<'a> Reader<'a> {
    /// A reader at the start of `bytes`.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    /// Takes the next `n` bytes, or None when fewer are left.
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        if self.bytes.len() < n {
            return None;
        }
        let (head, tail) = self.bytes.split_at(n);
        self.bytes = tail;
        Some(head)
    }

    /// Takes the next `N` bytes as an array.
    fn array<const N: usize>(&mut self) -> Option<[u8; N]> {
        self.take(N)?.try_into().ok()
    }

    /// Reads one byte.
    pub fn u8(&mut self) -> Option<u8> {
        self.array::<1>().map(|b| b[0])
    }

    /// Reads an unsigned 32-bit integer.
    pub fn u32(&mut self) -> Option<u32> {
        self.array().map(u32::from_le_bytes)
    }

    /// Reads a signed 32-bit integer.
    pub fn i32(&mut self) -> Option<i32> {
        self.array().map(i32::from_le_bytes)
    }

    /// Reads a signed 64-bit integer.
    pub fn i64(&mut self) -> Option<i64> {
        self.array().map(i64::from_le_bytes)
    }

    /// Reads a 32-bit float.
    pub fn f32(&mut self) -> Option<f32> {
        self.array().map(f32::from_le_bytes)
    }

    /// Reads a 64-bit float.
    pub fn f64(&mut self) -> Option<f64> {
        self.array().map(f64::from_le_bytes)
    }

    /// Reads a bool. A byte other than 0 or 1 gives None.
    pub fn bool(&mut self) -> Option<bool> {
        match self.u8()? {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }

    /// Reads a string of at most `MAX_STR` bytes. A longer length, a short
    /// input or bytes that are not UTF-8 give None.
    pub fn str(&mut self) -> Option<&'a str> {
        let len = usize::try_from(self.u32()?).ok()?;
        if len > MAX_STR {
            return None;
        }
        std::str::from_utf8(self.take(len)?).ok()
    }

    /// The number of bytes not read yet.
    pub fn remaining(&self) -> usize {
        self.bytes.len()
    }

    /// True when every byte was read. A message with extra bytes is not valid.
    pub fn is_done(&self) -> bool {
        self.bytes.is_empty()
    }
}

// Messages between areas: journey and respawn.
// Each message is one tag byte, then the fields of that tag. An unknown tag,
// a short message or extra bytes give None. A tag number is never used again
// for another meaning.

/// Server to client: the respawn area (server) sends, the journey area
/// (client) reads. Helpers `net_send_to_player` and `net_from_server`.
pub const JOURNEY_CHANNEL: &str = "bmh:journey";

/// Client to server: the journey area (client) sends, the respawn area
/// (server) reads and checks (decision D7). Helpers `net_send_to_server` and
/// `net_from_client`.
pub const RESPAWN_CHANNEL: &str = "bmh:respawn";

/// A signal of the journey, on `JOURNEY_CHANNEL`. GAPS.md C4 and C8.
/// Java reference: handlers/JourneySignal.java (title packets there).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JourneySignal {
    /// Tag 1. Sent at death, last, only when no area cancelled the death and
    /// the next respawn is a distant one, never in a hardcore world
    /// (JourneySignal.java:28-35). The client fades to black.
    Departure,
    /// Tag 2. The landing search found the spot. The client may now send the
    /// Respawn request that it held (GAPS.md C4). The server sends it when
    /// the search ends, and again as the answer to `RespawnRequest::Held`.
    Ready,
    /// Tag 3. Sent after the distant respawn (JourneySignal.java:37-47).
    /// `name` is the player name for the message `bmh.journey.message`.
    /// None shows no message.
    Arrival { name: Option<String> },
}

impl JourneySignal {
    const DEPARTURE: u8 = 1;
    const READY: u8 = 2;
    const ARRIVAL: u8 = 3;

    /// The bytes of this signal.
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        match self {
            Self::Departure => {
                w.u8(Self::DEPARTURE);
            }
            Self::Ready => {
                w.u8(Self::READY);
            }
            Self::Arrival { name } => {
                w.u8(Self::ARRIVAL).bool(name.is_some());
                if let Some(name) = name {
                    w.str(name);
                }
            }
        }
        w.finish()
    }

    /// The signal in `bytes`, or None when the bytes are not valid.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let mut r = Reader::new(bytes);
        let signal = match r.u8()? {
            Self::DEPARTURE => Self::Departure,
            Self::READY => Self::Ready,
            Self::ARRIVAL => {
                let name = if r.bool()? {
                    Some(r.str()?.to_owned())
                } else {
                    None
                };
                Self::Arrival { name }
            }
            _ => return None,
        };
        r.is_done().then_some(signal)
    }
}

/// A request of the client, on `RESPAWN_CHANNEL`. GAPS.md C4, decision D7.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RespawnRequest {
    /// Tag 1. The client holds the Respawn click of the death screen and
    /// waits for `JourneySignal::Ready`. The server answers `Ready` when the
    /// landing is already found, so a client that missed the signal does not
    /// wait for ever. The server answers only for a dead player that has a
    /// landing search of its own.
    Held,
}

impl RespawnRequest {
    const HELD: u8 = 1;

    /// The bytes of this request.
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::Held => vec![Self::HELD],
        }
    }

    /// The request in `bytes`, or None when the bytes are not valid.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        match bytes {
            [Self::HELD] => Some(Self::Held),
            _ => None,
        }
    }
}

// Messages of the amulet area, between its server side and its screens (GAPS.md A5, A6).
// The Java mod sent title packets (AmuletPointSignal.java) and command packets (PointScreen.java):
// decision D6 replaces both with these messages. A position is three i32 (x, y, z).

/// Both directions of the amulet: the server opens the screens, the screens ask for changes.
pub const AMULET_CHANNEL: &str = "bmh:amulet";

/// The most points that a list carries. The cap of the config is 16; a player above the cap keeps
/// their points, so the list leaves room. A longer list is cut.
pub const MAX_LIST_POINTS: usize = 256;

fn write_pos(w: &mut Writer, pos: [i32; 3]) {
    w.i32(pos[0]).i32(pos[1]).i32(pos[2]);
}

fn read_pos(r: &mut Reader<'_>) -> Option<[i32; 3]> {
    Some([r.i32()?, r.i32()?, r.i32()?])
}

/// Server to client, on `AMULET_CHANNEL`: open a screen (AmuletPointSignal.java).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AmuletSignal {
    /// Tag 1. The naming screen of the block at `pos` (AmuletPointSignal.java:50-62). `name` is the
    /// point's name, None for a new point. `used` is the player's point count, `max` the cap, `bed`
    /// true for a bed and false for a lodestone.
    Open {
        pos: [i32; 3],
        name: Option<String>,
        used: i32,
        max: i32,
        bed: bool,
    },
    /// Tag 2. The list of the player's points (AmuletPointSignal.java:30-48): the cap, then the name
    /// and position of each point.
    List {
        max: i32,
        points: Vec<(String, [i32; 3])>,
    },
    /// Tag 3. The amulet's cooldown starts for `ticks` ticks: the client draws the sweep on the icon
    /// (GAPS.md A4; Java sends `ClientboundCooldownPacket`).
    Cooldown { ticks: i32 },
}

impl AmuletSignal {
    const OPEN: u8 = 1;
    const LIST: u8 = 2;
    const COOLDOWN: u8 = 3;

    /// The bytes of this signal. A list longer than `MAX_LIST_POINTS` is cut.
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        match self {
            Self::Open { pos, name, used, max, bed } => {
                w.u8(Self::OPEN);
                write_pos(&mut w, *pos);
                w.bool(name.is_some());
                if let Some(name) = name {
                    w.str(name);
                }
                w.i32(*used).i32(*max).bool(*bed);
            }
            Self::List { max, points } => {
                let shown = &points[..points.len().min(MAX_LIST_POINTS)];
                // `shown` holds at most MAX_LIST_POINTS entries, so the count fits in a u32.
                w.u8(Self::LIST).i32(*max).u32(shown.len() as u32);
                for (name, pos) in shown {
                    w.str(name);
                    write_pos(&mut w, *pos);
                }
            }
            Self::Cooldown { ticks } => {
                w.u8(Self::COOLDOWN).i32(*ticks);
            }
        }
        w.finish()
    }

    /// The signal in `bytes`, or None when the bytes are not valid.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let mut r = Reader::new(bytes);
        let signal = match r.u8()? {
            Self::OPEN => {
                let pos = read_pos(&mut r)?;
                let name = if r.bool()? { Some(r.str()?.to_owned()) } else { None };
                Self::Open { pos, name, used: r.i32()?, max: r.i32()?, bed: r.bool()? }
            }
            Self::LIST => {
                let max = r.i32()?;
                let count = usize::try_from(r.u32()?).ok()?;
                if count > MAX_LIST_POINTS {
                    return None;
                }
                let mut points = Vec::with_capacity(count);
                for _ in 0..count {
                    let name = r.str()?.to_owned();
                    points.push((name, read_pos(&mut r)?));
                }
                Self::List { max, points }
            }
            Self::COOLDOWN => Self::Cooldown { ticks: r.i32()? },
            _ => return None,
        };
        r.is_done().then_some(signal)
    }
}

/// Client to server, on `AMULET_CHANNEL`: a change that a screen asks for (PointScreen.java:96-111,
/// PointListScreen.java:102-105). The server checks each one as the Java command did (decision D7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AmuletRequest {
    /// Tag 1. Save or rename the point on the block at `pos` (`/bmh point save`).
    Save { pos: [i32; 3], name: String },
    /// Tag 2. Remove the point on the block at `pos` (`/bmh point remove`).
    Remove { pos: [i32; 3] },
    /// Tag 3. Remove the point at `pos`, with no block needed (`/bmh point delete`).
    Delete { pos: [i32; 3] },
}

impl AmuletRequest {
    const SAVE: u8 = 1;
    const REMOVE: u8 = 2;
    const DELETE: u8 = 3;

    /// The bytes of this request.
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        match self {
            Self::Save { pos, name } => {
                w.u8(Self::SAVE);
                write_pos(&mut w, *pos);
                w.str(name);
            }
            Self::Remove { pos } => {
                w.u8(Self::REMOVE);
                write_pos(&mut w, *pos);
            }
            Self::Delete { pos } => {
                w.u8(Self::DELETE);
                write_pos(&mut w, *pos);
            }
        }
        w.finish()
    }

    /// The request in `bytes`, or None when the bytes are not valid.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let mut r = Reader::new(bytes);
        let request = match r.u8()? {
            Self::SAVE => Self::Save { pos: read_pos(&mut r)?, name: r.str()?.to_owned() },
            Self::REMOVE => Self::Remove { pos: read_pos(&mut r)? },
            Self::DELETE => Self::Delete { pos: read_pos(&mut r)? },
            _ => return None,
        };
        r.is_done().then_some(request)
    }
}

// Messages between areas: amulet (server) and sky (client), GAPS.md V1 and V2.
// The Java client read the reading from the amulet stack in a hand
// (StarReadingRenderer.java:172-209). A Horizon client cannot read the main
// hand stack outside the first-person hand pass (docs/horizon/log/amulet.md,
// "Reading contract for the sky area"), so the server sends each new reading.

/// Server to client: the amulet area (server) sends, the sky area (client)
/// reads. Helpers `net_send_to_player` and `net_from_server`.
pub const SKY_CHANNEL: &str = "bmh:sky";

/// The most lines that a reading carries: the points of a list, then the
/// world spawn. A longer reading is cut.
pub const MAX_READING_LINES: usize = MAX_LIST_POINTS + 1;

/// A signal of the sky, on `SKY_CHANNEL`.
#[derive(Debug, Clone, PartialEq)]
pub enum SkySignal {
    /// Tag 1. A new star reading of the player's amulet. The server sends it
    /// in the same use that writes the reading on the stack
    /// (AmuletReading.java:210-245). The layout: `time` (i64), a bool and a
    /// u32 for `spawn`, a u32 line count, then an f32 azimuth and a name for
    /// each line. A name that `names` lacks is written as "", as the Java
    /// client reads it (StarReadingRenderer.java:203).
    Reading(crate::contract::AmuletReading),
}

impl SkySignal {
    const READING: u8 = 1;

    /// The bytes of this signal. A reading longer than `MAX_READING_LINES`
    /// is cut, and a spawn index that a u32 cannot hold is left out.
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        match self {
            Self::Reading(reading) => {
                w.u8(Self::READING).i64(reading.time);
                let spawn = reading.spawn.and_then(|s| u32::try_from(s).ok());
                w.bool(spawn.is_some());
                if let Some(spawn) = spawn {
                    w.u32(spawn);
                }
                let shown = &reading.azimuths[..reading.azimuths.len().min(MAX_READING_LINES)];
                // `shown` holds at most MAX_READING_LINES entries, so the count fits in a u32.
                w.u32(shown.len() as u32);
                for (i, azimuth) in shown.iter().enumerate() {
                    w.f32(*azimuth).str(reading.names.get(i).map_or("", String::as_str));
                }
            }
        }
        w.finish()
    }

    /// The signal in `bytes`, or None when the bytes are not valid. The two
    /// lists of a decoded reading have the same length.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let mut r = Reader::new(bytes);
        let signal = match r.u8()? {
            Self::READING => {
                let time = r.i64()?;
                let spawn = if r.bool()? { Some(usize::try_from(r.u32()?).ok()?) } else { None };
                let count = usize::try_from(r.u32()?).ok()?;
                if count > MAX_READING_LINES {
                    return None;
                }
                let mut azimuths = Vec::with_capacity(count);
                let mut names = Vec::with_capacity(count);
                for _ in 0..count {
                    azimuths.push(r.f32()?);
                    names.push(r.str()?.to_owned());
                }
                Self::Reading(crate::contract::AmuletReading { azimuths, names, spawn, time })
            }
            _ => return None,
        };
        r.is_done().then_some(signal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn every_sky_signal() -> Vec<SkySignal> {
        use crate::contract::AmuletReading;
        vec![
            SkySignal::Reading(AmuletReading { azimuths: vec![], names: vec![], spawn: None, time: 0 }),
            SkySignal::Reading(AmuletReading {
                azimuths: vec![301.14682, 0.0, 359.99],
                names: vec!["Camp".to_owned(), "Étoile".to_owned(), String::new()],
                spawn: Some(2),
                time: 18_042,
            }),
            SkySignal::Reading(AmuletReading {
                azimuths: vec![90.0],
                names: vec![String::new()],
                spawn: Some(0),
                time: i64::MIN,
            }),
        ]
    }

    #[test]
    fn every_sky_signal_round_trips_and_refuses_cut_input() {
        for signal in every_sky_signal() {
            let bytes = signal.encode();
            assert_eq!(SkySignal::decode(&bytes), Some(signal.clone()));
            for cut in 0..bytes.len() {
                assert_eq!(SkySignal::decode(&bytes[..cut]), None, "{signal:?} cut at {cut}");
            }
            let mut long = bytes.clone();
            long.push(0);
            assert_eq!(SkySignal::decode(&long), None);
        }
    }

    #[test]
    fn the_sky_tag_and_layout_are_fixed_and_bad_input_gives_none() {
        use crate::contract::AmuletReading;
        let one = SkySignal::Reading(AmuletReading {
            azimuths: vec![90.0],
            names: vec!["A".to_owned()],
            spawn: None,
            time: 1,
        });
        assert_eq!(
            one.encode(),
            [1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0xb4, 0x42, 1, 0, 0, 0, b'A']
        );
        assert_eq!(SkySignal::decode(&[]), None);
        assert_eq!(SkySignal::decode(&[2]), None);
        // A reading that says it holds more lines than the cap is refused before any allocation.
        let mut w = Writer::new();
        w.u8(1).i64(0).bool(false).u32(u32::MAX);
        assert_eq!(SkySignal::decode(w.bytes()), None);
        assert_eq!(SKY_CHANNEL, crate::ids::channel("sky"));
    }

    #[test]
    fn a_reading_is_cut_and_its_missing_names_are_empty() {
        use crate::contract::AmuletReading;
        let long = SkySignal::Reading(AmuletReading {
            azimuths: vec![10.0; MAX_READING_LINES + 4],
            names: vec!["p".to_owned()],
            spawn: Some(MAX_READING_LINES + 3),
            time: 5,
        });
        let Some(SkySignal::Reading(cut)) = SkySignal::decode(&long.encode()) else {
            panic!("the cut reading must decode");
        };
        assert_eq!(cut.azimuths.len(), MAX_READING_LINES);
        assert_eq!(cut.names.len(), MAX_READING_LINES);
        assert_eq!(cut.names[0], "p");
        assert!(cut.names[1..].iter().all(String::is_empty));
        // The index stays as the server wrote it: the sky area tests it against the lines.
        assert_eq!(cut.spawn, Some(MAX_READING_LINES + 3));
        assert_eq!(cut.time, 5);
    }

    fn every_amulet_signal() -> Vec<AmuletSignal> {
        vec![
            AmuletSignal::Open { pos: [1, -64, 30_000_000], name: None, used: 0, max: 5, bed: true },
            AmuletSignal::Open { pos: [-7, 70, 2], name: Some("Home: étoile".to_owned()), used: 5, max: 5, bed: false },
            AmuletSignal::List { max: 16, points: vec![] },
            AmuletSignal::List { max: 1, points: vec![("A".to_owned(), [1, 2, 3]), (String::new(), [-4, 5, -6])] },
            AmuletSignal::Cooldown { ticks: 40 },
        ]
    }

    fn every_amulet_request() -> Vec<AmuletRequest> {
        vec![
            AmuletRequest::Save { pos: [1, 2, 3], name: "  Home ".to_owned() },
            AmuletRequest::Save { pos: [0, 0, 0], name: String::new() },
            AmuletRequest::Remove { pos: [-1, -2, -3] },
            AmuletRequest::Delete { pos: [i32::MIN, 0, i32::MAX] },
        ]
    }

    #[test]
    fn every_amulet_message_round_trips_and_refuses_cut_input() {
        for signal in every_amulet_signal() {
            let bytes = signal.encode();
            assert_eq!(AmuletSignal::decode(&bytes), Some(signal.clone()));
            for cut in 0..bytes.len() {
                assert_eq!(AmuletSignal::decode(&bytes[..cut]), None, "{signal:?} cut at {cut}");
            }
            let mut long = bytes.clone();
            long.push(0);
            assert_eq!(AmuletSignal::decode(&long), None);
        }
        for request in every_amulet_request() {
            let bytes = request.encode();
            assert_eq!(AmuletRequest::decode(&bytes), Some(request.clone()));
            for cut in 0..bytes.len() {
                assert_eq!(AmuletRequest::decode(&bytes[..cut]), None, "{request:?} cut at {cut}");
            }
        }
    }

    #[test]
    fn the_amulet_tags_are_fixed_and_bad_input_gives_none() {
        assert_eq!(AmuletRequest::Remove { pos: [1, 0, 0] }.encode(), [2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(AmuletRequest::decode(&[]), None);
        assert_eq!(AmuletRequest::decode(&[4]), None);
        assert_eq!(AmuletSignal::decode(&[4]), None);
        assert_eq!(AmuletSignal::Cooldown { ticks: 40 }.encode(), [3, 40, 0, 0, 0]);
        // A list that says it holds more points than the cap is refused before any allocation.
        let mut w = Writer::new();
        w.u8(2).i32(5).u32(u32::MAX);
        assert_eq!(AmuletSignal::decode(w.bytes()), None);
        // A list longer than the cap is cut when it is written.
        let points = vec![("p".to_owned(), [0, 0, 0]); MAX_LIST_POINTS + 3];
        let cut = AmuletSignal::decode(&AmuletSignal::List { max: 5, points }.encode());
        assert!(matches!(cut, Some(AmuletSignal::List { points, .. }) if points.len() == MAX_LIST_POINTS));
        assert_eq!(AMULET_CHANNEL, crate::ids::channel("amulet"));
    }

    fn every_signal() -> Vec<JourneySignal> {
        vec![
            JourneySignal::Departure,
            JourneySignal::Ready,
            JourneySignal::Arrival { name: None },
            JourneySignal::Arrival {
                name: Some("Steve".to_owned()),
            },
        ]
    }

    #[test]
    fn every_journey_signal_round_trips() {
        for signal in every_signal() {
            assert_eq!(
                JourneySignal::decode(&signal.encode()),
                Some(signal.clone())
            );
        }
    }

    #[test]
    fn the_journey_tags_are_fixed() {
        assert_eq!(JourneySignal::Departure.encode(), [1]);
        assert_eq!(JourneySignal::Ready.encode(), [2]);
        assert_eq!(JourneySignal::Arrival { name: None }.encode(), [3, 0]);
        assert_eq!(
            JourneySignal::Arrival {
                name: Some("A".to_owned())
            }
            .encode(),
            [3, 1, 1, 0, 0, 0, b'A']
        );
        assert_eq!(RespawnRequest::Held.encode(), [1]);
    }

    #[test]
    fn a_bad_journey_signal_gives_none() {
        assert_eq!(JourneySignal::decode(&[]), None);
        assert_eq!(JourneySignal::decode(&[0]), None);
        assert_eq!(JourneySignal::decode(&[4]), None);
        assert_eq!(JourneySignal::decode(&[1, 0]), None);
        for signal in every_signal() {
            let bytes = signal.encode();
            for cut in 0..bytes.len() {
                assert_eq!(
                    JourneySignal::decode(&bytes[..cut]),
                    None,
                    "{signal:?} cut at {cut}"
                );
            }
        }
    }

    #[test]
    fn the_respawn_request_round_trips_and_refuses_bad_bytes() {
        let held = RespawnRequest::Held;
        assert_eq!(RespawnRequest::decode(&held.encode()), Some(held));
        assert_eq!(RespawnRequest::decode(&[]), None);
        assert_eq!(RespawnRequest::decode(&[2]), None);
        assert_eq!(RespawnRequest::decode(&[1, 1]), None);
    }

    #[test]
    fn the_channels_follow_the_area_names() {
        assert_eq!(JOURNEY_CHANNEL, crate::ids::channel("journey"));
        assert_eq!(RESPAWN_CHANNEL, crate::ids::channel("respawn"));
    }

    fn sample() -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(7)
            .u32(4_000_000_000)
            .i32(-123_456)
            .i64(-9_000_000_000_000)
            .f32(1.5)
            .f64(-30_000_000.25)
            .bool(true)
            .bool(false)
            .str("Home: étoile");
        w.finish()
    }

    #[test]
    fn every_primitive_round_trips() {
        let bytes = sample();
        let mut r = Reader::new(&bytes);
        assert_eq!(r.u8(), Some(7));
        assert_eq!(r.u32(), Some(4_000_000_000));
        assert_eq!(r.i32(), Some(-123_456));
        assert_eq!(r.i64(), Some(-9_000_000_000_000));
        assert_eq!(r.f32(), Some(1.5));
        assert_eq!(r.f64(), Some(-30_000_000.25));
        assert_eq!(r.bool(), Some(true));
        assert_eq!(r.bool(), Some(false));
        assert_eq!(r.str(), Some("Home: étoile"));
        assert!(r.is_done());
        assert_eq!(r.u8(), None);
    }

    #[test]
    fn special_floats_round_trip_bit_for_bit() {
        let mut w = Writer::new();
        w.f64(f64::NAN).f32(f32::NEG_INFINITY).f64(-0.0);
        let bytes = w.finish();
        let mut r = Reader::new(&bytes);
        assert!(r.f64().is_some_and(f64::is_nan));
        assert_eq!(r.f32(), Some(f32::NEG_INFINITY));
        assert_eq!(r.f64().map(f64::to_bits), Some((-0.0f64).to_bits()));
    }

    #[test]
    fn every_truncation_gives_none_and_never_panics() {
        let bytes = sample();
        for cut in 0..bytes.len() {
            let mut r = Reader::new(&bytes[..cut]);
            let all = (|| {
                r.u8()?;
                r.u32()?;
                r.i32()?;
                r.i64()?;
                r.f32()?;
                r.f64()?;
                r.bool()?;
                r.bool()?;
                r.str()
            })();
            assert_eq!(all, None, "cut at {cut}");
        }
    }

    #[test]
    fn a_short_read_of_each_type_gives_none() {
        assert_eq!(Reader::new(&[]).u8(), None);
        assert_eq!(Reader::new(&[1, 2, 3]).u32(), None);
        assert_eq!(Reader::new(&[1, 2, 3]).i32(), None);
        assert_eq!(Reader::new(&[0; 7]).i64(), None);
        assert_eq!(Reader::new(&[0; 3]).f32(), None);
        assert_eq!(Reader::new(&[0; 7]).f64(), None);
        assert_eq!(Reader::new(&[]).bool(), None);
        assert_eq!(Reader::new(&[5, 0, 0, 0, b'a']).str(), None);
    }

    #[test]
    fn a_bad_bool_gives_none() {
        assert_eq!(Reader::new(&[2]).bool(), None);
        assert_eq!(Reader::new(&[255]).bool(), None);
    }

    #[test]
    fn a_string_over_the_cap_gives_none() {
        let mut w = Writer::new();
        w.u32((MAX_STR + 1) as u32);
        let mut bytes = w.finish();
        bytes.extend(std::iter::repeat_n(b'a', MAX_STR + 1));
        assert_eq!(Reader::new(&bytes).str(), None);
        let mut w = Writer::new();
        w.u32(u32::MAX);
        assert_eq!(Reader::new(w.bytes()).str(), None);
    }

    #[test]
    fn a_string_that_is_not_utf8_gives_none() {
        let bytes = [2, 0, 0, 0, 0xff, 0xfe];
        assert_eq!(Reader::new(&bytes).str(), None);
    }

    #[test]
    fn the_writer_cuts_a_long_string_on_a_char_boundary() {
        let long = "é".repeat(MAX_STR);
        let mut w = Writer::new();
        w.str(&long);
        let bytes = w.finish();
        let mut r = Reader::new(&bytes);
        let back = r.str();
        assert!(back.is_some_and(|s| s.len() <= MAX_STR && long.starts_with(s)));
        assert!(r.is_done());
    }

    #[test]
    fn remaining_counts_the_unread_bytes() {
        let bytes = sample();
        let mut r = Reader::new(&bytes);
        assert_eq!(r.remaining(), bytes.len());
        r.u8();
        assert_eq!(r.remaining(), bytes.len() - 1);
    }
}
