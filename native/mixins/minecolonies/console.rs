// The dedicated server console has no citizen to command.
fn minecolonies_console(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
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
    *ran = true;
    game.native_feedback(horizon_client::text::Text::literal(
        "Colony commands belong to a player. In game: /colony help",
    ));
}
