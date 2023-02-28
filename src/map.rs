use std::cmp::{max, min};
use rltk::{RandomNumberGenerator, RGB, Rltk};
use crate::rect::Rect;

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 50;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub fn xy_idx(x: i32, y: i32) -> usize {
    (y * MAP_WIDTH + x) as usize
}

pub fn draw_map(map: &[TileType], ctx: &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for tile in map.iter() {
        match tile {
            TileType::Floor => {
                ctx.set(x, y, RGB::from_f32(0.5, 0.5, 0.5), RGB::from_f32(0., 0., 0.), rltk::to_cp437('.'));
            }
            TileType::Wall => {
                ctx.set(x, y, RGB::from_f32(0.0, 0.5, 0.0), RGB::from_f32(0., 0., 0.), rltk::to_cp437('#'));
            }
        }

        x += 1;
        if x >= MAP_WIDTH {
            x = 0;
            y += 1;
        }
    }
}

pub fn new_map_rooms_and_corridors() -> (Vec<Rect>, Vec<TileType>) {
    const MAX_ROOMS : i32 = 30;
    const MIN_SIZE : i32 = 6;
    const MAX_SIZE : i32 = 10;

    let mut map = vec![TileType::Wall; 80*50];
    let mut rooms : Vec<Rect> = Vec::new();
    let mut rng = RandomNumberGenerator::new();

    for _ in 0..MAX_ROOMS {
        let w = rng.range(MIN_SIZE, MAX_SIZE);
        let h = rng.range(MIN_SIZE, MAX_SIZE);
        let x = rng.roll_dice(1, MAP_WIDTH - w - 1) - 1;
        let y = rng.roll_dice(1, MAP_HEIGHT - h - 1) - 1;
        let new_room = Rect::new(x, y, w, h);
        let mut is_room_valid = true;
        for other_room in rooms.iter() {
            if new_room.intersect(other_room) { is_room_valid = false }
        }
        if is_room_valid {
            apply_room_to_map(&new_room, &mut map);

            if !rooms.is_empty() {
                let (new_x, new_y) = new_room.center();
                let (prev_x, prev_y) = rooms[rooms.len()-1].center();
                if rng.range(0,2) == 1 {
                    apply_horizontal_tunnel(&mut map, prev_x, new_x, prev_y);
                    apply_vertical_tunnel(&mut map, prev_y, new_y, new_x);
                } else {
                    apply_vertical_tunnel(&mut map, prev_y, new_y, prev_x);
                    apply_horizontal_tunnel(&mut map, prev_x, new_x, new_y);
                }
            }

            rooms.push(new_room);
        }
    }

    (rooms, map)
}

fn apply_room_to_map(room: &Rect, map: &mut [TileType]) {
    for y in room.y1 + 1..=room.y2 {
        for x in room.x1 + 1..=room.x2 {
            map[xy_idx(x, y)] = TileType::Floor;
        }
    }
}

fn apply_horizontal_tunnel(map: &mut [TileType], x1: i32, x2: i32, y: i32) {
    for x in min(x1, x2)..=max(x1, x2) {
        let idx = xy_idx(x, y);
        if idx > 0 && idx < (MAP_WIDTH * MAP_HEIGHT) as usize {
            map[idx] = TileType::Floor;
        }
    }
}

fn apply_vertical_tunnel(map: &mut [TileType], y1: i32, y2: i32, x: i32) {
    for y in min(y1, y2)..=max(y1, y2) {
        let idx = xy_idx(x, y);
        if idx > 0 && idx < (MAP_WIDTH * MAP_HEIGHT) as usize {
            map[idx] = TileType::Floor;
        }
    }
}
