global EXPONENTIATE: [[Field; 2]; 2] = [[1, 1], [0, 0]];

fn main(x: Field, y: pub Field) {
    let mut acc: Field = 0;
    for i in 0..2 {
        for j in 0..2 {
            acc += EXPONENTIATE[i][j];
        }
    }
    assert(!acc.lt(x));
    assert(x != y);

    dummy_again(x, y);
}

fn dummy_again(x: Field, y: Field) {
    // Move this before as to change the value ids of the common constants
    // between this function and main
    assert(x != y);

    let mut acc: Field = 0;
    for i in 0..2 {
        for j in 0..2 {
            acc += EXPONENTIATE[i][j];
        }
    }
    assert(!acc.lt(x));
}
