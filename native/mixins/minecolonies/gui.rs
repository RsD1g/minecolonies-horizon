// Layout of the colony ledger. The nine container slots are hidden.
fn minecolonies_gui(block: &str, screen: &mut Option<horizon_client::mod_api::NativeScreen>) {
    let id = block.split('[').next().unwrap_or(block);
    if id != "minecolonies:ledger" {
        return;
    }
    *screen = Some(horizon_client::mod_api::NativeScreen {
        width: 176.0,
        height: 166.0,
        slots: vec![None; 9],
        inventory_y: 84.0,
        inventory_x: 8.0,
        own_labels: true,
        own_background: false,
    });
}
