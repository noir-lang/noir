global EXPONENTIATE: [[Field; 2]; 2] = [[1, 1], [0, 0]];

fn main(x: u32, y: pub Field) {
     /// Safety: testing context
    unsafe {
        check_acc_entry_point(x as Field, y);
        assert(entry_point(x) == 2);
    }
}

unconstrained fn check_acc_entry_point(x: Field, y: Field) {
    let mut acc: Field = 0;
    for i in 0..2 {
        for j in 0..2 {
            acc += EXPONENTIATE[i][j];
        }
    }
    assert(acc != 0);
    assert(x != y);
}

fn inner(x: u32) -> u32 {
    x + 1
}

unconstrained fn entry_point(x: u32) -> u32 {
    inner(x + 1)
}