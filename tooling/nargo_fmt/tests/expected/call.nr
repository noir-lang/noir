fn foo() {
    my_function(10, some_value, another_func(20, 30));

    outer_function(
        some_function(), // Original inner function call
        another_function(), // Original inner function call
    );

    outer_function(
        some_function(), // Original inner function call
        another_function(), // Original inner function call
    );

    my_function(
        // Comment
        some_value,
        /* Multiline
        Comment */
        another_func(20, 30),
    );

    my_function(
        some_function(10, "arg1", another_function()),
        another_func(20, some_function(), 30),
    );

    outer_function(
        some_function(),
        another_function(some_function(), some_value),
    );

    assert_eq(x, y);

    assert_eq(x, y, "message");

    assert(x);

    assert(x, "message");

    assert(x == y);

    assert(p4_affine.eq(Gaffine::new(
        6890855772600357754907169075114257697580319025794532037257385534741338397365,
        4338620300185947561074059802482547481416142213883829469920100239455078257889,
    )));
}
