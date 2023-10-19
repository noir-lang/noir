fn foo() {
    my_function(10, some_value, another_func(20, 30));

    outer_function(some_function(), // Original inner function call
        another_function() // Original inner function call
    );

    outer_function(some_function(), // Original inner function call
        another_function() // Original inner function call
    );

    my_function(// Comment
        some_value,
        /* Multiline
        Comment */
        another_func(20, 30));

    my_function(some_function(10, "arg1", another_function()), another_func(20, some_function(), 30));

    outer_function(some_function(), another_function(some_function(), some_value));
}
