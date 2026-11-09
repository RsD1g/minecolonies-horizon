// Right-click a citizen with the build tool to give them the next open job.
fn minecolonies_interact(
    game: &mut crate::game::Game<'_>,
    entity: i32,
    kind: &str,
    stack: Option<&horizon_client::items::ItemStack>,
    claimed: &mut bool,
) {
    use crate::game::minecolonies as m;
    if !game.native_on(m::ID) || *claimed || entity < 0 {
        return;
    }
    if kind != "minecraft:villager" {
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
    // `item` is already a string reference. `str::as_str` is still unstable on the launcher's Rust 1.99.
    let item = stack.map(|stack| stack.item.as_ref()).unwrap_or("");
    let Some(gesture) = m::gesture_entity(&actor, entity as u32, item) else {
        return;
    };
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
