use crate::block::*;
use crate::randwrapper::*;

#[derive(PartialEq, Eq)]
struct Vec2 {
    x: i32,
    y: i32,
}

#[derive(PartialEq, Eq)]
enum GamePhase {
    GenerateBlock,
    MoveBlock,
    GameOver,
}

enum Bound {
    Floor(i32),
    LeftWall(i32),
    RightWall(i32),
}

pub struct GameState<TBlockTypeRand>
where
    TBlockTypeRand: RangeRng<usize>,
{
    board_pos_x: i32,
    board_pos_y: i32,
    board_width: i32,
    board_height: i32,
    block_type_rng: TBlockTypeRand,
    block_count: usize,
    blocks: Box<[Block]>,
    block_positions: Box<[Cell]>,
    game_phase: GamePhase,
}

impl<TBlockTypeRand> GameState<TBlockTypeRand>
where
    TBlockTypeRand: RangeRng<usize>,
{
    pub fn new(
        board_pos_x: i32,
        board_pos_y: i32,
        board_width: i32,
        board_height: i32,
        block_type_rng: TBlockTypeRand,
    ) -> GameState<TBlockTypeRand> {
        let max_blocks = (board_width * board_height) as usize;
        GameState {
            board_pos_x,
            board_pos_y,
            board_width,
            board_height,
            block_type_rng,
            block_count: 0,
            blocks: (vec![Block::default(); max_blocks]).into_boxed_slice(),
            block_positions: (vec![Cell { x: 0, y: 0 }; max_blocks]).into_boxed_slice(),
            game_phase: GamePhase::GenerateBlock,
        }
    }

    pub fn tick(&mut self) {
        match self.game_phase {
            // Add a new block to the top of the board
            GamePhase::GenerateBlock => {
                assert_eq!(self.blocks.len(), self.block_positions.len());
                assert!(self.block_count < self.blocks.len());

                let new_block = Block::random(&mut self.block_type_rng);
                let start_col = (self.board_width - new_block.width()) / 2 + self.board_pos_x
                    - new_block.left();
                let start_row = self.board_pos_y - new_block.height();

                let start_pos = Cell {
                    x: start_col,
                    y: start_row,
                };

                self.blocks[self.block_count] = new_block;
                self.block_positions[self.block_count] = start_pos;
                self.block_count += 1;
                self.game_phase = GamePhase::MoveBlock;
            }

            // Move the latest block down across the board
            GamePhase::MoveBlock => {
                // we are always moving the last block
                let active_block_id = self.block_count - 1;

                if self.has_block_landed(active_block_id) {
                    let is_block_oob = self.block_positions[active_block_id].y < self.board_pos_y;
                    self.game_phase = if is_block_oob {
                        GamePhase::GameOver
                    } else {
                        GamePhase::GenerateBlock
                    }
                } else {
                    self.block_positions[active_block_id].y += 1;
                }
            }

            // The game is over; NOOP
            GamePhase::GameOver => (),
        }
    }

    pub fn move_block_horizontal(&mut self, horizontal_motion: i32) {
        match self.game_phase {
            GamePhase::MoveBlock => {
                let active_block_id = self.block_count - 1; // we are always moving the last block
                if self.can_block_move(active_block_id, horizontal_motion) {
                    self.block_positions[active_block_id].x += horizontal_motion;
                }
            }
            GamePhase::GenerateBlock | GamePhase::GameOver => (),
        }
    }

    pub fn rotate_block(&mut self, relative_rotation: i32) {
        match self.game_phase {
            GamePhase::MoveBlock => {
                let active_block = &mut self.blocks[self.block_count - 1]; // we are always moving the last block
                let rotated_block = active_block.rotate(relative_rotation);
                *active_block = rotated_block;
            }
            GamePhase::GenerateBlock | GamePhase::GameOver => (),
        }
    }

    pub fn block_count(&self) -> usize {
        self.block_count
    }

    pub fn block(&self, id: usize) -> (Cell, Block) {
        assert_eq!(self.blocks.len(), self.block_positions.len());
        (self.block_positions[id], self.blocks[id])
    }

    // NOTE (scmunro): this function was added mostly for testing purposes. If possible, I'd like
    // to justify removing this function and even that test if necessary or find a better way to
    // do this without writing 'test only' helpers.
    pub fn has_block_landed(&self, block_id: usize) -> bool {
        assert_eq!(self.blocks.len(), self.block_positions.len());

        let is_touching_floor = is_touching_bound(
            self.blocks[block_id],
            self.block_positions[block_id],
            Bound::Floor(self.board_pos_y + self.board_height),
        );

        if is_touching_floor {
            return true;
        }

        let do_blocks_collide_below = do_blocks_collide(
            self.blocks[block_id],
            self.block_positions[block_id],
            &self.blocks[0..self.block_count - 1],
            &self.block_positions[0..self.block_count - 1],
            Vec2 { x: 0, y: 1 },
        );

        do_blocks_collide_below
    }

    pub fn can_block_move(&self, block_id: usize, horizontal_motion: i32) -> bool {
        assert_eq!(self.blocks.len(), self.block_positions.len());

        if horizontal_motion == 0 {
            return false;
        }

        let wall_to_check = if horizontal_motion < 0 {
            Bound::LeftWall(self.board_pos_x - 1)
        } else {
            Bound::RightWall(self.board_pos_x + self.board_width)
        };

        let is_touching_wall = is_touching_bound(
            self.blocks[block_id],
            self.block_positions[block_id],
            wall_to_check,
        );

        if is_touching_wall {
            return false;
        }

        let motion_vec = Vec2 {
            x: horizontal_motion,
            y: 0,
        };

        let do_blocks_collide_side = do_blocks_collide(
            self.blocks[block_id],
            self.block_positions[block_id],
            &self.blocks[0..self.block_count - 1],
            &self.block_positions[0..self.block_count - 1],
            motion_vec,
        );

        !do_blocks_collide_side
    }

    pub fn is_game_over(&self) -> bool {
        self.game_phase == GamePhase::GameOver
    }
}

fn translate_cells(cells: &[Cell; 4], row_translation: i32, col_translation: i32) -> [Cell; 4] {
    let mut translated_cells: [Cell; 4] = *cells;
    for cell_index in 0..translated_cells.len() {
        translated_cells[cell_index].y += row_translation;
        translated_cells[cell_index].x += col_translation;
    }

    translated_cells
}

fn is_touching_bound(block: Block, block_pos: Cell, bound: Bound) -> bool {
    match bound {
        Bound::Floor(floor) => block.top() + block_pos.y + block.height() >= floor,
        Bound::LeftWall(left) => block.left() + block_pos.x <= left + 1,
        Bound::RightWall(right) => block.left() + block_pos.x + block.width() >= right,
    }
}

fn do_blocks_collide(
    block: Block,
    block_pos: Cell,
    other_blocks: &[Block],
    other_block_positions: &[Cell],
    move_vector: Vec2,
) -> bool {
    assert_eq!(other_blocks.len(), other_block_positions.len());

    let block_cells = translate_cells(
        &block.cells(),
        block_pos.y + move_vector.y,
        block_pos.x + move_vector.x,
    );

    // Only need to check for collisions against blocks that were created before this block id
    // since all other blocks will always be higher up in the grid.
    for (other_block, other_block_pos) in other_blocks.iter().zip(other_block_positions.iter()) {
        let other_block_cells =
            translate_cells(&other_block.cells(), other_block_pos.y, other_block_pos.x);

        for cell in block_cells.iter() {
            for other_cell in other_block_cells.iter() {
                if cell == other_cell {
                    return true;
                }
            }
        }
    }

    false
}
