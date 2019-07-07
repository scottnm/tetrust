mod block;
use crate::block::*;

extern crate arrayvec;
extern crate pancurses;
extern crate rand;

use arrayvec::ArrayVec;
use pancurses::{endwin, initscr, Window};
use rand::{thread_rng, Rng};
use std::{thread, time};

fn in_bounds(window: &Window, row: i32, col: i32) -> bool {
    row >= 0 && row < window.get_max_y() && col >= 0 && col < window.get_max_x()
}

fn render_block(window: &Window, (row, col): (i32, i32), block_type: BlockType) {
    let block_char = block_type.block_char();
    let block_cells = block_type.block_cells();
    for cell in block_cells.iter() {
        if in_bounds(window, cell.0, cell.1) {
            window.mvaddch(cell.0 + row, cell.1 + col, block_char);
        }
    }
}

fn main() {
    const BLOCK_GENERATION_PERIOD: time::Duration = time::Duration::from_millis(500); // generate a new block once a second
    const BLOCK_MOVE_PERIOD: time::Duration = time::Duration::from_millis(250);
    const MAX_BLOCKS: usize = 20;
    const RUN_TIME: time::Duration = time::Duration::from_secs(10); // run long enough to generate all blocks
    const RENDER_REFRESH_PERIOD: time::Duration = time::Duration::from_millis(16); // 60 fps

    let window = initscr();
    let mut rng = thread_rng();

    let start_time = time::Instant::now();
    let mut last_block_generation_timestamp = time::Instant::now();
    let mut last_move_timestamp = time::Instant::now();

    let mut block_types = ArrayVec::<[BlockType; MAX_BLOCKS]>::new();
    let mut block_positions = ArrayVec::<[(i32, i32); MAX_BLOCKS]>::new();

    while start_time.elapsed() < RUN_TIME {
        //
        // Game logic:
        // - generate a new block periodically
        // - move every block periodically
        //
        if last_block_generation_timestamp.elapsed() >= BLOCK_GENERATION_PERIOD {
            assert!(!block_types.is_full());
            assert!(!block_positions.is_full());

            let new_block_type = BlockType::random(&mut rng);
            let start_row: i32 = rng.gen_range(0, window.get_max_y());
            let start_col: i32 = rng.gen_range(0, window.get_max_x());

            block_types.push(new_block_type);
            block_positions.push((start_row, start_col));

            last_block_generation_timestamp = time::Instant::now();
        }

        if last_move_timestamp.elapsed() >= BLOCK_MOVE_PERIOD {
            for pos in &mut block_positions {
                pos.0 += 1;
            }
            last_move_timestamp = time::Instant::now();
        }

        //
        // Render the frame
        //
        window.erase();

        assert_eq!(block_types.len(), block_positions.len());
        for block_id in 0..block_types.len() {
            render_block(&window, block_positions[block_id], block_types[block_id]);
        }

        window.refresh();
        thread::sleep(RENDER_REFRESH_PERIOD);
    }

    endwin();
}
