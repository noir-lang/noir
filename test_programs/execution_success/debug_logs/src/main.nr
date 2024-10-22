fn main(x: Field, y: pub Field) {
    let string = "i: {i}, j: {j}";
    println(string);

    // TODO: fmtstr cannot be printed
    // let fmt_str: fmtstr<14, (Field, Field)> = f"i: {x}, j: {y}";
    // let fmt_fmt_str = f"fmtstr: {fmt_str}, i: {x}";
    // println(fmt_fmt_str);
    // A `fmtstr` lets you easily perform string interpolation.
    let fmt_str: fmtstr<14, (Field, Field)> = f"i: {x}, j: {y}";

    let fmt_str = string_identity(fmt_str);
    println(fmt_str);

    let fmt_str_no_type = f"i: {x}, j: {y}";
    println(fmt_str_no_type);

    let fmt_str_generic = string_with_generics(fmt_str_no_type);
    println(fmt_str_generic);

    let s = myStruct { y: x, x: y };
    println(s);

    println(f"randomstring{x}{x}");

    let fmt_str = string_with_partial_generics(f"i: {x}, s: {s}");
    println(fmt_str);

    println(x);
    println([x, y]);

    let foo = fooStruct { my_struct: s, foo: 15 };
    println(f"s: {s}, foo: {foo}");

    println(f"x: 0, y: 1");

    let s_2 = myStruct { x: 20, y: 30 };
    println(f"s1: {s}, s2: {s_2}");

    let bar = fooStruct { my_struct: s_2, foo: 20 };
    println(f"foo1: {foo}, foo2: {bar}");

    let struct_string = if x != 5 { f"{foo}" } else { f"{bar}" };
    println(struct_string);

    let one_tuple = (1, 2, 3);
    let another_tuple = (4, 5, 6);
    println(f"one_tuple: {one_tuple}, another_tuple: {another_tuple}");
    println(one_tuple);

    let tuples_nested = (one_tuple, another_tuple);
    println(f"tuples_nested: {tuples_nested}");
    println(tuples_nested);

    regression_2903();
    regression_2906();

    let first_array = [1, 2, 3];
    let second_array = [4, 5, 6];
    let arrays_nested = [first_array, second_array];
    println(f"first_array: {first_array}, second_array: {second_array}");
    println(f"arrays_nested: {arrays_nested}");

    let free_lambda = |x| x + 1;
    let sentinel: u32 = 8888;
    println(f"free_lambda: {free_lambda}, sentinel: {sentinel}");
    println(free_lambda);

    let one = 1;
    let closured_lambda = |x| x + one;
    println(f"closured_lambda: {closured_lambda}, sentinel: {sentinel}");
    println(closured_lambda);

    regression_4967();
}

fn string_identity(string: fmtstr<14, (Field, Field)>) -> fmtstr<14, (Field, Field)> {
    string
}

fn string_with_generics<let N: u32, T>(string: fmtstr<N, T>) -> fmtstr<N, T> {
    string
}

fn string_with_partial_generics<let N: u32, T>(
    string: fmtstr<N, (Field, T)>,
) -> fmtstr<N, (Field, T)> {
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

fn regression_2903() {
    let v: [str<1>; 1] = ["1"; 1];
    println(v); // will print [1]
    let a = v[0];
    println(a); // will print `1`
    let bytes = ["aaa", "bbb", "ccc"];
    println(bytes);
}

fn regression_2906() {
    let array_two_vals = [1, 2];
    println(f"array_two_vals: {array_two_vals}");

    let label_two_vals = "12";
    println(f"label_two_vals: {label_two_vals}");

    let array_five_vals = [1, 2, 3, 4, 5];
    println(f"array_five_vals: {array_five_vals}");

    let label_five_vals = "12345";
    println(f"label_five_vals: {label_five_vals}");

    println(
        f"array_five_vals: {array_five_vals}, label_five_vals: {label_five_vals}",
    );
}

fn regression_4967() {
    let sentinel: u32 = 8888;

    let slice_of_tuples: [(i32, u8)] = &[(11, 22), (33, 44)];
    println(f"slice_of_tuples: {slice_of_tuples}, sentinel: {sentinel}");
    println(slice_of_tuples);

    let slice_of_tuples_coerced: [(i32, u8)] = [(11, 22), (33, 44)];
    println(
        f"slice_of_tuples: {slice_of_tuples_coerced}, sentinel: {sentinel}",
    );
    println(slice_of_tuples_coerced);
}
