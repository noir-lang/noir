fn main(mut x: [u32; 5], z: Field) {
    let idx = z + 10;
    
    if z == 20 {
        x[0] = x[4];
    } else {
        // We should not hit out of bounds here as we have a predicate
        // that should not be hit
        if idx as u32 < 3 {
            x[idx] = 10;
        }
    }
    // Error should occur here as an index out of bounds read occurs 
    assert(x[idx] != 0);
}