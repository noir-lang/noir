fn main() {
    // Safety: does nothing
    unsafe {
        foo(@["KR"]);
    }
}

unconstrained fn foo(mut a: [str<2>]) {
    let _b = a.pop_front();
    println(a);
}
