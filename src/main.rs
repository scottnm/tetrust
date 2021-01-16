extern crate pancurses;
extern crate rand;
mod block;
mod game;
mod randwrapper;
mod tests;
mod util;

use crate::block::*;
use crate::game::*;
use crate::randwrapper::*;
use crate::util::*;
use std::time;

fn render_cell(
    window: &pancurses::Window,
    cell_rel_pos: Vec2,
    rel_pos_offset_x: i32,
    rel_pos_offset_y: i32,
    block_type: BlockType,
) {
    let sprite_char = block_type.sprite_char();
    let color_pair = pancurses::COLOR_PAIR(block_type as pancurses::chtype);
    window.attron(color_pair);
    window.mvaddch(
        cell_rel_pos.y + rel_pos_offset_y,
        cell_rel_pos.x + rel_pos_offset_x,
        sprite_char,
    );
    window.attroff(color_pair);
}

fn render_block(
    window: &pancurses::Window,
    block_rel_pos: Vec2,
    rel_pos_offset_x: i32,
    rel_pos_offset_y: i32,
    block: Block,
) {
    let sprite_char = block.sprite_char();
    let color_pair = pancurses::COLOR_PAIR(block.block_type as pancurses::chtype);
    window.attron(color_pair);
    for cell_pos in &block.cells() {
        // Ok to blit block sprite even if position is OOB
        window.mvaddch(
            cell_pos.y + block_rel_pos.y + rel_pos_offset_y,
            cell_pos.x + block_rel_pos.x + rel_pos_offset_x,
            sprite_char,
        );
    }
    window.attroff(color_pair);
}

fn setup_colors() {
    pancurses::start_color();

    assert!(BLOCKTYPES.len() < pancurses::COLOR_PAIRS() as usize);
    for block_type in BLOCKTYPES.iter() {
        pancurses::init_pair(
            *block_type as i16,
            pancurses::COLOR_BLACK,
            block_type.sprite_color(),
        );
    }
}

fn draw_frame(window: &pancurses::Window, frame_rect: &Rect) {
    let left = frame_rect.left;
    let top = frame_rect.top;
    let right = frame_rect.right();
    let bottom = frame_rect.bottom();

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

    const INPUT_POLL_PERIOD: time::Duration = time::Duration::from_millis(125);
    let mut frame_speed_modifier = 1.0f32;

    const TITLE: &str = "TETRUST";
    pancurses::noecho();
    pancurses::cbreak();
    pancurses::curs_set(0);
    pancurses::set_title(TITLE);
    window.nodelay(true);
    setup_colors();

    let mut last_frame_time = time::Instant::now();
    let mut last_input_handled = time::Instant::now();

    const BOARD_RECT: Rect = Rect {
        left: 1,
        top: 1,
        width: 10,
        height: 20,
    };

    const BOARD_FRAME_RECT: Rect = Rect {
        left: BOARD_RECT.left - 1,
        top: BOARD_RECT.top - 1,
        width: BOARD_RECT.width + 2,
        height: BOARD_RECT.height + 2,
    };

    const TITLE_RECT: Rect = Rect {
        left: BOARD_FRAME_RECT.right() + 2,
        top: BOARD_FRAME_RECT.top,
        width: (TITLE.len() + 4) as i32,
        height: 3,
    };

    const PREVIEW_FRAME_RECT: Rect = Rect {
        left: TITLE_RECT.left,
        top: TITLE_RECT.bottom() + 2,
        width: 6,
        height: 6,
    };

    const PREVIEW_RECT: Rect = Rect {
        left: PREVIEW_FRAME_RECT.left + 1,
        top: PREVIEW_FRAME_RECT.top + 1,
        width: PREVIEW_FRAME_RECT.width - 2,
        height: PREVIEW_FRAME_RECT.height - 2,
    };

    const SCORE_FRAME_RECT: Rect = Rect {
        left: PREVIEW_FRAME_RECT.left,
        top: PREVIEW_FRAME_RECT.bottom() + 2,
        width: 14,
        height: 4,
    };

    let mut game_state = GameState::new(
        BOARD_RECT.width,
        BOARD_RECT.height,
        Box::new(ThreadRangeRng::new()),
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
    let mut game_paused = false;

    loop {
        let delta_time = last_frame_time.elapsed().mul_f32(frame_speed_modifier);
        last_frame_time = time::Instant::now();

        // Input handling
        let next_key = window.getch();
        if let Some(pancurses::Input::Character(ch)) = next_key {
            match ch {
                // check for movement inputs
                'a' => inputs.move_left = true,
                'd' => inputs.move_right = true,
                'j' => inputs.rot_left = true,
                'l' => inputs.rot_right = true,

                // debug
                'q' => break,                          // kill game early
                'p' => game_paused = !game_paused,     // toggle the pause state
                'z' => frame_speed_modifier /= 2.0f32, // slowdown tick rate
                'x' => frame_speed_modifier = 1.0f32,  // reset tick rate
                'c' => frame_speed_modifier *= 2.0f32, // speed up tick rate
                _ => (),
            }
        };

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
        if !game_paused {
            game_state.update(delta_time);
        }

        // Render the next frame
        window.erase();

        // Render the tetris title
        draw_frame(&window, &TITLE_RECT);
        draw_text_centered(&window, TITLE, TITLE_RECT.center_x(), TITLE_RECT.center_y());

        // Render next piece preview
        draw_text_centered(
            &window,
            "Next",
            PREVIEW_FRAME_RECT.center_x(),
            PREVIEW_FRAME_RECT.top - 1,
        );
        draw_frame(&window, &PREVIEW_FRAME_RECT);
        render_block(
            &window,
            Vec2::zero(),
            PREVIEW_RECT.left,
            PREVIEW_RECT.top,
            game_state.preview_block(),
        );

        // Render the score pane
        draw_text_centered(
            &window,
            &format!("Level: {:05}", game_state.level()),
            SCORE_FRAME_RECT.center_x(),
            SCORE_FRAME_RECT.center_y() - 1,
        );
        draw_text_centered(
            &window,
            &format!("Score: {:05}", game_state.score()),
            SCORE_FRAME_RECT.center_x(),
            SCORE_FRAME_RECT.center_y(),
        );
        draw_frame(&window, &SCORE_FRAME_RECT);

        // Render the active piece
        draw_frame(&window, &BOARD_FRAME_RECT);
        if let Some((block, block_pos)) = game_state.active_block() {
            render_block(&window, block_pos, BOARD_RECT.left, BOARD_RECT.top, block);
        }

        // Render the settled pieces
        game_state.for_each_settled_piece(|block_type: BlockType, cell_pos: Vec2| {
            render_cell(
                &window,
                cell_pos,
                BOARD_RECT.left,
                BOARD_RECT.top,
                block_type,
            );
        });

        // If the game is over, render the game over text
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

            window.attron(pancurses::A_BLINK);
            draw_text_centered(
                &window,
                "Game Over",
                BOARD_RECT.center_x(),
                BOARD_RECT.center_y(),
            );
            window.attroff(pancurses::A_BLINK);
        }
        // If the game is paused, render pause text
        else if game_paused {
            window.attron(pancurses::A_BLINK);
            draw_text_centered(
                &window,
                "PAUSE",
                BOARD_RECT.center_x(),
                BOARD_RECT.center_y(),
            );
            window.attroff(pancurses::A_BLINK);
        }

        window.refresh();
    }

    pancurses::endwin();

    println!("Lose!");
    println!("Finished");
}
