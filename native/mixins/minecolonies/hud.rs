// Three lines in the top left: the colony, what it is building, the warehouse.
fn minecolonies_hud(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    use crate::game::minecolonies as m;
    use horizon_client::extras::Corner;
    if !game.native_on(m::ID) || game.native_player().is_none() {
        return;
    }
    if game.native_setting(m::ID, "hud").as_deref() == Some("false") {
        return;
    }
    if !game.native_corner_shown(Corner::TopRight) {
        return;
    }
    let stored = m::with_world(|world| world.client_hud());
    let text = if stored.is_empty() {
        "MineColonies\n/colony found".to_owned()
    } else {
        stored
    };
    let lines: Vec<&str> = text.lines().take(3).collect();
    if lines.is_empty() {
        return;
    }
    let width = lines
        .iter()
        .map(|line| game.native_text_width(line))
        .fold(0.0, f32::max)
        + 6.0;
    let height = lines.len() as f32 * 10.0 + 4.0;
    let (x, y) = corners.place(Corner::TopRight, width, height);
    game.native_fill(batch, [x, y, width, height], 0x7800_0000);
    for (index, line) in lines.iter().enumerate() {
        let color = if index == 0 { 0xFFE6_8A } else { 0xF4E8_D0 };
        game.native_text(
            batch,
            line,
            [x + 3.0, y + 3.0 + index as f32 * 10.0],
            color,
            true,
        );
    }
}
