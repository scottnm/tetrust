mod block;
use crate::block::*;

extern crate arrayvec;
extern crate pancurses;
extern crate rand;

use arrayvec::ArrayVec;
use pancurses::{endwin, initscr, Window};
use rand::{thread_rng, Rng};
use std::{thread, time};

fn translate_cells(cells: &[(i32, i32); 4], (row_translation, col_translation): (i32, i32)) -> [(i32, i32); 4] {
    let mut translated_cells: [(i32, i32); 4] = *cells;
    for cell_index in 0..translated_cells.len() {
        translated_cells[cell_index].0 += row_translation;
        translated_cells[cell_index].1 += col_translation;
    }

    translated_cells
}

fn touching_floor(block: BlockType, block_pos: (i32, i32), floor_pos: i32) -> bool {
    block_pos.0 + block.height() >= floor_pos
}

fn will_collide_with_other_block(block_types: &[BlockType], block_positions: &[(i32, i32)], block_id: usize) -> bool {
    let block = block_types[block_id];
    let block_pos = block_positions[block_id];
    let block_cells = translate_cells(&block.cells(), block_pos);

    // Only need to check for collisions against blocks that were created before this block id
    // since all other blocks will always be higher up in the grid.
    for other_block_id in 0..block_id {
        let other_block = block_types[other_block_id];
        let other_block_pos = block_positions[other_block_id];
        let other_block_cells = translate_cells(&other_block.cells(), other_block_pos);

        for cell in block_cells.iter() {
            for other_cell in other_block_cells.iter() {
                if (cell.1 == other_cell.1) && (cell.0 + 1 == other_cell.0) {
                    return true;
                }
            }
        }
    }

    false
}

fn render_block(window: &Window, (row, col): (i32, i32), block_type: BlockType) {
    let sprite_char = block_type.sprite_char();
    for cell in block_type.cells().iter() {
        // Ok to blit block sprite even if position is OOB
        window.mvaddch(cell.0 + row, cell.1 + col, sprite_char);
    }
}

fn main() {
    const BLOCK_GENERATION_PERIOD: time::Duration = time::Duration::from_millis(500); // generate a new block once a second
    const BLOCK_MOVE_PERIOD: time::Duration = time::Duration::from_millis(250);
    const MAX_BLOCKS: usize = 40;
    const RUN_TIME: time::Duration = time::Duration::from_secs(20); // run long enough to generate all blocks
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
            let start_col: i32 = rng.gen_range(0, window.get_max_x());

            block_types.push(new_block_type);
            block_positions.push((-new_block_type.height(), start_col));

            last_block_generation_timestamp = time::Instant::now();
        }

        if last_move_timestamp.elapsed() >= BLOCK_MOVE_PERIOD {
            assert_eq!(block_types.len(), block_positions.len());
            for block_id in 0..block_types.len() {
                // Don't update the block position if we are already touching the floor or are
                // about to collide with another block
                if !touching_floor(block_types[block_id], block_positions[block_id], window.get_max_y() - 1) &&
                   !will_collide_with_other_block(block_types.as_slice(), block_positions.as_slice(), block_id) {
                    block_positions[block_id].0 += 1;
                }
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
