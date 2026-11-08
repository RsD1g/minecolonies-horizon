// Other players do not break blocks inside a colony's claim.
fn minecolonies_guard(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    state: horizon_world::registry::BlockStateId,
    canceled: &mut bool,
) {
    use crate::game::minecolonies as m;
    let _ = state;
    if !game.native_on(m::ID) || *canceled {
        return;
    }
    let Some(player) = game.native_acting_player() else {
        return;
    };
    m::ensure_loaded(game.native_world_dir().as_deref());
    let message = m::with_world(|world| world.protect(&player.dimension, pos, player.uuid));
    if let Some(message) = message {
        *canceled = true;
        let _ = game.native_action_bar(player.uuid, horizon_client::text::Text::literal(message));
    }
}
