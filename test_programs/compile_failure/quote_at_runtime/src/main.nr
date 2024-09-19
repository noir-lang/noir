fn main() {
    foo(quote { test })
}

fn foo(q: Quoted) {
    println(q);
}
