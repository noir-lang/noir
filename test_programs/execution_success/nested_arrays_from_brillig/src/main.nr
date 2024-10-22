struct Header {
    params: [Field; 3],
}

struct MyNote {
    plain: Field,
    array: [Field; 2],
    header: Header,
}

fn access_nested(notes: [MyNote; 2]) -> Field {
    notes[0].array[1] + notes[1].array[0] + notes[0].plain + notes[1].header.params[0]
}

unconstrained fn create_inside_brillig(values: [Field; 6]) -> [MyNote; 2] {
    let header = Header { params: [values[0], values[1], values[2]] };
    let note0 = MyNote { array: [values[0], values[1]], plain: values[2], header };
    let note1 = MyNote { array: [values[3], values[4]], plain: values[5], header };
    [note0, note1]
}

fn main(values: [Field; 6]) {
    let notes = unsafe { create_inside_brillig(values) };
    assert(access_nested(notes) == (2 + 4 + 3 + 1));
}

