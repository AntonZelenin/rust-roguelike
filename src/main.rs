mod components;
mod map;
mod monster;
mod monster_ai_system;
mod player;
mod rect;
mod state;
mod visibility_system;

use crate::state::{RunState, State};
use crate::player::Player;
use rltk::{BaseMap, Point, RGB};
use specs::prelude::*;
use crate::components::{Name, Position, Renderable, Viewshed};
use crate::monster::Monster;

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;

    let mut gs = State {
        ecs: World::new(),
        run_state : RunState::Running,
    };
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Viewshed>();

    let map = map::Map::new_with_rooms_and_corridors();

    create_player(&mut gs, &map);
    create_monsters(&mut gs, &map);
    gs.ecs.insert(map);

    rltk::main_loop(context, gs)
}

fn create_player(gs: &mut State, map: &map::Map) {
    let (player_x, player_y) = map.rooms[0].center();
    gs.ecs
        .create_entity()
        .with(Player {})
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewshed { visible_tiles: Vec::new(), range: 8, dirty: true })
        .with(Name{name: "Player".to_string() })
        .build();
    gs.ecs.insert(Point::new(player_x, player_y));
}

fn create_monsters(gs: &mut State, map: &map::Map) {
    let mut rng = rltk::RandomNumberGenerator::new();

    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let (x,y) = room.center();

        let glyph : rltk::FontCharType;
        let name : String;
        let roll = rng.roll_dice(1, 2);
        match roll {
            1 => { glyph = rltk::to_cp437('g'); name = "Goblin".to_string(); }
            _ => { glyph = rltk::to_cp437('o'); name = "Orc".to_string(); }
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
            .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
            .with(Name{ name: format!("{} #{}", &name, i) })
            .build();
    }
}
