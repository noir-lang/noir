fn main() -> pub bool {
    let a = [[&mut false], [&mut true]];

    if *a[if *a[1][0] { 0 } else { 1 }][0] {
        true
    } else {
        false
    }
}
