// The server side of a ledger button.
fn minecolonies_gui_button(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    id: i32,
) {
    use crate::game::minecolonies as m;
    let block = game.native_block(pos);
    let name = block.split('[').next().unwrap_or(block.as_str());
    if name != "minecolonies:ledger" || !game.native_on(m::ID) {
        return;
    }
    let Some(player) = game.native_acting_player() else {
        return;
    };
    let dir = game.native_world_dir();
    m::ensure_loaded(dir.as_deref());
    let lang = m::Lang::parse(game.native_setting(m::ID, "lang").as_deref());
    m::with_world(|world| world.set_lang(lang));
    let actor = m::actor_of(
        player.uuid,
        &player.name,
        &player.dimension,
        player.position,
        player.yaw,
    );
    let lines = m::with_world(|world| world.ledger_press(&actor, id));
    for line in lines {
        game.native_feedback(horizon_client::text::Text::literal(line));
    }
    if m::with_world(|world| world.is_dirty()) {
        m::save_now(dir.as_deref());
    }
}
