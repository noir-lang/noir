fn main(x: Field, y: pub Field) {
// fn main(state: [Field; 2]) {
// fn main(mut state: [Field; 2]) -> pub [Field; 2] {
    // let xx = x * x;
    // let xxxx = xx * xx;

    // let c0 = 1;
    // let c1 = 2;
    // let c2 = 3;
    // let c3 = 4;

    // let xxxxx = x * xxxx;
    // std::as_witness(xxxxx);
    // let f = xxxxx * c0 + x * (c1 + c2 + c3) + y;

    // // let f = x * xxxx * c0 + x * (c1 + c2 + c3) + y;
    // // std::as_witness(f);
    // assert(f == 0);

    let state = [x, y];
    // let state = oh_no_not_again(state);
    let state = oh_no_not_again_as_witness(state);

    assert(state[0] + state[1] == 0);
}

fn oh_no_not_again(mut state: [Field; 2]) -> [Field; 2] {
    for _ in 0..200 {
        state[0] = state[0] * state[0] + state[1];
        state[1] += state[0];
    }
    state
}

fn oh_no_not_again_as_witness(mut state: [Field; 2]) -> [Field; 2] {
    for i in 0..200 {
        state[0] = (state[0] * state[0]) + state[1];
        std::as_witness(state[0]);
        state[1] += state[0];
        if (i & 1 == 1) {
            std::as_witness(state[1]);
        }
    }
    state
}