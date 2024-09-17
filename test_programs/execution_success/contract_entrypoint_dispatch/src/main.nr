mod avm;
mod function_selector;
mod function_signature;
mod point;

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

    let ifs = functions.map(
        |function: FunctionDefinition| {
        let name = function.name();
        let signature = crate::function_signature::function_signature(function);
        // Turn the Quoted value into into a str<N>
        let signature = std::meta::unquote!(quote { $signature });

        let decodes = function.parameters().map(|param: (Quoted, Type)| {
            let param_name = param.0;
            quote { let $param_name = crate::avm::avm_read_calldata(); }
        });
        let decode = decodes.join(quote { });

        let args = function.parameters().map(|param: (Quoted, Type)| param.0);
        let args = args.join(quote { , });
        let call = quote { $name($args) };

        let if_ = quote { 
            if selector == crate::function_selector::FunctionSelector::from_signature($signature) {
                $decode
                let return_value = $call;
                crate::avm::avm_return(return_value);
            }
        };
        if_
    }
    );
    let unknown_selector = f"Unknown selector";
    let ifs = ifs.push_back(quote { { assert(false, $unknown_selector); std::mem::zeroed() } });
    let dispatch = ifs.join(quote { else });

    let body = quote {
        pub fn public_entrypoint(selector: Field) {
            $dispatch
        }
    };

    body
}

fn main() {
    // Test calling one
    some_contract::public_entrypoint(0);

    // Test calling two
    some_contract::public_entrypoint(1);
}
