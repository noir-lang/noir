fn main() { 
    let x: [Field] = [3; 4];
    let _ = x.map(|y| y);
}

#[test]
fn test_main() {
    main();
}
