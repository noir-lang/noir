struct Pair {
    first: Field,
    second: Field
}

fn main(x: [Pair; 3], y: pub [Field; 6]) {
  for i in 0..3 {
    assert(x[i].first == y[2 * i]);
    assert(x[i].second == y[2 * i + 1]);
  }
}
