// TODO(#2568): Currently we only support trait constraints in a few cases.
// There's a bunch of other places where they can pop up:
//      - trait methods (trait Foo<T> where T: ... { )
//      - structs (struct Foo<T> where T: ...)
// import the traits from another module to ensure the where clauses are ok with that
mod the_trait;
use crate::the_trait::Asd;
use crate::the_trait::StaticTrait;

struct Add10 {
    x: Field,
}
struct Add20 {
    x: Field,
}
struct Add30 {
    x: Field,
}
struct AddXY {
    x: Field,
    y: Field,
}

impl Asd for Add10 {
    fn asd(self) -> Field {
        self.x + 10
    }
}
impl Asd for Add20 {
    fn asd(self) -> Field {
        self.x + 20
    }
}
impl Asd for Add30 {
    fn asd(self) -> Field {
        self.x + 30
    }
}

impl Asd for AddXY {
    fn asd(self) -> Field {
        self.x + self.y
    }
}

struct Static100 {}
impl StaticTrait for Static100 {
    // use default implementation for static_function, which returns 100
}

struct Static200 {}
impl StaticTrait for Static200 {
    fn static_function(slf: Self) -> Field {
        let _ = slf;
        200
    }
}

fn assert_asd_eq_100<T>(t: T)
where
    T: crate::the_trait::Asd,
{
    assert(t.asd() == 100);
}

fn add_one_to_static_function<T>(t: T) -> Field
where
    T: StaticTrait,
{
    T::static_function(t) + 1
}

fn main() {
    let x = Add10 { x: 90 };
    let z = Add20 { x: 80 };
    let a = Add30 { x: 70 };
    let xy = AddXY { x: 30, y: 70 };

    assert_asd_eq_100(x);
    assert_asd_eq_100(z);
    assert_asd_eq_100(a);
    assert_asd_eq_100(xy);

    assert(add_one_to_static_function(Static100 {}) == 101);
    assert(add_one_to_static_function(Static200 {}) == 201);
}
