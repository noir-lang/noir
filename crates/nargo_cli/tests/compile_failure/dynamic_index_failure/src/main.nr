fn main(mut x: [u32; 5], z: Field) {
    let idx = z + 10;
    
    x[z] = 4;

    // Dynamic index is greater than length of the array
    // assert(x[idx] != 0);

    // TODO(#2133): Provide more accurate call stacks for arrays merged in if statements
    if z == 20 {
        x[0] = x[4];
    } else {
        // TODO: Dynamic predicate still gives index out of bounds error
        if idx as u32 < 3 {
            x[idx] = 10;
        }
        // assert(x[idx] != 0);
        // x[idx] = 10;
        // for i in 0..5 {
        //     x[idx] = x[i];
        // }
    }
    // Error should occur here as the index out of bounds write occurs 
    assert(x[idx] != 0);
}