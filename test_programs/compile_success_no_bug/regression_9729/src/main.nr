fn foo(x: ()) {
    let (): () = x;
}

unconstrained fn bar(x: ()) {
    let (): () = x;
}

fn main() {
    foo(());
    unsafe { bar(()) };
    let _: fmtstr<5, ()> = f"Hello";
}
