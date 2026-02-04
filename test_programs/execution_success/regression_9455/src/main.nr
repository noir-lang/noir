unconstrained fn main(predicate: bool) {
    let num1 = &mut 0;
    let array = if predicate { [num1] } else { [num1] };
    *array[0] = 9;
    assert_eq(*array[0], 9);
}
