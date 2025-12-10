fn main() {
    let array = [1, 2, 3];
    let one = f"pre array ({array}) post array";
    println(one);

    let two = f"pre one ({one}) post one";
    println(two);
    let three = f"pre two ({two}) ({two}) post two";

    println(three);

    println(f"pre three ({three} {three} {three}) post three");
}
