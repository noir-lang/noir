// `#[transparent]` makes a single-field wrapper serialize as its inner field. `Prover.toml`
// therefore supplies `point` directly in `Point` shape (`[point] x/y`, no `inner` table), and the
// `pub` return is encoded the same way — the captured `Circuit output` echoes it back as a `Point`.
// Without transparency the wrapper would require `[point.inner]` and print as `Wrapper { .. }`.
struct Point {
    x: Field,
    y: Field,
}

#[transparent]
struct Wrapper {
    inner: Point,
}

fn main(point: Wrapper) -> pub Wrapper {
    // Transparency is an ABI-boundary concept only. Inside the program the value is still a
    // `Wrapper`, and `println` uses the printable-type metadata (unaffected by `#[transparent]`),
    // so it prints the wrapper in full — in contrast to the transparent `Circuit output` below.
    println(point);
    point
}
