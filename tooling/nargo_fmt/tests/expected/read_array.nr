fn read_array(x: [Field; 3]) {
    assert(x[0] == 1);

    let y = [1, 5, 27];


    assert(y[x[0]] == 5);

}
