// AmuletCompass.calculate uses a target one million blocks north of this model's owner.
fn bmh_amulet_north(
    model: &Resolve<'_>,
    wobble: bool,
    target: CompassTarget,
    state: &NeedleState,
) -> core::ops::ControlFlow<f32> {
    use core::ops::ControlFlow::{Break, Continue};
    if target != CompassTarget::Lodestone || model.stack.item != "bmh:navigator_amulet" {
        return Continue(());
    }
    let (Some(owner), Some(level)) = (model.ctx.owner, model.ctx.level) else { return Continue(()) };
    if level.dimension != "minecraft:overworld" {
        return Break(model.compass(wobble, CompassTarget::None, state));
    }
    let [x, _, z] = owner.position;
    let dx = x.floor() + 0.5 - x;
    let dz = (z - 1_000_000.0).floor() + 0.5 - z;
    let bearing = (dz.atan2(dx) / f64::from(std::f32::consts::PI * 2.0)) as f32;
    let yaw = positive_modulo(owner.visual_rotation_y_degrees / 360.0, 1.0);
    let angle = if owner.is_local_player && level.runs_normally {
        let mut state = state.0.lock().unwrap_or_else(|e| e.into_inner());
        let needle = &mut state[0];
        if needle.should_update(wobble, level.game_time) {
            needle.update(wobble, 0.8, level.game_time, 0.5 - (yaw - 0.25));
        }
        bearing + needle.value(wobble)
    } else {
        0.5 - (yaw - 0.25 - bearing)
    };
    Break(positive_modulo(angle, 1.0))
}
