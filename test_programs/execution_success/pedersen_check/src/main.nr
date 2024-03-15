use dep::std;

fn main(x: Field, y: Field, salt: Field, out_x: Field, out_y: Field, out_hash: Field) {
    let res = std::hash::pedersen_commitment_array([x, y]);
    assert(res.x == out_x);
    assert(res.y == out_y);

    let res_hash = std::hash::pedersen_hash_with_separator_array([x, y], 0);
    assert_eq(res_hash, out_hash);

    assert(res_hash != res.x);

    let raw_data = [x, y];
    let mut state = 0;
    for i in 0..2 {
        state = state * 8 + raw_data[i];
    }
    state += salt;
    let hash = std::hash::pedersen_commitment_array([state]);
    assert(std::hash::pedersen_commitment_array([43]).x == hash.x);
}

