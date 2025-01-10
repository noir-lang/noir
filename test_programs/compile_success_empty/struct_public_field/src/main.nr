use dependency::Point;

fn main() {
    let point = Point { x: 1, y: 2 };
    let _ = point.x;
    let Point { x, y } = point;
    let _ = (x, y);
}
