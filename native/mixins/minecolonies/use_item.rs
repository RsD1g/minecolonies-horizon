// Using the supply camp in the air founds the colony in front of you.
fn minecolonies_use_item(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    claimed: &mut bool,
) {
    minecolonies_hold(game, stack, claimed, None);
}

fn minecolonies_hold(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    claimed: &mut bool,
    clicked: Option<([i32; 3], [i32; 3])>,
) {
    use crate::game::minecolonies as m;
    if !game.native_on(m::ID) || *claimed {
        return;
    }
    let item = &*stack.item;
    let supply = item == "minecolonies:supplycamp";
    let tool = item == "minecolonies:buildtool";
    if !supply && !tool {
        return;
    }
    *claimed = true;
    let Some(player) = game.native_acting_player() else {
        return;
    };
    minecolonies_load(game);
    minecolonies_sync_lang(game);
    let actor = minecolonies_actor(&player);
    if tool {
        let _ = game.native_action_bar(
            actor.uuid,
            horizon_client::text::Text::literal("/colony build <hut>  ·  /colony help"),
        );
        return;
    }
    let (door, rot) = match clicked {
        Some((door, face)) => (door, m::rot_from_face(face, actor.yaw)),
        None => m::door_ahead(&actor, 5),
    };
    let had = m::with_world(|world| world.colonies_owned(actor.uuid));
    let reply = m::with_world(|world| world.found_at(&actor, "", door, rot));
    let has = m::with_world(|world| world.colonies_owned(actor.uuid));
    if !had && has {
        let _ = game.native_take_items("minecolonies:supplycamp", 1);
    }
    for line in reply.lines {
        game.native_feedback(horizon_client::text::Text::literal(line));
    }
    if m::with_world(|world| world.is_dirty()) {
        minecolonies_store(game);
    }
}
