use crate::logic::{eternal, ids};
use mod_api::{
    Compound, Flow, Hand, Host, HostSide, InteractionResult, ItemKey, ItemPart, ItemStack,
    ItemStackedOnOtherEvent, PlayerRef, RandomSource, Registrar, ServerHost, SoundSource, StatType,
    Tag, UseItemEvent,
};
use std::sync::OnceLock;

struct Keys {
    bundle: ItemKey,
    refused: Vec<ItemKey>,
}

static KEYS: OnceLock<Keys> = OnceLock::new();

pub(in crate::combat) fn register(registry: &mut dyn Registrar) {
    let bundle = registry.item_key(ids::ETERNAL_BUNDLE);
    let refused = ids::ETERNAL_ITEMS
        .into_iter()
        .chain(["minecraft:ender_chest"])
        .map(|id| registry.item_key(id))
        .collect();
    let _ = KEYS.set(Keys { bundle, refused });
    registry.item_part(bundle, BundlePart);
}

pub(crate) fn on_stacked(event: &mut ItemStackedOnOtherEvent<'_, '_>) -> Flow {
    if event.event.primary {
        return Flow::Continue;
    }
    let Some(keys) = KEYS.get() else {
        return Flow::Continue;
    };
    let (Some(carried), Some(clicked)) = (event.event.carried.get(), event.event.clicked.item())
    else {
        return Flow::Continue;
    };
    if (carried.item() == keys.bundle
        && carried.count() == 1
        && keys.refused.contains(&clicked.item()))
        || (clicked.item() == keys.bundle
            && clicked.count() == 1
            && event.event.clicked.allow_modification()
            && keys.refused.contains(&carried.item()))
    {
        event.event.cancellation_result = true;
        return Flow::Cancel;
    }
    Flow::Continue
}

pub(in crate::combat) fn return_refused(
    host: &mut dyn ServerHost,
    player: PlayerRef,
    slot: u32,
    stack: &ItemStack,
) {
    let (Some(keys), Some(contents)) = (KEYS.get(), contents_of(host, stack)) else {
        return;
    };
    let (refused, mut keep): (Vec<_>, Vec<_>) = contents
        .into_iter()
        .partition(|item| keys.refused.contains(&item.item()));
    if refused.is_empty() || !set_contents(host, player, slot, stack, &keep) {
        return;
    }
    let count = keep.len();
    for item in refused {
        if player.give_stack(host, item.clone()).is_err() {
            keep.push(item);
        }
    }
    if keep.len() != count {
        set_contents(host, player, slot, stack, &keep);
    }
}

struct BundlePart;

#[mod_api::horizon_part]
impl ItemPart for BundlePart {
    fn use_item(&self, host: HostSide, event: &mut UseItemEvent<'_>) -> Option<InteractionResult> {
        let server = match host {
            HostSide::Server(host) => host,
            HostSide::Client(_) => {
                return Some(if holds_items(event.stack) {
                    InteractionResult::SuccessServer
                } else {
                    InteractionResult::Pass
                });
            }
        };
        let player = server.player(event.player)?;
        let slot = match event.hand {
            Hand::Main => u32::try_from(player.selected_slot(server)?).ok()?,
            Hand::Off => 40,
        };
        let Some(contents) = contents_of(server, event.stack) else {
            return Some(InteractionResult::Pass);
        };
        if contents.is_empty() {
            return Some(InteractionResult::Pass);
        }
        if !set_contents(server, player, slot, event.stack, &[]) {
            return Some(InteractionResult::Fail);
        }
        spill(server, player, slot, event.stack, contents);
        Some(InteractionResult::SuccessServer)
    }
}

fn holds_items(bundle: &ItemStack) -> bool {
    matches!(super::component(bundle, "minecraft:bundle_contents"), Some(Tag::List(items)) if !items.is_empty())
}

fn contents_of(host: &dyn Host, bundle: &ItemStack) -> Option<Vec<ItemStack>> {
    match super::component(bundle, "minecraft:bundle_contents") {
        Some(Tag::List(items)) => items
            .iter()
            .map(|tag| match tag {
                Tag::Compound(item) => ItemStack::from_nbt(host, item),
                _ => None,
            })
            .collect(),
        _ => Some(Vec::new()),
    }
}

fn set_contents(
    host: &mut dyn ServerHost,
    player: PlayerRef,
    slot: u32,
    original: &ItemStack,
    contents: &[ItemStack],
) -> bool {
    let Some(contents) = contents
        .iter()
        .map(|stack| stack.to_nbt(host).map(Tag::Compound))
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    let mut bundle = original.clone();
    let mut patch = Compound::new();
    patch.insert("minecraft:bundle_contents".into(), Tag::List(contents));
    bundle.apply_components(&patch).is_ok()
        && player.set_inventory_slot(host, slot, Some(bundle)).is_ok()
}

fn spill(
    host: &mut dyn ServerHost,
    player: PlayerRef,
    slot: u32,
    bundle: &ItemStack,
    contents: Vec<ItemStack>,
) {
    let entity = player.entity();
    let (Some(level), Some(pos), Some(eye_height), Some([yaw, pitch])) = (
        player.level(host),
        entity.position(host),
        entity.eye_height(host),
        entity.rotation(host),
    ) else {
        set_contents(host, player, slot, bundle, &contents);
        return;
    };
    let [x, y, z] = eternal::drop_position(pos, eye_height);
    let count = contents.len();
    let mut remaining = Vec::new();
    for item in contents {
        let velocity = level.with_random(host, |random| {
            eternal::drop_velocity(yaw, pitch, true, &mut || random.next_double())
        });
        if let Ok(Some(dropped)) = level.spawn_item(
            host,
            x,
            y,
            z,
            item.clone(),
            eternal::DROP_PICKUP_DELAY as u32,
        ) {
            if let Some(velocity) = velocity {
                dropped.set_velocity(host, velocity);
            }
        } else {
            remaining.push(item);
        }
    }
    if !remaining.is_empty() {
        set_contents(host, player, slot, bundle, &remaining);
    }
    if remaining.len() < count {
        player.award_stat(host, StatType::Used, ids::ETERNAL_BUNDLE, 1);
        let pitch = level
            .with_random(host, |random| {
                eternal::bundle_sound_pitch(random.next_double())
            })
            .unwrap_or(1.0);
        level.play_sound(
            host,
            eternal::BUNDLE_DROP_SOUND,
            pos,
            SoundSource::Players,
            eternal::BUNDLE_SOUND_VOLUME,
            pitch,
        );
    }
}
