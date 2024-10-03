fn main() {
    let x = std::mem::zeroed();
    foo(x);
}

fn foo<N>(array: [Field; N]) {
    for elem in array {
        println(elem);
    }
}
