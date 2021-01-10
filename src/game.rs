use crate::block::*;
use crate::randwrapper::*;

#[derive(PartialEq, Eq)]
struct Vec2 {
    x: i32,
    y: i32,
}

#[derive(PartialEq, Eq)]
enum GamePhase {
    StartNextBlock,
    MoveBlock,
    GameOver,
}

#[derive(Clone, Copy)]
enum Bound {
    Floor(i32),
    LeftWall(i32),
    RightWall(i32),
}

pub struct GameState<TBlockTypeRand>
where
    TBlockTypeRand: RangeRng<usize>,
{
    board_width: i32,
    board_height: i32,
    block_type_rng: TBlockTypeRand,
    block_count: usize,
    blocks: Box<[Block]>,
    block_positions: Box<[Cell]>,
    next_block: Block,
    game_phase: GamePhase,
}

impl<TBlockTypeRand> GameState<TBlockTypeRand>
where
    TBlockTypeRand: RangeRng<usize>,
{
    pub fn new(
        board_width: i32,
        board_height: i32,
        mut block_type_rng: TBlockTypeRand,
    ) -> GameState<TBlockTypeRand> {
        let initial_block = Block::random(&mut block_type_rng);
        let max_blocks = (board_width * board_height) as usize;
        GameState {
            board_width,
            board_height,
            block_type_rng,
            block_count: 0,
            blocks: (vec![Block::default(); max_blocks]).into_boxed_slice(),
            block_positions: (vec![Cell { x: 0, y: 0 }; max_blocks]).into_boxed_slice(),
            next_block: initial_block,
            game_phase: GamePhase::StartNextBlock,
        }
    }

    #[cfg(test)]
    pub fn width(&self) -> i32 {
        self.board_width
    }

    #[cfg(test)]
    pub fn height(&self) -> i32 {
        self.board_height
    }

    pub fn tick(&mut self) {
        match self.game_phase {
            // Add a new block to the top of the board
            GamePhase::StartNextBlock => {
                assert_eq!(self.blocks.len(), self.block_positions.len());
                assert!(self.block_count < self.blocks.len());

                let new_next_block = Block::random(&mut self.block_type_rng);
                let next_block = std::mem::replace(&mut self.next_block, new_next_block);

                let start_col = (self.board_width - next_block.width()) / 2 - next_block.left();
                let start_row = -next_block.height();

                let start_pos = Cell {
                    x: start_col,
                    y: start_row,
                };

                self.blocks[self.block_count] = next_block;
                self.block_positions[self.block_count] = start_pos;
                self.block_count += 1;
                self.game_phase = GamePhase::MoveBlock;
            }

            // Move the latest block down across the board
            GamePhase::MoveBlock => {
                // we are always moving the last block
                let active_block_id = self.block_count - 1;

                if self.has_block_landed(active_block_id) {
                    let is_block_above_board = self.block_positions[active_block_id].y < 0;
                    self.game_phase = if is_block_above_board {
                        GamePhase::GameOver
                    } else {
                        GamePhase::StartNextBlock
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
            GamePhase::StartNextBlock | GamePhase::GameOver => (),
        }
    }

    pub fn rotate_block(&mut self, relative_rotation: i32) {
        // no rotation means no rotation. noop.
        if relative_rotation == 0 {
            return;
        }

        match self.game_phase {
            GamePhase::MoveBlock => {
                // we are always updating the last block
                let active_block = self.blocks[self.block_count - 1];

                // O blocks can always rotate since rotating doesn't actually change their shape.
                if active_block.block_type == BlockType::O {
                    return;
                }

                let maybe_rotated_block = self.try_rotate_block(
                    self.block_positions[self.block_count - 1],
                    active_block,
                    relative_rotation,
                );
                if let Some((rotated_block, kicked_pos)) = maybe_rotated_block {
                    self.blocks[self.block_count - 1] = rotated_block;
                    self.block_positions[self.block_count - 1] = kicked_pos;
                }
            }
            GamePhase::StartNextBlock | GamePhase::GameOver => (),
        }
    }

    pub fn preview_block(&self) -> Block {
        self.next_block
    }

    #[cfg(test)]
    pub fn active_block(&self) -> Option<(Block, Cell)> {
        if self.block_count > 0 {
            let last_block_id = self.block_count - 1;
            Some((
                self.blocks[last_block_id],
                self.block_positions[last_block_id],
            ))
        } else {
            None
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.game_phase == GamePhase::GameOver
    }

    #[cfg(test)]
    pub fn for_each_settled_piece<F>(&self, mut op: F)
    where
        F: FnMut(BlockType, Cell),
    {
        for (block, block_pos) in self.blocks[0..self.block_count - 1]
            .iter()
            .zip(self.block_positions[0..self.block_count - 1].iter())
        {
            let cell_positions = translate_cells(&block.cells(), block_pos.y, block_pos.x);
            for cell_pos in &cell_positions {
                op(block.block_type, *cell_pos);
            }
        }
    }

    #[cfg(test)]
    pub fn get_settled_piece_count(&self) -> usize {
        std::cmp::max(self.block_count - 1, 0) * 4
    }

    fn can_block_move(&self, block_id: usize, horizontal_motion: i32) -> bool {
        assert_eq!(self.blocks.len(), self.block_positions.len());

        if horizontal_motion == 0 {
            return false;
        }

        let wall_to_check = if horizontal_motion < 0 {
            self.left_wall()
        } else {
            self.right_wall()
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

    fn left_wall(&self) -> Bound {
        Bound::LeftWall(-1)
    }

    fn right_wall(&self) -> Bound {
        Bound::RightWall(self.board_width)
    }

    fn floor(&self) -> Bound {
        Bound::Floor(self.board_height)
    }

    fn try_rotate_block(
        &self,
        original_block_pos: Cell,
        original_block: Block,
        relative_rotation: i32,
    ) -> Option<(Block, Cell)> {
        let rotated_block = original_block.rotate(relative_rotation);
        let kicks = original_block
            .rot
            .get_kick_attempts(original_block.block_type, rotated_block.rot);

        for kick in &kicks {
            let kicked_block_pos = Cell {
                x: original_block_pos.x + kick.x,
                y: original_block_pos.y + kick.y,
            };

            let do_blocks_collide_after_kick = do_blocks_collide(
                rotated_block,
                kicked_block_pos,
                &self.blocks[0..self.block_count - 1],
                &self.block_positions[0..self.block_count - 1],
                Vec2 { x: 0, y: 0 },
            );

            if do_blocks_collide_after_kick {
                continue;
            }

            if is_touching_bound(rotated_block, kicked_block_pos, self.floor())
                || is_touching_bound(rotated_block, kicked_block_pos, self.left_wall())
                || is_touching_bound(rotated_block, kicked_block_pos, self.right_wall())
            {
                continue;
            }

            return Some((rotated_block, kicked_block_pos));
        }

        None
    }

    fn has_block_landed(&self, block_id: usize) -> bool {
        assert_eq!(self.blocks.len(), self.block_positions.len());

        let is_touching_floor = is_touching_bound(
            self.blocks[block_id],
            self.block_positions[block_id],
            self.floor(),
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
