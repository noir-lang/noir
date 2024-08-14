fn main() {
    let x = 2;
    let y = 3;
    let xy = x + y;

    std::static_assert(xy == 6, "2 + 3 != 6");
}
