// Crafting recipes, joined to the world when it loads.
fn minecolonies_recipes(files: &mut Vec<(String, Vec<u8>)>) {
    use crate::game::minecolonies as m;
    for (id, json) in m::recipe_files() {
        files.push((format!("recipe/minecolonies:{id}"), json.into_bytes()));
    }
}
