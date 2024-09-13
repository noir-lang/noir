use from_call_data::FromCallData;
use point::Point;

mod size_in_fields;
mod from_call_data;
mod function_selector;
mod function_signature;
mod point;
mod to_call_data;

#[aztec_contract]
mod some_contract {
    use crate::point::Point;

    #[aztec_public]
    pub fn one(x: Field) -> Field {
        println(f"Called one with x: {x}");
        x + 1
    }

    #[aztec_public]
    pub fn two(x: Field, point: Point) -> Point {
        println(f"Called two with x: {x}, point: {point}");
        Point { x: point.x + x, y: point.y + x }
    }

    pub fn other(_x: Field) {}
}

comptime fn aztec_contract(m: Module) -> Quoted {
    let functions = m.functions();
    let functions = functions.filter(|function: FunctionDefinition| function.has_named_attribute("aztec_public"));

    // Compute max field size and max return size
    let mut max_field_size = 0;
    let mut max_return_size = 0;
    for function in functions {
        let mut field_size = 0;
        for param in function.parameters() {
            field_size += size_in_fields::size_in_fields(param.1);
        }
        if field_size > max_field_size {
            max_field_size = field_size;
        }

        let return_size = size_in_fields::size_in_fields(function.return_type());
        if return_size > max_return_size {
            max_return_size = return_size;
        }
    }

    let ifs = functions.map(
        |function: FunctionDefinition| {
        let name = function.name();
        let signature = crate::function_signature::function_signature(function);
        // Turn the Quoted value into into a str<N>
        let signature = std::meta::unquote!(quote { $signature });

        let decodes = function.parameters().map(|param: (Quoted, Type)| {
            let param_name = param.0;
            quote { let ($param_name, _index) = crate::from_call_data::FromCallData::from_calldata(calldata, _index); }
        });
        let decode = decodes.join(quote { });

        let args = function.parameters().map(|param: (Quoted, Type)| param.0);
        let args = args.join(quote { , });
        let call = quote { $name($args) };

        let if_ = quote { 
            if selector == crate::function_selector::FunctionSelector::from_signature($signature) {
                $decode
                let result = $call;
                let _ = crate::to_call_data::ToCallData::to_calldata(result, &mut return_value, 0);
            }
        };
        if_
    }
    );
    let unknown_selector = f"Unknown selector";
    let ifs = ifs.push_back(quote { { assert(false, $unknown_selector); std::mem::zeroed() } });
    let dispatch = ifs.join(quote { else });

    let body = quote {
        pub fn public_entrypoint(selector: Field, calldata: [Field; $max_field_size]) -> [Field; $max_return_size] {
            let _index = 0;
            let mut return_value = [0; $max_return_size];

            $dispatch

            return_value
        }
    };

    body
}

fn main() {
    // Test calling one
    let calldata = [1, 2, 3];
    let result = some_contract::public_entrypoint(0, calldata);
    let (field, _index): (Field, Field) = FromCallData::from_calldata(result, 0);
    assert_eq(field, 2);
    println(field);

    // Test calling two
    let calldata = [1, 2, 3];
    let result = some_contract::public_entrypoint(1, calldata);
    let (point, _index): (Point, Field) = FromCallData::from_calldata(result, 0);
    assert_eq(point.x, 3);
    assert_eq(point.y, 4);
    println(point);
}
