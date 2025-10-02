mod dungeon;
mod tests;

fn main(
    watcher_map: pub [[bool; 8]; 8],
    dagger: pub dungeon::Square,
    path: [dungeon::Square; 8]
) {
    dungeon::is_valid_path(path, dagger);
    dungeon::is_safe_path(path, watcher_map);
}