mod components;
mod game_log;
mod gui;
mod map;
mod monster;
mod player;
mod rect;
mod spawner;
mod state;
mod visibility_system;
mod systems;

use crate::state::{RunState, State};
use crate::player::Player;
use specs::prelude::*;
use crate::components::{BlocksTile, CombatStats, InBackpack, Item, Name, Position, Potion, Renderable, SufferDamage, Viewshed, WantsToDrinkPotion, WantsToDropItem, WantsToMelee, WantsToPickupItem};
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

    register_components(&mut gs);

    let map = map::Map::new_with_rooms_and_corridors();
    let player = spawner::create_player(&mut gs, &map);
    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room, &map);
    }
    gs.ecs.insert(map);
    gs.ecs.insert(player);
    gs.ecs.insert(RunState::PreRun);
    gs.ecs.insert(game_log::GameLog { entries: vec!["Welcome to Rusty Roguelike".to_string()] });

    rltk::main_loop(context, gs)
}

fn register_components(gs: &mut State) {
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Position>();
    gs.ecs.register::<Potion>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<WantsToDrinkPotion>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<WantsToPickupItem>();
}
