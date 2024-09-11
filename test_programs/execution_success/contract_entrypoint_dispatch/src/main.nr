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
        let index = 0;

        if selector == FunctionSelector::from_signature("one(Field)") {
            let (x, _index) = FromCallData::from_calldata(calldata, index);
            one(x);
        } else if selector == FunctionSelector::from_signature("two(Field, Point)") {
            let (x, index) = FromCallData::from_calldata(calldata, index);
            let (point, _index) = FromCallData::from_calldata(calldata, index);
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
trait FromCallData<let N: u32> {
    /// Decodes Self by reading it from calldata at the given index, also returning the next index to read from
    fn from_calldata(calldata: [Field; N], index: Field) -> (Self, Field);
}

// TODO: allow deriving FromCallData for any struct

impl <let N: u32> FromCallData<N> for Field {
    fn from_calldata(calldata: [Field; N], index: Field) -> (Self, Field) {
        let value = calldata[index];
        (value, index + 1)
    }
}

impl <let N: u32> FromCallData<N> for Point {
    fn from_calldata(calldata: [Field; N], index: Field) -> (Self, Field) {
        let (x, index) = FromCallData::from_calldata(calldata, index);
        let (y, index) = FromCallData::from_calldata(calldata, index);
        (Self { x, y }, index)
    }
}

fn main() {
    let calldata = [1, 2];
    some_contract::public_entrypoint(0, calldata);
}
