struct Context {
    read_requests: Field,
}

impl Context {
    fn new() -> Context {
        Context { read_requests: 0 }
    }

    fn push_read_request(mut self: Self, request: Field) -> Context {
        self.read_requests += request;
        self
    }
}

struct Note {
    value: Field,
    is_real: bool,
}

fn read_2_notes_oracle(storage_slot: Field) -> (Note, Note) {
  let randomness = dep::std::hash::pedersen_with_separator([storage_slot], 0)[0];
  let note0_is_real =  ((randomness as u32) % 2) as bool;
    let note0 = Note {
        value: randomness + 1,
        is_real: note0_is_real,
    };

    let note1 = Note {
        value: randomness + 2,
        is_real: !note0_is_real,
    };

    (note0, note1)
}

fn is_real_note(note: Note) -> bool {
    note.is_real
}

fn note_hash(note: Note) -> Field {
    note.value
}

fn read_test(mut context: Context, storage_slot: Field) -> (Context, (Note,Note)) {
    let notes = read_2_notes_oracle(storage_slot);
    
    let mut note0_hash = 0;
    if is_real_note(notes.0) {
        note0_hash = note_hash(notes.0);
    }
    let mut note1_hash = 0;
    if is_real_note(notes.1) {
        note1_hash = note_hash(notes.1);
    }

    context = context.push_read_request(note0_hash);
    context = context.push_read_request(note1_hash);

    (context, notes)
}

fn main() {
    let mut context = Context::new();
    let result = read_test(context, 0);
    context = result.0;
    assert(context.read_requests != 0);
    let (note0, note1) = result.1;
    assert(note0.is_real == true);
    assert(note1.is_real == false);
}
