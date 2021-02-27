// For Expressions, not For Statements!
//
// Avoids mutability
//
// XXX(Dankrad): For loop is not clear
fn main(x : Witness, y : Witness) {
    const _a = 1;
    priv k = x + y;

    let j = for _k in _a..100 {
       priv z = x + k;
       z
    };

    constrain k == j[1];
}
    