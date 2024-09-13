pub comptime fn function_signature(function: FunctionDefinition) -> Quoted {
    let name = function.name();

    let types = function.parameters().map(
        |param: (Quoted, Type)| {
            let typ = param.1;
            f"{typ}".as_ctstring()
        }
    );
    let types = types.join(", ".as_ctstring());

    let signature = f"{name}({types})";
    quote { $signature }
}
