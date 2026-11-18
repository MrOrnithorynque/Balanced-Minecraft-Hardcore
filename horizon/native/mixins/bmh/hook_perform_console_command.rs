// Hook `perform_console_command`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_perform_console_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_settings::on_perform_console_command(game, command, ran);
}
