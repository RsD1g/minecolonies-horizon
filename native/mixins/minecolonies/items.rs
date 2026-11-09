// The supply camp, the build tool and the rally banner.
fn minecolonies_items(items: &mut Vec<horizon_mods::content::Item>) {
    for (path, template) in [
        ("supplycamp", "minecraft:paper"),
        ("supplyship", "minecraft:paper"),
        ("buildtool", "minecraft:stick"),
        ("rallybanner", "minecraft:paper"),
    ] {
        items.push(horizon_mods::content::Item {
            id: format!("minecolonies:{path}"),
            template: template.into(),
            max_stack_size: Some(1),
            rarity: Some(horizon_mods::content::Rarity::Uncommon),
            ..Default::default()
        });
    }
}
