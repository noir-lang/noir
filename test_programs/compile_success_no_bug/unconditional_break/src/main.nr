fn main() {
    // Safety:
    let x = unsafe { break_in_if_body() };
    assert(x);

    // Safety:
    unsafe {
        break_in_unary();
        break_in_infix_lhs();
        break_in_infix_rhs();
        break_in_array_element();
        break_in_if_condition();
        break_in_while_condition();
    };
}

unconstrained fn break_in_if_body() -> bool {
    let mut x = true;
    loop {
        if true {
            break;
        }
        x = false;
    }
    x
}

unconstrained fn break_in_unary() {
    let mut a = 0;
    loop {
        a = -{
            break;
            a
        };
    }
}

unconstrained fn break_in_infix_lhs() {
    let mut a = 0;
    loop {
        a = {
            break;
            0
        }
            - 0;
    }
}

unconstrained fn break_in_infix_rhs() {
    let mut a = 0;
    loop {
        a = 0
            + {
                break;
                0
            };
    }
}

unconstrained fn break_in_array_element() {
    loop {
        let _ = [
            1, {
                break;
                2
            }, 3,
        ];
    }
}

unconstrained fn break_in_if_condition() {
    loop {
        if {
            break;
            true
        } {
            let _ = 10;
        }
    }
}

unconstrained fn break_in_while_condition() {
    loop {
        while (
            {
                break;
                true
            }
        ) {
            let _ = 10;
        }

        break;
    }
}
