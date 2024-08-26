struct Context {}

#[aztec]
fn foo(_x: Field) {
    bar(1);
}

#[aztec]
fn bar(_x: Field) {}

fn aztec(f: FunctionDefinition) {
    let parameters = f.parameters();
    let parameters = parameters.push_back((quote { _context }, quote { Context }.as_type()));
    f.set_parameters(parameters);

    let body_exprs = f.body();
    let body_exprs = body_exprs.map(
        |body_expr| { 
        let body_expr: Expr = body_expr;
        body_expr.map(|expr: Expr|  {
            let func_call = expr.as_function_call();
            if func_call.is_some() {
              let (name, arguments) = func_call.unwrap();
              let name = quote { $name };
              let arguments = arguments.map(|arg| quote { $arg }).join(quote { , });
              Option::some(quote { name($arguments) }.as_expr().unwrap())
            } else {
              Option::some(expr)
            }
        })
    }
    );
}

fn main() {
    // let context = Context {};
    // foo(context);
}

