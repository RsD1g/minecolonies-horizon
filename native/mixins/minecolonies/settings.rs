// Language and the colony HUD, on this mod's settings page.
fn minecolonies_settings(
    game: &mut crate::game::Game<'_>,
    mod_id: &str,
    buttons: &mut Vec<String>,
) {
    use crate::game::minecolonies as m;
    if mod_id != m::ID || !game.native_on(m::ID) {
        return;
    }
    let lang = m::Lang::parse(game.native_setting(m::ID, "lang").as_deref());
    let hud = game.native_setting(m::ID, "hud").as_deref() != Some("false");
    buttons.extend(m::settings_labels(lang, hud));
}
