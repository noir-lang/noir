unconstrained fn main(x: u32) {
    if x == 0 {
        println("done!");
    } else {
        println(f"recursing: {x}");
        main(x - 1);
    }
}
