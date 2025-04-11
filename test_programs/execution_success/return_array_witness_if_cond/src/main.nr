// Regression taken from issue #7961 (https://github.com/noir-lang/noir/issues/7961)
fn main(a: bool, b: [[bool; 2]; 3]) -> pub [bool; 2] {
    // let z = !!a;
    // println(z);
    let x = b[0];
    if (!!a) {
        if (!!a) {
            b[0]
            // x
        } else {
            b[0]
            // x
        }
    } else {
        b[0]
        // x
    }
}