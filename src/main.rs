mod components;
mod game_log;
mod gui;
mod map;
mod menu;
mod player;
mod random_table;
mod rect;
mod spawner;
mod state;
mod visibility_system;
mod systems;

use rltk::Point;
use crate::components::*;
use crate::state::{RunState, State};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};
use crate::systems::particle;

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

    // it should be inserted earlier than the rest, otherwise it will crash
    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    let map = map::Map::new_with_rooms_and_corridors(1);

    let (player_x, player_y) = map.rooms[0].center();
    let player = spawner::create_player(&mut gs.ecs, player_x, player_y);
    gs.ecs.insert(Point::new(player_x, player_y));

    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room, &map, 1);
    }
    gs.ecs.insert(map);
    gs.ecs.insert(player);
    gs.ecs.insert(RunState::PreRun);
    gs.ecs.insert(game_log::GameLog { entries: vec!["Welcome to Rusty Roguelike".to_string()] });
    gs.ecs.insert(particle::ParticleBuilder::new());

    rltk::main_loop(context, gs)
}

fn register_components(gs: &mut State) {
    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<Confusion>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<DefenseBonus>();
    gs.ecs.register::<Equipped>();
    gs.ecs.register::<Equippable>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<MeleePowerBonus>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<ParticleLifetime>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Position>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToRemoveItem>();

    gs.ecs.register::<SerializationHelper>();
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
}
