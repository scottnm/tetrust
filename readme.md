# Tetrust

## Intent

- Practice rust
- Use a TUI (cuz tuis are cool)

## Goals

- Basic tetris rules
    - fixed sized grid
    - blocks fall one at a time
    - end game when top of screen reached

- Basic tetris mechanics
    - tetromino gravity
    - rotating tetrominos
    - dropping tetrominos
    - clearing lines
    - score

- Extensions
    - game music
    - ai/autoplay
    - ai/competitive
    - multiplayer
    - network multiplayer

### Steps

- [X] [Get a tetromino to render](https://github.com/scottnm/tetrust/commit/76babe55dcab890374494fc912e77d16b2fe0e48)

![Image of text-tetrominoes rendering](demo/01-render.gif)

- [X] [Get a tetromino to fall](https://github.com/scottnm/tetrust/commit/f3aca54cb39c7137e0c38f52fd2c4c8d9f23af4b)

![Image of text-tetrominoes falling](demo/02-fall.gif)

- [X] [Get tetrominos to stack on each other and floor](https://github.com/scottnm/tetrust/commit/915e61e7d227fea6e134da75f864629514f3c9f8)

![Image of text-tetrominoes stacking](demo/03-stack.gif)

- [X] [Add color to tetrominoes](https://github.com/scottnm/tetrust/commit/1c547fc7bc0d701fa8e7117592c61a0a5b693840)

![Image of tetrominoes with color](demo/04-color.gif)

- [X] [Generate tetrominos based on game rules](https://github.com/scottnm/tetrust/commit/b72efb7eb834d442885c35f5cbb8173c2b1ba887)

![Image of tetrominoes falling one at a time](demo/05-generate-by-rules.gif)

- [X] [Add some sort of test framework](https://github.com/scottnm/tetrust/commit/2d4fbc7ba4b3579150d3a3c889dd88d99c34e578)

![Image of unit tests passing](demo/06-test.png)

- [X] [Handle game lose state](https://github.com/scottnm/tetrust/commit/b72efb7eb834d442885c35f5cbb8173c2b1ba887)
- [X] [Handle left-right inputs](https://github.com/scottnm/tetrust/commit/a819261fdfd041bd8fbcc280d9661e78f355bdcd)

![Image of left-right collision](demo/07-lr-collision.gif)

- [X] [Constrain board size and add game over screen](https://github.com/scottnm/tetrust/commit/44bbeee4d17255c68c0f7c96ebe29a6b6c151b2a)

![Image of constrained board with game over screen](demo/08-constrained-gameover-blink.gif)

- [X] [Allow tetrominos to rotate](https://github.com/scottnm/tetrust/commit/3dd8bba32517b65c19e1ad4082612eb287630734)

![Image of pieces rotation](demo/09-rotation.gif)

- [X] [Preview blocks](https://github.com/scottnm/tetrust/commit/c8e859c5857bb7a48843ab7108bff9692a0370e0)

![Image of next block showing up in preview pane](demo/10-preview.gif)

- [X] [Handle pausing](https://github.com/scottnm/tetrust/commit/364add645b291dd330ccb3817eae0988b9a761e3)

![Image of pause screen](demo/11-pause.gif)

- [X] [Scoring and line clears](https://github.com/scottnm/tetrust/commit/b330acb)

![Image of scoring](demo/12-scoring.gif)

- [X] [Speed up pieces falling as more lines are cleared](https://github.com/scottnm/tetrust/commit/FILLMEIN)

![Image of increased fall speed](demo/13-fallspeed.gif)

- [ ] Handle quick fall
- [ ] Show piece fall preview
- [ ] Add start screen
- [ ] Add high score logging
- [ ] Add high score screen from start

