fn main() {
    let _ = foo::<[&mut Field; 0]>();
}

#[fold]
fn foo<T>() -> [T; 0] {
    std::mem::zeroed()
}
