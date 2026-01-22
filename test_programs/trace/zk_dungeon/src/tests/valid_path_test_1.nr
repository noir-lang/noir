use crate::dungeon::Square;
use crate::dungeon::is_valid_path;

#[test]
fn test_valid_path() {
    let path = [
        Square {x: 0, y: 0},
        Square {x: 1, y: 2},
        Square {x: 3, y: 3},
        Square {x: 4, y: 1},
        Square {x: 6, y: 0},
        Square {x: 7, y: 2},
        Square {x: 6, y: 4},
        Square {x: 5, y: 6}
    ];

    is_valid_path(path, Square {x: 5, y: 6});

    let path = [
        Square {x: 0, y: 0},
        Square {x: 2, y: 1},
        Square {x: 4, y: 2},
        Square {x: 6, y: 3},
        Square {x: 5, y: 1},
        Square {x: 3, y: 0},
        Square {x: 1, y: 1},
        Square {x: 2, y: 3}
    ];

    is_valid_path(path, Square {x: 2, y: 3});
}

#[test(should_fail)]
fn test_out_of_bounds() {
    let path = [
        Square {x: 0, y: 0},
        Square {x: 1, y: 2},
        Square {x: 3, y: 3},
        Square {x: 4, y: 1},
        Square {x: 6, y: 0},
        Square {x: 7, y: 2},
        Square {x: 8, y: 4}, // Out of bounds
        Square {x: 6, y: 5}
    ];

    is_valid_path(path, Square {x: 6, y: 5});
}

#[test(should_fail)]
fn test_invalid_move_1() {
    let path = [
        Square {x: 0, y: 0},
        Square {x: 2, y: 1},
        Square {x: 3, y: 2}, // Invalid move
        Square {x: 5, y: 3},
        Square {x: 6, y: 5},
        Square {x: 7, y: 7},
        Square {x: 5, y: 6},
        Square {x: 4, y: 4},
    ];

    is_valid_path(path, Square {x: 4, y: 4});
}

#[test(should_fail)]
fn test_invalid_move_2() {
    let path = [
        Square {x: 0, y: 0},
        Square {x: 2, y: 1},
        Square {x: 3, y: 3},
        Square {x: 5, y: 4},
        Square {x: 5, y: 7}, // Invalid move
        Square {x: 6, y: 5},
        Square {x: 4, y: 4},
        Square {x: 3, y: 2}
    ];

    is_valid_path(path, Square {x: 3, y: 2});
}

#[test(should_fail)]
fn test_invalid_start() {
    let path = [
        Square {x: 3, y: 3}, // Invalid start
        Square {x: 4, y: 1},
        Square {x: 6, y: 0},
        Square {x: 5, y: 2},
        Square {x: 7, y: 3},
        Square {x: 6, y: 2},
        Square {x: 5, y: 4},
        Square {x: 7, y: 5}
    ];

    is_valid_path(path, Square {x: 7, y: 5});
}

#[test(should_fail)]
fn test_invalid_end() {
    let path = [
        Square {x: 0, y: 0},
        Square {x: 2, y: 1},
        Square {x: 4, y: 2},
        Square {x: 6, y: 3},
        Square {x: 5, y: 1},
        Square {x: 3, y: 0},
        Square {x: 1, y: 1},
        Square {x: 2, y: 3}, // Invalid end
    ];

    is_valid_path(path, Square {x: 2, y: 0});
}
