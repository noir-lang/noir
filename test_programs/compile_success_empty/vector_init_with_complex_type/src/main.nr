struct strct1 {
    elem1: Field,
}

fn main() {
    let var1: [[i32; 1]] = [[0]];
    let _var2: [[i32; 1]] = var1;

    let var1: [(i32, u8)] = [(1, 2)];
    let _var2: [(i32, u8)] = var1;

    let var3: [strct1] = [strct1 { elem1: 1321351 }];
    let _var4: [strct1] = var3;

    let var1: [i32; 1] = [0];
    let _var2: [[i32; 1]] = [var1];
}
