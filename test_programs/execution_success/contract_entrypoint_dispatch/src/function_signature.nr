pub comptime fn function_signature(function: FunctionDefinition) -> Quoted {
    let name = function.name();
    let types = function.parameters().map(
        |param: (Quoted, Type)| {
            let typ = param.1;
            quote { $typ }
        }
    );
    if types.len() == 0 {
        let signature = f"{name}()";
        quote { $signature }
    } else if types.len() == 1 {
        let t0 = types[0];
        let signature = f"{name}({t0})";
        quote { $signature }
    } else if types.len() == 2 {
        let t0 = types[0];
        let t1 = types[1];
        let signature = f"{name}({t0}, {t1})";
        quote { $signature }
    } else if types.len() == 3 {
        let t0 = types[0];
        let t1 = types[1];
        let t2 = types[2];
        let signature = f"{name}({t0}, {t1}, {t2})";
        quote { $signature }
    } else if types.len() == 4 {
        let t0 = types[0];
        let t1 = types[1];
        let t2 = types[2];
        let t3 = types[3];
        let signature = f"{name}({t0}, {t1}, {t2}, {t3})";
        quote { $signature }
    } else {
        panic(f"We do not support generating signatures of more than 4 parameters")
    }
}
