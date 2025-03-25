fn main(x: u32, y: u32) -> pub Field {
    // Safety: testing context
    unsafe {
        let notes = create_notes(x, y);
        sum_x(notes, x, y)
    }
}

fn sum_x(notes: [Field; 2], x: u32, y: u32) -> Field {
    notes[x] + notes[y]
}

unconstrained fn create_notes(x: u32, y: u32) -> [Field; 2] {
    [x as Field, y as Field]
}
