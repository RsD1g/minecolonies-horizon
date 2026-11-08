// The supply camp founds a colony on the face you click. The build tool explains itself.
fn minecolonies_use_on(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    pos: [i32; 3],
    face: [i32; 3],
    claimed: &mut bool,
) {
    let door = [pos[0] + face[0], pos[1] + face[1], pos[2] + face[2]];
    minecolonies_hold(game, stack, claimed, Some((door, face)));
}
