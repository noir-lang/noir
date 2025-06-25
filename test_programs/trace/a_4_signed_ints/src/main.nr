fn main(mut x: Field, y: u64, z: i64) {
    x -= 100;
    let x_as_u64 = x as u64;
    let x_as_i64 = x as i64;
    println(f"x: {x} {x_as_u64} {x_as_i64}");

    let y_as_i64 = y as i64;
    println(f"y: {y} {y_as_i64}");

    println(f"z: {z}");
}
