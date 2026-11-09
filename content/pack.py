#!/usr/bin/env python3
"""Write the MineColonies content mod: hut blocks, the supply camp and the build tool."""

import json
import struct
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parent

HUTS = {
    "blockhuttownhall": ((28, 58, 112), (212, 168, 72), "Town Hall", "Ратуша", 13),
    "blockhutbuilder": ((112, 74, 42), (220, 176, 64), "Builder's Hut", "Хижина строителя", 15),
    "blockhutwarehouse": ((92, 96, 104), (186, 154, 92), "Warehouse", "Склад", 11),
    "blockhutcitizen": ((168, 122, 86), (232, 206, 170), "Residence", "Дом", 14),
    "blockhutfarmer": ((64, 112, 48), (196, 168, 64), "Farm", "Ферма", 7),
    "blockhutlumberjack": ((46, 92, 52), (120, 78, 42), "Lumberjack's Hut", "Хижина дровосека", 26),
    "blockhutminer": ((72, 72, 78), (196, 122, 64), "Mine", "Шахта", 11),
    "blockhutguardtower": ((96, 42, 42), (180, 180, 186), "Guard Tower", "Сторожевая башня", 28),
    "blockhuttavern": ((112, 48, 72), (212, 176, 92), "Tavern", "Таверна", 16),
    "blockhutdeliveryman": ((48, 92, 128), (220, 196, 120), "Courier's Hut", "Хижина курьера", 25),
    "blockhutfisherman": ((36, 96, 128), (214, 206, 160), "Fisherman's Hut", "Хижина рыбака", 12),
    "blockhutsawmill": ((120, 86, 48), (210, 176, 96), "Sawmill", "Лесопилка", 13),
    "blockhutbaker": ((168, 112, 64), (232, 196, 140), "Bakery", "Пекарня", 14),
    "blockhutrestaurant": ((140, 64, 48), (220, 180, 96), "Restaurant", "Ресторан", 15),
    "blockhutuniversity": ((64, 72, 120), (196, 186, 150), "University", "Университет", 11),
    "blockhutlibrary": ((92, 64, 48), (210, 190, 140), "Library", "Библиотека", 14),
    "blockhuthospital": ((180, 180, 186), (120, 168, 160), "Hospital", "Больница", 3),
    "blockhutbarracks": ((72, 78, 86), (160, 64, 48), "Barracks", "Казармы", 7),
    "blockhutsmeltery": ((64, 64, 68), (180, 90, 48), "Smeltery", "Плавильня", 12),
    "blockhutstonemason": ((110, 110, 114), (180, 180, 176), "Stonemason", "Каменотёс", 4),
    "blockhutcrusher": ((90, 90, 94), (120, 80, 60), "Crusher", "Дробилка", 8),
    "blockhutsifter": ((120, 110, 80), (180, 160, 90), "Sifter", "Сито", 6),
    "blockhutcomposter": ((70, 90, 40), (120, 80, 40), "Composter", "Компостер", 5),
    "blockhutblacksmith": ((50, 50, 54), (160, 80, 40), "Blacksmith", "Кузница", 11),
    "blockhutfletcher": ((90, 70, 40), (180, 150, 70), "Fletcher", "Лучник-мастер", 9),
    "blockhutglassblower": ((140, 180, 190), (220, 230, 240), "Glassblower", "Стеклодув", 3),
    "blockhutcowboy": ((120, 80, 50), (200, 160, 100), "Cowboy", "Коровник", 10),
    "blockhutshepherd": ((180, 180, 170), (120, 90, 60), "Shepherd", "Пастух", 8),
    "blockhutchickenherder": ((190, 170, 80), (220, 200, 120), "Chicken Herder", "Курятник", 14),
    "blockhutswineherder": ((160, 90, 90), (200, 140, 120), "Swineherder", "Свинарник", 6),
    "blockhutrabbithutch": ((170, 140, 110), (210, 190, 160), "Rabbit Hutch", "Крольчатник", 2),
    "blockhutbeekeeper": ((190, 150, 40), (80, 60, 30), "Beekeeper", "Пасека", 4),
    "blockhutschool": ((90, 120, 160), (200, 190, 140), "School", "Школа", 11),
    "blockhutgraveyard": ((70, 80, 70), (140, 140, 130), "Graveyard", "Кладбище", 3),
    "blockhutalchemist": ((80, 40, 100), (160, 120, 180), "Alchemist", "Алхимик", 5),
    "blockhutmechanic": ((70, 80, 90), (160, 140, 80), "Mechanic", "Механик", 7),
    "blockhutnetherworker": ((80, 30, 30), (160, 60, 40), "Nether Miner", "Шахтёр Незера", 14),
    "blockhutplantation": ((50, 110, 40), (160, 140, 50), "Plantation", "Плантация", 13),
    "blockhutflorist": ((160, 60, 80), (220, 140, 160), "Florist", "Цветовод", 2),
    "blockhutdyer": ((140, 40, 40), (200, 80, 60), "Dyer", "Красильщик", 1),
    "blockhutconcretemixer": ((140, 140, 140), (180, 180, 180), "Concrete Mixer", "Бетономешалка", 0),
    "blockhutarchery": ((90, 110, 60), (160, 130, 70), "Archery", "Стрельбище", 13),
    "blockhutcombatacademy": ((80, 70, 70), (140, 50, 50), "Combat Academy", "Боевая академия", 15),
}


def png(width, height, pixels):
    def chunk(tag, data):
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

    raw = b"".join(b"\x00" + pixels[y * width * 4 : (y + 1) * width * 4] for y in range(height))
    header = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")


def face(wall, trim):
    pixels = bytearray(16 * 16 * 4)
    for y in range(16):
        for x in range(16):
            border = x in (0, 15) or y in (0, 15)
            door = 6 <= x <= 9 and 7 <= y <= 14
            window = (x in (3, 4, 11, 12) and y in (4, 5, 6))
            beam = y == 7 and not door
            color = trim if border or door or window or beam else wall
            i = (y * 16 + x) * 4
            pixels[i : i + 3] = bytes(color)
            pixels[i + 3] = 255
    return png(16, 16, pixels)


def item_icon(kind):
    pixels = bytearray(16 * 16 * 4)
    for y in range(16):
        for x in range(16):
            if kind == "camp":
                on = (5 <= y <= 12 and 3 <= x <= 12) or (y == 4 and 5 <= x <= 10) or (y == 3 and 7 <= x <= 8)
                color = (146, 96, 48) if y >= 8 else (214, 186, 120)
            elif kind == "ship":
                on = (y in (8, 9) and 2 <= x <= 13) or (y == 7 and 4 <= x <= 11) or (x == 8 and 3 <= y <= 6)
                color = (92, 64, 36) if y >= 8 else (186, 154, 92)
            elif kind == "banner":
                on = (x == 4 and 2 <= y <= 14) or (5 <= x <= 12 and 3 <= y <= 9)
                color = (176, 48, 48) if x > 4 else (120, 78, 42)
            else:
                on = (x in (7, 8) and 2 <= y <= 14) or (y in (3, 4) and 4 <= x <= 11)
                color = (212, 170, 64) if y < 6 else (120, 78, 42)
            i = (y * 16 + x) * 4
            if on:
                pixels[i : i + 3] = bytes(color)
                pixels[i + 3] = 255
    return png(16, 16, pixels)


def write(path, text):
    path = ROOT / path
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text if text.endswith("\n") else text + "\n", encoding="utf-8")


def main():
    en = {}
    ru = {}
    for block, (wall, trim, en_name, ru_name, color) in HUTS.items():
        write(
            f"data/minecolonies/horizon/block/{block}.json",
            json.dumps(
                {
                    "template": "minecraft:oak_planks",
                    "item": True,
                    "properties": {"hardness": 2.0, "sounds": "minecraft:wood", "map_color": color},
                },
                indent=2,
            ),
        )
        write(
            f"data/minecolonies/loot_table/blocks/{block}.json",
            json.dumps(
                {
                    "type": "minecraft:block",
                    "pools": [
                        {
                            "condition": {"type": "minecraft:survives_explosion"},
                            "rolls": 1,
                            "entries": [{"type": "minecraft:item", "name": f"minecolonies:{block}"}],
                        }
                    ],
                    "random_sequence": f"minecolonies:blocks/{block}",
                },
                indent=2,
            ),
        )
        write(
            f"assets/minecolonies/blockstates/{block}.json",
            json.dumps({"variants": {"": {"model": f"minecolonies:block/{block}"}}}, indent=2),
        )
        write(
            f"assets/minecolonies/models/block/{block}.json",
            json.dumps(
                {"parent": "minecraft:block/cube_all", "textures": {"all": f"minecolonies:block/{block}"}},
                indent=2,
            ),
        )
        write(
            f"assets/minecolonies/models/item/{block}.json",
            json.dumps({"parent": f"minecolonies:block/{block}"}, indent=2),
        )
        write(
            f"assets/minecolonies/items/{block}.json",
            json.dumps(
                {"model": {"type": "minecraft:model", "model": f"minecolonies:block/{block}"}},
                indent=2,
            ),
        )
        path = ROOT / f"assets/minecolonies/textures/block/{block}.png"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(face(wall, trim))
        en[f"block.minecolonies.{block}"] = en_name
        ru[f"block.minecolonies.{block}"] = ru_name

    write(
        "data/minecraft/tags/block/mineable/axe.json",
        json.dumps({"values": [f"minecolonies:{block}" for block in HUTS]}, indent=2),
    )
    for item, template, rarity, en_name, ru_name in (
        ("supplycamp", "minecraft:paper", "uncommon", "Supply Camp", "Лагерь снабжения"),
        ("supplyship", "minecraft:paper", "uncommon", "Supply Ship", "Корабль снабжения"),
        ("buildtool", "minecraft:stick", "uncommon", "Build Tool", "Инструмент строительства"),
        ("rallybanner", "minecraft:paper", "uncommon", "Rally Banner", "Знамя сбора"),
    ):
        write(
            f"data/minecolonies/horizon/item/{item}.json",
            json.dumps({"template": template, "max_stack_size": 1, "rarity": rarity}, indent=2),
        )
        write(
            f"assets/minecolonies/models/item/{item}.json",
            json.dumps(
                {"parent": "minecraft:item/generated", "textures": {"layer0": f"minecolonies:item/{item}"}},
                indent=2,
            ),
        )
        write(
            f"assets/minecolonies/items/{item}.json",
            json.dumps(
                {"model": {"type": "minecraft:model", "model": f"minecolonies:item/{item}"}},
                indent=2,
            ),
        )
        path = ROOT / f"assets/minecolonies/textures/item/{item}.png"
        path.parent.mkdir(parents=True, exist_ok=True)
        icon = {"supplycamp": "camp", "supplyship": "ship", "rallybanner": "banner"}.get(item, "tool")
        path.write_bytes(item_icon(icon))
        en[f"item.minecolonies.{item}"] = en_name
        ru[f"item.minecolonies.{item}"] = ru_name

    write(
        "data/minecolonies/recipe/supplycamp.json",
        json.dumps(
            {
                "type": "minecraft:crafting_shapeless",
                "category": "misc",
                "ingredients": ["minecraft:chest", "minecraft:bread", "minecraft:oak_log"],
                "result": {"id": "minecolonies:supplycamp", "count": 1},
            },
            indent=2,
        ),
    )
    write(
        "data/minecolonies/recipe/buildtool.json",
        json.dumps(
            {
                "type": "minecraft:crafting_shaped",
                "category": "equipment",
                "pattern": [" S ", "SLS", " S "],
                "key": {"S": "minecraft:stick", "L": "minecraft:leather"},
                "result": {"id": "minecolonies:buildtool", "count": 1},
            },
            indent=2,
        ),
    )
    write(
        "data/minecolonies/recipe/supplyship.json",
        json.dumps(
            {
                "type": "minecraft:crafting_shaped",
                "category": "misc",
                "pattern": [" B ", "BBB", " B "],
                "key": {"B": "minecraft:oak_boat"},
                "result": {"id": "minecolonies:supplyship", "count": 1},
            },
            indent=2,
        ),
    )
    centers = {
        "blockhutbuilder": "minecraft:oak_door",
        "blockhutwarehouse": "minecraft:barrel",
        "blockhutcitizen": "minecraft:chest",
        "blockhutfarmer": "minecraft:wooden_hoe",
        "blockhutlumberjack": "minecraft:wooden_axe",
        "blockhutminer": "minecraft:wooden_pickaxe",
        "blockhutguardtower": "minecraft:wooden_sword",
        "blockhuttavern": "minecraft:red_bed",
        "blockhutdeliveryman": "minecraft:string",
        "blockhutfisherman": "minecraft:fishing_rod",
        "blockhutsawmill": "minecraft:iron_axe",
        "blockhutbaker": "minecraft:wheat",
        "blockhutrestaurant": "minecraft:bread",
        "blockhutuniversity": "minecraft:book",
        "blockhutlibrary": "minecraft:bookshelf",
        "blockhuthospital": "minecraft:glistering_melon_slice",
        "blockhutbarracks": "minecraft:iron_sword",
        "blockhutsmeltery": "minecraft:furnace",
        "blockhutstonemason": "minecraft:stone",
        "blockhutcrusher": "minecraft:cobblestone",
        "blockhutsifter": "minecraft:gravel",
        "blockhutcomposter": "minecraft:composter",
        "blockhutblacksmith": "minecraft:iron_ingot",
        "blockhutfletcher": "minecraft:flint",
        "blockhutglassblower": "minecraft:sand",
        "blockhutcowboy": "minecraft:leather",
        "blockhutshepherd": "minecraft:white_wool",
        "blockhutchickenherder": "minecraft:egg",
        "blockhutswineherder": "minecraft:porkchop",
        "blockhutrabbithutch": "minecraft:rabbit",
        "blockhutbeekeeper": "minecraft:honeycomb",
        "blockhutschool": "minecraft:feather",
        "blockhutgraveyard": "minecraft:soul_sand",
        "blockhutalchemist": "minecraft:brewing_stand",
        "blockhutmechanic": "minecraft:piston",
        "blockhutnetherworker": "minecraft:netherrack",
        "blockhutplantation": "minecraft:carrot",
        "blockhutflorist": "minecraft:poppy",
        "blockhutdyer": "minecraft:red_dye",
        "blockhutconcretemixer": "minecraft:white_concrete_powder",
        "blockhutarchery": "minecraft:bow",
        "blockhutcombatacademy": "minecraft:shield",
    }
    for block, center in centers.items():
        write(
            f"data/minecolonies/recipe/{block}.json",
            json.dumps(
                {
                    "type": "minecraft:crafting_shaped",
                    "category": "building",
                    "pattern": ["PPP", "PCP", "PPP"],
                    "key": {"P": "minecraft:oak_planks", "C": center},
                    "result": {"id": f"minecolonies:{block}", "count": 1},
                },
                indent=2,
            ),
        )
    write(
        "data/minecolonies/recipe/rallybanner.json",
        json.dumps(
            {
                "type": "minecraft:crafting_shaped",
                "category": "misc",
                "pattern": ["W", "S"],
                "key": {"W": "minecraft:white_banner", "S": "minecraft:stick"},
                "result": {"id": "minecolonies:rallybanner", "count": 1},
            },
            indent=2,
        ),
    )
    items = [f"minecolonies:{block}" for block in HUTS] + [
        "minecolonies:supplycamp",
        "minecolonies:supplyship",
        "minecolonies:buildtool",
        "minecolonies:rallybanner",
    ]
    write(
        "data/minecolonies/horizon/creative_tab/colony.json",
        json.dumps({"title": "itemGroup.minecolonies.colony", "icon": "minecolonies:blockhuttownhall", "items": items}, indent=2),
    )
    en["itemGroup.minecolonies.colony"] = "MineColonies"
    ru["itemGroup.minecolonies.colony"] = "МайнКолонии"
    write("assets/minecolonies/lang/en_us.json", json.dumps(en, indent=2, ensure_ascii=False))
    write("assets/minecolonies/lang/ru_ru.json", json.dumps(ru, indent=2, ensure_ascii=False))
    print(f"wrote {len(HUTS)} huts")


if __name__ == "__main__":
    main()
