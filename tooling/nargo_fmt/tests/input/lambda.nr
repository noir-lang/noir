fn foo() {
    let calculate_total =    |  a ,   b, c| {
        
        let average = (a + b + c) / 3;
        let difference = a - b;
        average + difference
        
    };
    let result = calculate_total(10, 5, 15);
    assert(result == 15);
}
