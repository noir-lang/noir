// issue #4290
fn loop_from<N, K>(_arr: [Field; N], _another: [Field; K]) -> u32 {
    let mut acc = 0;
    // N and K were of type Field but the iterator is u32
    for i in N..K {
        acc += i;
    }
    acc
}

fn main() {
    let _ = loop_from([0; 10], [0; 20]);
}
