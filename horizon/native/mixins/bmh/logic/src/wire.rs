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

#[cfg(test)]
mod tests {
    use super::*;

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
