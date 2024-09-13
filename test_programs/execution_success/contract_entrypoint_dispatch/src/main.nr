use field_size::size_in_fields;

mod field_size;
mod from_call_data;
mod function_selector;
mod function_signature;
mod point;

#[aztec_contract]
mod some_contract {
    use crate::point::Point;

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
    let functions = functions.filter(|function: FunctionDefinition| function.has_named_attribute("aztec_public"));

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
                $call
            }
        };
        if_
    }
    );
    let unknown_selector = f"Unknown selector";
    let ifs = ifs.push_back(quote { { assert(false, $unknown_selector); std::mem::zeroed() } });
    let dispatch = ifs.join(quote { else });

    let body = quote {
        pub fn public_entrypoint(selector: Field, calldata: [Field; $max_field_size]) {
            let _index = 0;

            $dispatch
        }
    };

    body
}

fn main() {
    let calldata = [1, 2, 3];
    some_contract::public_entrypoint(1, calldata);
}
