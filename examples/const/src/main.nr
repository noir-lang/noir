use dep::std;

fn main(x: Field) {
    let my_const = 2;
    let foo = my_const + x;

    let array = [0, 1, 2, 3];
    constrain array[foo] == x;
}
