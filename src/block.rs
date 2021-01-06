use crate::randwrapper::*;

#[derive(Clone, Copy, Debug)]
pub enum Rotation {
    Rot1,
    Rot2,
    Rot3,
    Rot4,
}

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
pub struct Block {
    pub rot: Rotation,
    pub block_type: BlockType,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    pub x: i32,
    pub y: i32,
}

macro_rules! cell_array {
    ( $(($x:expr,$y:expr)),* $(,)?) => {
        [
            $(
                Cell{x: $x, y: $y},
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

impl Rotation {
    fn rotate(&self, relative_rotation: i32) -> Self {
        enum RotationDirection {
            None,
            Left,
            Right,
        }

        let rotation_direction = match relative_rotation {
            0 => RotationDirection::None,
            -1 => RotationDirection::Left,
            1 => RotationDirection::Right,
            _ => panic!("Invalid relative rotation"),
        };

        match rotation_direction {
            RotationDirection::None => *self,
            RotationDirection::Left => match self {
                Rotation::Rot1 => Rotation::Rot4,
                Rotation::Rot2 => Rotation::Rot1,
                Rotation::Rot3 => Rotation::Rot2,
                Rotation::Rot4 => Rotation::Rot3,
            },
            RotationDirection::Right => match self {
                Rotation::Rot1 => Rotation::Rot2,
                Rotation::Rot2 => Rotation::Rot3,
                Rotation::Rot3 => Rotation::Rot4,
                Rotation::Rot4 => Rotation::Rot1,
            },
        }
    }
}

impl BlockType {
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
}

impl Block {
    pub fn default() -> Self {
        Block {
            rot: Rotation::Rot1,
            block_type: BlockType::I,
        }
    }

    pub fn random<T: RangeRng<usize>>(rng: &mut T) -> Self {
        Block {
            rot: Rotation::Rot1,
            block_type: BLOCKTYPES[rng.gen_range(1, BLOCKTYPES.len() + 1) - 1],
        }
    }

    pub fn sprite_char(&self) -> char {
        self.block_type.sprite_char()
    }

    pub fn cells(&self) -> [Cell; 4] {
        let rot = self.rot;
        match self.block_type {
            // - - - -    - - 0 -    - - - -    - 0 - -
            // 0 1 2 3 => - - 1 - => - - - - => - 1 - -
            // - - - -    - - 2 -    0 1 2 3    - 2 - -
            // - - - -    - - 3 -    - - - -    - 3 - -
            BlockType::I => match rot {
                Rotation::Rot1 => cell_array![(0, 1), (1, 1), (2, 1), (3, 1),],
                Rotation::Rot2 => cell_array![(2, 0), (2, 1), (2, 2), (2, 3),],
                Rotation::Rot3 => cell_array![(0, 2), (1, 2), (2, 2), (3, 2),],
                Rotation::Rot4 => cell_array![(1, 0), (1, 1), (1, 2), (1, 3),],
            },

            // - 0 1 -    - 0 1 -    - 0 1 -    - 0 1 -
            // - 2 3 - => - 2 3 - => - 2 3 - => - 2 3 -
            // - - - -    - - - -    - - - -    - - - -
            // - - - -    - - - -    - - - -    - - - -
            BlockType::O => cell_array![(1, 0), (2, 0), (1, 1), (2, 1),],

            // - 0 -    - 0 -    - - -    - 0 -
            // 1 2 3 => - 1 2 => 0 1 2 => 1 2 -
            // - - -    - 3 -    - 3 -    - 3 -
            BlockType::T => match rot {
                Rotation::Rot1 => cell_array![(1, 0), (0, 1), (1, 1), (2, 1),],
                Rotation::Rot2 => cell_array![(1, 0), (1, 1), (2, 1), (1, 2),],
                Rotation::Rot3 => cell_array![(0, 1), (1, 1), (2, 1), (1, 2),],
                Rotation::Rot4 => cell_array![(1, 0), (0, 1), (1, 1), (1, 2),],
            },

            // - 0 1    - 0 -    - - -    0 - -
            // 2 3 - => - 1 2 => - 0 1 => 1 2 -
            // - - -    - - 3    2 3 -    - 3 -
            BlockType::S => match rot {
                Rotation::Rot1 => cell_array![(1, 0), (2, 0), (0, 1), (1, 1),],
                Rotation::Rot2 => cell_array![(1, 0), (1, 1), (2, 1), (2, 2),],
                Rotation::Rot3 => cell_array![(1, 1), (2, 1), (0, 2), (1, 2),],
                Rotation::Rot4 => cell_array![(0, 0), (0, 1), (1, 1), (1, 2),],
            },

            // 0 1 -    - - 0    - - -    - 0 -
            // - 2 3 => - 1 2 => 0 1 - => 1 2 -
            // - - -    - 3 -    - 2 3    3 - -
            BlockType::Z => match rot {
                Rotation::Rot1 => cell_array![(0, 0), (1, 0), (1, 1), (2, 1),],
                Rotation::Rot2 => cell_array![(2, 0), (1, 1), (2, 1), (1, 2),],
                Rotation::Rot3 => cell_array![(0, 1), (1, 1), (1, 2), (2, 2),],
                Rotation::Rot4 => cell_array![(1, 0), (0, 1), (1, 1), (0, 2),],
            },

            // 0 - -    - 0 1    - - -    - 0 -
            // 1 2 3 => - 2 - => 0 1 2 => - 1 -
            // - - -    - 3 -    - - 3    2 3 -
            BlockType::J => match rot {
                Rotation::Rot1 => cell_array![(0, 0), (0, 1), (1, 1), (2, 1),],
                Rotation::Rot2 => cell_array![(1, 0), (2, 0), (1, 1), (1, 2),],
                Rotation::Rot3 => cell_array![(0, 1), (1, 1), (2, 1), (2, 2),],
                Rotation::Rot4 => cell_array![(1, 0), (1, 1), (0, 2), (1, 2),],
            },

            // - - 0    - 0 -    - - -    0 1 -
            // 1 2 3 => - 1 - => 0 1 2 => - 2 -
            // - - -    - 2 3    3 - -    - 3 -
            BlockType::L => match rot {
                Rotation::Rot1 => cell_array![(2, 0), (0, 1), (1, 1), (2, 1),],
                Rotation::Rot2 => cell_array![(1, 0), (1, 1), (1, 2), (2, 2),],
                Rotation::Rot3 => cell_array![(0, 1), (1, 1), (2, 1), (0, 2),],
                Rotation::Rot4 => cell_array![(0, 0), (1, 0), (1, 1), (1, 2),],
            },
        }
    }

    pub fn top(&self) -> i32 {
        // TODO (scottnm): handle different block orientations
        // NOTE (scottnm): Unwrap is safe because all blocks should have at least 1 cell
        self.cells().iter().min_by_key(|cell| cell.y).unwrap().y
    }

    pub fn left(&self) -> i32 {
        // TODO (scottnm): handle different block orientations
        // NOTE (scottnm): Unwrap is safe because all blocks should have at least 1 cell
        self.cells().iter().min_by_key(|cell| cell.x).unwrap().x
    }

    pub fn width(&self) -> i32 {
        // TODO (scottnm): handle different block orientations
        // NOTE (scottnm): Unwrap is safe because all blocks should have at least 1 cell
        let left_block = self.left();
        let right_block = self.cells().iter().max_by_key(|cell| cell.x).unwrap().x;
        right_block - left_block + 1
    }

    pub fn height(&self) -> i32 {
        // TODO (scottnm): handle different block orientations
        // NOTE (scottnm): Unwrap is safe because all blocks should have at least 1 cell
        let top_block = self.top();
        let bottom_block = self.cells().iter().max_by_key(|cell| cell.y).unwrap().y;
        bottom_block - top_block + 1
    }

    pub fn rotate(&self, relative_rotation: i32) -> Self {
        Self {
            rot: self.rot.rotate(relative_rotation),
            block_type: self.block_type,
        }
    }
}
