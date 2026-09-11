// A guarded match arm runs only when its pattern matches and its guard holds; otherwise the
// arms below it are tried. Runtime inputs keep the match from being constant-folded, so the
// generated decision tree is what is exercised.
enum Opt {
    None,
    Some(u32),
}

fn classify(o: Opt, limit: u32) -> u32 {
    match o {
        Opt::Some(v) if v > limit => 1,
        Opt::Some(v) if v == limit => 2,
        Opt::Some(_) => 3,
        Opt::None => 4,
    }
}

fn main(x: u32, y: u32) {
    assert(classify(Opt::Some(y), x) == 1);
    assert(classify(Opt::Some(x), x) == 2);
    assert(classify(Opt::Some(x), y) == 3);
    assert(classify(Opt::None, x) == 4);

    // A guard reads the variables its own pattern binds, and a failing guard falls through to
    // the next arm even when both arms have the same pattern.
    let pair = (x, y);
    let sum = match pair {
        (a, b) if a == b => 0,
        (a, b) => a + b,
    };
    assert(sum == x + y);
}
