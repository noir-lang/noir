mod consts;
use consts::EXPONENTIATE;

fn main(x: Field, y: pub Field) {
    /// Safety: testing context
    unsafe {
        assert(entry_point_no_globals(x) == 2);
        check_acc_entry_point(x, y);
        assert(entry_point_no_globals(x) == 2);
        entry_point_inner_func_globals(x as Field, y);
        assert(entry_point_no_globals(x) == 2);
        assert(entry_point_no_globals(x) == 2);
        assert(entry_point_no_globals(x) == 2);
        assert(entry_point_no_globals(x) == 2);
        assert(entry_point_no_globals(x) == 2);
        assert(entry_point_no_globals(x) == 2);
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

    assert(inner(x + 1) == 2);
}

fn inner(x: Field) -> Field {
    x + 1
}

unconstrained fn entry_point_no_globals(x: Field) -> Field {
    inner(x + 1)
}

unconstrained fn entry_point_inner_func_globals(x: Field, y: Field) {
    check_acc_entry_point(x, y);
}
