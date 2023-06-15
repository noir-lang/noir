use dep::std;
// Tests a very simple program.
// 
// The features being tested is basic conditonal on brillig
fn main(x: Field)  {
   assert(33 == bar(x as bool));
}


unconstrained fn bar(x : bool) -> Field {
    let a = foo(x, 4);
    a + 1
}

unconstrained fn foo(x : bool, y: Field) -> Field {
    assert(y==4);
    2*foo2(4)

}

unconstrained fn foo2(y: Field) -> Field {
    y*y
}
