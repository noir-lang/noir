
fn main(a: u32, mut c: [u32; 4]){
    if a == c[0] {
        constrain c[0] == 0;
    } else if a == c[1] {
        constrain c[1] == 0;
    } else if a == c[2] {
        constrain c[2] == 0;
    } else if a == c[3] {
        // expect to match this case
        constrain c[3] == 0;
    } else {
       constrain c[0] == 10;
    }
}

