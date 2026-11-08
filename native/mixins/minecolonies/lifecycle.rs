// Load the colony when the world opens, and write it back when the world closes.
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
    let dir = game.native_world_dir();
    match phase {
        WorldPhase::WorldOpenReady => m::ensure_loaded(dir.as_deref()),
        WorldPhase::WorldCloseFinal | WorldPhase::WorldCloseFailed => {
            m::save_and_drop(dir.as_deref())
        }
        _ => {}
    }
}
