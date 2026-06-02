fn main() {
    foo(());
}

#[fold]
fn foo(_: ()) {}
