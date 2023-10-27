fn main(){
    // This compiles
    let x : Foo<Field> = Foo {
        x : 12
    };

    // TurboFish not allowed
    let x = Foo::<Field> {
        x : 12
    };
}

struct Foo<N> {
    x : N,
}

struct Bar {
    x : FooField,
}