use dep::std;

fn main(x: Field, y: Field, salt: Field, out_x: Field, out_y: Field ) {
    let res = [x, y].pedersen();
    constrain res[0] == out_x;
    constrain res[1] == out_y;

    let raw_data = [x,y];
    let mut state = 0;
    for i in 0..2 {
        state = state * 8 + raw_data[i];
    }
    state += salt;
    let hash = [state].pedersen();
    constrain [43].pedersen()[0] == hash[0];
}
    
