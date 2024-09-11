#[aztec_contract]
mod some_contract {
    use super::Point;
    use super::FunctionSelector;
    use super::FromCallData;

    #[aztec(public)]
    pub fn one(x: Field) {
        println(f"Called one with x: {x}");
    }

    #[aztec(public)]
    pub fn two(x: Field, point: Point) {
        println(f"Called two with x: {x}, point: {point}");
    }

    pub fn other(_x: Field) {}

    // TODO: let the `aztec_contract` annotation above generate this method
    pub fn public_entrypoint(selector: Field, calldata: [Field; 2]) {
        if selector == FunctionSelector::from_signature("one(Field)") {
            let (x, _) = FromCallData::from_calldata(calldata);
            one(x);
        } else if selector == FunctionSelector::from_signature("two(Field, Point)") {
            let (x, calldata) = FromCallData::from_calldata(calldata);
            let (point, _) = FromCallData::from_calldata(calldata);
            two(x, point)
        } else {
            panic(f"Unknown selector");
        }
    }
}

struct Point {
    x: Field,
    y: Field,
}

struct FunctionSelector {
}

impl FunctionSelector {
    pub fn from_signature<let N: u32>(s: str<N>) -> Field {
        0
    }
}

/// A trait for types that can be decoded from a calldata field array
trait FromCallData<let N: u32, let M: u32> {
    /// Decodes Self by consuming it from the head of calldata, returning the remaining calldata
    fn from_calldata(calldata: [Field; N]) -> (Self, [Field; M]);
}

// TODO: allow deriving FromCallData for any struct

impl <let N: u32> FromCallData<N, N - 1> for Field {
    fn from_calldata(calldata: [Field; N]) -> (Self, [Field; (N - 1)]) {
        let slice = calldata.as_slice();
        let (value, slice) = slice.pop_front();
        (value, slice.as_array())
    }
}

impl <let N: u32> FromCallData<N, N - 2> for Point {
    fn from_calldata(calldata: [Field; N]) -> (Self, [Field; (N - 2)]) {
        let (x, calldata) = FromCallData::from_calldata(calldata);
        let (y, calldata) = FromCallData::from_calldata(calldata);
        (Self { x, y }, calldata)
    }
}

fn main() {
    let calldata = [1, 2];
    some_contract::public_entrypoint(0, calldata);
}
