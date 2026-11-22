use mod_api::{
    ConfigLoadingEvent, ConfigReloadingEvent, Flow, Hooks, HostSide, LevelLoadEvent,
    LivingDeathEvent, PlayerCommandEvent, RegisterCommandsEvent, RegisterConfigsEvent,
    RunModCommandEvent, ServerHost,
};

mod settings;
mod totem;

const ID: &str = "bmh";

#[derive(Default)]
struct Bmh;

#[mod_api::horizon_mod]
impl Hooks for Bmh {
    fn living_death(&self, host: &mut dyn ServerHost, event: &mut LivingDeathEvent<'_>) -> Flow {
        totem::on_death(host, event)
    }

    fn register_commands(&self, _host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        settings::register_commands(event);
    }

    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        settings::run_command(host, event)
    }

    fn player_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut PlayerCommandEvent<'_>,
    ) -> Flow {
        settings::rule_alias(host, event)
    }

    fn level_load(&self, host: &mut dyn ServerHost, event: &mut LevelLoadEvent<'_>) {
        if event.dimension == "minecraft:overworld" {
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
