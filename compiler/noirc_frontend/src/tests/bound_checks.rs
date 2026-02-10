use crate::tests::check_errors;

#[test]
fn overflowing_u8() {
    let src = r#"
        fn main() {
            let _: u8 = 256;
                        ^^^ The value `256` cannot fit into `u8` which has range `0..=255`
        }
        "#;
    check_errors(src);
}

#[test]
fn underflowing_u8() {
    let src = r#"
        fn main() {
            let _: u8 = -1;
                        ^^ The value `-1` cannot fit into `u8` which has range `0..=255`
        }
        "#;
    check_errors(src);
}

#[test]
fn overflowing_i8() {
    let src = r#"
        fn main() {
            let _: i8 = 128;
                        ^^^ The value `128` cannot fit into `i8` which has range `-128..=127`
        }
        "#;
    check_errors(src);
}

#[test]
fn underflowing_i8() {
    let src = r#"
        fn main() {
            let _: i8 = -129;
                        ^^^^ The value `-129` cannot fit into `i8` which has range `-128..=127`
        }
        "#;
    check_errors(src);
}
