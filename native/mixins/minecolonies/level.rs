// One dimension's tick: raise blueprints, walk citizens, resolve raids.
fn minecolonies_level(game: &mut crate::game::Game<'_>, dimension: &str) {
    use crate::game::minecolonies as m;

    fn apply(
        game: &mut crate::game::Game<'_>,
        dimension: &str,
        intent: crate::game::minecolonies::Intent,
    ) {
        use crate::game::minecolonies as m;
        match intent {
            m::Intent::Purge => {
                let _ = game.native_run_command("kill @e[type=minecraft:villager,tag=mc_citizen]");
                let _ = game.native_run_command("kill @e[type=minecraft:pillager,tag=mc_raider]");
            }
            m::Intent::Place {
                colony,
                building,
                blocks,
            } => {
                if blocks.is_empty() {
                    m::with_world(|world| world.place_done(colony, building, 0));
                    return;
                }
                match place(game, dimension, &blocks) {
                    Some(0) | None => m::with_world(|world| world.place_later(colony, building)),
                    Some(_) => m::with_world(|world| {
                        world.place_done(colony, building, blocks.len() as u32)
                    }),
                }
            }
            m::Intent::Spawn {
                who,
                dimension: dim,
                pos,
                name,
                kind,
                profession,
                tag,
            } => {
                if dim != dimension {
                    return;
                }
                if let Some(id) = summon(game, &dim, &kind, pos, &name, &profession, &tag) {
                    m::with_world(|world| world.note_mob(who, id));
                }
            }
            m::Intent::Rename {
                dimension: dim,
                entity,
                name,
            } => {
                if dim == dimension {
                    rename(game, &dim, entity, &name);
                }
            }
            m::Intent::Walk {
                dimension: dim,
                entity,
                to,
            } => {
                if dim != dimension {
                    return;
                }
                if game.native_mob(&dim, entity).is_none() {
                    m::with_world(|world| world.note_missing(entity));
                } else if !game.native_mob_navigate(&dim, entity, to, 1, 1.0) {
                    let _ = game.native_teleport_mob(&dim, entity, to);
                }
            }
            m::Intent::Attack {
                dimension: dim,
                entity,
                target,
            } => {
                if dim == dimension {
                    let _ = game.native_mob_attack(&dim, entity, target);
                }
            }
            m::Intent::Remove {
                dimension: dim,
                entity,
            } => {
                if dim == dimension {
                    let _ = game.native_remove_mob(&dim, entity);
                }
            }
            m::Intent::Stamp { dimension: dim, blocks } => {
                if dim == dimension {
                    place(game, &dim, &blocks);
                }
            }
            m::Intent::Talk { player, text } => {
                let _ = game.native_tell(player, horizon_client::text::Text::literal(text));
            }
            m::Intent::Sync { player, text } => {
                let _ =
                    game.native_net_send_to_player(player, "minecolonies:status", text.as_bytes());
            }
        }
    }

    fn place(
        game: &mut crate::game::Game<'_>,
        dimension: &str,
        blocks: &[crate::game::minecolonies::BlockAt],
    ) -> Option<u64> {
        if let Some(changed) = edit(game, dimension, blocks, |state| state.to_owned()) {
            return Some(changed);
        }
        if let Some(changed) = edit(game, dimension, blocks, soften) {
            return Some(changed);
        }
        edit(game, dimension, blocks, |_| {
            "minecraft:oak_planks".to_owned()
        })
    }

    fn soften(state: &str) -> String {
        if state.starts_with("minecolonies:") {
            "minecraft:lodestone".to_owned()
        } else if state.contains("door") {
            "minecraft:oak_planks".to_owned()
        } else {
            state.split('[').next().unwrap_or(state).to_owned()
        }
    }

    fn edit(
        game: &mut crate::game::Game<'_>,
        dimension: &str,
        blocks: &[crate::game::minecolonies::BlockAt],
        states: impl Fn(&str) -> String,
    ) -> Option<u64> {
        use crate::game::native_world_edit::{NativeBlockEdit, NativeBlockRun};
        let mut palette = Vec::new();
        let mut runs = Vec::with_capacity(blocks.len());
        for block in blocks {
            let state = states(&block.state);
            let index = match palette.iter().position(|have: &String| have == &state) {
                Some(index) => index,
                None => {
                    palette.push(state);
                    palette.len() - 1
                }
            };
            runs.push(NativeBlockRun {
                x: block.x,
                z: block.z,
                y: block.y,
                height: 1,
                surface: None,
                block: index as u32,
                only: None,
            });
        }
        let edit = NativeBlockEdit {
            palette,
            filters: Vec::new(),
            runs,
        };
        match game.native_edit_blocks(dimension, &edit) {
            Ok(changed) => Some(changed),
            Err(_) => None,
        }
    }

    fn summon(
        game: &mut crate::game::Game<'_>,
        dimension: &str,
        kind: &str,
        pos: [f64; 3],
        name: &str,
        profession: &str,
        tag: &str,
    ) -> Option<u32> {
        use horizon_world::nbt::{Compound, Tag};
        let mut nbt = Compound::new();
        // Horizon draws CustomName as plain text. A JSON text component shows as the raw string.
        nbt.insert("CustomName".into(), Tag::String(name.into()));
        nbt.insert("CustomNameVisible".into(), Tag::Byte(1));
        nbt.insert("PersistenceRequired".into(), Tag::Byte(1));
        nbt.insert("Tags".into(), Tag::List(vec![Tag::String(tag.into())]));
        if kind == "minecraft:villager" && !profession.is_empty() {
            let mut data = Compound::new();
            data.insert("profession".into(), Tag::String(profession.into()));
            data.insert("type".into(), Tag::String("minecraft:plains".into()));
            data.insert("level".into(), Tag::Int(1));
            nbt.insert("VillagerData".into(), Tag::Compound(data));
        }
        game.native_summon(dimension, kind, pos, nbt)
    }

    fn rename(game: &mut crate::game::Game<'_>, dimension: &str, entity: u32, name: &str) {
        use horizon_world::nbt::{Compound, Tag};
        let mut nbt = Compound::new();
        nbt.insert("CustomName".into(), Tag::String(name.into()));
        nbt.insert("CustomNameVisible".into(), Tag::Byte(1));
        let _ = game.native_merge_entity_nbt(dimension, entity, nbt);
    }

    if !game.native_on(m::ID) {
        return;
    }
    let dir = game.native_world_dir();
    m::ensure_loaded(dir.as_deref());
    let lang = m::Lang::parse(game.native_setting(m::ID, "lang").as_deref());
    m::with_world(|world| world.set_lang(lang));
    let intents = m::with_world(|world| world.tick(dimension));
    for intent in intents {
        apply(game, dimension, intent);
    }
    if m::with_world(|world| world.is_dirty()) {
        m::save_now(dir.as_deref());
    }
}
