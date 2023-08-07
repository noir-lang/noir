fn bad()  {
    let a: i32 = 100;
    let b: i32 = 200;

    let mut f = || a;

    // this should fail with a type error, since the closures have different environments & types
    f = || a + b; 
}