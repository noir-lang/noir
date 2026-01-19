fn main() {
    comptime let mut x = 1;

    println(x); // prints 1

    comptime {
        x += 1;
    }

    println(x); // prints 2
}
