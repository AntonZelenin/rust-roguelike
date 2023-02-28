mod map;
mod player;
mod components;
mod state;
mod rect;

use crate::state::State;
use crate::player::Player;
use rltk::RGB;
use specs::prelude::*;
use crate::components::{Position, Renderable};

// todo maybe resources directory is not needed
fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;

    let mut gs = State {
        ecs: World::new(),
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();

    let (rooms, map) = map::new_map_rooms_and_corridors();
    let (player_x, player_y) = rooms[0].center();

    gs.ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .build();
    gs.ecs.insert(map);

    rltk::main_loop(context, gs)
}
