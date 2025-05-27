struct Context {
    f: fn() -> bool,
}
fn main(c: bool) -> pub bool {
    let mut ctx = Context { f: bar };
    if c {
        ctx.f = qux;
    }
    foo(&mut ctx)
}

fn foo(ctx: &mut Context) -> bool {
    (ctx.f)()
}

fn bar() -> bool {
    false
}

fn qux() -> bool {
    true
}
