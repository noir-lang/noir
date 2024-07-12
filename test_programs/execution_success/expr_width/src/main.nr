fn main(x: Field, y: pub Field) {
    let state = [x, y];
    let state = oh_no_not_again(state);
    // let state = oh_no_not_again_as_witness(state);

    assert(state[0] + state[1] == 0x01c1);
}

fn oh_no_not_again(mut state: [Field; 2]) -> [Field; 2] {
    for _ in 0..200 {
    // for _ in 0..4 {
        state[0] = state[0] * state[0] + state[1];
        state[1] += state[0];
    }
    state
}

fn oh_no_not_again_as_witness(mut state: [Field; 2]) -> [Field; 2] {
    for i in 0..200 {
        state[0] = state[0] * state[0] + state[1];
        std::as_witness(state[0]);
        state[1] += state[0];
        if (i & 1 == 1) {
            std::as_witness(state[1]);
        }
    }
    state
}

// TODO: This still has a big blow-up
// global SIZE = 2;
// fn main(mut fields: [Field; SIZE], enables: [bool; SIZE], indices: [u64; SIZE]) -> pub [Field; SIZE] {
//     for i in 0..SIZE {
//         if enables[i] {
//             fields[indices[i]] = i as Field;
//         }
//     }
//     fields
// }