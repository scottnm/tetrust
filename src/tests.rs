// TODO: need to add tests for rotation kicks
#[cfg(test)]
mod tests {
    use crate::block::*;
    use crate::game::*;
    use crate::leaderboard::*;
    use crate::randwrapper::*;
    use crate::util::*;

    struct SelfDestructingFile {
        path: std::path::PathBuf,
    }

    impl SelfDestructingFile {
        fn new<P: AsRef<std::path::Path>>(path: P, data: &[u8]) -> Self {
            let file = std::fs::File::create(&path).unwrap();
            let mut writer = std::io::LineWriter::new(file);
            use std::io::Write;
            writer.write_all(data).unwrap();

            let mut pathbuf = std::path::PathBuf::new();
            pathbuf.push(path);
            SelfDestructingFile { path: pathbuf }
        }
    }

    impl Drop for SelfDestructingFile {
        fn drop(&mut self) {
            std::fs::remove_file(self.path.as_path()).unwrap();
        }
    }

    fn tick(game_state: &mut GameState) {
        const STEADY_TICK: std::time::Duration = std::time::Duration::from_millis(250);
        game_state.update(STEADY_TICK);
    }

    fn default_test_board(block_type_rng: Box<dyn RangeRng<usize>>) -> GameState {
        const TEST_BOARD_WIDTH: i32 = 20;
        const TEST_BOARD_HEIGHT: i32 = 30;
        GameState::new(TEST_BOARD_WIDTH, TEST_BOARD_HEIGHT, block_type_rng)
    }

    fn test_board_from_seed(
        board: &[Vec<bool>],
        active_block: Block,
        active_block_pos: Vec2,
        score: usize,
        line_score: usize,
    ) -> GameState {
        GameState::make_from_seed(
            board,
            active_block,
            active_block_pos,
            score,
            line_score,
            Box::new(ThreadRangeRng::new()),
        )
    }

    #[allow(dead_code)]
    fn print_board<T: RangeRng<usize>>(game_state: &GameState) {
        let mut board = vec![vec!['`'; game_state.width() as usize]; game_state.height() as usize];

        let fill_in_board = |block_type: BlockType, pos: Vec2| {
            board[pos.y as usize][pos.x as usize] = block_type.sprite_char();
        };

        game_state.for_each_settled_piece(fill_in_board);

        for _ in 0..game_state.width() {
            print!("-");
        }
        print!("\n");

        for row in &board {
            for cell in row {
                print!("{}", cell);
            }
            print!("\n");
        }

        for _ in 0..game_state.width() {
            print!("-");
        }
        print!("\n");
    }

    fn gen_wrapper<T: PartialOrd>(rng: &mut dyn RangeRng<T>, lower: T, upper: T) -> T {
        rng.gen_range(lower, upper)
    }

    #[test]
    fn test_thread_random() {
        // this test is mostly here to verify that things compile
        let mut rng = ThreadRangeRng::new();
        let first_value = rng.gen_range(0, 10);
        let next_value = gen_wrapper(&mut rng, 10, 20);
        assert_ne!(first_value, next_value);
    }

    #[test]
    fn test_single_value_random() {
        let mut rng = mocks::SingleValueRangeRng::new(10i32);
        let first_value = rng.gen_range(0, 100);
        for _ in 1..10 {
            let next_value = gen_wrapper(&mut rng, 0, 100);
            assert_eq!(first_value, next_value);
        }
    }

    #[test]
    fn test_game_over_is_final() {
        // This test gets the tetris board to a game over state and verifies that further game
        // ticks will not change the game state.

        let mut game_state = default_test_board(Box::new(ThreadRangeRng::new()));
        while !game_state.is_game_over() {
            tick(&mut game_state);
        }

        let expected_final_settled_piece_count = game_state.get_settled_piece_count();
        for _ in 0..5 {
            tick(&mut game_state);
        }
        assert_eq!(
            game_state.get_settled_piece_count(),
            expected_final_settled_piece_count
        );
        assert!(game_state.is_game_over());
    }

    #[test]
    fn test_expected_game_over() {
        const EXPECTED_FINAL_BOARD: [[bool; 4]; 16] = [
            [false, true, true, false],
            [false, true, true, false],
            [false, true, true, false],
            [false, true, true, false],
            [false, true, true, false],
            [false, true, true, false],
            [false, true, true, false],
            [false, true, true, false],
            [false, true, true, false],
            [false, true, true, false],
            [false, true, true, false],
            [false, true, true, false],
            [false, true, true, false],
            [false, true, true, false],
            [false, true, true, false],
            [false, true, true, false],
        ];

        const TEST_BOARD_WIDTH: i32 = EXPECTED_FINAL_BOARD[0].len() as i32;
        const TEST_BOARD_HEIGHT: i32 = EXPECTED_FINAL_BOARD.len() as i32;

        // This test generates only 'O' pieces perfectly stacking on each on the board and
        // verifies the end state
        let mut game_state = GameState::new(
            TEST_BOARD_WIDTH,
            TEST_BOARD_HEIGHT,
            Box::new(mocks::SingleValueRangeRng::new(BlockType::O as usize)),
        );

        while !game_state.is_game_over() {
            tick(&mut game_state);
        }

        /*
        // each 'O' piece will start horizontal and they all will perfectly stack
        const FINAL_BLOCK_COUNT: usize = (TEST_BOARD_HEIGHT as usize / 2) + 1;
        const FINAL_BLOCK_ID: usize = FINAL_BLOCK_COUNT - 1;
        assert_eq!(game_state.block_count(), FINAL_BLOCK_COUNT);
        assert!(has_block_landed_oob(&game_state, FINAL_BLOCK_ID));
        */

        // Verify all settled pieces are inbounds
        let expected_final_settled_piece_count: usize = EXPECTED_FINAL_BOARD
            .iter()
            .map(|row| row.iter().filter(|col| **col).count())
            .sum();
        assert_eq!(
            game_state.get_settled_piece_count(),
            expected_final_settled_piece_count
        );

        let mut out_of_bounds_settled_pieces = vec![];
        let collect_out_of_bounds_settled_pieces = |block_type: BlockType, pos: Vec2| {
            if pos.x < 0 || pos.x >= TEST_BOARD_WIDTH || pos.y < 0 || pos.y >= TEST_BOARD_HEIGHT {
                out_of_bounds_settled_pieces.push((block_type, pos));
            }
        };

        game_state.for_each_settled_piece(collect_out_of_bounds_settled_pieces);
        assert_eq!(out_of_bounds_settled_pieces, vec![]);

        let mut final_board: [[bool; 4]; 16] = [[false, false, false, false]; 16];
        let collect_final_board = |_, pos: Vec2| {
            final_board[pos.y as usize][pos.x as usize] = true;
        };
        game_state.for_each_settled_piece(collect_final_board);
        assert_eq!(final_board, EXPECTED_FINAL_BOARD);

        // Verify the out of bound active piece
        let maybe_active_block = game_state.active_block();
        assert!(maybe_active_block.is_some());

        let (_, active_block_pos) = maybe_active_block.unwrap();
        assert_eq!(active_block_pos, Vec2 { x: 0, y: -2 });
    }

    fn active_block_distance_to_left_wall(game_state: &GameState) -> i32 {
        let block = game_state.active_block().unwrap();
        let active_block_pos = block.1;
        active_block_pos.x + block.0.left()
    }

    fn active_block_distance_to_right_wall(game_state: &GameState) -> i32 {
        let block = game_state.active_block().unwrap();
        let active_block_width = block.0.width();
        let active_block_pos = block.1;
        let active_block_rightmost_cell =
            active_block_pos.x + block.0.left() + active_block_width - 1;

        (game_state.width() - 1) - active_block_rightmost_cell
    }

    fn drop_active_block(game_state: &mut GameState) {
        game_state.quick_drop();
        tick(game_state);
    }

    #[test]
    fn test_lr_collisions() {
        let mut game_state = default_test_board(Box::new(mocks::SingleValueRangeRng::new(
            BlockType::S as usize,
        )));

        // generate first block
        assert!(game_state.active_block().is_none());
        tick(&mut game_state);
        assert!(game_state.active_block().is_some());

        // verify that a block can be moved left which will change its position
        let distance_to_left_wall = active_block_distance_to_left_wall(&game_state);
        for _ in 0..distance_to_left_wall {
            game_state.move_active_block_horizontal(-1);
            assert_ne!(
                distance_to_left_wall,
                active_block_distance_to_left_wall(&game_state)
            );
        }

        // verify that once a block collides with the left wall it can't move left any further but
        // it can move right
        assert_eq!(active_block_distance_to_left_wall(&game_state), 0);
        game_state.move_active_block_horizontal(-1);
        assert_eq!(active_block_distance_to_left_wall(&game_state), 0);
        game_state.move_active_block_horizontal(1);
        assert_eq!(active_block_distance_to_left_wall(&game_state), 1);
        drop_active_block(&mut game_state);

        // verify that a block can be moved right which will change its position
        assert_eq!(game_state.get_settled_piece_count(), 4); // verify that the first piece has settled
        tick(&mut game_state); // generate the next block
        let distance_to_right_wall = active_block_distance_to_right_wall(&game_state);
        for _ in 0..distance_to_right_wall {
            game_state.move_active_block_horizontal(1);
            assert_ne!(
                distance_to_right_wall,
                active_block_distance_to_right_wall(&game_state)
            );
        }
        // verify that once a block collides with the right wall it can't move right any further
        // but it can move left
        assert_eq!(active_block_distance_to_right_wall(&game_state), 0);
        game_state.move_active_block_horizontal(1);
        assert_eq!(active_block_distance_to_right_wall(&game_state), 0);
        game_state.move_active_block_horizontal(-1);
        assert_eq!(active_block_distance_to_right_wall(&game_state), 1);
        drop_active_block(&mut game_state);

        // generate a stack of blocks in the middle
        //      xx
        //     xx
        //      oo
        //     oo
        //      xx
        //     xx
        for _ in 0..3 {
            drop_active_block(&mut game_state);
        }

        // move latest block off to the right
        //     =>  $$
        //     => $$
        //      xx
        //     xx
        //      oo
        //     oo
        //      xx
        //     xx
        let (active_block, _) = game_state.active_block().unwrap();
        for _ in 0..active_block.width() {
            game_state.move_active_block_horizontal(1);
        }

        // drop the latest block until its 1 block away from touching the bottom
        //      xx  |
        //     xx   V
        //      oo
        //     oo  $$
        //      xx$$
        //     xx
        for _ in 0..(game_state.height() - 1) {
            tick(&mut game_state);
        }

        let (active_block, active_block_pos) = game_state.active_block().unwrap();
        assert_eq!(
            active_block_pos.y,
            game_state.height() - 1 - active_block.height()
        );

        // can't move the last block to the left because of collision
        let left_wall_distance_before = active_block_distance_to_left_wall(&game_state);
        game_state.move_active_block_horizontal(-1);
        assert_eq!(
            active_block_distance_to_left_wall(&game_state),
            left_wall_distance_before
        );

        // drop the last block 1 more time so it is touching the bottom
        //      xx  |
        //     xx   V
        //      oo
        //     oo
        //      xx $$
        //     xx $$
        tick(&mut game_state);
        let (active_block, active_block_pos) = game_state.active_block().unwrap();
        assert_eq!(
            active_block_pos.y,
            game_state.height() - active_block.height()
        );

        // the last block can now move left
        //      xx
        //     xx
        //      oo
        //     oo
        //      xx$$ <=
        //     xx$$  <=
        game_state.move_active_block_horizontal(-1);
        assert_eq!(
            active_block_distance_to_left_wall(&game_state),
            left_wall_distance_before - 1
        );

        // verify the game can be finished
        while !game_state.is_game_over() {
            tick(&mut game_state);
        }
    }

    #[test]
    fn test_preview_block() {
        let mut preview_block = BlockType::T;
        let mut active_block = BlockType::S;

        // This test generates only 'I' pieces on the far-left column of the board and verifies the
        // number of pieces it takes to fill up the board
        let mut game_state = default_test_board(Box::new(mocks::SequenceRangeRng::new(&[
            preview_block as usize,
            active_block as usize,
        ])));
        assert!(game_state.active_block().is_none());
        assert_eq!(game_state.preview_block().block_type, preview_block);

        while !game_state.is_game_over() {
            // tick the game at least once after making the last block fall so that the preview
            // block becomes the active block and we get a new preview block
            tick(&mut game_state);

            // Verify the preview and active lbock have swapped places
            std::mem::swap(&mut preview_block, &mut active_block);
            assert_eq!(game_state.preview_block().block_type, preview_block);
            assert_eq!(
                game_state.active_block().unwrap().0.block_type,
                active_block
            );

            drop_active_block(&mut game_state);
        }
    }

    #[test]
    fn test_score_1_line() {
        let board = [
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![true, false, true, true, true, true],
        ];

        let active_block = Block {
            rot: Rotation::Rot2,
            block_type: BlockType::T,
        };
        let active_block_pos = Vec2::zero();

        let start_score = 120;
        let mut game_state =
            test_board_from_seed(&board, active_block, active_block_pos, start_score, 3);
        assert_eq!(game_state.score(), start_score);

        drop_active_block(&mut game_state);
        assert_eq!(game_state.score(), start_score + 40);
    }

    #[test]
    fn test_score_2_line() {
        let board = [
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![true, false, true, true, true, true],
            vec![true, false, true, true, true, true],
        ];

        let active_block = Block {
            rot: Rotation::Rot3,
            block_type: BlockType::L,
        };
        let active_block_pos = Vec2::zero();

        let start_score = 120;
        let mut game_state =
            test_board_from_seed(&board, active_block, active_block_pos, start_score, 3);
        assert_eq!(game_state.score(), start_score);

        drop_active_block(&mut game_state);
        assert_eq!(game_state.score(), start_score + 100);
    }

    #[test]
    fn test_score_3_line() {
        let board = [
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![true, false, true, true, true, true],
            vec![true, false, true, true, true, true],
            vec![true, false, true, true, true, true],
        ];

        let active_block = Block {
            rot: Rotation::Rot3,
            block_type: BlockType::I,
        };
        let active_block_pos = Vec2::zero();

        let start_score = 120;
        let mut game_state =
            test_board_from_seed(&board, active_block, active_block_pos, start_score, 3);
        assert_eq!(game_state.score(), start_score);

        drop_active_block(&mut game_state);
        assert_eq!(game_state.score(), start_score + 300);
    }

    #[test]
    fn test_score_4_line() {
        let board = [
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![true, false, true, true, true, true],
            vec![true, false, true, true, true, true],
            vec![true, false, true, true, true, true],
            vec![true, false, true, true, true, true],
        ];

        let active_block = Block {
            rot: Rotation::Rot3,
            block_type: BlockType::I,
        };
        let active_block_pos = Vec2::zero();

        let start_score = 120;
        let mut game_state =
            test_board_from_seed(&board, active_block, active_block_pos, start_score, 3);
        assert_eq!(game_state.score(), start_score);

        drop_active_block(&mut game_state);
        assert_eq!(game_state.score(), start_score + 1200);
    }

    #[test]
    fn test_level_calculation_1() {
        let board = [
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![true, false, true, true, true, true],
            vec![true, false, true, true, true, true],
            vec![true, false, true, true, true, true],
            vec![true, false, true, true, true, true],
        ];

        let active_block = Block {
            rot: Rotation::Rot3,
            block_type: BlockType::I,
        };
        let active_block_pos = Vec2::zero();

        let start_score = 160;
        let mut game_state =
            test_board_from_seed(&board, active_block, active_block_pos, start_score, 4);

        let start_level = 1;
        assert_eq!(game_state.score(), start_score);
        assert_eq!(game_state.level(), start_level);

        drop_active_block(&mut game_state);
        assert!(game_state.score() > start_score);
        assert_eq!(game_state.level(), start_level + 1);
    }

    #[test]
    fn test_level_calculation_2() {
        let board = [
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![false, false, false, false, false, false],
            vec![true, false, true, true, true, true],
            vec![true, false, true, true, true, true],
            vec![true, false, true, true, true, true],
            vec![true, false, true, true, true, true],
        ];

        let active_block = Block {
            rot: Rotation::Rot3,
            block_type: BlockType::I,
        };
        let active_block_pos = Vec2::zero();

        let start_score = 2400 + 40; // 9 lines = 2 tetrises + a single line clear
        let mut game_state =
            test_board_from_seed(&board, active_block, active_block_pos, start_score, 9);

        let start_level = 2;
        assert_eq!(game_state.score(), start_score);
        assert_eq!(game_state.level(), start_level);

        drop_active_block(&mut game_state);
        assert!(game_state.score() > start_score);
        assert_eq!(game_state.level(), start_level + 1);
    }

    #[test]
    fn test_leaderboard() {
        let mut leaderboard = Leaderboard::new();

        struct TestData {
            score: usize,
            expected_pos: usize,
        }

        let test_values = [
            TestData {
                score: 700,
                expected_pos: 0,
            },
            TestData {
                score: 800,
                expected_pos: 0,
            },
            TestData {
                score: 750,
                expected_pos: 1,
            },
            TestData {
                score: 500,
                expected_pos: 3,
            },
            TestData {
                score: 200,
                expected_pos: 4,
            },
            TestData {
                score: 250,
                expected_pos: 4,
            },
            TestData {
                score: 300,
                expected_pos: 4,
            },
            TestData {
                score: 775,
                expected_pos: 1,
            },
            TestData {
                score: 100,
                expected_pos: 8,
            },
            TestData {
                score: 50,
                expected_pos: 9,
            },
            TestData {
                score: 200,
                expected_pos: 8,
            },
        ];

        fn name_from_index(i: usize) -> String {
            format!("n{:02}", i)
        }

        let final_score_indices: [usize; Leaderboard::max_entries()] =
            [1, 7, 2, 0, 3, 6, 5, 4, 10, 8];
        let expected_entries: Vec<LeaderboardEntry> = final_score_indices
            .iter()
            .map(|final_score_index| LeaderboardEntry {
                name: name_from_index(*final_score_index),
                score: test_values[*final_score_index].score,
            })
            .collect();
        let expected_final_leaderboard = Leaderboard::from_raw(expected_entries);

        for (i, test) in test_values.iter().enumerate() {
            let place = leaderboard.get_place_on_leaderboard(test.score);
            assert_eq!(place, Some(test.expected_pos));

            let name = name_from_index(i);
            leaderboard.add_score(name, test.score);
        }

        // Verify that there are 10 entries on the leaderboard
        assert!(leaderboard.entry(final_score_indices.len() - 1).is_some());

        // Check each entry on the leaderboard
        assert_eq!(leaderboard, expected_final_leaderboard);
    }

    #[test]
    #[should_panic]
    fn test_leaderboard_add_fails() {
        const TEST_SCORE: usize = 100;

        let mut leaderboard = Leaderboard::new();

        for _ in 0..Leaderboard::max_entries() {
            assert!(leaderboard.get_place_on_leaderboard(TEST_SCORE).is_some());
            leaderboard.add_score("test", TEST_SCORE);
        }

        assert!(leaderboard
            .get_place_on_leaderboard(TEST_SCORE - 1)
            .is_none());
    }

    #[test]
    fn test_new_leaderboard_file_missing() {
        let leaderboard_from_file = Leaderboard::load("data/does_not_exist.leaderboard");
        assert!(leaderboard_from_file.is_err());
    }

    #[test]
    fn test_new_leaderboard_from_file() {
        let mut dummy_leaderboard = Leaderboard::new();
        dummy_leaderboard.add_score("ld5", 700);
        dummy_leaderboard.add_score("ld3", 900);
        dummy_leaderboard.add_score("ld4", 800);
        dummy_leaderboard.add_score("ld1", 1000);
        dummy_leaderboard.add_score("ld2", 1000);

        let dummy_file_name = "data/test_leaderboard";
        let _dummy_file = SelfDestructingFile::new(dummy_file_name, &dummy_leaderboard.serialize());
        let leaderboard_from_file = Leaderboard::load(dummy_file_name).unwrap();
        assert_eq!(leaderboard_from_file, dummy_leaderboard);
    }
}
