#[derive(Eq)]
struct Config {
    alpha: Field,
    beta: Field,
    gamma: Field,
    delta: Field,
    epsilon: Field,
    zeta: Field,
    eta: Field,
    theta: Field,
}

fn main() {
    comptime {
        // Path A: fields in definition order
        let a =
            Config { alpha: 1, beta: 2, gamma: 3, delta: 4, epsilon: 5, zeta: 6, eta: 7, theta: 8 };

        // Path B: fields in reverse order (semantically identical)
        let b =
            Config { theta: 8, eta: 7, zeta: 6, epsilon: 5, delta: 4, gamma: 3, beta: 2, alpha: 1 };

        // Direct comparison works
        assert(a == b);

        // But Quoted comparison can fail:
        let qa = quote { $a };
        let qb = quote { $b };

        assert(qa == qb);
    }
}
