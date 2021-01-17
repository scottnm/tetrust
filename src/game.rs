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

pub struct GameState {
    board_width: i32,
    board_height: i32,
    block_type_rng: Box<dyn RangeRng<usize>>,
    settled_cells: Box<[Option<BlockType>]>,
    next_block: Block,
    active_block: Block,
    active_block_pos: Vec2,
    game_phase: GamePhase,
    score: usize,
    line_score: usize,
    delta_time: std::time::Duration,
}

impl GameState {
    pub fn new(
        board_width: i32,
        board_height: i32,
        mut block_type_rng: Box<dyn RangeRng<usize>>,
    ) -> GameState {
        let initial_block = Block::random(block_type_rng.as_mut());
        let max_blocks = (board_width * board_height) as usize;
        GameState {
            board_width,
            board_height,
            block_type_rng,
            settled_cells: (vec![None; max_blocks]).into_boxed_slice(),
            next_block: initial_block,
            active_block: Block::default(), // this block will be immediately replaced
            active_block_pos: Vec2::zero(),
            game_phase: GamePhase::StartNextBlock,
            score: 0,
            line_score: 0,
            delta_time: std::time::Duration::from_millis(0),
        }
    }

    #[cfg(test)]
    pub fn make_from_seed(
        board: &[Vec<bool>],
        active_block: Block,
        active_block_pos: Vec2,
        score: usize,
        line_score: usize,
        block_type_rng: Box<dyn RangeRng<usize>>,
    ) -> Self {
        assert!(!board.is_empty());
        let width = board[0].len();

        let max_blocks = board.len() * board[0].len();

        let mut settled_cells = vec![None; max_blocks];
        for (row_index, row) in board.iter().enumerate() {
            for (col_index, cell) in row.iter().enumerate() {
                if *cell {
                    settled_cells[width * row_index + col_index] = Some(BlockType::I);
                }
            }
        }

        GameState {
            board_width: width as i32,
            board_height: board.len() as i32,
            block_type_rng,
            settled_cells: settled_cells.into_boxed_slice(),
            next_block: Block::default(),
            active_block,
            active_block_pos,
            game_phase: GamePhase::MoveBlock,
            score,
            line_score,
            delta_time: std::time::Duration::from_millis(0),
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

    pub fn update(&mut self, delta_time: std::time::Duration) {
        self.add_time(delta_time);
        while self.consume_next_tick() {
            match self.game_phase {
                // Add a new block to the top of the board
                GamePhase::StartNextBlock => {
                    let new_next_block = Block::random(self.block_type_rng.as_mut());
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
                            self.settle_active_block();

                            let num_rows_cleared = self.clear_rows(self.active_block_pos.y);
                            self.score += Self::calculate_clear_score(num_rows_cleared);
                            self.line_score += num_rows_cleared;

                            self.game_phase = GamePhase::StartNextBlock
                        }
                    } else {
                        self.move_active_block_down();
                    }
                }

                // The game is over; NOOP
                GamePhase::GameOver => (),
            }
        }
    }

    pub fn move_active_block_horizontal(&mut self, horizontal_motion: i32) {
        match self.game_phase {
            GamePhase::MoveBlock => {
                if self.can_active_block_move(horizontal_motion) {
                    self.active_block_pos.x += horizontal_motion;
                }
            }
            GamePhase::StartNextBlock | GamePhase::GameOver => (),
        }
    }

    pub fn move_active_block_down(&mut self) {
        self.active_block_pos.y += 1;
    }

    pub fn quick_drop(&mut self) {
        while !self.has_active_block_landed() {
            self.move_active_block_down();
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

    pub fn score(&self) -> usize {
        self.score
    }

    pub fn level(&self) -> usize {
        // each level is cleared by clearing 5 lines
        (self.line_score / 5) + 1
    }

    pub fn for_each_settled_piece<F>(&self, mut op: F)
    where
        F: FnMut(BlockType, Vec2),
    {
        for row in 0..self.board_height {
            for col in 0..self.board_width {
                if let Some(settled_cell) = self.settled_cells[self.cell_index(col, row)] {
                    op(settled_cell, Vec2 { x: col, y: row });
                }
            }
        }
    }

    #[cfg(test)]
    pub fn get_settled_piece_count(&self) -> usize {
        self.settled_cells.iter().filter(|c| c.is_some()).count()
    }

    fn add_time(&mut self, delta_time: std::time::Duration) {
        self.delta_time += delta_time
    }

    fn get_move_period(&self) -> std::time::Duration {
        // TODO: should this difficulty ramp be hand-tuned?
        let clamped_level_index = std::cmp::min(self.level() - 1, 10) as u64;
        std::time::Duration::from_millis(250 - (15 * clamped_level_index))
    }

    fn consume_next_tick(&mut self) -> bool {
        let move_period = self.get_move_period();
        if self.delta_time >= move_period {
            self.delta_time -= move_period;
            return true;
        }

        false
    }

    fn cell_index(&self, x: i32, y: i32) -> usize {
        (self.board_width * y + x) as usize
    }

    fn can_active_block_move(&self, horizontal_motion: i32) -> bool {
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

        let will_block_collide = self.does_block_collide_with_settled_blocks(
            self.active_block,
            self.active_block_pos,
            motion_vec,
        );

        !will_block_collide
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

            let does_block_collide_after_kick = self.does_block_collide_with_settled_blocks(
                rotated_block,
                kicked_block_pos,
                Vec2::zero(),
            );

            if does_block_collide_after_kick {
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
        let is_touching_floor =
            is_touching_bound(self.active_block, self.active_block_pos, self.floor());

        if is_touching_floor {
            return true;
        }

        let does_block_collide_below = self.does_block_collide_with_settled_blocks(
            self.active_block,
            self.active_block_pos,
            Vec2 { x: 0, y: 1 },
        );

        does_block_collide_below
    }

    fn settle_active_block(&mut self) {
        for cell in &translate_cells(
            &self.active_block.cells(),
            self.active_block_pos.y,
            self.active_block_pos.x,
        ) {
            let cell = &mut self.settled_cells[self.cell_index(cell.x, cell.y)];
            assert!(cell.is_none());
            *cell = Some(self.active_block.block_type);
        }
    }

    fn clear_rows(&mut self, start_row: i32) -> usize {
        let mut rows_to_check = [false, false, false, false];
        for cell in &self.active_block.cells() {
            rows_to_check[cell.y as usize] = true;
        }

        let mut rows_cleared = 0;
        for row in rows_to_check
            .iter()
            .enumerate()
            .filter(|(_, check_row)| **check_row)
            .map(|(i, _)| i as i32 + start_row)
        {
            let row_cleared = self.try_clear_row(row);
            if row_cleared {
                rows_cleared += 1;
            }
        }

        rows_cleared
    }

    fn try_clear_row(&mut self, row: i32) -> bool {
        let row_start = self.cell_index(0, row);
        let row_end = self.cell_index(self.board_width, row);
        let cells_in_row_count = self.settled_cells[row_start..row_end]
            .iter()
            .filter(|c| c.is_some())
            .count();

        if cells_in_row_count != self.board_width as usize {
            return false;
        }

        // Clear out the row
        for cell in self.settled_cells[row_start..row_end].iter_mut() {
            *cell = None;
        }

        // TODO: this could be more efficient if instead of bubbling up we just did a single shift copy
        // shift the cleared out row up
        for preceding_row in (0..row).rev() {
            for col in 0..self.board_width {
                let preceding_row_cell_index = self.cell_index(col, preceding_row);
                let next_row_cell_index = self.cell_index(col, preceding_row + 1);
                self.settled_cells
                    .swap(preceding_row_cell_index, next_row_cell_index);
            }
        }

        true
    }

    fn calculate_clear_score(num_cleared_lines: usize) -> usize {
        match num_cleared_lines {
            0 => 0,
            1 => 40,
            2 => 100,
            3 => 300,
            4 => 1200,
            _ => panic!("There is no way to clear more than 4 lines at once!"),
        }
    }

    fn does_block_collide_with_settled_blocks(
        &self,
        block: Block,
        block_pos: Vec2,
        move_vector: Vec2,
    ) -> bool {
        let moved_block_cells = translate_cells(
            &block.cells(),
            block_pos.y + move_vector.y,
            block_pos.x + move_vector.x,
        );

        fn in_ex_range(v: i32, lower: i32, upper: i32) -> bool {
            v >= lower && v < upper
        }
        for moved_block_cell in &moved_block_cells {
            if !in_ex_range(moved_block_cell.x, 0, self.board_width)
                || !in_ex_range(moved_block_cell.y, 0, self.board_height)
            {
                continue;
            }

            if self.settled_cells[self.cell_index(moved_block_cell.x, moved_block_cell.y)].is_some()
            {
                return true;
            }
        }

        false
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
