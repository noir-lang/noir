fn main(x: Field) {
    let my_const = 2;
    let foo = my_const + x;

    let array = [0, 1, 2, 3];
    constrain array[foo] == x;

    let my_const2 = 3;
    constrain array[my_const2] == 3;

    main(my_const2);
}
