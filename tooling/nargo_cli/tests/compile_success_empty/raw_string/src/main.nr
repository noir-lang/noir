global D = r#####"Hello "world""#####;

fn main() {
    let a = "Hello \"world\"";
    let b = r#"Hello "world""#;
    let c = r##"Hello "world""##;
    assert(a == b);
    assert(b == c);
    assert(c == D);
}
