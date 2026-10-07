// The macro's output resolves `MAX` in `token`, so the bound is `token::MAX` (10) and not the
// crate root's `MAX`.
global MAX: u32 = 1000000;

mod token {
    global MAX: u32 = 10;

    comptime fn max_bound() -> Quoted {
        quote { MAX }
    }

    pub fn check(x: u32) {
        let bound = comptime { max_bound!() };
        assert(x <= bound, "x exceeds token::MAX");
    }
}

fn main(x: u32) {
    token::check(x);
    assert(MAX != 0);
}
