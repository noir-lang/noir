// Merging two vectors of unequal capacity produces a vector whose backing capacity (2) can exceed
// its semantic length (1). Popping that vector must not leave a `remove_if_else` merge over-reading
// the popped result, which would otherwise read element 0 of an empty vector.
fn main(choose: bool, do_pop: bool) -> pub Field {
    let mut v: [Field] = if choose {
        [1].as_vector()
    } else {
        [2, 3].as_vector()
    };

    if do_pop {
        let (new_v, _) = v.pop_back();
        v = new_v;
    }

    if v.len() == 0 {
        0
    } else {
        v[0]
    }
}
