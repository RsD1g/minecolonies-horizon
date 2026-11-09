// Using the supply camp in the air founds the colony in front of you.
fn minecolonies_use_item(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
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
    match m::held(&actor, &stack.item, None) {
        m::Held::Ignore => {}
        m::Held::Tool => {
            *claimed = true;
            let _ = game.native_action_bar(
                actor.uuid,
                horizon_client::text::Text::literal("/colony build <hut>  ·  /colony help"),
            );
        }
        m::Held::Rally { lines } => {
            *claimed = true;
            for line in lines {
                game.native_feedback(horizon_client::text::Text::literal(line));
            }
            if m::with_world(|world| world.is_dirty()) {
                m::save_now(dir.as_deref());
            }
        }
        m::Held::Camp { lines, consume, item } => {
            *claimed = true;
            if consume {
                let _ = game.native_take_items(item, 1);
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
