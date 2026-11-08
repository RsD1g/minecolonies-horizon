// The world is about to close. Write the colony beside level.dat.
fn minecolonies_stopping(game: &mut crate::game::Game<'_>, dedicated: bool) {
    use crate::game::minecolonies as m;
    let _ = dedicated;
    if !game.native_on(m::ID) {
        return;
    }
    minecolonies_store(game);
}
