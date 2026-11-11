// Hook `render_hand_pose`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_render_hand_pose(
    game: &crate::game::Game<'_>,
    hand: horizon_client::first_person::Hand,
    stack: Option<&horizon_client::items::ItemStack>,
    partial: f32,
    swing: f32,
    equip: f32,
    x: &mut f32,
    y: &mut f32,
    z: &mut f32,
    pitch: &mut f32,
    yaw: &mut f32,
    roll: &mut f32,
    scale: &mut f32,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_ritual::on_render_hand_pose(
        game, hand, stack, partial, swing, equip, x, y, z, pitch, yaw, roll, scale,
    );
}
