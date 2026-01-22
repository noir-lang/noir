fn my_func(a: [u32]) {
    for q in a {
        println(q);
    }
}

fn main(x: Field, y: pub Field) {
    assert(x != y);
    let arr = [5, 4, 3, 2, 1];
    my_func(arr);
}
