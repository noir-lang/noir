global ONE: Field = 50;
global TWO: Field = 100;
global GLOBAL: fmtstr<18, (Field, Field)> = f"i: {ONE}, j: {TWO}";

fn main() {
    let string = "i: {i}, j: {j}";
    println(string);

    println(GLOBAL);
}
