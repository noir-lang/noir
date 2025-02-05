fn main(a : i32, b : i32, condition: bool) -> pub i32 {
    if condition {
        // If `condition` is not set then we should not trigger an assertion failure here.
        a / b 
    } else {
        0
    }
}