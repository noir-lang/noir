use dep::std::slice;
use dep::std;

unconstrained fn main(x: Field, y: Field) {
    let mut array = [x, y];

    // Dynamic dispatch typing
    let mut test_slice = wrapper(push_accepts_array_returns_slice, array);
    assert(test_slice[2] == 1);

    test_slice = wrapper(push_accepts_slice_returns_slice, array);
    assert(test_slice[2] == 1);


    //Get the slice from a function to correct the type until https://github.com/noir-lang/noir/issues/1931 is fixed
    let mut slice = get_me_a_slice(x, y);


    // No cast
    let another_array = push_accepts_array_returns_array(array);
    assert(another_array[2] == 1);
    
    slice = push_accepts_array_returns_slice(array);
    assert(slice[2] == 1);

    // Cast return value
    slice = push_accepts_array_returns_array(array);
    assert(slice[2] == 1);

    // Cast param
    slice = push_accepts_slice_returns_slice(array);
    assert(slice[2] == 1);
}

unconstrained fn get_me_a_slice(x: Field, y: Field) -> [Field] {
    [x, y]
}

unconstrained fn push_accepts_array_returns_slice<N>(array: [Field; N]) -> [Field] {
    let slice: [Field] = array;
    slice.push_back(1)
}

unconstrained fn push_accepts_array_returns_array(array: [Field; 2]) -> [Field; 3] {
    [array[0], array[1], 1]
}

unconstrained fn push_accepts_slice_returns_slice(slice: [Field]) -> [Field] {
    slice.push_back(1)
}

unconstrained fn wrapper(function: fn ([Field]) -> [Field], param: [Field; 2]) -> [Field] {
    function(param)
}