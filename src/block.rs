extern crate rand;
use rand::{rngs, Rng};

#[derive(Clone, Copy)]
pub enum BlockType {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

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

    pub fn to_char(&self) -> char {
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

    #[rustfmt::skip]
    pub fn to_block_array(&self) -> [(i32, i32); 4] {
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
}
