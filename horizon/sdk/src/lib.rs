use mod_api::{
    AnvilUpdateEvent, AttackEntityEvent, BlockDropsEvent, ClientHost, ConfigLoadingEvent,
    ConfigReloadingEvent, EntityJoinLevelEvent, Flow, Hooks, HostSide, ItemExpireEvent,
    ItemPickedUpEvent, ItemSpawnEvent, ItemStackedOnOtherEvent, LevelHost, LevelLoadEvent,
    LevelRandomHost, LevelTickPostEvent, LivingDeathEvent, LivingDropsEvent,
    LivingIncomingDamageEvent, LivingUseTotemEvent, LoopContext, PlayerBedLeaveEvent,
    PlayerBreakSpeedEvent, PlayerCommandEvent, PlayerDiedEvent, PlayerLoggedOutEvent,
    PlayerRespawnPositionEvent, PlayerRespawnedEvent, PlayerTickPostEvent, PlayerTotemEvent,
    RegisterCommandsEvent, RegisterConfigsEvent, Registrar, RegistrationContext,
    RunModCommandEvent, ServerHost, ServerStoppedEvent, ServerTickEvent, ServerTickPhase,
    SubmitCustomGeometryEvent, ValidateTicketsEvent,
};

// The same logic also runs as a separate test crate.
#[allow(dead_code)]
#[path = "logic/lib.rs"]
mod logic;

mod amulet;
mod combat;
mod homecoming;
mod journey;
mod respawn;
mod ritual;
mod settings;
mod totem;
mod totem_placement;

const ID: &str = "bmh";

#[derive(Default)]
struct Bmh;

#[mod_api::horizon_mod]
impl Hooks for Bmh {
    fn register(&self, registrar: &mut dyn Registrar, _context: &RegistrationContext) {
        respawn::register(registrar);
        journey::register(registrar);
        combat::register(registrar);
        amulet::register(registrar);
        ritual::register(registrar);
        totem_placement::register(registrar);
    }

    fn living_death(&self, host: &mut dyn ServerHost, event: &mut LivingDeathEvent<'_>) -> Flow {
        if matches!(totem::on_death(host, event), Flow::Cancel) {
            return Flow::Cancel;
        }
        respawn::record_inventory(host, event);
        if event.kind == "minecraft:player"
            && let Some(player) = respawn::player_by_entity(host, event.entity)
        {
            combat::forget(player.uuid());
            combat::eternal_support::record_death(host, player);
        }
        Flow::Continue
    }

    fn living_use_totem(
        &self,
        _level: &mut dyn LevelRandomHost,
        event: &mut LivingUseTotemEvent<'_>,
    ) -> Flow {
        totem::inspect_hand(event);
        Flow::Continue
    }

    fn player_totem(&self, host: &mut dyn ServerHost, event: &mut PlayerTotemEvent<'_>) {
        totem::used_hand_item(host, event.player);
    }

    fn item_picked_up(&self, host: &mut dyn ServerHost, event: &mut ItemPickedUpEvent<'_>) {
        combat::eternal_support::on_pickup(host, event);
    }

    fn player_bed_leave(&self, host: &mut dyn ServerHost, event: &mut PlayerBedLeaveEvent<'_>) {
        combat::rest::on_bed_leave(host, event.player);
    }

    fn player_died(&self, host: &mut dyn ServerHost, event: &mut PlayerDiedEvent<'_>) {
        respawn::on_player_died(host, event.player);
    }

    fn player_logged_out(&self, host: &mut dyn ServerHost, event: &mut PlayerLoggedOutEvent<'_>) {
        respawn::logout(host, event.player);
        combat::forget(event.player);
        amulet::logout(event.player);
        ritual::logout(event.player);
    }

    fn player_respawn_position(
        &self,
        host: &mut dyn ServerHost,
        event: &mut PlayerRespawnPositionEvent<'_>,
    ) {
        respawn::on_position(host, event);
    }

    fn level_tick_post(&self, host: HostSide<'_>, event: &mut LevelTickPostEvent) {
        if let HostSide::Server(host) = host {
            respawn::on_level_tick(host, event.level);
            combat::level_tick(host, event.level);
            if host.level("minecraft:overworld") == Some(event.level) {
                amulet::award_points(host);
            }
        }
    }

    fn living_incoming_damage(
        &self,
        _host: &mut dyn LevelRandomHost,
        event: &mut LivingIncomingDamageEvent<'_>,
    ) -> Flow {
        if event.kind == "minecraft:player"
            && respawn::blocks_damage(event.entity, event.source.damage_type)
        {
            return Flow::Cancel;
        }
        combat::incoming_damage(event);
        Flow::Continue
    }

    fn validate_tickets(&self, _host: &mut dyn ServerHost, event: &mut ValidateTicketsEvent<'_>) {
        respawn::validate_tickets(event);
    }

    fn server_tick(&self, host: &mut dyn ServerHost, event: &mut ServerTickEvent<'_>) {
        ritual::server_tick(host, event);
        if matches!(
            event.phase,
            ServerTickPhase::ServerTickEnd {
                world_running: true
            }
        ) {
            journey::server_tick(host);
            respawn::end_tick(host);
            totem::end_tick(host);
            combat::tick(host);
        }
    }

    fn client_tick_post(
        &self,
        host: &mut dyn mod_api::ClientHost,
        _event: &mut mod_api::ClientTickPostEvent,
    ) {
        amulet::client::tick(host);
    }

    fn client_level_unload(
        &self,
        _host: &mut dyn mod_api::ClientHost,
        _event: &mut mod_api::ClientLevelUnloadEvent<'_>,
    ) {
        amulet::client::unload();
    }

    fn screen_render_post(
        &self,
        host: &dyn mod_api::ClientHost,
        event: &mut mod_api::ScreenRenderPostEvent<'_>,
    ) {
        amulet::client::screen(host, event);
    }

    fn screen_mouse_button_pressed_pre(
        &self,
        host: &dyn mod_api::ClientHost,
        event: &mut mod_api::ScreenMouseButtonPressedPreEvent<'_>,
    ) -> Flow {
        if matches!(journey::click(host, event), Flow::Cancel) {
            return Flow::Cancel;
        }
        amulet::client::click(host, event)
    }

    fn screen_mouse_button_released_pre(
        &self,
        host: &dyn mod_api::ClientHost,
        event: &mut mod_api::ScreenMouseButtonReleasedPreEvent<'_>,
    ) -> Flow {
        if journey::owned_hold(host, event.screen.id) {
            return Flow::Cancel;
        }
        amulet::client::release(host, event)
    }

    fn screen_mouse_dragged_pre(
        &self,
        host: &dyn mod_api::ClientHost,
        event: &mut mod_api::ScreenMouseDraggedPreEvent<'_>,
    ) -> Flow {
        if journey::owned_hold(host, event.screen.id) {
            return Flow::Cancel;
        }
        amulet::client::drag(host, event)
    }

    fn screen_key_pressed_pre(
        &self,
        host: &dyn mod_api::ClientHost,
        event: &mut mod_api::ScreenKeyPressedPreEvent<'_>,
    ) -> Flow {
        if matches!(journey::key(host, event), Flow::Cancel) {
            return Flow::Cancel;
        }
        amulet::client::key(host, event)
    }

    fn screen_char_typed_pre(
        &self,
        host: &dyn mod_api::ClientHost,
        event: &mut mod_api::ScreenCharTypedPreEvent<'_>,
    ) -> Flow {
        if journey::owned_hold(host, event.screen.id) {
            return Flow::Cancel;
        }
        amulet::client::character(host, event)
    }

    fn render_level_stage(
        &self,
        host: &dyn mod_api::ClientHost,
        event: &mut mod_api::RenderLevelStageEvent<'_>,
    ) {
        amulet::client::draw_sky(host, event);
    }

    fn gui_render_post(
        &self,
        host: &dyn mod_api::ClientHost,
        event: &mut mod_api::GuiRenderPostEvent<'_>,
    ) {
        amulet::client::spyglass(host, event);
        journey::draw(host, event.batch);
    }

    fn render_frame_pre(
        &self,
        host: &mut dyn ClientHost,
        _event: &mut mod_api::RenderFramePreEvent,
    ) {
        journey::frame(host);
    }

    fn client_disconnecting(
        &self,
        _host: &dyn ClientHost,
        _event: &mut mod_api::ClientDisconnectingEvent,
    ) {
        journey::disconnect();
    }

    fn music(&self, _host: &dyn ClientHost, event: &mut mod_api::MusicEvent<'_>) {
        journey::music(event);
    }

    fn register_range_select_item_model_property(
        &self,
        _context: &mod_api::RegistrationContext,
        event: &mut mod_api::RegisterRangeSelectItemModelPropertyEvent<'_>,
    ) {
        event.properties.push(amulet::client::NORTH.into());
    }

    fn item_model_range_property(
        &self,
        host: &dyn mod_api::ClientHost,
        event: &mut mod_api::ItemModelRangePropertyEvent<'_>,
    ) {
        amulet::client::compass(host, event);
    }

    fn anvil_update(&self, host: HostSide, event: &mut AnvilUpdateEvent<'_>) -> Flow {
        match host {
            HostSide::Server(host) => {
                ritual::anvil_update(HostSide::Server(host), event);
                combat::anvil_update(HostSide::Server(host), event);
            }
            HostSide::Client(host) => {
                ritual::anvil_update(HostSide::Client(host), event);
                combat::anvil_update(HostSide::Client(host), event);
            }
        }
        Flow::Continue
    }

    fn living_entity_use_item_start(
        &self,
        host: HostSide,
        event: &mut mod_api::LivingEntityUseItemStartEvent<'_>,
    ) -> Flow {
        ritual::start(host, event)
    }

    fn living_entity_use_item_tick(
        &self,
        host: HostSide,
        event: &mut mod_api::LivingEntityUseItemTickEvent<'_>,
    ) -> Flow {
        ritual::tick(host, event)
    }

    fn living_entity_use_item_stop(
        &self,
        host: HostSide,
        event: &mut mod_api::LivingEntityUseItemStopEvent<'_>,
    ) -> Flow {
        ritual::stop(host, event);
        Flow::Continue
    }

    fn living_entity_use_item_finish(
        &self,
        host: HostSide,
        event: &mut mod_api::LivingEntityUseItemFinishEvent<'_>,
    ) {
        ritual::finish(host, event);
    }

    fn living_damage_post(
        &self,
        _host: &mut dyn mod_api::LevelRandomHost,
        event: &mut mod_api::LivingDamagePostEvent<'_>,
    ) {
        ritual::damage_post(event);
        combat::damage_post(event);
    }

    fn render_hand(
        &self,
        host: &dyn mod_api::ClientHost,
        event: &mut mod_api::RenderHandEvent<'_, '_>,
    ) -> Flow {
        ritual::render_hand(host, event)
    }

    fn attack_entity(&self, host: &mut dyn ServerHost, event: &mut AttackEntityEvent<'_>) -> Flow {
        combat::on_attack(host, event);
        Flow::Continue
    }

    fn player_tick_post(&self, host: HostSide, event: &mut PlayerTickPostEvent) {
        if let HostSide::Server(host) = host {
            combat::track_fall(host, event.player);
            combat::eternal_support::enforce_inventory(host, event.player);
        }
    }

    fn server_stopped(&self, _host: &mut dyn ServerHost, _event: &mut ServerStoppedEvent) {
        combat::clear();
    }

    fn item_expire(&self, host: &mut dyn ServerHost, event: &mut ItemExpireEvent<'_>) {
        combat::item_expire(host, event);
    }

    fn item_spawn(&self, host: &mut dyn ServerHost, event: &mut ItemSpawnEvent<'_>) {
        combat::item_spawn(host, event);
    }

    fn living_drops(&self, host: &mut dyn ServerHost, event: &mut LivingDropsEvent<'_>) -> Flow {
        combat::eternal_support::on_living_drops(host, event);
        Flow::Continue
    }

    fn player_respawned(&self, host: &mut dyn ServerHost, event: &mut PlayerRespawnedEvent<'_>) {
        combat::eternal_support::on_respawn(host, event.player);
    }

    fn player_break_speed(&self, host: HostSide, event: &mut PlayerBreakSpeedEvent) -> Flow {
        combat::break_speed(host, event);
        Flow::Continue
    }

    fn block_drops(&self, level: &dyn LevelHost, event: &mut BlockDropsEvent<'_>) -> Flow {
        combat::ore::block_drops(level, event);
        Flow::Continue
    }

    fn submit_custom_geometry(
        &self,
        host: &dyn ClientHost,
        event: &mut SubmitCustomGeometryEvent<'_>,
    ) {
        combat::ore::draw(host, event);
    }

    fn entity_join_level(
        &self,
        host: &mut dyn ServerHost,
        event: &mut EntityJoinLevelEvent<'_>,
    ) -> Flow {
        combat::entity_join_level(host, event);
        Flow::Continue
    }

    fn item_stacked_on_other(
        &self,
        _context: &dyn LoopContext,
        event: &mut ItemStackedOnOtherEvent<'_, '_>,
    ) -> Flow {
        combat::bundle::on_stacked(event)
    }

    fn register_commands(&self, _host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        settings::register_commands(event);
        amulet::register_commands(event);
    }

    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        if matches!(amulet::command(host, event), Flow::Cancel) {
            return Flow::Cancel;
        }
        settings::run_command(host, event)
    }

    fn player_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut PlayerCommandEvent<'_>,
    ) -> Flow {
        settings::player_command(host, event)
    }

    fn level_load(&self, host: &mut dyn ServerHost, event: &mut LevelLoadEvent<'_>) {
        if event.dimension == "minecraft:overworld" {
            respawn::reset();
            totem::reset();
            settings::load_world(host);
        }
    }

    fn register_configs(&self, _host: HostSide, event: &mut RegisterConfigsEvent<'_>) {
        settings::register_configs(event);
    }

    fn config_loading(&self, host: HostSide, event: &mut ConfigLoadingEvent<'_>) {
        settings::apply_config(host, event.config);
    }

    fn config_reloading(&self, host: HostSide, event: &mut ConfigReloadingEvent<'_>) {
        settings::apply_config(host, event.config);
    }
}
