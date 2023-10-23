use dep::std;

trait Default {
    fn default(x: Field, y: Field) -> Self;

    fn method2(x: Field) -> Field {
	x
    }

}

struct Foo {
    bar: Field,
    array: [Field; 2],
}

impl Default for Foo {
    fn default(x: Field,y: Field) -> Self {
        Self { bar: x, array: [x,y] }
    }

    fn method2(x: Field) -> Field {
	x * 3
    }
}

trait F {
    fn f1(self) -> Field;
    fn f2(self) -> Field { 2 }
    fn f3(self) -> Field { 3 }
    fn f4(self) -> Field { 4 }
    fn f5(self) -> Field { 5 }
}

struct Bar {}

impl F for Bar {
    fn f5(self) -> Field { 50 }
    fn f1(self) -> Field { 10 }
    fn f3(self) -> Field { 30 }
}

impl F for &mut Bar {
    fn f1(self) -> Field { 101 }
    fn f5(self) -> Field { 505 }
}

fn main(x: Field) {
    let first = Foo::method2(x);
    assert(first == 3 * x);

    let bar = Bar{};
    assert(bar.f1() == 10);
    assert(bar.f2() == 2);
    assert(bar.f3() == 30);
    assert(bar.f4() == 4);
    assert(bar.f5() == 50);

    let mut bar_mut = Bar{};
    assert((&mut bar_mut).f1() == 101);
    assert((&mut bar_mut).f2() == 2);
    assert((&mut bar_mut).f3() == 3);
    assert((&mut bar_mut).f4() == 4);
    assert((&mut bar_mut).f5() == 505);

    assert(bar_mut.f1() == 10);
    assert(bar_mut.f2() == 2);
    assert(bar_mut.f3() == 30);
    assert(bar_mut.f4() == 4);
    assert(bar_mut.f5() == 50);
}