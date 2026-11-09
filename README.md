# MineColonies for Horizon

An independent colony loop for [Horizon](https://github.com/horizon-rust/horizon), the Rust game that plays Minecraft 26.3. You found a colony, a builder raises huts from the warehouse, citizens take jobs, and a raid tests the guard. It is not a port of the Java mod and it is not affiliated with ldtteam. No Java sources or MineColonies assets are included. License: MIT.

Java, Forge and Fabric jars do not run in Horizon. This project is two mods that do: a content pack of hut blocks, and a native mod that plays the colony.

## Как играть

Оба мода должны быть включены, и мир нужно открыть заново после блоков контента.

1. Скрафтите лагерь снабжения: сундук, хлеб и дубовое бревно. Либо возьмите его во вкладке «МайнКолонии».
2. Поставьте его в обычном мире. Дверь ратуши встанет на грань, по которой вы кликнули. В воздухе колония встаёт в пяти блоках перед вами.
3. Ратуша поднимается сама. На складе уже есть доски, брёвна, булыжник, плиты, стекло, двери и хлеб.
4. При основании приходят четверо жителей. `/colony build builder`, затем `/colony citizens` и `/colony hire 1 builder`. Строитель кладёт по блоку за раз, пока на складе есть материал. Уровень его хижины ограничивает улучшение остальных: `/colony upgrade builder`, затем `/colony upgrade <хижина>`. Хижина не поднимается выше 5 уровня.
5. Дома дают места: ратуша на четверых, каждый дом ещё на двоих, домов четыре. `/colony build residence`. Навык жителя растёт, пока он работает, и упирается в уровень дома: без дома 10, дальше 20, 30, 40, 50 и 99.
6. Еда и материалы: `farm`, `fisher`, после исследований `lumberjack`, `miner`, `sawmill`, `bakery`. Повар в `restaurant` готовит из хлеба и рыбы. Склад и курьеры: `warehouse`, `courier`. Число курьеров — уровень склада × 2. Сдача и выдача: `/colony deposit`, `/colony withdraw planks 32`.
7. Таверна зовёт посетителей. `/colony visitors`, затем `/colony recruit 1` за 8 изумрудов со склада. Стража: `/colony hire <id> guard`, роль `/colony guard <id> knight|archer|druid`. После исследования `rally` знамя или `/colony rally` зовёт стражу к вам.
8. Университет открывает исследования `haste`, `drilling`, `scholarship`. Список: `/colony research`. Рейд сам начинается, когда жителей семеро; `/colony raid` вызывает его раньше. Без стражи колония теряет часть склада, а один житель ранен, пока нет больницы.

Язык чата и надпись справа сверху — в Options → Mods → MineColonies. По умолчанию английский; кнопка переключает на русский.

Команды: `/colony help`, `found [имя]`, `info`, `whoami`, `whereami`, `home`, `citizens`, `warehouse`, `build`, `upgrade`, `hire`, `guard`, `visitors`, `recruit`, `rally`, `deposit`, `withdraw`, `research`, `raid`, `recall`, `abandon confirm`.

Чужой игрок не ломает блоки в радиусе 64 блоков от двери ратуши. Новая колония не встаёт ближе 128 блоков к чужой.

## Сборка

Контент — папка `content/` (`horizon-mod.toml`, `assets/`, `data/`). Положите её в `horizon-mods/` или соберите пакет:

```sh
cargo install --git https://github.com/horizon-rust/horizon cargo-horizon-mod
cd content && cargo horizon-mod build
```

Текстуры заново рисует `python3 content/pack.py`.

Нативный мод — `native/`. Логика проверяется отдельно от игры:

```sh
cd native && cargo test
```

Проверка бандла — та, что описана в руководстве Horizon. `native.py` из клона [horizon-mods](https://github.com/horizon-rust/horizon-mods) ищет мод как `native/<имя>`:

```sh
git clone https://github.com/horizon-rust/horizon-mods
ln -s /path/to/this/repo/native horizon-mods/native/minecolonies
python3 horizon-mods/native/tools/native.py check minecolonies
```

В игре лаунчер собирает папку бандла:

```sh
horizon-launcher native-build /path/to/this/repo/native/mixins/minecolonies --trust-native-code
```

Мод рассчитан на Horizon с `NATIVE_HOOKS_API` не ниже 413, ревизия `c98a51a664c8ce8ec166ecedd85bb47b3687b336`. Пока правите код, в лаунчере удобнее профиль Dev build: игра собирается быстрее.

Оба мода нужны вместе. Контент даёт блоки и предметы. Нативный мод ведёт колонию, чат и защиту территории. Колония пишется в `<мир>/minecolonies/colony.json`.

## Чего в 0.1 нет

Это кусок петли основания, а не MineColonies целиком. Нет схематик и построек из NBT, запросов жителей, интерфейсов хижин, дерева исследований как в Java, счастья как отдельного экрана и настоящего набега варваров. Соответствие систем — в [PORT.md](PORT.md).

Ограничения среза:

- `/colony recall` заново вызывает жителей и разбойников. После загрузки мира старые сущности в незагруженных чанках могут остаться, пока чанк не прогрузится.
- Строитель ставит блоки только в загруженном чанке текущего измерения.
- На мирной сложности разбойники могут не появиться. Рейд всё равно считается в симуляции.
- На склад сдаются обычные стаки без чар и компонентов.
- Таймеры работы и рейда намеренно короткие, чтобы колонию было видно сразу.
