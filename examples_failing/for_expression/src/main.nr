// For Expressions, not For Statements!
//
// Avoids mutability
//
// XXX(Dankrad): For loop is not clear
fn main(x : Field, y : Field) {
    let a = 1;
    let k = x + y;

    let j = for _k in a..100 {
       let z = x + k;
       z
    };

    constrain k == j[1];
}
    
