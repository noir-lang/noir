fn main(x : Field, y : pub Field) {

    let mut slice: [Field] = [0; 2];

    assert(slice[0] == 0);
    assert(slice[0] != 1);

    slice[1] = y;
    assert(x != slice[1]);
}

