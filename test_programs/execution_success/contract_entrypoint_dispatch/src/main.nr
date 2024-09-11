use field_size::size_in_fields;

mod field_size;
mod from_call_data;
mod function_selector;
mod point;

#[aztec_contract]
mod some_contract {
    use crate::point::Point;
    use crate::function_selector::FunctionSelector;
    use crate::from_call_data::FromCallData;

    #[aztec_public]
    pub fn one(x: Field) {
        println(f"Called one with x: {x}");
    }

    #[aztec_public]
    pub fn two(x: Field, point: Point) {
        println(f"Called two with x: {x}, point: {point}");
    }

    pub fn other(_x: Field) {}
}

comptime fn aztec_contract(m: Module) -> Quoted {
    let functions = m.functions();

    // Compute max field size
    let mut max_field_size = 0;
    for function in functions {
        let mut field_size = 0;
        for param in function.parameters() {
            field_size += size_in_fields(param.1);
        }
        if field_size > max_field_size {
            max_field_size = field_size;
        }
    }

    quote {
        pub fn public_entrypoint(selector: Field, calldata: [Field; $max_field_size]) {
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
}

fn main() {
    let calldata = [1, 2, 3];
    some_contract::public_entrypoint(1, calldata);
}
