#[foo_impl]
struct Foo<let N: u32> {
    x: Field,
}

fn main() {
    let f: Foo<7> = Foo { x: 0 };
    f.foo();
}

comptime fn foo_impl(t: TypeDefinition) -> Quoted {
    let g = t.generics();

    let g_names = g
        .map(|generic| {
            let name = generic.typ();
            quote [ $name ]
        })
        .join(quote [,]);

    let g_intros = g
        .map(|generic| {
            let name = generic.typ();
            let typ = generic.numeric_type().unwrap();
            quote [ let $name: $typ ]
        })
        .join(quote [,]);

    quote {
        impl<$g_intros> Foo<$g_names> {
            fn foo(self) {
                let _ = self;
            }
        }
    }
}
