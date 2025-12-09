fn main(a: bool) -> pub str<2> {
    let (_, d) = {
        let b: [str<2>] = if a {
            &["Aa", "Bb", "Cc"]
        } else {
            &["Dd", "Ee"]
        };
        b.pop_front()
    };
    d[1]
}
