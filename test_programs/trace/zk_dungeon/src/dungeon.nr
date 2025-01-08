pub struct Square {
    x: Field,
    y: Field
}

fn is_valid_step(prev: Square, curr: Square) -> bool {
    ((prev.x + 2 == curr.x) & (prev.y + 1 == curr.y))
        | ((prev.x - 2 == curr.x) & (prev.y + 1 == curr.y))
        | ((prev.x + 2 == curr.x) & (prev.y - 1 == curr.y))
        | ((prev.x - 2 == curr.x) & (prev.y - 1 == curr.y))
        | ((prev.x + 1 == curr.x) & (prev.y + 2 == curr.y))
        | ((prev.x - 1 == curr.x) & (prev.y + 2 == curr.y))
        | ((prev.x + 1 == curr.x) & (prev.y - 2 == curr.y))
        | ((prev.x - 1 == curr.x) & (prev.y - 2 == curr.y))
}

fn is_within_bounds(square: Square) -> bool {
    (square.x as u8 >= 0) & (square.x as u8 < 8) & (square.y as u8 >= 0) & (square.y as u8 < 8)
}

pub fn is_valid_path(path: [Square; 8], dagger: Square) {
    assert(path[0].x == 0);
    assert(path[0].y == 0);
    assert(path[7].x == dagger.x);
    assert(path[7].y == dagger.y);
    for i in 1..8 {
        assert(is_within_bounds(path[i]));
        assert(is_valid_step(path[i - 1], path[i]));
    }
}

fn is_safe_step(square: Square, watcher_map: [[bool; 8]; 8]) -> bool {
    let mut result = true;
    for x in 0..8 {
        for y in 0..8 {
            if watcher_map[x][y] {
                let fx = x as Field;
                let fy = y as Field;
                if (square.x - fx == square.y - fy) | (square.x - fx == fy - square.y) {
                    result = false;
                }
            }
        }
    }
    result
}

pub fn is_safe_path(path: [Square; 8], watcher_map: [[bool; 8]; 8]) {
    for i in 0..8 {
        assert(is_safe_step(path[i], watcher_map));
    }
}
