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

fn render_block(
    window: &pancurses::Window,
    Cell { x: col, y: row }: Cell,
    block: BlockType,
    rot: Rotation,
) {
    let sprite_char = block.sprite_char();
    let color_pair = pancurses::COLOR_PAIR(block as pancurses::chtype);
    window.attron(color_pair);
    for cell in block.cells(rot).iter() {
        // Ok to blit block sprite even if position is OOB
        window.mvaddch(cell.y + row, cell.x + col, sprite_char);
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

fn draw_text_centered<S>(window: &pancurses::Window, text: S, x_center: i32, y_center: i32)
where
    S: AsRef<str>,
{
    window.mvaddstr(y_center, x_center - (text.as_ref().len() / 2) as i32, text);
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
    pancurses::curs_set(0);
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
    );

    struct Inputs {
        move_left: bool,
        move_right: bool,
        rot_left: bool,
        rot_right: bool,
    }

    let mut inputs = Inputs {
        move_left: false,
        move_right: false,
        rot_left: false,
        rot_right: false,
    };

    let mut game_over_blit_timer = Option::<time::Instant>::None;

    loop {
        // Input handling
        if let Some(pancurses::Input::Character(ch)) = window.getch() {
            match ch {
                // check for movement inputs
                'a' => inputs.move_left = true,
                'd' => inputs.move_right = true,
                'q' => inputs.rot_left = true,
                'e' => inputs.rot_right = true,

                // debug
                'p' => break,                                       // kill game early
                'z' => game_tick_period *= 2,                       // slowdown tick rate
                'x' => game_tick_period = DEFAULT_GAME_TICK_PERIOD, // reset tick rate
                'c' => game_tick_period /= 2,                       // speed up tick rate
                _ => (),
            }
        }

        if last_input_handled.elapsed() >= INPUT_POLL_PERIOD {
            last_input_handled = time::Instant::now();
            let mut horizontal_motion: i32 = 0;
            if inputs.move_left {
                horizontal_motion -= 1;
            }
            if inputs.move_right {
                horizontal_motion += 1;
            }
            game_state.move_block_horizontal(horizontal_motion);

            let mut relative_rotation: i32 = 0;
            if inputs.rot_left {
                relative_rotation -= 1;
            }
            if inputs.rot_right {
                relative_rotation += 1;
            }
            game_state.rotate_block(relative_rotation);

            inputs = Inputs {
                move_left: false,
                move_right: false,
                rot_left: false,
                rot_right: false,
            };
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
            let (position, block, rot) = game_state.block(block_id);
            render_block(&window, position, block, rot);
        }

        if game_state.is_game_over() {
            const GAME_OVER_DURATION: time::Duration = time::Duration::from_secs(3);
            match game_over_blit_timer {
                None => game_over_blit_timer = Some(time::Instant::now()),
                Some(timer) => {
                    if timer.elapsed() > GAME_OVER_DURATION {
                        break;
                    }
                }
            }

            const GAME_OVER_TEXT_X_CENTER: i32 = BOARD_X_OFFSET + BOARD_DIM_WIDTH / 2;
            const GAME_OVER_TEXT_Y_CENTER: i32 = BOARD_Y_OFFSET + BOARD_DIM_HEIGHT / 2;

            window.attron(pancurses::A_BLINK);
            draw_text_centered(
                &window,
                "Game Over",
                GAME_OVER_TEXT_X_CENTER,
                GAME_OVER_TEXT_Y_CENTER,
            );
            window.attroff(pancurses::A_BLINK);
        }

        window.refresh();
    }

    pancurses::endwin();

    println!("Lose!");
    println!("Finished");
}
