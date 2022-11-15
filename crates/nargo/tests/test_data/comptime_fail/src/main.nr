fn main(x: Field) {
    let my_const = 2;
    let array = [0, 1, 2, 3];

    // Error here:
    let foo = my_const + x;
    constrain array[foo] == x;

    let my_const2 = 3;
    constrain array[my_const2] == 3;

    // Using a comptime variable where a non-comptime variable is expected should be fine
    main(my_const2);
    constrain x != 0;
}
