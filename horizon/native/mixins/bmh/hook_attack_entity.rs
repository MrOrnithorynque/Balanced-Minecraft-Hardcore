// Capture the weapon before the server processes this attack.
fn bmh_hook_attack_entity(game: &mut crate::game::Game<'_>, entity: i32, kind: &str, canceled: bool) {
    crate::game::bmh_tools::on_attack_entity(game, entity, kind, canceled);
}
