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
    fn f2(self) -> Field { 2 };
    fn f3(self) -> Field { 3 };
    fn f4(self) -> Field { 4 };
    fn f5(self) -> Field { 5 };
}

struct Bar {}

impl F for Bar {
    fn f5(self) -> Field { 50 }
    fn f1(self) -> Field { 10 }
    fn f3(self) -> Field { 30 }
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
}