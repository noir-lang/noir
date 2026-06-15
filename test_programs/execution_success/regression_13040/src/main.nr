unconstrained fn main(cond: bool, idx: u32, go: bool) -> pub Field {
    let mut a: Field = 0;
    let mut b: Field = 0;
    let mut c: Field = 0;

    let mut sel: &mut Field = if cond { &mut a } else { &mut b };
    let mut other: &mut Field = &mut c;

    let cell0: &mut &mut Field = &mut sel;
    let cell1: &mut &mut Field = &mut other;
    let chosen: &mut &mut Field = if idx == 0 { cell0 } else { cell1 };
    *chosen = sel;
    let loaded: &mut Field = *chosen;
    let r2: &mut Field = if cond { loaded } else { &mut c };

    if go {
        a = 5;
        *r2 = 99;
        a
    } else {
        a
    }
}
