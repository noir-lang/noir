fn main() {
    //Regression for Issue #579
    let result1_true = test(true);
    assert(result1_true.array_param[0] == 1);
    let result1_false = test(false);
    assert(result1_false.array_param[0] == 0);
}

struct MyStruct579 {
    array_param: [u32; 2],
}

impl MyStruct579 {
    fn new(array_param: [u32; 2]) -> MyStruct579 {
        MyStruct579 { array_param }
    }
}

fn test(flag: bool) -> MyStruct579 {
    let mut my_struct = MyStruct579::new([0; 2]);

    if flag == true {
        my_struct = MyStruct579::new([1; 2]);
    }
    my_struct
}
