// Hook `tab_complete_v2`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_tab_complete_v2(
    game: &mut crate::game::Game<'_>,
    label: &str,
    arguments: &[String],
    range: [usize; 2],
    suggestions: &mut Vec<horizon_client::mod_api::Suggestion>,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_settings::on_tab_complete_v2(game, label, arguments, range, suggestions);
}
