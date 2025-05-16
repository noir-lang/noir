// Regression for issue #8521 (https://github.com/noir-lang/noir/issues/8521)
// We do not expect division by zero in the while loop to be hoisted
// x = 0
unconstrained fn main(x: u32) -> pub str<3> {
    let mut idx_b = 0;
    while true {
        if (idx_b == 0) {
            break
        } else {
            idx_b = (idx_b + 1);
            let c = (5 / x);
            println(c);
        }
    }
    "SQF"
}
