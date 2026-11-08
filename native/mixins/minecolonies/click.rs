// A click on the settings page. The file is what the server reads too.
fn minecolonies_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::minecolonies as m;
    if mod_id != m::ID || !game.native_on(m::ID) {
        return;
    }
    let lang = m::Lang::parse(game.native_setting(m::ID, "lang").as_deref());
    let hud = game.native_setting(m::ID, "hud").as_deref() != Some("false");
    let (lang, hud) = m::toggle_setting(button, lang, hud);
    game.native_set_setting(m::ID, "lang", lang.code());
    game.native_set_setting(m::ID, "hud", if hud { "true" } else { "false" });
}
