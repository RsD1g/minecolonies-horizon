// /colony and its usages, completed in chat like the game's own commands.
fn minecolonies_register(
    game: &mut crate::game::Game<'_>,
    commands: &mut Vec<horizon_client::mod_api::NativeCommand>,
) {
    use crate::game::minecolonies as m;
    if !game.native_on(m::ID) {
        return;
    }
    for usage in m::command_usages() {
        commands.push(horizon_client::mod_api::NativeCommand::new(usage));
    }
}
