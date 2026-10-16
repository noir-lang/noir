// Items that stdlib doc examples can use without defining them. The example
// checker (`check_stdlib_doc_examples.py`) imports an item from here into any
// example that mentions it without defining it. Show the definition in one
// doc comment, so readers can find it, and say there that other examples use it.

use std::hash::Hasher;

// A minimal hasher for `UHashMap` examples, shown in the `UHashMap` struct docs.
pub struct MyHasher {
    state: Field,
}

impl Hasher for MyHasher {
    fn finish(self) -> Field {
        self.state
    }

    fn write(&mut self, input: Field) {
        self.state = self.state * 31 + input;
    }
}

impl Default for MyHasher {
    fn default() -> Self {
        MyHasher { state: 0 }
    }
}
