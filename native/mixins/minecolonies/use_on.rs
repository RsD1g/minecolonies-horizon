// The supply camp founds a colony on the face you click. The build tool explains itself.
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
    match m::held(&actor, &stack.item, Some((door, face))) {
        m::Held::Ignore => {}
        m::Held::Tool => {
            *claimed = true;
            let _ = game.native_action_bar(
                actor.uuid,
                horizon_client::text::Text::literal("/colony build <hut>  ·  /colony help"),
            );
        }
        m::Held::Camp { lines, consume } => {
            *claimed = true;
            if consume {
                let _ = game.native_take_items("minecolonies:supplycamp", 1);
            }
            for line in lines {
                game.native_feedback(horizon_client::text::Text::literal(line));
            }
            if m::with_world(|world| world.is_dirty()) {
                m::save_now(dir.as_deref());
            }
        }
    }
}
