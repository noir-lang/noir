comptime mut global COUNTER: Field = 0;

comptime fn get_unique_id() -> Field {
    let id = COUNTER;
    COUNTER += 1;
    id
}

fn id1() -> Field {
    comptime {
        get_unique_id()
    }
}

fn id2() -> Field {
    comptime {
        get_unique_id()
    }
}

fn main() {
    // Order of comptime evaluation between functions isn't guaranteed
    // so we don't know if (id1 == 0 && id2 == 1) or if (id1 == 1 && id2 == 0).
    // we only know they are not equal
    let id1 = id1();
    let id2 = id2();
    assert(id1 != id2);
}
