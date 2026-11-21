// The host couples blast damage to knockback. Apply its knockback once to each protected drop.
fn bmh_blast_impulse(
    game: &mut Game<'_>,
    center: [f64; 3],
    radius: f32,
    interaction: BlockInteraction,
    blocks: &[[i32; 3]],
) {
    use horizon_client_lib::bmh_ground_items::set_filter;
    set_filter(None);
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    if radius >= 1.0e-5 && !items_ignore_explosion(game.ui.rules.bool("mob_griefing"), interaction) {
        let blast = Blast { vehicles_ignore: false, items_ignore: false, spared: None };
        for i in 0..game.dropped.len() {
            let item = &game.dropped[i];
            if !bmh_logic::ids::is_eternal(&item.stack.item) {
                continue;
            }
            if let Some(hit) = game.blast_hit(item, center, f64::from(radius * 2.0), &blast) {
                let id = item.id;
                let mut push = [hit.push.x, hit.push.y, hit.push.z];
                game.native_explosion_knockback(game.dim.id(), center, blocks, Some(id), &mut push);
                game.dropped[i].push_from_explosion(Vec3::new(push[0], push[1], push[2]));
            }
        }
    }
    set_filter(Some(bmh_logic::ids::is_eternal));
}
