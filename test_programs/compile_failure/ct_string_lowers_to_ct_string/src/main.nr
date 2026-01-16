use std::append::Append;

fn main() {
    let ct_string = comptime {
        let mut ct_string = CtString::new();
        ct_string = ct_string.append("Hello");
        ct_string = ct_string.append(" world");
        ct_string
    };
    println(ct_string);
}
