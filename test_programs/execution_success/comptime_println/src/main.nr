fn main() {
    let x = comptime {
        println("hello from compile-time!");
        1 + 2
    };
    println(x);
}
