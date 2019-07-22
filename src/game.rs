use crate::block::*;
use crate::randwrapper::*;

#[derive(PartialEq, Eq)]
enum GamePhase {
    GenerateBlock,
    MoveBlock,
    GameOver,
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

        is_resting_on_floor(
            self.blocks[block_id],
            self.block_positions[block_id],
            self.board_height,
        ) || is_resting_on_other_block(
            block_id,
            self.block_count,
            &self.blocks,
            &self.block_positions,
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

fn is_resting_on_floor(block: BlockType, block_pos: Cell, floor_pos: i32) -> bool {
    block_pos.0 + block.height() >= floor_pos
}

fn is_resting_on_other_block(
    block_id: usize,
    block_count: usize,
    blocks: &[BlockType],
    block_positions: &[Cell],
) -> bool {
    assert_eq!(blocks.len(), block_positions.len());
    assert!(blocks.len() >= block_count);

    let block = blocks[block_id];
    let block_pos = block_positions[block_id];
    let block_cells = translate_cells(&block.cells(), block_pos.0, block_pos.1);

    // Only need to check for collisions against blocks that were created before this block id
    // since all other blocks will always be higher up in the grid.
    for other_block_id in 0..block_count {
        if other_block_id == block_id {
            continue;
        }

        let other_block = blocks[other_block_id];
        let other_block_pos = block_positions[other_block_id];
        let other_block_cells =
            translate_cells(&other_block.cells(), other_block_pos.0, other_block_pos.1);

        for cell in block_cells.iter() {
            for other_cell in other_block_cells.iter() {
                if (cell.1 == other_cell.1) && (cell.0 + 1 == other_cell.0) {
                    return true;
                }
            }
        }
    }

    false
}
