//! MineColonies for Horizon. This file is the mod's library (`cargo test`)
//! and the module the bundle adds to the game as `crate::game::minecolonies`.
//!
//! The Java mod does not run in Horizon. What lives here is the colony itself:
//! claims, blueprints, the warehouse, jobs, research and raids. The hook
//! files around it only apply that to the world.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

pub const ID: &str = "minecolonies";
pub const VERSION: &str = "0.1.0";
pub const NAME: &str = "MineColonies";
pub const DESCRIPTION: &str =
    "Found a colony, raise its huts, and put citizens to work. A Horizon port of the MineColonies founding loop.";

pub const OVERWORLD: &str = "minecraft:overworld";
const SAVE_VERSION: u32 = 1;
const CLAIM: i64 = 64;
const RAID_EVERY: u64 = 24_000;
const FOOD_EVERY: u64 = 600;
const WORK_EVERY: u64 = 100;
const BUILD_NAG: u64 = 400;
const MAX_CITIZENS: usize = 16;
const RESIDENCE_CAP: usize = 4;

pub const GOODS: &[&str] = &[
    "minecraft:oak_planks",
    "minecraft:oak_log",
    "minecraft:cobblestone",
    "minecraft:oak_slab",
    "minecraft:glass",
    "minecraft:oak_door",
    "minecraft:bread",
    "minecraft:wheat",
    "minecraft:coal",
    "minecraft:iron_ingot",
];

const SUPPLY: &[(&str, u32)] = &[
    ("minecraft:oak_planks", 256),
    ("minecraft:oak_log", 128),
    ("minecraft:cobblestone", 256),
    ("minecraft:oak_slab", 128),
    ("minecraft:glass", 64),
    ("minecraft:oak_door", 16),
    ("minecraft:bread", 32),
];

const NAMES: &[&str] = &[
    "Ada", "Bram", "Cora", "Dain", "Elsa", "Finn", "Gita", "Hugo", "Ivo", "Juno", "Kael", "Lina",
    "Moss", "Nia", "Otto", "Pia", "Quinn", "Rosa", "Sol", "Tess",
];

thread_local! {
    static LIVE: RefCell<World> = RefCell::new(World::new());
    static READY: Cell<bool> = Cell::new(false);
}

pub fn world_ready() -> bool {
    READY.with(Cell::get)
}

pub fn load_saved(text: Option<&str>) {
    LIVE.with(|live| *live.borrow_mut() = World::open_file(text));
    READY.with(|ready| ready.set(true));
}

pub fn reset_world() {
    LIVE.with(|live| *live.borrow_mut() = World::new());
    READY.with(|ready| ready.set(false));
}

pub fn with_world<R>(f: impl FnOnce(&mut World) -> R) -> R {
    LIVE.with(|live| f(&mut live.borrow_mut()))
}

pub fn command_usages() -> &'static [&'static str] {
    &[
        "colony",
        "colony help",
        "colony found",
        "colony found <name:text>",
        "colony info",
        "colony citizens",
        "colony warehouse",
        "colony build <building:word>",
        "colony hire <id:integer> <job:word>",
        "colony deposit",
        "colony withdraw <item:word> <count:integer>",
        "colony research",
        "colony research <node:word>",
        "colony raid",
        "colony recall",
        "colony abandon",
        "colony abandon confirm",
    ]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Lang {
    En,
    Ru,
}

impl Default for Lang {
    fn default() -> Self {
        Lang::En
    }
}

impl Lang {
    pub fn parse(value: Option<&str>) -> Self {
        match value.map(str::trim) {
            Some("ru") => Lang::Ru,
            _ => Lang::En,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Ru => "ru",
        }
    }
}

fn tr(lang: Lang, en: &str, ru: &str) -> String {
    match lang {
        Lang::En => en.to_owned(),
        Lang::Ru => ru.to_owned(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    TownHall,
    Builder,
    Warehouse,
    Residence,
    Farm,
    Lumberjack,
    Miner,
    GuardTower,
    Tavern,
    Courier,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Job {
    Unemployed,
    Builder,
    Farmer,
    Lumberjack,
    Miner,
    Courier,
    Guard,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PlanBlock {
    x: i32,
    y: i32,
    z: i32,
    state: String,
    #[serde(default)]
    item: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Building {
    id: u32,
    kind: Kind,
    plan: Vec<PlanBlock>,
    placed: u32,
    post: [i32; 3],
    #[serde(skip)]
    inflight: bool,
    #[serde(skip)]
    tries: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Citizen {
    id: u32,
    name: String,
    job: Job,
    #[serde(skip)]
    entity: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Raid {
    hp: i32,
    until: u64,
    wanted: u8,
    #[serde(skip)]
    mobs: Vec<Option<u32>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Colony {
    id: u32,
    name: String,
    #[serde(serialize_with = "ser_u128", deserialize_with = "de_u128")]
    owner: u128,
    owner_name: String,
    dimension: String,
    center: [i32; 3],
    buildings: Vec<Building>,
    citizens: Vec<Citizen>,
    stock: BTreeMap<String, u32>,
    research: BTreeSet<String>,
    happiness: i32,
    next_citizen: u32,
    next_building: u32,
    next_build: u64,
    next_food: u64,
    next_work: u64,
    next_raid: u64,
    next_walk: u64,
    next_nag: u64,
    ore_tick: u32,
    #[serde(skip)]
    recall: bool,
    #[serde(skip)]
    last_snap: String,
    raid: Option<Raid>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct World {
    version: u32,
    clock: u64,
    next_id: u32,
    #[serde(default)]
    lang: Lang,
    colonies: Vec<Colony>,
    #[serde(skip)]
    warning: Option<String>,
    #[serde(skip)]
    client_hud: String,
    #[serde(skip)]
    pending: Vec<Intent>,
    #[serde(skip)]
    dirty: bool,
}

#[derive(Clone, Debug)]
pub struct Actor {
    pub uuid: u128,
    pub name: String,
    pub dimension: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub yaw: f32,
}

#[derive(Clone, Debug)]
pub struct Reply {
    pub handled: bool,
    pub lines: Vec<String>,
    pub order: Order,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Order {
    None,
    Deposit,
    Withdraw { item: String, count: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpawnWho {
    Citizen { colony: u32, id: u32 },
    Raider { colony: u32, index: u8 },
}

#[derive(Clone, Debug)]
pub struct BlockAt {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub state: String,
}

#[derive(Clone, Debug)]
pub enum Intent {
    Purge,
    Place {
        colony: u32,
        building: u32,
        blocks: Vec<BlockAt>,
    },
    Spawn {
        who: SpawnWho,
        dimension: String,
        pos: [f64; 3],
        name: String,
        kind: String,
        profession: String,
        tag: String,
    },
    Rename {
        dimension: String,
        entity: u32,
        name: String,
    },
    Walk {
        dimension: String,
        entity: u32,
        to: [f64; 3],
    },
    Attack {
        dimension: String,
        entity: u32,
        target: u32,
    },
    Remove {
        dimension: String,
        entity: u32,
    },
    Talk {
        player: u128,
        text: String,
    },
    Sync {
        player: u128,
        text: String,
    },
}

fn ser_u128<S: serde::Serializer>(n: &u128, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&n.to_string())
}

fn de_u128<'de, D: serde::Deserializer<'de>>(d: D) -> Result<u128, D::Error> {
    let text = String::deserialize(d)?;
    text.parse().map_err(serde::de::Error::custom)
}

impl Reply {
    fn ignore() -> Self {
        Self {
            handled: false,
            lines: Vec::new(),
            order: Order::None,
        }
    }

    fn say(lines: Vec<String>) -> Self {
        Self {
            handled: true,
            lines,
            order: Order::None,
        }
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            version: SAVE_VERSION,
            clock: 0,
            next_id: 1,
            lang: Lang::En,
            colonies: Vec::new(),
            warning: None,
            client_hud: String::new(),
            pending: Vec::new(),
            dirty: false,
        }
    }

    pub fn from_json(text: &str) -> Result<Self, String> {
        let mut world: Self = serde_json::from_str(text).map_err(|e| e.to_string())?;
        if world.version != SAVE_VERSION {
            return Err(format!(
                "colony file version {} is not {SAVE_VERSION}",
                world.version
            ));
        }
        world.pending.clear();
        world.dirty = false;
        world.warning = None;
        for colony in &mut world.colonies {
            colony.recall = true;
            colony.last_snap.clear();
            for citizen in &mut colony.citizens {
                citizen.entity = None;
            }
            for building in &mut colony.buildings {
                building.inflight = false;
                building.tries = 0;
            }
            if let Some(raid) = &mut colony.raid {
                raid.mobs.clear();
            }
        }
        Ok(world)
    }

    pub fn open_file(text: Option<&str>) -> Self {
        match text {
            None => Self::new(),
            Some(text) => match Self::from_json(text) {
                Ok(world) => world,
                Err(err) => {
                    let mut world = Self::new();
                    world.warning = Some(err);
                    world
                }
            },
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_owned())
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_saved(&mut self) {
        self.dirty = false;
    }

    pub fn set_lang(&mut self, lang: Lang) {
        if self.lang != lang {
            self.lang = lang;
            self.dirty = true;
            for colony in &mut self.colonies {
                colony.last_snap.clear();
            }
        }
    }

    pub fn lang(&self) -> Lang {
        self.lang
    }

    pub fn set_client_hud(&mut self, text: &str) {
        self.client_hud = text.to_owned();
    }

    pub fn client_hud(&self) -> String {
        self.client_hud.clone()
    }

    pub fn colonies_owned(&self, uuid: u128) -> bool {
        self.colonies.iter().any(|colony| colony.owner == uuid)
    }

    pub fn command(&mut self, actor: &Actor, line: &str) -> Reply {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("colony") else {
            return Reply::ignore();
        };
        if !rest.is_empty() && !rest.starts_with([' ', '\t']) {
            return Reply::ignore();
        }
        let rest = rest.trim();
        let mut reply = self.dispatch(actor, rest);
        if let Some(warning) = self.warning.take() {
            reply.lines.insert(0, warning);
        }
        reply
    }

    pub fn tick(&mut self, dimension: &str) -> Vec<Intent> {
        if dimension == OVERWORLD {
            self.clock = self.clock.saturating_add(1);
        }
        let clock = self.clock;
        let lang = self.lang;
        let mut out = Vec::new();
        if self
            .colonies
            .iter()
            .any(|c| c.dimension == dimension && c.recall)
        {
            out.push(Intent::Purge);
            for colony in &mut self.colonies {
                if colony.dimension == dimension {
                    colony.recall = false;
                }
            }
        }
        for colony in &mut self.colonies {
            if colony.dimension != dimension {
                continue;
            }
            if step(colony, clock, lang, &mut out) {
                self.dirty = true;
            }
        }
        if self.clock % 20 == 0 {
            for colony in &mut self.colonies {
                if colony.dimension != dimension {
                    continue;
                }
                let text = snapshot(colony, lang);
                if text != colony.last_snap {
                    colony.last_snap = text.clone();
                    out.push(Intent::Sync {
                        player: colony.owner,
                        text,
                    });
                }
            }
        }
        let mut pending = std::mem::take(&mut self.pending);
        pending.append(&mut out);
        pending
    }

    pub fn place_done(&mut self, colony: u32, building: u32, n: u32) {
        let lang = self.lang;
        let mut talk = None;
        let mut owner = None;
        if let Some(colony) = self.colonies.iter_mut().find(|c| c.id == colony) {
            if let Some(built) = colony.buildings.iter_mut().find(|b| b.id == building) {
                let start = built.placed as usize;
                if start < built.plan.len() {
                    let end = (start + n as usize).min(built.plan.len());
                    for block in &built.plan[start..end] {
                        if let Some(item) = &block.item {
                            pay(&mut colony.stock, item, 1);
                        }
                    }
                    built.placed = end as u32;
                    let finished = built.placed as usize >= built.plan.len();
                    let kind = built.kind;
                    built.inflight = false;
                    built.tries = 0;
                    if finished {
                        let label = kind_label(lang, kind).to_owned();
                        let name = colony.name.clone();
                        fill_beds(colony);
                        talk = Some(tr(
                            lang,
                            &format!("{label} in {name} is finished."),
                            &format!("«{label}» в колонии {name} достроена."),
                        ));
                        owner = Some(colony.owner);
                    }
                    self.dirty = true;
                } else {
                    built.inflight = false;
                }
            }
        }
        if let (Some(text), Some(player)) = (talk, owner) {
            self.pending.push(Intent::Talk { player, text });
        }
    }

    pub fn place_later(&mut self, colony: u32, building: u32) {
        let Some(colony) = self.colonies.iter_mut().find(|c| c.id == colony) else {
            return;
        };
        let Some(built) = colony.buildings.iter_mut().find(|b| b.id == building) else {
            return;
        };
        built.inflight = false;
        built.tries = built.tries.saturating_add(1);
        if built.tries >= 20 && (built.placed as usize) < built.plan.len() {
            built.placed += 1;
            built.tries = 0;
            self.dirty = true;
        }
    }

    pub fn note_mob(&mut self, who: SpawnWho, entity: u32) {
        match who {
            SpawnWho::Citizen { colony, id } => {
                if let Some(colony) = self.colonies.iter_mut().find(|c| c.id == colony) {
                    if let Some(citizen) = colony.citizens.iter_mut().find(|c| c.id == id) {
                        citizen.entity = Some(entity);
                    }
                }
            }
            SpawnWho::Raider { colony, index } => {
                if let Some(colony) = self.colonies.iter_mut().find(|c| c.id == colony) {
                    if let Some(raid) = &mut colony.raid {
                        if let Some(slot) = raid.mobs.get_mut(index as usize) {
                            *slot = Some(entity);
                        }
                    }
                }
            }
        }
    }

    pub fn note_missing(&mut self, entity: u32) {
        for colony in &mut self.colonies {
            for citizen in &mut colony.citizens {
                if citizen.entity == Some(entity) {
                    citizen.entity = None;
                }
            }
            if let Some(raid) = &mut colony.raid {
                for slot in &mut raid.mobs {
                    if *slot == Some(entity) {
                        *slot = None;
                    }
                }
            }
        }
    }

    pub fn protect(&self, dimension: &str, pos: [i32; 3], actor: u128) -> Option<String> {
        for colony in &self.colonies {
            if colony.dimension != dimension || !inside(colony.center, pos) {
                continue;
            }
            if colony.owner == actor {
                return None;
            }
            return Some(tr(
                self.lang,
                &format!("This land belongs to {}.", colony.name),
                &format!("Эта земля принадлежит колонии {}.", colony.name),
            ));
        }
        None
    }

    pub fn give_stock(&mut self, owner: u128, item: &str, count: u32) -> String {
        let lang = self.lang;
        let Some(colony) = self.colonies.iter_mut().find(|c| c.owner == owner) else {
            return tr(lang, "You have no colony.", "У вас нет колонии.");
        };
        if count > 0 && GOODS.contains(&item) {
            *colony.stock.entry(item.to_owned()).or_insert(0) += count;
            self.dirty = true;
        }
        tr(
            lang,
            &format!("The warehouse takes {count} {}.", short_item(item)),
            &format!("Склад принимает {count} {}.", short_item(item)),
        )
    }

    pub fn take_stock(&mut self, owner: u128, item: &str, count: u32) -> u32 {
        let Some(colony) = self.colonies.iter_mut().find(|c| c.owner == owner) else {
            return 0;
        };
        let have = colony.stock.get(item).copied().unwrap_or(0);
        let n = have.min(count);
        if n > 0 {
            pay(&mut colony.stock, item, n);
            self.dirty = true;
        }
        n
    }

    fn dispatch(&mut self, actor: &Actor, rest: &str) -> Reply {
        if rest.is_empty() || rest == "help" {
            return Reply::say(help_lines(self.lang));
        }
        if let Some(args) = tail(rest, "found") {
            return self.found(actor, args);
        }
        if tail(rest, "info").is_some() {
            return self.with_colony(actor, |c, lang| Reply::say(info_lines(c, lang)));
        }
        if tail(rest, "citizens").is_some() {
            return self.with_colony(actor, |c, lang| Reply::say(citizen_lines(c, lang)));
        }
        if tail(rest, "warehouse").is_some() {
            return self.with_colony(actor, |c, lang| Reply::say(stock_lines(c, lang)));
        }
        if let Some(args) = tail(rest, "build") {
            return self.build(actor, args);
        }
        if let Some(args) = tail(rest, "hire") {
            return self.hire(actor, args);
        }
        if tail(rest, "deposit").is_some() {
            return self.deposit_order(actor);
        }
        if let Some(args) = tail(rest, "withdraw") {
            return self.withdraw_order(actor, args);
        }
        if let Some(args) = tail(rest, "research") {
            return self.research(actor, args);
        }
        if tail(rest, "raid").is_some() {
            return self.call_raid(actor);
        }
        if tail(rest, "recall").is_some() {
            return self.recall(actor);
        }
        if let Some(args) = tail(rest, "abandon") {
            return self.abandon(actor, args);
        }
        Reply::say(vec![tr(
            self.lang,
            "Unknown colony command. /colony help",
            "Неизвестная команда. /colony help",
        )])
    }

    fn with_colony(&self, actor: &Actor, f: impl FnOnce(&Colony, Lang) -> Reply) -> Reply {
        match self.colonies.iter().find(|c| c.owner == actor.uuid) {
            Some(colony) => f(colony, self.lang),
            None => Reply::say(vec![tr(
                self.lang,
                "You have no colony. /colony found",
                "У вас нет колонии. /colony found",
            )]),
        }
    }

    fn found(&mut self, actor: &Actor, name: &str) -> Reply {
        let (door, rot) = door_ahead(actor, 5);
        self.found_at(actor, name, door, rot)
    }

    pub fn found_at(&mut self, actor: &Actor, name: &str, door: [i32; 3], rot: u8) -> Reply {
        let lang = self.lang;
        if self.colonies.iter().any(|c| c.owner == actor.uuid) {
            return Reply::say(vec![tr(
                lang,
                "You already have a colony.",
                "Колония у вас уже есть.",
            )]);
        }
        if actor.dimension != OVERWORLD {
            return Reply::say(vec![tr(
                lang,
                "Colonies are founded in the overworld.",
                "Колонию основывают в обычном мире.",
            )]);
        }
        if self.colonies.iter().any(|c| {
            c.dimension == actor.dimension && dist2(c.center, door) < (CLAIM * 2) * (CLAIM * 2)
        }) {
            return Reply::say(vec![tr(
                lang,
                "Another colony is too close.",
                "Слишком близко другая колония.",
            )]);
        }
        let name = clean_name(name);
        let name = if name.is_empty() {
            tr(
                lang,
                &format!("{}'s colony", actor.name),
                &format!("Колония {}", actor.name),
            )
        } else {
            name
        };
        let draft = layout(Kind::TownHall, door, rot);
        let mut stock = BTreeMap::new();
        for (item, count) in SUPPLY {
            stock.insert((*item).to_owned(), *count);
        }
        let id = self.next_id;
        self.next_id += 1;
        let clock = self.clock;
        let mut colony = Colony {
            id,
            name: name.clone(),
            owner: actor.uuid,
            owner_name: actor.name.clone(),
            dimension: actor.dimension.clone(),
            center: door,
            buildings: vec![Building {
                id: 1,
                kind: Kind::TownHall,
                plan: draft.plan,
                placed: 0,
                post: draft.post,
                inflight: false,
                tries: 0,
            }],
            citizens: Vec::new(),
            stock,
            research: BTreeSet::new(),
            happiness: 60,
            next_citizen: 1,
            next_building: 2,
            next_build: clock,
            next_food: clock + FOOD_EVERY,
            next_work: clock + WORK_EVERY,
            next_raid: clock + RAID_EVERY,
            next_walk: clock + 15,
            next_nag: clock + BUILD_NAG,
            ore_tick: 0,
            recall: false,
            last_snap: String::new(),
            raid: None,
        };
        add_citizen(&mut colony, Job::Unemployed);
        add_citizen(&mut colony, Job::Unemployed);
        self.colonies.push(colony);
        self.dirty = true;
        self.queue_sync(id);
        Reply::say(vec![tr(
            lang,
            &format!("{name} is founded. The supply camp is going up. /colony build builder"),
            &format!("{name} основана. Лагерь снабжения поднимается. /colony build builder"),
        )])
    }

    fn build(&mut self, actor: &Actor, args: &str) -> Reply {
        let lang = self.lang;
        let Some(kind) = parse_kind(args.trim()) else {
            return Reply::say(vec![tr(
                lang,
                "Buildings: builder, warehouse, residence, farm, lumberjack, miner, guardtower, tavern, courier.",
                "Здания: builder, warehouse, residence, farm, lumberjack, miner, guardtower, tavern, courier.",
            )]);
        };
        if kind == Kind::TownHall {
            return Reply::say(vec![tr(
                lang,
                "The town hall is already yours.",
                "Ратуша уже стоит.",
            )]);
        }
        let (door, rot) = door_ahead(actor, 4);
        let owner = actor.uuid;
        let dimension = actor.dimension.clone();
        let Some(index) = self.colonies.iter().position(|c| c.owner == owner) else {
            return Reply::say(vec![tr(
                lang,
                "You have no colony. /colony found",
                "У вас нет колонии. /colony found",
            )]);
        };
        let clock = self.clock;
        let colony = &mut self.colonies[index];
        if colony.dimension != dimension {
            return Reply::say(vec![tr(
                lang,
                "Step into your colony's dimension first.",
                "Сначала перейдите в измерение колонии.",
            )]);
        }
        if let Some(node) = research_for(kind) {
            if !colony.research.contains(node) {
                return Reply::say(vec![tr(
                    lang,
                    &format!("Research {node} first. /colony research"),
                    &format!("Сначала исследуйте {node}. /colony research"),
                )]);
            }
        }
        let same = colony.buildings.iter().filter(|b| b.kind == kind).count();
        let cap = if kind == Kind::Residence {
            RESIDENCE_CAP
        } else {
            1
        };
        if same >= cap {
            return Reply::say(vec![tr(
                lang,
                "That building is already planned.",
                "Такое здание уже запланировано.",
            )]);
        }
        if colony.buildings.iter().any(|b| b.kind == kind && !done(b)) {
            return Reply::say(vec![tr(
                lang,
                "That hut is still going up.",
                "Эта хижина ещё строится.",
            )]);
        }
        let draft = layout(kind, door, rot);
        if draft
            .plan
            .iter()
            .any(|b| !inside(colony.center, [b.x, b.y, b.z]))
        {
            return Reply::say(vec![tr(
                lang,
                "That spot is outside the colony claim. Stand closer to the town hall.",
                "Это место вне границы колонии. Встаньте ближе к ратуше.",
            )]);
        }
        if overlaps(&colony.buildings, &draft.plan) {
            return Reply::say(vec![tr(
                lang,
                "That spot overlaps another hut.",
                "Это место пересекается с другой хижиной.",
            )]);
        }
        let id = colony.next_building;
        colony.next_building += 1;
        colony.next_build = clock;
        colony.buildings.push(Building {
            id,
            kind,
            plan: draft.plan,
            placed: 0,
            post: draft.post,
            inflight: false,
            tries: 0,
        });
        let label = kind_label(lang, kind);
        self.dirty = true;
        let colony_id = colony.id;
        self.queue_sync(colony_id);
        Reply::say(vec![tr(
            lang,
            &format!("Blueprint for the {label} is placed. Hire a builder if you have not. /colony hire <id> builder"),
            &format!("Чертёж «{label}» поставлен. Наймите строителя, если его ещё нет. /colony hire <id> builder"),
        )])
    }

    fn hire(&mut self, actor: &Actor, args: &str) -> Reply {
        let lang = self.lang;
        let mut words = args.split_whitespace();
        let Some(id) = words.next().and_then(|w| w.parse::<u32>().ok()) else {
            return Reply::say(vec![tr(
                lang,
                "Usage: /colony hire <id> <job>",
                "Формат: /colony hire <id> <работа>",
            )]);
        };
        let Some(job) = words.next().and_then(parse_job) else {
            return Reply::say(vec![tr(
                lang,
                "Jobs: builder, farmer, lumberjack, miner, courier, guard, unemployed.",
                "Работы: builder, farmer, lumberjack, miner, courier, guard, unemployed.",
            )]);
        };
        if words.next().is_some() {
            return Reply::say(vec![tr(
                lang,
                "Usage: /colony hire <id> <job>",
                "Формат: /colony hire <id> <работа>",
            )]);
        }
        let owner = actor.uuid;
        let Some(colony) = self.colonies.iter_mut().find(|c| c.owner == owner) else {
            return Reply::say(vec![tr(lang, "You have no colony.", "У вас нет колонии.")]);
        };
        if job != Job::Unemployed {
            let kind = job_kind(job);
            if !colony.buildings.iter().any(|b| b.kind == kind) {
                return Reply::say(vec![tr(
                    lang,
                    "Place that hut's blueprint first. /colony build",
                    "Сначала поставьте чертёж хижины. /colony build",
                )]);
            }
            if colony.citizens.iter().any(|c| c.id != id && c.job == job) {
                return Reply::say(vec![tr(
                    lang,
                    "Someone already has that job.",
                    "Эта работа уже занята.",
                )]);
            }
        }
        let Some(citizen) = colony.citizens.iter_mut().find(|c| c.id == id) else {
            return Reply::say(vec![tr(
                lang,
                "No citizen with that id. /colony citizens",
                "Нет жителя с таким номером. /colony citizens",
            )]);
        };
        citizen.job = job;
        let name = citizen_title(lang, citizen);
        let entity = citizen.entity;
        let dimension = colony.dimension.clone();
        let player = colony.owner;
        self.dirty = true;
        if let Some(entity) = entity {
            self.pending.push(Intent::Rename {
                dimension,
                entity,
                name: name.clone(),
            });
        }
        self.pending.push(Intent::Talk {
            player,
            text: name.clone(),
        });
        Reply::say(vec![name])
    }

    fn deposit_order(&mut self, actor: &Actor) -> Reply {
        let lang = self.lang;
        if !self.colonies.iter().any(|c| c.owner == actor.uuid) {
            return Reply::say(vec![tr(lang, "You have no colony.", "У вас нет колонии.")]);
        }
        Reply {
            handled: true,
            lines: vec![tr(
                lang,
                "You hand goods to the warehouse.",
                "Вы сдаёте товары на склад.",
            )],
            order: Order::Deposit,
        }
    }

    fn withdraw_order(&mut self, actor: &Actor, args: &str) -> Reply {
        let lang = self.lang;
        let mut words = args.split_whitespace();
        let Some(item) = words.next().and_then(stock_alias) else {
            return Reply::say(vec![tr(
                lang,
                "Usage: /colony withdraw <planks|logs|cobble|slabs|glass|doors|bread|wheat|coal|iron> <count>",
                "Формат: /colony withdraw <planks|logs|cobble|slabs|glass|doors|bread|wheat|coal|iron> <число>",
            )]);
        };
        let Some(count) = words
            .next()
            .and_then(|w| w.parse::<u32>().ok())
            .filter(|n| *n > 0)
        else {
            return Reply::say(vec![tr(
                lang,
                "Give a count above zero.",
                "Нужно число больше нуля.",
            )]);
        };
        let Some(colony) = self.colonies.iter().find(|c| c.owner == actor.uuid) else {
            return Reply::say(vec![tr(lang, "You have no colony.", "У вас нет колонии.")]);
        };
        let have = colony.stock.get(item).copied().unwrap_or(0);
        if have == 0 {
            return Reply::say(vec![tr(
                lang,
                &format!("The warehouse has no {}.", short_item(item)),
                &format!("На складе нет {}.", short_item(item)),
            )]);
        }
        Reply {
            handled: true,
            lines: Vec::new(),
            order: Order::Withdraw {
                item: item.to_owned(),
                count: count.min(have),
            },
        }
    }

    fn research(&mut self, actor: &Actor, args: &str) -> Reply {
        let lang = self.lang;
        let args = args.trim();
        if args.is_empty() {
            return self.with_colony(actor, |c, lang| Reply::say(research_lines(c, lang)));
        }
        let Some(node) = research_node(args) else {
            return Reply::say(vec![tr(
                lang,
                "Unknown research. /colony research",
                "Неизвестное исследование. /colony research",
            )]);
        };
        let owner = actor.uuid;
        let Some(colony) = self.colonies.iter_mut().find(|c| c.owner == owner) else {
            return Reply::say(vec![tr(lang, "You have no colony.", "У вас нет колонии.")]);
        };
        if colony.research.contains(node.id) {
            return Reply::say(vec![tr(lang, "Already researched.", "Уже исследовано.")]);
        }
        for (item, cost) in node.cost {
            if colony.stock.get(*item).copied().unwrap_or(0) < *cost {
                return Reply::say(vec![tr(
                    lang,
                    &format!("Research needs {cost} {}.", short_item(item)),
                    &format!("Для исследования нужно {cost} {}.", short_item(item)),
                )]);
            }
        }
        for (item, cost) in node.cost {
            pay(&mut colony.stock, item, *cost);
        }
        colony.research.insert(node.id.to_owned());
        self.dirty = true;
        Reply::say(vec![tr(
            lang,
            &format!(
                "Researched {}. You can build the {}.",
                node.id,
                kind_label(lang, node.unlocks)
            ),
            &format!(
                "Исследовано {}. Можно строить «{}».",
                node.id,
                kind_label(lang, node.unlocks)
            ),
        )])
    }

    fn call_raid(&mut self, actor: &Actor) -> Reply {
        let lang = self.lang;
        let clock = self.clock;
        let owner = actor.uuid;
        let Some(colony) = self.colonies.iter_mut().find(|c| c.owner == owner) else {
            return Reply::say(vec![tr(lang, "You have no colony.", "У вас нет колонии.")]);
        };
        if colony.raid.is_some() {
            return Reply::say(vec![tr(
                lang,
                "A raid is already at the gates.",
                "Набег уже идёт.",
            )]);
        }
        begin_raid(colony, clock, lang, &mut self.pending);
        self.dirty = true;
        Reply::say(vec![tr(
            lang,
            "You sound the raid horn.",
            "Вы трубите в рог набега.",
        )])
    }

    fn recall(&mut self, actor: &Actor) -> Reply {
        let lang = self.lang;
        let owner = actor.uuid;
        let Some(colony) = self.colonies.iter_mut().find(|c| c.owner == owner) else {
            return Reply::say(vec![tr(lang, "You have no colony.", "У вас нет колонии.")]);
        };
        colony.recall = true;
        for citizen in &mut colony.citizens {
            citizen.entity = None;
        }
        if let Some(raid) = &mut colony.raid {
            raid.mobs.clear();
        }
        Reply::say(vec![tr(
            lang,
            "Citizens will gather again at their huts.",
            "Жители снова соберутся у хижин.",
        )])
    }

    fn abandon(&mut self, actor: &Actor, args: &str) -> Reply {
        let lang = self.lang;
        if args.trim() != "confirm" {
            return Reply::say(vec![tr(
                lang,
                "This leaves the buildings standing and dismisses the citizens. /colony abandon confirm",
                "Здания останутся, жители уйдут. /colony abandon confirm",
            )]);
        }
        let owner = actor.uuid;
        let Some(index) = self.colonies.iter().position(|c| c.owner == owner) else {
            return Reply::say(vec![tr(lang, "You have no colony.", "У вас нет колонии.")]);
        };
        let colony = self.colonies.remove(index);
        let dimension = colony.dimension.clone();
        for citizen in &colony.citizens {
            if let Some(entity) = citizen.entity {
                self.pending.push(Intent::Remove {
                    dimension: dimension.clone(),
                    entity,
                });
            }
        }
        if let Some(raid) = &colony.raid {
            for entity in raid.mobs.iter().flatten() {
                self.pending.push(Intent::Remove {
                    dimension: dimension.clone(),
                    entity: *entity,
                });
            }
        }
        self.pending.push(Intent::Purge);
        self.dirty = true;
        Reply::say(vec![tr(
            lang,
            &format!("{} is abandoned. The huts remain.", colony.name),
            &format!("{} распущена. Хижины остаются.", colony.name),
        )])
    }

    fn queue_sync(&mut self, id: u32) {
        let lang = self.lang;
        let synced = self.colonies.iter_mut().find(|c| c.id == id).map(|c| {
            let text = snapshot(c, lang);
            c.last_snap = text.clone();
            (c.owner, text)
        });
        if let Some((player, text)) = synced {
            self.pending.push(Intent::Sync { player, text });
        }
    }
}

fn tail<'a>(rest: &'a str, cmd: &str) -> Option<&'a str> {
    if rest == cmd {
        return Some("");
    }
    rest.strip_prefix(cmd)?.strip_prefix(' ')
}

fn clean_name(name: &str) -> String {
    name.chars()
        .filter(|c| !c.is_control())
        .take(32)
        .collect::<String>()
        .trim()
        .to_owned()
}

fn pay(stock: &mut BTreeMap<String, u32>, item: &str, n: u32) {
    let Some(have) = stock.get_mut(item) else {
        return;
    };
    if *have <= n {
        stock.remove(item);
    } else {
        *have -= n;
    }
}

fn inside(center: [i32; 3], pos: [i32; 3]) -> bool {
    dist2(center, pos) <= CLAIM * CLAIM
}

fn dist2(a: [i32; 3], b: [i32; 3]) -> i64 {
    let dx = a[0] as i64 - b[0] as i64;
    let dz = a[2] as i64 - b[2] as i64;
    dx * dx + dz * dz
}

pub fn rot_from_yaw(yaw: f32) -> u8 {
    let y = yaw.rem_euclid(360.0);
    ((y + 45.0) / 90.0) as u8 % 4
}

pub fn rot_from_face(face: [i32; 3], yaw: f32) -> u8 {
    match face {
        [0, 0, -1] => 0,
        [1, 0, 0] => 1,
        [0, 0, 1] => 2,
        [-1, 0, 0] => 3,
        _ => rot_from_yaw(yaw),
    }
}

pub fn door_ahead(actor: &Actor, dist: i32) -> ([i32; 3], u8) {
    let rot = rot_from_yaw(actor.yaw);
    let r = (actor.yaw as f64).to_radians();
    let x = actor.x + (-r.sin() * f64::from(dist)).round() as i32;
    let z = actor.z + (r.cos() * f64::from(dist)).round() as i32;
    ([x, actor.y, z], rot)
}

fn rotate(x: i32, z: i32, rot: u8) -> (i32, i32) {
    match rot & 3 {
        0 => (x, z),
        1 => (-z, x),
        2 => (-x, -z),
        _ => (z, -x),
    }
}

struct Draft {
    plan: Vec<PlanBlock>,
    post: [i32; 3],
}

fn size_of(kind: Kind) -> (i32, i32, i32) {
    match kind {
        Kind::TownHall => (7, 7, 5),
        Kind::Warehouse => (7, 5, 4),
        Kind::GuardTower => (5, 5, 6),
        _ => (5, 5, 4),
    }
}

fn layout(kind: Kind, door: [i32; 3], rot: u8) -> Draft {
    let (w, d, h) = size_of(kind);
    let (rdx, rdz) = rotate(w / 2, 0, rot);
    let anchor_x = door[0] - rdx;
    let anchor_y = door[1] - 1;
    let anchor_z = door[2] - rdz;
    let mut plan = Vec::new();
    for y in 0..h {
        for x in 0..w {
            for z in 0..d {
                let edge = x == 0 || z == 0 || x == w - 1 || z == d - 1;
                let corner = (x == 0 || x == w - 1) && (z == 0 || z == d - 1);
                let token = if y == 0 {
                    Token::Stone
                } else if y == h - 1 {
                    Token::Slab
                } else if x == w / 2 && z == 0 && (y == 1 || y == 2) {
                    if y == 1 {
                        Token::DoorLower
                    } else {
                        Token::DoorUpper
                    }
                } else if !edge {
                    if y == 1 && x == w / 2 && z == d / 2 {
                        Token::Hut
                    } else {
                        continue;
                    }
                } else if corner {
                    Token::Log
                } else if y == 2 {
                    Token::Glass
                } else {
                    Token::Planks
                };
                let (wx, wz) = rotate(x, z, rot);
                plan.push(PlanBlock {
                    x: anchor_x + wx,
                    y: anchor_y + y,
                    z: anchor_z + wz,
                    state: state_of(token, kind, rot),
                    item: cost_of(token).map(str::to_owned),
                });
            }
        }
    }
    let (sx, sz) = rotate(w / 2, 1, rot);
    Draft {
        plan,
        post: [anchor_x + sx, anchor_y + 1, anchor_z + sz],
    }
}

#[derive(Clone, Copy)]
enum Token {
    Stone,
    Planks,
    Log,
    Glass,
    Slab,
    DoorLower,
    DoorUpper,
    Hut,
}

fn cost_of(token: Token) -> Option<&'static str> {
    Some(match token {
        Token::Stone => "minecraft:cobblestone",
        Token::Planks => "minecraft:oak_planks",
        Token::Log => "minecraft:oak_log",
        Token::Glass => "minecraft:glass",
        Token::Slab => "minecraft:oak_slab",
        Token::DoorLower => "minecraft:oak_door",
        Token::DoorUpper | Token::Hut => return None,
    })
}

fn facing(rot: u8) -> &'static str {
    match rot & 3 {
        0 => "north",
        1 => "east",
        2 => "south",
        _ => "west",
    }
}

fn state_of(token: Token, kind: Kind, rot: u8) -> String {
    match token {
        Token::Stone => "minecraft:cobblestone".to_owned(),
        Token::Planks => "minecraft:oak_planks".to_owned(),
        Token::Log => "minecraft:oak_log[axis=y]".to_owned(),
        Token::Glass => "minecraft:glass".to_owned(),
        Token::Slab => "minecraft:oak_slab[type=bottom]".to_owned(),
        Token::DoorLower => format!(
            "minecraft:oak_door[facing={},half=lower,hinge=left,open=false]",
            facing(rot)
        ),
        Token::DoorUpper => format!(
            "minecraft:oak_door[facing={},half=upper,hinge=left,open=false]",
            facing(rot)
        ),
        Token::Hut => hut_block(kind).to_owned(),
    }
}

fn hut_block(kind: Kind) -> &'static str {
    match kind {
        Kind::TownHall => "minecolonies:blockhuttownhall",
        Kind::Builder => "minecolonies:blockhutbuilder",
        Kind::Warehouse => "minecolonies:blockhutwarehouse",
        Kind::Residence => "minecolonies:blockhutcitizen",
        Kind::Farm => "minecolonies:blockhutfarmer",
        Kind::Lumberjack => "minecolonies:blockhutlumberjack",
        Kind::Miner => "minecolonies:blockhutminer",
        Kind::GuardTower => "minecolonies:blockhutguardtower",
        Kind::Tavern => "minecolonies:blockhuttavern",
        Kind::Courier => "minecolonies:blockhutdeliveryman",
    }
}

fn overlaps(buildings: &[Building], plan: &[PlanBlock]) -> bool {
    plan.iter().any(|block| {
        buildings.iter().any(|built| {
            built
                .plan
                .iter()
                .any(|old| old.x == block.x && old.y == block.y && old.z == block.z)
        })
    })
}

fn done(building: &Building) -> bool {
    !building.plan.is_empty() && building.placed as usize >= building.plan.len()
}

fn add_citizen(colony: &mut Colony, job: Job) {
    let id = colony.next_citizen;
    colony.next_citizen += 1;
    let name = NAMES[(id as usize - 1) % NAMES.len()].to_owned();
    colony.citizens.push(Citizen {
        id,
        name,
        job,
        entity: None,
    });
}

fn fill_beds(colony: &mut Colony) {
    while colony.citizens.len() < beds(colony) && colony.citizens.len() < MAX_CITIZENS {
        add_citizen(colony, Job::Unemployed);
    }
}

fn beds(colony: &Colony) -> usize {
    colony
        .buildings
        .iter()
        .filter(|b| done(b))
        .map(|b| match b.kind {
            Kind::TownHall | Kind::Residence => 2,
            _ => 0,
        })
        .sum()
}

fn mood(colony: &Colony) -> i32 {
    let mut score = colony.happiness;
    if colony
        .buildings
        .iter()
        .any(|b| b.kind == Kind::Tavern && done(b))
    {
        score += 10;
    }
    let homeless = colony.citizens.len().saturating_sub(beds(colony)) as i32;
    score -= homeless * 8;
    score.clamp(10, 100)
}

fn step(colony: &mut Colony, clock: u64, lang: Lang, out: &mut Vec<Intent>) -> bool {
    let mut dirty = false;
    if let Some(raid) = &mut colony.raid {
        if raid.mobs.len() != raid.wanted as usize {
            raid.mobs.resize(raid.wanted as usize, None);
        }
    }
    place_work(colony, clock, lang, out);
    if clock >= colony.next_work {
        colony.next_work = clock + WORK_EVERY;
        if produce(colony) {
            dirty = true;
        }
    }
    if clock >= colony.next_food {
        colony.next_food = clock + FOOD_EVERY;
        if feed(colony, lang, out) {
            dirty = true;
        }
    }
    if colony.raid.is_none()
        && clock >= colony.next_raid
        && colony.citizens.len() >= 2
        && colony.buildings.iter().filter(|b| done(b)).count() >= 2
    {
        begin_raid(colony, clock, lang, out);
        dirty = true;
    }
    if raid_tick(colony, clock, lang, out) {
        dirty = true;
    }
    ensure_mobs(colony, lang, out);
    if clock >= colony.next_walk {
        colony.next_walk = clock + 15;
        walk(colony, out);
    }
    dirty
}

fn place_work(colony: &mut Colony, clock: u64, lang: Lang, out: &mut Vec<Intent>) {
    let Some(index) = colony
        .buildings
        .iter()
        .position(|b| !done(b) && !b.inflight)
    else {
        return;
    };
    let kind = colony.buildings[index].kind;
    let builder = colony.citizens.iter().any(|c| c.job == Job::Builder);
    if kind != Kind::TownHall && !builder {
        if clock >= colony.next_nag {
            colony.next_nag = clock + BUILD_NAG;
            out.push(Intent::Talk {
                player: colony.owner,
                text: tr(
                    lang,
                    "A blueprint is waiting. /colony hire <id> builder",
                    "Чертёж ждёт. /colony hire <id> builder",
                ),
            });
        }
        return;
    }
    if kind != Kind::TownHall && clock < colony.next_build {
        return;
    }
    let batch = if kind == Kind::TownHall {
        colony.buildings[index].plan.len() - colony.buildings[index].placed as usize
    } else {
        1
    };
    if batch == 0 {
        return;
    }
    let start = colony.buildings[index].placed as usize;
    let block = &colony.buildings[index].plan[start];
    if let Some(item) = &block.item {
        if colony.stock.get(item).copied().unwrap_or(0) == 0 {
            if clock >= colony.next_nag {
                colony.next_nag = clock + BUILD_NAG;
                out.push(Intent::Talk {
                    player: colony.owner,
                    text: tr(
                        lang,
                        &format!("The builder needs {}.", short_item(item)),
                        &format!("Строителю нужно {}.", short_item(item)),
                    ),
                });
            }
            return;
        }
    }
    let blocks = colony.buildings[index].plan[start..start + batch]
        .iter()
        .map(|b| BlockAt {
            x: b.x,
            y: b.y,
            z: b.z,
            state: b.state.clone(),
        })
        .collect();
    colony.buildings[index].inflight = true;
    if kind != Kind::TownHall {
        colony.next_build = clock + pace(colony);
    }
    out.push(Intent::Place {
        colony: colony.id,
        building: colony.buildings[index].id,
        blocks,
    });
}

fn pace(colony: &Colony) -> u64 {
    let courier = colony.citizens.iter().any(|c| c.job == Job::Courier)
        && colony
            .buildings
            .iter()
            .any(|b| b.kind == Kind::Courier && done(b));
    let base = if courier { 8 } else { 14 };
    let mood = mood(colony);
    if mood >= 80 {
        base
    } else if mood >= 50 {
        base + 6
    } else {
        base + 12
    }
}

fn produce(colony: &mut Colony) -> bool {
    let mut changed = false;
    let farmers = colony
        .citizens
        .iter()
        .filter(|c| c.job == Job::Farmer)
        .count() as u32;
    let loggers = colony
        .citizens
        .iter()
        .filter(|c| c.job == Job::Lumberjack)
        .count() as u32;
    let miners = colony
        .citizens
        .iter()
        .filter(|c| c.job == Job::Miner)
        .count() as u32;
    if farmers > 0
        && colony
            .buildings
            .iter()
            .any(|b| b.kind == Kind::Farm && done(b))
    {
        add_stock(&mut colony.stock, "minecraft:wheat", farmers);
        let wheat = colony.stock.get("minecraft:wheat").copied().unwrap_or(0);
        if wheat >= 3 {
            pay(&mut colony.stock, "minecraft:wheat", 3);
            add_stock(&mut colony.stock, "minecraft:bread", 1);
        }
        changed = true;
    }
    if loggers > 0
        && colony
            .buildings
            .iter()
            .any(|b| b.kind == Kind::Lumberjack && done(b))
    {
        add_stock(&mut colony.stock, "minecraft:oak_log", loggers);
        changed = true;
    }
    if miners > 0
        && colony
            .buildings
            .iter()
            .any(|b| b.kind == Kind::Miner && done(b))
    {
        add_stock(&mut colony.stock, "minecraft:cobblestone", miners);
        colony.ore_tick = colony.ore_tick.wrapping_add(1);
        if colony.ore_tick % 5 == 0 {
            add_stock(&mut colony.stock, "minecraft:coal", 1);
        }
        if colony.ore_tick % 12 == 0 {
            add_stock(&mut colony.stock, "minecraft:iron_ingot", 1);
        }
        changed = true;
    }
    changed
}

fn add_stock(stock: &mut BTreeMap<String, u32>, item: &str, n: u32) {
    *stock.entry(item.to_owned()).or_insert(0) += n;
}

fn feed(colony: &mut Colony, lang: Lang, out: &mut Vec<Intent>) -> bool {
    let people = colony.citizens.len() as u32;
    if people == 0 {
        return false;
    }
    let need = (people + 1) / 2;
    let have = colony.stock.get("minecraft:bread").copied().unwrap_or(0);
    if have >= need {
        pay(&mut colony.stock, "minecraft:bread", need);
        colony.happiness = (colony.happiness + 2).min(90);
        true
    } else {
        colony.happiness = (colony.happiness - 4).max(20);
        out.push(Intent::Talk {
            player: colony.owner,
            text: tr(lang, "The citizens are hungry.", "Жители голодны."),
        });
        true
    }
}

fn begin_raid(colony: &mut Colony, clock: u64, lang: Lang, out: &mut Vec<Intent>) {
    let done_n = colony.buildings.iter().filter(|b| done(b)).count().max(1);
    let wanted = (1 + done_n / 3).min(4) as u8;
    colony.raid = Some(Raid {
        hp: i32::from(wanted) * 10,
        until: clock + 600,
        wanted,
        mobs: vec![None; wanted as usize],
    });
    out.push(Intent::Talk {
        player: colony.owner,
        text: tr(
            lang,
            &format!(
                "Raiders approach {}. Guards can turn them back.",
                colony.name
            ),
            &format!(
                "К {} подходят рейдеры. Стража может их отбить.",
                colony.name
            ),
        ),
    });
}

fn raid_tick(colony: &mut Colony, clock: u64, lang: Lang, out: &mut Vec<Intent>) -> bool {
    let Some(raid) = &mut colony.raid else {
        return false;
    };
    if raid.mobs.len() != raid.wanted as usize {
        raid.mobs.resize(raid.wanted as usize, None);
    }
    for (index, slot) in raid.mobs.iter().enumerate() {
        if slot.is_none() {
            let index = index as i32;
            out.push(Intent::Spawn {
                who: SpawnWho::Raider {
                    colony: colony.id,
                    index: index as u8,
                },
                dimension: colony.dimension.clone(),
                pos: [
                    colony.center[0] as f64 + 24.0,
                    colony.center[1] as f64,
                    colony.center[2] as f64 + f64::from(index) * 2.0,
                ],
                name: "Raider".to_owned(),
                kind: "minecraft:pillager".to_owned(),
                profession: String::new(),
                tag: "mc_raider".to_owned(),
            });
        }
    }
    let target = raid.mobs.iter().flatten().copied().next();
    let guards: Vec<u32> = colony
        .citizens
        .iter()
        .filter(|c| c.job == Job::Guard)
        .filter_map(|c| c.entity)
        .collect();
    if clock % 10 == 0 && !guards.is_empty() {
        raid.hp -= 2 * guards.len() as i32;
        if let Some(target) = target {
            for entity in guards {
                out.push(Intent::Attack {
                    dimension: colony.dimension.clone(),
                    entity,
                    target,
                });
            }
        }
    }
    let success = raid.hp <= 0;
    let failed = !success && clock >= raid.until;
    if !success && !failed {
        return false;
    }
    let mobs: Vec<u32> = colony
        .raid
        .take()
        .unwrap()
        .mobs
        .into_iter()
        .flatten()
        .collect();
    for entity in mobs {
        out.push(Intent::Remove {
            dimension: colony.dimension.clone(),
            entity,
        });
    }
    colony.next_raid = clock + RAID_EVERY;
    if success {
        colony.happiness = (colony.happiness + 5).min(90);
        out.push(Intent::Talk {
            player: colony.owner,
            text: tr(lang, "The raid breaks.", "Набег отбит."),
        });
    } else {
        for value in colony.stock.values_mut() {
            *value = *value * 9 / 10;
        }
        colony.happiness = (colony.happiness - 15).max(20);
        out.push(Intent::Talk {
            player: colony.owner,
            text: tr(
                lang,
                "Raiders carry off part of the warehouse.",
                "Рейдеры уносят часть склада.",
            ),
        });
    }
    true
}

fn ensure_mobs(colony: &mut Colony, lang: Lang, out: &mut Vec<Intent>) {
    let posts: Vec<(Job, [i32; 3])> = colony
        .citizens
        .iter()
        .map(|c| (c.job, post_of(colony, c.job)))
        .collect();
    for (citizen, (_, post)) in colony.citizens.iter().zip(posts) {
        if citizen.entity.is_some() {
            continue;
        }
        out.push(Intent::Spawn {
            who: SpawnWho::Citizen {
                colony: colony.id,
                id: citizen.id,
            },
            dimension: colony.dimension.clone(),
            pos: [post[0] as f64 + 0.5, post[1] as f64, post[2] as f64 + 0.5],
            name: citizen_title(lang, citizen),
            kind: "minecraft:villager".to_owned(),
            profession: profession(citizen.job).to_owned(),
            tag: "mc_citizen".to_owned(),
        });
    }
}

fn walk(colony: &Colony, out: &mut Vec<Intent>) {
    for citizen in &colony.citizens {
        let Some(entity) = citizen.entity else {
            continue;
        };
        let post = post_of(colony, citizen.job);
        out.push(Intent::Walk {
            dimension: colony.dimension.clone(),
            entity,
            to: [post[0] as f64 + 0.5, post[1] as f64, post[2] as f64 + 0.5],
        });
    }
}

fn post_of(colony: &Colony, job: Job) -> [i32; 3] {
    let kind = match job {
        Job::Builder => Some(Kind::Builder),
        Job::Farmer => Some(Kind::Farm),
        Job::Lumberjack => Some(Kind::Lumberjack),
        Job::Miner => Some(Kind::Miner),
        Job::Courier => Some(Kind::Courier),
        Job::Guard => Some(Kind::GuardTower),
        Job::Unemployed => Some(Kind::TownHall),
    };
    colony
        .buildings
        .iter()
        .find(|b| Some(b.kind) == kind && done(b))
        .or_else(|| colony.buildings.iter().find(|b| b.kind == Kind::TownHall))
        .map(|b| b.post)
        .unwrap_or(colony.center)
}

fn profession(job: Job) -> &'static str {
    match job {
        Job::Builder => "minecraft:librarian",
        Job::Farmer => "minecraft:farmer",
        Job::Lumberjack => "minecraft:fletcher",
        Job::Miner => "minecraft:armorer",
        Job::Courier => "minecraft:cartographer",
        Job::Guard => "minecraft:weaponsmith",
        Job::Unemployed => "minecraft:nitwit",
    }
}

fn job_kind(job: Job) -> Kind {
    match job {
        Job::Builder => Kind::Builder,
        Job::Farmer => Kind::Farm,
        Job::Lumberjack => Kind::Lumberjack,
        Job::Miner => Kind::Miner,
        Job::Courier => Kind::Courier,
        Job::Guard => Kind::GuardTower,
        Job::Unemployed => Kind::TownHall,
    }
}

fn citizen_title(lang: Lang, citizen: &Citizen) -> String {
    let job = job_label(lang, citizen.job);
    match lang {
        Lang::En => format!("{} the {job}", citizen.name),
        Lang::Ru => format!("{} — {job}", citizen.name),
    }
}

fn parse_kind(word: &str) -> Option<Kind> {
    Some(match word {
        "builder" => Kind::Builder,
        "warehouse" => Kind::Warehouse,
        "residence" | "home" | "house" => Kind::Residence,
        "farm" | "farmer" => Kind::Farm,
        "lumberjack" | "lumber" | "forester" => Kind::Lumberjack,
        "miner" | "mine" => Kind::Miner,
        "guardtower" | "guard" | "guard_tower" => Kind::GuardTower,
        "tavern" => Kind::Tavern,
        "courier" | "deliveryman" | "delivery" => Kind::Courier,
        "townhall" | "town_hall" => Kind::TownHall,
        _ => return None,
    })
}

fn parse_job(word: &str) -> Option<Job> {
    Some(match word {
        "builder" => Job::Builder,
        "farmer" | "farm" => Job::Farmer,
        "lumberjack" | "lumber" | "forester" => Job::Lumberjack,
        "miner" | "mine" => Job::Miner,
        "courier" | "deliveryman" => Job::Courier,
        "guard" | "guardtower" => Job::Guard,
        "unemployed" | "none" | "fire" => Job::Unemployed,
        _ => return None,
    })
}

struct Node {
    id: &'static str,
    unlocks: Kind,
    cost: &'static [(&'static str, u32)],
}

fn research_node(id: &str) -> Option<Node> {
    Some(match id {
        "woodworking" => Node {
            id: "woodworking",
            unlocks: Kind::Lumberjack,
            cost: &[("minecraft:oak_log", 16)],
        },
        "mining" => Node {
            id: "mining",
            unlocks: Kind::Miner,
            cost: &[("minecraft:cobblestone", 32)],
        },
        "storage" => Node {
            id: "storage",
            unlocks: Kind::Warehouse,
            cost: &[("minecraft:oak_planks", 16)],
        },
        "logistics" => Node {
            id: "logistics",
            unlocks: Kind::Courier,
            cost: &[("minecraft:oak_planks", 16)],
        },
        "watching" => Node {
            id: "watching",
            unlocks: Kind::GuardTower,
            cost: &[("minecraft:cobblestone", 16)],
        },
        "hospitality" => Node {
            id: "hospitality",
            unlocks: Kind::Tavern,
            cost: &[("minecraft:bread", 8)],
        },
        _ => return None,
    })
}

fn research_for(kind: Kind) -> Option<&'static str> {
    Some(match kind {
        Kind::Lumberjack => "woodworking",
        Kind::Miner => "mining",
        Kind::Warehouse => "storage",
        Kind::Courier => "logistics",
        Kind::GuardTower => "watching",
        Kind::Tavern => "hospitality",
        Kind::TownHall | Kind::Builder | Kind::Residence | Kind::Farm => return None,
    })
}

pub fn stock_alias(word: &str) -> Option<&'static str> {
    Some(match word {
        "planks" | "oak_planks" => "minecraft:oak_planks",
        "logs" | "log" | "oak_log" => "minecraft:oak_log",
        "cobble" | "cobblestone" | "stone" => "minecraft:cobblestone",
        "slab" | "slabs" | "oak_slab" => "minecraft:oak_slab",
        "glass" => "minecraft:glass",
        "door" | "doors" | "oak_door" => "minecraft:oak_door",
        "bread" => "minecraft:bread",
        "wheat" => "minecraft:wheat",
        "coal" => "minecraft:coal",
        "iron" | "iron_ingot" => "minecraft:iron_ingot",
        _ => return None,
    })
}

fn short_item(item: &str) -> &str {
    item.rsplit_once(':').map(|(_, n)| n).unwrap_or(item)
}

fn kind_label(lang: Lang, kind: Kind) -> &'static str {
    match (lang, kind) {
        (Lang::En, Kind::TownHall) => "town hall",
        (Lang::Ru, Kind::TownHall) => "ратуша",
        (Lang::En, Kind::Builder) => "builder's hut",
        (Lang::Ru, Kind::Builder) => "хижина строителя",
        (Lang::En, Kind::Warehouse) => "warehouse",
        (Lang::Ru, Kind::Warehouse) => "склад",
        (Lang::En, Kind::Residence) => "residence",
        (Lang::Ru, Kind::Residence) => "дом",
        (Lang::En, Kind::Farm) => "farm",
        (Lang::Ru, Kind::Farm) => "ферма",
        (Lang::En, Kind::Lumberjack) => "lumberjack's hut",
        (Lang::Ru, Kind::Lumberjack) => "хижина дровосека",
        (Lang::En, Kind::Miner) => "mine",
        (Lang::Ru, Kind::Miner) => "шахта",
        (Lang::En, Kind::GuardTower) => "guard tower",
        (Lang::Ru, Kind::GuardTower) => "сторожевая башня",
        (Lang::En, Kind::Tavern) => "tavern",
        (Lang::Ru, Kind::Tavern) => "таверна",
        (Lang::En, Kind::Courier) => "courier's hut",
        (Lang::Ru, Kind::Courier) => "хижина курьера",
    }
}

fn job_label(lang: Lang, job: Job) -> &'static str {
    match (lang, job) {
        (Lang::En, Job::Unemployed) => "unemployed",
        (Lang::Ru, Job::Unemployed) => "без работы",
        (Lang::En, Job::Builder) => "builder",
        (Lang::Ru, Job::Builder) => "строитель",
        (Lang::En, Job::Farmer) => "farmer",
        (Lang::Ru, Job::Farmer) => "фермер",
        (Lang::En, Job::Lumberjack) => "lumberjack",
        (Lang::Ru, Job::Lumberjack) => "дровосек",
        (Lang::En, Job::Miner) => "miner",
        (Lang::Ru, Job::Miner) => "шахтёр",
        (Lang::En, Job::Courier) => "courier",
        (Lang::Ru, Job::Courier) => "курьер",
        (Lang::En, Job::Guard) => "guard",
        (Lang::Ru, Job::Guard) => "стражник",
    }
}

fn snapshot(colony: &Colony, lang: Lang) -> String {
    let mood = format!("{:.1}", mood(colony) as f32 / 10.0);
    let line1 = format!("{}  {mood}", colony.name);
    let line2 = if let Some(building) = colony.buildings.iter().find(|b| !done(b)) {
        format!(
            "{} {}/{}",
            kind_label(lang, building.kind),
            building.placed,
            building.plan.len()
        )
    } else {
        tr(
            lang,
            &format!("{} citizens", colony.citizens.len()),
            &format!("жителей: {}", colony.citizens.len()),
        )
    };
    let line3 = format!(
        "{} {}  {} {}  {} {}",
        short_item("minecraft:bread"),
        colony.stock.get("minecraft:bread").copied().unwrap_or(0),
        short_item("minecraft:oak_log"),
        colony.stock.get("minecraft:oak_log").copied().unwrap_or(0),
        short_item("minecraft:cobblestone"),
        colony
            .stock
            .get("minecraft:cobblestone")
            .copied()
            .unwrap_or(0),
    );
    format!("{line1}\n{line2}\n{line3}")
}

fn info_lines(colony: &Colony, lang: Lang) -> Vec<String> {
    let mut lines = vec![snapshot(colony, lang).replace('\n', " · ")];
    if let Some(raid) = &colony.raid {
        lines.push(tr(
            lang,
            &format!("Raid strength {}", raid.hp),
            &format!("Сила набега {}", raid.hp),
        ));
    }
    lines.push(tr(
        lang,
        &format!(
            "Claim centered at {} {} {}.",
            colony.center[0], colony.center[1], colony.center[2]
        ),
        &format!(
            "Граница вокруг {} {} {}.",
            colony.center[0], colony.center[1], colony.center[2]
        ),
    ));
    lines
}

fn citizen_lines(colony: &Colony, lang: Lang) -> Vec<String> {
    if colony.citizens.is_empty() {
        return vec![tr(lang, "No citizens yet.", "Жителей пока нет.")];
    }
    colony
        .citizens
        .iter()
        .map(|c| format!("{}  {}  {}", c.id, c.name, job_label(lang, c.job)))
        .collect()
}

fn stock_lines(colony: &Colony, lang: Lang) -> Vec<String> {
    if colony.stock.is_empty() {
        return vec![tr(lang, "The warehouse is empty.", "Склад пуст.")];
    }
    let mut lines = vec![tr(lang, "Warehouse", "Склад")];
    for (item, count) in &colony.stock {
        lines.push(format!("{} {count}", short_item(item)));
    }
    lines
}

fn research_lines(colony: &Colony, lang: Lang) -> Vec<String> {
    let mut lines = vec![tr(lang, "Research", "Исследования")];
    for id in [
        "woodworking",
        "mining",
        "storage",
        "logistics",
        "watching",
        "hospitality",
    ] {
        let node = research_node(id).expect("node");
        let mark = if colony.research.contains(id) {
            "✔"
        } else {
            "·"
        };
        let (item, cost) = node.cost[0];
        lines.push(format!("{mark} {id}  {cost} {}", short_item(item)));
    }
    lines.push(tr(
        lang,
        "Builder, residence and farm need no research.",
        "Строитель, дом и ферма не требуют исследования.",
    ));
    lines
}

fn help_lines(lang: Lang) -> Vec<String> {
    vec![tr(
        lang,
        " /colony found [name] · build <hut> · hire <id> <job> · deposit · withdraw <item> <n> · research [node] · raid · recall · abandon confirm",
        " /colony found [имя] · build <хижина> · hire <id> <работа> · deposit · withdraw <предмет> <n> · research [узел] · raid · recall · abandon confirm",
    )]
}

pub fn settings_labels(lang: Lang, hud: bool) -> Vec<String> {
    let language = match lang {
        Lang::En => "Language: English",
        Lang::Ru => "Язык: русский",
    };
    let hud = if hud {
        tr(lang, "Colony HUD: shown", "Надпись колонии: видна")
    } else {
        tr(lang, "Colony HUD: hidden", "Надпись колонии: скрыта")
    };
    vec![language.to_owned(), hud]
}

pub fn toggle_setting(button: usize, lang: Lang, hud: bool) -> (Lang, bool) {
    match button {
        0 => (if lang == Lang::En { Lang::Ru } else { Lang::En }, hud),
        1 => (lang, !hud),
        _ => (lang, hud),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn actor(uuid: u128, x: i32, z: i32) -> Actor {
        Actor {
            uuid,
            name: "Alex".into(),
            dimension: OVERWORLD.into(),
            x,
            y: 64,
            z,
            yaw: 0.0,
        }
    }

    fn drive(world: &mut World, n: u32) {
        let mut next = 1u32;
        for _ in 0..n {
            let intents = world.tick(OVERWORLD);
            for intent in intents {
                match intent {
                    Intent::Place {
                        colony,
                        building,
                        blocks,
                    } => {
                        world.place_done(colony, building, blocks.len() as u32);
                    }
                    Intent::Spawn { who, .. } => {
                        world.note_mob(who, next);
                        next += 1;
                    }
                    _ => {}
                }
            }
        }
    }

    fn owned(world: &World) -> &Colony {
        world.colonies.iter().find(|c| c.owner == 7).unwrap()
    }

    #[test]
    fn yaw_picks_the_door_that_faces_the_player() {
        assert_eq!(rot_from_yaw(0.0), 0);
        assert_eq!(rot_from_yaw(90.0), 1);
        assert_eq!(rot_from_yaw(180.0), 2);
        assert_eq!(rot_from_yaw(-90.0), 3);
        assert_eq!(rot_from_face([0, 1, 0], 90.0), 1);
        assert_eq!(rot_from_face([0, 0, 1], 0.0), 2);
    }

    #[test]
    fn every_hut_fits_in_the_supply_camp_and_keeps_its_block() {
        for kind in [
            Kind::TownHall,
            Kind::Builder,
            Kind::Warehouse,
            Kind::Residence,
            Kind::Farm,
            Kind::Lumberjack,
            Kind::Miner,
            Kind::GuardTower,
            Kind::Tavern,
            Kind::Courier,
        ] {
            let draft = layout(kind, [10, 64, 20], 0);
            assert!(
                draft.plan.iter().any(|b| b.state == hut_block(kind)),
                "{kind:?}"
            );
            let mut need: BTreeMap<&str, u32> = BTreeMap::new();
            for block in &draft.plan {
                if let Some(item) = &block.item {
                    *need.entry(item).or_insert(0) += 1;
                }
            }
            for (item, n) in need {
                let have = SUPPLY
                    .iter()
                    .find(|(id, _)| *id == item)
                    .map(|(_, c)| *c)
                    .unwrap_or(0);
                assert!(have >= n, "{kind:?} needs {n} {item}, supply has {have}");
            }
        }
    }

    #[test]
    fn founding_raises_a_town_hall_and_two_citizens() {
        let mut world = World::new();
        let reply = world.command(&actor(7, 0, 0), "colony found Riverside");
        assert!(reply.handled);
        assert!(reply.lines[0].contains("Riverside"));
        drive(&mut world, 3);
        let colony = owned(&world);
        assert!(colony
            .buildings
            .iter()
            .any(|b| b.kind == Kind::TownHall && done(b)));
        assert_eq!(colony.citizens.len(), 2);
        assert!(colony.stock.get("minecraft:bread").copied().unwrap_or(0) >= 32);
        let again = world.command(&actor(7, 0, 0), "colony found Other");
        assert!(again.lines[0].contains("already") || again.lines[0].contains("уже"));
    }

    #[test]
    fn builder_hut_then_farm_then_bread() {
        let mut world = World::new();
        world.command(&actor(7, 0, 0), "colony found Riverside");
        drive(&mut world, 5);
        let planned = world.command(&actor(7, 6, 0), "colony build builder");
        assert!(
            planned.lines[0].contains("builder") || planned.lines[0].contains("строител"),
            "{planned:?}"
        );
        world.command(&actor(7, 6, 0), "colony hire 1 builder");
        drive(&mut world, 2_500);
        assert!(owned(&world)
            .buildings
            .iter()
            .any(|b| b.kind == Kind::Builder && done(b)));
        let farm = world.command(&actor(7, 0, 12), "colony build farm");
        assert!(
            farm.handled
                && !farm.lines[0].contains("overlap")
                && !farm.lines[0].contains("пересек"),
            "{farm:?}"
        );
        drive(&mut world, 2_500);
        assert!(owned(&world)
            .buildings
            .iter()
            .any(|b| b.kind == Kind::Farm && done(b)));
        world.command(&actor(7, 0, 8), "colony hire 2 farmer");
        world.colonies[0].next_food = u64::MAX;
        world.colonies[0].next_work = 0;
        world.colonies[0].stock.insert("minecraft:wheat".into(), 3);
        let before = owned(&world)
            .stock
            .get("minecraft:bread")
            .copied()
            .unwrap_or(0);
        drive(&mut world, 5);
        let after = owned(&world)
            .stock
            .get("minecraft:bread")
            .copied()
            .unwrap_or(0);
        assert!(after > before, "bread {before} -> {after}");
    }

    #[test]
    fn research_gates_the_lumberjack_and_spends_logs() {
        let mut world = World::new();
        world.command(&actor(7, 0, 0), "colony found Mill");
        drive(&mut world, 2);
        let denied = world.command(&actor(7, 8, 0), "colony build lumberjack");
        assert!(denied.lines[0].contains("woodworking"));
        let logs = owned(&world).stock["minecraft:oak_log"];
        world.command(&actor(7, 0, 0), "colony research woodworking");
        assert!(owned(&world).stock["minecraft:oak_log"] < logs);
        let ok = world.command(&actor(7, 8, 0), "colony build lumberjack");
        assert!(ok.handled);
        assert!(owned(&world)
            .buildings
            .iter()
            .any(|b| b.kind == Kind::Lumberjack));
    }

    #[test]
    fn claims_stop_at_the_border_and_reject_a_neighbour() {
        let mut world = World::new();
        world.command(&actor(7, 0, 0), "colony found A");
        let center = owned(&world).center;
        assert!(world.protect(OVERWORLD, center, 7).is_none());
        assert!(world.protect(OVERWORLD, center, 9).is_some());
        assert!(world
            .protect(OVERWORLD, [center[0] + 500, center[1], center[2]], 9)
            .is_none());
        let near = world.command(&actor(9, center[0] + 10, center[2]), "colony found B");
        assert!(near.lines[0].contains("close") || near.lines[0].contains("близко"));
        let far = world.command(&actor(9, center[0] + 400, center[2]), "colony found B");
        assert!(far.lines[0].contains('B') || far.lines[0].contains('б') || far.handled);
        assert_eq!(world.colonies.len(), 2);
    }

    #[test]
    fn warehouse_withdraw_and_deposit_round_trip_in_the_books() {
        let mut world = World::new();
        world.command(&actor(7, 0, 0), "colony found A");
        let order = world.command(&actor(7, 0, 0), "colony withdraw planks 10");
        match order.order {
            Order::Withdraw { item, count } => {
                assert_eq!(item, "minecraft:oak_planks");
                assert_eq!(count, 10);
                assert_eq!(world.take_stock(7, &item, count), 10);
            }
            other => panic!("{other:?}"),
        }
        world.give_stock(7, "minecraft:oak_planks", 4);
        assert_eq!(owned(&world).stock["minecraft:oak_planks"], 250);
    }

    #[test]
    fn hungry_raid_and_a_guard_who_ends_it() {
        let mut world = World::new();
        world.command(&actor(7, 0, 0), "colony found A");
        drive(&mut world, 2);
        world.colonies[0].next_food = 0;
        world.colonies[0].stock.insert("minecraft:bread".into(), 0);
        world.tick(OVERWORLD);
        assert_eq!(owned(&world).happiness, 56);
        world.colonies[0].citizens[0].job = Job::Guard;
        world.command(&actor(7, 0, 0), "colony raid");
        assert!(owned(&world).raid.is_some());
        drive(&mut world, 120);
        assert!(
            owned(&world).raid.is_none(),
            "guard should break a small raid"
        );
    }

    #[test]
    fn a_raid_without_guards_steals_from_the_warehouse() {
        let mut world = World::new();
        world.command(&actor(7, 0, 0), "colony found A");
        drive(&mut world, 2);
        let logs = owned(&world).stock["minecraft:oak_log"];
        world.command(&actor(7, 0, 0), "colony raid");
        world.colonies[0].raid.as_mut().unwrap().until = world.clock;
        world.tick(OVERWORLD);
        assert!(owned(&world).raid.is_none());
        assert!(owned(&world).stock["minecraft:oak_log"] < logs);
    }

    #[test]
    fn the_save_remembers_the_colony_and_calls_citizens_home() {
        let mut world = World::new();
        world.command(&actor(7, 0, 0), "colony found Riverside");
        drive(&mut world, 5);
        world.note_mob(
            SpawnWho::Citizen {
                colony: owned(&world).id,
                id: 1,
            },
            42,
        );
        let text = world.to_json();
        let loaded = World::from_json(&text).unwrap();
        assert_eq!(loaded.colonies.len(), 1);
        assert_eq!(loaded.colonies[0].name, "Riverside");
        assert!(loaded.colonies[0].recall);
        assert!(loaded.colonies[0]
            .citizens
            .iter()
            .all(|c| c.entity.is_none()));
        assert!(loaded.colonies[0]
            .buildings
            .iter()
            .any(|b| b.kind == Kind::TownHall && done(b)));
    }

    #[test]
    fn russian_help_and_a_bad_file_warns_once() {
        let mut world = World::new();
        world.set_lang(Lang::Ru);
        let help = world.command(&actor(7, 0, 0), "colony help");
        assert!(help.lines[0].contains("hire"));
        assert!(help.lines[0].contains("имя") || help.lines[0].contains("работа"));
        let mut broken = World::open_file(Some("{"));
        let reply = broken.command(&actor(7, 0, 0), "colony");
        assert!(
            reply.lines[0].contains("EOF")
                || reply.lines[0].contains("expected")
                || reply.lines.len() > 1
        );
        let again = broken.command(&actor(7, 0, 0), "colony");
        assert_eq!(again.lines.len(), 1);
    }

    #[test]
    fn abandon_needs_confirm_and_a_stranger_command_is_not_ours() {
        let mut world = World::new();
        assert!(!world.command(&actor(7, 0, 0), "time set day").handled);
        assert!(!world.command(&actor(7, 0, 0), "colonyx").handled);
        world.command(&actor(7, 0, 0), "colony found A");
        assert_eq!(world.colonies.len(), 1);
        world.command(&actor(7, 0, 0), "colony abandon");
        assert_eq!(world.colonies.len(), 1);
        world.command(&actor(7, 0, 0), "colony abandon confirm");
        assert!(world.colonies.is_empty());
    }

    #[test]
    fn settings_toggle_language_and_the_hud() {
        let (lang, hud) = toggle_setting(0, Lang::En, true);
        assert_eq!(lang, Lang::Ru);
        let (lang, hud) = toggle_setting(1, lang, hud);
        assert!(!hud);
        assert_eq!(settings_labels(lang, hud).len(), 2);
    }
}
