// The server's colony lines, drawn by the HUD on this client.
fn minecolonies_net(game: &mut crate::game::Game<'_>, channel: &str, data: &[u8]) {
    use crate::game::minecolonies as m;
    if !game.native_on(m::ID) || channel != "minecolonies:status" {
        return;
    }
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    m::with_world(|world| world.set_client_hud(text));
}
