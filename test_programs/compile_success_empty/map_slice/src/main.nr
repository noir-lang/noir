fn foo(x: [Field]) -> [Field] {
    x
}

fn main() { 
    let x0: [Field] = [3333333; 444444];
    let _ = foo(x0);
    let x: [Field; 444444] = [3333333; 444444];
    let y: [Field] = x;
    let z = foo(y);
    let _ = z.map(|w| w);
}

#[test]
fn test_main() {
    main();
}
