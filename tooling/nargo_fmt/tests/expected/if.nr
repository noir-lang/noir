//@error_on_lost_comment=false
fn main() {
    let (x, y) = if is_square(gx1) {
        (x1, sqrt(gx1))
    } else {
        (x2, sqrt(gx2))
    };

    let n = if x != y {
        if x != 20 {
            slice.push_back(y)
        } else {
            slice
        }
    } else {
        slice
    };

    if false {
        ();
        ();
    }

    if false // lone if comment
    {
        ();
        ();
    }

    let a = if 0 > 1 { 0 } else { 0 };

    if true {
        ();
    } else if false {
        ();
        ();
    } else {
        ();
        ();
        ();
    }

    if true // else-if-chain if comment
    {
        ();
    } else if false // else-if-chain else-if comment
    {
        ();
        ();
    } else // else-if-chain else comment
    {
        ();
        ();
        ();
    }
}
