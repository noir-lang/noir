#[macro_use]
extern crate afl;
extern crate noirc_frontend;

fn main() {
    fuzz!(|data: &[u8]| {
        if let Ok(s) = std::str::from_utf8(data) {
            let _ = noirc_frontend::parser::parse_program(&s);
        }
    });
}
