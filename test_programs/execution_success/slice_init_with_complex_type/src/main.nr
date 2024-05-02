struct strct1 {
    elem1: Field,
}

fn main() {
    /* Panic when you initialize a slice of arrays with a value of a slice with the exact same type */
    let var1: [[i32; 1]] = [[0]];
    let var2: [[i32; 1]] = var1;

    /* Still panics with other composite types */
    let var1: [(i32, u8)] = [(1, 2)];
    let var2: [(i32, u8)] = var1;

    let var3: [strct1] = [strct1 { elem1: 1321351 }];
    let var4: [strct1] = var3;

    /* No panic if the slice is created in this initialization sentence, even with an existing array */
    let var1: [i32; 1] = [0];
    let var2: [[i32; 1]] = [var1];
}
