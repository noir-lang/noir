fn foo() {
    my_function(10, some_value, another_func(20, 30));

    outer_function(
        some_function(), // Original inner function call
        another_function() // Original inner function call
    );
}
