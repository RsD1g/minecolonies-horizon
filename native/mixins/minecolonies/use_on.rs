// Right-click a block: a hut block places its blueprint, the build tool places or opens that hut.
fn minecolonies_use_on(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    pos: [i32; 3],
    face: [i32; 3],
    claimed: &mut bool,
) {
    use crate::game::minecolonies as m;
    if !game.native_on(m::ID) || *claimed {
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
    let door = [pos[0] + face[0], pos[1] + face[1], pos[2] + face[2]];
    let block = game.native_block(pos);
    let Some(gesture) = m::gesture(&actor, &stack.item, Some((door, face)), Some(block.as_str()))
    else {
        return;
    };
    if let Some(item) = gesture.consume {
        let _ = game.native_take_items(item, 1);
    }
    if gesture.bar {
        if let Some(line) = gesture.lines.first() {
            let _ = game.native_action_bar(actor.uuid, horizon_client::text::Text::literal(line));
        }
    } else {
        for line in &gesture.lines {
            game.native_feedback(horizon_client::text::Text::literal(line));
        }
    }
    if m::with_world(|world| world.is_dirty()) {
        m::save_now(dir.as_deref());
    }
    *claimed = true;
}
