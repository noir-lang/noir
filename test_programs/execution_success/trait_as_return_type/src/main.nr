trait SomeTrait {
    fn magic_number(self) -> Field;
}

struct A {}
struct B {}
struct C {
    x: Field,
}

impl SomeTrait for A {
    fn magic_number(self) -> Field {
        2
    }
}

impl SomeTrait for B {
    fn magic_number(self) -> Field {
        4
    }
}

impl SomeTrait for C {
    fn magic_number(self) -> Field {
        self.x
    }
}

fn factory_a() -> impl SomeTrait {
    A {}
}

fn factory_b() -> impl SomeTrait {
    B {}
}

fn factory_c(x: Field) -> impl SomeTrait {
    C { x }
}
// x = 15
fn main(x: u32) {
    let a = factory_a();
    let b = B {};
    let b2 = factory_b();
    assert(a.magic_number() == 2);
    assert(b.magic_number() == 4);
    assert(b2.magic_number() == 4);
    let c = factory_c(10);
    assert(c.magic_number() == 10);
    assert(factory_c(13).magic_number() == 13);
}
