// The server checks the inventory with the player swapped in for their turn.
// The damage state machine still decides whether this hit needs a totem.
use super::{Body, Game, ItemStack};
use bmh_logic::{ids, totem};
use horizon_client::mod_api::NativeEffect;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::ops::ControlFlow::{self, Break, Continue};

thread_local! {
    // A guest turn can contain another guest's turn. Each has its own inventory.
    static ARMED: RefCell<BTreeSet<u128>> = const { RefCell::new(BTreeSet::new()) };
}

fn inventory_totems<'a>(game: &'a Game<'_>) -> impl Iterator<Item = (i32, &'a ItemStack)> {
    let main = game
        .inventory
        .iter()
        .enumerate()
        .map(|(i, stack)| (i as i32, stack));
    let armor = game
        .armor
        .iter()
        .enumerate()
        .rev()
        .map(|(i, stack)| (103 - i as i32, stack));
    main.chain(armor)
        .chain(std::iter::once((-106, &game.offhand)))
        .filter_map(|(slot, stack)| {
            let stack = stack.as_ref()?;
            (stack.count > 0 && totem::is_totem(&stack.item)).then_some((slot, stack))
        })
}

fn hand_protects(game: &Game<'_>) -> bool {
    [game.held(), game.offhand.as_ref()]
        .into_iter()
        .flatten()
        .any(|stack| stack.count > 0 && game.has_death_protection(stack))
}

pub(super) fn update_ready(game: &mut Game<'_>) {
    let Some(uuid) = game.lan_acting_uuid else {
        return;
    };
    if !game.native_on(crate::game::bmh::ID)
        || !game.player.as_ref().is_some_and(|p| !p.health.totem_ready)
    {
        return;
    }
    if inventory_totems(game).next().is_some() {
        game.player.as_mut().unwrap().health.totem_ready = true;
        ARMED.with_borrow_mut(|armed| armed.insert(uuid));
    }
}

pub(super) fn spend(game: &mut Game<'_>) -> ControlFlow<Option<Body>> {
    let Some(uuid) = game.lan_acting_uuid else {
        return Continue(());
    };
    if !ARMED.with_borrow_mut(|armed| armed.remove(&uuid)) {
        return Continue(());
    }
    let Some(player) = game.player.as_ref() else {
        return Continue(());
    };
    if !player.health.totem_used || hand_protects(game) {
        return Continue(());
    }
    let slot = inventory_totems(game).next().map(|(slot, _)| slot);
    let player = game.player.as_mut().unwrap();
    player.health.totem_used = false;
    if player.health.dead {
        // A second hit bypassed protection before the turn ended.
        return Break(None);
    }
    let Some(slot) = slot else {
        // The totem left during the turn. The lethal hit must still kill the player.
        player.health.dead = true;
        return Break(None);
    };
    let _ = player.set_health(totem::SAVE_HEALTH, false);
    let at = [player.position.x, player.position.y, player.position.z];
    let (width, height) = (
        f64::from(player.dimensions.width),
        f64::from(player.dimensions.height),
    );
    let stack = game.native_carried_stack(slot).expect("live totem slot");
    let protector = std::mem::replace(stack, ItemStack::new(ids::SLEEPING_TOTEM, 1));
    game.award_stat("used", &protector.item, 1);
    game.post_game_event(
        "minecraft:item_interact_finish",
        at,
        game.player_context(None),
    );
    let effects = totem::SAVE_EFFECTS.map(|effect| NativeEffect {
        id: effect.id.to_owned(),
        amplifier: effect.amplifier,
        duration: effect.duration,
        ambient: false,
        particles: true,
    });
    game.native_set_effects(uuid, &effects);
    if let Some(name) = game.lan_acting_name.as_deref() {
        crate::game::bmh_totem::record_save(name);
    }
    Break(Some(Body {
        at,
        width,
        height,
        protector: Some(protector),
    }))
}

pub(super) fn animation(game: &Game<'_>) -> ControlFlow<ItemStack> {
    if !game.native_on(crate::game::bmh::ID) || hand_protects(game) {
        return Continue(());
    }
    if let Some((_, stack)) = inventory_totems(game).next() {
        return Break(stack.clone());
    }
    if [game.held(), game.offhand.as_ref()]
        .into_iter()
        .flatten()
        .any(|stack| stack.item.as_ref() == ids::SLEEPING_TOTEM)
    {
        return Break(ItemStack::new(ids::TOTEM_OF_ETERNITY, 1));
    }
    Continue(())
}
