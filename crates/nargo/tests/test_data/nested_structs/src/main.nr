struct Parent {
    foo: Field,
    child: Child,
    bar: Field,
}

struct Child {
    child2: Child2,
    foo: u32,
    bar: u32,
}

struct Child2 {
    input_a: u30,
    input_b: u30,
    flag: bool,
}

fn main(y : pub Field, a: u32, b: u32) -> pub Field {
    // TODO: Currently broken for fixed return value problem discussed in the comments below 
    // let (struct_from_tuple, a_bool) = test_struct_in_tuple(true);
    // constrain struct_from_tuple.my_bool == true;
    // constrain a_bool == 10;

    // Always passes as the nested struct is not returned from a function
    let c2 = Child2 { input_a: 0 as u30, input_b: 1 as u30, flag: true };
    let c = Child { child2: c2, foo: a, bar: b };
    let p = Parent { foo: y, child: c, bar: y };
    constrain y == p.foo;

    // Fails unless we specify the return value. Otherwise we get a stack overflow
    let p2 = new_parent(c2, a, b);
    // Passes without a specified return value when I add a constrain stmt on the struct field
    // Trying to return p2.foo alone though will always fail without a specified return value in Prover.toml
    let q2 = p2.foo + y;
    constrain q2 == 3; 
    // Then a valid return
    p2.foo

    // Passes only when the field is constrained 
    // constrain p2.child.child2.input_b == 1 as u30;
    // p2.child.child2.input_b as Field 

    // Fails without change `let q2 = p2.bar + y`
    // p2.bar

    // Always passes as parent_wrapper returns a Field
    // let res = parent_wrapper(a, b);
    // constrain res == a as Field;
    // let q2 = res + y;
    // q2
}

fn parent_wrapper(a: u32, b: u32) -> Field {
    let c2 = Child2 { input_a: 0 as u30, input_b: 1 as u30, flag: true };

    let p2 = new_parent(c2, a, b);

    p2.foo
}

fn new_parent(c2: Child2, a: u32, b: u32) -> Parent {
    let c = Child { child2: c2, foo: a, bar: b };

    let p = Parent { foo: a as Field, child: c, bar: b as Field };

    p
}

struct MyStruct {
    my_bool: bool,
}
fn test_struct_in_tuple(a_bool : bool) -> (MyStruct, Field) {
    let my_struct = MyStruct {
        my_bool: a_bool,
    };
    (my_struct, 10)
}