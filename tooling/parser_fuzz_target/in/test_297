trait Serialize<N> {
    fn serialize(self) -> [Field; N];
}

struct ValueNote {
    value: Field,
}

impl Serialize<1> for ValueNote {
    fn serialize(self) -> [Field; 1] {
        [self.value]
    }
}

fn check<N>(serialized_note: [Field; N]) {
    assert(serialized_note[0] == 0);
}

fn oopsie<Note, N>(note: Note) where Note: Serialize<N> {
    let serialized_note = Note::serialize(note);

    check(serialized_note)
}

fn main(mut note: ValueNote) {
    oopsie(note);
}
