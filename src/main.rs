use device_query::{DeviceQuery, DeviceState, Keycode, MouseState};
use std::ops::Rem;
use std::{fs, isize};

#[cfg(windows)]
extern crate winapi;

#[cfg(windows)]
#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("C:\\Users\\Jaime\\Desktop\\Proyects\\handly_background\\cpp\\tri_ploter.cpp");

        fn draw_tri(
            x1: i32,
            y1: i32,
            x2: i32,
            y2: i32,
            x3: i32,
            y3: i32,
            maxX: i32,
            maxY: i32,
            r: i32,
            g: i32,
            b: i32,
        );
    }
}

use std::time::Instant;

const WORLD_W: isize = 160;
const WORLD_H: isize = 90;

const GOAL_FPS: f32 = 10.;
const BULK_FRAMES: usize = 40;

const SCREEN_W: f32 = 1535.;
const SCREEN_H: f32 = 863.;

#[derive(Debug, Clone)]
struct World {
    board: Vec<Vec<bool>>,
    unstable: Vec<(isize, isize)>,
}

impl World {
    fn new(width: isize, height: isize) -> Self {
        World {
            board: (0..height)
                .map(|_| (0..width).map(|_| false).collect())
                .collect(),
            unstable: Vec::new(),
        }
    }
    fn get(&self, x: isize, y: isize) -> bool {
        return self.board[((y + WORLD_H as isize).rem(WORLD_H as isize)) as usize]
            [((x + WORLD_W as isize).rem(WORLD_W as isize)) as usize];
    }

    fn set(&mut self, x: isize, y: isize, val: bool) {
        self.board[((y + 10 * WORLD_H as isize).rem(WORLD_H as isize)) as usize]
            [((x + WORLD_W as isize).rem(WORLD_W as isize)) as usize] = val;
    }
}

fn main() {
    let mut full_write_band_size: isize = 3;
    let mut full_write_start: isize = 0;
    let mut wait: f32 = 1000. / GOAL_FPS;
    let mut world = load_world("load_data.txt".to_string(), WORLD_H, WORLD_W);
    raster(&World::new(WORLD_W, WORLD_H), &world, true);

    loop {
        let mut tick_time = 0.;
        let mut raster_time = 0.;

        for _ in 0..BULK_FRAMES {
            //print_board(&world);

            let chrono = Instant::now();
            let new_world = tick(&world);
            tick_time += chrono.elapsed().as_millis() as f32;

            let chrono = Instant::now();
            raster(&world, &new_world, false);
            raster_time += chrono.elapsed().as_millis() as f32;

            world = new_world;

            raster_full_band(
                &world.board,
                full_write_start,
                full_write_start + full_write_band_size,
            );
            full_write_start = full_write_start + full_write_band_size;
            if full_write_start > WORLD_H {
                full_write_start = 0;
            }

            let device_state = DeviceState::new();

            let (x, y) = device_state.get_mouse().coords;

            let center_x = WORLD_W as f32 * (x as f32 / SCREEN_W);
            let center_y = WORLD_H as f32 * (y as f32 / SCREEN_H);

            let impact_size = 4;

            for dx in 0..impact_size {
                for dy in 0..impact_size {
                    world.set(center_x as isize + dx, center_y as isize + dy, true);
                    world
                        .unstable
                        .push((center_x as isize + dx, center_y as isize + dy))
                }
            }

            std::thread::sleep(std::time::Duration::from_micros((wait * 1000.) as u64));
        }

        {
            let time = tick_time + raster_time + (wait * BULK_FRAMES as f32);
            let time_per_frame = time / BULK_FRAMES as f32;
            let fps = 1000. / time_per_frame;

            let goal_time_per_frame = 1000. / GOAL_FPS;
            wait = goal_time_per_frame - (time_per_frame - wait);

            println!("fps: {}", fps);
            // println!(
            //     "  time_per_frame: {}",
            //     (tick_time + raster_time) / BULK_FRAMES as f32
            // );
            // println!("    tick_time: {}", tick_time / BULK_FRAMES as f32);
            // println!("    raster_time: {}", raster_time / BULK_FRAMES as f32);
            // println!("  goal_time_per_frame: {}", goal_time_per_frame);
            // println!("  wait: {}", wait);

            if wait < 0. {
                println!(" ----------------------------- ");
                println!("| unrealistic fps spectations |");
                println!(" ----------------------------- ");
                wait = 0.;
            }
        }
    }
}

fn load_world(path: String, desired_h: isize, desired_w: isize) -> World {
    let mut ret = World::new(desired_w, desired_h);

    let mut w: isize = 0;
    let mut h: isize = 1;

    let file_string = fs::read_to_string(path).expect("no such load file");
    let mut file_structure = Vec::new();

    let mut row = Vec::new();
    for c in file_string.chars() {
        if c == '\n' {
            (&mut file_structure).push(row.clone());
            row = Vec::new();
            h += 1;
        }
        if c == '#' {
            row.push(true);
            w = w.max(row.len() as isize);
        }
        if c == ' ' {
            row.push(false);
            w = w.max(row.len() as isize);
        }
    }

    let structure_x: isize = ((desired_w - w) / 2) as isize;
    let structure_y: isize = ((desired_h - h) / 2) as isize;

    for writting_x in 0..desired_w {
        for writting_y in 0..desired_h {
            ret.set(
                writting_x as isize,
                writting_y as isize,
                querry_structure(
                    writting_x as isize - structure_x,
                    writting_y as isize - structure_y,
                    &file_structure,
                    false,
                ),
            );

            ret.unstable.push((writting_x, writting_y));
        }
    }

    ret
}

fn querry_structure<T: Clone>(x: isize, y: isize, structure: &Vec<Vec<T>>, oob: T) -> T {
    if x < 0 || y < 0 {
        return oob;
    }
    if y >= structure.len() as isize {
        return oob;
    }
    if x >= structure[y as usize].len() as isize {
        return oob;
    }
    return structure[y as usize][x as usize].clone();
}

const ARROUND_DELTAS: [(isize, isize); 8] = [
    (-1, -1),
    (0, 1),
    (1, 1),
    (1, 0),
    (1, -1),
    (0, -1),
    (-1, 0),
    (-1, 1),
];

fn tick(old_world: &World) -> World {
    let mut ret = World::new(WORLD_W, WORLD_H);
    ret.board = old_world.board.clone();
    ret.unstable = Vec::new();

    for &(x, y) in old_world.unstable.iter() {
        // Any live cell with two or three live neighbours survives.
        // Any dead cell with three live neighbours becomes a live cell.
        // All other live cells die in the next generation. Similarly, all other dead cells stay dead.

        let cell = old_world.get(x as isize, y as isize);
        let neighbours = ARROUND_DELTAS.iter().fold(0, |acc, &(dx, dy)| {
            acc + if old_world.get(x as isize + dx, y as isize + dy) {
                1
            } else {
                0
            }
        });

        if cell && neighbours == 2 || neighbours == 3 {
            ret.set(x as isize, y as isize, true);
        } else if !cell && neighbours == 3 {
            ret.set(x as isize, y as isize, true);
        } else if cell {
            ret.set(x as isize, y as isize, false);
        }

        if ret.get(x as isize, y as isize) != old_world.get(x as isize, y as isize) {
            ret.unstable
                .push(((x + WORLD_W).rem(WORLD_W), (y + WORLD_H).rem(WORLD_H)));

            for (dx, dy) in ARROUND_DELTAS {
                ret.unstable.push((
                    ((x + WORLD_W) as isize + dx).rem(WORLD_W),
                    ((y + WORLD_H) as isize + dy).rem(WORLD_H),
                ));
            }
        }
    }

    ret.unstable.sort();
    ret.unstable.dedup();

    ret
}

fn draw_cell(x: isize, y: isize, val: bool) {
    let c = if val { 255 } else { 0 };
    ffi::draw_tri(
        x as i32,
        y as i32,
        x as i32 + 1,
        y as i32,
        x as i32,
        y as i32 + 1,
        WORLD_W as i32,
        WORLD_H as i32,
        c,
        c,
        c,
    );
    ffi::draw_tri(
        1 + x as i32,
        1 + y as i32,
        x as i32 + 1,
        y as i32,
        x as i32,
        y as i32 + 1,
        WORLD_W as i32,
        WORLD_H as i32,
        c,
        c,
        c,
    );
}

fn raster(old_world: &World, new_world: &World, force_full: bool) {
    for x in 0..WORLD_W {
        for y in 0..WORLD_H {
            if old_world.get(x, y) != new_world.get(x, y) || force_full {
                draw_cell(x, y, new_world.get(x, y));
            }
        }
    }
}

fn raster_full_band(board: &Vec<Vec<bool>>, y0: isize, yf: isize) {
    for (y, row) in board.iter().enumerate() {
        if (y as isize) >= y0 && (y as isize) < yf {
            for (x, &cell) in row.iter().enumerate() {
                draw_cell(x as isize, y as isize, board[y][x]);
            }
        }
    }
}

fn print_board(w: &World) {
    for row in &w.board {
        for &cell in row {
            print!("{}", if cell { "#" } else { " " })
        }
        println!("");
    }
    println!("unstable: {:?}", w.unstable);
}
