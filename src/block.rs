use crate::randwrapper::*;

// TODO (scottnm): separate out blocktype into blocktype and blockvisuals and blockdata (orientation+cells)
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

#[derive(Clone, Copy, Debug)]
pub struct Cell(pub i32, pub i32);

macro_rules! cell_array {
    ( $(($x:expr,$y:expr)),* $(,)?) => {
        [
            $(
                Cell($x,$y),
            )*
        ]
    };
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
    pub fn random<T: RangeRng<usize>>(rng: &mut T) -> BlockType {
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
    pub fn cells(&self) -> [Cell; 4] {
        match *self {
            BlockType::I =>
                cell_array![
                    (0, 0),
                    (1, 0),
                    (2, 0),
                    (3, 0),
                ],

            BlockType::O =>
                cell_array![
                    (0, 0), (0, 1),
                    (1, 0), (1, 1),
                ],

            BlockType::T =>
                cell_array![
                    (0, 0), (0, 1), (0, 2),
                            (1, 1),
                ],

            BlockType::S =>
                cell_array![
                            (0, 1), (0, 2),
                    (1, 0), (1, 1),
                ],

            BlockType::Z =>
                cell_array![
                    (0, 0), (0, 1),
                            (1, 1), (1, 2),
                ],

            BlockType::J =>
                cell_array![
                            (0, 1),
                            (1, 1),
                    (2, 0), (2, 1),
                ],

            BlockType::L =>
                cell_array![
                    (0, 0),
                    (1, 0),
                    (2, 0), (2, 1),
                ],
        }
    }

    pub fn width(&self) -> i32 {
        // TODO (scottnm): handle different block orientations
        // NOTE (scottnm): Unwrap is safe because all blocks should have at least 1 cell
        self.cells().iter().max_by_key(|cell| cell.1).unwrap().1 + 1
    }

    pub fn height(&self) -> i32 {
        // TODO (scottnm): handle different block orientations
        // NOTE (scottnm): Unwrap is safe because all blocks should have at least 1 cell
        self.cells().iter().max_by_key(|cell| cell.0).unwrap().0 + 1
    }
}
