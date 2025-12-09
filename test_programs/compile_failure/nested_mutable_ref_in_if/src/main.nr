fn main() {
    let mut ref = &mut 0;
    if false {
        ref = &mut 1;
    } else {
        ref = &mut 2;
    }
    println(*ref);
}
