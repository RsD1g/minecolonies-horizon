// /colony from a player. The reply is the command's own feedback.
fn minecolonies_command(
    game: &mut crate::game::Game<'_>,
    player: &horizon_client::mod_api::NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    use crate::game::minecolonies as m;
    if !game.native_on(m::ID) || !command.starts_with("colony") {
        return;
    }
    if command.len() > 6
        && !command
            .as_bytes()
            .get(6)
            .is_some_and(|b| b.is_ascii_whitespace())
    {
        return;
    }
    minecolonies_load(game);
    minecolonies_sync_lang(game);
    let actor = minecolonies_actor(player);
    let reply = m::with_world(|world| world.command(&actor, command));
    if !reply.handled {
        return;
    }
    *ran = true;
    match reply.order {
        m::Order::Deposit => {
            let mut any = false;
            for item in m::GOODS {
                let taken = game.native_take_items(item, 10_000);
                if taken == 0 {
                    continue;
                }
                any = true;
                let line = m::with_world(|world| world.give_stock(actor.uuid, item, taken));
                game.native_feedback(horizon_client::text::Text::literal(line));
            }
            if !any {
                let line = if m::with_world(|world| world.lang()) == m::Lang::Ru {
                    "В инвентаре нет товаров колонии."
                } else {
                    "Nothing in your pack is a colony good."
                };
                game.native_feedback(horizon_client::text::Text::literal(line));
            }
        }
        m::Order::Withdraw { item, count } => {
            let taken = m::with_world(|world| world.take_stock(actor.uuid, &item, count));
            let given = game.native_give_items(&item, taken);
            if given < taken {
                m::with_world(|world| world.give_stock(actor.uuid, &item, taken - given));
            }
            let line = if m::with_world(|world| world.lang()) == m::Lang::Ru {
                format!("Склад отдаёт вам {given}.")
            } else {
                format!("The warehouse hands you {given}.")
            };
            game.native_feedback(horizon_client::text::Text::literal(line));
        }
        m::Order::None => {}
    }
    for line in reply.lines {
        game.native_feedback(horizon_client::text::Text::literal(line));
    }
    if m::with_world(|world| world.is_dirty()) {
        minecolonies_store(game);
    }
}
