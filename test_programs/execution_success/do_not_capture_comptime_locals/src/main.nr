fn main() {
    comptime let x = 1;
    let f = || { println(x); };
    f();
}
