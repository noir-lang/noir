fn main(a: bool) -> pub str<2> {
    let (_, d) = {
        let b: [str<2>] = if a {
            ["Aa", "Bb", "Cc"].as_vector()
        } else {
            ["Dd", "Ee"].as_vector()
        };
        b.pop_front()
    };
    d[1]
}
