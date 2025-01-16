mod dungeon;

fn main(
    watcher_map: pub [[bool; 8]; 8],
    dagger_x: pub Field,
    dagger_y: pub Field,
    path_x: [Field; 8],
    path_y: [Field; 8],
) {
    dungeon::is_valid_path(path_x, path_y, dagger_x, dagger_y);
    dungeon::is_safe_path(path_x, path_y, watcher_map);
}
