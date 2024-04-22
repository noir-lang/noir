fn main() {
    let x = dep::std::unsafe::zeroed();
    foo(x);
}

fn foo<N>(array: [field; N]) {
    for elem in array {
        println(elem);
    }
}
