use dep::std::collections::vec::Vec;

use crate::dungeon::Square;
use crate::dungeon::is_safe_path;

#[test]
fn test_no_watchers_safe() {
    let watcher_map = [[false; 8]; 8];

    let path = [
        Square {x: 0, y: 0},
        Square {x: 2, y: 1},
        Square {x: 4, y: 2},
        Square {x: 5, y: 4},
        Square {x: 7, y: 5},
        Square {x: 5, y: 6},
        Square {x: 7, y: 7},        
        Square {x: 6, y: 5}
    ];

    is_safe_path(path, watcher_map);
}

#[test]
fn test_one_watcher_safe() {
    let mut watcher_map = [[false; 8]; 8];
    watcher_map[3][7] = true;

    let path = [
        Square {x: 0, y: 0},
        Square {x: 2, y: 1},
        Square {x: 4, y: 2},
        Square {x: 5, y: 4},
        Square {x: 3, y: 5},
        Square {x: 5, y: 6},
        Square {x: 3, y: 5},
        Square {x: 4, y: 7},
    ];

    is_safe_path(path, watcher_map);
}

#[test]
fn test_many_watchers_safe() {
    let mut watcher_map = [[false; 8]; 8];
    watcher_map[2][4] = true;
    watcher_map[2][5] = true;
    watcher_map[4][3] = true;
    watcher_map[5][2] = true;
    watcher_map[6][0] = true;

    let path = [
        Square {x: 0, y: 0},
        Square {x: 1, y: 2},
        Square {x: 3, y: 1},
        Square {x: 2, y: 3},
        Square {x: 4, y: 4},
        Square {x: 5, y: 6},
        Square {x: 6, y: 4},
        Square {x: 7, y: 2},
    ];

    is_safe_path(path, watcher_map);
}

#[test(should_fail)]
fn test_one_watcher_unsafe() {
    let mut watcher_map = [[false; 8]; 8];
    watcher_map[5][3] = true;

    let path = [
        Square {x: 0, y: 0},
        Square {x: 1, y: 2},
        Square {x: 3, y: 1},
        Square {x: 2, y: 3},
        Square {x: 1, y: 5},
        Square {x: 3, y: 6},
        Square {x: 5, y: 5},
        Square {x: 6, y: 3},
    ];

    is_safe_path(path, watcher_map);
}

#[test(should_fail)]
fn test_many_watchers_unsafe_1() {
    let mut watcher_map = [[false; 8]; 8];
    watcher_map[5][1] = true;
    watcher_map[6][2] = true;
    watcher_map[3][7] = true;

    let path = [
        Square {x: 0, y: 0},
        Square {x: 1, y: 2},
        Square {x: 0, y: 4},
        Square {x: 1, y: 6},
        Square {x: 2, y: 4}, // Unsafe
        Square {x: 4, y: 5},
        Square {x: 6, y: 6},
        Square {x: 7, y: 4}
    ];

    is_safe_path(path, watcher_map);
}

#[test(should_fail)]
fn test_many_watchers_unsafe_2() {
    let mut watcher_map = [[false; 8]; 8];
    watcher_map[6][1] = true;
    watcher_map[6][7] = true;
    watcher_map[3][7] = true;
    watcher_map[4][7] = true;

    let path = [
        Square {x: 0, y: 0},
        Square {x: 2, y: 1},
        Square {x: 3, y: 3},
        Square {x: 1, y: 2},
        Square {x: 2, y: 4},
        Square {x: 3, y: 2},
        Square {x: 5, y: 3}, // Unsafe
        Square {x: 4, y: 1}
    ];

    is_safe_path(path, watcher_map);
}