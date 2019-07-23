use crate::block::*;
use crate::randwrapper::*;

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

pub struct GameState<TBlockTypeRand, TBlockPosRand>
where
    TBlockTypeRand: RangeRng<usize>,
    TBlockPosRand: RangeRng<i32>,
{
    board_width: i32,
    board_height: i32,
    block_type_rng: TBlockTypeRand,
    block_pos_rng: TBlockPosRand,
    block_count: usize,
    blocks: Box<[BlockType]>,
    block_positions: Box<[Cell]>,
    game_phase: GamePhase,
}

impl<TBlockTypeRand, TBlockPosRand> GameState<TBlockTypeRand, TBlockPosRand>
where
    TBlockTypeRand: RangeRng<usize>,
    TBlockPosRand: RangeRng<i32>,
{
    pub fn new(
        board_width: i32,
        board_height: i32,
        block_type_rng: TBlockTypeRand,
        block_pos_rng: TBlockPosRand,
    ) -> GameState<TBlockTypeRand, TBlockPosRand> {
        let max_blocks = (board_width * board_height) as usize;
        GameState {
            board_width: board_width,
            board_height: board_height,
            block_type_rng: block_type_rng,
            block_pos_rng: block_pos_rng,
            block_count: 0,
            blocks: (vec![BlockType::I; max_blocks]).into_boxed_slice(),
            block_positions: (vec![Cell(0, 0); max_blocks]).into_boxed_slice(),
            game_phase: GamePhase::GenerateBlock,
        }
    }

    pub fn tick(&mut self) {
        match self.game_phase {
            GamePhase::GenerateBlock => {
                assert_eq!(self.blocks.len(), self.block_positions.len());
                assert!(self.block_count < self.blocks.len());

                let new_block = BlockType::random(&mut self.block_type_rng);
                let start_col: i32 = self
                    .block_pos_rng
                    .gen_range(0, self.board_width - new_block.width());

                self.blocks[self.block_count] = new_block;
                self.block_positions[self.block_count] = Cell(-new_block.height(), start_col);
                self.block_count += 1;
                self.game_phase = GamePhase::MoveBlock;
            }

            GamePhase::MoveBlock => {
                let moving_block_id = self.block_count - 1; // we are always moving the last block

                if self.has_block_landed(moving_block_id) {
                    self.game_phase = if self.block_positions[moving_block_id].0 < 0 {
                        GamePhase::GameOver
                    } else {
                        GamePhase::GenerateBlock
                    }
                } else {
                    self.block_positions[moving_block_id].0 += 1;
                }
            }

            GamePhase::GameOver => (),
        }
    }

    pub fn move_block_horizontal(&mut self, horizontal_motion: i32) {
        match self.game_phase {
            GamePhase::MoveBlock => {
                let moving_block_id = self.block_count - 1; // we are always moving the last block
                if self.can_block_move(moving_block_id, horizontal_motion) {
                    self.block_positions[moving_block_id].1 += horizontal_motion;
                }
            },
            GamePhase::GenerateBlock | GamePhase::GameOver => (),
        }
    }

    pub fn block_count(&self) -> usize {
        self.block_count
    }

    pub fn block(&self, id: usize) -> (Cell, BlockType) {
        assert_eq!(self.blocks.len(), self.block_positions.len());
        (self.block_positions[id], self.blocks[id])
    }

    // NOTE (scmunro): this function was added mostly for testing purposes. If possible, I'd like
    // to justify removing this function and even that test if necessary or find a better way to
    // do this without writing 'test only' helpers.
    pub fn has_block_landed(&self, block_id: usize) -> bool {
        assert_eq!(self.blocks.len(), self.block_positions.len());

        is_touching_bound(
            self.blocks[block_id],
            self.block_positions[block_id],
            Bound::Floor(self.board_height),
        ) || is_touching_block(
            block_id,
            self.block_count,
            &self.blocks,
            &self.block_positions,
            (1, 0),
        )
    }

    pub fn can_block_move(&self, block_id: usize, horizontal_motion: i32) -> bool {
        assert_eq!(self.blocks.len(), self.block_positions.len());

        if horizontal_motion == 0 {
            return false;
        }

        !is_touching_bound(
            self.blocks[block_id],
            self.block_positions[block_id],
            if horizontal_motion < 0 { Bound::LeftWall(0) } else { Bound::RightWall(self.board_width) },
        ) && !is_touching_block(
            block_id,
            self.block_count,
            &self.blocks,
            &self.block_positions,
            (0, horizontal_motion),
        )
    }

    pub fn is_game_over(&self) -> bool {
        self.game_phase == GamePhase::GameOver
    }
}

fn translate_cells(cells: &[Cell; 4], row_translation: i32, col_translation: i32) -> [Cell; 4] {
    let mut translated_cells: [Cell; 4] = *cells;
    for cell_index in 0..translated_cells.len() {
        translated_cells[cell_index].0 += row_translation;
        translated_cells[cell_index].1 += col_translation;
    }

    translated_cells
}

fn is_touching_bound(block: BlockType, block_pos: Cell, bound: Bound) -> bool {
    match bound {
        Bound::Floor(floor) => block_pos.0 + block.height() >= floor,
        Bound::LeftWall(left) => block_pos.1 <= left,
        Bound::RightWall(right) => block_pos.1 + block.width() >= right,
    }
}

fn is_touching_block(
    block_id: usize,
    block_count: usize,
    blocks: &[BlockType],
    block_positions: &[Cell],
    touch_vector: (i32, i32)
) -> bool {
    assert_eq!(blocks.len(), block_positions.len());
    assert!(blocks.len() >= block_count);

    let block_cells = translate_cells(
        &blocks[block_id].cells(),
        block_positions[block_id].0 + touch_vector.0,
        block_positions[block_id].1 + touch_vector.1,
    );

    // Only need to check for collisions against blocks that were created before this block id
    // since all other blocks will always be higher up in the grid.
    for other_block_id in 0..block_count {
        if other_block_id == block_id {
            continue;
        }

        let other_block_cells = translate_cells(
            &blocks[other_block_id].cells(),
            block_positions[other_block_id].0,
            block_positions[other_block_id].1,
        );

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
