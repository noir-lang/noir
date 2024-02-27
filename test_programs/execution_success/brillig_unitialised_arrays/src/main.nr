fn main(x: field, y: field) -> pub field {
    let notes = create_notes(x, y);
    sum_x(notes, x, y)
}

fn sum_x(notes: [field; 2], x: field, y: field) -> field {
    notes[x] + notes[y]
}

unconstrained fn create_notes(x: field, y: field) -> [field; 2] {
    [x, y]
}
