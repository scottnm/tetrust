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

fn main() {
    let window = pancurses::initscr();

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
        window.get_max_x(),
        window.get_max_y(),
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
                'q' => break, // kill game early
                'z' => game_tick_period *= 2, // slowdown tick rate
                'x' => game_tick_period = DEFAULT_GAME_TICK_PERIOD, // reset tick rate
                'c' => game_tick_period /= 2, // speed up tick rate
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
