#[macro_use]
extern crate afl;

use noirc_frontend;

fn main() {
    fuzz!(|data: &[u8]| {
        if let Ok(s) = std::str::from_utf8(data) {
            let _ = noirc_frontend::lexer::Lexer::lex(&s);
        }
    });
}
