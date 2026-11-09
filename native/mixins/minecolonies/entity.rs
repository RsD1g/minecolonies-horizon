// The colony ledger keeps a block entity so its screen can open.
fn minecolonies_entity(
    block: &str,
    entity: &mut Option<(String, horizon_world::nbt::Compound)>,
) {
    let id = block.split('[').next().unwrap_or(block);
    if id != "minecolonies:ledger" || entity.is_some() {
        return;
    }
    *entity = Some((
        "minecolonies:ledger".into(),
        horizon_world::nbt::Compound::new(),
    ));
}
