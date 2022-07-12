fn main(x: Field) {
    let my_const = 2;
    let array = [0, 1, 2, 3];

    // Error here:
    // let foo = my_const + x;
    // constrain array[foo] == x;
    constrain array[my_const - 1] == 1;

    let my_const2 = 3;
    constrain array[my_const2] == 3;

    // Using a const where a non-const is expected
    main(my_const2);

    constrain x != 0;
}
