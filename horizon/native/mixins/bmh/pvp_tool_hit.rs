// The host confirms player hits here. Mob hits use the native hurt_enemy hook.
fn bmh_pvp_tool_hit(landed: bool, game: &mut Game<'_>, host: &mut LanHost, victim: Victim, strike: &Strike) -> bool {
    if landed && game.native_on(crate::game::bmh::ID) && let Some(attacker) = strike.attacker {
        let Victim::Guest(victim) = victim;
        if let (Some(attacker), Some(victim)) = (host.hub.guest(attacker), host.hub.guest(victim)) {
            crate::game::bmh_tools::on_pvp_hit(game, attacker, victim);
        }
    }
    landed
}
