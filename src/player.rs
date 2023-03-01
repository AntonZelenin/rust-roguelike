use crate::components::Position;
use crate::state::State;

use rltk::{Rltk, VirtualKeyCode};
use specs::prelude::*;
use specs_derive::Component;
use std::cmp::{max, min};
use crate::map;
use crate::map::Map;

#[derive(Component, Debug)]
pub struct Player {}

fn try_move(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Map>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        if map.tiles[destination_idx] != map::TileType::Wall {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
        }
    }
}

pub fn read_input(gs: &mut State, ctx: &mut Rltk) {
    match ctx.key {
        None => {} // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left |
            VirtualKeyCode::A |
            VirtualKeyCode::Numpad4 |
            VirtualKeyCode::H => try_move(-1, 0, &mut gs.ecs),

            VirtualKeyCode::Right |
            VirtualKeyCode::D |
            VirtualKeyCode::Numpad6 |
            VirtualKeyCode::L => try_move(1, 0, &mut gs.ecs),

            VirtualKeyCode::Up |
            VirtualKeyCode::W |
            VirtualKeyCode::Numpad8 |
            VirtualKeyCode::K => try_move(0, -1, &mut gs.ecs),

            VirtualKeyCode::Down |
            VirtualKeyCode::S |
            VirtualKeyCode::Numpad2 |
            VirtualKeyCode::J => try_move(0, 1, &mut gs.ecs),

            _ => {}
        },
    }
}