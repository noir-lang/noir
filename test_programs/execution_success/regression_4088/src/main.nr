trait Serialize<let N: u32> {
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

fn check<let N: u32>(serialized_note: [Field; N]) {
    assert(serialized_note[0] == 0);
}

fn oopsie<Note, let N: u32>(note: Note)
where
    Note: Serialize<N>,
{
    let serialized_note = Note::serialize(note);

    check(serialized_note)
}

fn main(mut note: ValueNote) {
    oopsie(note);
}
