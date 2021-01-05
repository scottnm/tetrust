#[cfg(test)]
mod tests {
    use crate::block::*;
    use crate::game::*;
    use crate::randwrapper::*;

    #[allow(dead_code)] // ignore dead code because this is primarily used for debugging tests
    fn log_blocks<T, U>(game: &GameState<T, U>)
    where
        T: RangeRng<usize>,
        U: RangeRng<i32>,
    {
        for block_id in 0..game.block_count() {
            let block = game.block(block_id);
            println!("{}: {:?}", block_id, block)
        }
    }

    fn has_block_landed_oob<T, U>(game: &GameState<T, U>, block_id: usize) -> bool
    where
        T: RangeRng<usize>,
        U: RangeRng<i32>,
    {
        let block_pos = game.block(block_id).0;
        game.has_block_landed(block_id) && block_pos.0 < 0
    }

    fn has_any_block_landed_oob<T, U>(game: &GameState<T, U>) -> bool
    where
        T: RangeRng<usize>,
        U: RangeRng<i32>,
    {
        // NOTE (scottnm): iterate in reverse order because currently the rules require the last block to be the one
        // that exceeded the board.
        for block_id in (0..game.block_count()).rev() {
            if has_block_landed_oob(game, block_id) {
                return true;
            }
        }

        false
    }

    const TEST_BOARD_WIDTH: i32 = 20;
    const TEST_BOARD_HEIGHT: i32 = 30;

    fn gen_wrapper<T: PartialOrd>(rng: &mut RangeRng<T>, lower: T, upper: T) -> T {
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

        let mut game_state = GameState::new(
            0,
            0,
            TEST_BOARD_WIDTH,
            TEST_BOARD_HEIGHT,
            ThreadRangeRng::new(),
            ThreadRangeRng::new(),
        );
        while !game_state.is_game_over() {
            game_state.tick();
        }

        let block_count = game_state.block_count();
        for _ in 0..5 {
            game_state.tick();
        }
        assert_eq!(game_state.block_count(), block_count);
        assert!(game_state.is_game_over());
    }

    #[test]
    fn test_game_over_on_board_exceeded() {
        // This test verifies that a game over only happens a block exceeds the board

        let mut game_state = GameState::new(
            0,
            0,
            TEST_BOARD_WIDTH,
            TEST_BOARD_HEIGHT,
            ThreadRangeRng::new(),
            ThreadRangeRng::new(),
        );
        while !game_state.is_game_over() {
            game_state.tick();
        }
        assert!(has_any_block_landed_oob(&game_state));
    }

    #[test]
    fn test_expected_game_over() {
        // This test generates only 'I' pieces on the far-left column of the board and verifies the
        // number of pieces it takes to fill up the board
        let mut game_state = GameState::new(
            0,
            0,
            TEST_BOARD_WIDTH,
            TEST_BOARD_HEIGHT,
            mocks::SingleValueRangeRng::new(BlockType::I as usize),
            mocks::SingleValueRangeRng::new(0 as i32),
        );
        while !game_state.is_game_over() {
            game_state.tick();
        }
        const FINAL_BLOCK_COUNT: usize = (TEST_BOARD_HEIGHT as usize / 4) + 1;
        const FINAL_BLOCK_ID: usize = FINAL_BLOCK_COUNT - 1;
        assert_eq!(game_state.block_count(), FINAL_BLOCK_COUNT);
        assert!(has_block_landed_oob(&game_state, FINAL_BLOCK_ID));
    }

    fn last_block<T, U>(game_state: &GameState<T, U>) -> (Cell, BlockType)
    where
        T: RangeRng<usize>,
        U: RangeRng<i32>,
    {
        game_state.block(game_state.block_count() - 1)
    }

    fn last_block_distance_to_left_wall<T, U>(game_state: &GameState<T, U>) -> i32
    where
        T: RangeRng<usize>,
        U: RangeRng<i32>,
    {
        (last_block(game_state).0).1
    }

    fn last_block_distance_to_right_wall<T, U>(game_state: &GameState<T, U>) -> i32
    where
        T: RangeRng<usize>,
        U: RangeRng<i32>,
    {
        TEST_BOARD_WIDTH
            - last_block(game_state).1.width()
            - last_block_distance_to_left_wall(game_state)
    }

    fn fall_block<T, U>(game_state: &mut GameState<T, U>)
    where
        T: RangeRng<usize>,
        U: RangeRng<i32>,
    {
        let original_block_count = game_state.block_count();
        while original_block_count == game_state.block_count() {
            game_state.tick();
        }
    }

    #[test]
    fn test_lr_collisions() {
        let mut game_state = GameState::new(
            0,
            0,
            TEST_BOARD_WIDTH,
            TEST_BOARD_HEIGHT,
            mocks::SingleValueRangeRng::new(BlockType::S as usize),
            mocks::SingleValueRangeRng::new((TEST_BOARD_WIDTH / 2) as i32),
        );

        // generate first block
        game_state.tick();
        assert!(game_state.block_count() == 1);

        // verify that a block can be moved left which will change its position
        let distance_to_left_wall = last_block_distance_to_left_wall(&game_state);
        for _ in 0..distance_to_left_wall {
            game_state.move_block_horizontal(-1);
            assert_ne!(
                distance_to_left_wall,
                last_block_distance_to_left_wall(&game_state)
            );
        }

        // verify that once a block collides with the left wall it can't move left any further but
        // it can move right
        assert_eq!(last_block_distance_to_left_wall(&game_state), 0);
        game_state.move_block_horizontal(-1);
        assert_eq!(last_block_distance_to_left_wall(&game_state), 0);
        game_state.move_block_horizontal(1);
        assert_eq!(last_block_distance_to_left_wall(&game_state), 1);
        fall_block(&mut game_state);

        // verify that a block can be moved right which will change its position
        assert!(game_state.block_count() == 2);
        let distance_to_right_wall = last_block_distance_to_right_wall(&game_state);
        for _ in 0..distance_to_right_wall {
            game_state.move_block_horizontal(1);
            assert_ne!(
                distance_to_left_wall,
                last_block_distance_to_right_wall(&game_state)
            );
        }
        // verify that once a block collides with the right wall it can't move right any further
        // but it can move left
        assert_eq!(last_block_distance_to_right_wall(&game_state), 0);
        game_state.move_block_horizontal(1);
        assert_eq!(last_block_distance_to_right_wall(&game_state), 0);
        game_state.move_block_horizontal(-1);
        assert_eq!(last_block_distance_to_right_wall(&game_state), 1);
        fall_block(&mut game_state);

        // generate a stack of blocks in the middle
        //      xx
        //     xx
        //      oo
        //     oo
        //      xx
        //     xx
        for _ in 0..3 {
            fall_block(&mut game_state);
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
        for _ in 0..last_block(&game_state).1.width() {
            game_state.move_block_horizontal(1);
        }
        // drop the latest block until its 1 block away from touching the bottom
        //      xx  |
        //     xx   V
        //      oo
        //     oo  $$
        //      xx$$
        //     xx
        for _ in 0..TEST_BOARD_HEIGHT - 1 {
            game_state.tick();
        }
        assert_eq!(
            (last_block(&game_state).0).0,
            TEST_BOARD_HEIGHT - 1 - last_block(&game_state).1.height()
        );

        // can't move the last block to the left because of collision
        let left_wall_distance_before = last_block_distance_to_left_wall(&game_state);
        game_state.move_block_horizontal(-1);
        assert_eq!(
            last_block_distance_to_left_wall(&game_state),
            left_wall_distance_before
        );

        // drop the last block 1 more time so it is touching the bottom
        //      xx  |
        //     xx   V
        //      oo
        //     oo
        //      xx $$
        //     xx $$
        game_state.tick();
        assert_eq!(
            (last_block(&game_state).0).0,
            TEST_BOARD_HEIGHT - last_block(&game_state).1.height()
        );

        // the last block can now move left
        //      xx
        //     xx
        //      oo
        //     oo
        //      xx$$ <=
        //     xx$$  <=
        game_state.move_block_horizontal(-1);
        assert_eq!(
            last_block_distance_to_left_wall(&game_state),
            left_wall_distance_before - 1
        );
        while !game_state.is_game_over() {
            game_state.tick();
        }
    }
}
