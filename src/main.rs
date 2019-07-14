mod block;
extern crate pancurses;
extern crate rand;

use crate::block::*;
use rand::Rng;
use std::time;

fn translate_cells(cells: &[Cell; 4], row_translation: i32, col_translation: i32) -> [Cell; 4] {
    let mut translated_cells: [Cell; 4] = *cells;
    for cell_index in 0..translated_cells.len() {
        translated_cells[cell_index].0 += row_translation;
        translated_cells[cell_index].1 += col_translation;
    }

    translated_cells
}

fn is_resting_on_floor(block: BlockType, block_pos: Cell, floor_pos: i32) -> bool {
    block_pos.0 + block.height() >= floor_pos
}

fn is_resting_on_other_block(
    blocks: &[BlockType],
    block_positions: &[Cell],
    block_id: usize,
) -> bool {
    let block = blocks[block_id];
    let block_pos = block_positions[block_id];
    let block_cells = translate_cells(&block.cells(), block_pos.0, block_pos.1);

    // Only need to check for collisions against blocks that were created before this block id
    // since all other blocks will always be higher up in the grid.
    for other_block_id in 0..block_id {
        let other_block = blocks[other_block_id];
        let other_block_pos = block_positions[other_block_id];
        let other_block_cells =
            translate_cells(&other_block.cells(), other_block_pos.0, other_block_pos.1);

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

fn render_block(window: &pancurses::Window, Cell(row, col): Cell, block: BlockType) {
    let sprite_char = block.sprite_char();
    let color_pair = pancurses::COLOR_PAIR(block as pancurses::chtype);
    window.attron(color_pair);
    for cell in block.cells().iter() {
        // Ok to blit block sprite even if position is OOB
        window.mvaddch(cell.0 + row, cell.1 + col, sprite_char);
    }
    window.attroff(color_pair);
}

fn setup_colors() {
    pancurses::start_color();

    assert!(BLOCKTYPES.len() < pancurses::COLOR_PAIRS() as usize);
    for block in BLOCKTYPES.iter() {
        pancurses::init_pair(*block as i16, pancurses::COLOR_BLACK, block.sprite_color());
    }
}

fn main() {
    let mut rng = rand::thread_rng();
    let window = pancurses::initscr();

    const DEFAULT_BLOCK_MOVE_PERIOD: time::Duration = time::Duration::from_millis(250);
    let mut block_move_period = DEFAULT_BLOCK_MOVE_PERIOD;

    pancurses::noecho();
    pancurses::cbreak();
    pancurses::set_title("TETRUST");
    window.nodelay(true);
    setup_colors();

    let mut last_move_timestamp = time::Instant::now();
    let mut generate_block = true;

    let max_blocks = (window.get_max_x() * window.get_max_y()) as usize;
    let mut block_count = 0;
    let mut blocks = vec![BlockType::I; max_blocks];
    let mut block_positions = vec![Cell(0, 0); max_blocks];

    loop {
        //
        // Input handling:
        // - A -> slowdown time
        // - S -> reset time
        // - D -> speed up time
        //
        if let Some(pancurses::Input::Character(ch)) = window.getch() {
            match ch {
                'a' => block_move_period *= 2,
                's' => block_move_period = DEFAULT_BLOCK_MOVE_PERIOD,
                'd' => block_move_period /= 2,
                _ => (),
            }
        }

        //
        // Game logic:
        // - generate a new block whenever one is not already falling
        // - move every falling block periodically
        //
        if generate_block {
            generate_block = false;

            assert_eq!(blocks.len(), block_positions.len());
            assert!(block_count < blocks.len());

            let new_block = BlockType::random(&mut rng);
            let start_col: i32 = rng.gen_range(0, window.get_max_x() - new_block.width());

            blocks[block_count] = new_block;
            block_positions[block_count] = Cell(-new_block.height(), start_col);
            block_count += 1;
        }

        if last_move_timestamp.elapsed() >= block_move_period {
            last_move_timestamp = time::Instant::now();
            let moving_block_id = block_count - 1; // we are always moving the last block

            assert_eq!(blocks.len(), block_positions.len());
            let block_has_landed = is_resting_on_floor(
                blocks[moving_block_id],
                block_positions[moving_block_id],
                window.get_max_y(),
            ) || is_resting_on_other_block(
                &blocks[..block_count],
                &block_positions[..block_count],
                moving_block_id,
            );

            if block_has_landed {
                assert!(!generate_block); // we should have already consumed generate_block at this point
                generate_block = true;

                // if the last block was placed above the board, the game is over
                if block_positions[moving_block_id].0 < 0 {
                    println!("Lose!");
                    break;
                }
            } else {
                block_positions[moving_block_id].0 += 1;
            }
        }

        //
        // Render the frame
        //
        window.erase();

        assert_eq!(blocks.len(), block_positions.len());
        for block_id in 0..block_count {
            render_block(&window, block_positions[block_id], blocks[block_id]);
        }

        window.refresh();
    }

    pancurses::endwin();
    println!("Finished");
}
