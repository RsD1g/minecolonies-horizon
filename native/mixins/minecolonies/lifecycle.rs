// Load and store the colony in the world's own folder, so a backup keeps it.
fn minecolonies_world(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLifecycleContext,
    phase: horizon_client::mod_api::WorldPhase,
    error_text: &str,
) {
    use crate::game::minecolonies as m;
    use horizon_client::mod_api::WorldPhase;
    let _ = (context, error_text);
    if !game.native_on(m::ID) {
        return;
    }
    match phase {
        WorldPhase::WorldOpenReady => minecolonies_load(game),
        WorldPhase::WorldCloseFinal | WorldPhase::WorldCloseFailed => {
            minecolonies_store_and_drop(game)
        }
        _ => {}
    }
}

fn minecolonies_file(game: &crate::game::Game<'_>) -> Option<std::path::PathBuf> {
    Some(
        game.native_world_dir()?
            .join("minecolonies")
            .join("colony.json"),
    )
}

fn minecolonies_load(game: &mut crate::game::Game<'_>) {
    use crate::game::minecolonies as m;
    if m::world_ready() {
        return;
    }
    let text = minecolonies_file(game).and_then(|path| std::fs::read_to_string(path).ok());
    m::load_saved(text.as_deref());
}

fn minecolonies_store(game: &mut crate::game::Game<'_>) {
    use crate::game::minecolonies as m;
    if !m::world_ready() {
        return;
    }
    let Some(path) = minecolonies_file(game) else {
        return;
    };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let text = m::with_world(|world| world.to_json());
    if std::fs::write(path, text).is_ok() {
        m::with_world(|world| world.mark_saved());
    }
}

fn minecolonies_store_and_drop(game: &mut crate::game::Game<'_>) {
    use crate::game::minecolonies as m;
    minecolonies_store(game);
    m::reset_world();
}

fn minecolonies_actor(
    player: &horizon_client::mod_api::NativeServerPlayer,
) -> crate::game::minecolonies::Actor {
    crate::game::minecolonies::Actor {
        uuid: player.uuid,
        name: player.name.clone(),
        dimension: player.dimension.clone(),
        x: player.position[0].floor() as i32,
        y: player.position[1].floor() as i32,
        z: player.position[2].floor() as i32,
        yaw: player.yaw,
    }
}

fn minecolonies_sync_lang(game: &mut crate::game::Game<'_>) {
    use crate::game::minecolonies as m;
    let lang = m::Lang::parse(game.native_setting(m::ID, "lang").as_deref());
    m::with_world(|world| world.set_lang(lang));
}
