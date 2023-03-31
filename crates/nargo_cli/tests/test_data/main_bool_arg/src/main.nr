fn main(x : bool, y: [bool;2]) {
    if x {
        constrain 1 != 2;
    }

    constrain x;
    constrain y[0] != y[1];
}
