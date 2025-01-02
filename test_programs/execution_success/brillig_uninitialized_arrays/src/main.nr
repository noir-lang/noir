fn main(x: Field, y: Field) -> pub Field {
    unsafe {
        //@safety: testing context
        let notes = create_notes(x, y);
        sum_x(notes, x, y)
    }
}

fn sum_x(notes: [Field; 2], x: Field, y: Field) -> Field {
    notes[x] + notes[y]
}

unconstrained fn create_notes(x: Field, y: Field) -> [Field; 2] {
    [x, y]
}
