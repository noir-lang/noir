use dep::std;

fn main(x: Field) {
    // let a: u8 = x / 0;
    // let a: u8 = 3 / 0;
    let a: Field = 3 / 0;
    std::println(a);
}