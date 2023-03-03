mod components;
mod damage_system;
mod game_log;
mod gui;
mod map;
mod map_indexing_system;
mod melee_combat_system;
mod monster;
mod monster_ai_system;
mod player;
mod rect;
mod state;
mod visibility_system;

use crate::state::{RunState, State};
use crate::player::Player;
use rltk::{Point, RGB};
use specs::prelude::*;
use crate::components::{BlocksTile, CombatStats, Name, Position, Renderable, SufferDamage, Viewshed, WantsToMelee};
use crate::monster::Monster;

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let mut context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;
    context.with_post_scanlines(true);

    let mut gs = State {
        ecs: World::new(),
    };
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<WantsToMelee>();

    let map = map::Map::new_with_rooms_and_corridors();

    let player = create_player(&mut gs, &map);
    create_monsters(&mut gs, &map);
    gs.ecs.insert(map);
    gs.ecs.insert(player);
    gs.ecs.insert(RunState::PreRun);
    gs.ecs.insert(game_log::GameLog { entries: vec!["Welcome to Rusty Roguelike".to_string()] });

    rltk::main_loop(context, gs)
}

fn create_player(gs: &mut State, map: &map::Map) -> Entity {
    let (player_x, player_y) = map.rooms[0].center();
    let player_entity = gs.ecs
        .create_entity()
        .with(Player {})
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewshed { visible_tiles: Vec::new(), range: 8, dirty: true })
        .with(Name { name: "Player".to_string() })
        .with(CombatStats { max_hp: 30, hp: 30, defense: 2, power: 5 })
        .build();
    gs.ecs.insert(Point::new(player_x, player_y));
    return player_entity;
}

fn create_monsters(gs: &mut State, map: &map::Map) {
    let mut rng = rltk::RandomNumberGenerator::new();

    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let (x, y) = room.center();

        let glyph: rltk::FontCharType;
        let name: String;
        let roll = rng.roll_dice(1, 2);
        match roll {
            1 => {
                glyph = rltk::to_cp437('g');
                name = "Goblin".to_string();
            }
            _ => {
                glyph = rltk::to_cp437('o');
                name = "Orc".to_string();
            }
        }

        gs.ecs
            .create_entity()
            .with(Monster {})
            .with(Position { x, y })
            .with(Renderable {
                glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed { visible_tiles: Vec::new(), range: 8, dirty: true })
            .with(Name { name: format!("{} #{}", &name, i) })
            .with(BlocksTile {})
            .with(CombatStats { max_hp: 16, hp: 16, defense: 1, power: 4 })
            .build();
    }
}
