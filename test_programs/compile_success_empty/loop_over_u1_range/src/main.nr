fn main() {
    let zero: u1 = 0;
    for _ in zero..zero {}

    let one: u1 = 1;
    for _ in one..one {}

    for _ in zero..one {}
}
