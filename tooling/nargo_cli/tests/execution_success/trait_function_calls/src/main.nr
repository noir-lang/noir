// a more thorough test for trait function/method calls from within trait function/method calls

// This tests all combinations of caller+callee pairs. For each of these, the following combination of properties are tried:
//   *) method (has self parameter) vs function (no self parameter)
//   *) default vs overriden vs overriden (no default)

// TODO: function calls via `Self::` are not yet implemented
// Once this is implemented, this test should be updated.

// test order is:
// 1) trait method -> trait method
// 1a) trait default method -> trait default method
// 1b) trait default method -> trait overriden method
// 1c) trait default method -> trait overriden (no default) method
// 1d) trait overriden method -> trait default method
// 1e) trait overriden method -> trait overriden method
// 1f) trait overriden method -> trait overriden (no default) method
// 1g) trait overriden (no default) method -> trait default method
// 1h) trait overriden (no default) method -> trait overriden method
// 1i) trait overriden (no default) method -> trait overriden (no default) method
// 2) trait method -> trait function
// 2a) - subcases are the same as the above
// ...
// 2i)
// 3{a..i}) trait function -> trait method
// 4{a..i}) trait function -> trait function

// 1) trait method -> trait method
// 1a) trait default method -> trait default method
trait Trait1a {
    fn trait_method1(self) -> Field {
        self.trait_method2() * 7892 - self.vl
    };
    fn trait_method2(self) -> Field {
        43278
    }
}
struct Struct1a { vl: Field }
impl Trait1a for Struct1a { }

// 1b) trait default method -> trait overriden method
trait Trait1b {
    fn trait_method1(self) -> Field {
        self.trait_method2() * 2832 - self.vl
    };
    fn trait_method2(self) -> Field {
        9323
    }
}
struct Struct1b { vl: Field }
impl Trait1b for Struct1b {
    fn trait_method2(self) -> Field {
        2394
    }
}

// 1c) trait default method -> trait overriden (no default) method
trait Trait1c {
    fn trait_method1(self) -> Field {
        self.trait_method2() * 7635 - self.vl
    };
    fn trait_method2(self) -> Field;
}
struct Struct1c { vl: Field }
impl Trait1c for Struct1c {
    fn trait_method2(self) -> Field {
        5485
    }
}

// 1d) trait overriden method -> trait default method
trait Trait1d {
    fn trait_method1(self) -> Field {
        self.trait_method2() * 2825 - self.vl
    };
    fn trait_method2(self) -> Field {
        29341
    }
}
struct Struct1d { vl: Field }
impl Trait1d for Struct1d {
    fn trait_method1(self) -> Field {
        self.trait_method2() * 9342 - self.vl
    }
}

// 1e) trait overriden method -> trait overriden method
trait Trait1e {
    fn trait_method1(self) -> Field {
        self.trait_method2() * 85465 - self.vl
    };
    fn trait_method2(self) -> Field {
        2381
    }
}
struct Struct1e { vl: Field }
impl Trait1e for Struct1e {
    fn trait_method1(self) -> Field {
        self.trait_method2() * 47324 - self.vl
    }
    fn trait_method2(self) -> Field {
        58945
    }
}

// 1f) trait overriden method -> trait overriden (no default) method
trait Trait1f {
    fn trait_method1(self) -> Field {
        self.trait_method2() * 43257 - self.vl
    };
    fn trait_method2(self) -> Field;
}
struct Struct1f { vl: Field }
impl Trait1f for Struct1f {
    fn trait_method1(self) -> Field {
        self.trait_method2() * 34875 - self.vl
    }
    fn trait_method2(self) -> Field {
        5748
    }
}

// 1g) trait overriden (no default) method -> trait default method
trait Trait1g {
    fn trait_method1(self) -> Field;
    fn trait_method2(self) -> Field {
        37845
    }
}
struct Struct1g { vl: Field }
impl Trait1g for Struct1g {
    fn trait_method1(self) -> Field {
        self.trait_method2() * 7854 - self.vl
    }
}

// 1h) trait overriden (no default) method -> trait overriden method
trait Trait1h {
    fn trait_method1(self) -> Field;
    fn trait_method2(self) -> Field {
        7823
    }
}
struct Struct1h { vl: Field }
impl Trait1h for Struct1h {
    fn trait_method1(self) -> Field {
        self.trait_method2() * 3482 - self.vl
    }
    fn trait_method2(self) -> Field {
        8542
    }
}

// 1i) trait overriden (no default) method -> trait overriden (no default) method
trait Trait1i {
    fn trait_method1(self) -> Field;
    fn trait_method2(self) -> Field;
}
struct Struct1i { vl: Field }
impl Trait1i for Struct1i {
    fn trait_method1(self) -> Field {
        self.trait_method2() * 23478 - self.vl
    }
    fn trait_method2(self) -> Field {
        98543
    }
}

fn main() {
    let t1a = Struct1a { vl: 1234 };
    assert(t1a.trait_method1() == 341548742);
    let t1b = Struct1b { vl: 4444 };
    assert(t1b.trait_method1() == 6775364);
    let t1c = Struct1c { vl: 3946 };
    assert(t1c.trait_method1() == 41874029);
    let t1d = Struct1d { vl: 9234 };
    assert(t1d.trait_method1() == 274094388);
    let t1e = Struct1e { vl: 5438 };
    assert(t1e.trait_method1() == 2789507742);
    let t1f = Struct1f { vl: 6237 };
    assert(t1f.trait_method1() == 200455263);
    let t1g = Struct1g { vl: 43587 };
    assert(t1g.trait_method1() == 297191043);
    let t1h = Struct1h { vl: 3984 };
    assert(t1h.trait_method1() == 29739260);
    let t1i = Struct1i { vl: 9234 };
    assert(t1i.trait_method1() == 2313583320);
}
