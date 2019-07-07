mod block;
use crate::block::*;

extern crate pancurses;
extern crate rand;

use pancurses::{endwin, initscr, Window};
use rand::{thread_rng, Rng};
use std::{thread, time};

fn in_bounds(window: &Window, row: i32, col: i32) -> bool {
    row >= 0 && row < window.get_max_y() && col >= 0 && col < window.get_max_x()
}

fn render_block(window: &Window, row: i32, col: i32, b: BlockType) {
    let block_char = b.to_char();
    let tetromino_pos = b.to_block_array();
    for block_pos in tetromino_pos.iter() {
        if in_bounds(window, block_pos.0, block_pos.1) {
            window.mvaddch(block_pos.0 + row, block_pos.1 + col, block_char);
        }
    }
}

fn main() {
    let window = initscr();
    let mut rng = thread_rng();
    for _dashgroup in 0..10 {
        window.erase();
        for _dashiter in 0..5 {
            let x: i32 = rng.gen_range(0, window.get_max_x());
            let y: i32 = rng.gen_range(0, window.get_max_y());
            render_block(&window, y, x, BlockType::random(&mut rng));

            thread::sleep(time::Duration::from_millis(200));
            window.refresh();
        }
    }
    window.getch();
    endwin();
}
