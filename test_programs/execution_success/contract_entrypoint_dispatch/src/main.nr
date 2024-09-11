mod from_call_data;
mod function_selector;
mod point;

#[aztec_contract]
mod some_contract {
    use crate::point::Point;
    use crate::function_selector::FunctionSelector;
    use crate::from_call_data::FromCallData;

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
    pub fn public_entrypoint(selector: Field, calldata: [Field; 3]) {
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

fn main() {
    let calldata = [1, 2, 3];
    some_contract::public_entrypoint(1, calldata);
}
