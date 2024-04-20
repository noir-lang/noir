use dep::std;

trait MyDefault {
    fn my_default(x: field, y: field) -> Self;

    fn method2(x: field) -> field {
        x
    }
}

struct Foo {
    bar: field,
    array: [field; 2],
}

impl MyDefault for Foo {
    fn my_default(x: field,y: field) -> Self {
        Self { bar: x, array: [x,y] }
    }

    fn method2(x: field) -> field {
        x * 3
    }
}

trait F {
    fn f1(self) -> field;
    fn f2(_self: Self) -> field { 2 }
    fn f3(_self: Self) -> field { 3 }
    fn f4(_self: Self) -> field { 4 }
    fn f5(_self: Self) -> field { 5 }
}

struct Bar {}

impl F for Bar {
    fn f5(_self: Self) -> field { 50 }
    fn f1(_self: Self) -> field { 10 }
    fn f3(_self: Self) -> field { 30 }
}
// Impls on mutable references are temporarily disabled
// impl F for &mut Bar {
//     fn f1(self) -> field { 101 }
//     fn f5(self) -> field { 505 }
// }
fn main(x: field) {
    let first = Foo::method2(x);
    assert(first == 3 * x);

    let bar = Bar {};

    assert(bar.f1() == 10, "1");
    assert(bar.f2() == 2, "2");
    assert(bar.f3() == 30, "3");
    assert(bar.f4() == 4, "4");
    assert(bar.f5() == 50, "5");

    let mut bar_mut = Bar {};
    // Impls on mutable references are temporarily disabled
    // assert_eq((&mut bar_mut).f1(), 101);
    // assert((&mut bar_mut).f2() == 2, "7");
    // assert((&mut bar_mut).f3() == 3, "8");
    // assert((&mut bar_mut).f4() == 4, "9");
    // assert((&mut bar_mut).f5() == 505, "10");
    assert(bar_mut.f1() == 10, "10");
    assert(bar_mut.f2() == 2, "12");
    assert(bar_mut.f3() == 30, "13");
    assert(bar_mut.f4() == 4, "14");
    assert(bar_mut.f5() == 50, "15");
}

