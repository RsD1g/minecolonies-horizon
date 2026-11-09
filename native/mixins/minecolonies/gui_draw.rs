// Labels on the colony ledger.
fn minecolonies_gui_draw(
    game: &crate::game::Game<'_>,
    block: &str,
    batch: &mut horizon_client::hud::HudBatch,
    origin: [f32; 2],
) {
    use crate::game::minecolonies as m;
    let id = block.split('[').next().unwrap_or(block);
    if id != "minecolonies:ledger" {
        return;
    }
    let lang = m::Lang::parse(game.native_setting(m::ID, "lang").as_deref());
    let title = m::tr(lang, "Colony", "Колония");
    game.native_text(batch, &title, [origin[0] + 8.0, origin[1] + 6.0], 0x3F_2A_14, false);
    let labels = [
        m::tr(lang, "Info", "Сводка"),
        m::tr(lang, "Warehouse", "Склад"),
        m::tr(lang, "Citizens", "Жители"),
        m::tr(lang, "Research", "Исследование"),
        m::tr(lang, "Visitors", "Гости"),
    ];
    for (index, label) in labels.iter().enumerate() {
        let y = origin[1] + 20.0 + index as f32 * 12.0;
        game.native_fill(
            batch,
            [origin[0] + 8.0, y, origin[0] + 120.0, y + 11.0],
            0xFF_C4_A0_5A,
        );
        game.native_text(batch, label, [origin[0] + 12.0, y + 2.0], 0x3F_2A_14, false);
    }
}
