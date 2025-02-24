use std::meta::ctstring::AsCtString;

fn main() {
    comptime {
        for i in 9..11 {
            // Lengths are different on each iteration:
            // foo9, foo10
            let name = f"foo{i}".as_ctstring().as_quoted_str!();

            // So to call `from_signature` we need to delay the type check
            // by quoting the function call so that we re-typecheck on each iteration
            let hash = std::meta::unquote!(quote { from_signature($name) });
            assert(hash > 3);
        }
    }
}

fn from_signature<let N: u32>(_signature: str<N>) -> u32 {
    N
}
