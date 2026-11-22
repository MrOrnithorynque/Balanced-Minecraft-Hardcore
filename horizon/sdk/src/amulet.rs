use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::logic::amulet as logic;
use crate::logic::contract::{AmuletPoint, AmuletReading};
use crate::logic::wire::{AMULET_CHANNEL, AmuletRequest, AmuletSignal, SKY_CHANNEL, SkySignal};
use mod_api::{
    ClientHost, Command, CommandArgumentValue, CommandSource, Compound, Flow, Hand, Host, HostSide,
    IPayloadContext, IPayloadHandler, InteractionResult, InventoryTickEvent, ItemBarEvent,
    ItemPart, ItemStack, LevelRef, LightLayer, PayloadBytes, PayloadHandleError, PayloadId,
    PlayerRef, RegisterCommandsEvent, Registrar, RunModCommandEvent, ServerHost, SoundSource, Tag,
    Text, TooltipEvent, UseItemEvent, UseOnEvent,
};

pub(super) mod client;
mod store;

const OVERWORLD: &str = crate::logic::respawn::OVERWORLD;
const CUSTOM_DATA: &str = "minecraft:custom_data";
const ENCHANTMENTS: &str = "minecraft:enchantments";
const OFF_HAND_SLOT: u32 = 40;

static LAST_CHANGE: Mutex<BTreeMap<u128, i64>> = Mutex::new(BTreeMap::new());
static AWARDS: Mutex<Vec<(u128, &'static str)>> = Mutex::new(Vec::new());

struct Amulet;

#[mod_api::horizon_part]
impl ItemPart for Amulet {
    fn use_item(&self, host: HostSide, event: &mut UseItemEvent<'_>) -> Option<InteractionResult> {
        Some(match host {
            HostSide::Client(_) => InteractionResult::Consume,
            HostSide::Server(host) => use_in_air(host, event)?,
        })
    }

    fn use_on(&self, host: HostSide, event: &mut UseOnEvent<'_>) -> Option<InteractionResult> {
        let view: &dyn Host = match &host {
            HostSide::Server(host) => &**host,
            HostSide::Client(host) => &**host,
        };
        let player = view.player(event.player)?;
        let level = player.level(view)?;
        let block = block_at(view, level, event.pos)?;
        if !player.is_secondary_use_active(view).unwrap_or(false) || !block.can_hold {
            return None;
        }
        if let HostSide::Server(host) = host {
            if in_overworld(host, player) {
                let points = points(host, player);
                let name = logic::point_at(&points, event.pos, block.half)
                    .and_then(|index| points.get(index))
                    .map(|point| point.name.clone());
                send_signal(
                    host,
                    player,
                    AmuletSignal::Open {
                        pos: event.pos,
                        name,
                        used: points.len().try_into().unwrap_or(i32::MAX),
                        max: max_points(),
                        bed: block.bed,
                    },
                );
                cooldown(host, player, event.stack, logic::POINT_COOLDOWN_TICKS);
            } else {
                tell(host, player, logic::KEY_OVERWORLD, []);
            }
        }
        Some(InteractionResult::Success)
    }

    fn inventory_tick(&self, host: &mut dyn ServerHost, event: &mut InventoryTickEvent<'_>) {
        let Some(player) = host.players().acting_player() else {
            return;
        };
        let Some(level) = player.level(host) else {
            return;
        };
        if !level.game_time(host).is_some_and(logic::refill_tick) {
            return;
        }
        let Some(day) = level.day_time(host) else {
            return;
        };
        let mut stack = event.stack.clone();
        drop_legacy_data(&mut stack);
        if let Some(charges) = logic::refill(stack_charges(&stack), day) {
            write_charges(&mut stack, charges);
        }
        if &stack != event.stack {
            let slot = match event.slot {
                -106 => Some(OFF_HAND_SLOT),
                100..=103 => u32::try_from(event.slot - 64).ok(),
                slot => u32::try_from(slot).ok(),
            };
            if let Some(slot) = slot {
                let _ = player.set_inventory_slot(host, slot, Some(stack));
            }
        }
    }

    fn tooltip(&self, _host: &dyn ClientHost, event: &mut TooltipEvent<'_>) {
        let mut starlight = Text::translate(
            logic::KEY_STARLIGHT,
            vec![
                Text::literal(logic::charges(stack_charges(event.stack)).to_string()),
                Text::literal(logic::MAX_CHARGES.to_string()),
            ],
        );
        starlight.style.color = Some(format!("#{:06x}", logic::STARLIGHT_COLOR));
        event.lines.push((starlight, u32::MAX));
        for key in [
            logic::KEY_TOOLTIP_READ,
            logic::KEY_TOOLTIP_HOME,
            logic::KEY_TOOLTIP_LIST,
        ] {
            let mut hint = Text::translate(key, vec![]);
            hint.style.color = Some("gray".into());
            event.lines.push((hint, u32::MAX));
        }
    }

    fn item_bar(&self, _host: &dyn ClientHost, event: &mut ItemBarEvent<'_>) {
        *event.bar = Some((
            logic::bar_width(logic::charges(stack_charges(event.stack))),
            logic::STARLIGHT_COLOR,
        ));
    }
}

pub(super) fn register(registrar: &mut dyn Registrar) {
    let item = registrar.item_key(logic::AMULET);
    registrar.item_part(item, Amulet);
    registrar
        .payload_handlers()
        .registrar("1")
        .play_bidirectional(
            PayloadId(AMULET_CHANNEL.into()),
            Arc::new(ServerRequest),
            Some(Arc::new(ClientSignal)),
        )
        .play_to_client(PayloadId(SKY_CHANNEL.into()), Some(Arc::new(ClientSky)));
}

pub(super) fn points(host: &mut dyn ServerHost, player: PlayerRef) -> Vec<AmuletPoint> {
    store::load(host, player).unwrap_or_default()
}

fn in_overworld(host: &dyn Host, player: PlayerRef) -> bool {
    player
        .dimension(host)
        .is_some_and(|id| id.namespace() == "minecraft" && id.path() == "overworld")
}

struct PointBlock {
    can_hold: bool,
    bed: bool,
    half: Option<logic::BedHalf>,
}

fn block_at(host: &dyn Host, level: LevelRef, pos: [i32; 3]) -> Option<PointBlock> {
    let block = level.block(host, pos)?;
    let name = block.id(host);
    let bed = host
        .block_in_tag(block, logic::BEDS_TAG)
        .unwrap_or_else(|| logic::is_vanilla_bed(&name));
    let half = if bed {
        let part = level.block_property(host, pos, "part");
        let facing = level.block_property(host, pos, "facing");
        logic::bed_half(part.as_deref(), facing.as_deref())
    } else {
        None
    };
    Some(PointBlock {
        can_hold: bed || name == logic::LODESTONE,
        bed,
        half,
    })
}

pub(super) fn sky(host: &dyn Host, player: PlayerRef) -> Option<logic::Sky> {
    let level = player.level(host)?;
    let entity = player.entity();
    let [x, y, z] = entity.position(host)?;
    let eye_y = y + entity.eye_height(host)?;
    let eye = logic::block_of([x, eye_y, z]);
    let wet = logic::block_of([x, eye_y - logic::FLUID_EYE_OFFSET, z]);
    let wet_block = level
        .block(host, wet)
        .map(|block| block.id(host))
        .unwrap_or_default();
    let eye_block = level
        .block(host, eye)
        .map(|block| block.id(host))
        .unwrap_or_default();
    let waterlogged = level.block_property(host, wet, "waterlogged").as_deref() == Some("true");
    let (rain, thunder) = level.weather(host)?;
    let living = host.living(entity)?;
    let blinded = logic::BLINDING_EFFECTS
        .iter()
        .filter_map(|id| host.effect_key(id))
        .any(|effect| living.effect(host, effect).is_some());
    Some(logic::Sky {
        overworld: in_overworld(host, player),
        rain,
        thunder,
        day_time: level.day_time(host)?,
        sky_light: level
            .brightness(host, LightLayer::Sky, eye)
            .unwrap_or_default() as u8,
        blinded,
        eyes_covered: logic::holds_water(&wet_block, waterlogged)
            || logic::holds_lava(&wet_block)
            || logic::is_powder_snow(&eye_block),
    })
}

fn cooldown(host: &mut dyn ServerHost, player: PlayerRef, stack: &ItemStack, ticks: i64) {
    let group = host.item_cooldown_group(stack);
    if let Ok(ticks) = i32::try_from(ticks) {
        player.set_item_cooldown(host, &group, ticks);
    }
}

fn use_in_air(host: &mut dyn ServerHost, event: &UseItemEvent<'_>) -> Option<InteractionResult> {
    let player = host.player(event.player)?;
    let level = player.level(host)?;
    let now = level.game_time(host)?;
    let day = level.day_time(host)?;
    let sneaking = player.is_secondary_use_active(host).unwrap_or(false);
    let hidden = if sneaking {
        None
    } else {
        logic::hidden_reason(&sky(host, player)?, logic::HIDDEN_PREFIX)
    };
    let stored = stack_charges(event.stack);
    let refilled = logic::refill(stored, day);
    let current = refilled.unwrap_or(stored);
    let creative = player.is_creative(host).unwrap_or(false);
    let slot = match event.hand {
        Hand::Main => player.selected_slot(host)?.try_into().ok()?,
        Hand::Off => OFF_HAND_SLOT,
    };
    match logic::use_in_air(
        false,
        false,
        sneaking,
        hidden,
        logic::charges(current),
        creative,
    ) {
        logic::Use::Ignore => None,
        logic::Use::List => {
            send_list(host, player);
            cooldown(host, player, event.stack, logic::POINT_COOLDOWN_TICKS);
            Some(InteractionResult::SuccessServer)
        }
        logic::Use::Hidden(key) => {
            tell(host, player, &key, []);
            Some(InteractionResult::Fail)
        }
        logic::Use::Empty => {
            if let Some(full) = refilled {
                let mut stack = event.stack.clone();
                write_charges(&mut stack, full);
                let _ = player.set_inventory_slot(host, slot, Some(stack));
            }
            tell(host, player, logic::KEY_EMPTY, []);
            Some(InteractionResult::Fail)
        }
        logic::Use::Read => {
            let at = player.entity().position(host)?;
            let spawn = level.respawn_data(host).map(|spawn| spawn.pos);
            let reading = logic::reading(at, &points(host, player), spawn, now);
            let mut stack = event.stack.clone();
            if let Some(charges) = logic::charges_after_reading(current, creative, day).or(refilled)
            {
                write_charges(&mut stack, charges);
            }
            write_reading(&mut stack, &reading);
            let remaining = logic::charges(stack_charges(&stack));
            if let Err(error) = player.set_inventory_slot(host, slot, Some(stack)) {
                eprintln!("[BMH] Could not save the amulet reading: {error:?}");
                return Some(InteractionResult::Fail);
            }
            if let Some(name) = player.name(host) {
                eprintln!(
                    "[BMH] Star reading for {name}: azimuths {:?}, {remaining} charges left",
                    reading.azimuths,
                );
            }
            let message = SkySignal::Reading(reading);
            send(host, player, SKY_CHANNEL, message.encode());
            queue_award(player, logic::STARGAZER);
            chime(host, level, at, logic::READING_PITCH);
            tell(
                host,
                player,
                logic::KEY_READING,
                logic::block_of(at).map(|n| Text::literal(n.to_string())),
            );
            cooldown(host, player, event.stack, logic::READING_COOLDOWN_TICKS);
            Some(InteractionResult::SuccessServer)
        }
    }
}

fn tell<const N: usize>(host: &mut dyn ServerHost, player: PlayerRef, key: &str, args: [Text; N]) {
    let _ = player.action_bar(host, Text::translate(key, Vec::from(args)));
}

fn send_signal(host: &mut dyn ServerHost, player: PlayerRef, message: AmuletSignal) {
    send(host, player, AMULET_CHANNEL, message.encode());
}

fn send(host: &mut dyn ServerHost, player: PlayerRef, channel: &str, bytes: Vec<u8>) {
    if let Err(error) = player.send_payload(host, &PayloadId(channel.into()), PayloadBytes(bytes)) {
        eprintln!("[BMH] Could not send {channel}: {error:?}");
    }
}

fn send_list(host: &mut dyn ServerHost, player: PlayerRef) {
    let points = points(host, player)
        .into_iter()
        .map(|point| (point.name, point.pos))
        .collect();
    send_signal(
        host,
        player,
        AmuletSignal::List {
            max: max_points(),
            points,
        },
    );
}

fn max_points() -> i32 {
    crate::settings::config().max_amulet_points
}

fn chime(host: &mut dyn ServerHost, level: LevelRef, at: [f64; 3], pitch: f32) {
    level.play_sound(
        host,
        logic::CHIME_SOUND,
        at,
        SoundSource::Players,
        logic::CHIME_VOLUME,
        pitch,
    );
}

fn queue_award(player: PlayerRef, path: &'static str) {
    AWARDS
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .push((player.uuid(), path));
}

pub(super) fn award_points(host: &mut dyn ServerHost) {
    let awards = std::mem::take(&mut *AWARDS.lock().unwrap_or_else(|error| error.into_inner()));
    for (player, path) in awards {
        let Some(name) = host.player(player).and_then(|p| p.name(host)) else {
            continue;
        };
        if name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            let _ = host
                .commands()
                .run_command(&format!("advancement grant {name} only bmh:{path}"));
        }
    }
}

pub(super) fn logout(player: u128) {
    LAST_CHANGE
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .remove(&player);
    AWARDS
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .retain(|(uuid, _)| *uuid != player);
}

pub(super) fn register_commands(event: &mut RegisterCommandsEvent<'_>) {
    for usage in logic::COMMAND_USAGES {
        event
            .commands
            .push(Command::op(usage, logic::COMMAND_LEVEL));
    }
}

pub(super) fn command(host: &mut dyn ServerHost, event: &RunModCommandEvent<'_>) -> Flow {
    let mut literals = event.command.literals.iter().map(String::as_str);
    if literals.next() != Some("bmh") || literals.next() != Some("point") {
        return Flow::Continue;
    }
    let CommandSource::Player { player, .. } = *event.source else {
        return Flow::Cancel;
    };
    let pos = match event.command.argument("pos") {
        Some(CommandArgumentValue::BlockPos(pos)) => Some(*pos),
        _ => None,
    };
    let change = match (literals.next(), pos) {
        (Some("save"), Some(pos)) => {
            let Some(CommandArgumentValue::String(name)) = event.command.argument("name") else {
                return Flow::Cancel;
            };
            logic::Change::Save {
                pos,
                name: name.clone(),
            }
        }
        (Some("remove"), Some(pos)) => logic::Change::Remove { pos },
        (Some("delete"), _) => {
            let coordinate = |name| match event.command.argument(name) {
                Some(CommandArgumentValue::Integer(value)) => Some(*value),
                _ => None,
            };
            let (Some(x), Some(y), Some(z)) = (coordinate("x"), coordinate("y"), coordinate("z"))
            else {
                return Flow::Cancel;
            };
            logic::Change::Delete { pos: [x, y, z] }
        }
        _ => return Flow::Cancel,
    };
    change_point(host, player, change);
    Flow::Cancel
}

fn change_point(host: &mut dyn ServerHost, player: PlayerRef, change: logic::Change) {
    let Some(level) = player.level(host) else {
        return;
    };
    let Some(position) = player.entity().position(host) else {
        return;
    };
    let Some(now) = level.game_time(host) else {
        return;
    };
    let Some(living) = host.living(player.entity()) else {
        return;
    };
    let holds_amulet = Hand::BOTH
        .into_iter()
        .filter_map(|hand| living.hand_item(host, hand))
        .any(|stack| stack.item().id(host) == logic::AMULET);
    let asker = logic::Asker {
        holds_amulet,
        overworld: in_overworld(host, player),
        alive: living.alive(host).unwrap_or(false),
        spectator: player.is_spectator(host).unwrap_or(true),
        position,
    };
    if let logic::Change::Delete { pos } = change {
        if !asker.holds_amulet {
            tell(host, player, logic::KEY_NO_AMULET, []);
            return;
        }
        let Some(mut points) = store::load(host, player) else {
            return;
        };
        if let Some(removed) = logic::remove_exact(&mut points, pos)
            && store::save(host, player, points)
        {
            tell(
                host,
                player,
                logic::KEY_REMOVED,
                [Text::literal(removed.name)],
            );
            send_list(host, player);
        }
        return;
    }
    let pos = match change {
        logic::Change::Save { pos, .. } | logic::Change::Remove { pos } => pos,
        logic::Change::Delete { .. } => return,
    };
    if !level.is_loaded(host, pos) {
        let inside = level
            .height(host)
            .is_some_and(|(min, height)| pos[1] >= min && pos[1] < min.saturating_add(height));
        let key = if inside {
            "argument.pos.unloaded"
        } else {
            "argument.pos.outofworld"
        };
        let mut line = Text::translate(key, vec![]);
        line.style.color = Some("red".into());
        let _ = host.commands().feedback(line);
        return;
    }
    let block = block_at(host, level, pos);
    let can_hold = block.as_ref().is_some_and(|block| block.can_hold);
    let last = LAST_CHANGE
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(&player.uuid())
        .copied();
    if let Err(refusal) = logic::check_change(&asker, pos, can_hold, last, now) {
        if let Some(key) = refusal.key() {
            tell(host, player, key, []);
        }
        return;
    }
    let half = block.and_then(|block| block.half);
    let Some(mut points) = store::load(host, player) else {
        return;
    };
    match change {
        logic::Change::Save { name, .. } => {
            let name = logic::clean_name(&name);
            let renamed = logic::point_at(&points, pos, half).is_some();
            let max = max_points();
            if let Err(stop) = logic::check_save(&name, renamed, points.len(), max) {
                if stop == logic::SaveStop::Full {
                    tell(host, player, stop.key(), [Text::literal(max.to_string())]);
                } else {
                    tell(host, player, stop.key(), []);
                }
                return;
            }
            let renamed = logic::put(&mut points, pos, half, &name);
            let count = points.len();
            if !store::save(host, player, points) {
                return;
            }
            for path in logic::awards_after_put(count) {
                queue_award(player, path);
            }
            chime(host, level, position, logic::SAVE_PITCH);
            let key = if renamed {
                logic::KEY_RENAMED
            } else {
                logic::KEY_SAVED
            };
            tell(host, player, key, [Text::literal(name)]);
        }
        logic::Change::Remove { .. } => {
            let Some(removed) = logic::remove_on_block(&mut points, pos, half) else {
                return;
            };
            if !store::save(host, player, points) {
                return;
            }
            tell(
                host,
                player,
                logic::KEY_REMOVED,
                [Text::literal(removed.name)],
            );
        }
        logic::Change::Delete { .. } => return,
    }
    LAST_CHANGE
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .insert(player.uuid(), now);
}

fn custom_data(stack: &ItemStack) -> Option<&Compound> {
    match stack.components()?.as_nbt().get(&CUSTOM_DATA.into())? {
        Tag::Compound(data) => Some(data),
        _ => None,
    }
}

fn stack_charges(stack: &ItemStack) -> logic::StackCharges {
    let data = custom_data(stack);
    logic::StackCharges {
        charges: data
            .and_then(|data| data.get(&logic::CHARGES_TAG.into()))
            .and_then(store::number)
            .map(|value| value.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32),
        night: data
            .and_then(|data| data.get(&logic::NIGHT_TAG.into()))
            .and_then(store::number),
    }
}

fn edit_custom_data(stack: &mut ItemStack, edit: impl FnOnce(&mut Compound)) {
    let mut patch = stack.components().map(Compound::from).unwrap_or_default();
    let mut data = custom_data(stack).cloned().unwrap_or_default();
    edit(&mut data);
    if data.is_empty() {
        patch.remove(&CUSTOM_DATA.into());
    } else {
        patch.remove(&"!minecraft:custom_data".into());
        patch.insert(CUSTOM_DATA.into(), Tag::Compound(data));
    }
    let _ = stack.set_components(patch);
}

fn write_charges(stack: &mut ItemStack, charges: logic::StackCharges) {
    edit_custom_data(stack, |data| {
        if let Some(count) = charges.charges {
            data.insert(logic::CHARGES_TAG.into(), Tag::Int(count));
        }
        if let Some(night) = charges.night {
            data.insert(logic::NIGHT_TAG.into(), Tag::Long(night));
        }
    });
}

fn write_reading(stack: &mut ItemStack, reading: &AmuletReading) {
    edit_custom_data(stack, |data| {
        let azimuths = reading
            .azimuths
            .iter()
            .map(|angle| Tag::Float(angle.to_bits()))
            .collect();
        let names = reading
            .names
            .iter()
            .map(|name| Tag::String(name.as_str().into()))
            .collect();
        let mut fields = Compound::from([
            (logic::AZIMUTHS_TAG.into(), Tag::List(azimuths)),
            (logic::NAMES_TAG.into(), Tag::List(names)),
            (logic::TIME_TAG.into(), Tag::Long(reading.time)),
        ]);
        if let Some(spawn) = reading.spawn.and_then(|index| i32::try_from(index).ok()) {
            fields.insert(logic::SPAWN_TAG.into(), Tag::Int(spawn));
        }
        data.insert(logic::READING_TAG.into(), Tag::Compound(fields));
    });
}

fn drop_legacy_data(stack: &mut ItemStack) {
    if custom_data(stack).is_some_and(|data| data.contains_key(&logic::LEGACY_LAST_USE_TAG.into()))
    {
        edit_custom_data(stack, |data| {
            data.remove(&logic::LEGACY_LAST_USE_TAG.into());
        });
    }
    if stack.damage() != 0 {
        stack.clear_damage();
    }
    let Some(mut patch) = stack.components().map(Compound::from) else {
        return;
    };
    if let Some(Tag::Compound(enchantments)) = patch.get_mut(&ENCHANTMENTS.into()) {
        let mut changed = false;
        for enchantment in logic::USELESS_ENCHANTMENTS {
            changed |= enchantments.remove(&enchantment.into()).is_some();
        }
        if changed {
            let _ = stack.set_components(patch);
        }
    }
}

struct ClientSignal;
struct ClientSky;
struct ServerRequest;

impl IPayloadHandler for ClientSignal {
    fn handle(
        &self,
        payload: PayloadBytes,
        context: &mut dyn IPayloadContext,
    ) -> Result<(), PayloadHandleError> {
        if let Some(signal) = AmuletSignal::decode(&payload.0)
            && let Ok(host) = context.client_host()
        {
            client::receive(host, signal);
        }
        Ok(())
    }
}

impl IPayloadHandler for ClientSky {
    fn handle(
        &self,
        payload: PayloadBytes,
        _context: &mut dyn IPayloadContext,
    ) -> Result<(), PayloadHandleError> {
        if let Some(SkySignal::Reading(reading)) = SkySignal::decode(&payload.0) {
            client::reading(reading);
        }
        Ok(())
    }
}

impl IPayloadHandler for ServerRequest {
    fn handle(
        &self,
        payload: PayloadBytes,
        context: &mut dyn IPayloadContext,
    ) -> Result<(), PayloadHandleError> {
        let Some(request) = AmuletRequest::decode(&payload.0) else {
            return Ok(());
        };
        let Ok(player) = context.player() else {
            return Ok(());
        };
        let Ok(host) = context.server_host() else {
            return Ok(());
        };
        let change = match request {
            AmuletRequest::Save { pos, name } => logic::Change::Save { pos, name },
            AmuletRequest::Remove { pos } => logic::Change::Remove { pos },
            AmuletRequest::Delete { pos } => logic::Change::Delete { pos },
        };
        change_point(host, player, change);
        Ok(())
    }
}
