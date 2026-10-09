// The merges below leave `s` with a semantic length below its backing capacity, and the
// length of the last `push_back` is only known to be constant once lowered to ACIR. The
// merge after it reads every slot of the pushed vector's capacity, so the program must
// solve for every input.
fn main(c: bool, d: bool) {
    let mut s: [Field] = [1, 2, 3].as_vector();
    if c {
        s = s.push_back(4);
        s = s.push_back(5);
    }
    let (t, _e) = s.remove(0);
    s = t;
    if !c {
        s = s.push_back(6);
        s = s.push_back(7);
    }
    if d {
        s = s.push_back(8);
    } else {
        s = s.push_back(9);
        s = s.push_back(10);
    }
    let mut acc: Field = 0;
    for i in 0..8 {
        if i < s.len() {
            acc += s[i];
        }
    }
    assert(acc != 12345);
}
