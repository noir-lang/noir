fn main(i: u32, mut a: [(Field, bool); 3]) -> pub Field {
    a[i] = (7, true);
    a[0].0
}
