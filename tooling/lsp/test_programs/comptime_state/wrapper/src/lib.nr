// A library with no comptime state of its own that exposes a type of a library with state.
pub fn make() -> notelib::LibThing {
    notelib::LibThing {}
}
