use crate::logic::journey as logic;
use crate::logic::wire::{JOURNEY_CHANNEL, JourneySignal, RESPAWN_CHANNEL};
use mod_api::client::hud_layout::PoseOp;
use mod_api::text::TextContent;
use mod_api::{
    ClientHost, DrawContext, Flow, GuiDraw, Music, MusicEvent, PayloadId, PlayerRef, Registrar,
    ScreenKeyPressedPreEvent, ScreenMouseButtonPressedPreEvent, ServerHost, SoundPlay, SoundSource,
    Text, ToClient, ToServer, WorldEpoch,
};
use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const HOLD_SCREEN: &str = "bmh:journey";
const PERFORM_RESPAWN: u8 = 1;

struct Channels {
    signal: ToClient<Vec<u8>>,
    respawn: ToServer<u8>,
}

static CHANNELS: Mutex<Option<Channels>> = Mutex::new(None);
static REQUESTS: Mutex<BTreeMap<u128, PlayerRef>> = Mutex::new(BTreeMap::new());

struct Client {
    journey: logic::Client,
    clock: Option<Instant>,
    world: Option<WorldEpoch>,
    position: Option<[f64; 3]>,
    size: [f32; 2],
    serial: i64,
}

static CLIENT: Mutex<Client> = Mutex::new(Client {
    journey: logic::Client::new(),
    clock: None,
    world: None,
    position: None,
    size: [320.0, 240.0],
    serial: 0,
});

fn lock<T>(state: &Mutex<T>) -> MutexGuard<'_, T> {
    state.lock().unwrap_or_else(|error| error.into_inner())
}

impl Client {
    fn now(&mut self) -> i64 {
        let clock = self.clock.get_or_insert_with(Instant::now);
        i64::try_from(clock.elapsed().as_millis()).unwrap_or(i64::MAX)
    }

    fn seed(&mut self) -> i64 {
        self.serial = self.serial.wrapping_add(1);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos() as i64);
        nanos ^ self.serial.wrapping_mul(0x5DEE_CE66D)
    }

    fn set_world(&mut self, world: WorldEpoch) -> bool {
        if self.world == Some(world) {
            return false;
        }
        self.world = Some(world);
        self.position = None;
        self.journey.leave_world()
    }
}

pub(super) fn register(registrar: &mut dyn Registrar) {
    let mut event = registrar.payload_handlers();
    let mut payloads = event.registrar("1").optional();
    let signal = payloads.play_to_client_typed::<Vec<u8>>(
        PayloadId(JOURNEY_CHANNEL.into()),
        |data, context| {
            let Some(signal) = JourneySignal::decode(&data) else {
                return Ok(());
            };
            let Ok(player) = context.player() else {
                return Ok(());
            };
            let host = context
                .client_host()
                .map_err(|error| format!("{error:?}"))?;
            receive(host, player, signal);
            Ok(())
        },
    );
    let respawn = payloads.play_to_server_typed::<u8>(
        PayloadId(RESPAWN_CHANNEL.into()),
        |request, context| {
            if request == PERFORM_RESPAWN
                && let Ok(player) = context.player()
            {
                lock(&REQUESTS).insert(player.uuid(), player);
            }
            Ok(())
        },
    );
    *lock(&CHANNELS) = Some(Channels { signal, respawn });
}

pub(super) fn send(host: &mut dyn ServerHost, player: PlayerRef, signal: JourneySignal) {
    let token = lock(&CHANNELS)
        .as_ref()
        .map(|channels| channels.signal.clone());
    if let Some(token) = token {
        let _ = token.send(host, player, &signal.encode());
    }
}

pub(super) fn server_tick(host: &mut dyn ServerHost) {
    let requests = std::mem::take(&mut *lock(&REQUESTS));
    for player in requests.into_values() {
        // A dead player can be absent from the level's entity index.
        // The respawn API checks the connected player's state itself.
        if crate::respawn::awaiting_respawn(player.uuid())
            && let Err(error) = player.respawn(host)
        {
            eprintln!("[BMH] Journey respawn request failed: {error:?}");
        }
    }
}

fn playing(screen: Option<&str>) -> bool {
    matches!(screen, None | Some("playing" | "chat"))
}

fn receive(host: &mut dyn ClientHost, player: PlayerRef, signal: JourneySignal) {
    let screen = host.observed_screen_id();
    let playing = playing(screen.as_ref().map(|screen| screen.id.as_str()));
    let release = {
        let mut state = lock(&CLIENT);
        let release = state.set_world(player.entity().epoch());
        let now = state.now();
        let seed = state.seed();
        state.journey.signal(now, signal, playing, seed);
        release
    };
    if release {
        host.mod_screen_close(HOLD_SCREEN);
    }
}

pub(super) fn disconnect() {
    let mut state = lock(&CLIENT);
    state.journey.leave_world();
    state.position = None;
    state.world = None;
}

pub(super) fn owned_hold(host: &dyn ClientHost, screen: &str) -> bool {
    screen == "mod" && host.mod_screen() == Some(HOLD_SCREEN) && lock(&CLIENT).journey.holds_input()
}

pub(super) fn click(host: &dyn ClientHost, event: &ScreenMouseButtonPressedPreEvent<'_>) -> Flow {
    if owned_hold(host, event.screen.id) {
        return Flow::Cancel;
    }
    if event.screen.id != "dead" {
        return Flow::Continue;
    }
    let width = host
        .text_width(&Text::translate(logic::CONFIRM_TITLE_KEY, vec![]))
        .unwrap_or(0) as f32;
    let mut state = lock(&CLIENT);
    let now = state.now();
    let seed = state.seed();
    let [_, _, w, h] = event.screen.rect;
    if state
        .journey
        .death_click(now, event.button, event.screen.mouse, [w, h], width, seed)
    {
        Flow::Cancel
    } else {
        Flow::Continue
    }
}

pub(super) fn key(host: &dyn ClientHost, event: &ScreenKeyPressedPreEvent<'_>) -> Flow {
    if owned_hold(host, event.screen.id) {
        return Flow::Cancel;
    }
    if event.screen.id != "dead" {
        return Flow::Continue;
    }
    let mut state = lock(&CLIENT);
    let now = state.now();
    let seed = state.seed();
    if state
        .journey
        .death_key(now, event.key, event.modifiers, seed)
    {
        Flow::Cancel
    } else {
        Flow::Continue
    }
}

pub(super) fn frame(host: &mut dyn ClientHost) {
    let player = host.local_player();
    let screen = host.observed_screen_id();
    let screen = screen.as_ref().map(|screen| screen.id.as_str());
    let hold_open = host.mod_screen() == Some(HOLD_SCREEN);
    let covered = !playing(screen) && !hold_open && screen != Some("dead");
    let position = player.and_then(|player| player.entity().position(host));
    let chunk = player.is_some_and(|player| {
        player.level(host).is_some_and(|level| {
            player
                .entity()
                .block_position(host)
                .is_some_and(|pos| level.is_loaded(host, pos))
        })
    });
    let gui_scale = host.gui_scale().unwrap_or(1).max(1);
    let (commands, at, release) = {
        let mut state = lock(&CLIENT);
        let release = player.is_some_and(|player| state.set_world(player.entity().epoch()));
        if let Some(position) = position {
            state.position = Some(position);
        }
        let view = logic::View {
            now: state.now(),
            in_world: state.world.is_some(),
            death_screen: screen == Some("dead"),
            covered,
            loading: position.is_none() || covered,
            chunk,
            grid: logic::grid(
                f64::from(gui_scale),
                state
                    .size
                    .map(|axis| (axis * gui_scale as f32).ceil() as u32),
            ),
            seed: state.seed(),
        };
        (state.journey.frame(&view), state.position, release)
    };
    // The server sends Ready when its search ends. The held click needs no early request.
    if commands.send_respawn {
        let token = lock(&CHANNELS)
            .as_ref()
            .map(|channels| channels.respawn.clone());
        if let Some(token) = token
            && let Err(error) = token.send(host, &PERFORM_RESPAWN)
        {
            eprintln!("[BMH] Could not send the journey respawn request: {error:?}");
        }
    }
    if let Some(at) = at {
        for sound in commands.sounds {
            let play = SoundPlay::at(sound.id, SoundSource::Players, at)
                .with_volume(sound.volume)
                .with_pitch(sound.pitch);
            host.play_local_sound(&play, 0);
        }
    }
    if release || commands.release {
        host.mod_screen_close(HOLD_SCREEN);
    } else if commands.hold && !hold_open {
        host.mod_screen_open(HOLD_SCREEN);
    }
}

pub(super) fn draw(host: &dyn ClientHost, draw: &mut dyn DrawContext) {
    let [width, height] = draw.size();
    let picture = {
        let mut state = lock(&CLIENT);
        state.size = [width, height];
        state.journey.picture()
    };
    let Some(picture) = picture else {
        return;
    };
    if let Some(color) = picture.fade {
        draw.fill([0.0, 0.0, width, height], color);
    }
    let scale = host.gui_scale().unwrap_or(1).max(1) as f32;
    let pixel = picture.grid.pixel as f32 / scale;
    for fill in picture.sky.iter().chain(&picture.art) {
        draw.fill(
            [
                fill.x0 as f32 * pixel,
                fill.y0 as f32 * pixel,
                (fill.x1 - fill.x0) as f32 * pixel,
                (fill.y1 - fill.y0) as f32 * pixel,
            ],
            fill.argb,
        );
    }
    if let Some(message) = picture.message {
        let text = Text::new(TextContent::Translatable {
            key: logic::MESSAGE_KEY.into(),
            fallback: Some(logic::MESSAGE_FALLBACK.into()),
            arguments: vec![Text::literal(message.name.clone())],
        });
        let text_width = host.text_width(&text).unwrap_or(0);
        let layout = logic::message_layout(text_width, width as i32, height as i32);
        let _ = draw.gui_paint(&[
            GuiDraw::Push(vec![
                PoseOp::Translate(layout.translate[0], layout.translate[1]),
                PoseOp::Scale(layout.scale, layout.scale),
            ]),
            GuiDraw::Text {
                text,
                x: layout.at[0],
                y: layout.at[1],
                color: message.color(),
                shadow: false,
            },
            GuiDraw::Pop,
        ]);
    }
}

pub(super) fn music(event: &mut MusicEvent<'_>) {
    let action = {
        let mut state = lock(&CLIENT);
        let now = state.now();
        state.journey.music(now)
    };
    match action {
        logic::MusicAction::Keep => {}
        logic::MusicAction::Scale(part) => event.volume *= part,
        logic::MusicAction::Cut => {
            *event.music = Some(Music::new(
                logic::MUSIC_CUT,
                logic::MUSIC_CUT_DELAY,
                logic::MUSIC_CUT_DELAY,
                true,
            ));
        }
        logic::MusicAction::Quiet => *event.music = None,
    }
}
