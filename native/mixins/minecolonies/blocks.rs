// Hut blocks, so they exist even when the content pack is not installed.
fn minecolonies_blocks(blocks: &mut Vec<horizon_mods::content::Block>) {
    for path in [
        "blockhuttownhall",
        "blockhutbuilder",
        "blockhutwarehouse",
        "blockhutcitizen",
        "blockhutfarmer",
        "blockhutlumberjack",
        "blockhutminer",
        "blockhutguardtower",
        "blockhuttavern",
        "blockhutdeliveryman",
        "blockhutfisherman",
        "blockhutsawmill",
        "blockhutbaker",
        "blockhutrestaurant",
        "blockhutuniversity",
        "blockhutlibrary",
        "blockhuthospital",
        "blockhutbarracks",
    ] {
        blocks.push(horizon_mods::content::Block {
            id: format!("minecolonies:{path}"),
            template: "minecraft:oak_planks".into(),
            item: true,
            ..Default::default()
        });
    }
}
