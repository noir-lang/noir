struct Context {
    value: Field,
}

#[inject_context]
fn foo(x: Field) {
    if true {
        // 20 + 1 => 21
        bar(qux(x + 1));
    } else {
        assert(false);
    }
}

#[inject_context]
fn bar(x: Field) {
    let expected = _context.value;
    assert_eq(x, expected);
}

#[inject_context]
fn qux(x: Field) -> Field {
    // 21 * 2 => 42
    x * 2
}

fn inject_context(f: FunctionDefinition) {
    // Add a `_context: Context` parameter to the function
    let parameters = f.parameters();
    let parameters = parameters.push_front((quote { _context }, quote { Context }.as_type()));
    f.set_parameters(parameters);

    // Create a new body where every function call has `_context` added to the list of arguments.
    let body = f.body().map(mapping_function).quoted();
    f.set_body(body);
}

fn mapping_function(expr: Expr) -> Option<Expr> {
    let func_call = expr.as_function_call();
    if func_call.is_some() {
        let (name, arguments) = func_call.unwrap();
        let arguments = arguments.map(|arg: Expr| arg.map(mapping_function));
        let arguments = arguments.push_front(quote { _context }.as_expr().unwrap());
        let arguments = arguments.map(|arg: Expr| arg.quoted()).join(quote { , });
        Option::some(quote { $name($arguments) }.as_expr().unwrap())
    } else {
        Option::none()
    }
}

fn main() {
    let context = Context { value: 42 };
    foo(context, 20);
}

