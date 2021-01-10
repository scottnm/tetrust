use crate::block::*;
use crate::randwrapper::*;
use crate::util::*;

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
    settled_block_count: usize,
    // TODO: rename to something better (settled_cells?)
    settled_blocks: Box<[BlockType]>,
    // TODO: don't have the settled_block_positions and settled_blocks vecs be independent.
    //       make the position info implicit from the position in the settled_blocks arr
    settled_block_positions: Box<[Vec2]>,
    next_block: Block,
    active_block: Block,
    active_block_pos: Vec2,
    game_phase: GamePhase,
    score: i32,
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
            settled_block_count: 0,
            settled_blocks: (vec![BlockType::I; max_blocks]).into_boxed_slice(),
            settled_block_positions: (vec![Vec2::zero(); max_blocks]).into_boxed_slice(),
            next_block: initial_block,
            active_block: Block::default(), // this block will be immediately replaced
            active_block_pos: Vec2::zero(),
            game_phase: GamePhase::StartNextBlock,
            score: 0,
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
                assert_eq!(
                    self.settled_blocks.len(),
                    self.settled_block_positions.len()
                );
                assert!(self.settled_block_count < self.settled_blocks.len());

                let new_next_block = Block::random(&mut self.block_type_rng);
                let new_active_block = std::mem::replace(&mut self.next_block, new_next_block);

                let start_col =
                    (self.board_width - new_active_block.width()) / 2 - new_active_block.left();
                let start_row = -new_active_block.height();

                let new_active_block_pos = Vec2 {
                    x: start_col,
                    y: start_row,
                };

                self.active_block = new_active_block;
                self.active_block_pos = new_active_block_pos;
                self.game_phase = GamePhase::MoveBlock;
            }

            // Move the latest block down across the board
            GamePhase::MoveBlock => {
                if self.has_active_block_landed() {
                    let is_block_above_board = self.active_block_pos.y < 0;
                    if is_block_above_board {
                        self.game_phase = GamePhase::GameOver
                    } else {
                        // Bake the active block into the settled cell grid.
                        let mut rows_to_check = [false, false, false, false];
                        for cell in &self.active_block.cells() {
                            rows_to_check[cell.y as usize] = true;
                        }

                        self.settle_active_block();

                        let active_block_y_offset = self.active_block_pos.y;
                        for row in rows_to_check
                            .iter()
                            .enumerate()
                            .filter(|(_, check_row)| **check_row)
                            .map(|(i, _)| i as i32 + active_block_y_offset)
                        {
                            let row_cleared = self.try_clear_row(row);
                            if row_cleared {
                                // TODO: what are the actual scoring rules?
                                println!("Score!");
                                self.score += 1;
                            }
                        }

                        self.game_phase = GamePhase::StartNextBlock
                    }
                } else {
                    self.active_block_pos.y += 1;
                }
            }

            // The game is over; NOOP
            GamePhase::GameOver => (),
        }
    }

    pub fn move_block_horizontal(&mut self, horizontal_motion: i32) {
        match self.game_phase {
            GamePhase::MoveBlock => {
                if self.can_active_block_move(horizontal_motion) {
                    self.active_block_pos.x += horizontal_motion;
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
                let active_block = self.active_block;

                // O blocks can always rotate since rotating doesn't actually change their shape.
                if active_block.block_type == BlockType::O {
                    return;
                }

                let maybe_rotated_block = self.try_rotate_active_block(relative_rotation);
                if let Some((rotated_block, kicked_pos)) = maybe_rotated_block {
                    self.active_block = rotated_block;
                    self.active_block_pos = kicked_pos;
                }
            }
            GamePhase::StartNextBlock | GamePhase::GameOver => (),
        }
    }

    pub fn preview_block(&self) -> Block {
        self.next_block
    }

    // TODO: maybe active_block should actually be represented by an option and force the unwrap check in places
    pub fn active_block(&self) -> Option<(Block, Vec2)> {
        // If we are in the "StartNextBlock" phase it means that we've just placed our previous active block
        if self.game_phase == GamePhase::StartNextBlock {
            None
        } else {
            Some((self.active_block, self.active_block_pos))
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.game_phase == GamePhase::GameOver
    }

    pub fn for_each_settled_piece<F>(&self, mut op: F)
    where
        F: FnMut(BlockType, Vec2),
    {
        for (settled_cell, settled_cell_pos) in self.settled_blocks[0..self.settled_block_count]
            .iter()
            .zip(self.settled_block_positions.iter())
        {
            op(*settled_cell, *settled_cell_pos);
        }
    }

    #[cfg(test)]
    pub fn get_settled_piece_count(&self) -> usize {
        self.settled_block_count
    }

    fn can_active_block_move(&self, horizontal_motion: i32) -> bool {
        assert_eq!(
            self.settled_blocks.len(),
            self.settled_block_positions.len()
        );

        if horizontal_motion == 0 {
            return false;
        }

        let wall_to_check = if horizontal_motion < 0 {
            self.left_wall()
        } else {
            self.right_wall()
        };

        let is_touching_wall =
            is_touching_bound(self.active_block, self.active_block_pos, wall_to_check);

        if is_touching_wall {
            return false;
        }

        let motion_vec = Vec2 {
            x: horizontal_motion,
            y: 0,
        };

        let do_blocks_collide_side = do_blocks_collide(
            self.active_block,
            self.active_block_pos,
            &self.settled_block_positions[0..self.settled_block_count],
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

    fn try_rotate_active_block(&self, relative_rotation: i32) -> Option<(Block, Vec2)> {
        let original_block = self.active_block;
        let original_block_pos = self.active_block_pos;

        let rotated_block = original_block.rotate(relative_rotation);
        let kicks = original_block
            .rot
            .get_kick_attempts(original_block.block_type, rotated_block.rot);

        for kick in &kicks {
            let kicked_block_pos = Vec2 {
                x: original_block_pos.x + kick.x,
                y: original_block_pos.y + kick.y,
            };

            let do_blocks_collide_after_kick = do_blocks_collide(
                rotated_block,
                kicked_block_pos,
                &self.settled_block_positions[0..self.settled_block_count],
                Vec2::zero(),
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

    fn has_active_block_landed(&self) -> bool {
        assert_eq!(
            self.settled_blocks.len(),
            self.settled_block_positions.len()
        );

        let is_touching_floor =
            is_touching_bound(self.active_block, self.active_block_pos, self.floor());

        if is_touching_floor {
            return true;
        }

        let do_blocks_collide_below = do_blocks_collide(
            self.active_block,
            self.active_block_pos,
            &self.settled_block_positions[0..self.settled_block_count],
            Vec2 { x: 0, y: 1 },
        );

        do_blocks_collide_below
    }

    fn settle_active_block(&mut self) {
        for cell in &translate_cells(
            &self.active_block.cells(),
            self.active_block_pos.y,
            self.active_block_pos.x,
        ) {
            self.settled_block_positions[self.settled_block_count] = *cell;
            self.settled_blocks[self.settled_block_count] = self.active_block.block_type;
            self.settled_block_count += 1;
        }
    }

    fn try_clear_row(&mut self, row: i32) -> bool {
        let cells_in_row_count = self.settled_block_positions[0..self.settled_block_count]
            .iter()
            .filter(|pos| pos.y == row)
            .count();

        if cells_in_row_count != self.board_width as usize {
            return false;
        }

        // if every cell in a row was filled, clear it!
        for i in (0..self.settled_block_count).rev() {
            if self.settled_block_positions[i].y == row {
                self.settled_block_positions
                    .swap(i, self.settled_block_count - 1);
                self.settled_blocks.swap(i, self.settled_block_count - 1);
                self.settled_block_count -= 1;
            }
        }

        // after clearing the row, shift each position that was above that row down 1
        for pos in self.settled_block_positions.iter_mut() {
            if pos.y < row {
                pos.y += 1;
            }
        }

        true
    }
}

fn translate_cells(cells: &[Vec2; 4], row_translation: i32, col_translation: i32) -> [Vec2; 4] {
    let mut translated_cells: [Vec2; 4] = *cells;
    for cell_index in 0..translated_cells.len() {
        translated_cells[cell_index].y += row_translation;
        translated_cells[cell_index].x += col_translation;
    }

    translated_cells
}

fn is_touching_bound(block: Block, block_pos: Vec2, bound: Bound) -> bool {
    match bound {
        Bound::Floor(floor) => block.top() + block_pos.y + block.height() >= floor,
        Bound::LeftWall(left) => block.left() + block_pos.x <= left + 1,
        Bound::RightWall(right) => block.left() + block_pos.x + block.width() >= right,
    }
}

fn do_blocks_collide(
    block: Block,
    block_pos: Vec2,
    settled_cell_positions: &[Vec2],
    move_vector: Vec2,
) -> bool {
    let moved_block_cells = translate_cells(
        &block.cells(),
        block_pos.y + move_vector.y,
        block_pos.x + move_vector.x,
    );

    for other_cell in settled_cell_positions {
        for moved_cell in moved_block_cells.iter() {
            if moved_cell == other_cell {
                return true;
            }
        }
    }

    false
}
