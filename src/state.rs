use crate::{game_log, gui, map, player, spawner, systems};
use crate::components::{Position, Ranged, Renderable, WantsToUseItem, WantsToDropItem, Player, InBackpack, Viewshed, CombatStats, Equipped};
use crate::map::Map;
use crate::menu::main_menu;
use crate::systems::damage::DamageSystem;
use crate::systems::inventory::{ItemCollectionSystem, ItemDropSystem, ItemUseSystem};
use crate::systems::map_indexing::MapIndexingSystem;
use crate::systems::melee_combat::MeleeCombatSystem;
use crate::systems::monster_ai::MonsterAI;
use crate::visibility_system::VisibilitySystem;

use rltk::{GameState, Point, Rltk};
use specs::prelude::*;

pub struct State {
    pub ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);

        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);

        let mut map_index = MapIndexingSystem {};
        map_index.run_now(&self.ecs);

        let mut melee = MeleeCombatSystem {};
        melee.run_now(&self.ecs);

        let mut damage = DamageSystem {};
        damage.run_now(&self.ecs);
        systems::damage::delete_the_dead(&mut self.ecs);

        let mut pickup = ItemCollectionSystem {};
        pickup.run_now(&self.ecs);

        let mut potions = ItemUseSystem {};
        potions.run_now(&self.ecs);

        let mut drop_items = ItemDropSystem {};
        drop_items.run_now(&self.ecs);

        self.ecs.maintain();
    }

    fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
        let entities = self.ecs.entities();
        let player = self.ecs.read_storage::<Player>();
        let backpack = self.ecs.read_storage::<InBackpack>();
        let player_entity = self.ecs.fetch::<Entity>();
        let equipped = self.ecs.read_storage::<Equipped>();

        let mut to_delete: Vec<Entity> = Vec::new();
        for entity in entities.join() {
            let mut should_delete = true;

            // Don't delete the player
            let p = player.get(entity);
            if let Some(_p) = p {
                should_delete = false;
            }

            // Don't delete the player's equipment
            let bp = backpack.get(entity);
            if let Some(bp) = bp {
                if bp.owner == *player_entity {
                    should_delete = false;
                }
            }

            let eq = equipped.get(entity);
            if let Some(eq) = eq {
                if eq.owner == *player_entity {
                    should_delete = false;
                }
            }

            if should_delete {
                to_delete.push(entity);
            }
        }

        to_delete
    }

    fn goto_next_level(&mut self) {
        // Delete entities that aren't the player or his/her equipment
        let to_delete = self.entities_to_remove_on_level_change();
        for target in to_delete {
            self.ecs.delete_entity(target).expect("Unable to delete entity");
        }

        // Build a new map and place the player
        let world_map;
        let current_depth;
        {
            let mut world_map_resource = self.ecs.write_resource::<Map>();
            current_depth = world_map_resource.depth;
            *world_map_resource = Map::new_with_rooms_and_corridors(current_depth + 1);
            world_map = world_map_resource.clone();
        }

        // Spawn bad guys
        for room in world_map.rooms.iter().skip(1) {
            spawner::spawn_room(&mut self.ecs, room, &world_map, current_depth);
        }

        // Place the player and update resources
        let (player_x, player_y) = world_map.rooms[0].center();
        let mut player_position = self.ecs.write_resource::<Point>();
        *player_position = Point::new(player_x, player_y);
        let mut position_components = self.ecs.write_storage::<Position>();
        let player_entity = self.ecs.fetch::<Entity>();
        let player_pos_comp = position_components.get_mut(*player_entity);
        if let Some(player_pos_comp) = player_pos_comp {
            player_pos_comp.x = player_x;
            player_pos_comp.y = player_y;
        }

        // Mark the player's visibility as dirty
        let mut viewshed_components = self.ecs.write_storage::<Viewshed>();
        let vs = viewshed_components.get_mut(*player_entity);
        if let Some(vs) = vs {
            vs.dirty = true;
        }

        // Notify the player and give them some health
        let mut game_log = self.ecs.fetch_mut::<game_log::GameLog>();
        game_log.entries.push("You descend to the next level, and take a moment to heal.".to_string());
        let mut player_health_store = self.ecs.write_storage::<CombatStats>();
        let player_health = player_health_store.get_mut(*player_entity);
        if let Some(player_health) = player_health {
            player_health.hp = i32::max(player_health.hp, player_health.max_hp / 2);
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        let mut new_run_state;
        {
            let run_state = self.ecs.fetch::<RunState>();
            new_run_state = *run_state;
        }

        ctx.cls();

        match new_run_state {
            RunState::MainMenu { .. } => {}
            _ => {
                map::draw_map(&self.ecs, ctx);
                {
                    let positions = self.ecs.read_storage::<Position>();
                    let renderables = self.ecs.read_storage::<Renderable>();
                    let map = self.ecs.fetch::<Map>();

                    let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
                    data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
                    for (pos, render) in data.iter() {
                        let idx = map.xy_idx(pos.x, pos.y);
                        if map.visible_tiles[idx] { ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph) }
                    }

                    gui::draw_ui(&self.ecs, ctx);
                }
            }
        }

        match new_run_state {
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                new_run_state = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                new_run_state = player::read_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                self.ecs.maintain();
                new_run_state = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                self.ecs.maintain();
                new_run_state = RunState::AwaitingInput;
            }
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let is_ranged = self.ecs.read_storage::<Ranged>();
                        let is_item_ranged = is_ranged.get(item_entity);
                        if let Some(is_item_ranged) = is_item_ranged {
                            new_run_state = RunState::ShowTargeting { range: is_item_ranged.range, item: item_entity };
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent
                                .insert(*self.ecs.fetch::<Entity>(), WantsToUseItem { item: item_entity, target: None })
                                .expect("Unable to insert intent");
                            new_run_state = RunState::PlayerTurn;
                        }
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent
                            .insert(*self.ecs.fetch::<Entity>(), WantsToDropItem { item: item_entity })
                            .expect("Unable to insert intent");
                        new_run_state = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowTargeting { range, item } => {
                let result = gui::ranged_target(self, ctx, range);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        intent
                            .insert(*self.ecs.fetch::<Entity>(), WantsToUseItem { item, target: result.1 })
                            .expect("Unable to insert intent");
                        new_run_state = RunState::PlayerTurn;
                    }
                }
            }
            RunState::MainMenu { .. } => {
                let result = main_menu(self, ctx);
                match result {
                    gui::MainMenuResult::NoSelection { selected } => new_run_state = RunState::MainMenu { menu_selection: selected },
                    gui::MainMenuResult::Selected { selected } => {
                        match selected {
                            gui::MainMenuSelection::NewGame => new_run_state = RunState::PreRun,
                            gui::MainMenuSelection::LoadGame => {
                                systems::save_load::load_game(&mut self.ecs);
                                new_run_state = RunState::AwaitingInput;
                                systems::save_load::delete_save();
                            }
                            gui::MainMenuSelection::Quit => { std::process::exit(0); }
                        }
                    }
                }
            }
            RunState::SaveGame => {
                systems::save_load::save_game(&mut self.ecs);
                new_run_state = RunState::MainMenu { menu_selection: gui::MainMenuSelection::LoadGame };
            }
            RunState::NextLevel => {
                self.goto_next_level();
                new_run_state = RunState::PreRun;
            }
        }

        // todo why is it here? It's not used?
        {
            let mut run_writer = self.ecs.write_resource::<RunState>();
            *run_writer = new_run_state;
        }

        systems::damage::delete_the_dead(&mut self.ecs);
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    MonsterTurn,
    PlayerTurn,
    PreRun,
    ShowDropItem,
    ShowInventory,
    ShowTargeting { range: i32, item: Entity },
    MainMenu { menu_selection: gui::MainMenuSelection },
    SaveGame,
    NextLevel,
}
