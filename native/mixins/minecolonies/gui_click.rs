// A press on a ledger button becomes a menu button id.
fn minecolonies_gui_click(
    block: &str,
    at: [f32; 2],
    _right: bool,
    button: &mut Option<i32>,
) {
    let id = block.split('[').next().unwrap_or(block);
    if id != "minecolonies:ledger" {
        return;
    }
    if !(8.0..120.0).contains(&at[0]) {
        return;
    }
    let row = (at[1] - 20.0) / 12.0;
    if (0.0..5.0).contains(&row) {
        *button = Some(row as i32);
    }
}
