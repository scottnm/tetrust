#[macro_use]
extern crate savefile_derive;
extern crate pancurses;
extern crate rand;
extern crate savefile;
mod block;
mod game;
mod leaderboard;
mod randwrapper;
mod tests;
mod util;

use crate::block::*;
use crate::game::*;
use crate::leaderboard::*;
use crate::randwrapper::*;
use crate::util::*;
use std::time;

const TITLE: &str = "TETRUST";

const LEADERBOARD_FILE_NAME: &str = "data/leaderboard.bin";

const ASCII_ESC: char = 27 as char;
const ASCII_BACKSPACE: char = 8 as char;
const ASCII_DEL: char = 127 as char;
const ASCII_ENTER: char = 10 as char;

struct Colors {}

impl Colors {
    const MENU_COLOR_PALETTE: [i16; 4] = [
        pancurses::COLOR_CYAN,
        pancurses::COLOR_GREEN,
        pancurses::COLOR_MAGENTA,
        pancurses::COLOR_YELLOW,
    ];

    fn setup() {
        pancurses::start_color();

        assert!(
            (BLOCKTYPES.len() + Self::MENU_COLOR_PALETTE.len()) < pancurses::COLOR_PAIRS() as usize
        );

        fn block_color(block_type: BlockType) -> i16 {
            match block_type {
                BlockType::I => pancurses::COLOR_WHITE,
                BlockType::O => pancurses::COLOR_RED,
                BlockType::T => pancurses::COLOR_CYAN,
                BlockType::S => pancurses::COLOR_GREEN,
                BlockType::Z => pancurses::COLOR_MAGENTA,
                BlockType::J => pancurses::COLOR_YELLOW,
                BlockType::L => pancurses::COLOR_BLUE,
            }
        }

        // slots 1->BLOCKTYPES.len() are for block colors
        // must be kept in sync with get_block_color_pair()
        for block_type in BLOCKTYPES.iter() {
            pancurses::init_pair(
                *block_type as i16,
                pancurses::COLOR_BLACK,
                block_color(*block_type),
            );
        }

        // slots BLOCKTYPES.len()+1 and above are for menu colors
        // must be kept in sync with get_menu_color_pair()
        for (i, menu_color) in Self::MENU_COLOR_PALETTE.iter().enumerate() {
            pancurses::init_pair(
                (i + 1 + BLOCKTYPES.len()) as i16,
                *menu_color,
                pancurses::COLOR_BLACK,
            );
        }
    }

    fn get_block_color_pair(block_type: BlockType) -> pancurses::chtype {
        pancurses::COLOR_PAIR(block_type as pancurses::chtype)
    }

    fn get_menu_color_pair(menu_color_index: usize) -> pancurses::chtype {
        pancurses::COLOR_PAIR((menu_color_index + BLOCKTYPES.len() + 1) as pancurses::chtype)
    }
}

fn render_cell(
    window: &pancurses::Window,
    cell_rel_pos: Vec2,
    rel_pos_offset_x: i32,
    rel_pos_offset_y: i32,
    block_type: BlockType,
) {
    let sprite_char = block_type.sprite_char();
    let color_pair = Colors::get_block_color_pair(block_type);
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
    let color_pair = Colors::get_block_color_pair(block.block_type);
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

#[derive(Debug, Clone, Copy)]
enum Screen {
    StartMenu,
    Game,
    LeaderboardUpdate(usize),
    Leaderboard,
}

fn run_start_menu(window: &pancurses::Window) -> Option<Screen> {
    const TITLE_LINES: [&str; 7] = [
        r#" _____________"#,
        r#"/\____________\ ___  _____  ___  .   .   ___   _____"#,
        r#"\/___/\   \___/ \___    \   \ _)  \   \  \ ___    \"#,
        r#"     \ \   \     \___    \   \  \  \___\   ___\    \"#,
        r#"      \ \   \"#,
        r#"       \ \___\"#,
        r#"        \/___/"#,
    ];

    let (window_height, window_width) = window.get_max_yx();

    let title_rect = {
        let title_width = TITLE_LINES.iter().map(|line| line.len()).max().unwrap() as i32;
        const TITLE_HEIGHT: i32 = TITLE_LINES.len() as i32;

        Rect {
            // center the title horizontally
            left: (window_width - title_width) / 2,
            // place the title just above the horizontal divide
            top: (window_height / 2) - (TITLE_HEIGHT + 1),
            width: title_width,
            height: TITLE_HEIGHT,
        }
    };

    let mut menu_cursor: usize = 0;
    const MENU_OPTIONS: [&str; 3] = ["Start Game", "Leaderboard", "Quit"];
    const MENU_OPTION_RESULTS: [Option<Screen>; MENU_OPTIONS.len()] =
        [Some(Screen::Game), Some(Screen::Leaderboard), None];

    let menu_rect = {
        let menu_width = MENU_OPTIONS
            .iter()
            .map(|option_text| option_text.len())
            .max()
            .unwrap() as i32;
        const MENU_HEIGHT: i32 = MENU_OPTIONS.len() as i32;

        Rect {
            // center the menu options horizontally
            left: (window_width - menu_width) / 2,
            // place the menu options just below the horizontal divide
            top: (window_height / 2) + 1,
            // Add 2 characters to the menu width to account for the cursor
            width: menu_width + 2,
            height: MENU_HEIGHT,
        }
    };

    loop {
        // clear the screen
        window.erase();

        // Render the title card
        for (i, title_line) in TITLE_LINES.iter().enumerate() {
            let row_offset = (i as i32) + title_rect.top;
            let color_pair = Colors::get_menu_color_pair(i % 4);
            window.attron(color_pair);
            window.mvaddstr(row_offset, title_rect.left, title_line);
            window.attroff(color_pair);
        }

        // Render the menu options
        for (i, menu_line) in MENU_OPTIONS.iter().enumerate() {
            let row_offset = (i as i32) + menu_rect.top;
            if i == menu_cursor {
                window.mvaddstr(row_offset, menu_rect.left, "> ");
            }
            window.mvaddstr(row_offset, menu_rect.left + 2, menu_line);
        }

        // Input handling
        // TODO: I think this input system might need some refactoring to share with the start menu
        if let Some(pancurses::Input::Character(ch)) = window.getch() {
            match ch {
                // check for movement inputs
                'w' => {
                    menu_cursor = if menu_cursor == 0 {
                        MENU_OPTIONS.len() - 1
                    } else {
                        menu_cursor - 1
                    }
                }
                's' => {
                    menu_cursor = if menu_cursor == MENU_OPTIONS.len() - 1 {
                        0
                    } else {
                        menu_cursor + 1
                    }
                }
                ASCII_ENTER => return MENU_OPTION_RESULTS[menu_cursor],
                _ => (),
            }
        };

        // blit the next frame
        window.refresh();
    }
}

fn run_game(window: &pancurses::Window) -> Option<Screen> {
    const INPUT_POLL_PERIOD: time::Duration = time::Duration::from_millis(125);
    let mut frame_speed_modifier = 1.0f32;

    let mut last_frame_time = time::Instant::now();
    let mut last_input_handled = time::Instant::now();

    // standard tetris board size
    const BOARD_WIDTH: i32 = 10;
    const BOARD_HEIGHT: i32 = 20;

    let (window_height, window_width) = window.get_max_yx();
    let board_rect = Rect {
        left: (window_width / 2) - BOARD_WIDTH - 2, // arrange the board on the left side of the middle of the screen
        top: (window_height - BOARD_HEIGHT) / 2,    // center the board within the window
        width: BOARD_WIDTH,
        height: 20,
    };

    let board_frame_rect = Rect {
        left: board_rect.left - 1,
        top: board_rect.top - 1,
        width: board_rect.width + 2,
        height: board_rect.height + 2,
    };

    let title_rect = Rect {
        left: board_frame_rect.right() + 2,
        top: board_frame_rect.top,
        width: (TITLE.len() + 4) as i32,
        height: 3,
    };

    let preview_frame_rect = Rect {
        left: title_rect.left,
        top: title_rect.bottom() + 2,
        width: 6,
        height: 6,
    };

    let preview_rect = Rect {
        left: preview_frame_rect.left + 1,
        top: preview_frame_rect.top + 1,
        width: preview_frame_rect.width - 2,
        height: preview_frame_rect.height - 2,
    };

    let score_frame_rect = Rect {
        left: preview_frame_rect.left,
        top: preview_frame_rect.bottom() + 2,
        width: 14,
        height: 4,
    };

    let mut game_state = GameState::new(
        board_rect.width,
        board_rect.height,
        Box::new(ThreadRangeRng::new()),
    );

    struct Inputs {
        move_left: bool,
        move_right: bool,
        rot_left: bool,
        rot_right: bool,
        drop: bool,
    }

    let mut inputs = Inputs {
        move_left: false,
        move_right: false,
        rot_left: false,
        rot_right: false,
        drop: false,
    };

    let mut game_over_blit_timer = Option::<time::Instant>::None;
    let mut game_paused = false;

    loop {
        let delta_time = last_frame_time.elapsed().mul_f32(frame_speed_modifier);
        last_frame_time = time::Instant::now();

        // Input handling
        let next_key = window.getch();
        // TODO: I think this input system might need some refactoring to share with the start menu
        if let Some(input) = next_key {
            match input {
                // check for movement inputs
                pancurses::Input::Character('a') => inputs.move_left = true,
                pancurses::Input::Character('d') => inputs.move_right = true,
                pancurses::Input::Character('s') => inputs.drop = true,
                pancurses::Input::KeyLeft => inputs.rot_left = true,
                pancurses::Input::KeyRight => inputs.rot_right = true,

                // debug
                pancurses::Input::Character(ASCII_ESC) => break, // kill game early
                pancurses::Input::Character('p') => game_paused = !game_paused, // toggle the pause state
                pancurses::Input::Character('z') => frame_speed_modifier /= 2.0f32, // slowdown tick rate
                pancurses::Input::Character('x') => frame_speed_modifier = 1.0f32, // reset tick rate
                pancurses::Input::Character('c') => frame_speed_modifier *= 2.0f32, // speed up tick rate
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
            game_state.move_active_block_horizontal(horizontal_motion);

            let mut relative_rotation: i32 = 0;
            if inputs.rot_left {
                relative_rotation -= 1;
            }
            if inputs.rot_right {
                relative_rotation += 1;
            }
            game_state.rotate_block(relative_rotation);

            if inputs.drop {
                game_state.quick_drop();
            }

            inputs = Inputs {
                move_left: false,
                move_right: false,
                rot_left: false,
                rot_right: false,
                drop: false,
            };
        }

        // Tick the game state
        if !game_paused {
            game_state.update(delta_time);
        }

        // Render the next frame
        window.erase();

        // Render the tetris title
        draw_frame(&window, &title_rect);
        draw_text_centered(&window, TITLE, title_rect.center_x(), title_rect.center_y());

        // Render next piece preview
        draw_text_centered(
            &window,
            "Next",
            preview_frame_rect.center_x(),
            preview_frame_rect.top - 1,
        );
        draw_frame(&window, &preview_frame_rect);
        render_block(
            &window,
            Vec2::zero(),
            preview_rect.left,
            preview_rect.top,
            game_state.preview_block(),
        );

        // Render the score pane
        draw_text_centered(
            &window,
            &format!("Level: {:05}", game_state.level()),
            score_frame_rect.center_x(),
            score_frame_rect.center_y() - 1,
        );
        draw_text_centered(
            &window,
            &format!("Score: {:05}", game_state.score()),
            score_frame_rect.center_x(),
            score_frame_rect.center_y(),
        );
        draw_frame(&window, &score_frame_rect);

        // Render the board frame
        draw_frame(&window, &board_frame_rect);

        // Render the active piece
        if let Some((block, block_pos)) = game_state.active_block() {
            // TOOD: mayhaps refactor this into its own helper?
            // render the active piece's drop trail
            for cell in &block.cells() {
                let start_row = cell.y + block_pos.y;
                let col = cell.x + block_pos.x;
                for row in start_row..board_rect.height {
                    window.mvaddch(row + board_rect.top, col + board_rect.left, '-');
                }
            }

            render_block(&window, block_pos, board_rect.left, board_rect.top, block);
        }

        // Render the settled pieces
        game_state.for_each_settled_piece(|block_type: BlockType, cell_pos: Vec2| {
            render_cell(
                &window,
                cell_pos,
                board_rect.left,
                board_rect.top,
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
                board_rect.center_x(),
                board_rect.center_y(),
            );
            window.attroff(pancurses::A_BLINK);
        }
        // If the game is paused, render pause text
        else if game_paused {
            window.attron(pancurses::A_BLINK);
            draw_text_centered(
                &window,
                "PAUSE",
                board_rect.center_x(),
                board_rect.center_y(),
            );
            window.attroff(pancurses::A_BLINK);
        }

        window.refresh();
    }

    Some(Screen::LeaderboardUpdate(game_state.score()))
}

fn display_leaderboard(
    window: &pancurses::Window,
    leaderboard: &Leaderboard,
    skip_entry: Option<usize>,
) {
    let leaderboard_rect = {
        //      Leaderboard
        //
        // #00      FML      00000
        //
        // #01      FML      00000
        // ...
        // #10      FML      00000
        const LEADERBOARD_ENTRY_WIDTH: i32 = 21;
        const LEADERBOARD_HEIGHT: i32 = (2 * (Leaderboard::max_entries() + 1) - 1) as i32;

        let (window_height, window_width) = window.get_max_yx();
        Rect {
            left: (window_width - LEADERBOARD_ENTRY_WIDTH) / 2,
            top: (window_height - LEADERBOARD_HEIGHT) / 2,
            width: LEADERBOARD_ENTRY_WIDTH,
            height: LEADERBOARD_HEIGHT,
        }
    };

    let leaderboard_frame_rect = Rect {
        left: leaderboard_rect.left - 1,
        top: leaderboard_rect.top - 1,
        width: leaderboard_rect.width + 2,
        height: leaderboard_rect.height + 2,
    };

    draw_frame(&window, &leaderboard_frame_rect);
    draw_text_centered(
        &window,
        "Leaderboard",
        leaderboard_rect.center_x(),
        leaderboard_rect.top,
    );

    for i in 0..Leaderboard::max_entries() {
        let entry = leaderboard.entry(i);
        let (name, score) = entry
            .map(|e| (e.name.as_ref(), e.score))
            .unwrap_or(("---", 0));

        let mut leaderboard_pos = i + 1;
        if skip_entry.is_some() && skip_entry.unwrap() <= i {
            leaderboard_pos += 1;
        }

        if leaderboard_pos <= Leaderboard::max_entries() {
            let row_offset = (leaderboard_pos * 2) as i32;

            draw_text_centered(
                &window,
                &format!("#{:02}    {:3}    {:05}", leaderboard_pos, name, score),
                leaderboard_rect.center_x(),
                leaderboard_rect.top + row_offset,
            );
        }
    }
}

fn run_leaderboard_update(window: &pancurses::Window, score: usize) -> Option<Screen> {
    let mut leaderboard = {
        let leaderboard_from_file = Leaderboard::load(LEADERBOARD_FILE_NAME);
        leaderboard_from_file.unwrap_or(Leaderboard::new())
    };

    let new_leaderboard_entry_pos = leaderboard.get_place_on_leaderboard(score);
    if new_leaderboard_entry_pos.is_some() {
        let leaderboard_rect = {
            //      Leaderboard
            //
            // #00      FML      00000
            //
            // #01      FML      00000
            // ...
            // #10      FML      00000
            const LEADERBOARD_ENTRY_WIDTH: i32 = 21;
            const LEADERBOARD_HEIGHT: i32 = (2 * (Leaderboard::max_entries() + 1) - 1) as i32;

            let (window_height, window_width) = window.get_max_yx();
            Rect {
                left: (window_width - LEADERBOARD_ENTRY_WIDTH) / 2,
                top: (window_height - LEADERBOARD_HEIGHT) / 2,
                width: LEADERBOARD_ENTRY_WIDTH,
                height: LEADERBOARD_HEIGHT,
            }
        };

        let mut next_initial = 0;
        let mut initials = ['_'; 3];

        loop {
            if let Some(input) = window.getch() {
                match input {
                    // check for movement inputs
                    pancurses::Input::Character(ASCII_ENTER) | pancurses::Input::KeyEnter => break,
                    pancurses::Input::Character(ASCII_BACKSPACE)
                    | pancurses::Input::Character(ASCII_DEL)
                    | pancurses::Input::KeyBackspace
                    | pancurses::Input::KeyDC => {
                        next_initial = std::cmp::max(1, next_initial) - 1;
                        initials[next_initial] = '_';
                    }
                    pancurses::Input::Character(letter) => {
                        if next_initial < initials.len() {
                            initials[next_initial] = letter;
                            next_initial += 1;
                        }
                    }
                    _ => (),
                }
            };

            window.erase();
            display_leaderboard(&window, &leaderboard, new_leaderboard_entry_pos);

            let leaderboard_pos = new_leaderboard_entry_pos.unwrap() + 1;
            let row_offset = (leaderboard_pos * 2) as i32;

            // Render the new entry WIP space
            window.attron(pancurses::A_BLINK);
            draw_text_centered(
                &window,
                &format!(
                    "#{:02}    {}{}{}    {:05}",
                    leaderboard_pos, initials[0], initials[1], initials[2], score
                ),
                leaderboard_rect.center_x(),
                leaderboard_rect.top + row_offset,
            );
            window.attroff(pancurses::A_BLINK);

            window.refresh();
        }

        let name = initials
            .iter()
            .map(|initial| if *initial == '_' { ' ' } else { *initial })
            .collect::<String>();
        leaderboard.add_score(name, score);
        leaderboard.save(LEADERBOARD_FILE_NAME);
    }

    Some(Screen::Leaderboard)
}

fn run_leaderboard_display(window: &pancurses::Window) -> Option<Screen> {
    let leaderboard = {
        let leaderboard_from_file = Leaderboard::load(LEADERBOARD_FILE_NAME);
        leaderboard_from_file.unwrap_or(Leaderboard::new())
    };

    window.erase();
    display_leaderboard(&window, &leaderboard, None);
    window.refresh();
    std::thread::sleep(std::time::Duration::from_secs(3));

    Some(Screen::StartMenu)
}

fn main() {
    // setup the window
    let window = pancurses::initscr();
    pancurses::noecho(); // prevent key inputs rendering to the screen
    pancurses::cbreak();
    pancurses::curs_set(0);
    pancurses::set_title(TITLE);
    window.nodelay(true); // don't block waiting for key inputs (we'll poll)
    window.keypad(true); // let special keys be captured by the program (i.e. esc/backspace/del/arrow keys)

    // setup the color system
    Colors::setup();

    // Run the game until we quit
    let mut screen = Screen::StartMenu;
    loop {
        // Run the current screen until it signals a transition
        let next_screen = match screen {
            Screen::StartMenu => run_start_menu(&window),
            Screen::Game => run_game(&window),
            Screen::LeaderboardUpdate(score) => run_leaderboard_update(&window, score),
            Screen::Leaderboard => run_leaderboard_display(&window),
        };

        // If the transition includes a new screen start rendering that.
        screen = match next_screen {
            Some(s) => s,
            None => break,
        }
    }

    // Close the window
    pancurses::endwin();
}
