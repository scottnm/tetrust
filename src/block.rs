extern crate rand;
extern crate pancurses;

use rand::{rngs, Rng};

#[derive(Clone, Copy, Debug)]
pub enum BlockType {
    I = 1, // NOTE (scottnm): if our enum starts at 0, init_pair doesn't seem to function. Needs investigation
    O,
    T,
    S,
    Z,
    J,
    L,
}

// TODO: add Cell tuple struct

pub static BLOCKTYPES: [BlockType; 7] = [
    BlockType::I,
    BlockType::O,
    BlockType::T,
    BlockType::S,
    BlockType::Z,
    BlockType::J,
    BlockType::L,
];

impl BlockType {
    pub fn random(rng: &mut rngs::ThreadRng) -> BlockType {
        BLOCKTYPES[rng.gen_range(0, BLOCKTYPES.len())]
    }

    pub fn sprite_char(&self) -> char {
        match *self {
            BlockType::I => 'O',
            BlockType::O => 'X',
            BlockType::T => '+',
            BlockType::S => '>',
            BlockType::Z => '<',
            BlockType::J => '/',
            BlockType::L => '\\',
        }
    }

    pub fn sprite_color(&self) -> i16 {
        match *self {
            BlockType::I => pancurses::COLOR_WHITE,
            BlockType::O => pancurses::COLOR_RED,
            BlockType::T => pancurses::COLOR_CYAN,
            BlockType::S => pancurses::COLOR_GREEN,
            BlockType::Z => pancurses::COLOR_MAGENTA,
            BlockType::J => pancurses::COLOR_YELLOW,
            BlockType::L => pancurses::COLOR_BLUE,
        }
    }

    #[rustfmt::skip] // skip rust formatting so that my block declarations can look pleasant
    pub fn cells(&self) -> [(i32, i32); 4] {
        match *self {
            BlockType::I =>
                [
                    (0, 0),
                    (1, 0),
                    (2, 0),
                    (3, 0),
                ],

            BlockType::O =>
                [
                    (0, 0), (0, 1),
                    (1, 0), (1, 1),
                ],

            BlockType::T =>
                [
                    (0, 0), (0, 1), (0, 2),
                            (1, 1),
                ],

            BlockType::S =>
                [
                            (0, 1), (0, 2),
                    (1, 0), (1, 1),
                ],

            BlockType::Z =>
                [
                    (0, 0), (0, 1),
                            (1, 1), (1, 2),
                ],

            BlockType::J =>
                [
                            (0, 1),
                            (1, 1),
                    (2, 0), (2, 1),
                ],

            BlockType::L =>
                [
                    (0, 0),
                    (1, 0),
                    (2, 0), (2, 1),
                ],
        }
    }

    pub fn height(&self) -> i32 {
        // TODO (scottnm): handle different block orientations
        // NOTE (scottnm): Unwrap is safe because all blocks should have at least 1 cell
        self.cells().iter().max_by_key(|cell| cell.0).unwrap().0
    }
}
