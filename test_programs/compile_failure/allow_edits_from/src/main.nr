fn main() {
    foo::greet();
}

/// Globals can't modify things since `allow_edits_from` doesn't work with them
comptime global foo: () = {
    let foo = quote[crate::foo].as_module().unwrap();
    let greet = foo.functions()[0];
    greet.set_body(quote { println("hi!"); }.as_expr().unwrap());
};

mod foo {
    // Should have no error here since we've allowed edits from `modify` which is the original
    // attribute which edits this function
    #['allow_edits_from(bar::modify_greet)]
    pub fn greet() {}

    // Expecting an error here since `modify` will modify this `greet2` function but this
    // indirect modification is not allowed here.
    #['allow_edits_from(crate::foo::no_error)]
    pub fn greet2() {}

    // We should have no error if the attribute is directly placed on the function
    // it is modifying
    #[crate::bar::make_unconstrained]
    pub fn no_error() {}
}

mod bar {
    #[modify_greet]
    comptime fn modify_greet(_: FunctionDefinition) {
        helper();
    }

    comptime fn helper() {
        let foo = quote[crate::foo].as_module().unwrap();
        let greet = foo.functions()[0];
        greet.set_body(quote { println("hi!"); }.as_expr().unwrap());
    }

    comptime fn make_unconstrained(f: FunctionDefinition) {
        f.set_unconstrained(true);
    }
}

// comptime fn my_module(m: Module) -> Quoted {
//     m.add_item(quote {
//         fn injected_fn() -> Field {
//             42
//         }
//     });
// 
//     for f in m.functions() {
//         if f.name() == quote { injected_fn } {
//             let _body = f.body(); // fails: "Function already resolved"
//         }
//     }
// 
//     quote {}
// }
// 
// #[my_module]
// mod test_mod {}
// 
// fn main() {}
// 
// fn foo<let N: u32>() -> u32 {
//     N
// }
// 
// fn main() {
//     println(foo::<11 / 2>());
// }
