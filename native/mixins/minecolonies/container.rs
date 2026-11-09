// The ledger opens a one-row menu. Its slots stay hidden; the buttons are drawn on top.
fn minecolonies_container(
    block: &str,
    container: &mut Option<horizon_client::containers::ModContainer>,
) {
    let id = block.split('[').next().unwrap_or(block);
    if id != "minecolonies:ledger" {
        return;
    }
    *container = Some(horizon_client::containers::ModContainer {
        slots: 9,
        menu: Some("minecraft:generic_9x1".into()),
        ..Default::default()
    });
}
