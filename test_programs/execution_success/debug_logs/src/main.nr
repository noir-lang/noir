use dep::std;

fn main(x: Field, y: pub Field) {
    let string = "i: {i}, j: {j}";
    std::println(string);
    // A `fmtstr` lets you easily perform string interpolation.
    let fmt_str: fmtstr<14, (Field, Field)> = f"i: {x}, j: {y}";
    let fmt_str = string_identity(fmt_str);
    std::println(fmt_str);

    let fmt_str_no_type = f"i: {x}, j: {y}";
    std::println(fmt_str_no_type);

    let fmt_str_generic = string_with_generics(fmt_str_no_type);
    std::println(fmt_str_generic);

    let s = myStruct { y: x, x: y };
    std::println(s);

    std::println(f"randomstring{x}{x}");

    let fmt_str = string_with_partial_generics(f"i: {x}, s: {s}");
    std::println(fmt_str);

    std::println(x);
    std::println([x, y]);

    let foo = fooStruct { my_struct: s, foo: 15 };
    std::println(f"s: {s}, foo: {foo}");

    std::println(f"x: 0, y: 1");

    let s_2 = myStruct { x: 20, y: 30 };
    std::println(f"s1: {s}, s2: {s_2}");

    let bar = fooStruct { my_struct: s_2, foo: 20 };
    std::println(f"foo1: {foo}, foo2: {bar}");

    let struct_string = if x != 5 { f"{foo}" } else { f"{bar}" };
    std::println(struct_string);

    let one_tuple = (1, 2, 3);
    let another_tuple = (4, 5, 6);
    std::println(f"one_tuple: {one_tuple}, another_tuple: {another_tuple}");
    std::println(one_tuple);

    let tuples_nested = (one_tuple, another_tuple);
    std::println(f"tuples_nested: {tuples_nested}");
    std::println(tuples_nested);

    regression_2906();

    let first_array = [1, 2, 3];
    let second_array = [4, 5, 6];
    let arrays_nested = [first_array, second_array];
    std::println(f"first_array: {first_array}, second_array: {second_array}");
    std::println(f"arrays_nested: {arrays_nested}");

    let free_lambda = |x| x + 1;
    let sentinel: u32 = 8888;
    std::println(f"free_lambda: {free_lambda}, sentinel: {sentinel}");
    std::println(free_lambda);

    let one = 1;
    let closured_lambda = |x| x + one;
    std::println(f"closured_lambda: {closured_lambda}, sentinel: {sentinel}");
    std::println(closured_lambda);
}

fn string_identity(string: fmtstr<14, (Field, Field)>) -> fmtstr<14, (Field, Field)> {
    string
}

fn string_with_generics<N, T>(string: fmtstr<N, T>) -> fmtstr<N, T> {
    string
}

fn string_with_partial_generics<N, T>(string: fmtstr<N, (Field, T)>) -> fmtstr<N, (Field, T)> {
    string
}

struct myStruct {
    y: Field,
    x: Field,
}

struct fooStruct {
    my_struct: myStruct,
    foo: Field,
}

fn regression_2906() {
    let array_two_vals = [1, 2];
    dep::std::println(f"array_two_vals: {array_two_vals}");

    let label_two_vals = "12";
    dep::std::println(f"label_two_vals: {label_two_vals}");

    let array_five_vals = [1, 2, 3, 4, 5];
    dep::std::println(f"array_five_vals: {array_five_vals}");

    let label_five_vals = "12345";
    dep::std::println(f"label_five_vals: {label_five_vals}");

    dep::std::println(f"array_five_vals: {array_five_vals}, label_five_vals: {label_five_vals}");
}

