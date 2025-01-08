pub struct Square {
    x: Field,
    y: Field
}

pub fn eq_square(s1: Square, s2: Square) -> bool {
    (s1.x == s2.x) & (s1.y == s2.y)
}

pub fn in_board(s: Square) -> bool {
    (s.x as u8 >= 0) & (s.y as u8 >= 0) &
    (s.x as u8 <= 7) & (s.y as u8 <= 7)
}

pub fn is_valid_move(p1: Square, p2: Square) -> bool {
    assert(in_board(p1) & in_board(p2));
    let sub = Square { x: p2.x - p1.x, y: p2.y - p1.y };
    let moves: [Square; 8] = [Square {x:1,y:2}, Square {x:-1,y:2}, Square {x:1,y:-2}, Square {x:-1,y:-2},
                              Square {x:2,y:1}, Square {x:-2,y:1}, Square {x:2,y:-1}, Square {x:-2,y:-1}];
    let mut res: bool = false;
    for m in moves {
        if eq_square(sub, m) { res = true; }
    }
    res
}

pub fn is_valid_path(path: [Square; 8], dagger: Square) {
    // Part 1
    assert(eq_square(path[0], Square {x:0,y:0}));
    assert(eq_square(path[7], dagger));
    for i in 1..7 {
        assert(is_valid_move(path[i-1], path[i]));
    }
}

pub fn check(p: Square, watcher_map: [[bool; 8]; 8], offsetx: i8, offsety: i8) {
    let ret = Square { x: ((p.x as i8) + offsetx) as Field, y: ((p.y as i8) + offsety) as Field };
    // in_board -> !watcher_map
    if in_board(ret) {
        assert(!watcher_map[ret.x as u8][ret.y as u8]);
    }
}

pub fn is_safe_path(
    path: [Square; 8],
    watcher_map: [[bool; 8]; 8]
) {
    // Part 2
    for p in path {
        for offset in 0..7 {
            check(p, watcher_map,      offset,      offset);
            check(p, watcher_map,      offset, -1 * offset);
            check(p, watcher_map, -1 * offset,      offset);
            check(p, watcher_map, -1 * offset, -1 * offset);
        }
    }
}
