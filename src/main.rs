extern crate pancurses;
extern crate rand;
mod block;
mod game;
mod randwrapper;
mod tests;

use crate::block::*;
use crate::game::*;
use crate::randwrapper::*;
use std::time;

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

fn draw_frame(window: &pancurses::Window, left: i32, width: i32, top: i32, height: i32) {
    let right = left + width - 1;
    let bottom = top + height - 1;
    assert!(left < right);
    assert!(top < bottom);

    // draw corners
    window.mvaddch(top, left, pancurses::ACS_ULCORNER());
    window.mvaddch(top, right, pancurses::ACS_URCORNER());
    window.mvaddch(bottom, left, pancurses::ACS_LLCORNER());
    window.mvaddch(bottom, right, pancurses::ACS_LRCORNER());

    // draw horizontal borders
    for col in left + 1..right {
        window.mvaddch(top, col, pancurses::ACS_HLINE());
        window.mvaddch(bottom, col, pancurses::ACS_HLINE());
    }

    // draw vertical borders
    for row in top + 1..bottom {
        window.mvaddch(row, left, pancurses::ACS_VLINE());
        window.mvaddch(row, right, pancurses::ACS_VLINE());
    }
}

fn main() {
    let window = pancurses::initscr();

    const BOARD_X_OFFSET: i32 = 1;
    const BOARD_Y_OFFSET: i32 = 1;
    const BOARD_DIM_WIDTH: i32 = 10;
    const BOARD_DIM_HEIGHT: i32 = 20;

    const INPUT_POLL_PERIOD: time::Duration = time::Duration::from_millis(125);
    const DEFAULT_GAME_TICK_PERIOD: time::Duration = time::Duration::from_millis(250);
    let mut game_tick_period = DEFAULT_GAME_TICK_PERIOD;

    pancurses::noecho();
    pancurses::cbreak();
    pancurses::set_title("TETRUST");
    window.nodelay(true);
    setup_colors();

    let mut last_game_tick = time::Instant::now();
    let mut last_input_handled = time::Instant::now();

    let mut game_state = GameState::new(
        BOARD_X_OFFSET,
        BOARD_Y_OFFSET,
        BOARD_DIM_WIDTH,
        BOARD_DIM_HEIGHT,
        ThreadRangeRng::new(),
        ThreadRangeRng::new(),
    );

    let mut inputs = (false, false);

    while !game_state.is_game_over() {
        // Input handling
        if let Some(pancurses::Input::Character(ch)) = window.getch() {
            match ch {
                // check for movement inputs
                'a' => inputs.0 = true, // move left
                'd' => inputs.1 = true, // move right

                // debug
                'q' => break,                                       // kill game early
                'z' => game_tick_period *= 2,                       // slowdown tick rate
                'x' => game_tick_period = DEFAULT_GAME_TICK_PERIOD, // reset tick rate
                'c' => game_tick_period /= 2,                       // speed up tick rate
                _ => (),
            }
        }

        if last_input_handled.elapsed() >= INPUT_POLL_PERIOD {
            last_input_handled = time::Instant::now();
            let mut horizontal_motion: i32 = 0;
            if inputs.0 {
                horizontal_motion -= 1;
            }
            if inputs.1 {
                horizontal_motion += 1;
            }
            game_state.move_block_horizontal(horizontal_motion);
            inputs = (false, false);
        }

        // Tick the game state
        if last_game_tick.elapsed() >= game_tick_period {
            last_game_tick = time::Instant::now();
            game_state.tick();
        }

        // Render the frame
        window.erase();

        draw_frame(
            &window,
            BOARD_X_OFFSET - 1,
            BOARD_DIM_WIDTH + 2,
            BOARD_Y_OFFSET - 1,
            BOARD_DIM_HEIGHT + 2,
        );

        for block_id in 0..game_state.block_count() {
            let (position, block) = game_state.block(block_id);
            render_block(&window, position, block);
        }
        window.refresh();
    }

    pancurses::endwin();

    println!("Lose!");
    println!("Finished");
}
