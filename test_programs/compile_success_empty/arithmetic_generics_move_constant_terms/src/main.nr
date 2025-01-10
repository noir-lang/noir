trait FromCallData<let N: u32, let M: u32> {
    fn from_calldata(calldata: [Field; N]) -> (Self, [Field; M]);
}

struct Point {
    x: Field,
    y: Field,
}

impl<let N: u32> FromCallData<N, N - 1> for Field {
    fn from_calldata(calldata: [Field; N]) -> (Self, [Field; N - 1]) {
        let slice = calldata.as_slice();
        let (value, slice) = slice.pop_front();
        (value, slice.as_array())
    }
}

impl<let N: u32> FromCallData<N, N - 2> for Point {
    fn from_calldata(calldata: [Field; N]) -> (Self, [Field; N - 2]) {
        let (x, calldata) = FromCallData::from_calldata(calldata);
        let (y, calldata) = FromCallData::from_calldata(calldata);
        (Self { x, y }, calldata)
    }
}

fn main() {
    let calldata = [1, 2];
    let _: (Point, _) = FromCallData::from_calldata(calldata);
}
