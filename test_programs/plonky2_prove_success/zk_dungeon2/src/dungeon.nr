fn is_valid_step(prev_x: Field, prev_y: Field, curr_x: Field, curr_y: Field) -> bool {
    ((prev_x + 2 == curr_x) & (prev_y + 1 == curr_y))
        | ((prev_x - 2 == curr_x) & (prev_y + 1 == curr_y))
        | ((prev_x + 2 == curr_x) & (prev_y - 1 == curr_y))
        | ((prev_x - 2 == curr_x) & (prev_y - 1 == curr_y))
        | ((prev_x + 1 == curr_x) & (prev_y + 2 == curr_y))
        | ((prev_x - 1 == curr_x) & (prev_y + 2 == curr_y))
        | ((prev_x + 1 == curr_x) & (prev_y - 2 == curr_y))
        | ((prev_x - 1 == curr_x) & (prev_y - 2 == curr_y))
}

fn is_within_bounds(square_x: Field, square_y: Field) -> bool {
    (square_x as u8 >= 0) & (square_x as u8 < 8) & (square_y as u8 >= 0) & (square_y as u8 < 8)
}

pub fn is_valid_path(path_x: [Field; 8], path_y: [Field; 8], dagger_x: Field, dagger_y: Field) {
    assert(path_x[0] == 0);
    assert(path_y[0] == 0);
    assert(path_x[7] == dagger_x);
    assert(path_y[7] == dagger_y);
    for i in 1..8 {
        assert(is_within_bounds(path_x[i], path_y[i]));
        assert(is_valid_step(path_x[i - 1], path_y[i - 1], path_x[i], path_y[i]));
    }
}

fn is_safe_step(square_x: Field, square_y: Field, watcher_map: [[bool; 8]; 8]) -> bool {
    let mut result = true;
    for x in 0..8 {
        for y in 0..8 {
            if watcher_map[x][y] {
                let fx = x as Field;
                let fy = y as Field;
                if (square_x - fx == square_y - fy) | (square_x - fx == fy - square_y) {
                    result = false;
                }
            }
        }
    }
    result
}

pub fn is_safe_path(path_x: [Field; 8], path_y: [Field; 8], watcher_map: [[bool; 8]; 8]) {
    for i in 0..8 {
        assert(is_safe_step(path_x[i], path_y[i], watcher_map));
    }
}
