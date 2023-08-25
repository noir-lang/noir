use dep::std;

fn main(x: Field, y: Field, salt: Field, out_x: Field, out_y: Field ) {
    let res = std::hash::pedersen([x, y]);
    assert(res[0] == out_x);
    assert(res[1] == out_y);

    let raw_data = [x,y];
    let mut state = 0;
    for i in 0..2 {
        state = state * 8 + raw_data[i];
    }
    state += salt;
    let hash = std::hash::pedersen([state]);
    assert(std::hash::pedersen([43])[0] == hash[0]);
}
    
