struct Context {
    value: Field,
}

#[aztec]
fn foo(x: Field) {
    bar(x + 1);
}

#[aztec]
fn bar(x: Field) {
    assert_eq(x, _context.value);
}

fn aztec(f: FunctionDefinition) {
    // Add a `_context: Context` parameter to the function
    let parameters = f.parameters();
    let parameters = parameters.push_back((quote { _context }, quote { Context }.as_type()));
    f.set_parameters(parameters);

    // Create a new body where every function call has `_context` added to the list of arguments.
    let body_exprs = f.body();
    let body_exprs = body_exprs.map(
        |body_expr| { 
        let body_expr: Expr = body_expr;
        body_expr.map(|expr: Expr|  {
            let func_call = expr.as_function_call();
            if func_call.is_some() {
              let (name, arguments) = func_call.unwrap();
              let arguments = arguments.push_back(quote { _context }.as_expr().unwrap());
              let arguments = arguments.map(|arg| quote { $arg }).join(quote { , });
              Option::some(quote { $name($arguments) }.as_expr().unwrap())
            } else {
              Option::some(expr)
            }
        })
    }
    );
    let body = body_exprs.map(|expr| quote { $expr }).join(quote { ; });
    f.set_body(body);
}

fn main() {
    let context = Context { value: 42 };
    foo(41, context);
}

